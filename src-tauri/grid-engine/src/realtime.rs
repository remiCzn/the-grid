use crate::dsp::DspNode;
use crate::ringbuffer::EngineChannels;
use grid_core::compiler::ExecutionPlan;
use grid_core::message::{AudioCommand, TelemetryFrame};
use grid_core::model::ModuleKind;

pub const MAX_SCRATCH_BUFFERS: usize = 16;
pub const MAX_BLOCK_SIZE: usize = 1024;
pub const MAX_MODULE_SLOTS: usize = 32;

/// Emplacement DSP préalloué pour un module actif du graphe
#[derive(Debug, Clone)]
struct DspSlot {
    module_id: u32,
    kind: ModuleKind,
    node: DspNode,
    input_buffers: [Option<usize>; 2],
    output_buffers: [Option<usize>; 2],
    is_active: bool,
}

impl DspSlot {
    fn new(sample_rate: f32) -> Self {
        Self {
            module_id: 0,
            kind: ModuleKind::Oscillator,
            node: DspNode::from_kind(ModuleKind::Oscillator, sample_rate, &[]),
            input_buffers: [None; 2],
            output_buffers: [None; 2],
            is_active: false,
        }
    }
}

/// Contexte d'exécution temps réel vivant sur le thread audio.
/// Strictement conforme aux règles d'or temps réel :
/// - Zéro allocation dynamique de mémoire (no malloc).
/// - Zéro verrouillage bloquant (no mutex).
/// - Zéro appel système ou I/O direct.
pub struct RealtimeAudioProcessor {
    pub channels: EngineChannels,
    pub is_playing: bool,
    pub master_gain: f32,
    pub sample_rate: f32,
    pub current_plan: Option<ExecutionPlan>,

    scratch_buffers: [[f32; MAX_BLOCK_SIZE]; MAX_SCRATCH_BUFFERS],
    dsp_slots: Vec<DspSlot>,

    master_left_buf_idx: Option<usize>,
    master_right_buf_idx: Option<usize>,

    frame_counter: u64,
    samples_since_last_telemetry: usize,
    telemetry_interval_samples: usize,
    current_peak_l: f32,
    current_peak_r: f32,
    current_sum_sq_l: f32,
    current_sum_sq_r: f32,
    sample_accum_count: usize,
}

impl RealtimeAudioProcessor {
    pub fn new(channels: EngineChannels, sample_rate: f32) -> Self {
        let telemetry_fps = 60.0;
        let interval = (sample_rate / telemetry_fps).max(64.0) as usize;

        let mut dsp_slots = Vec::with_capacity(MAX_MODULE_SLOTS);
        for _ in 0..MAX_MODULE_SLOTS {
            dsp_slots.push(DspSlot::new(sample_rate));
        }

        Self {
            channels,
            is_playing: false,
            master_gain: 0.8,
            sample_rate,
            current_plan: None,
            scratch_buffers: [[0.0f32; MAX_BLOCK_SIZE]; MAX_SCRATCH_BUFFERS],
            dsp_slots,
            master_left_buf_idx: None,
            master_right_buf_idx: None,
            frame_counter: 0,
            samples_since_last_telemetry: 0,
            telemetry_interval_samples: interval,
            current_peak_l: 0.0,
            current_peak_r: 0.0,
            current_sum_sq_l: 0.0,
            current_sum_sq_r: 0.0,
            sample_accum_count: 0,
        }
    }

    pub fn set_telemetry_interval(&mut self, interval_samples: usize) {
        self.telemetry_interval_samples = interval_samples.max(1);
    }

    fn apply_plan_to_slots(&mut self, plan: &ExecutionPlan) {
        for slot in &mut self.dsp_slots {
            slot.is_active = false;
        }
        self.master_left_buf_idx = None;
        self.master_right_buf_idx = None;

        for (i, step) in plan.steps.iter().enumerate() {
            if i >= self.dsp_slots.len() {
                break;
            }

            let mut in_bufs = [None; 2];
            for (idx, b) in step.input_buffers.iter().enumerate().take(2) {
                in_bufs[idx] = Some((b.0 as usize) % MAX_SCRATCH_BUFFERS);
            }

            let mut out_bufs = [None; 2];
            for (idx, b) in step.output_buffers.iter().enumerate().take(2) {
                out_bufs[idx] = Some((b.0 as usize) % MAX_SCRATCH_BUFFERS);
            }

            if step.kind == ModuleKind::AudioOut {
                self.master_left_buf_idx = in_bufs[0];
                self.master_right_buf_idx = in_bufs[1];
            }

            let slot = &mut self.dsp_slots[i];
            slot.module_id = step.module_id.0;
            slot.kind = step.kind;
            slot.node = DspNode::from_kind(step.kind, self.sample_rate, &step.initial_parameters);
            slot.input_buffers = in_bufs;
            slot.output_buffers = out_bufs;
            slot.is_active = true;
        }
    }

    fn update_parameter(&mut self, module_id: u32, param_id: u32, value: f32) {
        for slot in &mut self.dsp_slots {
            if slot.is_active && slot.module_id == module_id {
                slot.node.set_parameter(param_id, value);
                break;
            }
        }
    }

    #[inline(always)]
    pub fn process_interleaved_buffer(&mut self, output: &mut [f32]) {
        while let Ok(cmd) = self.channels.command_consumer.pop() {
            match cmd {
                AudioCommand::SetPlaying(playing) => {
                    self.is_playing = playing;
                }
                AudioCommand::SetMasterGain(gain) => {
                    self.master_gain = gain.clamp(0.0, 2.0);
                }
                AudioCommand::SetParameter {
                    module_id,
                    param_id,
                    value,
                } => {
                    self.update_parameter(module_id, param_id, value);
                }
                AudioCommand::LoadExecutionPlan(plan) => {
                    self.apply_plan_to_slots(&plan);
                    self.current_plan = Some(plan);
                }
                AudioCommand::Reset => {
                    self.current_peak_l = 0.0;
                    self.current_peak_r = 0.0;
                    self.current_sum_sq_l = 0.0;
                    self.current_sum_sq_r = 0.0;
                    self.sample_accum_count = 0;
                    self.samples_since_last_telemetry = 0;
                    for slot in &mut self.dsp_slots {
                        if slot.is_active {
                            slot.node.reset();
                        }
                    }
                }
            }
        }

        let num_frames = (output.len() / 2).min(MAX_BLOCK_SIZE);

        if !self.is_playing || self.current_plan.is_none() {
            output.fill(0.0);
            for _ in 0..num_frames {
                self.accumulate_sample(0.0, 0.0);
            }
            return;
        }

        // Exécution de chaque slot DSP actif
        for slot in &mut self.dsp_slots {
            if !slot.is_active || slot.kind == ModuleKind::AudioOut {
                continue;
            }

            if let Some(out_idx) = slot.output_buffers[0] {
                let in_0_opt = slot.input_buffers[0];
                let in_1_opt = slot.input_buffers[1];

                let (left, right) = self.scratch_buffers.split_at_mut(out_idx);
                let (out_slice_arr, right_rest) = right.split_at_mut(1);
                let out_slice = &mut out_slice_arr[0][..num_frames];

                let get_in_slice = |idx: usize| -> &[f32] {
                    if idx < out_idx {
                        &left[idx][..num_frames]
                    } else if idx > out_idx {
                        &right_rest[idx - out_idx - 1][..num_frames]
                    } else {
                        &[]
                    }
                };

                match (in_0_opt, in_1_opt) {
                    (Some(i0), Some(i1)) => {
                        let s0 = get_in_slice(i0);
                        let s1 = get_in_slice(i1);
                        let inputs = [s0, s1];
                        let mut outputs = [out_slice];
                        slot.node.process_block(&inputs, &mut outputs);
                    }
                    (Some(i0), None) => {
                        let s0 = get_in_slice(i0);
                        let inputs = [s0];
                        let mut outputs = [out_slice];
                        slot.node.process_block(&inputs, &mut outputs);
                    }
                    (None, Some(i1)) => {
                        let s1 = get_in_slice(i1);
                        let inputs = [s1];
                        let mut outputs = [out_slice];
                        slot.node.process_block(&inputs, &mut outputs);
                    }
                    (None, None) => {
                        let mut outputs = [out_slice];
                        slot.node.process_block(&[], &mut outputs);
                    }
                }
            }
        }

        // Rendu final vers les buffers de sortie entrelacés
        for frame_idx in 0..num_frames {
            let left_in = self
                .master_left_buf_idx
                .map(|idx| self.scratch_buffers[idx][frame_idx])
                .unwrap_or(0.0);

            let right_in = self
                .master_right_buf_idx
                .map(|idx| self.scratch_buffers[idx][frame_idx])
                .unwrap_or(left_in);

            let out_l = left_in * self.master_gain;
            let out_r = right_in * self.master_gain;

            output[frame_idx * 2] = out_l;
            output[frame_idx * 2 + 1] = out_r;

            self.accumulate_sample(out_l, out_r);
        }

        if output.len() > num_frames * 2 {
            output[num_frames * 2..].fill(0.0);
        }
    }

    #[inline(always)]
    fn accumulate_sample(&mut self, left: f32, right: f32) {
        let abs_l = left.abs();
        let abs_r = right.abs();

        if abs_l > self.current_peak_l {
            self.current_peak_l = abs_l;
        }
        if abs_r > self.current_peak_r {
            self.current_peak_r = abs_r;
        }

        self.current_sum_sq_l += left * left;
        self.current_sum_sq_r += right * right;
        self.sample_accum_count += 1;
        self.samples_since_last_telemetry += 1;

        if self.samples_since_last_telemetry >= self.telemetry_interval_samples {
            self.emit_telemetry();
        }
    }

    #[inline(always)]
    fn emit_telemetry(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_add(1);

        let count = self.sample_accum_count.max(1) as f32;
        let rms_l = (self.current_sum_sq_l / count).sqrt();
        let rms_r = (self.current_sum_sq_r / count).sqrt();

        let telemetry = TelemetryFrame {
            frame_index: self.frame_counter,
            peak_left: self.current_peak_l,
            peak_right: self.current_peak_r,
            rms_left: rms_l,
            rms_right: rms_r,
        };

        let _ = self.channels.telemetry_producer.push(telemetry);

        self.current_peak_l = 0.0;
        self.current_peak_r = 0.0;
        self.current_sum_sq_l = 0.0;
        self.current_sum_sq_r = 0.0;
        self.sample_accum_count = 0;
        self.samples_since_last_telemetry = 0;
    }
}

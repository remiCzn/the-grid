use crate::dsp::{AdsrEnvelope, Oscillator, StateVariableFilter, VcaGain, Waveform};
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
    osc: Oscillator,
    vca: VcaGain,
    svf: StateVariableFilter,
    env: AdsrEnvelope,
    is_active: bool,
}

impl DspSlot {
    fn new(sample_rate: f32) -> Self {
        Self {
            module_id: 0,
            kind: ModuleKind::Oscillator,
            osc: Oscillator::new(sample_rate),
            vca: VcaGain::new(),
            svf: StateVariableFilter::new(sample_rate),
            env: AdsrEnvelope::new(sample_rate),
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

        for (i, step) in plan.steps.iter().enumerate() {
            if i >= self.dsp_slots.len() {
                break;
            }
            let slot = &mut self.dsp_slots[i];
            slot.module_id = step.module_id.0;
            slot.kind = step.kind;
            slot.is_active = true;

            match step.kind {
                ModuleKind::Oscillator => {
                    let freq = step.initial_parameters.get(0).copied().unwrap_or(440.0);
                    slot.osc.set_frequency(freq);
                    slot.osc.waveform = Waveform::Sawtooth;
                }
                ModuleKind::Gain => {
                    let gain = step.initial_parameters.get(0).copied().unwrap_or(1.0);
                    slot.vca.set_gain(gain);
                }
                ModuleKind::Filter => {
                    let cutoff = step.initial_parameters.get(0).copied().unwrap_or(1000.0);
                    let res = step.initial_parameters.get(1).copied().unwrap_or(0.707);
                    slot.svf.set_cutoff(cutoff);
                    slot.svf.set_resonance(res);
                }
                ModuleKind::Envelope => {
                    let a = step.initial_parameters.get(0).copied().unwrap_or(10.0);
                    let d = step.initial_parameters.get(1).copied().unwrap_or(100.0);
                    let s = step.initial_parameters.get(2).copied().unwrap_or(0.7);
                    let r = step.initial_parameters.get(3).copied().unwrap_or(200.0);
                    slot.env.set_attack(a);
                    slot.env.set_decay(d);
                    slot.env.set_sustain(s);
                    slot.env.set_release(r);
                }
                ModuleKind::Constant => {}
                ModuleKind::AudioOut => {
                    let gain = step.initial_parameters.get(0).copied().unwrap_or(0.8);
                    slot.vca.set_gain(gain);
                }
            }
        }
    }

    fn update_parameter(&mut self, module_id: u32, param_id: u32, value: f32) {
        for slot in &mut self.dsp_slots {
            if slot.is_active && slot.module_id == module_id {
                match slot.kind {
                    ModuleKind::Oscillator => {
                        if param_id == 0 {
                            slot.osc.set_frequency(value);
                        }
                    }
                    ModuleKind::Gain => {
                        if param_id == 0 {
                            slot.vca.set_gain(value);
                        }
                    }
                    ModuleKind::Filter => {
                        if param_id == 0 {
                            slot.svf.set_cutoff(value);
                        } else if param_id == 1 {
                            slot.svf.set_resonance(value);
                        }
                    }
                    ModuleKind::Envelope => match param_id {
                        0 => slot.env.set_attack(value),
                        1 => slot.env.set_decay(value),
                        2 => slot.env.set_sustain(value),
                        3 => slot.env.set_release(value),
                        _ => {}
                    },
                    ModuleKind::AudioOut => {
                        if param_id == 0 {
                            slot.vca.set_gain(value);
                        }
                    }
                    ModuleKind::Constant => {}
                }
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
                        slot.svf.reset();
                        slot.env.reset();
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

        let plan = self.current_plan.as_ref().unwrap();

        let mut master_left_buf_idx: Option<usize> = None;
        let mut master_right_buf_idx: Option<usize> = None;

        for (step_idx, step) in plan.steps.iter().enumerate() {
            if step_idx >= self.dsp_slots.len() {
                break;
            }
            let slot = &mut self.dsp_slots[step_idx];
            if !slot.is_active {
                continue;
            }

            match slot.kind {
                ModuleKind::Oscillator => {
                    if let Some(out_buf) = step.output_buffers.get(0) {
                        let out_idx = (out_buf.0 as usize) % MAX_SCRATCH_BUFFERS;
                        let out_slice = &mut self.scratch_buffers[out_idx][..num_frames];
                        slot.osc.process_block(out_slice, None);
                    }
                }
                ModuleKind::Envelope => {
                    if let Some(out_buf) = step.output_buffers.get(0) {
                        let out_idx = (out_buf.0 as usize) % MAX_SCRATCH_BUFFERS;
                        let out_slice = &mut self.scratch_buffers[out_idx][..num_frames];
                        slot.env.process_block(out_slice, None);
                    }
                }
                ModuleKind::Filter => {
                    if let (Some(in_buf), Some(out_buf)) =
                        (step.input_buffers.get(0), step.output_buffers.get(0))
                    {
                        let in_idx = (in_buf.0 as usize) % MAX_SCRATCH_BUFFERS;
                        let out_idx = (out_buf.0 as usize) % MAX_SCRATCH_BUFFERS;
                        if in_idx != out_idx {
                            let (in_slice, out_slice) = if in_idx < out_idx {
                                let (left, right) = self.scratch_buffers.split_at_mut(out_idx);
                                (&left[in_idx][..num_frames], &mut right[0][..num_frames])
                            } else {
                                let (left, right) = self.scratch_buffers.split_at_mut(in_idx);
                                (&right[0][..num_frames], &mut left[out_idx][..num_frames])
                            };
                            slot.svf.process_block(in_slice, out_slice);
                        }
                    }
                }
                ModuleKind::Gain => {
                    if let (Some(in_buf), Some(out_buf)) =
                        (step.input_buffers.get(0), step.output_buffers.get(0))
                    {
                        let in_idx = (in_buf.0 as usize) % MAX_SCRATCH_BUFFERS;
                        let out_idx = (out_buf.0 as usize) % MAX_SCRATCH_BUFFERS;
                        if in_idx != out_idx {
                            let (in_slice, out_slice) = if in_idx < out_idx {
                                let (left, right) = self.scratch_buffers.split_at_mut(out_idx);
                                (&left[in_idx][..num_frames], &mut right[0][..num_frames])
                            } else {
                                let (left, right) = self.scratch_buffers.split_at_mut(in_idx);
                                (&right[0][..num_frames], &mut left[out_idx][..num_frames])
                            };
                            slot.vca.process_block(in_slice, out_slice, None);
                        }
                    }
                }
                ModuleKind::AudioOut => {
                    if let Some(left_buf) = step.input_buffers.get(0) {
                        master_left_buf_idx = Some((left_buf.0 as usize) % MAX_SCRATCH_BUFFERS);
                    }
                    if let Some(right_buf) = step.input_buffers.get(1) {
                        master_right_buf_idx = Some((right_buf.0 as usize) % MAX_SCRATCH_BUFFERS);
                    }
                }
                ModuleKind::Constant => {
                    if let Some(out_buf) = step.output_buffers.get(0) {
                        let out_idx = (out_buf.0 as usize) % MAX_SCRATCH_BUFFERS;
                        self.scratch_buffers[out_idx][..num_frames].fill(0.0);
                    }
                }
            }
        }

        for frame_idx in 0..num_frames {
            let left_in = master_left_buf_idx
                .map(|idx| self.scratch_buffers[idx][frame_idx])
                .unwrap_or(0.0);

            let right_in = master_right_buf_idx
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

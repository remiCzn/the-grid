use crate::dsp::DspNode;
use crate::ringbuffer::EngineChannels;
use crate::voice::MonophonicVoiceAllocator;
use grid_core::compiler::ExecutionPlan;
use grid_core::message::{AudioCommand, TelemetryFrame};
use grid_core::model::ModuleKind;

pub const MAX_SCRATCH_BUFFERS: usize = 16;
pub const MAX_BLOCK_SIZE: usize = 1024;
pub const MAX_MODULE_SLOTS: usize = 32;
pub const MAX_SLOT_INPUTS: usize = 4;
pub const MAX_SLOT_OUTPUTS: usize = 8;

/// Emplacement DSP préalloué pour un module actif du graphe
#[derive(Debug, Clone)]
struct DspSlot {
    module_id: u32,
    kind: ModuleKind,
    node: DspNode,
    input_buffers: [Option<usize>; MAX_SLOT_INPUTS],
    output_buffers: [Option<usize>; MAX_SLOT_OUTPUTS],
    is_active: bool,
}

impl DspSlot {
    fn new(sample_rate: f32) -> Self {
        Self {
            module_id: 0,
            kind: ModuleKind::Oscillator,
            node: DspNode::from_kind(ModuleKind::Oscillator, sample_rate, &[]),
            input_buffers: [None; MAX_SLOT_INPUTS],
            output_buffers: [None; MAX_SLOT_OUTPUTS],
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
    pub voice_allocator: MonophonicVoiceAllocator,

    scratch_buffers: [[f32; MAX_BLOCK_SIZE]; MAX_SCRATCH_BUFFERS],
    slot_output_scratch: [[f32; MAX_BLOCK_SIZE]; MAX_SLOT_OUTPUTS],
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
            voice_allocator: MonophonicVoiceAllocator::default(),
            scratch_buffers: [[0.0f32; MAX_BLOCK_SIZE]; MAX_SCRATCH_BUFFERS],
            slot_output_scratch: [[0.0f32; MAX_BLOCK_SIZE]; MAX_SLOT_OUTPUTS],
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

            let mut in_bufs = [None; MAX_SLOT_INPUTS];
            for (idx, b_opt) in step.input_buffers.iter().enumerate().take(MAX_SLOT_INPUTS) {
                in_bufs[idx] = b_opt.map(|b| (b.0 as usize) % MAX_SCRATCH_BUFFERS);
            }

            let mut out_bufs = [None; MAX_SLOT_OUTPUTS];
            for (idx, b_opt) in step
                .output_buffers
                .iter()
                .enumerate()
                .take(MAX_SLOT_OUTPUTS)
            {
                out_bufs[idx] = b_opt.map(|b| (b.0 as usize) % MAX_SCRATCH_BUFFERS);
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

    /// Traite un evenement MIDI recu sur le thread audio.
    ///
    /// Cette fonction est appelee dans la boucle temps reel a chaque evenement
    /// consomme de maniere lock-free et sans allocation dynamique.
    #[inline(always)]
    fn handle_midi_event(&mut self, event: grid_core::midi::MidiEvent) {
        self.voice_allocator.process_event(&event);
    }

    /// Traite un bloc d'echantillons entrelaces.
    ///
    /// Alias de `process_interleaved_buffer` conformement a la specification.
    #[inline(always)]
    pub fn process_block(&mut self, output: &mut [f32]) {
        self.process_interleaved_buffer(output);
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
                    self.voice_allocator.reset();
                    for slot in &mut self.dsp_slots {
                        if slot.is_active {
                            slot.node.reset();
                        }
                    }
                }
            }
        }

        while let Ok(event) = self.channels.midi_consumer.pop() {
            self.handle_midi_event(event);
        }

        let num_frames = (output.len() / 2).min(MAX_BLOCK_SIZE);

        if !self.is_playing || self.current_plan.is_none() {
            output.fill(0.0);
            for _ in 0..num_frames {
                self.accumulate_sample(0.0, 0.0);
            }
            return;
        }

        // Synchronisation de l'allocateur de voix avec les modules MidiIn
        let voice_state = *self.voice_allocator.state();
        for slot in &mut self.dsp_slots {
            if slot.is_active && slot.kind == ModuleKind::MidiIn {
                if let DspNode::MidiIn(ref mut node) = slot.node {
                    node.update_from_voice_state(&voice_state);
                }
            }
        }

        // Execution de chaque slot DSP actif
        for slot in &mut self.dsp_slots {
            if !slot.is_active || slot.kind == ModuleKind::AudioOut {
                continue;
            }

            let in_0 = slot.input_buffers[0].map(|idx| &self.scratch_buffers[idx][..num_frames]);
            let in_1 = slot.input_buffers[1].map(|idx| &self.scratch_buffers[idx][..num_frames]);

            let (out_0, rest) = self.slot_output_scratch.split_at_mut(1);
            let (out_1, rest) = rest.split_at_mut(1);
            let (out_2, rest) = rest.split_at_mut(1);
            let (out_3, rest) = rest.split_at_mut(1);
            let (out_4, _) = rest.split_at_mut(1);

            let mut out_slices = [
                &mut out_0[0][..num_frames],
                &mut out_1[0][..num_frames],
                &mut out_2[0][..num_frames],
                &mut out_3[0][..num_frames],
                &mut out_4[0][..num_frames],
            ];

            match (in_0, in_1) {
                (Some(s0), Some(s1)) => {
                    let inputs = [s0, s1];
                    slot.node.process_block(&inputs, &mut out_slices);
                }
                (Some(s0), None) => {
                    let inputs = [s0];
                    slot.node.process_block(&inputs, &mut out_slices);
                }
                (None, Some(s1)) => {
                    let inputs = [s1];
                    slot.node.process_block(&inputs, &mut out_slices);
                }
                (None, None) => {
                    slot.node.process_block(&[], &mut out_slices);
                }
            }

            for (port_idx, &out_buf_idx_opt) in slot.output_buffers.iter().enumerate() {
                if let Some(buf_idx) = out_buf_idx_opt {
                    if port_idx < out_slices.len() {
                        self.scratch_buffers[buf_idx][..num_frames]
                            .copy_from_slice(out_slices[port_idx]);
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

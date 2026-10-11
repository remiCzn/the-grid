use crate::ringbuffer::EngineChannels;
use grid_core::compiler::ExecutionPlan;
use grid_core::message::{AudioCommand, TelemetryFrame};

/// Contexte d'exécution temps réel vivant sur le thread audio.
/// Respecte strictement les règles d'or temps réel :
/// - Aucune allocation dynamique de mémoire (no malloc).
/// - Aucun verrouillage bloquant (no mutex).
/// - Aucun appel système ou I/O direct.
pub struct RealtimeAudioProcessor {
    pub channels: EngineChannels,
    pub is_playing: bool,
    pub master_gain: f32,
    pub sample_rate: f32,
    pub current_plan: Option<ExecutionPlan>,

    // Télémétrie et décimation
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
        // Fréquence cible de télémétrie par défaut : 60 Hz
        let telemetry_fps = 60.0;
        let interval = (sample_rate / telemetry_fps).max(64.0) as usize;

        Self {
            channels,
            is_playing: false,
            master_gain: 0.8,
            sample_rate,
            current_plan: None,
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

    /// Règle l'intervalle d'échantillons entre deux émissions de télémétrie (décimation)
    pub fn set_telemetry_interval(&mut self, interval_samples: usize) {
        self.telemetry_interval_samples = interval_samples.max(1);
    }

    /// Traitement principal exécuté au sein du callback audio.
    /// `output` est le buffer entrelacé (interleaved stereo : L, R, L, R, ...).
    #[inline(always)]
    pub fn process_interleaved_buffer(&mut self, output: &mut [f32]) {
        // 1. Dépilage non bloquant de toutes les commandes disponibles
        while let Ok(cmd) = self.channels.command_consumer.pop() {
            match cmd {
                AudioCommand::SetPlaying(playing) => {
                    self.is_playing = playing;
                }
                AudioCommand::SetMasterGain(gain) => {
                    self.master_gain = gain.clamp(0.0, 2.0);
                }
                AudioCommand::SetParameter { .. } => {}
                AudioCommand::LoadExecutionPlan(plan) => {
                    self.current_plan = Some(plan);
                }
                AudioCommand::Reset => {
                    self.current_peak_l = 0.0;
                    self.current_peak_r = 0.0;
                    self.current_sum_sq_l = 0.0;
                    self.current_sum_sq_r = 0.0;
                    self.sample_accum_count = 0;
                    self.samples_since_last_telemetry = 0;
                }
            }
        }

        // 2. Traitement des frames audio
        if !self.is_playing {
            output.fill(0.0);
            for _ in 0..(output.len() / 2) {
                self.accumulate_sample(0.0, 0.0);
            }
            return;
        }

        // Mode lecture : calcul frame par frame sans allocation
        for frame in output.chunks_exact_mut(2) {
            let left_sample = 0.0 * self.master_gain;
            let right_sample = 0.0 * self.master_gain;

            frame[0] = left_sample;
            frame[1] = right_sample;

            self.accumulate_sample(left_sample, right_sample);
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

        // Envoi non bloquant : si plein, drop immédiat (drop-on-full)
        let _ = self.channels.telemetry_producer.push(telemetry);

        self.current_peak_l = 0.0;
        self.current_peak_r = 0.0;
        self.current_sum_sq_l = 0.0;
        self.current_sum_sq_r = 0.0;
        self.sample_accum_count = 0;
        self.samples_since_last_telemetry = 0;
    }
}

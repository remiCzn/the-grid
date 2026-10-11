#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Générateur d'enveloppe ADSR linéaire/exponentiel (Zero-Alloc)
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    pub stage: EnvelopeStage,
    pub current_value: f32,
    pub sample_rate: f32,

    pub attack_time_ms: f32,
    pub decay_time_ms: f32,
    pub sustain_level: f32,
    pub release_time_ms: f32,

    last_gate: bool,
}

impl AdsrEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            stage: EnvelopeStage::Idle,
            current_value: 0.0,
            sample_rate: sample_rate.max(1.0),
            attack_time_ms: 10.0,
            decay_time_ms: 100.0,
            sustain_level: 0.7,
            release_time_ms: 200.0,
            last_gate: false,
        }
    }

    pub fn set_attack(&mut self, ms: f32) {
        self.attack_time_ms = ms.clamp(0.5, 10000.0);
    }

    pub fn set_decay(&mut self, ms: f32) {
        self.decay_time_ms = ms.clamp(0.5, 10000.0);
    }

    pub fn set_sustain(&mut self, level: f32) {
        self.sustain_level = level.clamp(0.0, 1.0);
    }

    pub fn set_release(&mut self, ms: f32) {
        self.release_time_ms = ms.clamp(0.5, 10000.0);
    }

    pub fn gate_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
    }

    pub fn gate_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    pub fn reset(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.current_value = 0.0;
        self.last_gate = false;
    }

    /// Calcule un échantillon selon le signal de gate (seuil > 0.5)
    #[inline(always)]
    pub fn process_sample(&mut self, gate_signal: f32) -> f32 {
        let is_gate_high = gate_signal > 0.5;

        // Détection de front montant (Trigger / Gate ON)
        if is_gate_high && !self.last_gate {
            self.gate_on();
        } else if !is_gate_high && self.last_gate {
            self.gate_off();
        }
        self.last_gate = is_gate_high;

        let dt_sec = 1.0 / self.sample_rate;

        match self.stage {
            EnvelopeStage::Idle => {
                self.current_value = 0.0;
            }
            EnvelopeStage::Attack => {
                let attack_sec = (self.attack_time_ms * 0.001).max(0.0001);
                let step = dt_sec / attack_sec;
                self.current_value += step;
                if self.current_value >= 1.0 {
                    self.current_value = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                let decay_sec = (self.decay_time_ms * 0.001).max(0.0001);
                let step = dt_sec / decay_sec;
                self.current_value -= step * (1.0 - self.sustain_level);
                if self.current_value <= self.sustain_level {
                    self.current_value = self.sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                self.current_value = self.sustain_level;
            }
            EnvelopeStage::Release => {
                let release_sec = (self.release_time_ms * 0.001).max(0.0001);
                let step = dt_sec / release_sec;
                self.current_value -= step * self.sustain_level.max(0.001);
                if self.current_value <= 0.0 {
                    self.current_value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }

        self.current_value.clamp(0.0, 1.0)
    }

    /// Traite un bloc d'échantillons
    #[inline(always)]
    pub fn process_block(&mut self, output: &mut [f32], gate_input: Option<&[f32]>) {
        if let Some(gate) = gate_input {
            let len = output.len().min(gate.len());
            for i in 0..len {
                output[i] = self.process_sample(gate[i]);
            }
        } else {
            for sample in output.iter_mut() {
                *sample = self.process_sample(1.0); // Gate On constant par défaut si non câblé
            }
        }
    }
}

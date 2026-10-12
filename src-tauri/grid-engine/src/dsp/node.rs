use crate::dsp::{AdsrEnvelope, Oscillator, StateVariableFilter, VcaGain, Waveform};
use grid_core::model::ModuleKind;

/// Générateur de tension constante (DC / CV statique)
#[derive(Debug, Clone)]
pub struct ConstantNode {
    pub value: f32,
}

impl ConstantNode {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(-1.0, 1.0),
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(-1.0, 1.0);
    }
}

/// Nœud DSP unifié pour l'exécution audio temps réel sans allocation dynamique
#[derive(Debug, Clone)]
pub enum DspNode {
    Oscillator(Oscillator),
    Gain(VcaGain),
    Filter(StateVariableFilter),
    Envelope(AdsrEnvelope),
    Constant(ConstantNode),
    AudioOut,
}

impl DspNode {
    /// Construit le nœud correspondant à partir du ModuleKind et des paramètres initiaux
    pub fn from_kind(kind: ModuleKind, sample_rate: f32, initial_params: &[f32]) -> Self {
        match kind {
            ModuleKind::Oscillator => {
                let mut osc = Oscillator::new(sample_rate);
                if let Some(&freq) = initial_params.first() {
                    osc.set_frequency(freq);
                }
                osc.waveform = Waveform::Sawtooth;
                Self::Oscillator(osc)
            }
            ModuleKind::Gain => {
                let mut vca = VcaGain::new();
                if let Some(&gain) = initial_params.first() {
                    vca.set_gain(gain);
                }
                Self::Gain(vca)
            }
            ModuleKind::Filter => {
                let mut svf = StateVariableFilter::new(sample_rate);
                if let Some(&cutoff) = initial_params.first() {
                    svf.set_cutoff(cutoff);
                }
                if let Some(&res) = initial_params.get(1) {
                    svf.set_resonance(res);
                }
                Self::Filter(svf)
            }
            ModuleKind::Envelope => {
                let mut env = AdsrEnvelope::new(sample_rate);
                if let Some(&a) = initial_params.first() {
                    env.set_attack(a);
                }
                if let Some(&d) = initial_params.get(1) {
                    env.set_decay(d);
                }
                if let Some(&s) = initial_params.get(2) {
                    env.set_sustain(s);
                }
                if let Some(&r) = initial_params.get(3) {
                    env.set_release(r);
                }
                Self::Envelope(env)
            }
            ModuleKind::Constant => {
                let val = initial_params.first().copied().unwrap_or(0.0);
                Self::Constant(ConstantNode::new(val))
            }
            ModuleKind::AudioOut => Self::AudioOut,
        }
    }

    /// Exécute le calcul DSP sur un bloc d'échantillons
    #[inline(always)]
    pub fn process_block(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {
        match self {
            Self::Oscillator(osc) => {
                if let Some(out_buf) = outputs.first_mut() {
                    let cv = inputs.first().copied();
                    osc.process_block(out_buf, cv);
                }
            }
            Self::Gain(vca) => {
                if let (Some(in_buf), Some(out_buf)) = (inputs.first(), outputs.first_mut()) {
                    let cv = inputs.get(1).copied();
                    vca.process_block(in_buf, out_buf, cv);
                }
            }
            Self::Filter(svf) => {
                if let (Some(in_buf), Some(out_buf)) = (inputs.first(), outputs.first_mut()) {
                    svf.process_block(in_buf, out_buf);
                }
            }
            Self::Envelope(env) => {
                if let Some(out_buf) = outputs.first_mut() {
                    let gate = inputs.first().copied();
                    env.process_block(out_buf, gate);
                }
            }
            Self::Constant(c) => {
                if let Some(out_buf) = outputs.first_mut() {
                    out_buf.fill(c.value);
                }
            }
            Self::AudioOut => {}
        }
    }

    /// Met à jour la valeur d'un paramètre spécifique selon son identifiant
    #[inline(always)]
    pub fn set_parameter(&mut self, param_id: u32, value: f32) {
        match self {
            Self::Oscillator(osc) => {
                if param_id == 0 {
                    osc.set_frequency(value);
                }
            }
            Self::Gain(vca) => {
                if param_id == 0 {
                    vca.set_gain(value);
                }
            }
            Self::Filter(svf) => {
                if param_id == 0 {
                    svf.set_cutoff(value);
                } else if param_id == 1 {
                    svf.set_resonance(value);
                }
            }
            Self::Envelope(env) => match param_id {
                0 => env.set_attack(value),
                1 => env.set_decay(value),
                2 => env.set_sustain(value),
                3 => env.set_release(value),
                _ => {}
            },
            Self::Constant(c) => {
                if param_id == 0 {
                    c.set_value(value);
                }
            }
            Self::AudioOut => {}
        }
    }

    /// Réinitialise l'état interne (phases, filtres, étages d'enveloppe)
    #[inline(always)]
    pub fn reset(&mut self) {
        match self {
            Self::Oscillator(osc) => {
                osc.phase = 0.0;
            }
            Self::Gain(vca) => {
                vca.current_gain = vca.target_gain;
            }
            Self::Filter(svf) => {
                svf.reset();
            }
            Self::Envelope(env) => {
                env.reset();
            }
            Self::Constant(_) => {}
            Self::AudioOut => {}
        }
    }
}

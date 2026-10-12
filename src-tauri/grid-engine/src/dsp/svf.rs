use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

/// Filtre à état variable (SVF Topology-Preserving Transform) 2 pôles résonant (Zero-Alloc)
#[derive(Debug, Clone)]
pub struct StateVariableFilter {
    pub cutoff: f32,
    pub resonance: f32,
    pub sample_rate: f32,
    pub mode: FilterMode,

    // États internes du filtre (intégrateurs)
    ic1eq: f32,
    ic2eq: f32,
}

impl StateVariableFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            cutoff: 1000.0,
            resonance: 0.707, // Butterworth Q
            sample_rate: sample_rate.max(1.0),
            mode: FilterMode::LowPass,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.clamp(10.0, self.sample_rate * 0.49);
    }

    pub fn set_resonance(&mut self, res: f32) {
        // Q entre 0.1 et 20.0
        self.resonance = res.clamp(0.1, 20.0);
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Calcule un échantillon selon la méthode TPT (Bilinear Transform)
    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Fréquence pré-warped
        let g = (PI * self.cutoff / self.sample_rate).tan();
        let k = 1.0 / self.resonance;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        match self.mode {
            FilterMode::LowPass => v2,
            FilterMode::BandPass => v1,
            FilterMode::HighPass => input - k * v1 - v2,
            FilterMode::Notch => input - k * v1,
        }
    }

    /// Traite un bloc d'échantillons
    #[inline(always)]
    pub fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len().min(output.len());
        for i in 0..len {
            output[i] = self.process_sample(input[i]);
        }
    }
}

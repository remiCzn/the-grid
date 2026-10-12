/// Amplificateur contrôlé en tension (VCA) avec lissage de gain linéaire (Zero-Alloc)
#[derive(Debug, Clone)]
pub struct VcaGain {
    pub target_gain: f32,
    pub current_gain: f32,
    pub smoothing_factor: f32,
}

impl VcaGain {
    pub fn new() -> Self {
        Self {
            target_gain: 1.0,
            current_gain: 1.0,
            // Coefficient de filtre passe-bas 1 pôle pour lisser le gain (tau ~5ms)
            smoothing_factor: 0.05,
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.target_gain = gain.clamp(0.0, 2.0);
    }

    /// Calcule un échantillon en combinant gain statique et CV externe
    #[inline(always)]
    pub fn process_sample(&mut self, input: f32, cv: f32) -> f32 {
        // Lissage du gain
        self.current_gain += (self.target_gain - self.current_gain) * self.smoothing_factor;

        let total_gain = (self.current_gain * (1.0 + cv)).max(0.0);
        input * total_gain
    }

    /// Traite un bloc d'échantillons
    #[inline(always)]
    pub fn process_block(&mut self, input: &[f32], output: &mut [f32], cv_input: Option<&[f32]>) {
        let len = input.len().min(output.len());
        if let Some(cv) = cv_input {
            for i in 0..len {
                let cv_val = if i < cv.len() { cv[i] } else { 0.0 };
                output[i] = self.process_sample(input[i], cv_val);
            }
        } else {
            for i in 0..len {
                output[i] = self.process_sample(input[i], 0.0);
            }
        }
    }
}

impl Default for VcaGain {
    fn default() -> Self {
        Self::new()
    }
}


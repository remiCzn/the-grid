use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Sawtooth,
    Square,
    Triangle,
}

/// Oscillateur à bande limitée (Zero-Alloc) avec anti-aliasing PolyBLEP
#[derive(Debug, Clone)]
pub struct Oscillator {
    pub phase: f32,
    pub frequency: f32,
    pub sample_rate: f32,
    pub waveform: Waveform,
    pub pulse_width: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            frequency: 440.0,
            sample_rate: sample_rate.max(1.0),
            waveform: Waveform::Sawtooth,
            pulse_width: 0.5,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        // Bornage pour éviter les repliements spectraux ou fréquences négatives
        self.frequency = freq.clamp(0.1, self.sample_rate * 0.49);
    }

    /// Résidu polynomial PolyBLEP pour adoucir les discontinuités d'échelon
    #[inline(always)]
    fn poly_blep(t: f32, dt: f32) -> f32 {
        if dt <= 0.0 {
            return 0.0;
        }
        if t < dt {
            let t = t / dt;
            // 2 * t - t^2 - 1.0
            t + t - t * t - 1.0
        } else if t > 1.0 - dt {
            let t = (t - 1.0) / dt;
            // t^2 + 2 * t + 1.0
            t * t + t + t + 1.0
        } else {
            0.0
        }
    }

    /// Calcule un échantillon et avance la phase
    #[inline(always)]
    pub fn process_sample(&mut self, pitch_cv: f32) -> f32 {
        // Modulation de pitch par CV (norme 1V/octave : f = f0 * 2^(CV))
        let effective_freq = if pitch_cv != 0.0 {
            (self.frequency * 2.0f32.powf(pitch_cv)).clamp(0.1, self.sample_rate * 0.49)
        } else {
            self.frequency
        };

        let dt = effective_freq / self.sample_rate;
        let t = self.phase;

        let sample = match self.waveform {
            Waveform::Sine => (2.0 * PI * t).sin(),
            Waveform::Sawtooth => {
                let naive = 2.0 * t - 1.0;
                naive - Self::poly_blep(t, dt)
            }
            Waveform::Square => {
                let pw = self.pulse_width.clamp(0.01, 0.99);
                let naive = if t < pw { 1.0 } else { -1.0 };
                let t_shift = (t + 1.0 - pw).fract();
                naive + Self::poly_blep(t, dt) - Self::poly_blep(t_shift, dt)
            }
            Waveform::Triangle => {
                // Intégration d'onde carrée ou calcul triangle
                let mut tri = if t < 0.5 {
                    4.0 * t - 1.0
                } else {
                    3.0 - 4.0 * t
                };
                // Légère atténuation des pointes si nécessaire
                tri = tri.clamp(-1.0, 1.0);
                tri
            }
        };

        // Incrémentation et repliement de la phase [0.0, 1.0[
        self.phase = (self.phase + dt).fract();

        sample
    }

    /// Traite un bloc complet d'échantillons
    #[inline(always)]
    pub fn process_block(&mut self, output: &mut [f32], pitch_cv: Option<&[f32]>) {
        if let Some(cv) = pitch_cv {
            let len = output.len().min(cv.len());
            for i in 0..len {
                output[i] = self.process_sample(cv[i]);
            }
        } else {
            for sample in output.iter_mut() {
                *sample = self.process_sample(0.0);
            }
        }
    }
}

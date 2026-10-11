pub mod envelope;
pub mod oscillator;
pub mod svf;
pub mod vca;

pub use envelope::{AdsrEnvelope, EnvelopeStage};
pub use oscillator::{Oscillator, Waveform};
pub use svf::{FilterMode, StateVariableFilter};
pub use vca::VcaGain;

pub type Sample = f32;

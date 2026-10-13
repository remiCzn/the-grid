pub mod envelope;
pub mod midi_in;
pub mod node;
pub mod oscillator;
pub mod svf;
pub mod vca;

pub use envelope::{AdsrEnvelope, EnvelopeStage};
pub use midi_in::MidiInNode;
pub use node::{ConstantNode, DspNode};
pub use oscillator::{Oscillator, Waveform};
pub use svf::{FilterMode, StateVariableFilter};
pub use vca::VcaGain;

pub type Sample = f32;

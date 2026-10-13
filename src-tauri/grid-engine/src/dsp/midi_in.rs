//! Module DSP MidiIn generant les signaux de controle a partir de l'etat de voix.

use crate::voice::VoiceState;

/// Noeud DSP generant 5 signaux de controle pour le graphe modulaire :
/// - Sortie 0 : Pitch CV (1 V/octave, C4 = 0.0V)
/// - Sortie 1 : Gate (0.0 ou 1.0)
/// - Sortie 2 : Velocite (0.0 a 1.0)
/// - Sortie 3 : Molette de modulation CC 1 (0.0 a 1.0)
/// - Sortie 4 : Pitch Bend (-1.0 a +1.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiInNode {
    pub voice_state: VoiceState,
}

impl Default for MidiInNode {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiInNode {
    pub const fn new() -> Self {
        Self {
            voice_state: VoiceState {
                pitch_cv: 0.0,
                gate: 0.0,
                velocity: 0.0,
                mod_wheel: 0.0,
                pitch_bend: 0.0,
                is_gate_triggered: false,
            },
        }
    }

    /// Met a jour l'etat interne a partir de l'allocateur de voix.
    #[inline(always)]
    pub fn update_from_voice_state(&mut self, state: &VoiceState) {
        self.voice_state = *state;
    }

    /// Remplit les tranches de buffers de sortie avec les signaux de controle courants.
    #[inline(always)]
    pub fn process_block(&mut self, outputs: &mut [&mut [f32]]) {
        if let Some(buf) = outputs.first_mut() {
            buf.fill(self.voice_state.pitch_cv);
        }
        if let Some(buf) = outputs.get_mut(1) {
            buf.fill(self.voice_state.gate);
        }
        if let Some(buf) = outputs.get_mut(2) {
            buf.fill(self.voice_state.velocity);
        }
        if let Some(buf) = outputs.get_mut(3) {
            buf.fill(self.voice_state.mod_wheel);
        }
        if let Some(buf) = outputs.get_mut(4) {
            buf.fill(self.voice_state.pitch_bend);
        }
    }

    /// Reinitialise l'etat interne de la voix.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.voice_state = VoiceState::default();
    }
}

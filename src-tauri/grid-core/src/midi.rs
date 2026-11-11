//! Module de gestion des evenements et du protocole MIDI 1.0.
//!
//! Ce module definit les types d'evenements compacts et fournit un parseur
//! d'octets MIDI bruts sans allocation dynamique.

use serde::{Deserialize, Serialize};

/// Evenement MIDI compact compatible temps reel.
///
/// Sa taille memoire est inferieure ou egale a 8 octets, et il implemente `Copy`
/// et `Send` pour permettre son transit efficace a travers des ring buffers SPSC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiEvent {
    /// Note On : declenchement d'une note avec velocite.
    NoteOn {
        /// Canal MIDI (0..15).
        channel: u8,
        /// Numero de note MIDI (0..127).
        note: u8,
        /// Velocite (1..127). Une velocite de 0 est convertie en NoteOff par le parseur.
        velocity: u8,
    },
    /// Note Off : extinction d'une note.
    NoteOff {
        /// Canal MIDI (0..15).
        channel: u8,
        /// Numero de note MIDI (0..127).
        note: u8,
        /// Velocite de relachement (0..127).
        velocity: u8,
    },
    /// Control Change : changement de controleur continu.
    ControlChange {
        /// Canal MIDI (0..15).
        channel: u8,
        /// Numero de controleur (0..127).
        controller: u8,
        /// Valeur du controleur (0..127).
        value: u8,
    },
    /// Pitch Bend : modulation de hauteur 14-bit centree a 0.
    PitchBend {
        /// Canal MIDI (0..15).
        channel: u8,
        /// Valeur bipolaire centree sur 0, dans la plage [-8192, 8191].
        value: i16,
    },
    /// All Notes Off : coupure de toutes les notes actives sur le canal donne.
    AllNotesOff {
        /// Canal MIDI (0..15).
        channel: u8,
    },
}

impl MidiEvent {
    /// Cree un evenement Note On.
    pub const fn note_on(channel: u8, note: u8, velocity: u8) -> Self {
        Self::NoteOn {
            channel,
            note,
            velocity,
        }
    }

    /// Cree un evenement Note Off.
    pub const fn note_off(channel: u8, note: u8, velocity: u8) -> Self {
        Self::NoteOff {
            channel,
            note,
            velocity,
        }
    }

    /// Cree un evenement Control Change.
    pub const fn control_change(channel: u8, controller: u8, value: u8) -> Self {
        Self::ControlChange {
            channel,
            controller,
            value,
        }
    }

    /// Cree un evenement Pitch Bend.
    pub const fn pitch_bend(channel: u8, value: i16) -> Self {
        Self::PitchBend { channel, value }
    }

    /// Cree un evenement All Notes Off.
    pub const fn all_notes_off(channel: u8) -> Self {
        Self::AllNotesOff { channel }
    }

    /// Renvoie le canal MIDI associe a cet evenement (0..15).
    pub const fn channel(&self) -> u8 {
        match self {
            Self::NoteOn { channel, .. } => *channel,
            Self::NoteOff { channel, .. } => *channel,
            Self::ControlChange { channel, .. } => *channel,
            Self::PitchBend { channel, .. } => *channel,
            Self::AllNotesOff { channel } => *channel,
        }
    }
}

/// Parse une tranche d'octets MIDI standard (MIDI 1.0) en un `MidiEvent`.
///
/// Renvoie `Some(MidiEvent)` si le message est valide et reconnu, ou `None` sinon.
/// Les tranches de longueur differente de 3 octets sont rejetees.
/// Les messages NoteOn avec une velocite egale a 0 sont traduits en `NoteOff`.
pub fn parse_midi_message(bytes: &[u8]) -> Option<MidiEvent> {
    if bytes.len() != 3 {
        return None;
    }

    let status = bytes[0];
    // Les octets de statut ont leur bit de poids fort a 1 (>= 0x80)
    // et les messages systeme commencent a 0xF0
    if !(0x80..0xF0).contains(&status) {
        return None;
    }

    let msg_type = status & 0xF0;
    let channel = status & 0x0F;

    let data1 = bytes[1];
    let data2 = bytes[2];

    // Dans la norme MIDI 1.0, les octets de donnees doivent avoir le bit 7 a 0
    if (data1 & 0x80) != 0 || (data2 & 0x80) != 0 {
        return None;
    }

    match msg_type {
        0x90 => {
            if data2 == 0 {
                Some(MidiEvent::NoteOff {
                    channel,
                    note: data1,
                    velocity: 0,
                })
            } else {
                Some(MidiEvent::NoteOn {
                    channel,
                    note: data1,
                    velocity: data2,
                })
            }
        }
        0x80 => Some(MidiEvent::NoteOff {
            channel,
            note: data1,
            velocity: data2,
        }),
        0xB0 => {
            if data1 == 120 || data1 == 123 {
                Some(MidiEvent::AllNotesOff { channel })
            } else {
                Some(MidiEvent::ControlChange {
                    channel,
                    controller: data1,
                    value: data2,
                })
            }
        }
        0xE0 => {
            // Pitch Bend 14 bits : LSB dans data1, MSB dans data2
            let raw_14bit = ((data2 as u16) << 7) | (data1 as u16);
            let bipolar_val = (raw_14bit as i32) - 8192;
            Some(MidiEvent::PitchBend {
                channel,
                value: bipolar_val as i16,
            })
        }
        _ => None,
    }
}

impl TryFrom<&[u8]> for MidiEvent {
    type Error = ();

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        parse_midi_message(bytes).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_slice() {
        let bytes = [0x90, 60, 100];
        let event = MidiEvent::try_from(&bytes[..]);
        assert_eq!(
            event,
            Ok(MidiEvent::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100
            })
        );

        let invalid = [0x90, 60];
        assert_eq!(MidiEvent::try_from(&invalid[..]), Err(()));

        let trailing_garbage = [0x90, 60, 100, 0x00];
        assert_eq!(MidiEvent::try_from(&trailing_garbage[..]), Err(()));
    }
}


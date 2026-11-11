//! Pont de communication MIDI lock-free (SPSC).
//!
//! Ce module definit le canal de communication unidirectionnel permettant
//! a l'hote (UI ou peripherique MIDI) d'envoyer des evenements MIDI au thread
//! audio temps reel sans allocation dynamique ni blocage par mutex.

use grid_core::midi::MidiEvent;
use rtrb::{Consumer, Producer, PushError, RingBuffer};

/// Capacite par defaut du ring buffer MIDI (1024 evenements).
pub const MIDI_QUEUE_CAPACITY: usize = 1024;

/// Producteur MIDI lock-free destine a l'hote ou au gestionnaire d'evenements MIDI.
pub struct MidiProducer {
    inner: Producer<MidiEvent>,
}

impl MidiProducer {
    /// Cree une nouvelle instance de `MidiProducer` enveloppant un producteur rtrb.
    pub fn new(inner: Producer<MidiEvent>) -> Self {
        Self { inner }
    }

    /// Tente d'envoyer un evenement MIDI dans la file lock-free.
    ///
    /// Retourne `Ok(())` si l'evenement a ete depose dans le buffer,
    /// ou `Err(PushError::Full(event))` si le buffer est sature.
    /// Ne bloque jamais et n'alloue aucune memoire sur le tas.
    pub fn try_push(&mut self, event: MidiEvent) -> Result<(), PushError<MidiEvent>> {
        self.inner.push(event)
    }

    /// Renvoie le nombre d'emplacements disponibles dans le ring buffer.
    pub fn slots(&self) -> usize {
        self.inner.slots()
    }

    /// Indique si le ring buffer est actuellement plein.
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }
}


/// Consommateur MIDI lock-free destine au thread audio temps reel.
pub type MidiConsumer = Consumer<MidiEvent>;

/// Cree un pont MIDI SPSC avec la capacite specifiee.
pub fn create_midi_bridge(capacity: usize) -> (MidiProducer, MidiConsumer) {
    let (producer, consumer) = RingBuffer::new(capacity);
    (MidiProducer::new(producer), consumer)
}

/// Cree un pont MIDI SPSC avec la capacite standard de 1024 evenements.
pub fn create_default_midi_bridge() -> (MidiProducer, MidiConsumer) {
    create_midi_bridge(MIDI_QUEUE_CAPACITY)
}

use crate::midi_bridge::{create_default_midi_bridge, MidiConsumer, MidiProducer};
use grid_core::message::{AudioCommand, TelemetryFrame};
use rtrb::{Consumer, Producer, RingBuffer};

/// Capacite par defaut du ring buffer de commandes
pub const COMMAND_QUEUE_CAPACITY: usize = 256;
/// Capacite par defaut du ring buffer de telemetrie
pub const TELEMETRY_QUEUE_CAPACITY: usize = 64;

/// Canaux detenus par le Host (UI / backend non-temps-reel)
pub struct HostChannels {
    pub command_producer: Producer<AudioCommand>,
    pub telemetry_consumer: Consumer<TelemetryFrame>,
    pub midi_producer: MidiProducer,
}

/// Canaux detenus par le Realtime Audio Thread
pub struct EngineChannels {
    pub command_consumer: Consumer<AudioCommand>,
    pub telemetry_producer: Producer<TelemetryFrame>,
    pub midi_consumer: MidiConsumer,
}

/// Cree la paire de canaux SPSC bidirectionnels incluant les canaux MIDI
pub fn create_channels() -> (HostChannels, EngineChannels) {
    let (cmd_p, cmd_c) = RingBuffer::new(COMMAND_QUEUE_CAPACITY);
    let (tel_p, tel_c) = RingBuffer::new(TELEMETRY_QUEUE_CAPACITY);
    let (midi_p, midi_c) = create_default_midi_bridge();

    (
        HostChannels {
            command_producer: cmd_p,
            telemetry_consumer: tel_c,
            midi_producer: midi_p,
        },
        EngineChannels {
            command_consumer: cmd_c,
            telemetry_producer: tel_p,
            midi_consumer: midi_c,
        },
    )
}


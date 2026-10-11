use grid_core::message::{AudioCommand, TelemetryFrame};
use rtrb::{Consumer, Producer, RingBuffer};

/// Capacité par défaut du ring buffer de commandes
pub const COMMAND_QUEUE_CAPACITY: usize = 256;
/// Capacité par défaut du ring buffer de télémétrie
pub const TELEMETRY_QUEUE_CAPACITY: usize = 64;

/// Canaux détenus par le Host (UI / backend non-temps-réel)
pub struct HostChannels {
    pub command_producer: Producer<AudioCommand>,
    pub telemetry_consumer: Consumer<TelemetryFrame>,
}

/// Canaux détenus par le Realtime Audio Thread
pub struct EngineChannels {
    pub command_consumer: Consumer<AudioCommand>,
    pub telemetry_producer: Producer<TelemetryFrame>,
}

/// Crée la paire de canaux SPSC bidirectionnels
pub fn create_channels() -> (HostChannels, EngineChannels) {
    let (cmd_p, cmd_c) = RingBuffer::new(COMMAND_QUEUE_CAPACITY);
    let (tel_p, tel_c) = RingBuffer::new(TELEMETRY_QUEUE_CAPACITY);

    (
        HostChannels {
            command_producer: cmd_p,
            telemetry_consumer: tel_c,
        },
        EngineChannels {
            command_consumer: cmd_c,
            telemetry_producer: tel_p,
        },
    )
}

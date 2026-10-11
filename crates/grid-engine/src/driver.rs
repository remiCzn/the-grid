use crate::realtime::RealtimeAudioProcessor;
use crate::ringbuffer::{create_channels, HostChannels};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("Aucun périphérique de sortie audio trouvé")]
    NoOutputDevice,
    #[error("Configuration du stream audio par défaut introuvable: {0}")]
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),
    #[error("Erreur de création du stream audio: {0}")]
    BuildStreamError(#[from] cpal::BuildStreamError),
    #[error("Erreur lors de la lecture du stream: {0}")]
    PlayStreamError(#[from] cpal::PlayStreamError),
}

pub struct AudioDriver {
    _stream: cpal::Stream,
    pub host_channels: HostChannels,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioDriver {
    pub fn init() -> Result<Self, DriverError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(DriverError::NoOutputDevice)?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let (host_channels, engine_channels) = create_channels();
        let mut processor = RealtimeAudioProcessor::new(engine_channels, sample_rate as f32);

        let err_fn = |err| {
            // Note: en prod temps réel, éviter d'allouer ou logguer directement si appelé en IRQ
            log::error!("Erreur sur le stream audio: {:?}", err);
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    processor.process_interleaved_buffer(data);
                },
                err_fn,
                None,
            )?,
            _ => {
                unimplemented!("Format d'échantillonnage non f32 non encore supporté");
            }
        };

        stream.play()?;

        Ok(Self {
            _stream: stream,
            host_channels,
            sample_rate,
            channels,
        })
    }
}

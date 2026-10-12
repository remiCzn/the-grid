use crate::telemetry::TelemetryRelay;
use grid_core::compiler::PatchCompiler;
use grid_core::message::AudioCommand;
use grid_core::model::Patch;
use grid_engine::driver::AudioDriver;
use grid_engine::ringbuffer::HostChannels;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

/// Garde le `cpal::Stream` en vie. Il n'est jamais utilisé hors du thread qui l'a créé, seulement droppé.
pub struct DriverHandle(pub AudioDriver);

// SAFETY: le stream cpal n'est pas `Send` sur macOS, mais on ne fait que le conserver derrière un Mutex.
unsafe impl Send for DriverHandle {}

/// État global partagé géré par Tauri côté Host (non-audio thread)
pub struct AppState {
    pub channels: Mutex<Option<HostChannels>>,
    pub driver: Mutex<Option<DriverHandle>>,
    pub current_patch: Mutex<Option<Patch>>,
    pub telemetry_relay: TelemetryRelay,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(None),
            driver: Mutex::new(None),
            current_patch: Mutex::new(None),
            telemetry_relay: TelemetryRelay::new(),
        }
    }

    /// Initialise le driver audio CPAL si ce n'est pas déjà fait et démarre la télémétrie
    pub fn ensure_audio_initialized(self: &Arc<Self>, app_handle: &AppHandle) -> Result<(), String> {
        let mut driver_lock = self.driver.lock().map_err(|e| e.to_string())?;
        if driver_lock.is_none() {
            let mut driver = AudioDriver::init().map_err(|e| format!("Erreur driver audio: {e}"))?;
            
            let mut channels_lock = self.channels.lock().map_err(|e| e.to_string())?;
            *channels_lock = driver.host_channels.take();
            *driver_lock = Some(DriverHandle(driver));

            // Démarrer la télémétrie
            self.telemetry_relay.start(app_handle.clone(), Arc::clone(self));
        }
        Ok(())
    }

    /// Envoie une commande vers le thread audio de manière non-bloquante
    pub fn send_audio_command(&self, cmd: AudioCommand) -> Result<(), String> {
        let mut lock = self.channels.lock().map_err(|e| e.to_string())?;
        if let Some(channels) = lock.as_mut() {
            channels
                .command_producer
                .push(cmd)
                .map_err(|_| "Queue de commandes audio saturée".to_string())?;
            Ok(())
        } else {
            Err("Moteur audio non initialisé".to_string())
        }
    }

    /// Valide un patch, compile son graphe topologique et pousse le nouveau plan vers l'audio
    pub fn apply_patch(&self, patch: Patch) -> Result<(), String> {
        // 1. Validation topologique et compilation en plan linéaire
        let plan = PatchCompiler::compile(&patch).map_err(|e| e.to_string())?;

        // 2. Transmission non-bloquante au moteur audio (si initialisé)
        if self.channels.lock().map_err(|e| e.to_string())?.is_some() {
            self.send_audio_command(AudioCommand::LoadExecutionPlan(plan))?;
        }

        // 4. Sauvegarde dans l'état host
        let mut patch_lock = self.current_patch.lock().map_err(|e| e.to_string())?;
        *patch_lock = Some(patch);

        Ok(())
    }
}

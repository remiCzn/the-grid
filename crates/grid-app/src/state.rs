use grid_core::compiler::compile_patch;
use grid_core::graph::AudioGraph;
use grid_core::message::AudioCommand;
use grid_core::model::Patch;
use grid_engine::ringbuffer::HostChannels;
use std::sync::Mutex;

/// État global partagé géré par Tauri côté Host (non-audio thread)
pub struct AppState {
    pub channels: Mutex<Option<HostChannels>>,
    pub current_patch: Mutex<Option<Patch>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(None),
            current_patch: Mutex::new(None),
        }
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
        // 1. Validation topologique et détection de cycles
        let graph = AudioGraph::from_patch(&patch).map_err(|e| e.to_string())?;

        // 2. Compilation en plan linéaire d'exécution
        let plan = compile_patch(&graph, &patch).map_err(|e| e.to_string())?;

        // 3. Transmission non-bloquante au moteur audio
        self.send_audio_command(AudioCommand::LoadExecutionPlan(plan))?;

        // 4. Sauvegarde dans l'état host
        let mut patch_lock = self.current_patch.lock().map_err(|e| e.to_string())?;
        *patch_lock = Some(patch);

        Ok(())
    }
}

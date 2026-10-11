use crate::compiler::ExecutionPlan;
use serde::{Deserialize, Serialize};

/// Commandes envoyées de l'hôte (UI/Backend) vers le thread audio via SPSC Ring Buffer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioCommand {
    /// Active ou désactive le moteur audio
    SetPlaying(bool),
    /// Règle le gain général (Master Gain)
    SetMasterGain(f32),
    /// Met à jour la valeur d'un paramètre spécifique sur un module donné
    SetParameter {
        module_id: u32,
        param_id: u32,
        value: f32,
    },
    /// Met à jour le plan d'exécution audio
    LoadExecutionPlan(ExecutionPlan),
    /// Réinitialise tous les états internes / buffers du DSP
    Reset,
}

/// Télémétrie envoyée du thread audio vers l'hôte (décimée à 30-60 Hz).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TelemetryFrame {
    /// Timestamp relatif ou index de buffer d'échantillons
    pub frame_index: u64,
    /// Niveau crête gauche (0.0..=1.0+)
    pub peak_left: f32,
    /// Niveau crête droit (0.0..=1.0+)
    pub peak_right: f32,
    /// Niveau RMS gauche
    pub rms_left: f32,
    /// Niveau RMS droit
    pub rms_right: f32,
}

impl Default for TelemetryFrame {
    fn default() -> Self {
        Self {
            frame_index: 0,
            peak_left: 0.0,
            peak_right: 0.0,
            rms_left: 0.0,
            rms_right: 0.0,
        }
    }
}

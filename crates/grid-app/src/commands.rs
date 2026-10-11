use crate::state::AppState;
use grid_core::message::AudioCommand;
use grid_core::model::Patch;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn init_audio(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    state.ensure_audio_initialized(&app_handle)
}

#[tauri::command]
pub fn start_audio(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let _ = state.ensure_audio_initialized(&app_handle);
    state.send_audio_command(AudioCommand::SetPlaying(true))
}

#[tauri::command]
pub fn stop_audio(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.send_audio_command(AudioCommand::SetPlaying(false))
}

#[tauri::command]
pub fn set_master_gain(gain: f32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.send_audio_command(AudioCommand::SetMasterGain(gain))
}

#[tauri::command]
pub fn set_parameter(
    module_id: u32,
    param_id: u32,
    value: f32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.send_audio_command(AudioCommand::SetParameter {
        module_id,
        param_id,
        value,
    })
}

#[tauri::command]
pub fn apply_patch(patch: Patch, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.apply_patch(patch)
}

#[tauri::command]
pub fn get_current_patch(state: State<'_, Arc<AppState>>) -> Result<Option<Patch>, String> {
    let lock = state.current_patch.lock().map_err(|e| e.to_string())?;
    Ok(lock.clone())
}

#[tauri::command]
pub fn export_patch(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let lock = state.current_patch.lock().map_err(|e| e.to_string())?;
    if let Some(patch) = lock.as_ref() {
        serde_json::to_string_pretty(patch).map_err(|e| e.to_string())
    } else {
        Err("Aucun patch actif à exporter".to_string())
    }
}

#[tauri::command]
pub fn import_patch(json_str: String, state: State<'_, Arc<AppState>>) -> Result<Patch, String> {
    let patch: Patch = serde_json::from_str(&json_str)
        .map_err(|e| format!("Erreur de format JSON du patch: {e}"))?;
    state.apply_patch(patch.clone())?;
    Ok(patch)
}

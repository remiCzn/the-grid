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
    // S'assurer que le driver est initialisé
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

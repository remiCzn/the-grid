#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use commands::{
    apply_patch, get_current_patch, set_master_gain, set_parameter, start_audio, stop_audio,
};
use state::AppState;

fn main() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_audio,
            stop_audio,
            set_master_gain,
            set_parameter,
            apply_patch,
            get_current_patch
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors de l'exécution de l'application Tauri");
}

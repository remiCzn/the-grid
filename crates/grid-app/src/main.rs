#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use grid_app::commands::{
    apply_patch, get_current_patch, init_audio, set_master_gain, set_parameter, start_audio,
    stop_audio,
};
use grid_app::state::AppState;
use std::sync::Arc;

fn main() {
    let app_state = Arc::new(AppState::new());

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            init_audio,
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

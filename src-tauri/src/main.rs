#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use grid_app::commands;
use grid_app::state::AppState;
use std::sync::Arc;

fn main() {
    let app_state = Arc::new(AppState::new());

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::init_audio,
            commands::start_audio,
            commands::stop_audio,
            commands::set_master_gain,
            commands::set_parameter,
            commands::apply_patch,
            commands::get_current_patch,
            commands::export_patch,
            commands::import_patch,
            commands::send_midi_event
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors de l'exécution de l'application Tauri");
}

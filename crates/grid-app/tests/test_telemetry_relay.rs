use grid_app::state::AppState;
use grid_app::telemetry::TelemetryRelay;
use grid_core::message::AudioCommand;
use grid_core::model::Patch;

#[test]
fn test_app_state_initial_state() {
    let state = AppState::new();
    assert!(state.channels.lock().unwrap().is_none());
    assert!(state.current_patch.lock().unwrap().is_none());
    assert!(state.driver.lock().unwrap().is_none());

    let res = state.send_audio_command(AudioCommand::SetPlaying(true));
    assert_eq!(res.unwrap_err(), "Moteur audio non initialisé");
}

#[test]
fn test_app_state_apply_patch() {
    let state = AppState::new();
    let patch = Patch::new("Test State Patch");

    let res = state.apply_patch(patch.clone());
    assert!(res.is_ok());

    let stored = state.current_patch.lock().unwrap();
    assert_eq!(stored.as_ref().unwrap().name, "Test State Patch");
}

#[test]
fn test_telemetry_relay_lifecycle() {
    let relay = TelemetryRelay::new();
    relay.stop();
}

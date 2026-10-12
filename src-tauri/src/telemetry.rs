use crate::state::AppState;
use grid_core::message::TelemetryFrame;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Relais de télémétrie asynchrone décimé (Host -> Frontend WebView)
pub struct TelemetryRelay {
    is_running: Arc<AtomicBool>,
}

impl TelemetryRelay {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Démarre le worker thread de polling de télémétrie (cadencé à ~60 Hz)
    pub fn start(&self, app_handle: AppHandle, state: Arc<AppState>) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            // Déjà en cours d'exécution
            return;
        }

        let running = Arc::clone(&self.is_running);

        thread::Builder::new()
            .name("grid-telemetry-relay".to_string())
            .spawn(move || {
                let interval = Duration::from_millis(16); // ~60 Hz

                while running.load(Ordering::Relaxed) {
                    let mut latest_frame: Option<TelemetryFrame> = None;

                    // Dépiler toutes les trames disponibles pour ne conserver que la plus récente
                    if let Ok(mut lock) = state.channels.lock() {
                        if let Some(channels) = lock.as_mut() {
                            while let Ok(frame) = channels.telemetry_consumer.pop() {
                                latest_frame = Some(frame);
                            }
                        }
                    }

                    // Émettre l'événement vers le frontend
                    if let Some(frame) = latest_frame {
                        let _ = app_handle.emit("telemetry_update", frame);
                    }

                    thread::sleep(interval);
                }
            })
            .expect("Impossible de démarrer le thread de télémétrie");
    }

    /// Arrête le worker thread de télémétrie
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

//! Startup rules (b) and (d): every qsc call runs on a blocking thread
//! (qsc's Argon2id and file IO are blocking; qsc's blocking HTTP client,
//! when slice B arrives, panics if constructed inside an async context),
//! STRICTLY SERIALLY — one process-wide single-flight gate — and the marker
//! queue is drained after every call. The UI reads `busy` to reflect
//! in-flight state.

use crate::markers::MarkerBuffer;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct CoreGateway {
    gate: tauri::async_runtime::Mutex<()>,
    in_flight: AtomicBool,
    pub markers: MarkerBuffer,
}

impl Default for CoreGateway {
    fn default() -> Self {
        CoreGateway {
            gate: tauri::async_runtime::Mutex::new(()),
            in_flight: AtomicBool::new(false),
            markers: MarkerBuffer::default(),
        }
    }
}

impl CoreGateway {
    /// Run one core call. At most one closure is ever in flight process-wide;
    /// the marker queue is drained before the guard is released.
    pub async fn call<T, F>(&self, f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let _guard = self.gate.lock().await;
        self.in_flight.store(true, Ordering::SeqCst);
        let out = tauri::async_runtime::spawn_blocking(f)
            .await
            .expect("core task join");
        self.markers.drain_from_core();
        self.in_flight.store(false, Ordering::SeqCst);
        out
    }

    pub fn busy(&self) -> bool {
        self.in_flight.load(Ordering::SeqCst)
    }
}

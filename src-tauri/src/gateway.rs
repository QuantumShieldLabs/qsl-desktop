//! Startup rules (b) and (d): every qsc call runs on a blocking thread
//! (qsc's Argon2id and file IO are blocking; qsc's blocking HTTP client,
//! when slice B arrives, panics if constructed inside an async context),
//! STRICTLY SERIALLY — one process-wide single-flight gate — and the marker
//! queue is drained after every call. The UI reads `busy` to reflect
//! in-flight state.

use crate::debug_log::DebugLog;
use crate::markers::MarkerBuffer;
use qsc::output::event::Outcome;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// NA-0779 (`D-0048`): what `gw.command` may learn from a command's RETURN VALUE -- an outcome
/// and, for a failure, the error's code (looked up in the closed reason vocabulary by the log;
/// free text never enters). RULING_NA0779_005 R2 F-04: the outcome is SEMANTIC, not the `Result`
/// variant -- `Ok(t)` is whatever `t` says of itself, so a rejected passphrase, an unreachable
/// relay or an unfinished handshake reads `out=fail` with a closed reason. Every type a
/// `call_named` closure returns implements it: the plain ones say `ok`; `UnlockDto`,
/// `RelayTestDto` and `Finished` (commands.rs) know better.
pub trait CommandOutcome {
    fn outcome(&self) -> (Outcome, Option<String>);
}

/// An error's CODE -- the closed token the log looks up in the engine's reason vocabulary
/// (`String` errors in this crate ARE codes; `ErrorDto` carries its wire code beside a detail
/// that never enters). Free text is never read past the lookup.
pub trait ErrorCode {
    fn code(&self) -> String;
}

impl ErrorCode for String {
    fn code(&self) -> String {
        self.clone()
    }
}

impl ErrorCode for &str {
    fn code(&self) -> String {
        (*self).to_string()
    }
}

impl<T: CommandOutcome, E: ErrorCode> CommandOutcome for Result<T, E> {
    fn outcome(&self) -> (Outcome, Option<String>) {
        match self {
            Ok(t) => t.outcome(),
            Err(e) => (Outcome::Fail, Some(e.code())),
        }
    }
}

/// A list or an optional value is an answer, not a failure (an empty invite list, `None` from
/// an accept that consumed nothing this poll -- named in D-0048 004d, left as `ok`).
impl<T> CommandOutcome for Vec<T> {
    fn outcome(&self) -> (Outcome, Option<String>) {
        (Outcome::Ok, None)
    }
}
impl<T> CommandOutcome for Option<T> {
    fn outcome(&self) -> (Outcome, Option<String>) {
        (Outcome::Ok, None)
    }
}

macro_rules! infallible_outcome {
    ($($t:ty),* $(,)?) => {
        $(impl CommandOutcome for $t {
            fn outcome(&self) -> (Outcome, Option<String>) {
                (Outcome::Ok, None)
            }
        })*
    };
}
infallible_outcome!(
    (),
    bool,
    String,
    crate::state::LaunchState,
    crate::commands::ProtectionDto,
    crate::commands::IdentityDto,
    crate::commands::ConnectStatusDto,
);

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

    /// NA-0779 (`D-0048`): the same serial call, named. ONE `gw.command` event per call -- the
    /// command's NAME (a member of the registered set), its outcome, its error code (a member of
    /// the engine's closed reason vocabulary, else `?`) and its duration. The closure's ARGUMENTS
    /// are not visible here and never enter; with the log off the push is one mutex look.
    pub async fn call_named<T, F>(&self, name: &'static str, f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: CommandOutcome + Send + 'static,
    {
        let t0 = Instant::now();
        let out = self.call(f).await;
        let (outcome, reason) = out.outcome();
        DebugLog::global().gw_command(name, outcome, reason.as_deref(), t0.elapsed().as_millis());
        out
    }

    pub fn busy(&self) -> bool {
        self.in_flight.load(Ordering::SeqCst)
    }
}

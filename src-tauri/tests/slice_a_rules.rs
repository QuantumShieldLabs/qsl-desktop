//! D595 slice A — the four startup rules, each test-proven, plus the
//! zero-networking and no-raw-globals source scans.
//!
//! Env discipline: QSC_CONFIG_DIR is process-global, so every test that
//! touches it (or qsc's process lock state) serializes on ENV_LOCK and
//! re-bootstraps into its own temp dir (qsc re-reads the env per call).

use qsl_desktop_app::gateway::CoreGateway;
use qsl_desktop_app::{bootstrap, paths};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
}

fn fresh_env(tmp: &tempfile::TempDir) {
    qsc::vault::protection::lock(None);
    bootstrap(tmp.path()).expect("bootstrap");
}

/// Rule (a): bootstrap fixes QSC_CONFIG_DIR into the app-scoped dir, creates
/// both dirs 0700, and routes markers InApp — a subsequent core call's marker
/// lands in the queue, not on stdout (stdout is not capturable here, but the
/// queue APPEARING proves the InApp branch ran; the routing enum has exactly
/// two variants).
#[test]
fn rule_a_bootstrap_env_dirs_and_routing() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    let qsc_dir = paths::qsc_config_dir(tmp.path());
    assert_eq!(
        std::env::var("QSC_CONFIG_DIR").unwrap(),
        qsc_dir.to_string_lossy()
    );
    for d in [tmp.path(), qsc_dir.as_path()] {
        let mode = fs::metadata(d).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "dir {} not 0700", d.display());
    }

    qsc::vault::vault_init_with_passphrase("rule-a-pass").expect("init");
    let queued = {
        let q = qsc::output::marker_queue();
        let g = q.lock().unwrap_or_else(|p| p.into_inner());
        g.len()
    };
    assert!(
        queued >= 1,
        "vault_init marker should be queued under InApp"
    );
    // leave the queue clean for later tests
    let q = qsc::output::marker_queue();
    q.lock().unwrap_or_else(|p| p.into_inner()).clear();
    qsc::vault::protection::lock(None);
}

/// Rule (b): after EVERY gateway call the qsc queue is drained completely
/// into the bounded buffer.
#[test]
fn rule_b_drain_after_every_call() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    let gw = CoreGateway::default();
    tauri::async_runtime::block_on(async {
        gw.call(|| qsc::vault::vault_init_with_passphrase("rule-b-pass"))
            .await
            .expect("init");
    });
    let remaining = {
        let q = qsc::output::marker_queue();
        let g = q.lock().unwrap_or_else(|p| p.into_inner());
        g.len()
    };
    assert_eq!(remaining, 0, "qsc queue must be empty after a gateway call");
    let (buffered, dropped) = gw.markers.stats();
    assert!(
        buffered >= 1,
        "the drained marker must be in the app buffer"
    );
    assert_eq!(dropped, 0);
    qsc::vault::protection::lock(None);
}

/// Rule (c): GUI code never touches the raw unlock globals — the one-call
/// NA-0658 surface is the only lock-state path. Source-scan over src/ (the
/// shipped code; this test file lives outside it).
#[test]
fn rule_c_no_raw_global_symbols_in_src() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = ["set_vault_unlocked", "set_process_passphrase"];
    for entry in walk(&src) {
        let text = fs::read_to_string(&entry).unwrap();
        for f in forbidden {
            assert!(
                !text.contains(f),
                "forbidden raw-global symbol `{f}` found in {}",
                entry.display()
            );
        }
    }
}

/// Slice-A boundary: ZERO networking code. Source-scan over src/ and ui/.
#[test]
fn zero_networking_in_src_and_ui() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let roots = [manifest.join("src"), manifest.join("../ui")];
    let forbidden = ["reqwest", "hyper", "http://", "https://", "ws://", "wss://"];
    for root in roots {
        for entry in walk(&root) {
            let text = fs::read_to_string(&entry).unwrap();
            for f in forbidden {
                assert!(
                    !text.contains(f),
                    "networking token `{f}` found in {}",
                    entry.display()
                );
            }
        }
    }
}

/// Rule (d): STRICTLY SERIAL core calls — concurrent gateway callers never
/// overlap (observed concurrency stays exactly 1).
#[test]
fn rule_d_strictly_serial_single_flight() {
    let gw = Arc::new(CoreGateway::default());
    let current = Arc::new(AtomicU32::new(0));
    let max_seen = Arc::new(AtomicU32::new(0));

    tauri::async_runtime::block_on(async {
        let mut handles = Vec::new();
        for _ in 0..4 {
            let gw = gw.clone();
            let current = current.clone();
            let max_seen = max_seen.clone();
            handles.push(tauri::async_runtime::spawn(async move {
                gw.call(move || {
                    let now = current.fetch_add(1, Ordering::SeqCst) + 1;
                    max_seen.fetch_max(now, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(40));
                    current.fetch_sub(1, Ordering::SeqCst);
                })
                .await;
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
    });
    assert_eq!(
        max_seen.load(Ordering::SeqCst),
        1,
        "core calls overlapped; the single-flight gate failed"
    );
}

fn walk(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap_or_else(|_| panic!("readdir {}", d.display())) {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    assert!(!out.is_empty(), "scan root {} is empty", root.display());
    out
}

//! Desktop lock follow-up F2 (SR-15 of PR #63, findings L5-a and L7-a): four commands must
//! refuse while the vault is LOCKED, the RD-01 shape. `settings_set` and `relay_config_set`
//! wrote settings.json on the locked screen (autolock 0 = never); `relay_test` and
//! `relay_probe` opened a connection on it. Delta symbols: F2-SET, F2-RCS, F2-RT, F2-RP -- the
//! `require_unlocked()?` first in each (inside the gateway closure for the two probes).
//!
//! RED at 8b6f18f7: `settings_set` answers Ok while locked and rewrites settings.json. GREEN
//! with the four gates: each answers `vault_locked`, settings.json is byte-identical, and the
//! loopback listener counts no connection.
//!
//! The UNLOCKED arm is the control: the same writes land and the same probes REACH the
//! listener, so a count of zero while locked is a measurement, not a listener that cannot see.

use serde_json::{json, Value};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

type MockWebview = tauri::WebviewWindow<tauri::test::MockRuntime>;

const PASS: &str = "f2-gate-correct-passphrase";

/// Both tests set the same process-global environment (HOME, QSC_CONFIG_DIR, the relay
/// token variables), so they must not run on parallel threads. Poisoning is ignored: one
/// panicking test is the failure to report, not the other one's PoisonError.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serialized() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn private_dir(p: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(p).unwrap();
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o700)).unwrap();
}

fn invoke(wv: &MockWebview, cmd: &str, args: Value) -> Result<Value, String> {
    let body = match args {
        Value::Null => tauri::ipc::InvokeBody::default(),
        v => tauri::ipc::InvokeBody::Json(v),
    };
    match tauri::test::get_ipc_response(
        wv,
        tauri::webview::InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: wv.url().expect("webview origin url"),
            body,
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    ) {
        Ok(resp) => Ok(resp.deserialize::<Value>().expect("json")),
        Err(e) => Err(format!("{e:?}")),
    }
}

/// A loopback listener that counts every connection it accepts and closes it at once, so a
/// TLS client sees EOF immediately instead of waiting out its timeout. The count is taken
/// BEFORE the close, so it is final by the time the client's call returns.
fn counting_listener() -> (u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let count = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&count);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            seen.fetch_add(1, Ordering::SeqCst);
            drop(stream);
        }
    });
    (port, count)
}

/// Hermetic app with an UNLOCKED vault (vault_create unlocks), the rd01 composition.
fn boot(
    root: &std::path::Path,
) -> (
    tauri::App<tauri::test::MockRuntime>,
    MockWebview,
    std::path::PathBuf,
) {
    let home = root.join("home");
    std::fs::create_dir_all(home.join(".config")).unwrap();
    std::env::set_var("HOME", &home);
    std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
    std::env::set_var("XDG_DATA_HOME", home.join(".local/share"));
    std::env::set_var("QSC_DISABLE_KEYCHAIN", "1");
    for k in [
        "QSC_RELAY_TOKEN",
        "RELAY_TOKEN",
        "QSC_RELAY_CA_FILE",
        "RELAY_CA_FILE",
    ] {
        std::env::remove_var(k);
    }
    let data_dir = root.join("qsld-data");
    private_dir(&data_dir);
    let qsc_dir = data_dir.join("qsc");
    private_dir(&qsc_dir);
    std::env::set_var("QSC_CONFIG_DIR", &qsc_dir);
    qsc::output::init_output_policy(false);
    qsc::output::set_marker_routing(qsc::output::MarkerRouting::InApp);

    let app = qsl_desktop_app::configure_builder(
        tauri::test::mock_builder(),
        qsl_desktop_app::AppState {
            data_dir: data_dir.clone(),
            gw: qsl_desktop_app::gateway::CoreGateway::default(),
        },
    )
    .build(tauri::generate_context!())
    .expect("mock app");
    let wv: MockWebview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::default())
            .build()
            .expect("mock main webview");
    invoke(
        &wv,
        "vault_create",
        json!({"passphrase": PASS, "confirm": PASS}),
    )
    .expect("create");
    (app, wv, data_dir.join("settings.json"))
}

fn assert_refused(cmd: &str, r: &Result<Value, String>) {
    match r {
        Err(e) => assert!(
            e.contains("vault_locked"),
            "{cmd} must refuse with vault_locked while locked, got Err({e})"
        ),
        Ok(v) => panic!("{cmd} must refuse while locked, got Ok({v})"),
    }
}

#[test]
fn f2_locked_settings_and_relay_commands_refuse_and_touch_nothing() {
    let _serial = serialized();
    let root = tempfile::tempdir().unwrap();
    let (_app, wv, settings_json) = boot(root.path());
    let (port, count) = counting_listener();
    let url = format!("https://127.0.0.1:{port}");

    // settings.json exists before the lock, written the ordinary way while unlocked.
    invoke(
        &wv,
        "settings_set",
        json!({"autolockMinutes": 15, "selfAlias": "Before"}),
    )
    .expect("settings_set while unlocked");
    let before = std::fs::read(&settings_json).expect("settings.json written while unlocked");

    invoke(&wv, "lock_now", Value::Null).expect("lock");
    let status = invoke(&wv, "protection_status", Value::Null).expect("status");
    assert_eq!(status["locked"], json!(true), "premise: locked: {status}");

    // THE ACT, from the locked screen. Each value differs from what is stored, so a write
    // would change the bytes.
    let set = invoke(
        &wv,
        "settings_set",
        json!({"autolockMinutes": 0, "selfAlias": "LockedWriter"}),
    );
    eprintln!("F2 settings_set while locked -> {set:?}");
    assert_refused("settings_set", &set);
    let rcs = invoke(&wv, "relay_config_set", json!({"url": url}));
    eprintln!("F2 relay_config_set while locked -> {rcs:?}");
    assert_refused("relay_config_set", &rcs);
    let rt = invoke(&wv, "relay_test", json!({"url": url}));
    eprintln!("F2 relay_test while locked -> {rt:?}");
    assert_refused("relay_test", &rt);
    let rp = invoke(
        &wv,
        "relay_probe",
        json!({"address": url, "token": "f2-probe-token"}),
    );
    eprintln!("F2 relay_probe while locked -> {rp:?}");
    assert_refused("relay_probe", &rp);

    let after = std::fs::read(&settings_json).expect("settings.json still present");
    assert!(
        after == before,
        "settings.json must be byte-identical after the locked calls"
    );
    let n = count.load(Ordering::SeqCst);
    eprintln!("F2 listener connections while locked: {n}");
    assert_eq!(n, 0, "no connection may be attempted while locked");
}

#[test]
fn f2_unlocked_control_settings_write_and_relay_reaches() {
    let _serial = serialized();
    let root = tempfile::tempdir().unwrap();
    let (_app, wv, _settings_json) = boot(root.path());
    let (port, count) = counting_listener();
    let url = format!("https://127.0.0.1:{port}");

    invoke(
        &wv,
        "settings_set",
        json!({"autolockMinutes": 7, "selfAlias": "Control"}),
    )
    .expect("settings_set while unlocked");
    invoke(&wv, "relay_config_set", json!({"url": url})).expect("relay_config_set while unlocked");
    let s = invoke(&wv, "settings_get", Value::Null).expect("settings_get");
    eprintln!("F2 control settings_get: {s}");
    assert_eq!(
        s["autolock_minutes"],
        json!(7),
        "the autolock was written: {s}"
    );
    assert_eq!(
        s["self_alias"],
        json!("Control"),
        "the alias was written: {s}"
    );
    assert_eq!(
        s["relay_url"],
        json!(url),
        "the relay address was written: {s}"
    );

    let rt = invoke(&wv, "relay_test", json!({"url": url}));
    let after_rt = count.load(Ordering::SeqCst);
    eprintln!("F2 control relay_test -> {rt:?}; connections {after_rt}");
    assert!(rt.is_ok(), "relay_test answers while unlocked: {rt:?}");
    assert!(after_rt >= 1, "relay_test must reach the listener");

    let rp = invoke(
        &wv,
        "relay_probe",
        json!({"address": url, "token": "f2-probe-token"}),
    );
    let after_rp = count.load(Ordering::SeqCst);
    eprintln!("F2 control relay_probe -> {rp:?}; connections {after_rp}");
    assert!(rp.is_ok(), "relay_probe answers while unlocked: {rp:?}");
    assert!(after_rp > after_rt, "relay_probe must reach the listener");
}

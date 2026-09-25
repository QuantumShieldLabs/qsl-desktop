//! REVIEW-desktop RD-01: the two commands that
//! change the unlock-protection setting must refuse while the vault is LOCKED, because
//! qsc's `set_attempt_limit` zeroes the failure counter and the last-failure time on every
//! arm or disarm (vault/protection.rs:242-248 at the pin). Delta symbol: RD01-WIPE-GATE.
//!
//! RED at 92cba80a: `wipe_disarm` is accepted on the locked screen and the escalating unlock
//! delay reads zero afterwards. GREEN with fixes/RD-01.diff: refused with `vault_locked`, and the
//! counter and the delay are untouched.

use serde_json::{json, Value};

type MockWebview = tauri::WebviewWindow<tauri::test::MockRuntime>;

const PASS: &str = "rd01-correct-passphrase";

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

#[test]
fn rd01_wipe_arm_and_disarm_refuse_while_locked() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    std::fs::create_dir_all(home.join(".config")).unwrap();
    std::env::set_var("HOME", &home);
    std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
    std::env::set_var("XDG_DATA_HOME", home.join(".local/share"));
    std::env::set_var("QSC_DISABLE_KEYCHAIN", "1");
    let data_dir = root.path().join("qsld-data");
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
    invoke(&wv, "lock_now", Value::Null).expect("lock");

    // Three wrong guesses: the schedule's first real delay (5 s at failed_unlocks = 3).
    for _ in 0..3 {
        let r =
            invoke(&wv, "unlock_attempt", json!({"passphrase": "wrong-guess"})).expect("attempt");
        assert_eq!(r["kind"], "rejected", "a wrong guess is rejected: {r}");
    }
    let before = invoke(&wv, "protection_status", Value::Null).expect("status");
    eprintln!("RD01 before: {before}");
    assert_eq!(before["locked"], json!(true));
    assert_eq!(before["failed_unlocks"], json!(3));
    assert!(
        before["retry_after_s"].as_u64().unwrap() > 0,
        "a delay is in force: {before}"
    );

    // THE ACT, from the locked screen.
    let disarm = invoke(&wv, "wipe_disarm", Value::Null);
    let arm = invoke(&wv, "wipe_arm", json!({"limit": 100}));
    let after = invoke(&wv, "protection_status", Value::Null).expect("status");
    eprintln!("RD01 wipe_disarm while locked -> {disarm:?}");
    eprintln!("RD01 wipe_arm while locked -> {arm:?}");
    eprintln!("RD01 after: {after}");

    assert!(
        disarm.is_err(),
        "wipe_disarm must refuse while locked, got {disarm:?}"
    );
    assert!(
        arm.is_err(),
        "wipe_arm must refuse while locked, got {arm:?}"
    );
    assert_eq!(
        after["failed_unlocks"],
        json!(3),
        "the failure counter must survive: {after}"
    );
    assert!(
        after["retry_after_s"].as_u64().unwrap() > 0,
        "the delay must survive: {after}"
    );
}

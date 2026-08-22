//! NA-0754 (D-0035) — THE PERSIST BOUNDARY: a probe writes NOTHING.
//!
//! This is the engine half of the lane's central invariant — WHAT IS PERSISTED HAS
//! CONNECTED AT LEAST ONCE — and it is the seal the GUI harness structurally cannot
//! carry. Three reasons the proof has to live here rather than in a scenario:
//!
//!   1. `relay_token_show` is a BARE BOOL by design (FLAG-3), so a scenario watching
//!      the surface cannot tell "token unchanged" from "token replaced by a different
//!      token" — `configured` reads true on both sides. Only a direct
//!      `vault::secret_get` comparison of the EXACT STRING can. That is the whole
//!      reason this file exists.
//!   2. The runner has no file-hash op, so "settings.json unchanged" is a per-key
//!      claim there. Here the values are read back directly.
//!   3. ⚠ AND THE CONTROL NEEDS NO RELAY. Every accepting arm's control must be aimed
//!      at the arm that CAN FAIL and be PROVEN TO DIFFER FIRST. A "nothing moved"
//!      assertion is worthless unless the same probes are shown MOVING under a real
//!      write — so each test writes through the trio commands first, proves all three
//!      observables moved, and only then proves the probe leaves them alone. No relay
//!      is reachable at any point; the differ-proof is relay-free by construction.
//!
//! Claim boundary, stated: this file proves the ENGINE persists nothing on a probe.
//! It does not click anything. Whether the FRONT END only calls the persist commands
//! after a Connected result is the harness's half (scenario `f_j`) and the operator's
//! acceptance flight.

use serde_json::{json, Value};

const PASS: &str = "na0754-persist-boundary-passphrase";

/// The address every probe here is aimed at. `.test` is a RESERVED, non-resolving
/// TLD (RFC 2606), so the probe classifies `Unreachable` without touching a network
/// anyone owns — deterministic offline, and a genuine RED rung.
const UNREACHABLE: &str = "https://relay.example.test:8443";

type MockWebview = tauri::WebviewWindow<tauri::test::MockRuntime>;

/// ⚠ THIS FILE'S TESTS MUST NOT RUN CONCURRENTLY, and the reason is the same fact
/// the lane is built on: `relay_probe` reaches qsc through PROCESS-GLOBAL
/// environment variables, and `boot()` sets HOME / QSC_CONFIG_DIR / QSLD_DATA_DIR
/// the same way. Cargo runs a binary's tests on parallel threads by default, so
/// without this every test would be reading another test's environment — and the
/// failure would be an intermittent one that looks like a product bug.
///
/// Held for the WHOLE body, not just `boot()`, because the env stays load-bearing
/// for the duration. Poisoning is ignored deliberately: one test panicking is a
/// real failure to report, and turning the other three into confusing
/// `PoisonError`s would hide it behind an unrelated symptom.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serialized() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn create_private_dir_0700(p: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(p)?;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o700))
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
        Ok(resp) => Ok(resp
            .deserialize::<Value>()
            .expect("IPC response deserializes as JSON")),
        Err(e) => Err(format!("{e:?}")),
    }
}

fn ok(wv: &MockWebview, cmd: &str, args: Value) -> Value {
    match invoke(wv, cmd, args) {
        Ok(v) => v,
        Err(e) => panic!("command `{cmd}` rejected at the IPC boundary: {e}"),
    }
}

/// The three observables, read at their SOURCE rather than through a summary.
///
/// ⚠ The token is read with `vault::secret_get`, NOT `relay_token_show`. The command
/// returns `{configured: bool}` and a bool cannot distinguish "unchanged" from
/// "replaced" — the vacuous-seal shape this file exists to avoid.
#[derive(Debug, PartialEq, Eq)]
struct Stored {
    relay_url: String,
    token: Option<String>,
    ca_path_hash: Option<String>,
}

fn read_stored(wv: &MockWebview) -> Stored {
    let cfg = ok(wv, "relay_config_get", Value::Null);
    let ca = ok(wv, "relay_ca_file_show", Value::Null);
    Stored {
        relay_url: cfg["relay_url"].as_str().unwrap_or_default().to_string(),
        token: qsc::vault::secret_get(qsc::store::TUI_RELAY_TOKEN_SECRET_KEY)
            .expect("vault readable"),
        ca_path_hash: ca["path_hash"].as_str().map(|s| s.to_string()),
    }
}

/// Hermetic app + an unlocked vault, mirroring `bootstrap()`'s startup rule (a)
/// exactly as the NA-0700 replay harness does.
fn boot(root: &std::path::Path) -> (tauri::App<tauri::test::MockRuntime>, MockWebview) {
    let home = root.join("home");
    std::fs::create_dir_all(home.join(".config")).expect("home");
    std::env::set_var("HOME", &home);
    std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
    std::env::set_var("XDG_DATA_HOME", home.join(".local/share"));
    std::env::set_var("QSC_DISABLE_KEYCHAIN", "1");
    let data_dir = root.join("qsld-data");
    std::env::set_var("QSLD_DATA_DIR", &data_dir);
    create_private_dir_0700(&data_dir).expect("data dir");
    let qsc_dir = data_dir.join("qsc");
    create_private_dir_0700(&qsc_dir).expect("qsc dir");
    std::env::set_var("QSC_CONFIG_DIR", &qsc_dir);
    // ⚠ The probe's explicit values arrive through these two variables, so a leaked
    // one from an earlier run would silently override the vault and make this file's
    // central claim untestable. Cleared before every boot.
    std::env::remove_var("QSC_RELAY_TOKEN");
    std::env::remove_var("RELAY_TOKEN");
    std::env::remove_var("QSC_RELAY_CA_FILE");
    std::env::remove_var("RELAY_CA_FILE");
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
    .expect("mock app builds with the run-path composition and real context");

    let wv: MockWebview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::default())
            .build()
            .expect("main webview");
    ok(
        &wv,
        "vault_create",
        json!({ "passphrase": PASS, "confirm": PASS }),
    );
    (app, wv)
}

/// A real PEM certificate file, so the CA rung can be driven on its ACCEPTING arm
/// and not only on its failures. Self-signed, generated once and pinned as bytes so
/// the fixture needs no dependency and no network.
fn write_pem_fixture(path: &std::path::Path) {
    std::fs::write(path, PEM_FIXTURE).expect("write PEM fixture");
}

// A minimal self-signed CA certificate in PEM form. Its VALIDITY as a trust anchor
// is irrelevant here — the only thing under test is that
// `reqwest::Certificate::from_pem_bundle` PARSES it, which is what separates
// `relay_ca_file_invalid` from a CA that is accepted and carried into the probe.
const PEM_FIXTURE: &str = include_str!("fixtures/na0754_ca.pem");

// ─────────────────────────────────────────────────────────────────────────────
// Y1 — THE INVARIANT, ENGINE HALF: a probe on a FAILING rung persists NOTHING,
// and the control proves all three observables CAN move.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn a_failed_probe_persists_nothing_and_the_control_proves_the_probes_can_move() {
    let _serial = serialized();
    let root = tempfile::tempdir().expect("tmp root");
    let (_app, wv) = boot(root.path());

    // ── THE DIFFER-CONTROL, FIRST. Aimed at the arm that CAN fail: if these three
    // observables could not move, the "nothing moved" assertion below would pass
    // vacuously. Nothing is asserted about the probe until this has been shown.
    let before_any = read_stored(&wv);
    assert_eq!(
        before_any.relay_url, "",
        "a fresh profile stores no address"
    );
    assert_eq!(before_any.token, None, "a fresh profile stores no token");
    assert_eq!(
        before_any.ca_path_hash, None,
        "a fresh profile stores no CA"
    );

    let ca_file = root.path().join("real-ca.pem");
    write_pem_fixture(&ca_file);
    let good_url = "https://relay.working.test:8443";
    ok(&wv, "relay_config_set", json!({ "url": good_url }));
    ok(&wv, "relay_token_set", json!({ "token": "TOKEN-ALPHA" }));
    ok(
        &wv,
        "relay_ca_file_set",
        json!({ "path": ca_file.to_str().expect("utf8 path") }),
    );

    let working = read_stored(&wv);
    assert_ne!(
        working.relay_url, before_any.relay_url,
        "DIFFER-CONTROL: the address MUST move under a real write"
    );
    assert_ne!(
        working.token, before_any.token,
        "DIFFER-CONTROL: the token MUST move under a real write"
    );
    assert_ne!(
        working.ca_path_hash, before_any.ca_path_hash,
        "DIFFER-CONTROL: the CA hash MUST move under a real write"
    );
    assert_eq!(
        working.token.as_deref(),
        Some("TOKEN-ALPHA"),
        "the EXACT token string is what is compared, not a bool"
    );

    // ── NOW THE CLAIM. A probe carrying a DIFFERENT triple, on a failing rung.
    // If the probe wrote anything, every one of the three would move — and the
    // control above has just proven each of them is capable of moving.
    let other_ca = root.path().join("other-ca.pem");
    write_pem_fixture(&other_ca);
    let res = ok(
        &wv,
        "relay_probe",
        json!({
            "address": UNREACHABLE,
            "token": "TOKEN-BRAVO-NEVER-PERSISTED",
            "caPath": other_ca.to_str().expect("utf8 path"),
        }),
    );
    assert_eq!(
        res["kind"], "unreachable",
        "the .test address is a genuine RED rung"
    );

    let after = read_stored(&wv);
    assert_eq!(
        after, working,
        "A FAILED PROBE PERSISTED SOMETHING — the invariant is broken"
    );
    assert_eq!(
        after.token.as_deref(),
        Some("TOKEN-ALPHA"),
        "the working token survived byte-for-byte"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Y3 — THE CLOBBER IS DEAD, engine half: a probe with a BROKEN address leaves the
// proven-good configuration exactly where it was. The :844 class's named grave.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn probing_a_broken_address_cannot_clobber_a_working_configuration() {
    let _serial = serialized();
    let root = tempfile::tempdir().expect("tmp root");
    let (_app, wv) = boot(root.path());

    let ca_file = root.path().join("real-ca.pem");
    write_pem_fixture(&ca_file);
    let working_url = "https://relay.known-good.test:8443";
    ok(&wv, "relay_config_set", json!({ "url": working_url }));
    ok(
        &wv,
        "relay_token_set",
        json!({ "token": "KNOWN-GOOD-TOKEN" }),
    );
    ok(
        &wv,
        "relay_ca_file_set",
        json!({ "path": ca_file.to_str().expect("utf8 path") }),
    );
    let working = read_stored(&wv);
    assert_eq!(working.relay_url, working_url);

    // The flight's exact gesture: retype a broken address over a working one and
    // press Test. Under the OLD model this wrote first and then reported failure,
    // which is how a proven-good address was lost.
    let res = ok(
        &wv,
        "relay_probe",
        json!({ "address": "https://typo.example.test:8443", "token": Value::Null, "caPath": Value::Null }),
    );
    assert_eq!(res["kind"], "unreachable");

    assert_eq!(
        read_stored(&wv),
        working,
        "the WORKING configuration must survive a failed test of a broken address"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Y5 — THE CA TRUTH: all four rungs, driven OFFLINE.
//
// ⛳ WHY THIS WORKS WITH NO RELAY. `relay_server_info` builds its HTTP client —
// and therefore reads, and PEM-parses, the configured CA — BEFORE it opens a
// socket, returning the CA code as an `Err` with no request formed. So a CA
// failure is distinguishable from a network failure by outcome alone: a CA fault
// yields `Err(relay_ca_file_*)`, while a VALID CA on an unreachable address yields
// `Ok(unreachable)`. The unreachable classification IS the proof the CA rung passed.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn the_ca_rung_reports_which_check_failed_and_a_real_pem_passes_it() {
    let _serial = serialized();
    let root = tempfile::tempdir().expect("tmp root");
    let (_app, wv) = boot(root.path());

    // ARM 1 — MISSING: a path with no file at it.
    let missing = root.path().join("nope").join("absent.pem");
    let e = invoke(
        &wv,
        "relay_probe",
        json!({ "address": UNREACHABLE, "token": Value::Null, "caPath": missing.to_str().unwrap() }),
    )
    .expect_err("a missing CA file must fail the probe locally");
    assert!(
        e.contains("relay_ca_file_missing"),
        "expected relay_ca_file_missing, got {e}"
    );

    // ARM 2 — UNREADABLE: a directory is not a file.
    let dir_as_ca = root.path().join("a-directory");
    std::fs::create_dir_all(&dir_as_ca).expect("dir");
    let e = invoke(
        &wv,
        "relay_probe",
        json!({ "address": UNREACHABLE, "token": Value::Null, "caPath": dir_as_ca.to_str().unwrap() }),
    )
    .expect_err("a directory must fail the probe locally");
    assert!(
        e.contains("relay_ca_file_unreadable"),
        "expected relay_ca_file_unreadable, got {e}"
    );

    // ARM 3 — INVALID: a real, readable file that holds no certificate. THE GARBAGE
    // PATH THE OLD PANE CONGRATULATED — `relay_ca_file_set` would have stored this
    // silently and then reported it as configured (ENG-0222).
    let not_a_cert = root.path().join("shopping-list.txt");
    std::fs::write(&not_a_cert, b"milk\neggs\n").expect("write");
    let e = invoke(
        &wv,
        "relay_probe",
        json!({ "address": UNREACHABLE, "token": Value::Null, "caPath": not_a_cert.to_str().unwrap() }),
    )
    .expect_err("a non-certificate file must fail the probe locally");
    assert!(
        e.contains("relay_ca_file_invalid"),
        "expected relay_ca_file_invalid, got {e}"
    );

    // ARM 4 — VALID, THE ACCEPTING ARM: a real PEM parses, so the probe gets PAST the
    // CA rung and fails on the network instead. `Ok(unreachable)` rather than any
    // `Err(relay_ca_file_*)` is the proof the certificate was accepted.
    let real = root.path().join("real-ca.pem");
    write_pem_fixture(&real);
    let res = ok(
        &wv,
        "relay_probe",
        json!({ "address": UNREACHABLE, "token": Value::Null, "caPath": real.to_str().unwrap() }),
    );
    assert_eq!(
        res["kind"], "unreachable",
        "a valid PEM must pass the CA rung and fail only on the network"
    );

    // And none of the four wrote anything.
    let after = read_stored(&wv);
    assert_eq!(
        after.ca_path_hash, None,
        "no CA rung, valid or invalid, may persist a path"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// THE RESIDUAL HAZARD, SEALED. `relay_probe` overrides qsc's secret resolution by
// setting two PROCESS-GLOBAL environment variables for the duration of one probe.
// A leaked variable would silently override the vault for every later call in the
// process, so the restore is not a detail — it is the boundary.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn the_probe_restores_the_environment_it_borrowed_including_absence() {
    let _serial = serialized();
    let root = tempfile::tempdir().expect("tmp root");
    let (_app, wv) = boot(root.path());

    // ARM 1 — ABSENT STAYS ABSENT. `remove_var`, not `set_var("")`: qsc trims an
    // empty value to None and falls through to the vault, so "" would NOT restore
    // absence and the distinction is load-bearing.
    assert!(
        std::env::var("QSC_RELAY_TOKEN").is_err(),
        "precondition: unset"
    );
    let real = root.path().join("real-ca.pem");
    write_pem_fixture(&real);
    ok(
        &wv,
        "relay_probe",
        json!({ "address": UNREACHABLE, "token": "BORROWED", "caPath": real.to_str().unwrap() }),
    );
    assert!(
        std::env::var("QSC_RELAY_TOKEN").is_err(),
        "QSC_RELAY_TOKEN leaked out of the probe — it would override the vault for every later call"
    );
    assert!(
        std::env::var("QSC_RELAY_CA_FILE").is_err(),
        "QSC_RELAY_CA_FILE leaked out of the probe"
    );

    // ARM 2 — A PRE-EXISTING VALUE IS RESTORED, not clobbered. The operator's own
    // QSC_RELAY_TOKEN must survive a probe that borrowed the variable.
    std::env::set_var("QSC_RELAY_TOKEN", "OPERATORS-OWN");
    ok(
        &wv,
        "relay_probe",
        json!({ "address": UNREACHABLE, "token": "BORROWED-AGAIN", "caPath": Value::Null }),
    );
    assert_eq!(
        std::env::var("QSC_RELAY_TOKEN").ok().as_deref(),
        Some("OPERATORS-OWN"),
        "a pre-existing environment value must be restored exactly"
    );
    std::env::remove_var("QSC_RELAY_TOKEN");

    // ARM 3 — NOT BORROWED AT ALL: a `None` argument must leave the variable alone
    // rather than removing it, so blank-means-keep really does reach the vault.
    std::env::set_var("QSC_RELAY_TOKEN", "UNTOUCHED");
    ok(
        &wv,
        "relay_probe",
        json!({ "address": UNREACHABLE, "token": Value::Null, "caPath": Value::Null }),
    );
    assert_eq!(
        std::env::var("QSC_RELAY_TOKEN").ok().as_deref(),
        Some("UNTOUCHED"),
        "a probe that borrows nothing must not disturb the environment"
    );
    std::env::remove_var("QSC_RELAY_TOKEN");
}

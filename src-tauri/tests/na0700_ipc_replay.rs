//! NA-0700 (D634 A2-FINAL item 9 / D-0025) — the IPC replay harness.
//!
//! A MAJORITY of registered commands -- 29 of 46 at NA-0776's base 83019356 -- are
//! invoked through tauri's REAL IPC ingestion on the mock runtime — real serde arg
//! decoding, real camelCase→snake_case mapping, real State injection — with the
//! arg-key sets HARVESTED from the `main.js` call sites and replayed literally
//! ⚠ THIS LINE ONCE READ "Every registered command", which was honest at 27 commands
//! and became FALSE as the surface grew to 46 (NA-0776 / RULING_010 R11). The exact
//! boundary is not prose: `the_replay_exclusion_is_the_frozen_baseline` below pins the
//! seventeen names that are NOT replayed, so the gap is a checked fact rather than a
//! claim. ⚠ The test NAME's "27" is a KNOWN STALE COUNT, deferred by ruling to the
//! successor coverage lane where the rename rides the real replay work. (SR-20: the consumer's real
//! emission, including `confirmPhrase`, `autolockMinutes`, `selfAlias`,
//! `contentHeight`). DTO wire shapes are pinned as serialized — the `kind`
//! strings the FE string-matches on.
//!
//! Claim boundary (R108, absolute): this harness does NOT click, type, or read
//! the interface; it closes the IPC half of the blindness. The rendered-DOM
//! driver is NA-0701.

use serde_json::{json, Value};

const PASS: &str = "na0700-replay-passphrase";

type MockWebview = tauri::WebviewWindow<tauri::test::MockRuntime>;

/// Drive one command through tauri's real IPC ingestion. `Ok` carries the
/// serialized response as JSON (the wire shape the FE receives); `Err` carries
/// the IPC rejection.
/// The 0700 half of `create_private_dir` (lib.rs) — the step this harness has to
/// perform for its replication of `bootstrap()` to be true. See the call sites below.
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
            // The webview's own origin, so the ACL's local-window scope match
            // is the one production performs.
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

/// NA-0776 (RULING_010 R10) -- THE REPLAY EXCLUSION, PINNED AS A SET EQUALITY.
///
/// SET EQUALITY OVER SEVENTEEN NAMES, never a count: a count would let membership
/// rotate silently. RED ON BOTH POLARITIES, and the shrinkage direction is DESIRED:
///   - an EIGHTEENTH unreplayed command reds this (the original purpose: a new command
///     cannot be added and left un-replayed without someone noticing);
///   - REPLAYING one of the sixteen ALSO reds this, until the list is updated -- which
///     turns a coverage change in EITHER direction into a visible, deliberate act.
///
/// Sixteen are INHERITED at base 83019356 -- they were never replayed, and the gap is
/// weighted to the contact and invite surfaces. Replaying them needs vault-state
/// fixtures and is outside this lane; it is FILED via the promotion as a desktop
/// coverage finding for a successor lane. `restart_app` is excluded BY RULING
/// (RULING_009): it calls `AppHandle::restart()`, which on the mock runtime is
/// `not implemented` and takes the test process down, so no in-process harness can
/// drive it.
#[test]
fn the_replay_exclusion_is_the_frozen_baseline() {
    /// INHERITED at base 83019356 -- not replayed before this lane, and not by it.
    const INHERITED: &[&str] = &[
        "connect_status",
        "contact_list",
        "contact_request_accept",
        "contact_request_block",
        "contact_request_ignore",
        "contact_requests",
        "contact_set_display_name",
        "home_dir",
        "invite_accept",
        "invite_clear",
        "invite_create",
        "invite_finish",
        "invite_list",
        "invite_redeem",
        "invite_revoke",
        "relay_probe",
    ];
    /// Excluded BY RULING (RULING_009 sec 1): terminates the process.
    const BY_RULING: &[&str] = &["restart_app"];

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib = std::fs::read_to_string(root.join("src/lib.rs")).expect("lib.rs");
    let block = &lib[lib.find("generate_handler!").expect("handler list")..];
    let block = &block[..block.find("])").expect("handler list end")];
    let registered: std::collections::BTreeSet<String> = block
        .lines()
        .filter_map(|l| {
            let t = l.trim().trim_end_matches(',');
            let t = t.strip_prefix("commands::").unwrap_or(t);
            (!t.is_empty() && t.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()))
                .then(|| t.to_string())
        })
        .collect();
    assert!(
        registered.len() >= 40,
        "only {} commands parsed from the handler list -- the needle drifted from its \
         shape, which would make this pin vacuous",
        registered.len()
    );

    // The invoked set, read from THIS file's own source with comments stripped so the
    // prose above cannot count as coverage, and with the two constant blocks removed so
    // the pin's own expected values cannot count either.
    let me = std::fs::read_to_string(root.join("tests/na0700_ipc_replay.rs")).expect("self");
    let me: String = me.lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
    let body = match (me.find("const INHERITED"), me.find("let root = std::path::PathBuf")) {
        (Some(a), Some(b)) if b > a => format!("{}{}", &me[..a], &me[b..]),
        _ => me.clone(),
    };
    let invoked: std::collections::BTreeSet<String> = registered
        .iter()
        .filter(|c| body.contains(&format!("\"{c}\"")))
        .cloned()
        .collect();

    let mut expected: Vec<String> =
        INHERITED.iter().chain(BY_RULING).map(|s| s.to_string()).collect();
    expected.sort();
    let mut actual: Vec<String> = registered.difference(&invoked).cloned().collect();
    actual.sort();

    assert_eq!(
        actual, expected,
        "THE REPLAY EXCLUSION SET MOVED. Seventeen commands are expected to be \
         un-replayed ({} inherited at base 83019356 + restart_app by ruling). A LONGER \
         list means a command was added and not replayed; a SHORTER one means coverage \
         GREW and this pin must be updated to record it -- both are deliberate acts, \
         which is why both red.",
        INHERITED.len()
    );
}

#[test]
fn all_27_registered_commands_invoke_through_real_ipc_with_fe_arg_shapes() {
    // Hermetic env, mirroring run()'s startup rule (a): isolated HOME/XDG so
    // `cli_vault_present` probes no real profile; QSLD_DATA_DIR into a
    // tempdir; bootstrap() fixes QSC_CONFIG_DIR + InApp routing + the
    // redacting output policy exactly as the shipped app does.
    let root = tempfile::tempdir().expect("tmp root");
    let home = root.path().join("home");
    std::fs::create_dir_all(home.join(".config")).expect("home");
    std::env::set_var("HOME", &home);
    std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
    std::env::set_var("XDG_DATA_HOME", home.join(".local/share"));
    std::env::set_var("QSC_DISABLE_KEYCHAIN", "1");
    let data_dir = root.path().join("qsld-data");
    std::env::set_var("QSLD_DATA_DIR", &data_dir);
    // bootstrap()'s steps minus install_panic_redaction_hook: the hook is
    // production behaviour (qsc-owned, tested there) and in a test binary it
    // redacts every assertion panic into `panic_redacted`, which destroys
    // diagnosability. Dirs, env, policy and routing are replicated exactly.
    //
    // ⚠ NA-0705 (D640, R190): the dirs are created at 0700 because that is what
    // `create_private_dir` (lib.rs) actually does — `create_dir_all` PLUS
    // `set_permissions(0o700)`. This harness previously used bare `create_dir_all`
    // while claiming to replicate bootstrap, which under a 002 umask yields 0775.
    // Latent at the old qsc pin; at the bumped pin the replaced vault path resolver
    // carries a real `ConfigSource` into `enforce_safe_parents`, which refuses a
    // group-writable config dir — `vault_create` failed with `unsafe_parent_perms`.
    // The product was right; the replication was not.
    create_private_dir_0700(&data_dir).expect("data dir");
    let qsc_dir = data_dir.join("qsc");
    create_private_dir_0700(&qsc_dir).expect("qsc dir");
    std::env::set_var("QSC_CONFIG_DIR", &qsc_dir);
    qsc::output::init_output_policy(false);
    qsc::output::set_marker_routing(qsc::output::MarkerRouting::InApp);

    // The run-path composition on the mock runtime: same managed state, same
    // generate_handler! set, via the one extracted composition point — built
    // with the REAL generate_context!(), because the compiled ACL
    // (capabilities/default.json) is part of the IPC boundary under test: the
    // mock context's empty capability set rejects every command with
    // "not allowed", which is the ACL working, not the commands failing.
    let app = qsl_desktop_app::configure_builder(
        tauri::test::mock_builder(),
        qsl_desktop_app::AppState {
            data_dir: data_dir.clone(),
            gw: qsl_desktop_app::gateway::CoreGateway::default(),
        },
    )
    .build(tauri::generate_context!())
    .expect("mock app builds with the run-path composition and real context");

    // Config-declared windows are a run()-phase creation, which never happens
    // under the mock runtime — the harness builds the `main` webview itself,
    // under the SAME label the ACL's capability set names.
    let wv: MockWebview =
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::default())
            .build()
            .expect("mock main webview");

    // ---- fresh profile ----------------------------------------------------
    let info = ok(&wv, "app_info", Value::Null);
    assert_eq!(info["display_name"], "QuantumShield Chat");
    for k in ["display_name", "version", "slice"] {
        assert!(info.get(k).is_some(), "AppInfoDto wire key `{k}`");
    }

    assert_eq!(ok(&wv, "launch_state", Value::Null), json!("s0"));
    assert_eq!(ok(&wv, "cli_vault_present", Value::Null), json!(false));

    // ---- vault lifecycle 1 (main.js:278 keys: passphrase + confirm) -------
    ok(
        &wv,
        "vault_create",
        json!({"passphrase": PASS, "confirm": PASS}),
    );

    let ident = ok(&wv, "identity_ensure", Value::Null);
    for k in [
        "fingerprint",
        "verify_code",
        "purpose_line",
        "pq_line",
        "mechanism_line",
    ] {
        assert!(ident.get(k).is_some(), "IdentityDto wire key `{k}`");
    }
    assert!(ident["fingerprint"].as_str().is_some_and(|s| !s.is_empty()));

    let shown = ok(&wv, "identity_show", Value::Null);
    assert_eq!(shown["fingerprint"], ident["fingerprint"]);

    // The InApp drain path is live in this composition: identity emissions
    // landed in the queue and the gateway drained them into the buffer.
    let ms = ok(&wv, "marker_stats", Value::Null);
    assert!(ms.get("buffered").is_some() && ms.get("dropped").is_some());
    assert!(
        ms["buffered"].as_u64().unwrap_or(0) >= 1,
        "identity emissions should have been drained into the marker buffer, got {ms}"
    );
    assert_eq!(ok(&wv, "core_busy", Value::Null), json!(false));

    // NA-0776 (3.3): the notice surface joins the replayed set. `notice_list` takes no
    // args and returns an ARRAY of {kind, count}; `notice_dismiss` takes the FE's own
    // `kind` key. Empty here is the honest state: no marker has been drained on the
    // mock runtime, so the surface has nothing to show.
    let notices = ok(&wv, "notice_list", Value::Null);
    assert!(notices.as_array().is_some(), "notice_list must return an array");
    assert_eq!(notices.as_array().unwrap().len(), 0);
    ok(&wv, "notice_dismiss", json!({"kind": "invite_finish_hs_unconsumed"}));
    // a kind outside the whitelist is accepted and ignored, never an IPC error
    ok(&wv, "notice_dismiss", json!({"kind": "not_a_whitelisted_kind"}));

    // identity written, settings.json absent → s1; writing settings → s2.
    assert_eq!(ok(&wv, "launch_state", Value::Null), json!("s1"));
    let s = ok(&wv, "settings_get", Value::Null);
    assert_eq!(s["autolock_minutes"], 60);
    // main.js:206 keys: autolockMinutes + selfAlias (the camelCase boundary).
    ok(
        &wv,
        "settings_set",
        json!({"autolockMinutes": 30, "selfAlias": "Vic"}),
    );
    let s2 = ok(&wv, "settings_get", Value::Null);
    assert_eq!(s2["autolock_minutes"], 30);
    assert_eq!(s2["self_alias"], "Vic");
    assert_eq!(ok(&wv, "launch_state", Value::Null), json!("s2"));

    // ---- protection group -------------------------------------------------
    let p = ok(&wv, "protection_status", Value::Null);
    for k in [
        "failed_unlocks",
        "wipe_after",
        "retry_after_s",
        "locked",
        "wipe_min",
        "wipe_max",
    ] {
        assert!(p.get(k).is_some(), "ProtectionDto wire key `{k}`");
    }
    assert_eq!(p["locked"], false);
    // Pick the arm limit from the DTO's own bounds (first-party acquisition).
    let wipe_min = p["wipe_min"].as_u64().expect("wipe_min");
    ok(&wv, "wipe_arm", json!({"limit": wipe_min}));
    ok(&wv, "wipe_disarm", Value::Null);

    // ---- relay group (unlocked vault holds token + CA) --------------------
    let rc = ok(&wv, "relay_config_get", Value::Null);
    assert_eq!(rc["relay_url"], "");
    // main.js:1029 key: url
    ok(
        &wv,
        "relay_config_set",
        json!({"url": "http://127.0.0.1:9"}),
    );
    let rc2 = ok(&wv, "relay_config_get", Value::Null);
    assert!(rc2["relay_url"]
        .as_str()
        .is_some_and(|u| u.contains("127.0.0.1")));

    ok(
        &wv,
        "relay_token_set",
        json!({"token": "na0700-replay-bearer-token"}),
    );
    assert_eq!(
        ok(&wv, "relay_token_show", Value::Null),
        json!({"configured": true})
    );
    ok(&wv, "relay_token_clear", Value::Null);
    assert_eq!(
        ok(&wv, "relay_token_show", Value::Null),
        json!({"configured": false})
    );

    let ca = root.path().join("na0700-test-ca.pem");
    std::fs::write(
        &ca,
        b"-----BEGIN CERTIFICATE-----\nMA==\n-----END CERTIFICATE-----\n",
    )
    .expect("ca file");
    ok(
        &wv,
        "relay_ca_file_set",
        json!({"path": ca.to_str().expect("ca path")}),
    );
    let cs = ok(&wv, "relay_ca_file_show", Value::Null);
    assert_eq!(cs["configured"], true);
    assert!(cs["path_hash"].as_str().is_some());
    ok(&wv, "relay_ca_file_clear", Value::Null);

    // relay_test against a closed loopback port — the pre-classified outcome
    // arrives as the tagged DTO the FE string-matches: kind=unreachable.
    let rt = ok(&wv, "relay_test", json!({"url": "http://127.0.0.1:9"}));
    assert_eq!(rt["kind"], "unreachable", "RelayTestDto: {rt}");

    // ---- presentation (main.js:77 keys: surface + contentHeight) ----------
    let ui = ok(
        &wv,
        "ui_surface_changed",
        json!({"surface": "scr-main", "contentHeight": 400.0}),
    );
    assert_eq!(ui, Value::Null);

    // ---- lock / unlock (main.js:452 key: passphrase) ----------------------
    ok(&wv, "lock_now", Value::Null);
    let u = ok(&wv, "unlock_attempt", json!({"passphrase": PASS}));
    assert_eq!(u["kind"], "unlocked", "UnlockDto: {u}");

    // ---- destroy (main.js:811 keys: passphrase + confirmPhrase) -----------
    ok(
        &wv,
        "destroy_vault",
        json!({"passphrase": PASS, "confirmPhrase": "destroy my vault"}),
    );
    assert_eq!(ok(&wv, "launch_state", Value::Null), json!("s0"));

    // ---- vault lifecycle 2: the rejected variant + erase ------------------
    ok(
        &wv,
        "vault_create",
        json!({"passphrase": PASS, "confirm": PASS}),
    );
    let rej = ok(
        &wv,
        "unlock_attempt",
        json!({"passphrase": "not-the-passphrase"}),
    );
    assert_eq!(rej["kind"], "rejected", "UnlockDto: {rej}");
    assert!(rej.get("failed_unlocks").is_some() && rej.get("retry_after_s").is_some());
    // main.js:542 key: confirmPhrase — the app-level erase works regardless of
    // lock state by design.
    ok(
        &wv,
        "erase_all",
        json!({"confirmPhrase": "erase everything"}),
    );
    assert_eq!(ok(&wv, "launch_state", Value::Null), json!("s0"));

    // ---- the boundary can go red: a missing required arg is REJECTED ------
    let missing = invoke(&wv, "relay_config_set", json!({}));
    assert!(
        missing.is_err(),
        "IPC ingestion accepted relay_config_set without its `url` arg — the \
         arg-mapping instrument cannot be trusted if this passes"
    );
}

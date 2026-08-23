//! NA-0751 (D-0032) — THE SLICE-4 GATEWAY SURFACE, THROUGH REAL IPC.
//!
//! The twelve new `qsc::facade` pass-through commands are driven through tauri's
//! REAL IPC ingestion on the mock runtime — real serde arg decoding, real
//! camelCase→snake_case mapping, real State injection, the REAL generated context
//! so the compiled ACL is the one production enforces — exactly the NA-0700
//! pattern. DTO wire keys are pinned as SERIALIZED, because the wire shape is what
//! a screen consumes, not the Rust type.
//!
//! ⚠ THE ERROR SET IS THIRTY-NINE, NOT TWENTY-SEVEN. `FacadeError::Store` fans out
//! over `ErrorCode::as_str`, so the pinned discriminant set is 26 non-`Store`
//! variants + 13 `Store` codes. (38 → 39 at NA-0755 v2: `clear_refused` joined.) One of them, `lock_upgrade_refused`, is the code
//! the `Store` variant exists to keep reachable; a collapse of `Store` to one
//! discriminant would make it unreachable to a GUI, and this file asserts it
//! survives all the way to the DTO the front end receives.
//!
//! Claim boundary: this harness closes the IPC half. It does not click, type or
//! read the interface — that is the gui_driver harness.

use serde_json::{json, Value};

type MockWebview = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn create_private_dir(p: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(p)?;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o700))
}

fn invoke(wv: &MockWebview, cmd: &str, args: Value) -> Result<Value, Value> {
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
        Err(e) => Err(e),
    }
}

fn ok(wv: &MockWebview, cmd: &str, args: Value) -> Value {
    match invoke(wv, cmd, args) {
        Ok(v) => v,
        Err(e) => panic!("command `{cmd}` rejected at the IPC boundary: {e}"),
    }
}

/// The typed-failure arm: the command REACHED the facade and came back with an
/// `ErrorDto`. An IPC-layer rejection ("not allowed", arg-decode failure) is a
/// DIFFERENT outcome and must not be mistaken for one — it comes back without a
/// `code` key, and this asserts the key is there.
fn err_code(wv: &MockWebview, cmd: &str, args: Value) -> String {
    match invoke(wv, cmd, args) {
        Ok(v) => panic!("command `{cmd}` unexpectedly SUCCEEDED: {v}"),
        Err(v) => {
            v.get("code")
                .and_then(|c| c.as_str())
                .unwrap_or_else(|| {
                    panic!("`{cmd}` failed WITHOUT an ErrorDto `code` key — that is an IPC-layer rejection, not a typed facade failure: {v}")
                })
                .to_string()
        }
    }
}

struct Rig {
    _root: tempfile::TempDir,
    app: tauri::App<tauri::test::MockRuntime>,
}

impl Rig {
    fn new() -> Rig {
        let root = tempfile::tempdir().expect("tmp root");
        let home = root.path().join("home");
        std::fs::create_dir_all(home.join(".config")).expect("home");
        std::env::set_var("HOME", &home);
        std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
        std::env::set_var("XDG_DATA_HOME", home.join(".local/share"));
        std::env::set_var("QSC_DISABLE_KEYCHAIN", "1");
        let data_dir = root.path().join("qsld-data");
        std::env::set_var("QSLD_DATA_DIR", &data_dir);
        create_private_dir(&data_dir).expect("data dir");
        let qsc_dir = data_dir.join("qsc");
        create_private_dir(&qsc_dir).expect("qsc dir");
        std::env::set_var("QSC_CONFIG_DIR", &qsc_dir);
        qsc::output::init_output_policy(false);
        qsc::output::set_marker_routing(qsc::output::MarkerRouting::InApp);
        let app = qsl_desktop_app::configure_builder(
            tauri::test::mock_builder(),
            qsl_desktop_app::AppState {
                data_dir,
                gw: qsl_desktop_app::gateway::CoreGateway::default(),
            },
        )
        .build(tauri::generate_context!())
        .expect("mock app builds with the run-path composition and real context");
        Rig { _root: root, app }
    }

    fn webview(&self) -> MockWebview {
        tauri::WebviewWindowBuilder::new(&self.app, "main", tauri::WebviewUrl::default())
            .build()
            .expect("mock main webview")
    }
}

/// The twelve commands and the arg shapes a front end really emits (camelCase at
/// the wire, which tauri maps to the snake_case parameters).
fn the_twelve() -> Vec<(&'static str, Value)> {
    vec![
        ("connect_status", json!({"peer": "nobody"})),
        ("contact_list", Value::Null),
        ("contact_requests", Value::Null),
        ("contact_request_accept", json!({"alias": "nobody"})),
        ("contact_request_ignore", json!({"alias": "nobody"})),
        ("contact_request_block", json!({"alias": "nobody"})),
        ("invite_list", Value::Null),
        (
            "invite_create",
            // ⚠ `recipientLabel` is APPENDED LAST, matching the order the front end emits and
            // the order `design_polish.rs` pins. Inserting it between `selfLabel` and `relay`
            // would red that pin — a pin whose outcome depends on where a key is typed.
            json!({"selfLabel": null, "relay": "http://127.0.0.1:9", "ttlSecs": 3600,
                   "recipientLabel": null}),
        ),
        (
            "invite_redeem",
            json!({"code": "not-a-code", "alias": "nobody", "selfLabel": null}),
        ),
        (
            "invite_accept",
            json!({"selfLabel": null, "inviteId": "nope", "alias": "nobody", "max": 4}),
        ),
        (
            "invite_finish",
            json!({"selfLabel": null, "alias": "nobody", "relay": "http://127.0.0.1:9", "max": 4}),
        ),
        ("invite_revoke", json!({"inviteId": "nope"})),
        ("invite_clear", json!({"inviteId": "nope"})),
    ]
}

/// EVERY new command reaches the facade through real IPC and returns a TYPED
/// failure on a locked vault.
///
/// This is the registration + ACL + arg-decoding seal in one: a command missing
/// from `generate_handler!`, or refused by the compiled capability set, or given
/// an arg shape the front end does not emit, fails here with an IPC rejection —
/// which `err_code` distinguishes from a typed `ErrorDto` by name.
///
/// ⚠ `connect_status` is deliberately NOT in this list: it returns a STATUS, never
/// a `Result::Err`, on a locked vault — that is its `VaultLocked` reason, and it is
/// asserted separately below. Twelve verbs gate on the lock; one reports it.
///
/// ⚠ **THE NAME IS COUNT-NEUTRAL ON PURPOSE** (SR-15 **M-5**). The previous name carried
/// "twelve" in the identifier, so growing the surface forced a rename — and a rename is a
/// DISAPPEARANCE to the CI-enforced test inventory. The count now lives in an assertion, where
/// it can move without dragging a CI gate behind it.
#[test]
fn every_gateway_command_reaches_the_facade_through_real_ipc() {
    let rig = Rig::new();
    let wv = rig.webview();

    // The lock-gated eleven.
    let mut seen: Vec<(String, String)> = Vec::new();
    for (cmd, args) in the_twelve() {
        if cmd == "connect_status" {
            continue;
        }
        let code = err_code(&wv, cmd, args);
        seen.push((cmd.to_string(), code));
    }
    assert_eq!(seen.len(), 12, "twelve lock-gated verbs were driven");

    // Every one refuses for the LOCK, by the facade's own vocabulary — not by an
    // IPC accident and not by a different cause that would also produce an Err.
    for (cmd, code) in &seen {
        assert!(
            code == "locked" || code == "vault_unavailable",
            "`{cmd}` should refuse a locked vault with the lock's own code, got `{code}`"
        );
    }

    // The twelfth reports the lock as a STATUS with both wire keys present.
    let s = ok(&wv, "connect_status", json!({"peer": "nobody"}));
    for k in ["state", "reason"] {
        assert!(s.get(k).is_some(), "ConnectStatusDto wire key `{k}`");
    }
    assert_eq!(s["state"], json!("inactive"));

    // ⚠ THE LOCK DOES NOT SHADOW AN EARLIER FACT, and that is the property worth
    // sealing here. A first draft of this test asserted `vault_locked` and MISSED:
    // the rig is a fresh profile, so the tuple answers `missing_seed` — decided
    // before any vault secret is touched, and NOT cured by unlocking. The facade's
    // own doc states the rule ("ORDER IS LOAD-BEARING, and it is NOT 'check the
    // lock first' … reporting VaultLocked would SHADOW the operative fact"), so the
    // facade was right and the expectation was wrong.
    //
    // Asserting the un-shadowed fact is STRONGER evidence for the ordering rule
    // than asserting the override would have been: a facade that checked the lock
    // first would return `vault_locked` here and go RED.
    assert_eq!(
        s["reason"],
        json!("missing_seed"),
        "a locked fresh profile reports the OPERATIVE fact, not the lock — a \
         `vault_locked` here would mean the lock had shadowed it"
    );

    // The override arm itself (`session_invalid` ∧ locked -> `vault_locked`) is
    // sealed PROTOCOL-side across two binaries with `count == 7` over the union;
    // reaching it needs a session blob written under the real vault key, which this
    // pass-through harness deliberately does not fabricate.
}

/// The `ErrorDto` a failing command puts on the wire carries the facade's stable
/// discriminant, and `detail` is absent for THIS named variant.
///
/// ⚠⚠ **THE DOC AND THE ASSERTION MESSAGE WERE BOTH FALSIFIED BY NA-0755 v2 AND ARE
/// CORRECTED HERE** (SR-15 **M-4**). They said *"`detail` is absent for a named variant"* and
/// *"a NAMED variant carries no detail"* — a FALSE GENERALITY now that `VaultUnavailable`
/// carries its source code. The test kept passing either way, because it drives
/// `invite_revoke` on a locked vault, which returns `Locked` via `require_unlocked_here` and
/// never reaches the widened arm. **A green test asserting a falsehood is a negative-value
/// instrument** — it would have gone on certifying the old rule forever.
#[test]
fn the_error_dto_wire_shape_is_the_stable_discriminant_not_a_debug_rendering() {
    let rig = Rig::new();
    let wv = rig.webview();
    let v = invoke(&wv, "invite_revoke", json!({"inviteId": "nope"}))
        .expect_err("a locked vault refuses invite_revoke");
    for k in ["code", "detail"] {
        assert!(v.get(k).is_some(), "ErrorDto wire key `{k}` is present");
    }
    let code = v["code"].as_str().expect("code is a string");
    assert!(
        code.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
        "the discriminant is a snake_case wire token, not a Debug rendering: `{code}`"
    );
    assert!(
        !code.contains("Locked") && !code.contains('('),
        "`{code}` looks like `{{e:?}}` — the Debug form must NOT reach the wire"
    );
    assert_eq!(
        v["detail"],
        Value::Null,
        "`Locked` carries no detail — scoped to THIS variant, not to named variants in general: \
         `VaultUnavailable` carries its source code since NA-0755 v2"
    );
}

/// THE THIRTY-NINE.
///
/// Every one of the facade's discriminants survives the desktop's `ErrorDto`
/// conversion, all distinct, with `lock_upgrade_refused` among them.
#[test]
fn the_pinned_discriminant_set_survives_the_dto_at_its_asserted_size() {
    use qsc::facade::FacadeError as E;
    use qsc::model::ErrorCode as C;

    let singles = [
        E::Locked,
        E::VaultUnavailable(None),
        E::Expired,
        E::AlreadyRedeemed,
        E::RevokedLocally,
        E::SoftCapReached,
        E::Malformed,
        E::NotFound,
        E::Revoked,
        E::ExpiredAtRelay,
        E::AlreadyUsed,
        E::RateLimited,
        E::RelaySlotsFull,
        E::RelayRejected,
        E::RevokeInvalid,
        E::RelayUnauthorized,
        E::CommitmentMismatch,
        E::SignatureInvalid,
        E::EnvelopeMalformed,
        E::EnvelopeVersionSkew,
        E::RelayTlsUntrusted,
        E::RelayCaFile,
        E::RelayEndpointInvalid,
        E::StoreUnavailable,
        E::InviteClearRefused,
        E::Other(String::new()),
    ];
    assert_eq!(singles.len(), 26, "26 non-Store variants");

    let store_codes = [
        C::MissingHome,
        C::InvalidPolicyProfile,
        C::UnsafePathSymlink,
        C::UnsafeParentPerms,
        C::LockOpenFailed,
        C::LockContended,
        C::LockFailed,
        C::LockUpgradeRefused,
        C::IoWriteFailed,
        C::IoReadFailed,
        C::ParseFailed,
        C::IdentitySecretUnavailable,
        C::IdentitySelfAmbiguous,
    ];
    assert_eq!(store_codes.len(), 13, "13 Store codes");

    // Through the DESKTOP's conversion — the boundary under test, not the facade's
    // own `as_wire`, which the protocol side already seals.
    let mut wire: Vec<String> = singles
        .into_iter()
        .map(|e| qsl_desktop_app::commands::ErrorDto::from(e).code)
        .collect();
    for c in store_codes {
        wire.push(qsl_desktop_app::commands::ErrorDto::from(E::Store(c)).code);
    }

    assert_eq!(wire.len(), 39, "the pinned discriminant set is 39, not 27");
    let mut sorted = wire.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        39,
        "all 39 discriminants are DISTINCT at the DTO"
    );

    assert!(
        wire.iter().any(|w| w == "lock_upgrade_refused"),
        "`lock_upgrade_refused` — the code `Store` exists to keep reachable — must \
         survive to the boundary a GUI reads"
    );

    // MUTATION CONTROL: rename ONE discriminant and the comparison must go RED.
    // Without this, a set-equality assertion that could never fail would prove
    // nothing about the set it claims to pin.
    let mut mutated = wire.clone();
    let i = mutated
        .iter()
        .position(|w| w == "lock_upgrade_refused")
        .expect("the code to mutate is present");
    mutated[i] = "lock_upgrade_refused_RENAMED".to_string();
    assert_ne!(
        mutated, wire,
        "the control must DIFFER from the truth, or it discriminates nothing"
    );
    assert!(
        !mutated.iter().any(|w| w == "lock_upgrade_refused"),
        "under the mutation the pinned code is ABSENT — so the assertion above \
         would have gone RED, which is what makes it evidence"
    );
}

/// `FacadeError::Other` carries a payload and the DTO carries it — the residual stays
/// diagnosable instead of collapsing to a bare code.
///
/// ⚠⚠ **RENAMED AND RE-DOCUMENTED AT NA-0755 v2** (SR-15 **M-4**). The old name and doc said
/// `Other` was **the ONE** variant with a payload and that the named ones have none — FALSE
/// since `VaultUnavailable` became self-diagnosing, and **false while staying green**, because
/// this test drives `Locked` and never reaches the widened arm. The claim is now scoped to what
/// it actually exercises, and the widened arm has its own coverage.
#[test]
fn the_residual_variant_carries_its_payload_through_the_dto() {
    use qsc::facade::FacadeError as E;
    let other = qsl_desktop_app::commands::ErrorDto::from(E::Other("upstream detail".into()));
    assert_eq!(other.code, "other");
    assert_eq!(other.detail.as_deref(), Some("upstream detail"));

    let named = qsl_desktop_app::commands::ErrorDto::from(E::Locked);
    assert_eq!(named.code, "locked");
    assert_eq!(
        named.detail, None,
        "a named variant has no payload to carry"
    );
}

/// LIVENESS CONTROL for the registration seal above.
///
/// `all_twelve_gateway_commands_reach_the_facade_through_real_ipc` proves the twelve
/// are registered by requiring a typed `ErrorDto` back. That is evidence ONLY if the
/// same instrument would FAIL for a command that is not registered — otherwise a
/// wholesale ACL refusal or a missing `generate_handler!` entry could pass unnoticed.
///
/// Here an unregistered name is driven through the identical path: it must come back
/// as an IPC rejection WITHOUT a `code` key, which is exactly the branch `err_code`
/// panics on. The seal discriminates.
#[test]
fn an_unregistered_command_is_an_ipc_rejection_not_a_typed_failure() {
    let rig = Rig::new();
    let wv = rig.webview();

    let v = invoke(&wv, "no_such_gateway_command", Value::Null)
        .expect_err("an unregistered command cannot succeed");
    assert!(
        v.get("code").is_none(),
        "an IPC rejection carries no ErrorDto `code` — if it did, the registration \
         seal could not tell a missing command from a typed facade failure: {v}"
    );

    // And the positive half in the same breath: a REGISTERED command on the same
    // webview does come back with a `code`. Both arms, one rig, so the difference
    // is the registration and nothing else.
    let reg = invoke(&wv, "invite_revoke", json!({"inviteId": "nope"}))
        .expect_err("a locked vault refuses invite_revoke");
    assert!(
        reg.get("code").is_some(),
        "a registered command returns a typed ErrorDto: {reg}"
    );
}

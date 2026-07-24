//! The Tauri command surface. Every qsc call goes through the CoreGateway
//! (rules b and d); lock state is touched ONLY through the NA-0658 one-call
//! surface (rule c). Slice A: zero networking commands exist.

use crate::paths;
use crate::settings::{self, AppSettings};
use crate::state::{resolve_launch_state, SELF_LABEL};
use crate::AppState;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::State;

/// The two deliberate typed phrases. The forgotten-passphrase erase is
/// app-level file removal ONLY and must never masquerade as the tokened core
/// destroy; each has its own distinct phrase.
pub const ERASE_CONFIRM_PHRASE: &str = "erase everything";
pub const DESTROY_CONFIRM_PHRASE: &str = "destroy my vault";

/// The user-facing display name (D596 item 6): window title + About ONLY.
/// The identifier, productName, binary name, and repo name never change.
pub const APP_DISPLAY_NAME: &str = "QuantumShield Chat";

/// D596 Appendix A copy (claim-discipline binding; no assurance adjectives).
/// The plain-English PQ line is the visible copy; the mechanism naming lives
/// behind the "Show technical details" disclosure.
pub const PQ_LINE: &str = "Designed to stay secure even against future quantum computers.";
pub const MECHANISM_LINE: &str =
    "Post-quantum hybrid: ML-KEM-768 (key agreement) + ML-DSA-65 (signatures)";
pub const VERIFY_PURPOSE_LINE: &str = "Verification codes exist so you and a contact can \
     confirm you're really talking to each other — they catch man-in-the-middle substitution.";

#[derive(Serialize)]
pub struct IdentityDto {
    pub fingerprint: String,
    pub verify_code: String,
    pub purpose_line: &'static str,
    pub pq_line: &'static str,
    pub mechanism_line: &'static str,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnlockDto {
    Unlocked,
    Rejected {
        failed_unlocks: u32,
        retry_after_s: u64,
    },
    Delayed {
        failed_unlocks: u32,
        retry_after_s: u64,
    },
    Wiped,
}

#[derive(Serialize)]
pub struct ProtectionDto {
    pub failed_unlocks: u32,
    pub wipe_after: Option<u32>,
    pub retry_after_s: u64,
    pub locked: bool,
    pub wipe_min: u32,
    pub wipe_max: u32,
}

#[derive(Serialize)]
pub struct MarkerStatsDto {
    pub buffered: usize,
    pub dropped: u64,
}

#[derive(Serialize)]
pub struct AppInfoDto {
    pub display_name: &'static str,
    pub version: &'static str,
    pub slice: &'static str,
}

fn identity_dto(rec: &qsc::identity::IdentityPublicRecord) -> IdentityDto {
    let fp = qsc::identity::identity_fingerprint_from_identity(&rec.kem_pk, &rec.sig_pk);
    let code = qsc::identity::format_verification_code_from_fingerprint(&fp);
    IdentityDto {
        fingerprint: fp,
        verify_code: code,
        purpose_line: VERIFY_PURPOSE_LINE,
        pq_line: PQ_LINE,
        mechanism_line: MECHANISM_LINE,
    }
}

#[tauri::command]
pub async fn launch_state(st: State<'_, AppState>) -> Result<String, String> {
    let data = st.data_dir.clone();
    let s = st.gw.call(move || resolve_launch_state(&data)).await;
    Ok(s.as_str().to_string())
}

#[tauri::command]
pub fn cli_vault_present() -> bool {
    paths::cli_vault_present()
}

#[tauri::command]
pub async fn vault_create(
    st: State<'_, AppState>,
    passphrase: String,
    confirm: String,
) -> Result<(), String> {
    if passphrase.is_empty() {
        return Err("empty_passphrase".into());
    }
    if passphrase != confirm {
        return Err("mismatch".into());
    }
    st.gw
        .call(move || -> Result<(), String> {
            qsc::vault::vault_init_with_passphrase(&passphrase).map_err(|e| e.to_string())?;
            match qsc::vault::protection::unlock_guarded(&passphrase).map_err(|e| e.to_string())? {
                qsc::vault::protection::GuardedUnlockOutcome::Unlocked => Ok(()),
                other => Err(format!("post_init_unlock_unexpected:{other:?}")),
            }
        })
        .await
}

#[tauri::command]
pub async fn identity_ensure(st: State<'_, AppState>) -> Result<IdentityDto, String> {
    st.gw
        .call(move || {
            let rec = qsc::identity::identity_ensure(SELF_LABEL).map_err(|e| format!("{e:?}"))?;
            Ok(identity_dto(&rec))
        })
        .await
}

#[tauri::command]
pub async fn identity_show(st: State<'_, AppState>) -> Result<Option<IdentityDto>, String> {
    st.gw
        .call(move || {
            let rec = qsc::identity::identity_read_self_public(SELF_LABEL)
                .map_err(|e| format!("{e:?}"))?;
            Ok(rec.map(|r| identity_dto(&r)))
        })
        .await
}

#[tauri::command]
pub async fn unlock_attempt(
    st: State<'_, AppState>,
    passphrase: String,
) -> Result<UnlockDto, String> {
    st.gw
        .call(move || {
            use qsc::vault::protection::GuardedUnlockOutcome as O;
            match qsc::vault::protection::unlock_guarded(&passphrase).map_err(|e| e.to_string())? {
                O::Unlocked => Ok(UnlockDto::Unlocked),
                O::Rejected {
                    failed_unlocks,
                    retry_after_s,
                } => Ok(UnlockDto::Rejected {
                    failed_unlocks,
                    retry_after_s,
                }),
                O::Delayed {
                    failed_unlocks,
                    retry_after_s,
                } => Ok(UnlockDto::Delayed {
                    failed_unlocks,
                    retry_after_s,
                }),
                O::Wiped { .. } => Ok(UnlockDto::Wiped),
            }
        })
        .await
}

#[tauri::command]
pub async fn lock_now(st: State<'_, AppState>) -> Result<(), String> {
    st.gw.call(|| qsc::vault::protection::lock(None)).await;
    Ok(())
}

#[tauri::command]
pub async fn protection_status(st: State<'_, AppState>) -> Result<ProtectionDto, String> {
    st.gw
        .call(move || {
            let s = qsc::vault::protection::protection_status().map_err(|e| e.to_string())?;
            Ok(ProtectionDto {
                failed_unlocks: s.failed_unlocks,
                wipe_after: s.wipe_after,
                retry_after_s: s.retry_after_s,
                locked: !qsc::vault_unlocked(),
                wipe_min: qsc::store::VAULT_ATTEMPT_LIMIT_MIN,
                wipe_max: qsc::store::VAULT_ATTEMPT_LIMIT_MAX,
            })
        })
        .await
}

#[tauri::command]
pub async fn wipe_arm(st: State<'_, AppState>, limit: u32) -> Result<(), String> {
    if !(qsc::store::VAULT_ATTEMPT_LIMIT_MIN..=qsc::store::VAULT_ATTEMPT_LIMIT_MAX).contains(&limit)
    {
        return Err("wipe_limit_out_of_bounds".into());
    }
    st.gw
        .call(move || {
            qsc::vault::protection::wipe_after_failed_unlocks_arm(limit).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub async fn wipe_disarm(st: State<'_, AppState>) -> Result<(), String> {
    st.gw
        .call(|| {
            qsc::vault::protection::wipe_after_failed_unlocks_disarm().map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub fn settings_get(st: State<'_, AppState>) -> AppSettings {
    settings::load(&st.data_dir)
}

#[tauri::command]
pub fn settings_set(
    st: State<'_, AppState>,
    autolock_minutes: u32,
    self_alias: String,
) -> Result<(), String> {
    // Load-mutate-save so the slice-B relay_url (and any future key) survives an
    // autolock/alias save — settings_set owns ONLY these two fields.
    let mut s = settings::load(&st.data_dir);
    s.autolock_minutes = autolock_minutes;
    s.self_alias = self_alias.trim().to_string();
    settings::save(&st.data_dir, &s)
}

#[tauri::command]
pub async fn destroy_vault(
    st: State<'_, AppState>,
    passphrase: String,
    confirm_phrase: String,
) -> Result<(), String> {
    if confirm_phrase != DESTROY_CONFIRM_PHRASE {
        return Err("confirm_phrase_mismatch".into());
    }
    st.gw
        .call(move || {
            let token =
                qsc::vault::protection::DestroyConfirmToken::confirm_with_passphrase(&passphrase);
            qsc::vault::protection::destroy_with_passphrase(&passphrase, token)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub async fn erase_all(st: State<'_, AppState>, confirm_phrase: String) -> Result<(), String> {
    if confirm_phrase != ERASE_CONFIRM_PHRASE {
        return Err("confirm_phrase_mismatch".into());
    }
    let data = st.data_dir.clone();
    st.gw.call(move || erase_all_impl(&data)).await
}

/// The forgotten-passphrase escape (D595): app-level removal of the
/// app-scoped data ONLY. Without the passphrase the vault is already
/// permanent ciphertext, so honest erasure is the only remedy. This function
/// never touches the CLI's profile (guarded) and never calls the tokened
/// core destroy (that API is passphrase-committed and serves the opposite
/// case: a user who KNOWS the passphrase).
pub fn erase_all_impl(data_dir: &Path) -> Result<(), String> {
    let qsc_dir = paths::qsc_config_dir(data_dir);
    if let Some(cli) = paths::cli_default_config_dir() {
        let cli_canon = cli.canonicalize().ok();
        for candidate in [data_dir.canonicalize().ok(), qsc_dir.canonicalize().ok()] {
            if candidate.is_some() && candidate == cli_canon {
                return Err("erase_refused_cli_dir".into());
            }
        }
    }
    qsc::vault::protection::lock(None);
    if qsc_dir.exists() {
        fs::remove_dir_all(&qsc_dir).map_err(|e| e.to_string())?;
    }
    let sf = paths::settings_file(data_dir);
    if sf.exists() {
        fs::remove_file(&sf).map_err(|e| e.to_string())?;
    }
    crate::create_private_dir(&qsc_dir)?;
    Ok(())
}

#[tauri::command]
pub fn marker_stats(st: State<'_, AppState>) -> MarkerStatsDto {
    let (buffered, dropped) = st.gw.markers.stats();
    MarkerStatsDto { buffered, dropped }
}

#[tauri::command]
pub fn core_busy(st: State<'_, AppState>) -> bool {
    st.gw.busy()
}

#[tauri::command]
pub fn app_info() -> AppInfoDto {
    AppInfoDto {
        display_name: APP_DISPLAY_NAME,
        version: env!("CARGO_PKG_VERSION"),
        slice: "B (server connectivity: point the app at a relay and test the connection)",
    }
}

// ===========================================================================
// GUI slice B — server connectivity (D609 GATE 2).
//
// Thin forwarders onto the qsc surface NA-0672 shipped. ⚠ R1: EVERY qsc call
// runs inside `st.gw.call(...)` on the serial blocking gate — qsc's blocking
// HTTP client PANICS if constructed in an async context, which is exactly what
// the gate exists to prevent. NONE of these construct an HTTP client or touch
// `relay_server_info_from_parts`: the probe is called WHOLE, and the already-
// classified outcome is mapped to a serde DTO here (rendering, not re-
// classifying) — the relay taxonomy lives in qsc and is re-derived nowhere.
// ===========================================================================

/// The flattened server-info document rendered by the pane's "Connected"
/// state — mirrors `qsc::transport::ServerInfoDoc` minus `auth_mode` (carried
/// on the outcome). The pane renders the REAL fields; the mockup values are
/// placeholders.
#[derive(Serialize)]
pub struct ServerInfoDocDto {
    pub name: String,
    pub version: String,
    pub api: Vec<String>,
    pub max_body_bytes: u64,
    pub max_queue_depth: u64,
    pub retention_ttl_secs: u64,
    pub directory_mode: String,
    pub attachments_service_url: Option<String>,
    pub kt_mode: String,
    pub min_client_version: Option<String>,
}

/// A 1:1 rendering of `qsc::transport::RelayServerInfoOutcome` for the FE —
/// NOT a re-classification (R1). `auth_mode` is "open" | "bearer".
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelayTestDto {
    // `doc` is boxed so the Reachable variant is not far larger than the
    // others (clippy::large_enum_variant); serde serializes Box<T> transparently.
    Reachable {
        auth_mode: String,
        doc: Box<ServerInfoDocDto>,
    },
    AuthRequired {
        token_was_sent: bool,
    },
    NotAQslRelay,
    CertNotTrusted,
    Unreachable,
}

#[derive(Serialize)]
pub struct RelayConfigDto {
    pub relay_url: String,
}

/// Token presence ONLY — a bare bool (FLAG-3: a token is secret, no hash).
#[derive(Serialize)]
pub struct RelayTokenStatusDto {
    pub configured: bool,
}

/// CA-file presence + a redacted path hash — the path is PUBLIC material, so a
/// hash is acceptable (the deliberate asymmetry with the bare-bool token).
#[derive(Serialize)]
pub struct RelayCaStatusDto {
    pub configured: bool,
    pub path_hash: Option<String>,
}

fn relay_auth_mode_str(m: qsc::transport::RelayAuthMode) -> &'static str {
    match m {
        qsc::transport::RelayAuthMode::Open => "open",
        qsc::transport::RelayAuthMode::Bearer => "bearer",
    }
}

fn relay_test_dto(outcome: qsc::transport::RelayServerInfoOutcome) -> RelayTestDto {
    use qsc::transport::RelayServerInfoOutcome as O;
    match outcome {
        O::Reachable { auth_mode, doc } => RelayTestDto::Reachable {
            auth_mode: relay_auth_mode_str(auth_mode).to_string(),
            doc: Box::new(ServerInfoDocDto {
                name: doc.name,
                version: doc.version,
                api: doc.api,
                max_body_bytes: doc.max_body_bytes,
                max_queue_depth: doc.max_queue_depth,
                retention_ttl_secs: doc.retention_ttl_secs,
                directory_mode: doc.directory_mode,
                attachments_service_url: doc.attachments_service_url,
                kt_mode: doc.kt_mode,
                min_client_version: doc.min_client_version,
            }),
        },
        O::AuthRequired { token_was_sent } => RelayTestDto::AuthRequired { token_was_sent },
        O::NotAQslRelay => RelayTestDto::NotAQslRelay,
        O::CertNotTrusted => RelayTestDto::CertNotTrusted,
        O::Unreachable => RelayTestDto::Unreachable,
    }
}

/// Read the persisted relay endpoint (NON-SECRET; from settings.json).
#[tauri::command]
pub fn relay_config_get(st: State<'_, AppState>) -> RelayConfigDto {
    RelayConfigDto {
        relay_url: settings::load(&st.data_dir).relay_url,
    }
}

/// Persist the relay endpoint (URL ONLY; token + CA live in the qsc vault).
/// Validates with `normalize_relay_endpoint`; on a malformed address returns
/// the code for INLINE field validation (R2a) — never a results card, since no
/// probe was attempted. Stores the normalized form (what the probe uses).
#[tauri::command]
pub fn relay_config_set(st: State<'_, AppState>, url: String) -> Result<(), String> {
    let normalized =
        qsc::adversarial::route::normalize_relay_endpoint(&url).map_err(|c| c.to_string())?;
    let mut s = settings::load(&st.data_dir);
    s.relay_url = normalized;
    settings::save(&st.data_dir, &s)
}

/// Probe `GET {url}/v1/server-info` through the serial blocking gate (R1) and
/// return the pre-classified outcome. `Err` carries a LOCAL-config code
/// (`relay_endpoint_*` for a bad address, `relay_ca_file_*` for an unreadable
/// configured CA, `relay_server_info_failed` for a client build failure); the
/// FE maps it per R2 — the CA-file case is its OWN line, NOT CertNotTrusted.
#[tauri::command]
pub async fn relay_test(st: State<'_, AppState>, url: String) -> Result<RelayTestDto, String> {
    let outcome = st
        .gw
        .call(move || qsc::transport::relay_server_info(&url))
        .await;
    match outcome {
        Ok(o) => Ok(relay_test_dto(o)),
        Err(code) => Err(code.to_string()),
    }
}

/// Set the relay bearer token — into the qsc vault via the trio, NEVER
/// `vault::secret_set` directly. Empty is rejected by qsc.
#[tauri::command]
pub async fn relay_token_set(st: State<'_, AppState>, token: String) -> Result<(), String> {
    st.gw
        .call(move || qsc::transport::relay_token_set(&token).map_err(|c| c.to_string()))
        .await
}

#[tauri::command]
pub async fn relay_token_clear(st: State<'_, AppState>) -> Result<(), String> {
    st.gw
        .call(|| qsc::transport::relay_token_clear().map_err(|c| c.to_string()))
        .await
}

/// Presence ONLY — the bare bool (FLAG-3: no hash of a secret).
#[tauri::command]
pub async fn relay_token_show(st: State<'_, AppState>) -> Result<RelayTokenStatusDto, String> {
    Ok(st
        .gw
        .call(|| RelayTokenStatusDto {
            configured: qsc::transport::relay_token_show().configured,
        })
        .await)
}

/// Set the operator CA-file path — into the qsc vault via the trio. qsc
/// validates the file exists (`relay_ca_file_missing`).
#[tauri::command]
pub async fn relay_ca_file_set(st: State<'_, AppState>, path: String) -> Result<(), String> {
    st.gw
        .call(move || qsc::transport::relay_ca_file_set(&path).map_err(|c| c.to_string()))
        .await
}

#[tauri::command]
pub async fn relay_ca_file_clear(st: State<'_, AppState>) -> Result<(), String> {
    st.gw
        .call(|| qsc::transport::relay_ca_file_clear().map_err(|c| c.to_string()))
        .await
}

/// CA-file presence + redacted path hash (the path is public; the deliberate
/// asymmetry with the bare-bool token). ⚠ Resolves through `vault::secret_get`
/// and fails CLOSED when locked → a locked vault reads configured=false, not
/// "unknown" (Appendix F.7). Safe ONLY because Settings is unlock-gated.
#[tauri::command]
pub async fn relay_ca_file_show(st: State<'_, AppState>) -> Result<RelayCaStatusDto, String> {
    Ok(st
        .gw
        .call(|| {
            let s = qsc::transport::relay_ca_file_show();
            RelayCaStatusDto {
                configured: s.configured,
                path_hash: s.path_hash,
            }
        })
        .await)
}

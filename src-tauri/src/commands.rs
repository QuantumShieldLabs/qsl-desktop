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

pub const PQ_LINE: &str =
    "Post-quantum hybrid: ML-KEM-768 (key agreement) + ML-DSA-65 (signatures)";
pub const FINGERPRINT_PURPOSE_LINE: &str =
    "Contacts use this fingerprint to verify it's really you.";

#[derive(Serialize)]
pub struct IdentityDto {
    pub fingerprint: String,
    pub verify_code: String,
    pub purpose_line: &'static str,
    pub pq_line: &'static str,
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
    pub version: &'static str,
    pub slice: &'static str,
}

fn identity_dto(rec: &qsc::identity::IdentityPublicRecord) -> IdentityDto {
    let fp = qsc::identity::identity_fingerprint_from_identity(&rec.kem_pk, &rec.sig_pk);
    let code = qsc::identity::format_verification_code_from_fingerprint(&fp);
    IdentityDto {
        fingerprint: fp,
        verify_code: code,
        purpose_line: FINGERPRINT_PURPOSE_LINE,
        pq_line: PQ_LINE,
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
pub fn settings_set(st: State<'_, AppState>, autolock_minutes: u32) -> Result<(), String> {
    settings::save(&st.data_dir, &AppSettings { autolock_minutes })
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
        version: env!("CARGO_PKG_VERSION"),
        slice: "A (serverless skeleton; server connectivity arrives in a future update)",
    }
}

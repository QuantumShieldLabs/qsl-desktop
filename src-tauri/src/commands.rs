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

/// NA-0705 (D640 §2c(i), A2.2) — THE VAULT-VERSION PRE-FLIGHT.
///
/// The qsc bump crossed a `QSCV01` -> `QSCV02` envelope break that is a HARD BREAK:
/// no migration, no dual-format read (D628 Ruling 2). qsc names the refusal correctly
/// (`vault_version_unsupported`) at both of its parse sites — but `unlock_guarded`
/// collapses EVERY `Err` into one branch (`protection.rs:156`), counts a failed attempt
/// (`:175`) and, at an armed limit, WIPES THE VAULT (`:180`). Measured before this
/// pre-flight existed: three CORRECT passphrases against a `QSCV01` vault produced
/// `rejected`, `rejected`, `wiped` — the vault destroyed by the right passphrase.
///
/// ⚠ THEREFORE: on a recognized-but-old envelope the desktop MUST NOT CALL
/// `unlock_guarded` AT ALL (R185 §2.3). Classification GATES the call; it does not
/// interpret its result. The same gate is owed on the destroy door, which reaches the
/// same refusal by an independent route (F-2).
///
/// The class fix — teaching the guard itself to distinguish non-passphrase errors — is
/// filed as a spine successor and is NOT this lane's (R187 §3 F-2 / Q8.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultVersionState {
    /// No vault file — nothing to classify (the S0/S1 case).
    Absent,
    /// `QSCV02`: the format this build reads and writes.
    Current,
    /// `QSCV01`: written by an older build. Unreadable here, and refusing it by name
    /// is the only honest answer.
    KnownOld,
    /// Present but unreadable or unrecognized — deliberately NOT treated as `KnownOld`.
    Unknown,
}

/// Classify the on-disk envelope through qsc's ONE owner of magic recognition
/// (D-1334), reading only the 6 magic bytes. `paths::vault_file` and the file
/// `unlock_guarded` opens provably resolve to the same path: the desktop sets
/// `QSC_CONFIG_DIR` once at `lib.rs:306`, and qsc's `config_dir()` keeps that at top
/// precedence (verified byte-identical across the bump, SR-15 F-7).
pub fn vault_version_state(data_dir: &Path) -> VaultVersionState {
    use qsc::adversarial::vault_format::{classify_vault_magic, VaultMagicClass};
    let path = paths::vault_file(data_dir);
    let Ok(bytes) = fs::read(&path) else {
        return if path.exists() {
            VaultVersionState::Unknown
        } else {
            VaultVersionState::Absent
        };
    };
    if bytes.len() < 6 {
        return VaultVersionState::Unknown;
    }
    match classify_vault_magic(&bytes[..6]) {
        VaultMagicClass::Current => VaultVersionState::Current,
        VaultMagicClass::KnownOld => VaultVersionState::KnownOld,
        VaultMagicClass::Unknown => VaultVersionState::Unknown,
    }
}

/// The one error code both doors return for a recognized-but-old envelope — the same
/// name qsc uses, so the desktop never invents a second vocabulary for one cause.
pub const VAULT_VERSION_UNSUPPORTED: &str = "vault_version_unsupported";

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
    /// NA-0705: the vault was written by an older build (`QSCV01`). A distinct cause
    /// gets a distinct name — this is NOT a passphrase failure and must never be
    /// rendered as one.
    VersionUnsupported,
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
    /// NA-0776 (`ENG-0275`, spec v2 3.4): WHICH BUILD THIS IS, so a flight can say.
    /// Either a 40-hex commit or the literal "unknown" -- never empty, never invented.
    /// `dirty` and a build timestamp are deliberately ABSENT; see build.rs for why a
    /// field that is believed and can be wrong is worse than an absent one.
    pub build_commit: &'static str,
    /// NA-0763 (`D-0040`, ruling `R4`) — THE TEST-ONLY TEMPO SEAM, and the
    /// reason it lives HERE rather than on `AppSettings`.
    ///
    /// `AppInfoDto` is `Serialize`-ONLY: it is never deserialized and has no
    /// `save` path, so `settings::save` cannot round-trip a test tempo because
    /// the value is not of a shape it can ever hold. That answers R4's binding
    /// constraint STRUCTURALLY rather than by a caller's discipline.
    ///
    /// `None` in every ordinary run — the harness sets `QSLD_TICK_MS` per
    /// launch the same way `runner.py` already sets `QSLD_DATA_DIR`.
    pub tick_override_ms: Option<u64>,
}

fn identity_dto(rec: &qsc::identity::IdentityPublicRecord) -> IdentityDto {
    let fp = qsc::identity::identity_fingerprint_from_identity(&rec.kem_pk, &rec.sig_pk);
    let code = qsc::identity::identity_voice_form(&fp);
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
    let s = st
        .gw
        .call_named("launch_state", move || resolve_launch_state(&data))
        .await;
    // NA-0776 (3.5): this is the only place the app forms a belief about the store.
    crate::record_believed_state(s);
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
    // NA-0776 (3.5) DOOR 1 -- external-wipe detection, FAILING CLOSED. A process that
    // believed it had a store and now resolves S0 was wiped from outside; creating a
    // vault inside it is exactly ENG-0276's shape. Refuse and require a relaunch.
    if crate::store_vanished(&st.data_dir) {
        return Err(crate::STORE_VANISHED.into());
    }
    st.gw
        .call_named("vault_create", move || -> Result<(), String> {
            // NA-0705 (F-10): this guard call needs NO version pre-flight, and the reason
            // is a precondition worth writing down rather than rediscovering. The wizard
            // that reaches `vault_create` is reachable ONLY from launch state S0, and
            // `resolve_launch_state` returns S0 only when `vault_file(data_dir)` does NOT
            // exist (`state.rs:71`) — so no envelope, old or otherwise, can be present.
            // The vault unlocked below is the one created on the line above.
            // ⚠ If a future change ever lets the wizard be reached with a vault file
            // present, this route silently reopens and owes the same pre-flight.
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
        .call_named("identity_ensure", move || {
            let rec = qsc::identity::identity_ensure(SELF_LABEL).map_err(|e| format!("{e:?}"))?;
            Ok(identity_dto(&rec))
        })
        .await
}

#[tauri::command]
pub async fn identity_show(st: State<'_, AppState>) -> Result<Option<IdentityDto>, String> {
    st.gw
        .call_named("identity_show", move || {
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
    let data = st.data_dir.clone();
    let r = st
        .gw
        .call_named("unlock_attempt", move || {
            unlock_attempt_impl(&data, &passphrase)
        })
        .await;
    // NA-0779 (`D-0048`): AT UNLOCK the stored switch is read and applied -- with the log on
    // the engine sink is installed here and `gw.unlock` opens the session's record. The
    // switch is a setting, never an environment variable.
    if let Ok(UnlockDto::Unlocked) = &r {
        let dl = crate::settings::load(&st.data_dir).debug_log;
        crate::debug_log::DebugLog::global().on_unlock(dl.on, dl.level);
    }
    r
}

/// NA-0705 (D640 A2.2): the unlock decision as a plain function, mirroring
/// `destroy_vault_impl` — the seam the QSCV01 refusal instrument drives.
pub fn unlock_attempt_impl(data_dir: &Path, passphrase: &str) -> Result<UnlockDto, String> {
    // NA-0776 (3.5) DOOR 2 -- and the ORDER is load-bearing, ruled at RULING_005 R8.
    // This runs BEFORE `unlock_guarded`, because that path WRITES: it reaches
    // `protection_state_load`, whose second line is `ensure_store_layout`, which
    // re-materialises `qsc/` and `store.meta` and takes a lock on a FRESH `.qsc.lock`
    // inode. A check placed after it would interrogate a store the check itself had
    // just re-created -- and would feed the very inode hazard MAJOR-12 filed.
    if crate::store_vanished(data_dir) {
        return Err(crate::STORE_VANISHED.to_string());
    }
    {
        // ⚠ DOOR 1. The pre-flight GATES the guard — it does not interpret its result.
        // Reaching `unlock_guarded` with a QSCV01 envelope is what burns an attempt and,
        // at an armed limit, wipes the vault.
        if vault_version_state(data_dir) == VaultVersionState::KnownOld {
            return Ok(UnlockDto::VersionUnsupported);
        }
        {
            use qsc::vault::protection::GuardedUnlockOutcome as O;
            match qsc::vault::protection::unlock_guarded(passphrase).map_err(|e| e.to_string())? {
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
                O::Wiped { .. } => {
                    // NA-0753 (R376 §1; ENG-0217, desktop D-0034): the armed
                    // "Erase vault after failed attempts" wipe runs inside qsc,
                    // which owns only its OWN directory — correct by design, and
                    // exactly why the app-level residue is ours to clear. Shape A,
                    // mirroring `destroy_vault_impl`: `settings.json` AND its
                    // `.tmp` staging sibling, so no prior-profile value (the relay
                    // address, the display alias) crosses the wipe boundary —
                    // the D597 item-13 rule ENG-0048 enforces on the other two
                    // vault-destroying sites.
                    let sf = paths::settings_file(data_dir);
                    for p in [sf.clone(), sf.with_extension("json.tmp")] {
                        if p.exists() {
                            fs::remove_file(&p).map_err(|e| e.to_string())?;
                        }
                    }
                    // NA-0776 (3.6-v3.1 sec 5): THE ARMED PATH IS THE ONE THAT MATTERS.
                    // It is the only wipe that never reloads (main.js:541), and its
                    // "Start over" control historically called route(), so a NEW VAULT
                    // WAS CREATED INSIDE THE SAME PROCESS -- ENG-0276's own
                    // reproduction, reachable through the shipped UI. The marker is set
                    // here; the restart happens when the user leaves this screen, which
                    // is the point where continuation would otherwise occur, and which
                    // preserves the ceremony's message instead of yanking it away.
                    crate::mark_webview_wipe_pending();
                    Ok(UnlockDto::Wiped)
                }
            }
        }
    }
}

/// NA-0779 (`D-0048`): `cause` is OPTIONAL and closed -- the idle timer passes `autolock`, every
/// other caller passes nothing and reads as `user`; anything else reads as `user` too. The ring
/// is cleared AFTER the engine's lock, keeping `gw.lock` with its cause as the new ring's first
/// event; the sink is removed while locked.
#[tauri::command]
pub async fn lock_now(st: State<'_, AppState>, cause: Option<String>) -> Result<(), String> {
    st.gw
        .call_named("lock_now", || qsc::vault::protection::lock(None))
        .await;
    crate::debug_log::DebugLog::global().on_lock(cause.as_deref().unwrap_or("user"));
    Ok(())
}

#[tauri::command]
pub async fn protection_status(st: State<'_, AppState>) -> Result<ProtectionDto, String> {
    st.gw
        .call_named("protection_status", move || {
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
        .call_named("wipe_arm", move || {
            qsc::vault::protection::wipe_after_failed_unlocks_arm(limit).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub async fn wipe_disarm(st: State<'_, AppState>) -> Result<(), String> {
    st.gw
        .call_named("wipe_disarm", || {
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
    let data = st.data_dir.clone();
    let r = st
        .gw
        .call_named("destroy_vault", move || {
            destroy_vault_impl(&data, &passphrase)
        })
        .await;
    // NA-0779 (`D-0048`): the passphrase-committed destroy is an erase for the log too.
    if r.is_ok() {
        crate::debug_log::DebugLog::global().on_erase();
    }
    r
}

/// The tokened core destroy (passphrase-committed — the opposite case from
/// erase) plus its app-level boundary consequence (ENG-0048; D-0024, spine
/// D-1337): `settings.json` is profile-scoped, and its existence is the
/// D-0018 "identity step finished" signal — a file surviving destroy would
/// forge S2 for the NEXT profile's onboarding. It dies with the vault,
/// mirroring erase; the `.tmp` staging sibling (the settings.rs write path)
/// goes with it.
pub fn destroy_vault_impl(data_dir: &Path, passphrase: &str) -> Result<(), String> {
    // ⚠ DOOR 2 (NA-0705, F-2). Destroy reaches the same version refusal by an
    // INDEPENDENT route: at the new pin `destroy_with_passphrase` peeks the envelope
    // through the same parser BEFORE it examines the passphrase. Gating here means the
    // desktop owns the refusal and its name, rather than depending on the ordering of
    // qsc's internals to produce it.
    if vault_version_state(data_dir) == VaultVersionState::KnownOld {
        return Err(VAULT_VERSION_UNSUPPORTED.to_string());
    }
    // NA-0705 (D640 A2.1): the constructor is now value-neutral — `confirm(typed)` just
    // carries what the human typed, and what the commitment must EQUAL is decided at the
    // destroy site by a runtime branch on the peeked `key_source`: keychain vaults
    // (`== 2`) require the literal VAULT_DESTROY_INTENT_PHRASE, every other vault requires
    // the passphrase. Passing the passphrase here is correct ONLY under this precondition:
    // ⚠ THE DESKTOP CAN NEVER HOLD A KEYCHAIN VAULT — it declares `qsc` with no `features`
    // key, qsc's `default = []`, and `keyring` is absent from Cargo.lock, so the whole
    // `#[cfg(feature = "keychain")]` region is compiled out. If that ever changes, this
    // value-identical line becomes silently wrong (it would earn
    // `vault_destroy_confirm_mismatch`) and must pass the intent phrase instead.
    let token = qsc::vault::protection::DestroyConfirmToken::confirm(passphrase);
    qsc::vault::protection::destroy_with_passphrase(passphrase, token)
        .map_err(|e| e.to_string())?;
    let sf = paths::settings_file(data_dir);
    for p in [sf.clone(), sf.with_extension("json.tmp")] {
        if p.exists() {
            fs::remove_file(&p).map_err(|e| e.to_string())?;
        }
    }
    // NA-0776 (3.6-v3.1 sec 4): the wipe succeeded, so mark the webview directory for
    // deletion at the NEXT bootstrap -- the only point with no WebContext alive.
    crate::mark_webview_wipe_pending();
    Ok(())
}

/// NA-0776 (3.6-v3.1 sec 4/5) -- the explicit restart. A PROCESS restart, not
/// `window.location.reload()`: the reload does not reset the WebContext, which lives
/// for the process, and it leaves every module-scope value in the front end intact.
/// This is cure (B), and it is what makes the bootstrap deletion sound.
///
/// ⚠ WHY THE RESTART IS ITS OWN COMMAND RATHER THAN THE TAIL OF EACH WIPE. Putting
/// `app.restart()` inside `erase_all`/`destroy_vault` makes those commands terminate
/// the process, so the NA-0700 IPC replay harness -- whose whole purpose is to invoke
/// EVERY registered command through real IPC -- can no longer invoke them: the mock
/// runtime's `restart` is `not implemented` and the harness dies. Weakening that
/// harness to accommodate a cure is not available (the kickoff forbids it), so the
/// restart is issued by the CALLER, at exactly the site that calls
/// `window.location.reload()` today.
/// THE DURABLE HALF IS STILL RUST-SIDE AND UNCONDITIONAL: each wipe sets the marker
/// itself, so the webview deletion happens at the next bootstrap even if a restart is
/// never issued. The restart breaks process continuity; the marker guarantees the
/// deletion. They fail independently, which is why A5 (the missed-marker witness) is a
/// separate arm.
#[tauri::command]
pub fn restart_app<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    // NA-0779 (`D-0048`): the ring dies with the process; the cause is named for the record.
    crate::debug_log::DebugLog::global().on_lock("restart");
    app.restart();
}

#[tauri::command]
pub async fn erase_all(st: State<'_, AppState>, confirm_phrase: String) -> Result<(), String> {
    if confirm_phrase != ERASE_CONFIRM_PHRASE {
        return Err("confirm_phrase_mismatch".into());
    }
    let data = st.data_dir.clone();
    let r = st
        .gw
        .call_named("erase_all", move || erase_all_impl(&data))
        .await;
    // NA-0779 (`D-0048`): `gw.erase`, then the ring is WIPED and the switch returns to off with
    // the rest of the settings (the file is gone with the profile).
    if r.is_ok() {
        crate::debug_log::DebugLog::global().on_erase();
    }
    r
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
    // NA-0776 (3.6-v3.1 sec 4): mark the webview directory for the next bootstrap.
    crate::mark_webview_wipe_pending();
    crate::create_private_dir(&qsc_dir)?;
    Ok(())
}

/// NA-0776 (spec v2 3.3 / `ENG-0274`) -- the declined-frame notice's DTO.
/// `{ kind, count }` and nothing else. `first_seen_ms`/`last_seen_ms` were specified in
/// v1 and REMOVED: they have no source in `MarkerBuffer` (cold read BLOCKER-4) and,
/// independently, per-attempt timing metadata is a deliberate acquisition this house
/// declines to make as a side effect of a DTO shape (NOTE-4) -- whoever can see the
/// window would learn when connection attempts happened, in a tool whose emitting lane
/// deliberately stripped every other correlator.
#[derive(serde::Serialize)]
pub struct NoticeDto {
    /// Always a member of `markers::NOTICE_KINDS` -- the classifier's return type makes
    /// that structural, not a convention.
    pub kind: &'static str,
    /// The UNDISMISSED count: monotonic total minus this kind's dismiss watermark.
    pub count: u64,
}

/// The notice surface. A plain sync command, like `marker_stats`: it reads an in-memory
/// buffer and must not queue behind core calls on the serial gateway.
/// ⚠ It returns a CLASSIFICATION, never a marker line. There is no route from here to
/// raw marker text.
#[tauri::command]
pub fn notice_list(st: State<'_, AppState>) -> Vec<NoticeDto> {
    st.gw
        .markers
        .notices()
        .into_iter()
        .map(|(kind, count)| NoticeDto { kind, count })
        .collect()
}

/// Dismiss one kind. Rust-side watermark, so it SURVIVES the `window.location.reload()`
/// that erase and destroy both perform (cold read MINOR-11). A kind outside the
/// whitelist is ignored.
#[tauri::command]
pub fn notice_dismiss(st: State<'_, AppState>, kind: String) {
    st.gw.markers.dismiss(&kind);
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

/// NA-0776 (3.4): the stamp's acceptance rule, PURE so the "unknown" branch is
/// drivable without a second build -- the `parse_tick_override` precedent
/// (settings.rs). `env!` could not be used at all here: it is a COMPILE ERROR on an
/// absent variable, so the red arm the spec names would not exist (cold read MAJOR-10).
/// A malformed stamp degrades to "unknown" rather than shipping garbage a reader would
/// believe.
pub fn build_commit_or_unknown(stamped: Option<&'static str>) -> &'static str {
    match stamped {
        Some(s) if s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()) => s,
        _ => "unknown",
    }
}

#[tauri::command]
pub fn app_info() -> AppInfoDto {
    AppInfoDto {
        display_name: APP_DISPLAY_NAME,
        version: env!("CARGO_PKG_VERSION"),
        slice: "B (relay connectivity: point the app at a relay and test the connection)",
        build_commit: build_commit_or_unknown(option_env!("QSLD_BUILD_COMMIT")),
        tick_override_ms: settings::tick_override_from_env(),
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

/// A rendering of `qsc::transport::RelayServerInfoOutcome` for the FE — NOT a
/// re-classification (R1). `auth_mode` is "open" | "bearer".
///
/// ⚠ NA-0705 (F-4): this stopped being 1:1 at the qsc bump. `ServerInfoDoc` gained
/// three invite-related limits — `invite_max_expiry_secs`, `invite_max_slots`,
/// `max_invite_bundle_bytes` — which this DTO does not carry, so the Server pane shows
/// 10 of the relay's 13 advertised fields. Nothing breaks (every wire field carries
/// `#[serde(default)]`, so an older relay's document still parses and still reaches
/// `Reachable`); surfacing the three is Server-pane design work and is FILED, not done
/// here. The comment is corrected so the claim matches the code.
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
        .call_named("relay_test", move || {
            qsc::transport::relay_server_info(&url)
        })
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
        .call_named("relay_token_set", move || {
            qsc::transport::relay_token_set(&token).map_err(|c| c.to_string())
        })
        .await
}

#[tauri::command]
pub async fn relay_token_clear(st: State<'_, AppState>) -> Result<(), String> {
    st.gw
        .call_named("relay_token_clear", || {
            qsc::transport::relay_token_clear().map_err(|c| c.to_string())
        })
        .await
}

/// Presence ONLY — the bare bool (FLAG-3: no hash of a secret).
#[tauri::command]
pub async fn relay_token_show(st: State<'_, AppState>) -> Result<RelayTokenStatusDto, String> {
    Ok(st
        .gw
        .call_named("relay_token_show", || RelayTokenStatusDto {
            configured: qsc::transport::relay_token_show().configured,
        })
        .await)
}

/// Set the operator CA-file path — into the qsc vault via the trio.
///
/// ⚠ THIS CALL DOES NOT VALIDATE THE PATH, and a comment here used to say it did
/// ("qsc validates the file exists"). Measured against the pinned qsc at
/// `transport/mod.rs:2250`: `relay_ca_file_set` trims, rejects ONLY the empty
/// string, and writes to the vault. It never touches the filesystem. The false
/// claim is why a garbage path could be stored and then reported as configured —
/// NA-0754 / ENG-0222, the defect's ROOT rather than its symptom.
///
/// The real check lives one layer down and runs at PROBE time:
/// `relay_http_client()` does exists (`relay_ca_file_missing` /
/// `relay_ca_file_unreadable`) plus a REAL PEM parse
/// (`relay_ca_file_invalid`), and `relay_server_info` returns those codes
/// BEFORE it opens a socket — which is why `relay_probe` can validate a CA
/// path with no relay reachable at all.
#[tauri::command]
pub async fn relay_ca_file_set(st: State<'_, AppState>, path: String) -> Result<(), String> {
    st.gw
        .call_named("relay_ca_file_set", move || {
            qsc::transport::relay_ca_file_set(&path).map_err(|c| c.to_string())
        })
        .await
}

#[tauri::command]
pub async fn relay_ca_file_clear(st: State<'_, AppState>) -> Result<(), String> {
    st.gw
        .call_named("relay_ca_file_clear", || {
            qsc::transport::relay_ca_file_clear().map_err(|c| c.to_string())
        })
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
        .call_named("relay_ca_file_show", || {
            let s = qsc::transport::relay_ca_file_show();
            RelayCaStatusDto {
                configured: s.configured,
                path_hash: s.path_hash,
            }
        })
        .await)
}

// ─────────────────────────────────────────────────────────────────────────────
// NA-0754 (D-0035) — TEST-AND-SAVE-ON-PROOF. The two functions the model needs.
//
// THE INVARIANT: what is persisted has connected at least once. That requires
// probing a triple the user has TYPED but that is NOT YET STORED — the exact
// thing the old model could not do, which is why R-B2 ruled "validating IS
// writing" and why a failed test could clobber a working config.

/// Probe a relay with an EXPLICIT (address, token, CA path) triple, persisting
/// NOTHING. This is the whole of the NA-0754 model's engine half.
///
/// `token`/`ca_path` are `None` to mean "use whatever is stored" — which is
/// R-B3's blank-means-keep, unchanged — and `Some(v)` to mean "probe with THIS
/// value instead".
///
/// ⚠ HOW THE EXPLICIT VALUES REACH qsc, AND WHY IT IS SOUND HERE. qsc resolves
/// both secrets itself: `relay_auth_token()` reads env `QSC_RELAY_TOKEN` → env
/// `RELAY_TOKEN` → the vault, and `relay_ca_file()` reads env `QSC_RELAY_CA_FILE`
/// → env `RELAY_CA_FILE` → the vault. **Env is consulted FIRST in both chains**,
/// so setting it overrides the stored value for the duration of one probe. ZERO
/// qsc bytes change.
///
/// ⚠ THE RESTORE IS UNCONDITIONAL, and that is the load-bearing detail. `EnvGuard`
/// restores each variable to its exact prior state (including ABSENT, which is
/// distinct from empty) in `Drop`, so the restore runs on the success path, on
/// every early return, AND while a panic unwinds — a leaked `QSC_RELAY_TOKEN`
/// would silently override the vault for every later call in the process.
///
/// ⚠ THE BOUNDARY, STATED IN THE RECORD RATHER THAN HIDDEN IN A COMMENT (D-0035):
/// `std::env::set_var` is PROCESS-GLOBAL, while the `CoreGateway` serializes qsc
/// calls only. The set → probe → restore sequence runs entirely INSIDE one
/// `gw.call` closure — one blocking thread, inside the process-wide single-flight
/// guard — so no other qsc call can observe the mutated environment. A non-qsc
/// thread reading these two variables concurrently is the residual hazard; nothing
/// in this tree does, measured, but the boundary is real and is recorded, not
/// asserted away.
///
/// ⚠ WHAT THIS CANNOT DO, also recorded: it cannot probe with NO token while one
/// IS stored. An empty env value is trimmed to nothing and falls THROUGH to the
/// vault, so absence is not expressible without a write. Ruled at R379 §Q1: the
/// x control is the removal path, and it deletes immediately and offline.
#[tauri::command]
pub async fn relay_probe(
    st: State<'_, AppState>,
    address: String,
    token: Option<String>,
    ca_path: Option<String>,
) -> Result<RelayTestDto, String> {
    let outcome = st
        .gw
        .call_named("relay_probe", move || {
            let _token_guard = EnvGuard::set("QSC_RELAY_TOKEN", token.as_deref());
            let _ca_guard = EnvGuard::set("QSC_RELAY_CA_FILE", ca_path.as_deref());
            qsc::transport::relay_server_info(&address)
        })
        .await;
    match outcome {
        Ok(o) => Ok(relay_test_dto(o)),
        Err(code) => Err(code.to_string()),
    }
}

/// The user's home directory, so the front end can expand a leading `~/` in the
/// CA-path field VISIBLY, before the path is used (design bank v2 item 4).
///
/// The webview cannot resolve `~` on its own: `$HOME` is a process fact and no
/// other command exposes it. Returns an empty string when `HOME` is unset, and
/// the caller then refuses `~` rather than guessing — a wrong guess would send
/// the probe at a path the user never typed.
#[tauri::command]
pub fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_default()
}

/// Restores one environment variable to its EXACT prior state on drop —
/// including ABSENT, which `set_var("")` would not reproduce, because qsc
/// trims an empty value to `None` and falls through to the vault. Drop runs on
/// success, on early return, and during panic unwind.
struct EnvGuard {
    key: &'static str,
    prior: Option<String>,
    engaged: bool,
}

impl EnvGuard {
    fn set(key: &'static str, value: Option<&str>) -> EnvGuard {
        let prior = std::env::var(key).ok();
        let engaged = value.is_some();
        if let Some(v) = value {
            std::env::set_var(key, v);
        }
        EnvGuard {
            key,
            prior,
            engaged,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if !self.engaged {
            return;
        }
        match &self.prior {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// NA-0751 (D-0032) — THE SLICE-4 GATEWAY SURFACE.
//
// Twelve pass-through wrappers over `qsc::facade`, the typed spine that landed on
// protocol main `9dcded4d`. They CALL; they never decide: every wrapper is one
// `st.gw.call` around one facade verb, so the module-doc invariant above (rules b
// and d — every qsc call through the CoreGateway, on the blocking gate, strictly
// serially) holds for the new surface exactly as it does for the old.
//
// ⚠ THE SERIALIZATION IS LOAD-BEARING, not incidental. `facade::invite_revoke`'s own
// doc records that on error the local revoke MAY already have committed, and that a
// screen tells the outcomes apart by calling `invite_list` AFTER the error — "two
// calls the surface already carries, serialized by the desktop gateway's single
// flight". That sequence is only sound because both wrappers run inside `gw.call`.
//
// ⚠ `facade::invite_list_at` is DELIBERATELY NOT EXPOSED. It is a clock-injection
// seam for deterministic tests; reaching it through IPC would let the front end
// choose the time an expiry is judged against. The desktop ships `invite_list` only.

/// One facade failure, as the front end receives it.
///
/// `code` is the facade's own stable wire discriminant — one of the pinned THIRTY-NINE
/// (26 non-`Store` variants + `Store`'s 13-member fan-out over `ErrorCode::as_str`).
/// It is NOT `{e:?}`: a Debug rendering is a Rust detail that may change without the
/// wire contract changing, and the front end string-matches on this value.
///
/// ⚠ THE SET IS 39, NOT 27, because `Store` fans out. One of those thirty-nine,
/// `lock_upgrade_refused`, is the code the `Store` variant exists to keep reachable —
/// collapsing `Store` to a single discriminant would make it unreachable to a GUI.
///
/// ⚠ NA-0755 v2 moved it 38 → 39: `clear_refused` joined the set. The count is asserted in
/// PROSE at two places in this file and in five more across both repos, and the SR-15 read
/// measured that a checker finding only the two obvious ones under-sweeps by ~5×.
#[derive(Serialize)]
pub struct ErrorDto {
    pub code: String,
    /// The residual's payload for `FacadeError::Other`, and — since NA-0755 v2 — the SOURCE
    /// CODE for `VaultUnavailable`.
    ///
    /// ⚠ **THIS DOC'S PREVIOUS SENTENCE WAS FALSIFIED AND IS REPLACED.** It read *"Carried for
    /// `FacadeError::Other` alone … `None` for every named variant, so a screen cannot
    /// accidentally render a detail that is not there."* That is no longer true: exactly ONE
    /// named variant now carries a payload. It is stated rather than quietly widened, because
    /// the front end's own rendering rule depends on knowing which arms can be non-`None`.
    ///
    /// ⚠ Still safe to render: `VaultUnavailable`'s payload is closed at SEVEN static
    /// lowercase tokens (the six `VAULT_FAMILY` members plus `identity_secret_unavailable`), so
    /// **no user bytes can ride it**. `Other`'s payload is shape-sealed to `^[a-z][a-z0-9_]*$`.
    /// Every other named variant is still `None`.
    pub detail: Option<String>,
}

/// NA-0779 (`D-0048`): what `gw.command` learns from a facade failure -- the wire CODE only; the
/// detail (free text) never enters the log.
impl crate::gateway::ErrorCode for ErrorDto {
    fn code(&self) -> String {
        self.code.clone()
    }
}

/// NA-0779 (`D-0048`): the plain DTOs the gateway's closures return read as `ok` outcomes.
impl crate::gateway::CommandOutcome for RelayTokenStatusDto {
    fn outcome(&self) -> (qsc::output::event::Outcome, Option<String>) {
        (qsc::output::event::Outcome::Ok, None)
    }
}

impl crate::gateway::CommandOutcome for RelayCaStatusDto {
    fn outcome(&self) -> (qsc::output::event::Outcome, Option<String>) {
        (qsc::output::event::Outcome::Ok, None)
    }
}

impl From<qsc::facade::FacadeError> for ErrorDto {
    fn from(e: qsc::facade::FacadeError) -> ErrorDto {
        let code = e.as_wire().to_string();
        let detail = match &e {
            qsc::facade::FacadeError::Other(s) => Some(s.clone()),
            // NA-0755 v2 (R381 §1): the self-diagnosing vault arm. `vault_unavailable` collapses
            // six provenances into one word, which is exactly why the operator's screenshot
            // could not diagnose itself. The source code rides `detail` for this arm alone.
            qsc::facade::FacadeError::VaultUnavailable(src) => src.map(|s| s.to_string()),
            _ => None,
        };
        ErrorDto { code, detail }
    }
}

/// The two renderings of one fingerprint — `facade::FingerprintPair`.
#[derive(Serialize)]
pub struct FingerprintDto {
    pub full: String,
    pub voice: String,
}

impl From<&qsc::facade::FingerprintPair> for FingerprintDto {
    fn from(p: &qsc::facade::FingerprintPair) -> FingerprintDto {
        FingerprintDto {
            full: p.full.clone(),
            voice: p.voice.clone(),
        }
    }
}

/// `facade::ConnectStatus`.
///
/// ⚠ `ConnectState` and `ContactState` are the two facade enums with NO `as_wire` of
/// their own, so their wire strings are minted HERE and are this module's contract.
/// Every other enum on this surface is rendered by the facade's own `as_wire`, and
/// those strings are the protocol's to change, not the desktop's.
#[derive(Serialize)]
pub struct ConnectStatusDto {
    pub state: String,
    pub reason: String,
}

fn connect_state_wire(s: qsc::facade::ConnectState) -> &'static str {
    match s {
        qsc::facade::ConnectState::Active => "active",
        qsc::facade::ConnectState::Inactive => "inactive",
    }
}

fn contact_state_wire(s: qsc::facade::ContactState) -> &'static str {
    match s {
        qsc::facade::ContactState::Pinned => "pinned",
        qsc::facade::ContactState::Verified => "verified",
        qsc::facade::ContactState::Changed => "changed",
        qsc::facade::ContactState::Unverified => "unverified",
    }
}

/// `facade::ContactSummary`. `fingerprint` is `Option` because typed ABSENCE is the
/// honest answer for a contact whose stored value is not a 64-hex fingerprint — the
/// facade's `W8` seal. A screen that renders `null` as a blank row is correct; one that
/// renders a placeholder string is not.
#[derive(Serialize)]
pub struct ContactDto {
    pub alias: String,
    pub fingerprint: Option<FingerprintDto>,
    pub pinned: bool,
    pub blocked: bool,
    pub state: String,
    /// NA-0764 (`D-0041`; spine `D-1405`, ruling `R6`) — what YOU call this contact, or `None`
    /// to show the alias.
    ///
    /// ⚠ RENDER-ONLY. `alias` remains the identity every verb takes; this string is never a
    /// key, never a route token and never an argument. The front end seals that structurally:
    /// `display_name` appears in ZERO invoke-argument positions, with its own can-fail proof.
    pub display_name: Option<String>,
    /// How many devices this contact has. **A PROJECTION**, not the device array.
    ///
    /// ⚠ THE COUNT AND NEVER THE ARRAY (`R6`). `ContactDeviceRecord` carries `device_id`,
    /// `fp`, `sig_fp`, `kem_pk` and `route_token` — identifiers and key material — and a
    /// screen needs a number.
    pub device_count: usize,
}

impl From<&qsc::facade::ContactSummary> for ContactDto {
    fn from(c: &qsc::facade::ContactSummary) -> ContactDto {
        ContactDto {
            alias: c.alias.clone(),
            fingerprint: c.fingerprint.as_ref().map(FingerprintDto::from),
            pinned: c.pinned,
            blocked: c.blocked,
            state: contact_state_wire(c.state).to_string(),
            display_name: c.display_name.clone(),
            device_count: c.device_count,
        }
    }
}

/// `facade::ContactRequestSummary`.
#[derive(Serialize)]
pub struct ContactRequestDto {
    pub alias: String,
    pub state: String,
    pub device_id: Option<String>,
    pub seen_at: Option<u64>,
}

impl From<&qsc::facade::ContactRequestSummary> for ContactRequestDto {
    fn from(r: &qsc::facade::ContactRequestSummary) -> ContactRequestDto {
        ContactRequestDto {
            alias: r.alias.clone(),
            state: r.state.as_wire().to_string(),
            device_id: r.device_id.clone(),
            seen_at: r.seen_at,
        }
    }
}

/// `facade::InviteSummary`.
#[derive(Serialize)]
pub struct InviteDto {
    pub invite_id: String,
    pub state: String,
    pub expiry: u64,
    pub revocable: bool,
    /// The local note: who this invite is for. `None`, never `Some("")` — the engine normalises
    /// at the mint boundary so no consumer has to special-case an empty string.
    ///
    /// ⚠ SEMANTICALLY SENSITIVE: it names who you associate with. Vault-backed at rest, never
    /// sent (the invite payload has no room for it by construction), and egress-sealed.
    pub label: Option<String>,
    /// Unix seconds at mint, or `None` when the record carries the serde-default zero.
    ///
    /// ⚠ `Option`, not a bare `u64`: the engine's field defaults to 0 = 1 Jan 1970, and a screen
    /// promising "dated" rows must render "—" for that rather than a confident wrong answer.
    pub created: Option<u64>,
}

impl From<&qsc::facade::InviteSummary> for InviteDto {
    fn from(i: &qsc::facade::InviteSummary) -> InviteDto {
        InviteDto {
            invite_id: i.invite_id.clone(),
            state: i.state.as_wire().to_string(),
            expiry: i.expiry,
            revocable: i.revocable,
            label: i.label.clone(),
            created: i.created,
        }
    }
}

#[tauri::command]
pub async fn connect_status(
    st: State<'_, AppState>,
    peer: String,
) -> Result<ConnectStatusDto, ErrorDto> {
    st.gw
        .call_named("connect_status", move || {
            let s = qsc::facade::connect_status(&peer);
            Ok(ConnectStatusDto {
                state: connect_state_wire(s.state).to_string(),
                reason: s.reason.as_wire().to_string(),
            })
        })
        .await
}

#[tauri::command]
pub async fn contact_list(st: State<'_, AppState>) -> Result<Vec<ContactDto>, ErrorDto> {
    st.gw
        .call_named("contact_list", move || {
            let rows = qsc::facade::contact_list()?;
            Ok(rows.iter().map(ContactDto::from).collect())
        })
        .await
}

#[tauri::command]
pub async fn contact_requests(st: State<'_, AppState>) -> Result<Vec<ContactRequestDto>, ErrorDto> {
    st.gw
        .call_named("contact_requests", move || {
            let rows = qsc::facade::contact_requests()?;
            Ok(rows.iter().map(ContactRequestDto::from).collect())
        })
        .await
}

#[tauri::command]
pub async fn contact_request_accept(
    st: State<'_, AppState>,
    alias: String,
) -> Result<(), ErrorDto> {
    st.gw
        .call_named("contact_request_accept", move || {
            Ok(qsc::facade::contact_request_accept(&alias)?)
        })
        .await
}

#[tauri::command]
pub async fn contact_request_ignore(
    st: State<'_, AppState>,
    alias: String,
) -> Result<(), ErrorDto> {
    st.gw
        .call_named("contact_request_ignore", move || {
            Ok(qsc::facade::contact_request_ignore(&alias)?)
        })
        .await
}

#[tauri::command]
pub async fn contact_request_block(st: State<'_, AppState>, alias: String) -> Result<(), ErrorDto> {
    st.gw
        .call_named("contact_request_block", move || {
            Ok(qsc::facade::contact_request_block(&alias)?)
        })
        .await
}

// NA-0765 (`D-0042`) — RENAME. The facade verb shipped at NA-0764 and was never
// REACHABLE: it was not in `generate_handler!`, so a `ui/`-only edit set could not
// reach it at all. This is the whole of the desktop's Rust cost for A3.
//
// ⚠ `alias` IS THE KEY and `display_name` is a LOCAL LABEL. The two are not
// interchangeable: `alias` keys `ContactsStore.peers`, `identity_read_pin` and
// `qsp_session_for_channel`, so passing the label where the key belongs would reach
// identity pins and live sessions while looking perfectly reasonable in review. The
// front end is sealed against exactly that by a structural census that counts
// `display_name` in invoke-argument positions and holds it at ZERO.
//
// ⚠ `None` CLEARS the name. The engine normalises an empty or all-whitespace string
// to `None`, so no caller has to special-case `Some("")`.
#[tauri::command]
pub async fn contact_set_display_name(
    st: State<'_, AppState>,
    alias: String,
    display_name: Option<String>,
) -> Result<(), ErrorDto> {
    st.gw
        .call_named("contact_set_display_name", move || {
            Ok(qsc::facade::contact_set_display_name(
                &alias,
                display_name.as_deref(),
            )?)
        })
        .await
}

#[tauri::command]
pub async fn invite_list(st: State<'_, AppState>) -> Result<Vec<InviteDto>, ErrorDto> {
    st.gw
        .call_named("invite_list", move || {
            let rows = qsc::facade::invite_list()?;
            Ok(rows.iter().map(InviteDto::from).collect())
        })
        .await
}

/// ⚠ `recipient_label` is LAST and is NOT `self_label` (SR-15 **B-2**). `self_label` selects
/// WHICH IDENTITY mints; `recipient_label` names WHO THE INVITE IS FOR. Both are `Option`s of
/// the same type, so the compiler cannot tell a transposition from the truth — and on a profile
/// with zero identities the swap is silently adopted as the identity's own label. The engine
/// carries the transposition seal; this layer carries the naming and the position.
#[tauri::command]
pub async fn invite_create(
    st: State<'_, AppState>,
    self_label: Option<String>,
    relay: String,
    ttl_secs: u64,
    recipient_label: Option<String>,
) -> Result<String, ErrorDto> {
    st.gw
        .call_named("invite_create", move || {
            Ok(qsc::facade::invite_create(
                self_label.as_deref(),
                &relay,
                ttl_secs,
                recipient_label.as_deref(),
            )?)
        })
        .await
}

/// Remove a local `creating` row that can never become actionable.
///
/// ⚠ NOT a repair. If the relay registered the slot it stays open until it expires and cannot be
/// revoked from here — the token was dropped unpersisted by construction. The copy says exactly
/// that and never says "safe".
#[tauri::command]
pub async fn invite_clear(st: State<'_, AppState>, invite_id: String) -> Result<(), ErrorDto> {
    st.gw
        .call_named("invite_clear", move || {
            Ok(qsc::facade::invite_clear(&invite_id)?)
        })
        .await
}

#[tauri::command]
pub async fn invite_redeem(
    st: State<'_, AppState>,
    code: String,
    alias: String,
    self_label: Option<String>,
) -> Result<String, ErrorDto> {
    st.gw
        .call_named("invite_redeem", move || {
            Ok(qsc::facade::invite_redeem(
                &code,
                &alias,
                self_label.as_deref(),
            )?)
        })
        .await
}

#[tauri::command]
pub async fn invite_accept(
    st: State<'_, AppState>,
    self_label: Option<String>,
    invite_id: String,
    alias: String,
    max: usize,
) -> Result<Option<String>, ErrorDto> {
    st.gw
        .call_named("invite_accept", move || {
            Ok(qsc::facade::invite_accept(
                self_label.as_deref(),
                &invite_id,
                &alias,
                max,
            )?)
        })
        .await
}

#[tauri::command]
pub async fn invite_finish(
    st: State<'_, AppState>,
    self_label: Option<String>,
    alias: String,
    relay: String,
    max: usize,
) -> Result<bool, ErrorDto> {
    st.gw
        .call_named("invite_finish", move || {
            Ok(qsc::facade::invite_finish(
                self_label.as_deref(),
                &alias,
                &relay,
                max,
            )?)
        })
        .await
}

#[tauri::command]
pub async fn invite_revoke(st: State<'_, AppState>, invite_id: String) -> Result<(), ErrorDto> {
    st.gw
        .call_named("invite_revoke", move || {
            Ok(qsc::facade::invite_revoke(&invite_id)?)
        })
        .await
}

// NA-0750 (D-0031): the DTO-value instrument for the `qsl-fp-v1` fingerprint.
//
// ⚠ IN-CRATE BY NECESSITY, ruled at R365 §4 (ask D, option β). `identity_dto` is
// private, so an integration test under src-tauri/tests/ cannot reach it, and widening
// its visibility for a test's benefit was refused. This is the measured house pattern
// (settings.rs, markers.rs).
//
// The fixture is DETERMINISTIC — fixed bytes, no vault, no keygen, no I/O — because a
// generated identity differs on every run, which would make a before/after comparison
// differ for the wrong reason and pass vacuously.
#[cfg(test)]
mod tests {
    use super::*;

    /// ML-KEM-768 / ML-DSA-65 public-key-sized fixture. The byte values are arbitrary
    /// but FIXED, so every figure asserted below is reproducible by anyone.
    fn fixture() -> qsc::identity::IdentityPublicRecord {
        qsc::identity::IdentityPublicRecord {
            kem_pk: (0..1184usize).map(|i| ((i * 7 + 13) % 256) as u8).collect(),
            sig_pk: (0..1952usize)
                .map(|i| ((i * 11 + 29) % 256) as u8)
                .collect(),
        }
    }

    /// W1(i): `identity_dto` returns EXACTLY what qsc computes for the same record.
    /// Equality on the extracted values, never `contains`.
    ///
    /// ⚠ The routing constraint carried from NA-0749: the voice tier reaches the
    /// COMBINED identity fingerprint and nothing else. `fp` here comes from
    /// `identity_fingerprint_from_identity`, which is the sanctioned route.
    #[test]
    fn identity_dto_returns_exactly_the_qsc_computed_pair() {
        let rec = fixture();
        let fp = qsc::identity::identity_fingerprint_from_identity(&rec.kem_pk, &rec.sig_pk);
        let voice = qsc::identity::identity_voice_form(&fp);
        let dto = identity_dto(&rec);
        assert_eq!(
            dto.fingerprint, fp,
            "the DTO's fingerprint must BE the qsc value"
        );
        assert_eq!(
            dto.verify_code, voice,
            "the DTO's verify_code must BE the qsc voice form"
        );
    }

    /// W1 shape, both tiers of the ratified design: 64 lowercase hex with no prefix,
    /// and exactly 30 ASCII digits.
    ///
    /// ⚠ The 30-digit assertion is also what catches a regression into
    /// `identity_voice_form`'s documented `""` sentinel, which it returns for any input
    /// that is not a well-formed FULL form.
    #[test]
    fn identity_dto_shapes_are_the_ratified_two_tiers() {
        let dto = identity_dto(&fixture());
        assert_eq!(dto.fingerprint.len(), 64, "full form is 64 hex characters");
        assert!(
            dto.fingerprint
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "full form must be LOWERCASE hex, got {}",
            dto.fingerprint
        );
        assert!(
            !dto.fingerprint.starts_with("QSCFP-"),
            "the QSCFP- prefix is retired"
        );
        assert_eq!(dto.verify_code.len(), 30, "voice form is exactly 30 digits");
        assert!(
            dto.verify_code.bytes().all(|b| b.is_ascii_digit()),
            "voice form must be ASCII digits only, got {}",
            dto.verify_code
        );
    }

    /// W2 — NA-0748's V3 seal, INVERTED: across this pin bump the values MUST CHANGE.
    ///
    /// Both baselines were measured at the OLD pin `e917e7e8` on this exact fixture.
    /// Asserting only the NEW shape would pass against the old format too for the
    /// fingerprint's prefix check alone, so the old values are pinned here as the thing
    /// that must not reappear — a green in the old form would be a false pass.
    #[test]
    fn w2_the_values_moved_off_the_retired_format() {
        const OLD_FP: &str = "QSCFP-4527910e41bb92b4478d95ad8b42eee0";
        const OLD_CODE: &str = "4527-910E-41BB-92B4-V";
        let dto = identity_dto(&fixture());
        assert_ne!(dto.fingerprint, OLD_FP, "the fingerprint must MOVE");
        assert_ne!(dto.verify_code, OLD_CODE, "the verification code must MOVE");
        assert!(
            !dto.fingerprint.contains("QSCFP"),
            "no QSCFP token survives in the full form"
        );
        assert!(
            !dto.verify_code.contains('-'),
            "the grouped, check-charactered form is gone"
        );
    }
}

// ===== NA-0779 (spine `D-1422`; desktop `D-0048`): THE DEBUG LOG's FOUR COMMANDS =====
// STOP 002 sec 8 drew TWO (event, export); the live viewer (RULING 002 R2 (a)) needs a READ path
// and the switch needs a WRITE path that is not `settings_set` (its two-field arity is pinned),
// so the count is four. All four are plain SYNC commands like `marker_stats`: they read or
// write an in-memory ring and must not queue behind core calls on the serial gateway.

/// The switch's state as the pane needs it after a control action.
#[derive(serde::Serialize)]
pub struct DebugLogStateDto {
    pub on: bool,
    pub level: crate::settings::DebugLogLevel,
}

/// The viewer's poll: the lines since `since_seq` (at most `max`), the counts, the switch.
#[tauri::command]
pub fn debug_log_read(since_seq: u64, max: u32) -> crate::debug_log::ReadDto {
    crate::debug_log::DebugLog::global().read(since_seq, max.min(2048) as usize)
}

/// The switch and the level, PERSISTED through the settings file's 0600 writer and applied at
/// runtime; and Clear. `action` is a member of a closed set; anything else is refused.
#[tauri::command]
pub fn debug_log_control<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    st: State<'_, AppState>,
    action: String,
) -> Result<DebugLogStateDto, String> {
    use tauri::Emitter;
    let log = crate::debug_log::DebugLog::global();
    match action.as_str() {
        "on" | "off" | "level_events" | "level_detailed" => {
            let mut s = crate::settings::load(&st.data_dir);
            match action.as_str() {
                "on" => s.debug_log.on = true,
                "off" => s.debug_log.on = false,
                "level_events" => s.debug_log.level = crate::settings::DebugLogLevel::Events,
                _ => s.debug_log.level = crate::settings::DebugLogLevel::Detailed,
            }
            crate::settings::save(&st.data_dir, &s)?;
            log.apply_action(&action);
        }
        "clear" => {
            log.apply_action("clear");
        }
        _ => return Err("debug_log_action_refused".into()),
    }
    let state = DebugLogStateDto {
        on: log.is_on(),
        level: log.level(),
    };
    // The pill on every screen learns the switch by PUSH (the menu's own mechanism, lib.rs
    // `app.emit`), so a switch flipped by any caller -- the pane, the harness -- shows at once.
    let _ = app.emit(
        "debug-log-switch",
        serde_json::json!({"on": state.on, "level": state.level}),
    );
    Ok(state)
}

/// THE FRONT END's ONE DOOR: a `ui.*` name from the closed desktop vocabulary with typed fields;
/// a name or field outside it is REFUSED with a `gw.log action=refused` notice of its own and a
/// closed reason -- never a copy of what was offered.
#[tauri::command]
pub fn debug_log_event(
    name: String,
    fields: std::collections::BTreeMap<String, String>,
) -> Result<bool, String> {
    let pairs: Vec<(&str, &str)> = fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    crate::debug_log::DebugLog::global()
        .push_from_ui(&name, &pairs)
        .map_err(|r| r.as_str().to_string())
}

/// What the Copy button and the harness receive when no directory is given: the export's bytes.
#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum DebugLogExportDto {
    Written(crate::debug_log::ExportDto),
    Text {
        text: String,
        bytes: usize,
        sha256: String,
    },
}

/// THE EXPORT. With `dir`: ONE file written where the operator chose (the CA-file precedent: a
/// typed directory, no dialog plugin), created new at 0600, its path and sha256 returned. Without
/// `dir`: the SAME bytes returned as text and nothing written -- the Copy button's source, so the
/// clipboard carries exactly the export (RULING 002 R2 (b): one allowlist, one renderer).
#[tauri::command]
pub fn debug_log_export(dir: Option<String>) -> Result<DebugLogExportDto, String> {
    let log = crate::debug_log::DebugLog::global();
    let build = build_commit_or_unknown(option_env!("QSLD_BUILD_COMMIT"));
    match dir {
        Some(d) if !d.trim().is_empty() => {
            let dto = log.export_to_dir(std::path::Path::new(d.trim()), build)?;
            Ok(DebugLogExportDto::Written(dto))
        }
        _ => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let text = log.export_text(
                build,
                "0000000000000000",
                &qsc::output::event::utc_rfc3339_ms(now),
            );
            let sha = crate::debug_log::sha256_hex(text.as_bytes());
            Ok(DebugLogExportDto::Text {
                bytes: text.len(),
                sha256: sha,
                text,
            })
        }
    }
}

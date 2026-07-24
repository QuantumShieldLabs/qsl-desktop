//! D595 slice A — the acceptance flow matrix at the backend level, against
//! the REAL qsc surface on disk. "Kill and relaunch" is honest here because
//! the launch state machine derives everything from what exists on disk
//! (no onboarding-complete flag): a fresh `resolve_launch_state` IS what a
//! relaunched process computes. Delay/wipe timing uses the NA-0658
//! clock-injection seam (`unlock_guarded_at`) — no wall-clock sleeps.

use qsc::vault::protection::{
    lock, protection_status, unlock_guarded, unlock_guarded_at, wipe_after_failed_unlocks_arm,
    GuardedUnlockOutcome as O,
};
use qsl_desktop_app::commands::erase_all_impl;
use qsl_desktop_app::state::{resolve_launch_state, LaunchState, SELF_LABEL};
use qsl_desktop_app::{bootstrap, paths, settings};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
}

fn fresh_env(tmp: &tempfile::TempDir) {
    lock(None);
    bootstrap(tmp.path()).expect("bootstrap");
}

/// (c′) The reduced deferred path: fresh dir → S0 → vault → identity → S2
/// with NO server configured anywhere (slice A has no server surface at
/// all; the settings file carries only the autolock key).
#[test]
fn c_prime_deferred_path_to_honest_disconnected() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S0);

    // wizard step 1 (vault) — the command validates empty/mismatch before
    // this point; the core sequence is init + guarded unlock:
    qsc::vault::vault_init_with_passphrase("deferred-path-pass").expect("init");
    assert!(matches!(
        unlock_guarded("deferred-path-pass").expect("unlock"),
        O::Unlocked
    ));

    // wizard step 2 (identity):
    let rec = qsc::identity::identity_ensure(SELF_LABEL).expect("identity");
    let fp = qsc::identity::identity_fingerprint_from_identity(&rec.kem_pk, &rec.sig_pk);
    assert!(!fp.is_empty());
    let code = qsc::identity::format_verification_code_from_fingerprint(&fp);
    assert!(!code.is_empty());

    // relaunch resolves S2. Slice B adds a Server pane, but this flow never
    // configures a relay — so the profile still carries ONLY the autolock key
    // (relay_url is omitted while empty), and the footer's honest state is
    // "no server configured".
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S2);
    let v = serde_json::to_value(settings::load(tmp.path())).unwrap();
    let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(
        keys,
        vec!["autolock_minutes"],
        "an unconfigured profile carries no relay key"
    );
    lock(None);
}

/// (d) The interruption matrix. Kill after vault creation → relaunch
/// resolves S1 → unlock → identity resumes; kill after identity → relaunch
/// resolves S2.
#[test]
fn d_interruption_matrix() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    // "kill after vault creation": vault exists, identity does not, process
    // state gone (lock() models the dead process's absent memory).
    qsc::vault::vault_init_with_passphrase("interrupt-pass").expect("init");
    lock(None);
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S1);

    // relaunch: unlock first, then the wizard resumes at identity —
    // identity_ensure is idempotent and safe to (re-)run.
    assert!(matches!(
        unlock_guarded("interrupt-pass").expect("unlock"),
        O::Unlocked
    ));
    let rec1 = qsc::identity::identity_ensure(SELF_LABEL).expect("identity");

    // "kill after identity": relaunch resolves S2; identity_ensure re-run
    // returns the SAME record (no mutation).
    lock(None);
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S2);
    assert!(matches!(
        unlock_guarded("interrupt-pass").expect("unlock"),
        O::Unlocked
    ));
    let rec2 = qsc::identity::identity_ensure(SELF_LABEL).expect("identity idempotent");
    assert_eq!(rec1.kem_pk, rec2.kem_pk);
    assert_eq!(rec1.sig_pk, rec2.sig_pk);
    lock(None);
}

/// (e) The unlock lifecycle: honest typed feedback, the escalating delay
/// (clock-injected), lock()/re-entry (the autolock firing path), and the
/// counter reset on success.
#[test]
fn e_unlock_lifecycle_typed_feedback_delay_and_reentry() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    qsc::vault::vault_init_with_passphrase("right-pass").expect("init");
    lock(None);

    let t0 = 1_000_000u64;
    // failures 1-2 are free (retry_after 0):
    match unlock_guarded_at("wrong-1", t0).unwrap() {
        O::Rejected {
            failed_unlocks,
            retry_after_s,
        } => {
            assert_eq!((failed_unlocks, retry_after_s), (1, 0));
        }
        o => panic!("expected Rejected, got {o:?}"),
    }
    match unlock_guarded_at("wrong-2", t0 + 1).unwrap() {
        O::Rejected {
            failed_unlocks,
            retry_after_s,
        } => {
            assert_eq!((failed_unlocks, retry_after_s), (2, 0));
        }
        o => panic!("expected Rejected, got {o:?}"),
    }
    // 3rd failure engages the 5 s delay:
    match unlock_guarded_at("wrong-3", t0 + 2).unwrap() {
        O::Rejected {
            failed_unlocks,
            retry_after_s,
        } => {
            assert_eq!((failed_unlocks, retry_after_s), (3, 5));
        }
        o => panic!("expected Rejected, got {o:?}"),
    }
    // inside the window: Delayed, nothing counted, remaining wait as a VALUE
    // (this is exactly what the unlock screen renders as the countdown):
    match unlock_guarded_at("right-pass", t0 + 4).unwrap() {
        O::Delayed {
            failed_unlocks,
            retry_after_s,
        } => {
            assert_eq!(failed_unlocks, 3);
            assert_eq!(retry_after_s, 3);
        }
        o => panic!("expected Delayed, got {o:?}"),
    }
    // after the window the correct passphrase unlocks and resets:
    assert!(matches!(
        unlock_guarded_at("right-pass", t0 + 8).unwrap(),
        O::Unlocked
    ));
    let s = protection_status().unwrap();
    assert_eq!(s.failed_unlocks, 0);

    // the autolock firing path is the one-call lock(); re-entry works:
    assert!(qsc::vault_unlocked());
    lock(None);
    assert!(!qsc::vault_unlocked());
    assert!(!qsc::vault::has_process_passphrase());
    assert!(matches!(
        unlock_guarded_at("right-pass", t0 + 9).unwrap(),
        O::Unlocked
    ));
    lock(None);
}

/// (e, wipe arm) Armed wipe-after-N triggers at N, the vault is gone, and
/// the app lands in S0 (the honest wiped notice is UI; the state transition
/// is proven here). Unarmed default is separately proven safe below.
#[test]
fn e_armed_wipe_lands_s0() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    qsc::vault::vault_init_with_passphrase("wipe-pass").expect("init");
    assert!(matches!(unlock_guarded("wipe-pass").unwrap(), O::Unlocked));
    wipe_after_failed_unlocks_arm(2).expect("arm");
    let st = protection_status().unwrap();
    assert_eq!(st.wipe_after, Some(2));
    lock(None);

    let t0 = 2_000_000u64;
    assert!(matches!(
        unlock_guarded_at("wrong-1", t0).unwrap(),
        O::Rejected {
            failed_unlocks: 1,
            ..
        }
    ));
    match unlock_guarded_at("wrong-2", t0 + 1).unwrap() {
        O::Wiped { marker } => {
            assert_eq!(marker, qsc::store::QSC_ERR_VAULT_WIPED_AFTER_FAILED_UNLOCKS);
        }
        o => panic!("expected Wiped, got {o:?}"),
    }
    assert!(!paths::vault_file(tmp.path()).exists());
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S0);
}

/// The unarmed DEFAULT is safe: many wrong attempts never wipe; the correct
/// passphrase still unlocks (delays skipped via clock injection).
#[test]
fn unarmed_default_never_wipes() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    qsc::vault::vault_init_with_passphrase("keep-pass").expect("init");
    lock(None);
    let mut t = 3_000_000u64;
    for i in 0..10u32 {
        match unlock_guarded_at("wrong", t).unwrap() {
            O::Rejected {
                failed_unlocks,
                retry_after_s,
            } => {
                assert_eq!(failed_unlocks, i + 1);
                t += retry_after_s + 1;
            }
            o => panic!("expected Rejected on attempt {i}, got {o:?}"),
        }
    }
    assert!(paths::vault_file(tmp.path()).exists());
    assert!(matches!(
        unlock_guarded_at("keep-pass", t).unwrap(),
        O::Unlocked
    ));
    lock(None);
}

/// The two-step forgotten-passphrase erase: app-level file removal only,
/// lands S0, never touches the CLI profile (guard + non-interference both
/// proven). The typed-phrase gate itself is command-level and constant.
#[test]
fn erase_all_lands_s0_and_never_touches_cli_dir() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    // a fake CLI profile lives under this XDG_CONFIG_HOME for the duration:
    let cli_home = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", cli_home.path());
    let cli_qsc = cli_home.path().join("qsc");
    std::fs::create_dir_all(&cli_qsc).unwrap();
    let cli_vault = cli_qsc.join("vault.qsv");
    std::fs::write(&cli_vault, b"cli-vault-bytes").unwrap();
    assert!(paths::cli_vault_present());

    fresh_env(&tmp);
    qsc::vault::vault_init_with_passphrase("forgotten").expect("init");
    lock(None);
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S1);

    // locked, passphrase forgotten → erase (the UI reaches this only through
    // two deliberate steps and the typed phrase):
    erase_all_impl(tmp.path()).expect("erase");
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S0);
    assert!(!paths::vault_file(tmp.path()).exists());

    // the CLI profile is byte-untouched:
    assert_eq!(std::fs::read(&cli_vault).unwrap(), b"cli-vault-bytes");

    // the guard refuses to erase the CLI dir itself:
    assert_eq!(
        erase_all_impl(&cli_qsc).unwrap_err(),
        "erase_refused_cli_dir"
    );
    std::env::remove_var("XDG_CONFIG_HOME");
}

/// NOT part of the suite (ignored): the scripted-acceptance seeding hook.
/// Run explicitly with QSLD_SEED_DIR (+ QSLD_SEED_MODE=vault|vault_identity,
/// QSLD_SEED_PASS) to pre-seed a THROWAWAY data dir for the xvfb-run
/// screenshot acceptance. Test-binary-only; the app itself has no env
/// passphrase ingress of any kind.
#[test]
#[ignore]
fn seed_acceptance_dir() {
    let dir = std::path::PathBuf::from(std::env::var("QSLD_SEED_DIR").expect("QSLD_SEED_DIR"));
    let mode = std::env::var("QSLD_SEED_MODE").unwrap_or_else(|_| "vault".into());
    let pass = std::env::var("QSLD_SEED_PASS").unwrap_or_else(|_| "acceptance-pass".into());
    let _g = env_lock().lock().unwrap();
    lock(None);
    bootstrap(&dir).expect("bootstrap");
    qsc::vault::vault_init_with_passphrase(&pass).expect("seed init");
    if mode == "vault_identity" {
        assert!(matches!(unlock_guarded(&pass).unwrap(), O::Unlocked));
        qsc::identity::identity_ensure(SELF_LABEL).expect("seed identity");
    }
    lock(None);
}

/// The tokened core destroy (Settings → Danger zone): wrong passphrase
/// refused with the vault intact; the correct passphrase + token destroys;
/// the app lands in S0.
#[test]
fn destroy_requires_correct_passphrase_then_lands_s0() {
    let _g = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    fresh_env(&tmp);

    qsc::vault::vault_init_with_passphrase("destroy-pass").expect("init");
    lock(None);

    let bad = qsc::vault::protection::DestroyConfirmToken::confirm_with_passphrase("wrong");
    assert!(qsc::vault::protection::destroy_with_passphrase("wrong", bad).is_err());
    assert!(paths::vault_file(tmp.path()).exists());

    let good = qsc::vault::protection::DestroyConfirmToken::confirm_with_passphrase("destroy-pass");
    qsc::vault::protection::destroy_with_passphrase("destroy-pass", good).expect("destroy");
    assert!(!paths::vault_file(tmp.path()).exists());
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S0);
    assert!(!qsc::vault_unlocked());
}

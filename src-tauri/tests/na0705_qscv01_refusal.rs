//! NA-0705 (D640 §2c(i), A2.2; R185 §2.6; R188 §4.4) — THE QSCV01 REFUSAL INSTRUMENT.
//!
//! The qsc pin bump crosses a `QSCV01` -> `QSCV02` vault-format HARD BREAK with no
//! migration and no dual-format read (D628 Ruling 2). A vault written by the shipped
//! build, met by the bumped desktop, must refuse by ITS OWN NAME — and must never be
//! mistaken for a wrong passphrase, because that mistake is destructive: the guard
//! counts every `Err` as a failed attempt (`protection.rs:156` is_ok -> `:175`
//! increment -> `:180` wipe), so an armed wipe-after-N erases the vault after N
//! CORRECT passphrases.
//!
//! ⚠ NO OTHER GATE IN THIS LANE CAN FAIL ON THIS. The compile passes (no signature
//! moved), and the suite, the six gui-driver flows and the rig e2e walk all create
//! FRESH vaults, which are `QSCV02` and never meet the old envelope. This file is the
//! purpose-built instrument that closes that hole.
//!
//! BOTH DOORS are covered (F-2): `unlock_attempt` reaches the refusal through
//! `unlock_guarded`, and `destroy_vault` reaches it independently — the destroy path
//! peeks the envelope through the same parser BEFORE it examines the passphrase.

use qsc::vault::protection::{
    lock, protection_status, wipe_after_failed_unlocks_arm, wipe_after_failed_unlocks_disarm,
};
use qsl_desktop_app::commands::{destroy_vault_impl, unlock_attempt_impl};
use qsl_desktop_app::{bootstrap, paths};
use std::sync::{Mutex, OnceLock};

/// The passphrase is always the CORRECT one. That is the whole point: the defect is
/// that a correct passphrase reads as a wrong one.
const PASS: &str = "qscv01-correct-pass";

fn env_lock() -> &'static Mutex<()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
}

/// Write a vault with the CURRENT build, then rewind its 6-byte magic to the shipped
/// build's `QSCV01`. Everything after the magic is a genuinely valid envelope — and
/// that is faithful, because the version arm sits at the magic check, BEFORE any key
/// derivation or decryption. This is exactly the "recognized-but-old" input that
/// `classify_vault_magic` maps to `KnownOld`.
fn make_qscv01_vault(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    lock(None);
    bootstrap(tmp.path()).expect("bootstrap");
    // `bootstrap` installs qsc's panic-redaction hook (lib.rs:309), which rewrites every
    // panic — including a test assertion failure — to `code=panic_redacted`. Correct in
    // the shipped app, blinding in a test. Drop it so this instrument's failures are
    // readable; the hook's own behaviour is not what we are measuring here.
    let _ = std::panic::take_hook();
    qsc::vault::vault_init_with_passphrase(PASS).expect("init");
    lock(None);
    let vf = paths::vault_file(tmp.path());
    let mut bytes = std::fs::read(&vf).expect("read vault");
    assert_eq!(
        &bytes[..6],
        b"QSCV02",
        "fixture premise: the bumped build must write QSCV02, else this instrument \
         is not testing what it claims"
    );
    bytes[..6].copy_from_slice(b"QSCV01");
    std::fs::write(&vf, &bytes).expect("write QSCV01 fixture");
    vf
}

/// The serialized `kind` the frontend actually switches on — asserting the wire string
/// rather than the Rust variant keeps this instrument compilable, and therefore
/// BEHAVIOURALLY red, before the remediation exists.
fn unlock_kind(data_dir: &std::path::Path) -> String {
    let dto = unlock_attempt_impl(data_dir, PASS).expect("unlock must not hard-error");
    serde_json::to_value(&dto).expect("serialize")["kind"]
        .as_str()
        .expect("kind")
        .to_string()
}

// ---------------------------------------------------------------- DOOR 1: unlock

#[test]
fn qscv01_unlock_refuses_with_its_own_name_not_as_a_wrong_passphrase() {
    let _g = env_lock().lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    make_qscv01_vault(&tmp);
    wipe_after_failed_unlocks_disarm().expect("disarm");

    let observed = unlock_kind(tmp.path());
    println!("OBSERVED unlock kind = {observed:?} (want \"version_unsupported\")");
    assert_eq!(
        observed, "version_unsupported",
        "a CORRECT passphrase against a QSCV01 vault must refuse by its own name; \
         'rejected' here is the app telling the user 'Wrong passphrase'"
    );
}

#[test]
fn qscv01_unlock_does_not_burn_a_failed_attempt() {
    let _g = env_lock().lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    make_qscv01_vault(&tmp);
    wipe_after_failed_unlocks_disarm().expect("disarm");

    let before = protection_status().expect("status").failed_unlocks;
    let _ = unlock_attempt_impl(tmp.path(), PASS);
    let after = protection_status().expect("status").failed_unlocks;
    println!("OBSERVED failed_unlocks before={before} after={after} (want equal)");

    assert_eq!(
        after, before,
        "a version refusal is not an authentication failure and must not advance the \
         failed-unlock counter"
    );
}

#[test]
fn qscv01_unlock_never_wipes_even_with_the_armed_limit_reached() {
    let _g = env_lock().lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let vf = make_qscv01_vault(&tmp);

    const N: u32 = 3;
    wipe_after_failed_unlocks_arm(N).expect("arm");

    for i in 1..=N {
        let kind = unlock_kind(tmp.path());
        println!(
            "OBSERVED attempt {i}/{N}: kind={kind:?} vault_exists={}",
            vf.exists()
        );
        assert_ne!(
            kind, "wiped",
            "attempt {i} of {N} with the CORRECT passphrase wiped the vault"
        );
        assert!(
            vf.exists(),
            "the vault file was destroyed on attempt {i} of {N} — by a correct passphrase"
        );
    }

    wipe_after_failed_unlocks_disarm().expect("disarm");
    assert!(
        vf.exists(),
        "the vault must survive the armed limit entirely"
    );
}

// --------------------------------------------------------------- DOOR 2: destroy

#[test]
fn qscv01_destroy_refuses_by_name() {
    let _g = env_lock().lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let vf = make_qscv01_vault(&tmp);

    let err = destroy_vault_impl(tmp.path(), PASS).expect_err("destroy must refuse");
    assert!(
        err.contains("vault_version_unsupported"),
        "destroy must name the version refusal; got {err:?}"
    );
    assert!(vf.exists(), "nothing may be destroyed on a refusal");
}

#[test]
fn qscv01_destroy_does_not_burn_a_failed_attempt() {
    let _g = env_lock().lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    make_qscv01_vault(&tmp);
    wipe_after_failed_unlocks_disarm().expect("disarm");

    let before = protection_status().expect("status").failed_unlocks;
    let _ = destroy_vault_impl(tmp.path(), PASS);
    let after = protection_status().expect("status").failed_unlocks;

    assert_eq!(
        after, before,
        "the destroy door must not advance the unlock failure counter either"
    );
}

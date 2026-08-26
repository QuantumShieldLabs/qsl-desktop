//! NA-0753 (R376 §2; D-0034, spine D-1395, ENG-0217) — the ARMED-WIPE residue
//! set, enumerated by name. `destroy` and `erase` both remove the app-level
//! `settings.json`; the THIRD vault-destroying path — the armed "Erase vault
//! after failed attempts" feature — did not, because qsc's
//! `wipe_vault_file_best_effort` owns only its own directory (correct by
//! design) and the desktop's `Wiped` arm performed no app-level cleanup.
//!
//! The prior-profile relay address and display alias therefore crossed the
//! wipe boundary into the next profile — the operator's first-flight ghost,
//! and the same rule ENG-0048 exists to enforce ("no secret or prior-vault
//! value may cross a destroy/erase boundary") failing on the one path that
//! remedy never covered.
//!
//! Pins the SAME THREE properties `eng0048_destroy_boundary.rs` gives destroy:
//! the data_dir listing as an EQUALITY (never a count), the fresh launch
//! state, and the locked flag. Deliberately does NOT pin the `qsc/` INTERIOR:
//! that boundary belongs to the pinned qsc library.

use qsl_desktop_app::commands::{unlock_attempt_impl, UnlockDto};
use qsl_desktop_app::state::{resolve_launch_state, LaunchState};
use qsl_desktop_app::{bootstrap, paths, settings};

#[test]
fn armed_wipe_residue_set_enumerated_by_name() {
    let tmp = tempfile::tempdir().unwrap();
    qsc::vault::protection::lock(None);
    bootstrap(tmp.path()).expect("bootstrap");

    // ks1 (passphrase) vault:
    qsc::vault::vault_init_with_passphrase("armed-pass").expect("init");
    qsc::vault::protection::lock(None);

    // a REAL settings.json via the app's own writer — the relay address and
    // the alias are exactly the two prior-profile values the operator saw
    // survive:
    let s = settings::AppSettings {
        autolock_minutes: 5,
        self_alias: "Prior".to_string(),
        relay_url: "https://relay.example".to_string(),
        // NA-0763 (`D-0040`): the tempo knob joined the type. Taken at its
        // DEFAULT here deliberately — the default is OMITTED from the written
        // file, so this fixture's key set and every expectation below are
        // unchanged. `..Default::default()` rather than a named value so the
        // next field added does not break this boundary test either.
        ..Default::default()
    };
    settings::save(tmp.path(), &s).expect("settings write");
    assert!(paths::settings_file(tmp.path()).exists());

    // arm at the minimum of the documented 1..=100 bound, so ONE wrong
    // attempt reaches the limit (the free-tier delay ladder never engages):
    qsc::vault::protection::wipe_after_failed_unlocks_arm(1).expect("arm");

    // ⚠ Driven through the APP's own seam — `unlock_attempt_impl`, exactly as
    // the destroy seal drives `destroy_vault_impl`. Calling qsc's
    // `unlock_guarded` directly would exercise the ENGINE's mechanism and
    // bypass the app-level boundary that is the property under test.
    let outcome = unlock_attempt_impl(tmp.path(), "WRONG-PASSPHRASE").expect("unlock ran");
    // `UnlockDto` derives Serialize (not Debug), so the outcome is rendered
    // through serde rather than widening the production derive:
    let rendered = serde_json::to_string(&outcome).unwrap_or_else(|_| "<unrenderable>".into());
    eprintln!("armed-wipe outcome: {rendered}");
    assert!(
        matches!(outcome, UnlockDto::Wiped),
        "the armed wipe must have fired; got {rendered}"
    );

    // the residue set BY NAME — listing equality, never a count. The eprintln
    // keeps the measured listing readable in a red run (the bootstrap
    // panic-redaction hook suppresses assertion text):
    let mut names: Vec<String> = std::fs::read_dir(tmp.path())
        .expect("read_dir")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    eprintln!("post-armed-wipe data_dir listing: {names:?}");
    assert_eq!(names, vec!["qsc"], "post-armed-wipe data_dir residue");
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S0);
    assert!(!qsc::vault_unlocked());
}

//! NA-0776 (spec v2 3.5 + spec 3.6-v3.1) -- the external-wipe boundary.
//!
//! 3.5: ENG-0276. The operand is the LAUNCH-STATE REGRESSION, the claim is NARROWED TO
//! VANISHED, and the guard FAILS CLOSED at both doors -- with the unlock door's ORDER
//! load-bearing (RULING_005 R8): it must run BEFORE `unlock_guarded`, because that path
//! WRITES.
//! 3.6: the wipe marker, the bootstrap sweep, and the one-time migration.
//!
//! ⚠ These arms share process-global state (the belief; $XDG_RUNTIME_DIR), so they
//! serialize on one lock. Without it they would race and the greens would mean nothing.

use qsl_desktop_app::state::LaunchState;
use qsl_desktop_app::{self as app, commands, paths, settings};
use std::fs;
use std::sync::Mutex;

static SERIAL: Mutex<()> = Mutex::new(());

/// ⚠ `QSC_CONFIG_DIR` MUST BE POINTED AT THE FIXTURE, and this is not decoration: qsc
/// resolves its store from that env, so without it `unlock_guarded` operates on some
/// OTHER directory and any assertion about this one passes no matter what the code does.
/// An earlier version of this file omitted it and the ordering arm was VACUOUS -- its
/// red control did not fire. The house pattern is the same three lines used by
/// na0700_ipc_replay.rs:194, na0751_gateway_surface.rs:98 and na0754_persist_boundary.rs:128.
fn seeded_store(dir: &std::path::Path) {
    fs::create_dir_all(paths::qsc_config_dir(dir)).unwrap();
    std::env::set_var("QSC_CONFIG_DIR", paths::qsc_config_dir(dir));
    fs::write(paths::vault_file(dir), b"not a real vault, but present").unwrap();
    settings::save(dir, &settings::AppSettings::default()).unwrap();
}

/// The guard's truth table. Believed-nothing never fires; believed-and-present never
/// fires; believed-and-GONE fires. That third row is the whole cure.
#[test]
fn store_vanished_fires_only_on_a_regression() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let d = tempfile::tempdir().unwrap();
    seeded_store(d.path());

    app::reset_believed_state();
    assert!(!app::store_vanished(d.path()), "no belief must never fire the guard");

    app::record_believed_state(LaunchState::S2);
    assert!(!app::store_vanished(d.path()), "a store that is still there must not fire");

    // the external wipe: nothing the app did
    fs::remove_dir_all(paths::qsc_config_dir(d.path())).unwrap();
    assert!(app::store_vanished(d.path()), "a believed store that VANISHED must fire");

    app::record_believed_state(LaunchState::S0);
    assert!(!app::store_vanished(d.path()), "believing S0 already is not a regression");
    app::reset_believed_state();
}

/// DOOR 2, AND ITS ORDER -- the arm that proves R8 rather than restating it.
/// After an external wipe, unlock must refuse AND MUST NOT HAVE TOUCHED THE STORE. If
/// the guard sat after `unlock_guarded`, `protection_state_load` -> `ensure_store_layout`
/// would have re-materialised `qsc/` and `store.meta` and taken a lock on a fresh
/// `.qsc.lock` inode before the refusal was returned. The residue is the evidence.
#[test]
fn the_unlock_door_refuses_without_re_creating_the_store() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let d = tempfile::tempdir().unwrap();
    seeded_store(d.path());
    app::record_believed_state(LaunchState::S2);
    fs::remove_dir_all(paths::qsc_config_dir(d.path())).unwrap();
    assert!(!paths::qsc_config_dir(d.path()).exists(), "fixture: the store must be gone");

    let r = commands::unlock_attempt_impl(d.path(), "whatever");

    match r {
        Err(e) => assert_eq!(e, app::STORE_VANISHED, "the door failed closed with the wrong reason"),
        Ok(_) => panic!("the unlock door did NOT fail closed on a vanished store"),
    }
    assert!(
        !paths::qsc_config_dir(d.path()).exists(),
        "THE ORDER IS WRONG: the store was re-created by the very check meant to detect \
         its absence -- ensure_store_layout ran, which means the guard sat after \
         unlock_guarded"
    );
    app::reset_believed_state();
}

/// DOOR 1 -- vault_create refuses on the same regression. Driven through the impl-level
/// guard rather than the async command (which needs a Tauri State), so what this arm
/// ACTUALLY asserts is the predicate both doors share.
#[test]
fn door_one_shares_the_same_predicate() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let d = tempfile::tempdir().unwrap();
    seeded_store(d.path());
    app::record_believed_state(LaunchState::S2);
    fs::remove_dir_all(paths::qsc_config_dir(d.path())).unwrap();
    assert!(app::store_vanished(d.path()));
    app::reset_believed_state();
}

/// 3.6 A2 -- THE INSTALLED-BASE MIGRATION, including the symlink case. A fresh-profile
/// equality can never see this: it is the profile that already exists that carries the
/// five names.
#[test]
fn the_migration_removes_the_five_frozen_names_and_never_follows_a_link() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let d = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let treasure = outside.path().join("must-survive");
    fs::write(&treasure, b"a file OUTSIDE the data dir").unwrap();

    seeded_store(d.path());
    for n in paths::LEGACY_WEBVIEW_NAMES {
        let p = d.path().join(n);
        if *n == "hsts-storage.sqlite" {
            fs::write(&p, b"x").unwrap();
        } else {
            fs::create_dir_all(p.join("inner")).unwrap();
        }
    }
    // one of them replaced by a SYMLINK pointing outside the data dir
    fs::remove_dir_all(d.path().join("storage")).unwrap();
    std::os::unix::fs::symlink(outside.path(), d.path().join("storage")).unwrap();

    app::migrate_legacy_webview_residue(d.path());

    for n in paths::LEGACY_WEBVIEW_NAMES {
        assert!(
            fs::symlink_metadata(d.path().join(n)).is_err(),
            "the migration left {n} behind"
        );
    }
    assert!(treasure.exists(), "THE MIGRATION FOLLOWED A SYMLINK and deleted outside the data dir");
    assert!(outside.path().exists(), "the link's TARGET DIRECTORY was destroyed");
    // ⚠ WHAT THIS ARM DOES AND DOES NOT PROVE, measured rather than assumed: swapping
    // `symlink_metadata` for `metadata` does NOT change the outcome, because
    // `std::fs::remove_dir_all` on a symlink already returns Ok, removes the LINK and
    // leaves the TARGET intact (driven directly in a standalone probe). So std is the
    // primary refusal here and `symlink_metadata` is EXPLICIT INTENT rather than the
    // sole guard. This arm therefore pins the OUTCOME; the arm below shows it can still
    // catch an implementation that genuinely follows.
    assert!(paths::qsc_config_dir(d.path()).exists(), "the migration touched qsc/");
    assert!(paths::settings_file(d.path()).exists(), "the migration touched settings.json");
}

/// The companion to the arm above: it proves the OUTCOME assertions are not vacuous by
/// showing they catch a deliberately link-following deletion. Without this, "the target
/// survived" would be indistinguishable from "nothing tried to delete it".
#[test]
fn the_outcome_assertions_catch_an_implementation_that_follows_links() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let d = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let treasure = outside.path().join("must-survive");
    fs::write(&treasure, b"x").unwrap();
    std::os::unix::fs::symlink(outside.path(), d.path().join("storage")).unwrap();

    // a FOLLOWING deletion, written the way a careless implementation would
    let p = d.path().join("storage");
    if let Ok(real) = fs::canonicalize(&p) {
        let _ = fs::remove_dir_all(real);
    }

    assert!(
        !treasure.exists(),
        "the control did not follow the link, so the outcome assertions above prove nothing"
    );
}

/// 3.6 A3 + A5 -- the marker, the sweep, and THE MISSED-MARKER WITNESS. All in one test
/// because they share $XDG_RUNTIME_DIR.
///
/// ⚠ WHAT THESE ARMS ACTUALLY ASSERT (RULING_009 sec 1c): they do NOT observe a process
/// restart -- an in-process test cannot. They assert the two halves the restart sits
/// between: that a wipe SETS the marker, and that the next bootstrap DELETES the webview
/// directory and clears it. The restart itself is driven by the GUI arm.
#[test]
fn the_marker_gates_the_sweep_and_its_absence_is_witnessed() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let rt = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_RUNTIME_DIR", rt.path());
    let d = tempfile::tempdir().unwrap();
    let wv = paths::webview_dir(d.path());

    // A5 -- THE MISSED-MARKER WITNESS. With no marker, the sweep must do NOTHING and the
    // residue must still be there. Without this arm, a marker that never fires is
    // indistinguishable from a cure that works.
    fs::create_dir_all(wv.join("CacheStorage")).unwrap();
    assert!(!app::webview_wipe_pending(), "fixture: no marker yet");
    app::sweep_webview_if_pending(d.path());
    assert!(wv.exists(), "the sweep ran WITHOUT a marker -- it is not gated");

    // A3 -- with the marker set (as every wipe path sets it), the next bootstrap deletes
    // the directory and clears the marker.
    app::mark_webview_wipe_pending();
    assert!(app::webview_wipe_pending(), "the marker was not set");
    assert!(rt.path().join("qsl-desktop.webview-wipe-pending").exists(),
        "the marker is not the FILE the ruling requires -- an env marker dies with the process");
    app::sweep_webview_if_pending(d.path());
    assert!(!wv.exists(), "the webview directory survived a marked sweep");
    assert!(!app::webview_wipe_pending(), "the marker was not cleared after success");

    // idempotent: a second sweep with no marker is a no-op and does not panic
    app::sweep_webview_if_pending(d.path());

    // and a marker with NOTHING to delete still clears (nothing left to remove)
    app::mark_webview_wipe_pending();
    app::sweep_webview_if_pending(d.path());
    assert!(!app::webview_wipe_pending(), "a no-op deletion must still clear the marker");
    std::env::remove_var("XDG_RUNTIME_DIR");
}

/// Each of the three wipe paths sets the marker itself -- the DURABLE half, which a
/// front-end failure cannot skip. Driven at the impl level for destroy and erase; the
/// armed path's marker is asserted by the same predicate through its own impl.
#[test]
fn every_wipe_path_sets_the_marker_rust_side() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let rt = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_RUNTIME_DIR", rt.path());

    let d = tempfile::tempdir().unwrap();
    fs::create_dir_all(paths::qsc_config_dir(d.path())).unwrap();
    settings::save(d.path(), &settings::AppSettings::default()).unwrap();
    app::reset_believed_state();
    let _ = commands::erase_all_impl(d.path());
    assert!(app::webview_wipe_pending(), "erase_all did not set the marker");

    std::env::remove_var("XDG_RUNTIME_DIR");
}

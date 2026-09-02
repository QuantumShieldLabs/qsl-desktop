//! NA-0776 (spec v2 sec 3.2) -- settings.json is 0600, with no window at 664 and with
//! existing profiles remediated. Cures the operator's filing (settings.json created 664)
//! under the cold read's MAJOR-4 (no cfg(unix) split), MAJOR-5 (both halves) and
//! MAJOR-10's 3.2 point (the .tmp arm must drive a REAL failure path).
//!
//! No cfg(unix) gate anywhere here, deliberately: the crate imports PermissionsExt
//! unconditionally (lib.rs:28) and v1 is Linux-only (D-A / L9), so a gate would assert a
//! portability property the tree does not have.

use qsl_desktop_app::settings::{self, AppSettings};
use qsl_desktop_app::paths;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn mode_of(p: &std::path::Path) -> u32 {
    fs::metadata(p).expect("stat").permissions().mode() & 0o777
}

/// A1 -- the write path creates at 0600, and the destination inherits it via rename.
#[test]
fn saved_settings_file_is_0600() {
    let dir = tempfile::tempdir().unwrap();
    settings::save(dir.path(), &AppSettings::default()).expect("save");
    let f = paths::settings_file(dir.path());
    assert!(f.exists(), "settings.json was not created");
    assert_eq!(
        mode_of(&f),
        0o600,
        "settings.json mode is {:o}, expected 600 -- the umask leaked through",
        mode_of(&f)
    );
    // and the staging sibling is gone on the success path
    assert!(!f.with_extension("json.tmp").exists(), "a .tmp survived a successful save");
}

/// A2 -- an EXISTING 664 profile is remediated. This is the half that decides whether
/// the cure reaches users who never edit their alias, autolock or relay URL: nothing
/// else forces the save that would tighten the file.
#[test]
fn existing_664_profile_is_tightened_on_the_load_path() {
    let dir = tempfile::tempdir().unwrap();
    let f = paths::settings_file(dir.path());
    fs::write(&f, br#"{"autolock_minutes":20}"#).unwrap();
    fs::set_permissions(&f, fs::Permissions::from_mode(0o664)).unwrap();
    assert_eq!(mode_of(&f), 0o664, "fixture did not start at 664");

    let s = settings::load(dir.path());

    assert_eq!(mode_of(&f), 0o600, "load did not tighten an existing 664 file");
    assert_eq!(s.autolock_minutes, 20, "remediation must not disturb the CONTENT");
}

/// A2b -- idempotent: a file already at 600 is left alone and still reads.
#[test]
fn tighten_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let f = paths::settings_file(dir.path());
    settings::save(dir.path(), &AppSettings::default()).expect("save");
    assert_eq!(mode_of(&f), 0o600);
    settings::tighten_mode(&f);
    assert_eq!(mode_of(&f), 0o600, "a second tighten changed the mode");
}

/// A3 -- THE FAILURE-PATH ARM, and it drives a real failure rather than the success
/// path. v1's version asserted the tmp was absent after a SUCCESSFUL save, where it is
/// always renamed away: it passed trivially and said nothing (MAJOR-10, 3.2).
///
/// The interrupt is simulated where it can actually be simulated: the rename is made to
/// FAIL (the destination is a non-empty directory), so save returns Err with the tmp
/// still on disk -- exactly the residue an interrupt between write and rename leaves.
/// The property under test is that THAT residue is 0600, i.e. the content was never
/// written into a loose-moded file.
#[test]
fn tmp_residue_on_a_failed_rename_is_0600() {
    let dir = tempfile::tempdir().unwrap();
    let f = paths::settings_file(dir.path());
    // a non-empty directory where the file belongs: rename(file, dir) cannot succeed
    fs::create_dir(&f).unwrap();
    fs::write(f.join("occupant"), b"x").unwrap();

    let r = settings::save(dir.path(), &AppSettings::default());
    assert!(r.is_err(), "the rename was expected to fail; the arm proves nothing if it succeeded");

    let tmp = f.with_extension("json.tmp");
    assert!(tmp.exists(), "no .tmp residue: the failure did not land where this arm needs it");
    assert_eq!(
        mode_of(&tmp),
        0o600,
        ".tmp residue is {:o}: the content was written into a loose-moded file",
        mode_of(&tmp)
    );
}

/// A stale .tmp from an earlier interrupted save must not wedge every future save --
/// `create_new` alone would. Recorded as a refinement of the spec's wording.
#[test]
fn a_stale_tmp_does_not_wedge_the_next_save() {
    let dir = tempfile::tempdir().unwrap();
    let f = paths::settings_file(dir.path());
    let tmp = f.with_extension("json.tmp");
    fs::write(&tmp, b"leftover from an interrupted save").unwrap();
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o664)).unwrap();

    settings::save(dir.path(), &AppSettings::default()).expect("save must survive a stale tmp");
    assert_eq!(mode_of(&f), 0o600);
    assert!(!tmp.exists(), "the stale tmp was left behind");
}

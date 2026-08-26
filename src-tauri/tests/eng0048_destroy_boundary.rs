//! ENG-0048 (D631 §7 as amended; D-0024, spine D-1337) — the desktop-level
//! destroy residue set, enumerated by name: after a tokened destroy through
//! the app's own destroy path, the data_dir holds exactly `qsc/` — no
//! `settings.json`, no `.tmp` staging sibling. The listing is asserted as an
//! EQUALITY, never a count. Deliberately does NOT pin the `qsc/` INTERIOR:
//! that boundary belongs to the pinned qsc library (asserted by qsc's own
//! residue test at head; the desktop pin `ab5041cd` is pre-D-1336).

use qsl_desktop_app::commands::destroy_vault_impl;
use qsl_desktop_app::state::{resolve_launch_state, LaunchState};
use qsl_desktop_app::{bootstrap, paths, settings};

#[test]
fn destroy_residue_set_enumerated_by_name() {
    let tmp = tempfile::tempdir().unwrap();
    qsc::vault::protection::lock(None);
    bootstrap(tmp.path()).expect("bootstrap");

    // ks1 (passphrase) vault:
    qsc::vault::vault_init_with_passphrase("residue-pass").expect("init");
    qsc::vault::protection::lock(None);

    // a REAL settings.json via the app's own writer — alias + relay
    // non-default (the two survive-as-field verdicts and the dies-as-field
    // alias all die WITH the file under Shape A):
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

    // the tokened destroy through the app's destroy path:
    destroy_vault_impl(tmp.path(), "residue-pass").expect("destroy");

    // the residue set BY NAME — listing equality, never a count. The
    // eprintln keeps the measured listing readable in a red run (the
    // bootstrap panic-redaction hook suppresses assertion text):
    let mut names: Vec<String> = std::fs::read_dir(tmp.path())
        .expect("read_dir")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    eprintln!("post-destroy data_dir listing: {names:?}");
    assert_eq!(names, vec!["qsc"], "post-destroy data_dir residue");
    assert_eq!(resolve_launch_state(tmp.path()), LaunchState::S0);
    assert!(!qsc::vault_unlocked());
}

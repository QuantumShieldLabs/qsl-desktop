//! GUI-local NON-SECRET settings. Anything secret lives in the qsc vault —
//! never here; the allowlist test pins the file's key set.

use crate::paths::settings_file;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

pub const AUTOLOCK_DEFAULT_MINUTES: u32 = 60;

/// The delivery-ladder tempo knob (spine `D-1404`, desktop `D-0040`; the
/// ladder's section 6 / 1.4). Three positions are STORED from day one so no
/// migration is needed later, while rung 1 honours only the tempo halves.
///
/// ⚠ `PullOnly` is NOT dead weight and is NOT a synonym by accident: at rung 1
/// everything is pull, so the ladder defines its rung-1 semantics as exactly
/// the private tempo (design 1.4, verbatim: "pull-only = the private tempo").
/// It becomes distinguishable only when a socket rung exists.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Tempo {
    /// Tight beat. The blessed DEFAULT (operator, 2026-08-26).
    #[default]
    Instant,
    /// Long beat: presence is revealed at a coarser grain.
    Private,
    /// Rung 1: the private tempo.
    PullOnly,
}

impl Tempo {
    /// `skip_serializing_if` for the tempo field: the DEFAULT position is
    /// OMITTED from the file, so a profile that never chose a tempo keeps the
    /// prior key set EXACTLY and `settings_key_allowlist`'s default-case
    /// assertion stays byte-unchanged. Same shape as `self_alias`/`relay_url`.
    fn is_default(&self) -> bool {
        matches!(self, Tempo::Instant)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppSettings {
    /// Idle autolock: ON by default at 60 minutes; adjustable; the wizard is
    /// exempt (enforced UI-side). 0 is VALID and means never-auto-lock (the
    /// D598 operator decision; the UI's idle timer never fires at 0 and the
    /// danger banner renders; range validation is UI-side per F2).
    pub autolock_minutes: u32,
    /// The optional local-only display alias ("What should this device call
    /// you?"); empty renders as "You". NON-SECRET by ruling (a display
    /// label). Empty is OMITTED from the serialized file so a fresh profile
    /// keeps the slice-A key set exactly.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub self_alias: String,
    /// The relay endpoint the Server pane points at (slice B). NON-SECRET by
    /// ruling: a public address, not a credential — the bearer token and the
    /// CA-file path live in the qsc vault, never here (D609 R6). Empty is
    /// OMITTED so an unconfigured profile keeps the prior key set exactly,
    /// the same `self_alias` pattern. Added to the allowlist test
    /// deliberately. The `deny_unknown_fields` downgrade property is
    /// KNOWINGLY untouched (D609 R6): a slice-B file carrying `relay_url`
    /// fails to parse on a slice-A reader and falls back to the default —
    /// a pre-existing class (`self_alias` already carries it), and
    /// downgrades are not a supported path.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub relay_url: String,
    /// The delivery tempo (`D-0040`). NON-SECRET: a rhythm selector, not a
    /// credential. OMITTED while it holds the default so an existing profile
    /// keeps its prior key set exactly — the `self_alias`/`relay_url` pattern.
    /// The `deny_unknown_fields` downgrade property is inherited KNOWINGLY and
    /// unchanged (D609 R6): a file carrying `tempo` falls back to the default
    /// on a reader that predates it, and downgrades are not a supported path.
    /// ⚠ NOTHING IN THIS LANE WRITES THIS FIELD: `settings_set` keeps its
    /// two-field arity and the visible control lands Lane C-era.
    #[serde(default, skip_serializing_if = "Tempo::is_default")]
    pub tempo: Tempo,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            autolock_minutes: AUTOLOCK_DEFAULT_MINUTES,
            self_alias: String::new(),
            relay_url: String::new(),
            tempo: Tempo::default(),
        }
    }
}

pub fn load(data_dir: &Path) -> AppSettings {
    let path = settings_file(data_dir);
    // NA-0776 (3.2 / MAJOR-5): remediate a profile created before the 0600 cure.
    // Nothing else forces the save that would tighten it -- the only two non-test
    // callers of `save` are `settings_set` and `relay_config_set`, and neither runs at
    // launch -- so a user who never edits their alias, autolock or relay URL would keep
    // a 664 file indefinitely. Also called from `bootstrap`, which IS a launch path.
    tighten_mode(&path);
    match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

pub fn save(data_dir: &Path, s: &AppSettings) -> Result<(), String> {
    let path = settings_file(data_dir);
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?;
    // NA-0776 (3.2 / MAJOR-5): the tmp is CREATED at 0600 and the bytes are written
    // into a file that was never group- or world-readable. `fs::write` creates at the
    // umask and puts the CONTENT in before any chmod could run -- a short window that
    // contains the data. `rename` carries the tmp's mode, so the destination is 600
    // with no window of its own and no chmod-after.
    // A stale tmp from an interrupted save is removed first: `create_new` would
    // otherwise wedge every future save on a leftover file.
    if tmp.exists() {
        fs::remove_file(&tmp).map_err(|e| e.to_string())?;
    }
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|e| e.to_string())?;
    f.write_all(&bytes).map_err(|e| e.to_string())?;
    drop(f);
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

/// NA-0776 (3.2 / MAJOR-5): tighten an existing settings file to 0600, in place.
/// Idempotent, and deliberately QUIET: a failure to tighten must never stop the app
/// from reading a file it can otherwise read. No cfg(unix) gate -- the crate imports
/// `PermissionsExt` unconditionally at lib.rs:28 and v1 is Linux-only (D-A / L9), so a
/// gate here would assert a portability property the tree does not have (MAJOR-4).
pub fn tighten_mode(path: &Path) {
    if let Ok(md) = fs::metadata(path) {
        if md.permissions().mode() & 0o777 != 0o600 {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        }
    }
}

/// The env name of the test-only tempo seam. Deliberately the `QSLD_` family
/// the harness already injects per launch (`runner.py` sets `QSLD_DATA_DIR`
/// the same way), and the same shape as `paths.rs`'s `$QSLD_DATA_DIR`
/// override — a precedent, not an invention.
pub const TICK_OVERRIDE_ENV: &str = "QSLD_TICK_MS";

/// Parse the seam value. A PURE function of its argument so it is testable
/// without mutating the process environment — `std::env::set_var` is unsound
/// across the test harness's threads, so the impure half stays one line long
/// and the logic that can be wrong is tested directly.
///
/// Refuses zero and refuses anything unparseable: a malformed seam must leave
/// the shipped tempo standing rather than produce a 0 ms busy-loop.
pub fn parse_tick_override(raw: Option<&str>) -> Option<u64> {
    match raw?.trim().parse::<u64>() {
        Ok(ms) if ms > 0 => Some(ms),
        _ => None,
    }
}

/// Read the seam from the environment. ⚠⚠ WHERE THIS VALUE LIVES IS THE WHOLE
/// POINT OF RULING `R4`, and it is answered STRUCTURALLY: the seam is carried
/// on `AppInfoDto` — a `Serialize`-ONLY type that is never deserialized and has
/// no `save` path — and it is deliberately NOT a field of `AppSettings`, the
/// persisted type. So `settings::save` cannot round-trip a test tempo because
/// the value is not of a shape it can ever hold, rather than because a caller
/// remembered to avoid it.
///
/// ⚠ The first design routed this through a `load_effective` on the settings
/// read path with `#[serde(skip)]`. That was MEASURED broken before it shipped:
/// `skip` omits the field from serialization, and Tauri's IPC serializes with
/// the same impl, so the value could never have reached the UI at all.
pub fn tick_override_from_env() -> Option<u64> {
    parse_tick_override(std::env::var(TICK_OVERRIDE_ENV).ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_sixty_minutes() {
        assert_eq!(AppSettings::default().autolock_minutes, 60);
    }

    /// 0 is VALID and means never-auto-lock (D598 item 2): it saves and
    /// loads like any other value; no backend range bound exists (F2 —
    /// range validation is UI-side and visible).
    #[test]
    fn roundtrip_and_zero_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = AppSettings {
            autolock_minutes: 30,
            self_alias: "Vic".to_string(),
            ..Default::default()
        };
        save(dir.path(), &s).unwrap();
        assert_eq!(load(dir.path()), s);
        s.autolock_minutes = 0;
        save(dir.path(), &s).unwrap();
        assert_eq!(load(dir.path()).autolock_minutes, 0);
    }

    /// The settings file is non-secret by construction: its serialized key
    /// set is exactly the allowlist. A new field must be added here
    /// deliberately (and must never be a secret). The D596 self-alias is a
    /// local display label: OMITTED while empty (a fresh profile keeps the
    /// slice-A key set), present exactly once when set.
    #[test]
    fn settings_key_allowlist() {
        let v = serde_json::to_value(AppSettings::default()).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["autolock_minutes"]);

        let with_alias = AppSettings {
            self_alias: "Vic".to_string(),
            ..Default::default()
        };
        let v = serde_json::to_value(&with_alias).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["autolock_minutes", "self_alias"]);

        // The D609 slice-B relay endpoint: NON-SECRET (a public address), added
        // to the allowlist deliberately. OMITTED while empty (skip_serializing_if),
        // so an unconfigured profile keeps the prior key set; present exactly once
        // when set. NOTE: `to_value` builds a serde_json::Map (BTreeMap), so the
        // key ORDER here is ALPHABETICAL, not struct-declaration order (the file
        // written by `to_vec_pretty(&AppSettings)` uses declaration order); this
        // test pins the key SET, which is what "non-secret by construction" needs.
        let with_relay = AppSettings {
            relay_url: "https://relay.example".to_string(),
            ..Default::default()
        };
        let v = serde_json::to_value(&with_relay).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["autolock_minutes", "relay_url"]);

        // NA-0763 (`D-0040`): the tempo knob, added to the allowlist DELIBERATELY
        // and NON-SECRET by construction (a rhythm selector, not a credential).
        // ⚠ The DEFAULT position is OMITTED (`skip_serializing_if`), which is why
        // the default-case assertion at the top of this test is untouched: a
        // profile that never chose a tempo keeps exactly the prior key set.
        let with_tempo = AppSettings {
            tempo: Tempo::Private,
            ..Default::default()
        };
        let v = serde_json::to_value(&with_tempo).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["autolock_minutes", "tempo"]);

        // And the blessed default really is the omitted one, stated as its own
        // assertion so a future default flip cannot pass this test silently.
        assert_eq!(AppSettings::default().tempo, Tempo::Instant);

        let both = AppSettings {
            self_alias: "Vic".to_string(),
            relay_url: "https://relay.example".to_string(),
            ..Default::default()
        };
        let v = serde_json::to_value(&both).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["autolock_minutes", "relay_url", "self_alias"]);
    }

    /// An alias-bearing file from this version loads on a reader that also
    /// understands only the slice-A key (serde default) — and an old file
    /// without the key loads here with the empty default.
    #[test]
    fn self_alias_absent_defaults_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = settings_file(dir.path());
        std::fs::write(&path, br#"{ "autolock_minutes": 20 }"#).unwrap();
        let s = load(dir.path());
        assert_eq!(s.autolock_minutes, 20);
        assert_eq!(s.self_alias, "");
    }

    /// ⚠⚠ RULING `R4`'s BINDING CONSTRAINT: no test-tempo value may ever reach
    /// the persisted file. Guarded here as a PROPERTY over every field
    /// combination the type admits, rather than over one hand-picked struct.
    ///
    /// ⚠ RED ARM, MEASURED (not asserted): while an earlier revision carried a
    /// `tick_override_ms` field on `AppSettings`, dropping its `#[serde(skip)]`
    /// turned BOTH this family and `settings_key_allowlist` red, the file
    /// reading `{"autolock_minutes":60,"tick_override_ms":137}`. Re-add any
    /// seam-shaped field and these arms fail again.
    #[test]
    fn no_tick_seam_key_can_reach_the_persisted_file() {
        let combos = [
            AppSettings::default(),
            AppSettings {
                tempo: Tempo::Private,
                ..Default::default()
            },
            AppSettings {
                tempo: Tempo::PullOnly,
                self_alias: "Vic".to_string(),
                relay_url: "https://relay.example".to_string(),
                ..Default::default()
            },
        ];
        for s in combos.iter() {
            let dir = tempfile::tempdir().unwrap();
            save(dir.path(), s).unwrap();
            let raw = std::fs::read_to_string(settings_file(dir.path())).unwrap();
            assert!(
                !raw.contains("tick"),
                "a tick-seam token reached the settings file: {raw}"
            );
            assert_eq!(&load(dir.path()), s, "round-trip must be lossless");
        }
    }

    /// The seam parser is total and refuses everything that would be worse than
    /// leaving the shipped tempo standing — a 0 ms beat most of all.
    #[test]
    fn the_seam_parser_refuses_zero_and_junk() {
        assert_eq!(parse_tick_override(None), None);
        assert_eq!(parse_tick_override(Some("")), None);
        assert_eq!(parse_tick_override(Some("0")), None);
        assert_eq!(parse_tick_override(Some("-5")), None);
        assert_eq!(parse_tick_override(Some("abc")), None);
        assert_eq!(parse_tick_override(Some("400")), Some(400));
        assert_eq!(parse_tick_override(Some("  400  ")), Some(400));
    }

    /// A file written by a tempo-bearing profile round-trips, and a file that
    /// predates the key loads with the blessed default.
    #[test]
    fn tempo_roundtrips_and_absent_defaults_instant() {
        let dir = tempfile::tempdir().unwrap();
        let s = AppSettings {
            tempo: Tempo::PullOnly,
            ..Default::default()
        };
        save(dir.path(), &s).unwrap();
        assert_eq!(load(dir.path()).tempo, Tempo::PullOnly);

        let path = settings_file(dir.path());
        std::fs::write(&path, br#"{ "autolock_minutes": 20 }"#).unwrap();
        assert_eq!(load(dir.path()).tempo, Tempo::Instant);
    }
}

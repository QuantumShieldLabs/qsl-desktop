//! GUI-local NON-SECRET settings. Anything secret lives in the qsc vault —
//! never here; the allowlist test pins the file's key set.

use crate::paths::settings_file;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const AUTOLOCK_DEFAULT_MINUTES: u32 = 60;

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
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            autolock_minutes: AUTOLOCK_DEFAULT_MINUTES,
            self_alias: String::new(),
            relay_url: String::new(),
        }
    }
}

pub fn load(data_dir: &Path) -> AppSettings {
    let path = settings_file(data_dir);
    match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

pub fn save(data_dir: &Path, s: &AppSettings) -> Result<(), String> {
    let path = settings_file(data_dir);
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?;
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
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
        let mut s = AppSettings::default();
        s.autolock_minutes = 30;
        s.self_alias = "Vic".to_string();
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

        let mut with_alias = AppSettings::default();
        with_alias.self_alias = "Vic".to_string();
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
        let mut with_relay = AppSettings::default();
        with_relay.relay_url = "https://relay.example".to_string();
        let v = serde_json::to_value(&with_relay).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["autolock_minutes", "relay_url"]);

        let mut both = AppSettings::default();
        both.self_alias = "Vic".to_string();
        both.relay_url = "https://relay.example".to_string();
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
}

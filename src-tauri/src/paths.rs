//! App-scoped path resolution. The GUI owns its own data directory, distinct
//! from the CLI's default configuration directory (R8: one profile, one
//! program). QSC_CONFIG_DIR is pointed at `<data dir>/qsc` once at bootstrap,
//! which is what makes the F2 launch-state probe deterministic.

use std::path::{Path, PathBuf};

pub const APP_DIR_NAME: &str = "org.quantumshieldlabs.qsldesktop";

/// The vault store filename inside a qsc config dir. F2 (D595): qsc resolves
/// its store at `$QSC_CONFIG_DIR/vault.qsv`; this name is qsc-format-internal
/// and the coupling is recorded on the improvement ledger with a pub-probe
/// remedy owed to a future pin-advance lane.
pub const QSC_VAULT_FILE_NAME: &str = "vault.qsv";

/// The app-scoped data directory: `$QSLD_DATA_DIR` override (tests and
/// scripted acceptance), else `$XDG_DATA_HOME/<id>`, else
/// `$HOME/.local/share/<id>`.
pub fn app_data_dir() -> Result<PathBuf, String> {
    if let Ok(v) = std::env::var("QSLD_DATA_DIR") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    if let Ok(v) = std::env::var("XDG_DATA_HOME") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v).join(APP_DIR_NAME));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Ok(PathBuf::from(home).join(".local/share").join(APP_DIR_NAME));
        }
    }
    Err("app_data_dir_unresolvable".into())
}

pub fn qsc_config_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("qsc")
}

pub fn vault_file(data_dir: &Path) -> PathBuf {
    qsc_config_dir(data_dir).join(QSC_VAULT_FILE_NAME)
}

pub fn settings_file(data_dir: &Path) -> PathBuf {
    data_dir.join("settings.json")
}

/// The CLI's default config dir, mirrored WITHOUT consulting QSC_CONFIG_DIR
/// (ours points into the app dir): `$XDG_CONFIG_HOME/qsc` else
/// `$HOME/.config/qsc`. Used only for the S0 courtesy notice and the erase
/// guard; never written.
pub fn cli_default_config_dir() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("XDG_CONFIG_HOME") {
        if !v.is_empty() {
            return Some(PathBuf::from(v).join("qsc"));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home).join(".config").join("qsc"));
        }
    }
    None
}

pub fn cli_vault_present() -> bool {
    cli_default_config_dir()
        .map(|d| d.join(QSC_VAULT_FILE_NAME).exists())
        .unwrap_or(false)
}

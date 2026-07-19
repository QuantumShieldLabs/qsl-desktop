//! qsl-desktop — slice A: the serverless skeleton (D595 / spine D-1282 /
//! repo-local D-0002). Tauri v2 shell consuming qsc in-process as a
//! rev-pinned git dependency. Slice A contains ZERO networking code; the
//! server-connectivity surface is slice B (owed).

pub mod commands;
pub mod gateway;
pub mod markers;
pub mod paths;
pub mod settings;
pub mod state;

use gateway::CoreGateway;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub struct AppState {
    pub data_dir: PathBuf,
    pub gw: CoreGateway,
}

/// Startup rule (a): process environment + output policy + marker routing are
/// fixed ONCE, before any thread exists and before the Tauri runtime is
/// built. QSC_CONFIG_DIR points into the app-scoped dir (0700); the output
/// policy is the redacting default (set-once in qsc, R2 — chosen deliberately
/// here); marker routing is InApp so no marker ever prints to a stdout
/// nobody reads.
pub fn bootstrap(data_dir: &Path) -> Result<(), String> {
    create_private_dir(data_dir)?;
    let qsc_dir = paths::qsc_config_dir(data_dir);
    create_private_dir(&qsc_dir)?;
    std::env::set_var("QSC_CONFIG_DIR", &qsc_dir);
    qsc::output::init_output_policy(false);
    qsc::output::set_marker_routing(qsc::output::MarkerRouting::InApp);
    qsc::output::install_panic_redaction_hook();
    Ok(())
}

pub(crate) fn create_private_dir(p: &Path) -> Result<(), String> {
    fs::create_dir_all(p).map_err(|e| format!("create {}: {e}", p.display()))?;
    fs::set_permissions(p, fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("chmod {}: {e}", p.display()))?;
    Ok(())
}

pub fn run() {
    let data_dir = paths::app_data_dir().expect("app data dir unresolvable");
    bootstrap(&data_dir).expect("bootstrap failed");
    let app_state = AppState {
        data_dir,
        gw: CoreGateway::default(),
    };
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::launch_state,
            commands::cli_vault_present,
            commands::vault_create,
            commands::identity_ensure,
            commands::identity_show,
            commands::unlock_attempt,
            commands::lock_now,
            commands::protection_status,
            commands::wipe_arm,
            commands::wipe_disarm,
            commands::settings_get,
            commands::settings_set,
            commands::destroy_vault,
            commands::erase_all,
            commands::marker_stats,
            commands::core_busy,
            commands::app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running qsl-desktop");
}

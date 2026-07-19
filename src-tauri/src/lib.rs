//! qsl-desktop — slice A: the serverless skeleton (D595 / spine D-1282 /
//! repo-local D-0002; round-2 design pass D597 / spine D-1284 / D-0004).
//! Tauri v2 shell consuming qsc in-process as a rev-pinned git dependency.
//! Slice A contains ZERO networking code; the server-connectivity surface
//! is slice B (owed).

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
use tauri::menu::{
    AboutMetadataBuilder, MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem,
    SubmenuBuilder,
};
use tauri::{Emitter, Manager};

pub struct AppState {
    pub data_dir: PathBuf,
    pub gw: CoreGateway,
}

/// Item 15 (D597): handles to the two state-dependent File entries. R1:
/// disabling for live state is honesty, not a placeholder — both entries
/// are wired and enabled exactly while an unlocked surface is showing.
struct MenuHandles {
    settings: MenuItem<tauri::Wry>,
    lock_now: MenuItem<tauri::Wry>,
}

/// Item 15 (R1): the frontend reports every surface change; File > Settings
/// and File > Lock now are enabled only while an unlocked surface (the main
/// window or Settings view) is showing. Presentation state only — no core
/// call, no persistence.
#[tauri::command]
fn ui_surface_changed(app: tauri::AppHandle, surface: String) {
    let unlocked = surface == "scr-main" || surface == "scr-settings";
    if let Some(h) = app.try_state::<MenuHandles>() {
        let _ = h.settings.set_enabled(unlocked);
        let _ = h.lock_now.set_enabled(unlocked);
    }
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
        .setup(|app| {
            // Item 15 (D597): the native menu — the pinned tauri 2 core
            // menu API only; WORKING entries only, nothing unbuilt.
            let settings_item = MenuItemBuilder::with_id("qsl-settings", "Settings")
                .enabled(false)
                .build(app)?;
            let lock_item = MenuItemBuilder::with_id("qsl-lock-now", "Lock now")
                .enabled(false)
                .build(app)?;
            let file = SubmenuBuilder::new(app, "File")
                .item(&settings_item)
                .item(&lock_item)
                .separator()
                .item(&PredefinedMenuItem::quit(app, Some("Quit"))?)
                .build()?;
            let edit = SubmenuBuilder::new(app, "Edit")
                .item(&PredefinedMenuItem::cut(app, None)?)
                .item(&PredefinedMenuItem::copy(app, None)?)
                .item(&PredefinedMenuItem::paste(app, None)?)
                .item(&PredefinedMenuItem::select_all(app, Some("Select all"))?)
                .build()?;
            let view = SubmenuBuilder::new(app, "View")
                .item(&MenuItemBuilder::with_id("qsl-reload", "Reload").build(app)?)
                .item(&MenuItemBuilder::with_id("qsl-fullscreen", "Full screen").build(app)?)
                .build()?;
            // About: factual metadata only (name + version + the retained
            // honesty line) — claim discipline applies to menus too.
            let about_meta = AboutMetadataBuilder::new()
                .name(Some(commands::APP_DISPLAY_NAME.to_string()))
                .version(Some(env!("CARGO_PKG_VERSION").to_string()))
                .comments(Some(
                    "This build makes no network connections and no security-assurance claims."
                        .to_string(),
                ))
                .build();
            let help = SubmenuBuilder::new(app, "Help")
                .item(&PredefinedMenuItem::about(
                    app,
                    Some("About"),
                    Some(about_meta),
                )?)
                .build()?;
            let menu = MenuBuilder::new(app)
                .item(&file)
                .item(&edit)
                .item(&view)
                .item(&help)
                .build()?;
            app.set_menu(menu)?;
            app.manage(MenuHandles {
                settings: settings_item,
                lock_now: lock_item,
            });
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "qsl-settings" => {
                let _ = app.emit("menu-open-settings", ());
            }
            "qsl-lock-now" => {
                let _ = app.emit("menu-lock-now", ());
            }
            "qsl-reload" => {
                // The same full-reset mechanism item 13 relies on — safe by
                // construction: all durable state is backend-side.
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.eval("location.reload()");
                }
            }
            "qsl-fullscreen" => {
                if let Some(w) = app.get_webview_window("main") {
                    let fs = w.is_fullscreen().unwrap_or(false);
                    let _ = w.set_fullscreen(!fs);
                }
            }
            _ => {}
        })
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
            ui_surface_changed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running qsl-desktop");
}

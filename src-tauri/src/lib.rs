//! QSL desktop client — slices A–B: vault, identity and unlock lifecycle, plus
//! relay connectivity. Research-stage; makes no security-assurance claims.
//!
//! (D595 / spine D-1282 / repo-local D-0002; round-2 design pass D597 / spine
//! D-1284 / D-0004. Slice B's relay connectivity: D609 / spine D-1295.)
//! Tauri v2 shell consuming qsc in-process as a rev-pinned git dependency.
//!
//! NA-0686 / D-1325 (ENG-0088): the previous first line still described this
//! crate as a slice-A-only shell with no networking code, a claim slice B
//! retired. It survived two slices because the claim-discipline guard read this
//! file for a different needle; it is now covered by
//! `claim_discipline_covers_cargo_metadata_and_module_docs`.
//!
//! ⚠ That guard matches on the retired phrases themselves, so this note
//! deliberately paraphrases rather than quotes them — a comment explaining a
//! retired claim must not reintroduce it. The guard caught exactly that mistake
//! in this very comment while it was being written.

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
use std::sync::Mutex;
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

/// Item 10 (D598/E.1) as amended by round 4a (D601/F1): ONE MODE PER PRE-MAIN
/// SURFACE. The window is still resized on the MODE transition (not
/// per-render) through the same single shared path; compact modes hide the
/// menu bar, the full mode shows it. Presentation state only — no core call.
///
/// The round-3 table poured FIVE screens with visibly different content
/// heights into TWO sizes, and whichever screen was shorter than its class
/// got the surplus as dead space — measured at Phase 1 as 153px (23.2% of the
/// window) on wizard step 1 and 164px (39.0%) on unlock. Each surface now
/// carries its own height so the content ends at the padding.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WindowMode {
    /// Wizard step 1, "Create your vault" — the tallest pre-main surface.
    WizardVault,
    /// Wizard step 2, "Your identity" — taller than step 1 by the
    /// verification code block and its explanatory copy.
    WizardIdentity,
    /// Unlock — the daily front door, and the shortest of the gates.
    Unlock,
    /// Erase everything — sized to the TALLER of its two states (the typed
    /// -phrase form and the 30-second countdown panel), since one window
    /// serves both without a resize between them.
    Erase,
    /// The wiped notice, reachable only from a failed-unlock wipe.
    Wiped,
    /// Settings: a VIEW whose content columns are capped, so its width is
    /// DERIVED from those caps rather than guessed. Split out of `Full` by
    /// NA-0680 Finding 1 — sharing a mode with the main window is why opening
    /// Settings never resized anything and left ~212px of dead space to the
    /// right of a 560px content column.
    Settings,
    /// The main window: a three-pane shell whose content pane is `1fr`, i.e.
    /// it has no natural content size and is MEANT to fill. It keeps a default
    /// size, and that is not the defect class Finding 1 covers — no instance in
    /// the acceptance flight named it.
    Main,
}

pub fn mode_for_surface(surface: &str) -> WindowMode {
    match surface {
        "scr-wizard-vault" => WindowMode::WizardVault,
        "scr-wizard-identity" => WindowMode::WizardIdentity,
        "scr-unlock" => WindowMode::Unlock,
        "scr-erase" => WindowMode::Erase,
        "scr-wiped" => WindowMode::Wiped,
        "scr-settings" => WindowMode::Settings,
        _ => WindowMode::Main, // scr-main
    }
}

/// (size, min-size, menu-visible) per mode — the E.1 window table.
///
/// The compact minimum is a single modest floor rather than "min == size" as
/// round 3 had it: F4's acceptance requires the verification code to stay
/// legible at every size the window can take, DEMONSTRATED SMALL, and a
/// window whose minimum equals its initial size cannot be dragged smaller at
/// all. The floor is what makes the wrap remedy observable.
///
/// It must sit at or below the SHORTEST pre-main window (the wiped notice at
/// 210) — a floor above it would be silently re-imposed by `set_min_size`
/// and the window would never take the size this table asks for.
pub const COMPACT_MIN: (f64, f64) = (360.0, 200.0);

/// The operator's chosen reading width for every pre-main surface, found by
/// hand-resizing the identity window until the copy composed correctly. Width
/// and height are COUPLED: narrowing to 360 wraps the body copy into more
/// lines, so these heights are MEASURED AT THIS WIDTH and are not valid at any
/// other. Changing this constant invalidates every height below.
pub const PRE_MAIN_WIDTH: f64 = 360.0;

/// Finding 1 (instance 4): the Settings width is DERIVED from the layout
/// constants it must contain, not chosen. `.settings-layout` is
/// `52px | 160px | 1fr`, `.pane` caps at 560px, and `.settings-pane` pads
/// 20px each side. A window wider than their sum is dead space by
/// construction — which is exactly what 1024 produced.
pub const SETTINGS_ICON_RAIL_W: f64 = 52.0;
pub const SETTINGS_NAV_RAIL_W: f64 = 160.0;
/// ⚠ CORRECTED by the acceptance flight: this is `.pane-form`'s cap (520), NOT
/// `.pane`'s (560). The first derivation used 560 and produced a window 40px
/// too wide — visible as ASYMMETRIC insets, 20px from the nav rail to the
/// start of a section hairline but 60px from its end to the window edge. The
/// hairlines span `.pane-form`, so `.pane-form` is the constant that decides
/// the width; `.pane`'s cap only bounds panes that carry no form and is never
/// the widest thing on screen.
pub const SETTINGS_PANE_MAX_W: f64 = 520.0;
pub const SETTINGS_PANE_PAD_W: f64 = 40.0; // --sp-x20 left + right
pub const SETTINGS_WIDTH: f64 =
    SETTINGS_ICON_RAIL_W + SETTINGS_NAV_RAIL_W + SETTINGS_PANE_MAX_W + SETTINGS_PANE_PAD_W;

pub fn window_mode_spec(mode: WindowMode) -> ((f64, f64), (f64, f64), bool) {
    // Heights measured headlessly at a 360px viewport in WebKit2 4.1 — the
    // same engine tauri uses on Linux — against the real ui/index.html, with
    // fitCode's shrink/wrap replicated so the verification code's rendered
    // size is included. Each is the natural content height plus the screen's
    // 28px top and bottom padding, rounded up to the next multiple of 5 so a
    // sub-pixel difference cannot clip the last element or trip the card's
    // overflow scrollbar. Measured -> landed: wizard-1 583->585,
    // wizard-2 620->625 (the operator's independent hand measurement was 621),
    // unlock 250->255, erase 273->275, wiped 217->220.
    match mode {
        WindowMode::WizardVault => ((PRE_MAIN_WIDTH, 585.0), COMPACT_MIN, false),
        WindowMode::WizardIdentity => ((PRE_MAIN_WIDTH, 625.0), COMPACT_MIN, false),
        WindowMode::Unlock => ((PRE_MAIN_WIDTH, 255.0), COMPACT_MIN, false),
        // Sized to the TALLER of its two states: the typed-phrase form
        // measured 273 and the countdown panel 253, and one window serves
        // both without a resize between them.
        WindowMode::Erase => ((PRE_MAIN_WIDTH, 275.0), COMPACT_MIN, false),
        WindowMode::Wiped => ((PRE_MAIN_WIDTH, 220.0), COMPACT_MIN, false),
        // Finding 1: width DERIVED (see SETTINGS_WIDTH); the minimum no longer
        // carries the old 800x600 floor, which would have silently re-imposed
        // itself over any content-driven width.
        WindowMode::Settings => ((SETTINGS_WIDTH, 700.0), (SETTINGS_WIDTH, 400.0), true),
        WindowMode::Main => ((1024.0, 700.0), (640.0, 400.0), true),
    }
}

struct WindowModeState(Mutex<Option<WindowMode>>);

/// The last height actually applied, so a content re-measure that changes
/// nothing does not call `set_size` on every keystroke.
struct AppliedHeight(Mutex<Option<f64>>);

/// R-14 as RE-SCOPED by the acceptance flight (Finding 1): the height a window
/// should take, given the frontend's MEASURED content height.
///
/// ⚠ THIS IS CONTENT-DRIVEN IN BOTH DIRECTIONS. THE EARLIER "FLOOR" IS GONE.
///
/// The first implementation returned `max(table, measured)`, on my reasoning
/// that the per-surface table values encoded a chosen reading composition worth
/// preserving. **That was an inference the operator never stated, and the live
/// flight disproved it**: six of the seven observed instances were windows too
/// TALL, which is precisely a floor holding a window open when its content is
/// shorter. The unlock window is the proof in one surface — it used to clip,
/// then over-corrected to dead space, so its height was never tracking content
/// in either direction.
///
/// So: the measurement governs. The only clamp is the mode's own absolute
/// minimum, which exists so a window can never become un-draggable, not to
/// encode a preferred size. The table height survives ONLY as the pre-first-
/// report fallback — the window has to open at something before the frontend
/// has measured anything.
pub fn height_for(mode: WindowMode, measured_content: Option<f64>) -> f64 {
    let ((_, fallback), (_, min_h), _) = window_mode_spec(mode);
    match measured_content {
        Some(m) => m.max(min_h),
        None => fallback,
    }
}

/// Finding 1: the full size, both dimensions, one place. Width is the mode's
/// own (a reading width for pre-main, a DERIVED width for Settings); height
/// tracks content.
pub fn size_for(mode: WindowMode, measured_content: Option<f64>) -> (f64, f64) {
    let ((w, _), _, _) = window_mode_spec(mode);
    (w, height_for(mode, measured_content))
}

fn apply_window_mode(
    app: &tauri::AppHandle,
    w: &tauri::WebviewWindow<tauri::Wry>,
    mode: WindowMode,
    measured_content: Option<f64>,
) {
    let (_, min, menu_visible) = window_mode_spec(mode);
    let (width, height) = size_for(mode, measured_content);
    // E.1 order: set_min_size, then set_size, then center — the pinned
    // tauri 2 core window API only.
    let _ = w.set_min_size(Some(tauri::LogicalSize::new(min.0, min.1)));
    let _ = w.set_size(tauri::LogicalSize::new(width, height));
    let _ = w.center();
    // Menu visibility by ATTACHMENT, not gtk-hide: tao's set_visible(true)
    // is gtk show_all() on Linux, which resurrects hidden child widgets —
    // a merely-hidden menubar reappears whenever the F1 deferred first
    // show is processed. A REMOVED menubar has nothing to resurrect; the
    // full mode re-attaches the app-wide menu (still the pinned tauri 2
    // core menu API only).
    if menu_visible {
        if w.menu().is_none() {
            if let Some(m) = app.menu() {
                let _ = w.set_menu(m);
            }
        }
        let _ = w.show_menu();
    } else if w.menu().is_some() {
        let _ = w.remove_menu();
    }
}

/// Item 15 (R1): the frontend reports every surface change; File > Settings
/// and File > Lock now are enabled only while an unlocked surface (the main
/// window or Settings view) is showing. Item 10 (E.1) rides the same
/// report: the window mode is applied when it CHANGES, and the F1 launch
/// sequence shows the still-hidden window only after the first report has
/// sized it — no 1024x700 -> compact snap ever renders. Presentation state
/// only — no core call, no persistence.
/// ⚠ R-14: `content_height` is the frontend's MEASURED height for the active
/// pre-main surface, and it arrives on EVERY sync — not only on a mode change.
/// It has to: the autolock path calls `show("scr-unlock")` and writes
/// "Locked after inactivity." into the feedback line AFTERWARDS, so a resize
/// that fired only on the surface change would miss the very content that
/// motivated this fix. `None` means "not a pre-main surface" (or not
/// measurable), and the table governs alone.
#[tauri::command]
fn ui_surface_changed(app: tauri::AppHandle, surface: String, content_height: Option<f64>) {
    let unlocked = surface == "scr-main" || surface == "scr-settings";
    if let Some(h) = app.try_state::<MenuHandles>() {
        let _ = h.settings.set_enabled(unlocked);
        let _ = h.lock_now.set_enabled(unlocked);
    }
    let mode = mode_for_surface(&surface);
    if let Some(w) = app.get_webview_window("main") {
        let changed = {
            let st = app.state::<WindowModeState>();
            let mut cur = st.0.lock().unwrap_or_else(|p| p.into_inner());
            let changed = *cur != Some(mode);
            *cur = Some(mode);
            changed
        };
        let height = height_for(mode, content_height);
        let height_changed = {
            let st = app.state::<AppliedHeight>();
            let mut cur = st.0.lock().unwrap_or_else(|p| p.into_inner());
            let differs = *cur != Some(height);
            *cur = Some(height);
            differs
        };
        if changed {
            apply_window_mode(&app, &w, mode, content_height);
        } else if height_changed {
            // Same surface, content changed size — track it, without
            // re-centering or touching the menu. Guarded on an actual change so
            // a re-measure that agrees with the last one issues no window call.
            let (width, h) = size_for(mode, content_height);
            let _ = w.set_size(tauri::LogicalSize::new(width, h));
        }
        if !w.is_visible().unwrap_or(true) {
            let _ = w.show();
            let _ = w.set_focus();
        }
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
        .manage(WindowModeState(Mutex::new(None)))
        .manage(AppliedHeight(Mutex::new(None)))
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
                    // Slice B (D609 R4): the app now reaches a relay, so the
                    // "no network connections" clause is retired — but the
                    // surviving true clause STAYS: no security-assurance claims.
                    "This build makes no security-assurance claims.".to_string(),
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
            // F1 fail-open: the window launches hidden (tauri.conf.json
            // windows[0] visible:false) and is normally shown by the first
            // sized surface report. If the frontend never reports (a boot
            // fault), show the window anyway after a bounded wait — an
            // invisible app is the worse failure.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                if let Some(w) = handle.get_webview_window("main") {
                    if !w.is_visible().unwrap_or(true) {
                        let _ = w.show();
                    }
                }
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
            // slice B (D609 GATE 2): server connectivity — thin forwarders onto
            // the qsc surface, every qsc call through the serial blocking gate.
            commands::relay_config_get,
            commands::relay_config_set,
            commands::relay_test,
            commands::relay_token_set,
            commands::relay_token_clear,
            commands::relay_token_show,
            commands::relay_ca_file_set,
            commands::relay_ca_file_clear,
            commands::relay_ca_file_show,
            ui_surface_changed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running qsl-desktop");
}

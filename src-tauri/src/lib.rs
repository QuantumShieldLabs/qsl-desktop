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
pub mod debug_log;
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
struct MenuHandles<R: tauri::Runtime> {
    settings: MenuItem<R>,
    lock_now: MenuItem<R>,
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

fn apply_window_mode<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    w: &tauri::WebviewWindow<R>,
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
fn ui_surface_changed<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    surface: String,
    content_height: Option<f64>,
) {
    let unlocked = surface == "scr-main" || surface == "scr-settings";
    // NA-0779 (`D-0048`): `ui.surface` -- the screen id (one of seven, refused otherwise), from
    // the one place the front end already reports every transition.
    debug_log::DebugLog::global().ui_surface(&surface);
    if let Some(h) = app.try_state::<MenuHandles<R>>() {
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
/// NA-0776 (3.6-v3.1 sec 4) -- THE WIPE MARKER'S NAME under $XDG_RUNTIME_DIR.
pub const WIPE_MARKER_NAME: &str = "qsl-desktop.webview-wipe-pending";
/// The FALLBACK carrier, used ONLY when XDG_RUNTIME_DIR is unset. Its residual is
/// stated in the spec: under the fallback, ANY crash mid-delete loses the re-fire.
pub const WIPE_MARKER_ENV: &str = "QSLD_WEBVIEW_WIPE_PENDING";

/// The marker path, or None when XDG_RUNTIME_DIR is unset (the fallback's trigger).
pub fn wipe_marker_path() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR").map(|d| PathBuf::from(d).join(WIPE_MARKER_NAME))
}

/// Set IMMEDIATELY after a wipe returns success and BEFORE the restart. A FILE, not an
/// env var: an env marker dies with the process, so a crash inside the bootstrap delete
/// would lose it and the next MANUAL start would skip the re-fire silently, leaving
/// PRE-WIPE webview data after a wipe (RULING_008 sec 2).
pub fn mark_webview_wipe_pending() {
    match wipe_marker_path() {
        Some(p) => {
            let _ = fs::write(&p, b"");
        }
        None => std::env::set_var(WIPE_MARKER_ENV, "1"),
    }
}

pub fn webview_wipe_pending() -> bool {
    if let Some(p) = wipe_marker_path() {
        if p.exists() {
            return true;
        }
    }
    std::env::var_os(WIPE_MARKER_ENV).is_some()
}

fn clear_webview_wipe_pending() {
    if let Some(p) = wipe_marker_path() {
        let _ = fs::remove_file(&p);
    }
    std::env::remove_var(WIPE_MARKER_ENV);
}

/// The deletion itself. Runs at BOOTSTRAP, which is the only point in the process where
/// no webview exists: `run()` calls `bootstrap` before `configure_builder(...).setup()`,
/// and the window -- hence the WebContext -- is built inside that setup. Deleting under
/// a live WebContext is refused: WebKitGTK holds open handles and recreates directories
/// on the next write, and `window.location.reload()` does not reset it.
///
/// ⚠ THE MARKER IS CLEARED ONLY AFTER THE DELETION RETURNS SUCCESS. An I/O failure
/// LEAVES IT SET, so the deletion re-fires on any later start -- an interrupted delete
/// completes rather than being silently skipped. A missing directory counts as success:
/// there is nothing left to remove.
pub fn sweep_webview_if_pending(data_dir: &Path) {
    if !webview_wipe_pending() {
        return;
    }
    let wv = paths::webview_dir(data_dir);
    match fs::remove_dir_all(&wv) {
        Ok(()) => clear_webview_wipe_pending(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => clear_webview_wipe_pending(),
        Err(_) => { /* leave the marker SET: it re-fires on the next start */ }
    }
}

/// NA-0776 (3.6-v3.1 sec 6) -- the ONE-TIME installed-base migration. Removes the five
/// frozen names from the OLD location so a profile created before the cure is not left
/// exactly as broken as before it.
/// ⚠ `symlink_metadata`, never `metadata`: a symlink is removed AS A LINK and never
/// followed, so a link planted at one of these names cannot make this delete something
/// outside the data dir.
/// ⚠ It is written to run on EVERY start and to ACT once: after the relocation nothing
/// recreates those names at the old location, so the second start finds nothing. The
/// cost is five `symlink_metadata` calls; the benefit is no "have I migrated" flag to
/// go wrong.
pub fn migrate_legacy_webview_residue(data_dir: &Path) {
    for name in paths::LEGACY_WEBVIEW_NAMES {
        let p = data_dir.join(name);
        match fs::symlink_metadata(&p) {
            Ok(md) if md.is_dir() => {
                let _ = fs::remove_dir_all(&p);
            }
            Ok(_) => {
                let _ = fs::remove_file(&p);
            }
            Err(_) => {}
        }
    }
}

/// NA-0776 (spec v2 3.5) -- WHAT THIS PROCESS BELIEVES ABOUT THE STORE.
///
/// A process-scoped static rather than an `AppState` field, for two reasons: the belief
/// IS per-process (the same scope qsc uses for `PROCESS_PASSPHRASE`), and a new struct
/// field would force edits to three unrelated test construction sites for a value none
/// of them cares about.
///
/// THE OPERAND IS THE LAUNCH-STATE REGRESSION: a process that believed S1/S2 and now
/// resolves S0 has detected a wipe it did not perform, with ZERO new bytes on disk. The
/// two candidates the cold read refused are not used -- a desktop-written identity token
/// lands inside BOTH sealed listing-equality pins, and inode/dev is probabilistic
/// (inodes are recycled) and non-portable.
/// ⚠ NARROWED TO VANISHED. `store.meta` is a byte-identical constant on every store
/// forever, so a REPLACED store is not distinguishable by any operand available here;
/// the promotion's ENG-0276 amendment records that case as NOT COVERED.
static BELIEVED_STATE: std::sync::OnceLock<Mutex<Option<state::LaunchState>>> =
    std::sync::OnceLock::new();

fn believed_cell() -> &'static Mutex<Option<state::LaunchState>> {
    BELIEVED_STATE.get_or_init(|| Mutex::new(None))
}

/// Recorded when `launch_state` reports -- the only place the app forms a belief.
pub fn record_believed_state(s: state::LaunchState) {
    *believed_cell().lock().unwrap_or_else(|p| p.into_inner()) = Some(s);
}

pub fn believed_state() -> Option<state::LaunchState> {
    *believed_cell().lock().unwrap_or_else(|p| p.into_inner())
}

/// Test-only in practice: nothing in the product clears a belief.
pub fn reset_believed_state() {
    *believed_cell().lock().unwrap_or_else(|p| p.into_inner()) = None;
}

/// TRUE when this process believed it had a store and the store is now gone.
/// ⚠ TOCTOU, STATED: this NARROWS the hazard and never closes it. The check is separated
/// from the act it guards by a window an external wipe can still land in.
pub fn store_vanished(data_dir: &Path) -> bool {
    matches!(
        believed_state(),
        Some(state::LaunchState::S1) | Some(state::LaunchState::S2)
    ) && state::resolve_launch_state(data_dir) == state::LaunchState::S0
}

/// The refusal both doors return. FAILS CLOSED: the app refuses and requires a
/// relaunch; there is no silent in-place recovery.
pub const STORE_VANISHED: &str = "store_vanished_relaunch_required";

pub fn bootstrap(data_dir: &Path) -> Result<(), String> {
    // NA-0776 (3.6-v3.1): SWEEP FIRST, then the migration, then everything else -- the
    // internal order is ruled, so the two bootstrap residents cannot interleave wrongly.
    // Both run BEFORE any webview exists and before the settings chmod at the end.
    sweep_webview_if_pending(data_dir);
    migrate_legacy_webview_residue(data_dir);
    create_private_dir(data_dir)?;
    let qsc_dir = paths::qsc_config_dir(data_dir);
    create_private_dir(&qsc_dir)?;
    std::env::set_var("QSC_CONFIG_DIR", &qsc_dir);
    qsc::output::init_output_policy(false);
    qsc::output::set_marker_routing(qsc::output::MarkerRouting::InApp);
    // NA-0776 (3.3 / cold read MAJOR-8): PIN THE MARKER FORMAT. qsc chooses plain vs
    // jsonl from `QSC_MARK_FORMAT` in the launching environment (output/mod.rs:257-262).
    // Unpinned, a `QSC_MARK_FORMAT=jsonl` environment would make the notice classifier
    // match nothing and the footer stay EMPTY -- failing closed for privacy but OPEN
    // into silence for the surface's purpose, with every test still green. Pinning here,
    // beside the routing call, makes the parse total.
    std::env::set_var("QSC_MARK_FORMAT", "plain");
    qsc::output::install_panic_redaction_hook();
    // NA-0776 (3.2 / MAJOR-5): the launch-path half of the 0600 remediation. `load` is
    // NOT a launch path -- measured: its only callers are settings_get, settings_set,
    // relay_config_get and relay_config_set, none of which runs at startup -- so the
    // spec's "at launch" is satisfied HERE, and `load` keeps its own call as defence
    // in depth. Idempotent and quiet.
    settings::tighten_mode(&paths::settings_file(data_dir));
    Ok(())
}

pub(crate) fn create_private_dir(p: &Path) -> Result<(), String> {
    fs::create_dir_all(p).map_err(|e| format!("create {}: {e}", p.display()))?;
    fs::set_permissions(p, fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("chmod {}: {e}", p.display()))?;
    Ok(())
}

/// NA-0700 (D-0025): the run-path composition — managed state plus the EXACT
/// `invoke_handler` command set — applied to a caller-supplied builder, so the
/// IPC replay harness registers the same commands on the mock runtime that
/// `run()` registers on Wry. The handler list below is the one place the
/// registered set is written; `run()` composes through here.
pub fn configure_builder<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    app_state: AppState,
) -> tauri::Builder<R> {
    builder
        .manage(app_state)
        .manage(WindowModeState(Mutex::new(None)))
        .manage(AppliedHeight(Mutex::new(None)))
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
            commands::notice_list,
            commands::notice_dismiss,
            commands::restart_app,
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
            // NA-0754 (D-0035) — test-and-save-on-proof: the explicit-triple probe
            // that persists nothing, and the home directory the CA field needs to
            // expand a leading `~/` visibly before the path is used.
            commands::relay_probe,
            commands::home_dir,
            // NA-0751 (D-0032) — the Slice-4 gateway surface, THIRTEEN pass-through
            // wrappers over `qsc::facade` (twelve at NA-0751; `invite_clear` joined at
            // NA-0755 v2, and its registration is covered by the same seal). `invite_list_at` is deliberately absent:
            // it is a clock-injection seam and must not be reachable from the front end.
            commands::connect_status,
            commands::contact_list,
            commands::contact_requests,
            commands::contact_request_accept,
            commands::contact_request_ignore,
            commands::contact_request_block,
            // NA-0765 (`D-0042`): Rename. The facade verb landed at NA-0764 and was
            // unreachable until this line existed.
            commands::contact_set_display_name,
            commands::invite_list,
            commands::invite_create,
            commands::invite_redeem,
            commands::invite_accept,
            commands::invite_finish,
            commands::invite_revoke,
            commands::invite_clear,
            commands::debug_log_read,
            commands::debug_log_control,
            commands::debug_log_event,
            commands::debug_log_export,
            ui_surface_changed,
        ])
}

pub fn run() {
    let data_dir = paths::app_data_dir().expect("app data dir unresolvable");
    bootstrap(&data_dir).expect("bootstrap failed");
    // NA-0776 (3.6-v3.1 sec 1): the webview's own directory, from the app's OWN
    // resolver -- so QSLD_DATA_DIR redirection moves both together.
    let webview_dir = paths::webview_dir(&data_dir);
    let app_state = AppState {
        data_dir,
        gw: CoreGateway::default(),
    };
    // NA-0776 (3.3): arm the test-only marker injection seam. Inert unless the harness
    // sets QSLD_INJECT_MARKER; never reachable from the front end.
    app_state.gw.markers.inject_from_env();
    configure_builder(tauri::Builder::default(), app_state)
        .setup(move |app| {
            // ===== NA-0776 (3.6-v3.1 sec 2): THE MAIN WINDOW IS BUILT HERE =====
            // NOT from tauri.conf.json's `windows` block, which is retired. Only this
            // route reaches `.data_directory()`'s DIRECT setter
            // (webview_window.rs:1024-1025); the from_config path resolves the value
            // into a LOCAL COPY that the attribute conversion drops, so the config
            // route is dead by construction at these pins -- traced by the targeted
            // read and confirmed by the ordered two-arm probe (WITH the setter: all
            // five WebKit names under webview/ and none at the default; WITHOUT: the
            // default landing, no webview/ at all).
            // EVERY config property is mirrored, re-read from tauri.conf.json AT THE
            // EDIT and named here so a dropped one is visible in review:
            //   label "main" · title "QuantumShield Chat" · visible false
            //   width 360 · height 585
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::default())
                .title("QuantumShield Chat")
                .inner_size(360.0, 585.0)
                .visible(false)
                .data_directory(webview_dir.clone())
                .build()?;
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
        .run(tauri::generate_context!())
        .expect("error while running qsl-desktop");
}

//! D597 round-2 source disciplines (additive; the slice-A test files stay
//! byte-frozen). These pin the round-2 corrections — the migrated token
//! values, the two-check checklist, the removed meter/list/dot, the status
//! banner component, the ceremony pattern, the item-13 reset calls, the
//! full-bleed shell, the menu tree, and the landed design-spec files — so
//! a later edit cannot silently drift back.

use std::fs;
use std::path::Path;

fn repo_file(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}

fn ui_file(name: &str) -> String {
    repo_file(&format!("ui/{name}"))
}

/// Item 3: exactly TWO checklist rows; the common-password check and its
/// list are gone (the design_system.rs check was removed with them — the
/// one sanctioned amendment).
#[test]
fn checklist_is_exactly_two_checks() {
    let html = ui_file("index.html");
    assert_eq!(
        html.matches(r#"class="req""#).count(),
        2,
        "the checklist must have exactly two rows"
    );
    assert!(!html.contains("req-common"), "the third check must be gone");
    let js = ui_file("main.js");
    assert!(
        !js.contains("COMMON_PASSWORDS"),
        "the common-passwords list must be removed"
    );
}

/// Item 2: the strength meter is removed entirely.
#[test]
fn strength_meter_absent() {
    assert!(!ui_file("index.html").contains(r#"id="strength""#));
    assert!(!ui_file("main.js").contains("strengthEstimate"));
    assert!(!ui_file("style.css").contains(".strength"));
}

/// Item 1: Confirm passphrase sits directly below Passphrase — nothing
/// between the two fields inside the stack.
#[test]
fn confirm_directly_below_passphrase() {
    let html = ui_file("index.html");
    let a = html.find(r#"id="vault-pass""#).expect("vault-pass");
    let b = html.find(r#"id="vault-confirm""#).expect("vault-confirm");
    let between = &html[a..b];
    assert!(
        !between.contains("<div") && !between.contains("<p"),
        "nothing may sit between the passphrase fields: `{between}`"
    );
}

/// Item 4: the wizard step-2 heading and step label read "Your identity".
#[test]
fn step2_heading_is_your_identity() {
    let html = ui_file("index.html");
    assert!(html.contains("<h1>Your identity</h1>"));
    assert!(html.contains("Step 2 of 2 — Your identity"));
    assert!(!html.contains("This is you"));
}

/// Item 5: the verification code renders on one line, never wrapping,
/// with the shrink-to-fit helper wired on both surfaces.
#[test]
fn verify_code_single_line() {
    let css = ui_file("style.css");
    let block_start = css.find(".verify-code").expect(".verify-code");
    let block = &css[block_start..css[block_start..].find('}').unwrap() + block_start];
    assert!(
        block.contains("white-space: nowrap"),
        "code must never wrap"
    );
    let js = ui_file("main.js");
    assert!(js.contains("function fitCode"), "shrink-to-fit helper");
    assert!(
        js.matches("fitCode(byId(").count() >= 2,
        "fitCode wired on wizard + Identity pane"
    );
}

/// Item 6 (§5): the ceremony instruction is its own ONE line on destroy
/// AND erase; the erase screen's extra hint paragraph is deleted.
#[test]
fn ceremony_one_line_instruction() {
    let html = ui_file("index.html");
    assert!(html
        .contains(r#"<p class="instruction">Type <code>destroy my vault</code> to confirm</p>"#));
    assert!(html
        .contains(r#"<p class="instruction">Type <code>erase everything</code> to confirm</p>"#));
    assert!(
        !html.contains("No passphrase is needed"),
        "the erase screen's extra prose is deleted"
    );
}

/// Item 7: the autolock helper is verbatim and restates no number.
#[test]
fn autolock_helper_verbatim() {
    let html = ui_file("index.html");
    assert!(html.contains(
        "On by default. Applies to the main window and settings; the setup wizard is exempt."
    ));
    assert!(!html.contains("On by default (15"));
}

/// Item 8: Arm carries the destructive tier; Disarm the secondary tier.
#[test]
fn arm_destructive_disarm_secondary() {
    let html = ui_file("index.html");
    let arm = html
        .split("<button")
        .find(|t| t.contains("btn-wipe-arm"))
        .expect("arm button");
    assert!(arm.contains(r#"class="danger""#), "Arm = destructive tier");
    let disarm = html
        .split("<button")
        .find(|t| t.contains("btn-wipe-disarm"))
        .expect("disarm button");
    assert!(
        disarm.contains(r#"class="secondary""#),
        "Disarm = secondary"
    );
}

/// Item 9: the duplicated warning prose below the erase status is deleted;
/// the checkbox line carries the warning.
#[test]
fn duplicated_erase_warning_deleted() {
    let html = ui_file("index.html");
    assert!(!html.contains("A guest — or a child"));
    assert!(html.contains("Reaching the limit erases the vault permanently — I understand"));
}

/// Item 10: the disabled tier is neutral bg + muted text — never dimmed
/// accent (no opacity dim on disabled buttons).
#[test]
fn disabled_tier_never_dimmed_accent() {
    let css = ui_file("style.css");
    let start = css.find("button:disabled").expect("disabled tier");
    let block = &css[start..css[start..].find('}').unwrap() + start];
    assert!(block.contains("var(--neutral-bg)"), "neutral disabled bg");
    assert!(block.contains("var(--fg-muted)"), "muted disabled text");
    assert!(!block.contains("opacity"), "never a dimmed tier color");
}

/// Item 11: no identity dot in the main-window rail.
#[test]
fn rail_identity_dot_absent() {
    let html = ui_file("index.html");
    assert!(!html.contains("btn-rail-identity"));
    assert!(!html.contains("rail-initial"));
    assert!(!html.contains("rail-id "));
    assert!(!ui_file("main.js").contains("updateRailInitial"));
}

/// Item 12 (§2): the status banner component exists with the three status
/// classes; both Vault and Security status lines use it; the role colors
/// live in the token block.
#[test]
fn status_banner_component() {
    let css = ui_file("style.css");
    for class in [
        ".status-banner",
        ".status-danger",
        ".status-accent",
        ".status-neutral",
    ] {
        assert!(css.contains(class), "missing {class}");
    }
    let start = css.find(":root {").unwrap();
    let root = &css[start..start + css[start..].find('}').unwrap()];
    for color in [
        "#3A1D1D", "#8A3A3A", "#F0A0A0", "#1C2A3E", "#2E5A8E", "#8FBAF0", "#2A2A2E",
    ] {
        assert!(root.contains(color), "role color {color} must be a token");
    }
    let html = ui_file("index.html");
    assert!(html.contains(r#"id="wipe-state" class="status-banner"#));
    assert!(html.contains(r#"id="autolock-status" class="status-banner"#));
    let js = ui_file("main.js");
    for copy in [
        "Off — wrong attempts never erase the vault",
        "Armed — erases after ",
        "Locks after ",
    ] {
        assert!(js.contains(copy), "banner copy `{copy}`");
    }
}

/// Item 13 (F2 + §5 STATE RULE): destroy/erase completion performs a full
/// webview reload; every screen transition clears the ceremony fields; the
/// wizard never pre-fills a prior alias.
#[test]
fn item13_state_reset_pinned() {
    let js = ui_file("main.js");
    assert!(
        js.matches("window.location.reload()").count() >= 2,
        "destroy AND erase completion must reload the webview"
    );
    assert!(js.contains("function clearCeremonyState"));
    let show_fn = &js[js.find("function show(").unwrap()..];
    let show_body = &show_fn[..show_fn.find("\n}").unwrap()];
    assert!(
        show_body.contains("clearCeremonyState()"),
        "every screen transition clears ceremony state"
    );
    assert!(js.contains("function resetDestroyFlow"));
    assert!(
        !js.contains("byId(\"alias-input\").value = currentSettings.self_alias"),
        "the wizard must not pre-fill a prior alias"
    );
    assert!(
        js.contains(r#"byId("alias-input").value = "";"#),
        "the wizard alias starts empty"
    );
}

/// Item 14 (D.1-D.3): the full-bleed shell grids and the Settings icon
/// rail (Settings is a view, not a modal).
#[test]
fn full_bleed_shell() {
    let css = ui_file("style.css");
    assert!(css.contains("grid-template-columns: 52px 210px 1fr"));
    assert!(css.contains("grid-template-columns: 52px 160px 1fr"));
    let html = ui_file("index.html");
    let settings = &html[html.find(r#"id="scr-settings""#).expect("scr-settings")..];
    assert!(
        settings.contains(r#"class="rail""#),
        "the icon rail is present in the Settings view"
    );
    assert!(settings.contains("settings-rail"));
    assert!(
        !html.contains("&#8592; Back"),
        "no Back pseudo-entry; the rail is live"
    );
}

/// The migrated §1 token values (page/card/field/hairline, 17px titles,
/// radius 12/8) — and the old values gone.
#[test]
fn token_values_migrated() {
    let css = ui_file("style.css");
    let start = css.find(":root {").unwrap();
    let root = &css[start..start + css[start..].find('}').unwrap()];
    for needle in [
        "--bg: #1D1D1F",
        "--bg-raised: #252528",
        "--bg-inset: #1A1A1C",
        "--border: #3A3A3E",
        "--fg: #E8E8E8",
        "--fg-secondary: #A8A8A8",
        "--fg-muted: #7A7A7A",
        "--fs-title: 17px",
        "--fs-body: 13px",
        "--radius: 12px",
        "--radius-small: 8px",
        "--accent-fill: #3D7BC4",
    ] {
        assert!(root.contains(needle), "missing migrated token `{needle}`");
    }
    assert!(
        !css.contains("#14161a"),
        "the old page surface must be gone"
    );
    assert!(!css.contains("--fs-hero"), "the old hero size must be gone");
}

/// Item 15: the native menu tree is source-pinned — the four submenus,
/// the working entries, the R1 state gating, and zero placeholders.
#[test]
fn menu_tree_pinned() {
    let lib = repo_file("src-tauri/src/lib.rs");
    for needle in [
        "SubmenuBuilder::new(app, \"File\")",
        "SubmenuBuilder::new(app, \"Edit\")",
        "SubmenuBuilder::new(app, \"View\")",
        "SubmenuBuilder::new(app, \"Help\")",
        "\"qsl-settings\", \"Settings\"",
        "\"qsl-lock-now\", \"Lock now\"",
        "\"qsl-reload\", \"Reload\"",
        "\"qsl-fullscreen\", \"Full screen\"",
        "PredefinedMenuItem::quit",
        "PredefinedMenuItem::cut",
        "PredefinedMenuItem::copy",
        "PredefinedMenuItem::paste",
        "PredefinedMenuItem::select_all",
        "PredefinedMenuItem::about",
        "fn ui_surface_changed",
        "ui_surface_changed,",
    ] {
        assert!(lib.contains(needle), "missing menu pin `{needle}`");
    }
    // R1: the two state-dependent entries start disabled and are gated on
    // the unlocked surfaces.
    assert!(lib.contains(".enabled(false)"));
    assert!(lib.contains("surface == \"scr-main\" || surface == \"scr-settings\""));
}

/// The design authority landed in-repo (byte-exactness is cmp-proven in
/// the lane evidence; this pins presence + identity lines).
#[test]
fn design_spec_files_landed() {
    let spec = repo_file("docs/DESIGN_SPEC.md");
    assert!(spec.starts_with("# QSC Desktop Design Spec v1"));
    assert!(spec.contains("## 7. Acceptance standard"));
    let appendix = repo_file("docs/DESIGN_SPEC_AppendixD.md");
    assert!(appendix.starts_with("# Appendix D — Reference markup"));
    assert!(appendix.contains("## D.8 — Acceptance"));
}

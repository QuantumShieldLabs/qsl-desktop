//! D598 round-3 source disciplines (additive; slice_a_flows.rs,
//! slice_a_rules.rs, design_system.rs, and design_round2.rs stay
//! byte-frozen). These pin the round-3 corrections — the de-spinnered
//! number inputs with visible validation, the autolock 60/0-never
//! semantics and its never-fire guard, the quoted danger phrases, the
//! shared ceremony card, the compact window modes with the menu-bar
//! visibility rule, the "Delete vault?" rename, the 30-second erase
//! countdown gate, and the three-file design authority — so a later edit
//! cannot silently drift back.

use qsl_desktop_app::settings::{self, AppSettings, AUTOLOCK_DEFAULT_MINUTES};
use qsl_desktop_app::{mode_for_surface, window_mode_spec, WindowMode};
use std::fs;
use std::path::Path;

fn repo_file(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}

fn ui_file(name: &str) -> String {
    repo_file(&format!("ui/{name}"))
}

fn manifest_file(name: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}

/// Item 1 (E.2): no native number inputs anywhere — the two numeric fields
/// are text/numeric-inputmode; the spinner-suppression CSS exists; the
/// fields are ~64px and centered.
#[test]
fn no_native_number_inputs() {
    let html = ui_file("index.html");
    assert!(
        !html.contains(r#"type="number""#),
        "native number inputs must be gone (E.2)"
    );
    assert_eq!(
        html.matches(r#"inputmode="numeric""#).count(),
        2,
        "both numeric fields carry inputmode=numeric"
    );
    let css = ui_file("style.css");
    assert!(css.contains("appearance: textfield"), "E.2 spinner rule");
    assert!(css.contains("::-webkit-outer-spin-button"));
    assert!(css.contains("::-webkit-inner-spin-button"));
    let num_start = css.find("input.num {").expect("input.num rule");
    let num_block = &css[num_start..css[num_start..].find('}').unwrap() + num_start];
    assert!(num_block.contains("width: 64px"), "E.2 ~64px field");
    assert!(num_block.contains("text-align: center"));
}

/// Item 1 (E.2): the autolock range is 0-1440 (the landed 720 attr is
/// gone); the erase limit stays 1-100; invalid entries get the danger
/// border + an inline message (visible rejection, never silent).
#[test]
fn visible_validation_pinned() {
    let html = ui_file("index.html");
    let autolock_tag = html
        .split("<input")
        .find(|t| t.contains("autolock-min"))
        .expect("autolock input");
    assert!(
        autolock_tag.contains(r#"min="0""#),
        "0 is a valid entry (E.3)"
    );
    assert!(autolock_tag.contains(r#"max="1440""#), "E.2 upper bound");
    assert!(
        !html.contains(r#"max="720""#),
        "the landed 720 attr is gone"
    );
    let wipe_tag = html
        .split("<input")
        .find(|t| t.contains("wipe-limit"))
        .expect("wipe input");
    assert!(wipe_tag.contains(r#"min="1""#) && wipe_tag.contains(r#"max="100""#));
    let js = ui_file("main.js");
    assert!(js.contains("function validateNum"), "the shared validator");
    assert!(
        js.contains(r#"classList.toggle("invalid""#),
        "invalid entries mark the field"
    );
    assert!(js.contains("Enter a whole number from 0 to 1440."));
    assert!(js.contains("Enter a whole number from 1 to 100."));
    let css = ui_file("style.css");
    let inv = css.find("input.invalid").expect("input.invalid rule");
    let inv_block = &css[inv..css[inv..].find('}').unwrap() + inv];
    assert!(inv_block.contains("var(--danger-border)"), "danger border");
}

/// Item 2 (E.3): default 60; 0 accepted by the settings API and meaning
/// never-auto-lock; no backend range bound (F2).
#[test]
fn autolock_default_sixty_and_zero_valid() {
    assert_eq!(AUTOLOCK_DEFAULT_MINUTES, 60);
    assert_eq!(AppSettings::default().autolock_minutes, 60);
    let dir = tempfile::tempdir().unwrap();
    let mut s = AppSettings::default();
    s.autolock_minutes = 0;
    settings::save(dir.path(), &s).expect("0 must be accepted (E.3)");
    assert_eq!(settings::load(dir.path()).autolock_minutes, 0);
    s.autolock_minutes = 100_000;
    settings::save(dir.path(), &s).expect("no backend cap (F2 default)");
    let src = manifest_file("src/settings.rs");
    assert!(
        !src.contains("autolock_minimum_one_minute"),
        "the 0-reject and its error string are removed"
    );
}

/// Item 2b (E.3, BINDING encoded rule): the idle timer must NEVER fire at
/// 0 — the explicit never-fire guard sits in the timer before the elapsed
/// comparison (at 0 the minutes*60000 comparison would lock immediately).
#[test]
fn autolock_zero_never_fire_guard() {
    let js = ui_file("main.js");
    let guard = js
        .find("if (autolockMinutes === 0) return;")
        .expect("the 0 never-fire guard must exist");
    let fire = js
        .find("autolockMinutes * 60 * 1000")
        .expect("the elapsed comparison");
    assert!(
        guard < fire,
        "the guard must precede the elapsed-time comparison"
    );
    assert!(
        js.contains("let autolockMinutes = 60;"),
        "the frontend mirror follows the 60 default"
    );
    assert!(
        js.contains("autolock_minutes: 60"),
        "the settings mirror follows the 60 default"
    );
}

/// Item 2c (E.3): the banner state machine — both copies VERBATIM; the 0
/// state renders the danger banner (the recorded R2 extension).
#[test]
fn autolock_banner_state_machine() {
    let js = ui_file("main.js");
    assert!(js.contains("Never locks — anyone with access to this device can open your vault"));
    assert!(js.contains("Locks after "));
    let f = js
        .find("function renderAutolockBanner")
        .expect("renderAutolockBanner");
    let body = &js[f..f + js[f..].find("\n}").unwrap()];
    assert!(
        body.contains(r#"setBanner(el, "danger""#),
        "value == 0 -> the danger banner"
    );
    assert!(
        body.contains(r#"setBanner(el, "accent""#),
        "value > 0 -> the accent banner"
    );
    assert!(body.contains("minutes === 0"), "the 0 branch is explicit");
}

/// Item 3 (E.4): the danger phrases render quoted — the quotes are part of
/// the RENDERED text (CSS content on the phrase element; the round-2
/// markup needles stay byte-frozen) and the phrase renders in danger mono.
#[test]
fn quoted_danger_phrases_rendered() {
    let css = ui_file("style.css");
    assert!(
        css.contains(".instruction code::before, .instruction code::after"),
        "the quote-content selectors"
    );
    let q = css.find(".instruction code::before").expect("quote rule");
    let block = &css[q..css[q..].find('}').unwrap() + q];
    assert!(block.contains(r#"content: '"'"#), "the rendered quotes");
    let color = css.find(".instruction code {").expect("phrase color rule");
    let cblock = &css[color..css[color..].find('}').unwrap() + color];
    assert!(
        cblock.contains("var(--danger-text)"),
        "the phrase renders danger"
    );
    // The phrase markup itself stays the round-2 byte-frozen form.
    let html = ui_file("index.html");
    assert!(html.contains("Type <code>destroy my vault</code> to confirm"));
    assert!(html.contains("Type <code>erase everything</code> to confirm"));
}

/// Items 4-5 (E.4): the shared ceremony card on BOTH destructive surfaces;
/// destroy fields full width with the label above (no label-wrap).
#[test]
fn ceremony_card_both_surfaces() {
    let html = ui_file("index.html");
    assert!(
        html.matches(r#"ceremony-card"#).count() >= 2,
        "both destructive surfaces carry the ceremony card"
    );
    let erase = &html[html.find(r#"id="scr-erase""#).expect("scr-erase")..];
    let erase = &erase[..erase.find("</section>").unwrap()];
    assert!(erase.contains("ceremony-card"), "erase ceremony card");
    assert!(erase.contains("ceremony-head"), "erase ceremony head");
    let vault = &html[html.find(r#"id="pane-vault""#).expect("pane-vault")..];
    assert!(vault.contains("ceremony-card"), "destroy ceremony card");
    assert!(vault.contains("ceremony-head"), "destroy ceremony head");
    // Item 4: both destroy fields full width, field-label above the input.
    assert!(
        !html.contains("<label>Current passphrase"),
        "label-wrap gone"
    );
    assert!(vault.contains(r#"<span class="field-label">Current passphrase</span>"#));
    for id in ["destroy-pass", "destroy-phrase", "erase-phrase"] {
        let tag = html
            .split("<input")
            .find(|t| t.contains(id))
            .unwrap_or_else(|| panic!("{id} input"));
        assert!(tag.contains(r#"class="full""#), "{id} must be full width");
    }
    // E.4 card DNA via tokens.
    let css = ui_file("style.css");
    let c = css.find(".ceremony-card {").expect(".ceremony-card rule");
    let cblock = &css[c..css[c..].find('}').unwrap() + c];
    assert!(cblock.contains("var(--bg)"), "E.4 bg #1D1D1F");
    assert!(cblock.contains("var(--danger-border)"), "E.4 border");
    assert!(cblock.contains("var(--radius)"), "E.4 radius 12");
    assert!(
        cblock.contains("var(--sp-x20) var(--sp-x22)"),
        "E.4 padding"
    );
    let h = css
        .find(".ceremony-card .ceremony-head")
        .expect("head rule");
    let hblock = &css[h..css[h..].find('}').unwrap() + h];
    assert!(hblock.contains("var(--fs-title)") && hblock.contains("var(--danger-text)"));
}

/// Item 6 (E.7): the arm checkbox — >=16px box, one-line label (the label
/// element itself is the clickable surface).
#[test]
fn arm_checkbox_hit_area() {
    let css = ui_file("style.css");
    let l = css.find("label.inline {").expect("label.inline rule");
    let lblock = &css[l..css[l..].find('}').unwrap() + l];
    assert!(lblock.contains("white-space: nowrap"), "ONE line");
    let cb = css
        .find(r#"label.inline input[type="checkbox"]"#)
        .expect("checkbox size rule");
    let cblock = &css[cb..css[cb..].find('}').unwrap() + cb];
    assert!(
        cblock.contains("width: 16px") && cblock.contains("height: 16px"),
        ">=16px box"
    );
}

/// Item 7 (E.3): the helper sits DIRECTLY under the autolock banner — the
/// error surface is out of that slot (it renders at the validation
/// placement above the banner).
#[test]
fn helper_directly_under_banner() {
    let html = ui_file("index.html");
    let banner = html.find(r#"id="autolock-status""#).expect("banner");
    let helper = html.find("On by default.").expect("helper");
    assert!(banner < helper, "banner before helper");
    let between = &html[banner..helper];
    assert!(
        !between.contains("autolock-error"),
        "no error surface between banner and helper"
    );
    let err = html.find(r#"id="autolock-error""#).expect("error surface");
    assert!(
        err < banner,
        "the error renders at the field, above the banner"
    );
}

/// Item 8 (E.7): the Settings code box matches the wizard's proportions.
#[test]
fn settings_code_box_bounded() {
    let css = ui_file("style.css");
    let s = css.find("#settings-code {").expect("#settings-code rule");
    let block = &css[s..css[s..].find('}').unwrap() + s];
    assert!(block.contains("max-width: 420px"));
    assert!(block.contains("margin: 0 auto"));
}

/// Item 9 (E.7): rail icons ~21px (svg + --fs-glyph move together);
/// status-bar text 12px text-secondary.
#[test]
fn legibility_bump() {
    let css = ui_file("style.css");
    let start = css.find(":root {").unwrap();
    let root = &css[start..start + css[start..].find('}').unwrap()];
    assert!(root.contains("--fs-glyph: 21px"), "glyph token bumped");
    assert!(css.contains(".rail-btn svg { width: 21px; height: 21px; }"));
    let sl = css.find(".status-line {").expect(".status-line rule");
    let block = &css[sl..css[sl..].find('}').unwrap() + sl];
    assert!(block.contains("var(--fs-hint)"), "12px status bar (E.7)");
    assert!(block.contains("var(--fg-secondary)"), "#A8A8A8 status bar");
}

/// Item 10 (E.1) as amended by round 4a (D601/F1): the window-mode table —
/// every pre-main surface maps to its OWN mode with its own size, and the
/// menu stays hidden on all of them; sizing rides the surface-report carrier
/// and the F1 launch sequence (visible:false + compact initial size, shown
/// after the first sized report).
#[test]
fn window_modes_and_menu_visibility() {
    assert_eq!(
        mode_for_surface("scr-wizard-vault"),
        WindowMode::WizardVault
    );
    assert_eq!(
        mode_for_surface("scr-wizard-identity"),
        WindowMode::WizardIdentity
    );
    assert_eq!(mode_for_surface("scr-unlock"), WindowMode::Unlock);
    assert_eq!(mode_for_surface("scr-erase"), WindowMode::Erase);
    assert_eq!(mode_for_surface("scr-wiped"), WindowMode::Wiped);
    assert_eq!(mode_for_surface("scr-main"), WindowMode::Full);
    assert_eq!(mode_for_surface("scr-settings"), WindowMode::Full);

    // Round 4a: one size per pre-main surface, and no two pre-main surfaces
    // share a height any more — that sharing WAS the dead space.
    let ((w, h), _, menu) = window_mode_spec(WindowMode::WizardVault);
    assert_eq!((w, h, menu), (360.0, 585.0, false));
    let ((w, h), _, menu) = window_mode_spec(WindowMode::WizardIdentity);
    assert_eq!((w, h, menu), (360.0, 625.0, false));
    let ((w, h), _, menu) = window_mode_spec(WindowMode::Unlock);
    assert_eq!((w, h, menu), (360.0, 255.0, false));
    let ((w, h), _, menu) = window_mode_spec(WindowMode::Erase);
    assert_eq!((w, h, menu), (360.0, 275.0, false));
    let ((w, h), _, menu) = window_mode_spec(WindowMode::Wiped);
    assert_eq!((w, h, menu), (360.0, 220.0, false));

    // Width is the operator's measured reading width and is SHARED by every
    // pre-main surface; the heights are only valid at that width.
    for m in [
        WindowMode::WizardVault,
        WindowMode::WizardIdentity,
        WindowMode::Unlock,
        WindowMode::Erase,
        WindowMode::Wiped,
    ] {
        let ((w, _), _, _) = window_mode_spec(m);
        assert_eq!(w, 360.0, "every pre-main surface shares the reading width");
    }
    let ((w, h), (mw, mh), menu) = window_mode_spec(WindowMode::Full);
    assert_eq!((w, h, mw, mh, menu), (1024.0, 700.0, 800.0, 600.0, true));

    // Every pre-main surface shares ONE modest minimum so the window can be
    // dragged small enough to exercise the F4 wrap remedy.
    for m in [
        WindowMode::WizardVault,
        WindowMode::WizardIdentity,
        WindowMode::Unlock,
        WindowMode::Erase,
        WindowMode::Wiped,
    ] {
        let ((w, h), (mw, mh), _) = window_mode_spec(m);
        assert_eq!((mw, mh), (360.0, 200.0), "compact floor");
        assert!(
            w >= mw && h >= mh,
            "initial size must not start below the floor"
        );
    }

    let lib = manifest_file("src/lib.rs");
    for needle in [
        "show_menu",
        "remove_menu",
        "set_min_size",
        "set_size",
        "center",
        "fn apply_window_mode",
        "is_visible",
    ] {
        assert!(lib.contains(needle), "window-mode pin `{needle}`");
    }
    // F1: the sanctioned windows[0] amendment — hidden launch at the
    // compact initial size.
    let conf = manifest_file("tauri.conf.json");
    assert!(conf.contains(r#""visible": false"#), "F1 hidden launch");
    assert!(
        conf.contains(r#""width": 360"#),
        "F1 compact initial width — round 4a: the reading width"
    );
    assert!(
        conf.contains(r#""height": 585"#),
        "F1 compact initial height — round 4a: the wizard-step-1 literal"
    );
}

/// Item 10b (E.1) as amended by round 4a: the compact screens carry the
/// uniform 28px content padding on the SCREEN (the window is the card, so
/// the screen owns the padding the card used to), stretch, no void.
#[test]
fn compact_card_fills_window() {
    let css = ui_file("style.css");
    let sel = css
        .find("#scr-wizard-vault.screen, #scr-wizard-identity.screen, #scr-unlock.screen")
        .expect("compact screen rule");
    let block = &css[sel..css[sel..].find('}').unwrap() + sel];
    assert!(
        block.contains("padding: var(--sp-x28)"),
        "round 4a page padding 28px"
    );
    assert!(block.contains("align-items: stretch"), "no vertical void");
    assert!(css.contains("#scr-erase .card, #scr-wiped .card { width: 100%;"));
}

/// Round 4a (F2 as REVISED at census review): NO container chrome survives on
/// ANY pre-main screen — neutral or danger. The chrome-stripping rule must
/// zero background, border, radius and padding, and must RE-HOME the flex
/// column the .card provided.
#[test]
fn pre_main_screens_have_no_card_chrome() {
    let css = ui_file("style.css");
    let sel = css
        .rfind("#scr-wizard-vault .card, #scr-wizard-identity .card, #scr-unlock .card")
        .expect("round-4a chrome-strip rule");
    let block = &css[sel..css[sel..].find('}').unwrap() + sel];
    for needle in [
        "background: none",
        "border: none",
        "border-radius: 0",
        "padding: 0",
        "flex-direction: column",
    ] {
        assert!(block.contains(needle), "chrome strip must set `{needle}`");
    }
}

/// Round 4a: the SETTINGS destroy ceremony is NOT a pre-main screen and keeps
/// its card. The E.4 `.ceremony-card` rule therefore survives intact — the
/// pre-main strip is ID-scoped and must never be widened to the bare class.
#[test]
fn settings_ceremony_card_survives_the_strip() {
    let css = ui_file("style.css");
    let c = css.find(".ceremony-card {").expect(".ceremony-card rule");
    let cblock = &css[c..css[c..].find('}').unwrap() + c];
    assert!(cblock.contains("var(--danger-border)"), "E.4 border kept");
    assert!(cblock.contains("var(--bg)"), "E.4 bg kept");
    let html = ui_file("index.html");
    let vault = &html[html.find(r#"id="pane-vault""#).expect("pane-vault")..];
    assert!(
        vault.contains("ceremony-card"),
        "settings keeps its ceremony"
    );
}

/// Item 11a (E.6): "Delete vault?" as the danger link; the old wording
/// removed everywhere; the unlock error renders inline only when present
/// (no reserved gap).
#[test]
fn delete_vault_link_and_inline_error() {
    let html = ui_file("index.html");
    let js = ui_file("main.js");
    assert!(
        !html.contains("Forgot your passphrase?") && !js.contains("Forgot your passphrase?"),
        "the old wording is removed everywhere"
    );
    assert!(html.contains(">Delete vault?</a>"));
    let link = html
        .split("<a ")
        .find(|t| t.contains("link-forgot"))
        .expect("the unlock link");
    assert!(link.contains("link-danger"), "the danger link class");
    let css = ui_file("style.css");
    let l = css
        .find(".linkrow a.link-danger")
        .expect("link-danger rule");
    let lblock = &css[l..css[l..].find('}').unwrap() + l];
    assert!(lblock.contains("var(--danger-link)"), "E.6 #C87A7A");
    assert!(lblock.contains("underline"));
    let root = &css[css.find(":root {").unwrap()..];
    let root = &root[..root.find('}').unwrap()];
    assert!(root.contains("--danger-link: #C87A7A"), "tokenized");
    let f = css.find(".feedback {").expect(".feedback rule");
    let fblock = &css[f..css[f..].find('}').unwrap() + f];
    assert!(
        !fblock.contains("min-height"),
        "no reserved unlock-error gap (E.6)"
    );
}

/// Item 11b (E.5): the countdown panel exists with the verbatim copy; the
/// erase_all invoke is reachable ONLY from the countdown-complete path
/// (exactly one invoke, inside the interval, after the zero check); Cancel
/// and every state transition abort.
#[test]
fn countdown_gates_the_erase_commit() {
    let html = ui_file("index.html");
    let erase = &html[html.find(r#"id="scr-erase""#).expect("scr-erase")..];
    let erase = &erase[..erase.find("</section>").unwrap()];
    assert!(erase.contains(r#"id="erase-form""#), "the replaceable form");
    assert!(erase.contains("countdown-panel"));
    assert!(erase.contains(r#"id="countdown-number""#));
    assert!(erase.contains(r#"id="countdown-label""#));
    assert!(erase.contains("Close this window or press Cancel to stop."));
    assert!(erase.contains(r#"id="btn-erase-countdown-cancel""#));

    let js = ui_file("main.js");
    assert_eq!(
        js.matches(r#"invoke("erase_all""#).count(),
        1,
        "exactly ONE erase_all invoke exists"
    );
    let invoke_at = js.find(r#"invoke("erase_all""#).unwrap();
    let interval_at = js
        .find("eraseCountdownTimer = setInterval")
        .expect("the countdown interval");
    let zero_check = js
        .find("if (eraseCountdownLeft > 0) return;")
        .expect("the zero gate");
    assert!(
        interval_at < zero_check && zero_check < invoke_at,
        "the invoke sits inside the interval, after the zero gate"
    );
    assert!(js.contains("Erasing in ${eraseCountdownLeft} seconds…"));
    assert!(js.contains("eraseCountdownLeft = 30"), "counts from 30");
    assert!(js.contains("function eraseCountdownAbort"));
    let clear = js.find("function clearCeremonyState").unwrap();
    let clear_body = &js[clear..clear + js[clear..].find("\n}").unwrap()];
    assert!(
        clear_body.contains("eraseCountdownAbort()"),
        "every state transition aborts a running countdown"
    );
}

/// Item 12: the three-file design authority — Appendix E present with the
/// E.1 window table; the amended files carry no surviving contradiction
/// needle.
#[test]
fn design_authority_self_consistent() {
    let e = repo_file("docs/DESIGN_SPEC_AppendixE.md");
    assert!(e.starts_with("# Appendix E — Round-3 reference markup"));
    assert!(e.contains("| Screen                         | Window size (approx) | Menu bar |"));
    let spec = repo_file("docs/DESIGN_SPEC.md");
    let d = repo_file("docs/DESIGN_SPEC_AppendixD.md");
    for (name, text) in [("DESIGN_SPEC.md", &spec), ("AppendixD", &d)] {
        assert!(
            !text.contains("11px text-muted"),
            "{name}: old status-bar line survives"
        );
        assert!(
            !text.contains(r#"value="15""#),
            "{name}: the 15-minute example survives"
        );
        assert!(
            !text.contains("Locks after 15 minutes"),
            "{name}: the 15-minute banner example survives"
        );
        assert!(
            !text.contains("width 56px"),
            "{name}: the 56px input note survives"
        );
        assert!(!text.contains(r#"max="720""#), "{name}: 720 survives");
    }
    assert!(
        spec.contains("[E.3]") && d.contains("[E.1]"),
        "amendments cite E.x"
    );
}

/// Round 4a (F4): the verification code can no longer clip silently. The
/// below-floor escape is a `.verify-code.wrapped` modifier that MUST sit
/// AFTER the base `.verify-code` block — both to win the cascade and because
/// the frozen needle in design_round2.rs slices from the FIRST `.verify-code`
/// to the next `}` and requires `white-space: nowrap` inside it.
#[test]
fn verify_code_never_clips_silently() {
    let css = ui_file("style.css");
    let base = css.find(".verify-code {").expect(".verify-code base rule");
    let wrapped = css
        .find(".verify-code.wrapped")
        .expect("round-4a wrapped modifier");
    assert!(
        wrapped > base,
        "the .wrapped override must FOLLOW the base block or the frozen \
         design_round2 needle stops reading the base rule"
    );
    let wblock = &css[wrapped..css[wrapped..].find('}').unwrap() + wrapped];
    assert!(wblock.contains("white-space: normal"), "wrap released");
    assert!(wblock.contains("overflow: visible"), "clip released");
    // The base block keeps the frozen properties untouched.
    let bblock = &css[base..css[base..].find('}').unwrap() + base];
    assert!(bblock.contains("white-space: nowrap"), "base stays nowrap");

    let js = ui_file("main.js");
    assert!(
        js.contains(r#"el.classList.add("wrapped")"#),
        "fitCode must apply the wrap at the floor"
    );
    assert!(
        js.contains(r#"el.classList.remove("wrapped")"#),
        "fitCode must re-measure from a clean slate so it can grow back"
    );
    // The resize refit — absent from the entire ui/ tree before this lane.
    assert!(
        js.contains(r#"window.addEventListener("resize""#),
        "the code must refit on resize"
    );
    for id in ["identity-code", "settings-code"] {
        assert!(js.contains(id), "resize refit covers `{id}`");
    }
}

/// Round 4a (item D): the destroy-vault ceremony REPLACES its trigger button
/// rather than sitting below it, and every collapse path restores it. The
/// gates themselves are behavior-frozen — passphrase, typed phrase, and the
/// tokened core call are asserted untouched here so a restyle cannot weaken
/// them.
#[test]
fn destroy_ceremony_replaces_its_trigger() {
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"byId("btn-destroy-open").classList.add("hidden")"#),
        "opening the ceremony hides the trigger"
    );
    assert!(
        js.contains(r#"byId("btn-destroy-open").classList.remove("hidden")"#),
        "cancel restores the trigger"
    );
    assert!(
        js.contains(r#"if (dopen) dopen.classList.remove("hidden")"#),
        "a state transition also restores the trigger"
    );
    // Gates unchanged.
    assert!(
        js.contains(r#"if (phrase !== "destroy my vault")"#),
        "typed gate"
    );
    assert!(
        js.contains(r#"invoke("destroy_vault""#),
        "tokened core call"
    );
    assert!(
        js.contains("confirmPhrase: phrase"),
        "phrase rides the call"
    );
}

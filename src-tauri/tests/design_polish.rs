//! NA-0680 / D615 polish-lane source disciplines (additive; design_system.rs,
//! design_round2.rs and design_round3.rs carry their own pins and are amended
//! in place where this lane moves them).
//!
//! ⚠ EVERY test in this file ships with a proof it can fail — the operator's
//! standing negative-control rule. The specific regression each one must catch
//! is named in its doc comment, because a needle whose failure mode was never
//! chosen tends to pin the wrong thing. Evidence:
//! `docs/governance/evidence/NA-0680_as_built.md`.
//!
//! GATE 1 rows only. The GATE-2 rows (Identity/Vault panes, content-driven
//! window sizing, R-17/R-18) land with GATE 2.

use qsl_desktop_app::{height_for, window_mode_spec, WindowMode};
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

/// Strip `<!-- ... -->` blocks. Needles that ban a substring across a region
/// MUST run on markup only: the comment documenting a ban contains the banned
/// word, so an unstripped check fires on its own rationale. Learned three
/// times in this lane.
fn strip_html_comments(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(i) = rest.find("<!--") {
        out.push_str(&rest[..i]);
        rest = match rest[i..].find("-->") {
            Some(j) => &rest[i + j + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// Slice `css` from `sel` to the next `}`.
fn rule_block<'a>(css: &'a str, sel: &str) -> &'a str {
    let start = css
        .find(sel)
        .unwrap_or_else(|| panic!("missing rule `{sel}`"));
    let end = css[start..].find('}').expect("unterminated rule") + start;
    &css[start..=end]
}

/// R-1: focus is 1px accent with NO glow, everywhere.
///
/// ⚠ MUST GO RED IF: the rule regresses to the shipped `outline: 2px` — the
/// exact state this replaces — or if a `box-shadow` glow is introduced. The
/// operator ruled this control explicitly, because the mockup mis-described
/// the shipped mechanism as a box-shadow when it was an outline, and a lane
/// that "removed the glow" would have edited nothing and shipped 2px intact.
#[test]
fn focus_ring_is_one_px_accent_no_glow() {
    let css = ui_file("style.css");

    // The app-wide rule: 1px, flush, accent.
    let fv = rule_block(&css, ":focus-visible {");
    assert!(
        fv.contains("outline: 1px solid var(--accent-fill)"),
        "focus-visible must be a 1px accent outline, got: {fv}"
    );
    assert!(
        fv.contains("outline-offset: 0"),
        "the ring must be FLUSH — a detached ring is the 2px look being replaced"
    );

    // Inputs: a true accent BORDER, outline suppressed (mockup 10 `.proposed`).
    let inp = rule_block(&css, "input:focus {");
    assert!(
        inp.contains("outline: none"),
        "the input outline must be suppressed so the ring is the border alone"
    );
    assert!(
        inp.contains("border-color: var(--accent-fill)"),
        "the input's own border turns accent on focus"
    );

    // The regression this exists to catch: NO 2px focus outline may survive
    // anywhere, and no focus glow may be introduced.
    for (sel, _) in [(":focus-visible", 0), ("input:focus", 0)] {
        let b = rule_block(&css, sel);
        assert!(
            !b.contains("outline: 2px"),
            "`{sel}` regressed to the 2px outline this lane replaced"
        );
    }
    // No focus GLOW. Scoped to rules whose selector mentions `:focus` — a
    // blanket `!css.contains("box-shadow")` is wrong and was caught by this
    // test failing on its first run: `.settings-rail .cat.active` uses an
    // inset box-shadow as the active-nav bar, which has nothing to do with
    // focus. Banning the string file-wide would have forced an unrelated
    // control to be rewritten to satisfy a focus rule.
    for chunk in css.split('}') {
        let Some((selector, decls)) = chunk.split_once('{') else {
            continue;
        };
        if selector.contains(":focus") {
            assert!(
                !decls.contains("box-shadow"),
                "focus glow reintroduced on `{}` — R-1 is 1px accent, no box-shadow",
                selector.trim()
            );
        }
    }
    // Checkboxes move with the rest.
    let cb = rule_block(&css, r#"input[type="checkbox"]:focus-visible"#);
    assert!(cb.contains("outline: 1px solid var(--accent-fill)"));
}

/// R-8: ONE merged intro on the vault step; the per-field length hint is gone.
///
/// ⚠ MUST GO RED IF: the deleted hint returns (two surfaces stating the same
/// rule is where the weaker wording survives), or the merged intro loses the
/// anti-pattern sentence — the clause that does the actual work, since "longer
/// is better" without "a short complex password is not" is the advice users
/// already think they are following.
#[test]
fn vault_step_intro_is_merged_and_hint_removed() {
    let html = ui_file("index.html");
    assert!(
        html.contains("Everything this app stores is encrypted with the passphrase you choose."),
        "the merged intro must lead the step"
    );
    assert!(
        html.contains("Your passphrase <b>length</b> matters most"),
        "`length` is emphasised with <b>, not caps"
    );
    assert!(
        html.contains("4–5 random words, or 12+ random characters from a password manager"),
        "both recipes survive the merge"
    );
    assert!(
        html.contains(r#"A short "complex" password is not."#),
        "THE ANTI-PATTERN CLAUSE — the sentence that makes the advice actionable"
    );
    assert!(
        !html.contains("Length matters most — a few random words beat a short complex password."),
        "the superseded per-field hint must not survive alongside the merged intro"
    );
}

/// R-9: the no-recovery callout is ACCENT, never amber, and keeps its weight.
///
/// ⚠ MUST GO RED IF: the callout reverts to the `--warn-*` family, or loses
/// its border (prominence was explicitly to be kept — this is a recolour, not
/// a de-emphasis), or the bold sentence is trimmed back to two words.
#[test]
fn no_recovery_callout_is_accent_not_amber() {
    let css = ui_file("style.css");
    let b = rule_block(&css, ".callout {");
    assert!(
        b.contains("var(--accent-bg)") && b.contains("var(--accent-border)"),
        "the callout renders in the accent role, got: {b}"
    );
    assert!(
        !b.contains("--warn-") && !b.contains("--amber"),
        "no amber may survive in the callout — amber is not in the severity vocabulary"
    );
    assert!(
        b.contains("border:"),
        "full prominence: the callout stays bordered"
    );

    let html = ui_file("index.html");
    assert!(
        html.contains(r#"<div class="callout">There is <strong>no recovery.</strong>"#),
        "the whole `There is no recovery.` sentence is bold"
    );
    assert!(
        !html.contains(r#"class="warn""#),
        "the stale `warn` class name must not survive an accent recolour"
    );
    // The amber tokens STAY defined — `.alert-amber` still uses them and is
    // out of this lane's scope. Removing them would be a second, unruled change.
    assert!(
        css.contains("--amber:") && css.contains(".alert-amber"),
        "the amber token + its one remaining consumer are deliberately untouched"
    );
}

/// R-6: the wizard identity step follows mockup 07B's order — name FIRST.
///
/// ⚠ MUST GO RED IF: the order regresses so the verification code precedes the
/// name field. The name is the only thing the user supplies on this step, and
/// burying it under a code they cannot act on yet is the layout this replaces.
#[test]
fn identity_step_orders_name_before_code() {
    let html = ui_file("index.html");
    let step = &html[html
        .find(r#"id="scr-wizard-identity""#)
        .expect("wizard identity section")..];
    let step = &step[..step.find("</section>").expect("section end")];

    let name = step.find("Your name").expect("the name label");
    let code = step
        .find("Your verification code")
        .expect("the code caption");
    let tech = step.find("Show technical details").expect("the disclosure");
    let cont = step.find(r#"id="btn-identity-done""#).expect("the action");
    assert!(
        name < code && code < tech && tech < cont,
        "mockup 07B order is name -> code -> technical details -> one action \
         (got name={name} code={code} tech={tech} action={cont})"
    );
    assert!(
        step.contains(r#"placeholder="e.g. Victor""#),
        "the empty name field carries a placeholder"
    );
    assert!(
        step.contains(">Continue</button>"),
        "ONE action, labelled Continue"
    );
}

/// R-7: the name is REQUIRED — Continue is disabled until it is non-empty
/// after trimming.
///
/// ⚠ MUST GO RED IF: the button ships enabled, or the gate tests the raw value
/// instead of the trimmed one (a single space would then satisfy a "required"
/// field, which is the failure that makes required-field gates worthless).
#[test]
fn identity_continue_requires_a_trimmed_name() {
    let html = ui_file("index.html");
    let tag = html
        .split("<button")
        .find(|t| t.contains("btn-identity-done"))
        .expect("the Continue button");
    assert!(
        tag.contains("disabled"),
        "Continue must ARRIVE disabled — an empty required field cannot start actionable"
    );

    let js = ui_file("main.js");
    assert!(
        js.contains("function updateIdentityContinue"),
        "the gate is one named function, not an inline handler"
    );
    let f = js.find("function updateIdentityContinue").unwrap();
    let body = &js[f..f + js[f..].find("\n}").expect("function end")];
    assert!(
        body.contains(".value.trim() === \"\""),
        "the gate must test the TRIMMED value — whitespace is not a name"
    );
    assert!(
        js.contains(r#"byId("alias-input").addEventListener("input", updateIdentityContinue)"#),
        "the gate re-evaluates as the user types"
    );
    assert!(
        js.contains("updateIdentityContinue(); // R-7: an empty field must arrive DISABLED"),
        "the gate is re-applied when the step is (re-)entered, not only on input"
    );
}

/// R-3: "Stored only on this device — never sent anywhere" is TRUE, and stays
/// true.
///
/// ⚠ MUST GO RED IF: a reader of `self_alias` appears outside `settings.rs` —
/// which is exactly what an invite or contact card carrying a display name
/// would add. The claim was verified at `44237b2` (three call sites, none
/// outward); this pins the property rather than the audit, because the
/// messaging epic is building invites right now and NA-0675 already paid for
/// one claim that was true when written.
#[test]
fn self_alias_has_no_reader_outside_settings() {
    let html = ui_file("index.html");
    assert!(
        html.contains("Stored only on this device — never sent anywhere."),
        "the claim ships on the wizard's name field"
    );

    // The Rust surface: `self_alias` may be declared/serialised in settings.rs
    // and forwarded by the one command that writes it. Anything else reading
    // it is a new consumer and must be reviewed against the claim.
    for src in [
        "src/lib.rs",
        "src/state.rs",
        "src/paths.rs",
        "src/gateway.rs",
        "src/markers.rs",
    ] {
        let text = manifest_file(src);
        assert!(
            !text.contains("self_alias"),
            "`{src}` reads self_alias — the claim \"never sent anywhere\" must be \
             re-verified before this is allowed"
        );
    }
    // commands.rs may touch it ONLY inside settings_set (the writer).
    let cmds = manifest_file("src/commands.rs");
    let occurrences = cmds.matches("self_alias").count();
    assert_eq!(
        occurrences, 3,
        "commands.rs may reference self_alias exactly 3 times (the settings_set \
         parameter, its doc comment, and the assignment). A 4th is a new reader."
    );
}

// ===========================================================================
// GATE 2 — Settings panes, content-driven window sizing, R-17/R-18.
// ===========================================================================

/// R-2/R-10: the Identity and Vault panes use the hairline section idiom, and
/// the hairline count FOLLOWS the section count.
///
/// ⚠ MUST GO RED IF: the adjacent-sibling rule is replaced by a blanket
/// `border-bottom`, which is the shape that leaves a trailing rule at the
/// bottom of the pane the moment someone adds a section. Also red if
/// `.srv-sect` is edited — the Server pane is shipped with live acceptance
/// evidence and D615 §9 makes it read-only for this lane.
#[test]
fn panes_use_the_hairline_section_idiom() {
    let css = ui_file("style.css");
    assert!(
        css.contains(".pane-sect + .pane-sect { border-top: 1px solid var(--border); }"),
        "the hairline must come from the ADJACENT-SIBLING rule, so the count \
         follows the section count"
    );
    let b = rule_block(&css, ".pane-sect {");
    assert!(b.contains("padding: var(--sp-6)"), "--sp-6 section padding");

    // ⚠ The Server pane's own rules are UNTOUCHED. Reusing the idiom in a new
    // class is permitted; editing theirs is not.
    assert!(
        css.contains(".srv-sect + .srv-sect { border-top: 1px solid var(--border); }"),
        "the Server pane's shipped rule must survive byte-intact"
    );

    let html = ui_file("index.html");
    for pane in [r#"id="pane-identity""#, r#"id="pane-vault""#] {
        let p = &html[html.find(pane).unwrap_or_else(|| panic!("{pane}"))..];
        let p = &p[..p.find("</div>\n      <div id=\"pane-").unwrap_or(p.len())];
        assert!(p.contains("pane-sect"), "{pane} uses the section idiom");
    }
}

/// R-4: the verification code has ONE merged explainer and NO copy button.
///
/// ⚠ MUST GO RED IF: a copy button appears (the code is read aloud to a
/// contact, not pasted — a copy affordance advertises a workflow that does not
/// exist), or the explainer splits back into two stacked paragraphs.
#[test]
fn verification_code_has_one_explainer_and_no_copy_button() {
    let html = ui_file("index.html");
    let pane = &html[html.find(r#"id="pane-identity""#).expect("identity pane")..];
    let pane = &pane[..pane.find(r#"id="pane-server""#).expect("next pane")];
    // ⚠ Strip HTML comments before the substring ban. Caught by this assertion
    // failing on its first run: the comment EXPLAINING that there is no copy
    // button contains the word "copy". Third instance of this shape in the
    // lane — a needle that bans a substring across a region must exclude the
    // prose that documents the ban, or it fires on its own rationale.
    let markup = strip_html_comments(pane);
    assert!(
        !markup.to_lowercase().contains("copy"),
        "no copy button/affordance on the verification code"
    );
    assert!(
        pane.contains(r#"id="settings-explainer""#),
        "ONE merged explainer element"
    );
    assert!(
        !pane.contains(r#"id="settings-purpose""#) && !pane.contains(r#"id="settings-pq""#),
        "the two superseded stacked paragraphs are gone"
    );
    let js = ui_file("main.js");
    assert!(
        js.contains(
            r#"byId("settings-explainer").textContent = rec.purpose_line + " " + rec.pq_line"#
        ),
        "the merged explainer joins both lines"
    );
}

/// R-11: the erase-after-N controls are CONTEXTUAL, and F7's tier holds.
///
/// ⚠ MUST GO RED IF: Disarm renders while the feature is off (it was a dead
/// control — that is the defect), or the contextual toggle is done by DOM
/// removal instead of `.hidden` (which would unpin the button tiers), or
/// Disarm loses the mandatory `danger` tier token.
#[test]
fn erase_after_n_controls_are_contextual() {
    let js = ui_file("main.js");
    let f = js
        .find("function renderWipeState")
        .expect("renderWipeState");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];
    assert!(
        body.contains(r#"byId("btn-wipe-arm").classList.toggle("hidden", armed)"#),
        "Arm hides while armed"
    );
    assert!(
        body.contains(r#"byId("btn-wipe-disarm").classList.toggle("hidden", !armed)"#),
        "Disarm hides while off — it was a dead control there"
    );
    let html = ui_file("index.html");
    let disarm = html
        .split("<button")
        .find(|t| t.contains("btn-wipe-disarm"))
        .expect("disarm");
    assert!(
        disarm.contains("danger danger-outline"),
        "F7: the danger TIER TOKEN is mandatory, outline is the modifier"
    );
}

/// R-11/R-15: the armed line states how many attempts REMAIN.
///
/// ⚠ MUST GO RED IF: the remaining count is dropped, or computed from
/// something other than `wipe_after - failed_unlocks`, or allowed to go
/// negative. R-15's Phase-0 finding is what makes this honest: the destroy
/// pane never reaches this counter, so unlock attempts are the only thing
/// that walks the vault toward erasure.
#[test]
fn armed_line_shows_remaining_attempts() {
    let js = ui_file("main.js");
    let f = js
        .find("function remainingBeforeWipe")
        .expect("remainingBeforeWipe");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];
    assert!(
        body.contains("s.wipe_after - s.failed_unlocks"),
        "remaining is derived from the DTO already fetched — no new command"
    );
    assert!(
        body.contains("Math.max(0,"),
        "the count can never render negative"
    );
    assert!(
        js.contains("` · ${left} remaining`"),
        "the armed line states the remaining count"
    );
}

/// F1: quiet status lines carry danger TEXT, never danger CHROME.
///
/// ⚠ MUST GO RED IF: a background, border or fill from the danger family is
/// added to the status line. That is the exact edit the F1 refinement forbids,
/// and it is an easy one to make while "restoring emphasis".
#[test]
fn status_lines_carry_danger_text_never_chrome() {
    let css = ui_file("style.css");
    let b = rule_block(&css, ".status-line-quiet.is-danger");
    assert!(
        b.contains("color: var(--danger-text)"),
        "danger TEXT is allowed"
    );
    for banned in ["background", "border", "box-shadow"] {
        assert!(
            !b.contains(banned),
            "danger CHROME (`{banned}`) is reserved for the destroy ceremony"
        );
    }
}

/// R-17: user-facing errors are mapped BY SITE and never open with a bare code.
///
/// ⚠ MUST GO RED IF: `vault_locked` is given a single global wording. In the
/// DESTROY pane it means WRONG PASSPHRASE — Settings is unlock-gated, so the
/// vault is demonstrably unlocked and "your vault is locked" would be FALSE at
/// the one site the finding named as its example.
#[test]
fn errors_are_mapped_per_site_never_bare_codes() {
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"vault_locked: "That passphrase doesn't match. Nothing was destroyed.""#),
        "the destroy site says WRONG PASSPHRASE, not `your vault is locked`"
    );
    assert!(
        js.contains("function destroyErrorText"),
        "a per-site mapper"
    );
    assert!(js.contains("function unlockErrorText"), "a per-site mapper");
    // The fall-through always leads with a sentence; the code survives only in
    // parentheses. This is the mechanism behind NA-0674's naked
    // `vault_write_failed`.
    let f = js.find("function plainError").expect("plainError");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];
    assert!(
        body.contains("return `${lead} (${s})`"),
        "the fall-through leads with prose and parenthesises the code"
    );
    // No site may concatenate a raw error onto prose any more.
    for banned in [
        r#""Destroy refused: " + e"#,
        r#""Unlock failed: " + e"#,
        r#""Erase failed: " + e"#,
        r#""Not saved: " + e"#,
    ] {
        assert!(
            !js.contains(banned),
            "raw-code concatenation survives: {banned}"
        );
    }
}

/// R-18: the Unlock button's re-enable is STATE-driven, and the countdown
/// handle is nulled when a countdown ends.
///
/// ⚠ MUST GO RED IF: the handle stops being nulled at expiry, or the predicate
/// goes back to comparing a className. That combination is the shipped defect:
/// after one countdown the handle stayed truthy forever, so the re-enable
/// depended entirely on the feedback element's class string — and the catch
/// branch sets `"feedback reject"`, leaving Unlock PERMANENTLY DISABLED with a
/// raw error above it.
#[test]
fn unlock_reenable_is_state_driven() {
    let js = ui_file("main.js");
    let f = js.find("function startCountdown").expect("startCountdown");
    let body = &js[f..f + js[f..].find("\n}\n").expect("fn end")];
    assert!(
        body.contains("countdownTimer = null;"),
        "the handle MUST be nulled when the countdown ends"
    );
    assert!(
        js.contains("if (countdownTimer === null) btn.disabled = false;"),
        "the re-enable asks about the countdown, not about a class string"
    );
    assert!(
        !js.contains(r#"if (!countdownTimer || byId("unlock-feedback").className === "feedback")"#),
        "the superseded className-coupled predicate must not survive"
    );
}

/// R-14: the window table is a FLOOR, and the sync runs on the path that
/// actually trips the defect.
///
/// ⚠ MUST GO RED IF: the sync is wired only to `show()`. The autolock path
/// calls `show("scr-unlock")` and writes "Locked after inactivity." into the
/// feedback line AFTERWARDS — a fix tested only on surface-change passes its
/// own test and still clips "Delete vault?" below the fold. That is the
/// hollow-proof shape the operator ruled against explicitly.
#[test]
fn window_height_syncs_on_the_autolock_path_not_just_surface_change() {
    let js = ui_file("main.js");
    assert!(js.contains("function syncWindowHeight"), "the sync helper");
    assert!(
        js.contains("contentHeight: measurePreMainHeight()"),
        "the measurement rides the existing surface-change carrier"
    );

    // PATH (b), the one that matters: the autolock write must be FOLLOWED by a
    // sync, in that order.
    let write = js
        .find(r#"byId("unlock-feedback").textContent = "Locked after inactivity.";"#)
        .expect("the autolock feedback write");
    let after = &js[write..];
    let sync = after
        .find("syncWindowHeight();")
        .expect("a sync after the write");
    let next_fn = after.find("\n}").unwrap_or(after.len());
    assert!(
        sync < next_fn,
        "the autolock path MUST re-sync after writing the feedback line — \
         syncing only at show() misses the very content R-14 exists for"
    );

    // And the measurement must include the screen's padding, or a surface will
    // be sized to its content and clip at the edges.
    let f = js
        .find("function measurePreMainHeight")
        .expect("the measurer");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];
    assert!(
        body.contains("card.scrollHeight") && body.contains("pad"),
        "content height + the screen's own vertical padding"
    );
}

/// R-14, tested as BEHAVIOUR rather than as text: `height_for` is a floor.
///
/// ⚠ MUST GO RED IF: the function starts returning the measurement
/// unconditionally (which would let a short surface shrink below the
/// operator's chosen reading composition — the round-4a work), or starts
/// ignoring the measurement (which reinstates the clip this lane exists to
/// fix). Both directions are asserted, because "take the max" is exactly the
/// property and either half alone is wrong.
#[test]
fn window_height_table_is_a_floor_not_a_fixed_size() {
    for mode in [
        WindowMode::WizardVault,
        WindowMode::WizardIdentity,
        WindowMode::Unlock,
        WindowMode::Erase,
        WindowMode::Wiped,
    ] {
        let ((_, table), _, _) = window_mode_spec(mode);

        // No measurement -> the table governs, exactly as before this lane.
        assert_eq!(height_for(mode, None), table, "{mode:?}: no measurement");

        // Content SHORTER than the table -> the table still governs. This is
        // what keeps the operator's reading composition.
        assert_eq!(
            height_for(mode, Some(table - 40.0)),
            table,
            "{mode:?}: a short surface must NOT shrink below the floor"
        );

        // Content TALLER than the table -> the window grows. This is the fix.
        assert_eq!(
            height_for(mode, Some(table + 18.0)),
            table + 18.0,
            "{mode:?}: the window must grow to fit content the table never saw"
        );
    }

    // The concrete R-14 case, stated in its own numbers: the unlock window is
    // 255 measured against an EMPTY feedback line; one line of 12px/1.5 hint
    // text is ~18px, and the card is `overflow-y: auto`, so at a fixed 255 the
    // "Delete vault?" link fell below the fold.
    let ((_, unlock), _, _) = window_mode_spec(WindowMode::Unlock);
    assert_eq!(unlock, 255.0, "the floor is unchanged by this lane");
    assert_eq!(height_for(WindowMode::Unlock, Some(273.0)), 273.0);
}

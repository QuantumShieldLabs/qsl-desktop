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

use qsl_desktop_app::{height_for, size_for, window_mode_spec, WindowMode};
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

/// Strip `//` line comments from Rust source. Same lesson as
/// `strip_html_comments`, learned a FOURTH time: a needle that bans a
/// substring across a file fires on the comment explaining the ban. Here the
/// resolver's own note — recording WHY the `self_alias` signal was ruled out —
/// tripped two separate `self_alias` bans.
fn strip_rust_comments(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Strip `//` line comments from JS. Same lesson as the HTML and Rust
/// strippers — a substring ban fires on the comment explaining the ban.
fn strip_js_line_comments(js: &str) -> String {
    js.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") {
                ""
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
        step.contains(r#"placeholder="e.g. Alex""#),
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
        // ⚠ CODE only. `state.rs` documents WHY the `self_alias` resume signal
        // was ruled out (ENG-0076), and an unstripped ban fires on that prose —
        // the fourth time in this lane that a substring ban hit the comment
        // explaining the ban.
        let text = strip_rust_comments(&manifest_file(src));
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

/// R-14: the autolock path resizes AFTER writing its notice — the specific
/// case the operator ruled must be exercised, not just surface-change.
///
/// ⛔ AMENDED after the re-flight. This test used to look for the literal
/// `byId("unlock-feedback").textContent = "Locked after inactivity."` followed
/// by a `syncWindowHeight()`. That pinned the INSTANCE, and the re-flight
/// proved the instance was never the point: five other writers of the same
/// element had the same defect and this test was blind to all of them. The
/// class is now pinned by
/// `unlock_feedback_has_exactly_one_writer_and_it_resizes`; this one keeps the
/// autolock path specifically covered, because the operator ruled it must be.
///
/// ⚠ MUST GO RED IF: the autolock path stops routing its write through the
/// resizing writer — e.g. by setting `textContent` directly again.
#[test]
fn window_height_syncs_on_the_autolock_path_not_just_surface_change() {
    let js = ui_file("main.js");
    assert!(js.contains("function syncWindowHeight"), "the sync helper");
    assert!(
        js.contains("contentHeight: measurePreMainHeight()"),
        "the measurement rides the existing surface-change carrier"
    );

    // The autolock path must write through the RESIZING writer, and do so
    // after showing the unlock screen — the write lands after `show()`, so the
    // surface-change sync has already run against an empty line.
    //
    // ⚠ Anchored on the elapsed-time comparison, which is UNIQUE to the idle
    // timer. `showUnlockScreen("main")` appears three times (route, the idle
    // timer, the menu Lock-now) and a bare `find` returns route's — the same
    // "first match is not the one you mean" shape as the `.verify-code` slice.
    let timer = js
        .find("autolockMinutes * 60 * 1000")
        .expect("the idle-timer comparison");
    let branch = &js[timer..];
    let branch = &branch[..branch.find("}, 5000);").unwrap_or(branch.len())];
    assert!(
        branch.contains(r#"await showUnlockScreen("main");"#),
        "the idle timer redirects to unlock"
    );
    let show = branch.find("showUnlockScreen").unwrap();
    let write = branch
        .find(r#"setUnlockFeedback("locked-notice", "Locked after inactivity.")"#)
        .expect("the autolock notice must go through the RESIZING writer");
    assert!(
        show < write,
        "the notice is written AFTER the redirect — which is exactly why it \
         needs its own resize"
    );

    // The measurement must include the screen's padding, or a surface is sized
    // to its content and clips at the edges.
    let f = js
        .find("function measurePreMainHeight")
        .expect("the measurer");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];
    assert!(
        body.contains("card.scrollHeight") && body.contains("pad"),
        "content height + the screen's own vertical padding"
    );
}

/// Finding 1: `height_for` tracks its input in BOTH directions, with no floor.
///
/// ⚠ SCOPE, STATED HONESTLY BECAUSE THIS TEST ONCE OVERSTATED IT. This feeds
/// `height_for` SYNTHETIC values. It pins the FUNCTION, not the pipeline — and
/// the acceptance flight proved that distinction is not academic: this test
/// passed green on a build whose windows were visibly wrong in six places,
/// because the defect was in what REACHED the function (a stretched card
/// measuring the window). The pipeline is pinned separately by
/// `measurement_releases_the_stretch_before_reading`, and the OUTCOME can only
/// be judged by flying it — there is no input driver on the build host.
///
/// ⛔ SUPERSEDES this test's earlier form, which asserted the opposite — that a
/// surface shorter than its table value must NOT shrink. That floor was my
/// inference, never an operator instruction, and it CAUSED six of the seven
/// instances the live flight found: windows held open above their content.
///
/// ⚠ MUST GO RED IF: any mode stops tracking its measurement — in either
/// direction. The operator's ruling was explicit that a test covering one
/// window would be hollow, so this iterates EVERY mode, including the two that
/// are not measured today: if a future lane starts measuring Settings or Main,
/// the property is already pinned.
#[test]
fn every_window_tracks_its_content_in_both_directions() {
    let all = [
        WindowMode::WizardVault,
        WindowMode::WizardIdentity,
        WindowMode::Unlock,
        WindowMode::Erase,
        WindowMode::Wiped,
        WindowMode::Settings,
        WindowMode::Main,
    ];
    for mode in all {
        let ((_, fallback), (_, min_h), _) = window_mode_spec(mode);

        // No measurement yet -> the fallback, so the window opens at something.
        assert_eq!(height_for(mode, None), fallback, "{mode:?}: unmeasured");

        // ⚠ THE REVERSAL: content SHORTER than the fallback must SHRINK the
        // window. A `max(fallback, measured)` floor fails exactly here, and
        // that failure is what the flight observed six times.
        // ⚠ The probe is DERIVED from each mode's own headroom, not a fixed
        // offset. A flat `fallback - 60` underflowed Unlock (255-60 < 200) and
        // Wiped (220-60 < 200) and tripped this very guard — the assertion
        // caught a defect in its own input, which is the point of asserting the
        // precondition instead of assuming it.
        assert!(
            fallback > min_h,
            "{mode:?}: fallback must exceed the safety min"
        );
        let short = min_h + (fallback - min_h) / 2.0;
        assert_eq!(
            height_for(mode, Some(short)),
            short,
            "{mode:?}: a window MUST shrink to content — dead space is the defect"
        );

        // Content TALLER -> grows (the original R-14 clip).
        assert_eq!(height_for(mode, Some(fallback + 18.0)), fallback + 18.0);

        // The ONLY clamp is the absolute minimum, and it exists so the window
        // cannot become un-draggable — not to encode a preferred size.
        assert_eq!(
            height_for(mode, Some(min_h - 50.0)),
            min_h,
            "{mode:?}: clamp"
        );

        // "No window exceeds its content box by more than the intended
        // padding": the measurement already carries the surface's padding, so
        // the returned height must EQUAL it, never pad it again.
        for probe in [min_h + 1.0, fallback, fallback + 200.0] {
            assert_eq!(
                size_for(mode, Some(probe)).1,
                probe,
                "{mode:?}: height must equal the measured content box, not exceed it"
            );
        }
    }

    // Width is the mode's own and must not drift with content.
    for mode in all {
        let ((w, _), _, _) = window_mode_spec(mode);
        assert_eq!(
            size_for(mode, Some(1234.0)).0,
            w,
            "{mode:?}: width is stable"
        );
    }
}

/// Finding 1: the frontend measures EVERY pre-main surface, not just the one
/// that was reported.
///
/// ⚠ MUST GO RED IF: a pre-main screen is added to the app and left out of the
/// measured set — it would then silently inherit its fallback height forever,
/// which is how five of the seven instances stayed invisible until a human
/// looked at them.
#[test]
fn every_pre_main_surface_is_measured() {
    let js = ui_file("main.js");
    let f = js.find("const PRE_MAIN_SCREENS").expect("the measured set");
    let decl = &js[f..f + js[f..].find("];").expect("end of list")];
    for screen in [
        "scr-wizard-vault",
        "scr-wizard-identity",
        "scr-unlock",
        "scr-erase",
        "scr-wiped",
    ] {
        assert!(decl.contains(screen), "`{screen}` must be measured");
    }
    // And the Rust side must have a mode for each of them.
    let lib = manifest_file("src/lib.rs");
    for surface in [
        "scr-wizard-vault",
        "scr-wizard-identity",
        "scr-unlock",
        "scr-erase",
        "scr-wiped",
    ] {
        assert!(lib.contains(surface), "`{surface}` must map to a mode");
    }
    // Settings must NOT share the main window's mode any more (instance 4).
    assert!(
        lib.contains(r#""scr-settings" => WindowMode::Settings"#),
        "Settings needs its own mode, or opening it never resizes anything"
    );
}

/// Finding 4: the section heading rhythm matches mockup 09.
///
/// ⚠ MUST GO RED IF: the h3 top margin returns and stacks on the section
/// padding — 32 + 12 = 44px above a heading where the mockup draws 32. Scoped
/// to `.pane-sect h3`, so a heading outside a section is unaffected.
#[test]
fn section_headings_do_not_stack_margin_on_padding() {
    let css = ui_file("style.css");
    let b = rule_block(&css, ".pane-sect h3");
    assert!(
        b.contains("margin: 0 0 var(--sp-2) 0"),
        "no top margin; 8px below, per mockup 09"
    );
    // The section padding itself was never the defect and must stay at --sp-6.
    let sect = rule_block(&css, ".pane-sect {");
    assert!(
        sect.contains("padding: var(--sp-6) 0"),
        "32px section padding"
    );
}

/// Finding 3: the technical-details disclosure is GONE from onboarding and
/// KEPT in Settings.
///
/// ⚠ MUST GO RED IF: the disclosure returns to the wizard (premature — nothing
/// is being verified yet at that point), OR if it is removed from Settings too
/// (that is where identity detail is acted on). Both directions matter, which
/// is why both are asserted.
#[test]
fn technical_details_is_settings_only() {
    let html = ui_file("index.html");
    let wiz = &html[html.find(r#"id="scr-wizard-identity""#).expect("wizard")..];
    let wiz = &wiz[..wiz.find("</section>").expect("end")];
    let wiz_markup = strip_html_comments(wiz);
    assert!(
        !wiz_markup.contains("Show technical details"),
        "onboarding must NOT carry the disclosure"
    );
    assert!(
        !wiz_markup.contains("identity-fp") && !wiz_markup.contains("identity-mech"),
        "and neither its fingerprint nor its mechanism line"
    );

    let pane = &html[html.find(r#"id="pane-identity""#).expect("identity pane")..];
    let pane = &pane[..pane.find(r#"id="pane-server""#).expect("next pane")];
    assert!(
        pane.contains("Show technical details") && pane.contains("settings-fp"),
        "Settings KEEPS the disclosure — that is where identity detail is acted on"
    );
}

/// Finding 2: the verification code cannot clip its own glyphs.
///
/// ⚠ MUST GO RED IF: the explicit line-height is dropped back to the inherited
/// value. `overflow: hidden` is load-bearing for the shrink-to-fit logic and
/// stays, so the only defence against a clipped glyph bottom is a line box
/// taller than the glyph box at EVERY size fitCode can land on.
/// ⓘ This pins the mechanism; the PIXEL result needs the operator's re-flight,
/// because there is no input driver on the build host.
#[test]
fn verification_code_box_cannot_clip_its_glyphs() {
    let css = ui_file("style.css");
    let b = rule_block(&css, ".verify-code {");
    assert!(
        b.contains("line-height:"),
        "an EXPLICIT line-height — the inherited one left no headroom"
    );
    assert!(
        b.contains("white-space: nowrap"),
        "the frozen base property holds"
    );
    let js = ui_file("main.js");
    // fitCode changes the rendered size, so it must precede the measurement or
    // the window is sized against a code that is about to change.
    let fit = js
        .find(r#"fitCode(byId("identity-code"));"#)
        .expect("the wizard fit call");
    let after = &js[fit..];
    assert!(
        after[..after.find("\n}").unwrap_or(after.len())].contains("syncWindowHeight();"),
        "the height must be re-measured AFTER fitCode changes the code's size"
    );
}

/// ENG-0076 / D-0018: nothing may write `settings.json` before the onboarding
/// Continue, or the resume signal breaks silently.
///
/// ⚠ MUST GO RED IF: a write path to `settings.json` becomes reachable BEFORE
/// Continue. The whole fix rests on "settings.json does not exist until
/// Continue ran" — established by tracing every writer, not by assuming it. A
/// future pre-Continue write would make a killed onboarding look completed,
/// resume would resolve S2, and R-7's gate would be bypassed again **with no
/// other test failing**. This is that test.
///
/// The alternative signal — `self_alias` absent — was RULED OUT and must not
/// return: `skip_serializing_if = "String::is_empty"` omits an empty alias, so
/// key-absent also matches "name cleared in Settings" and matches every
/// pre-R-7 profile, which D615's F4 forbids re-routing.
#[test]
fn no_settings_write_precedes_onboarding_continue() {
    let js = ui_file("main.js");

    // The boot path READS settings and must never write them — otherwise a
    // launched-but-abandoned onboarding would look completed.
    let boot = js.find("// ---- boot").expect("the boot block");
    assert!(
        !js[boot..].contains("saveSettings"),
        "the boot path must not write settings.json"
    );

    // Backend: the two commands reachable before Continue must not persist.
    let cmds = manifest_file("src/commands.rs");
    for (sig, name) in [
        ("pub async fn vault_create", "vault_create"),
        ("pub fn settings_get", "settings_get"),
    ] {
        let f = cmds.find(sig).unwrap_or_else(|| panic!("{name} not found"));
        let body = &cmds[f..f + cmds[f..].find("\n}").expect("fn end")];
        assert!(
            !body.contains("settings::save"),
            "`{name}` is reachable before Continue and MUST NOT write settings.json"
        );
    }

    // Continue IS a save site — it is what creates the file.
    let cont = js
        .find(r#"byId("btn-identity-done").addEventListener"#)
        .expect("the Continue handler");
    let handler = &js[cont..cont + js[cont..].find("\n});").expect("handler end")];
    assert!(
        handler.contains("saveSettings()"),
        "Continue must write settings — that write IS the finished-step signal"
    );

    // And the resolver reads it, on the ruled signal only.
    let state_src = manifest_file("src/state.rs");
    assert!(
        state_src.contains("settings_file(data_dir).exists()"),
        "resume must gate S2 on settings.json existing"
    );
    assert!(
        !strip_rust_comments(&state_src).contains("self_alias"),
        "the self_alias signal was ruled out and must not return"
    );
}

/// ⚠ FACT 1 (the acceptance flight's root cause): the measurement must RELEASE
/// the card's stretch before reading, or it measures the window.
///
/// `.screen` is `display:flex` with `align-items: stretch`, so the card's
/// height IS the window height. A stretched box whose content is shorter
/// reports its own height from `scrollHeight`, making the measurement
/// self-referential — `measured = window_height` — so the window could grow
/// but never shrink. Two different surfaces reported an identical 388x765
/// because the size was inherited, not computed.
///
/// ⚠ MUST GO RED IF: the un-stretch is removed. That single line is the whole
/// difference between a content-driven window and a feedback loop, and its
/// absence is INVISIBLE to every other test in this repository — the sizing
/// test above passed throughout.
#[test]
fn measurement_releases_the_stretch_before_reading() {
    let js = ui_file("main.js");
    let f = js
        .find("function measurePreMainHeight")
        .expect("the measurer");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];

    let release = body
        .find(r#"card.style.alignSelf = "flex-start""#)
        .expect("the stretch MUST be released before the read");
    let read = body.find("card.scrollHeight").expect("the read");
    let restore = body
        .find("card.style.alignSelf = prevAlignSelf")
        .expect("the stretch must be restored");
    assert!(
        release < read && read < restore,
        "order must be release -> read -> restore (got {release}/{read}/{restore})"
    );

    // And the stretch it is releasing must actually still be there — if the
    // stylesheet stops stretching the card, this dance is dead code and should
    // be removed deliberately rather than left as cargo.
    let css = ui_file("style.css");
    assert!(
        css.contains("align-items: stretch"),
        "the un-stretch exists because the card IS stretched; if that changed, \
         revisit the measurement rather than leaving this in place"
    );
}

/// ⚠ FACT 2: the card's children must not SHRINK — a too-short window scrolls,
/// it does not crush content.
///
/// Nothing in the stylesheet set `flex-shrink`, so every child of the flex
/// column carried the default `1`. A window shorter than its content squashed
/// the children, and `.verify-code`'s `overflow: hidden` turned that squash
/// into a clipped glyph bottom. **This is also the ORIGINAL R-14 defect**:
/// "Delete vault?" was never below a scroll fold, it was squashed — which is
/// why raising the window height appeared to fix it.
///
/// ⚠ MUST GO RED IF: the rule is dropped. The failure it prevents is silent
/// and content-dependent — it appears only when a window is shorter than its
/// content, which is precisely the case no fixed-height test exercises.
#[test]
fn pre_main_card_children_never_shrink() {
    let css = ui_file("style.css");
    let start = css
        .find("#scr-wizard-vault .card > *")
        .expect("the no-shrink rule must exist");
    let block = &css[start..css[start..].find('}').expect("rule end") + start];
    assert!(
        block.contains("flex-shrink: 0"),
        "children must not shrink; the card scrolls instead"
    );
    for screen in [
        "#scr-wizard-identity .card > *",
        "#scr-unlock .card > *",
        "#scr-erase .card > *",
        "#scr-wiped .card > *",
    ] {
        assert!(
            block.contains(screen),
            "`{screen}` must be covered — a surface left out squashes silently"
        );
    }
    // The card must still be able to scroll, or disabling shrink just clips.
    assert!(
        css.contains("overflow-y: auto"),
        "the card scrolls when content exceeds it"
    );
}

/// Instance 4: the Settings width is derived from the constant that actually
/// renders.
///
/// ⚠ MUST GO RED IF: the derivation goes back to `.pane`'s 560 cap. That
/// produced a window 40px too wide, visible as asymmetric insets — 20px from
/// the nav rail to a section hairline, 60px from its end to the window edge.
/// The hairlines span `.pane-form`, so `.pane-form` decides the width.
#[test]
fn settings_width_derives_from_the_form_cap() {
    let css = ui_file("style.css");
    let form = rule_block(&css, ".pane-form {");
    assert!(
        form.contains("max-width: 520px"),
        "the form cap is the constant the window derives from"
    );
    assert_eq!(
        window_mode_spec(WindowMode::Settings).0 .0,
        52.0 + 160.0 + 520.0 + 40.0,
        "the window must equal rail + nav + FORM cap + padding, so both insets match"
    );
}

/// ⚠ R-14, THIRD OCCURRENCE: writing the unlock feedback MUST resize the
/// window, and this pins the CLASS rather than the instance.
///
/// The re-flight found "Delete vault?" vanishing the moment a wrong passphrase
/// was entered: the feedback text appears, content grows, the window does not,
/// and the link is pushed out of view. Enlarging the window by hand brought it
/// back — the signature of content outgrowing an unresized window.
///
/// ⚠ WHY THIS TEST IS SHAPED THIS WAY. The previous fix wired the sync at the
/// ONE write the finding named (the autolock notice) while D615's own rule
/// says "after ANY write to a conditional element" — so five other writers
/// silently kept the bug. **Asserting that the known writers call the sync
/// would repeat that mistake**: it can only ever cover the writers that exist
/// today. Instead this asserts there is exactly ONE way to write the element,
/// and that way resizes — so a new writer added later cannot reintroduce the
/// defect without failing here.
///
/// ⚠ MUST GO RED IF: any code writes `#unlock-feedback` outside the helper, or
/// the helper stops resizing.
#[test]
fn unlock_feedback_has_exactly_one_writer_and_it_resizes() {
    let js = ui_file("main.js");

    // The helper writes AND resizes — the two are one operation.
    let f = js
        .find("function setUnlockFeedback")
        .expect("the single writer must exist");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];
    assert!(
        body.contains("fb.textContent = text") && body.contains("syncWindowHeight()"),
        "the writer must resize in the same call, not rely on callers remembering"
    );

    // ⚠ NOBODY ELSE may touch the element. Comments are stripped so the
    // explanation of this rule does not trip it — the fourth time that has
    // mattered in this lane.
    let code = strip_js_line_comments(&js);
    let writers = code.matches("unlock-feedback").count();
    assert_eq!(
        writers, 1,
        "exactly ONE reference to #unlock-feedback may exist in code (inside \
         setUnlockFeedback); found {writers}. A second writer is how this defect \
         returned twice."
    );

    // And the element is genuinely conditional — it is empty until written,
    // which is what makes an unresized write shift the layout.
    let html = ui_file("index.html");
    assert!(
        html.contains(r#"<div id="unlock-feedback" class="feedback"></div>"#),
        "the feedback element starts EMPTY; that is why writing it changes height"
    );
}

/// ⚠ R-14, FOURTH OCCURRENCE (ENG-0123 — the GUI driver's first machine
/// catch): writing the ceremony error MUST resize the window, and this pins
/// the CLASS on the erase screen's error line.
///
/// NA-0701's driver clicked where a user would click and was refused: after a
/// wrong ceremony phrase the error line appears, content grows, the window
/// does not — and BOTH Erase and Cancel fall outside the card's clip on the
/// app's most stressful screen. Same shape as the unlock feedback above; same
/// structural remedy: exactly ONE way to write the element, that way resizes,
/// and the reference count is the tripwire against a second writer.
///
/// ⚠ MUST GO RED IF: any code writes `#erase-error` outside the helper, or
/// the helper stops resizing.
#[test]
fn erase_error_has_exactly_one_writer_and_it_resizes() {
    let js = ui_file("main.js");

    // The helper writes AND resizes — the two are one operation.
    let f = js
        .find("function setEraseError")
        .expect("the single writer must exist");
    let body = &js[f..f + js[f..].find("\n}").expect("fn end")];
    assert!(
        body.contains("el.textContent = text") && body.contains("syncWindowHeight()"),
        "the writer must resize in the same call, not rely on callers remembering"
    );

    // ⚠ NOBODY ELSE may touch the element. Comments are stripped so the
    // explanation of this rule does not trip it.
    let code = strip_js_line_comments(&js);
    let writers = code.matches("erase-error").count();
    assert_eq!(
        writers, 1,
        "exactly ONE reference to #erase-error may exist in code (inside \
         setEraseError); found {writers}. A second writer is how the R-14 class \
         returned three times before the driver caught this one."
    );

    // And the element is genuinely conditional — it is empty until written,
    // which is what makes an unresized write shift the layout.
    let html = ui_file("index.html");
    assert!(
        html.contains(r#"<div id="erase-error" class="error"></div>"#),
        "the error element starts EMPTY; that is why writing it changes height"
    );
}

/// NA-0752 (D-0033; seal F1b, ruled at R374) — THE THREE UNDRIVABLE FOOTER
/// SENTENCES ARE PRESENT, AND THEIR ARMS ARE WIRED.
///
/// ⚠⚠ THIS TEST PROVES **PRESENCE**, NEVER **BEHAVIOUR**, and the distinction is
/// the whole reason it exists rather than a scenario. Three rows of the ruled
/// status-footer table cannot be driven end-to-end by the GUI harness, each for
/// a measured structural reason: `missing_home` is unreachable while
/// `bootstrap()` sets `QSC_CONFIG_DIR` before the runtime exists;
/// `unsafe_parent` needs the app's own 0700 qsc dir chmodded mid-session, and no
/// harness op writes or chmods the profile; the footer lives INSIDE `scr-main`
/// so it is never on screen while the vault is locked; and `unrecognized`
/// requires an EIGHTH upstream reason string, i.e. it is untestable by
/// construction. The two DRIVABLE rows are asserted for real, by equality on
/// extracted text, in `f_h_status_footer_truth`.
///
/// The desk-side behaviour these rows render IS proven where it lives:
/// `na0751_facade_locked_control.rs:141-174` asserts `ConnectReason::VaultLocked`
/// on a fabricated blob with both arms shown to differ. This test closes the
/// remaining gap — that the FOOTER still carries the sentence and still routes
/// the reason to it.
///
/// THE REGRESSION IT MUST CATCH: a refactor that drops an arm, or silently
/// reworded copy, leaving a row that can never render. Both are invisible to
/// the scenario, because the scenario never reaches these rows.
#[test]
fn na0752_the_three_undrivable_footer_sentences_are_present_and_wired() {
    let js = ui_file("main.js");

    // The sentences, as CONST DECLARATIONS — not merely as text somewhere in the
    // file. A string that survived only inside a comment would satisfy a naive
    // `contains` and render nothing.
    for (name, sentence) in [
        (
            "STATUS_FOOTER_STORAGE",
            "Storage problem — check Settings › Vault.",
        ),
        ("STATUS_FOOTER_LOCKED", "Locked — unlock to connect."),
        (
            "STATUS_FOOTER_UNKNOWN",
            "Status unknown — please report this.",
        ),
    ] {
        let decl = format!("const {name} = \"{sentence}\";");
        assert!(
            js.contains(&decl),
            "the ruled sentence must exist as the `{name}` declaration, verbatim: {decl}"
        );
    }

    // And each arm is WIRED: the reason tokens the desk actually emits must be
    // matched against, or the sentence is present and unreachable.
    let f = js
        .find("function statusFooterLine")
        .expect("the footer mapping is ONE named pure function");
    let body = &js[f..f + js[f..].find("\n}").expect("function end")];
    for token in [
        "missing_home",
        "unsafe_parent",
        "vault_locked",
        "unrecognized",
    ] {
        assert!(
            body.contains(&format!("\"{token}\"")),
            "`{token}` must be matched inside statusFooterLine, or its row is dead copy"
        );
    }

    // The residual arm is reached by a desk that did not answer at all, which is
    // how a typed IPC failure avoids rendering as silence.
    assert!(
        body.contains("reason === null"),
        "a rejected invoke must land on the honest tripwire, not on an empty footer"
    );
}

/// NA-0753 (R376 §3, R377 §3; D-0034) — THE PORT HINT, PRESENCE-SEALED.
///
/// ⚠ PRESENCE IS NOT BEHAVIOUR, and this seal says so in its own doc. The
/// blessed sentence renders ONLY in the `unreachable` result state, which
/// needs a real connection attempt to an address that refuses one — the
/// harness cannot drive that without a network dependency and a timeout. So
/// the sentence is pinned here by PRESENCE and by its ATTACHMENT to the
/// unreachable branch; `f_i_flight_fixes` drives the gate, which is the arm
/// that CAN be driven.
///
/// ⚠ MUST GO RED IF: the sentence is edited, drifts out of the `unreachable`
/// case into another result state, or reverts to the design bank's ASCII
/// double-hyphen. The bank is pure-ASCII transport armor; every user-facing
/// string in this app uses the em-dash house form (ruled at R377 §3, and the
/// deviation from the bank's bytes is enumerated in D-0034).
#[test]
fn port_hint_rides_the_unreachable_helper() {
    let js = ui_file("main.js");
    let hint = "If your relay operator uses a non-standard port, include it — for example https://relay.example.org:8443.";
    assert!(
        js.contains(hint),
        "the blessed port-hint sentence is present"
    );
    let start = js
        .find(r#"case "unreachable":"#)
        .expect("the unreachable case");
    let rest = &js[start..];
    let end = rest
        .find(r#"case "not_a_qsl_relay""#)
        .expect("the case that follows unreachable");
    assert!(
        rest[..end].contains(hint),
        "the hint must live INSIDE the unreachable branch, not merely somewhere in the file"
    );
    assert!(
        !js.contains("include it -- for example"),
        "house typography: the em-dash form ships, never the bank's ASCII armor"
    );
}

/// NA-0753 (R376 §3; ENG-0218) — THE GATE MUST NEVER PARSE WITH `new URL()`.
///
/// The whole point of the relay-address gate is to refuse what WHATWG URL
/// parsing ACCEPTS: an all-digit host is read as a packed IPv4 integer, so
/// `https://1234` becomes `https://0.0.4.210` — a real server nobody typed.
/// The engine does exactly that (`qsc` route.rs:50-74; FILED as ENG-0218 for a
/// guarded engine lane, deliberately not patched here). The webview's own
/// `new URL()` performs the SAME expansion, so "simplifying" the gate with it
/// would silently reinstate the defect while every accepting-arm test kept
/// passing.
///
/// ⚠ MUST GO RED IF: `new URL(` appears in main.js outside a comment.
#[test]
fn the_relay_gate_never_uses_the_webview_url_parser() {
    let js = ui_file("main.js");
    assert!(
        js.contains("function relayGateCheck"),
        "the gate function is present"
    );
    let code = strip_js_line_comments(&js);
    assert!(
        !code.contains("new URL("),
        "the gate splits the authority BY HAND — `new URL()` re-introduces the \
         integer-IP expansion the gate exists to refuse"
    );
}

/// NA-0753 (R377 §1; D-0034) — the grouped verification code is wired on BOTH
/// surfaces, and it is ONE text node, never a `<br>` split.
///
/// ⚠ MUST GO RED IF: a surface stops grouping, or someone reinstates the
/// mockup's `<br>`. That is not a style preference: `.verify-code` is
/// `white-space: nowrap; overflow: hidden`, and `fitCode()` only releases the
/// clip (adding `.wrapped`) when `scrollWidth > clientWidth`. A `<br>` halves
/// each line's width, so the escape could never fire and the second line would
/// clip SILENTLY — the exact class `verify_code_never_clips_silently` exists to
/// prevent. That mechanism is why R377 §1 ruled option (B); the resulting
/// delta from mockup-07's fixed 3+3 split is enumerated in D-0034.
#[test]
fn grouped_verification_code_is_wired_on_both_surfaces() {
    let js = ui_file("main.js");
    assert!(js.contains("function groupedCode"), "the grouper exists");
    assert!(
        js.contains(r#"byId("identity-code").textContent = groupedCode("#),
        "the onboarding surface groups"
    );
    assert!(
        js.contains(r#"byId("settings-code").textContent = groupedCode("#),
        "the Settings > Identity surface groups"
    );
    let start = js.find("function groupedCode").expect("grouper");
    let body = &js[start..];
    let body = &body[..body.find("\n}").expect("grouper body end")];
    assert!(
        !body.contains("createElement(\"br\")") && !body.contains("<br>"),
        "ONE text node — a <br> split would defeat fitCode's clip escape (R377 §1)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// NA-0754 (D-0035) — TEST-AND-SAVE-ON-PROOF: the source disciplines behind the
// invariant. These are SOURCE PINS, and presence is not behaviour — the
// behavioural halves live in `na0754_persist_boundary.rs` (engine, with its
// counterfactual red runs) and scenario `f_j` (surface). What these catch is a
// later edit quietly reintroducing the SHAPE the lane removed.
// ─────────────────────────────────────────────────────────────────────────────

/// NA-0754 — THE ORDER IS THE INVARIANT: the probe runs before any write.
///
/// ⚠ MUST GO RED IF: someone reintroduces a persist call ahead of the probe in
/// the Test handler. That is the whole defect class — under the old model the
/// handler committed everything and THEN probed the just-saved state, so a
/// failed test clobbered a proven-good configuration. This pins the ORDER in the
/// one function where it is decided, by position, not by comment.
#[test]
fn the_test_handler_probes_before_it_persists() {
    let js = ui_file("main.js");
    let start = js
        .find(r#"byId("btn-relay-test").addEventListener"#)
        .expect("the Test handler exists");
    let body = &js[start..];
    let probe = body
        .find(r#"invoke("relay_probe""#)
        .expect("the handler probes");
    let persist = body
        .find("persistProvenSettings(proven)")
        .expect("the handler persists through the ruled order");
    assert!(
        probe < persist,
        "the probe MUST precede the persist — a write before the probe is the \
         clobber class this lane removed structurally"
    );
    // And the persist is gated on the ACCEPTING outcome, not merely sequenced
    // after the probe: sequence without a gate would still save on a red rung.
    let gate = body
        .find(r#"res.kind === "reachable""#)
        .expect("the persist is gated on Connected");
    assert!(
        gate < persist,
        "persisting must be GATED on a Connected result, not just ordered after the probe"
    );
}

/// NA-0754 (R379 §Q2) — R-B1's ORIGINAL ORDER, RESTORED: token → CA → settings LAST.
///
/// ⚠ MUST GO RED IF: settings.json moves back to first. It was first only because
/// R-B2 forced it there when validating meant writing; with the probe now first
/// that forcing is gone. settings.json is the OBSERVABLE configuration (the status
/// footer and relaunch both read it), so writing it last is what keeps the
/// surviving configuration coherent when a vault write fails.
#[test]
fn the_persist_order_is_token_then_ca_then_settings_last() {
    let js = ui_file("main.js");
    let start = js
        .find("async function persistProvenSettings(")
        .expect("the persist function exists");
    let body = &js[start..];
    let end = body.find("\n}\n").expect("persist body end");
    let body = &body[..end];
    let token = body
        .find(r#"invoke("relay_token_set""#)
        .expect("token write");
    let ca = body
        .find(r#"invoke("relay_ca_file_set""#)
        .expect("CA write");
    let settings = body
        .find(r#"invoke("relay_config_set""#)
        .expect("settings write");
    assert!(
        token < ca && ca < settings,
        "the ruled order is vault token -> vault CA -> settings.json LAST \
         (got token@{token} ca@{ca} settings@{settings})"
    );
}

/// NA-0754 (design bank v2 item 2) — the clear controls delete IMMEDIATELY and
/// never route through a commit.
///
/// ⚠ MUST GO RED IF: someone reintroduces a pending-removal flag. The old
/// "remove it" links set a flag that only committed inside the commit function,
/// whose first step was the address — so a mistyped address made a stored token
/// undeletable, breaking the house rule that a stored secret must ALWAYS be
/// removable (ENG-0225). The affordance's whole value is that it works when
/// nothing else does.
#[test]
fn the_clear_controls_delete_immediately_and_offline() {
    let js = ui_file("main.js");
    assert!(
        !js.contains("PendingRemoval"),
        "a pending-removal flag is back — removal must not depend on a commit path"
    );
    let start = js
        .find("async function clearStoredSecret(")
        .expect("the immediate clear exists");
    let body = &js[start..];
    let end = body.find("\n}\n").expect("clear body end");
    let body = &body[..end];
    assert!(
        body.contains(r#""relay_token_clear""#) && body.contains(r#""relay_ca_file_clear""#),
        "both clears go straight to the vault trio"
    );
    // Neither clear may reach the network: those two commands are pure vault
    // writes, and adding a probe here would make the affordance relay-dependent.
    assert!(
        !body.contains("relay_probe") && !body.contains("relay_config_set"),
        "a clear must not probe or touch settings — it has to work with no relay reachable"
    );
    for id in ["relay-token-clear", "relay-ca-clear"] {
        assert!(
            ui_file("index.html").contains(&format!(r#"id="{id}""#)),
            "the {id} control must exist to host the affordance"
        );
    }
}

/// NA-0754 (design bank v2 item 4) — a leading `~/` is expanded VISIBLY, in the
/// field, before the path is used; anything else shell-shaped is refused.
///
/// ⚠ MUST GO RED IF: the write-back disappears, or the gate starts guessing when
/// $HOME is unresolvable. Expanding invisibly would send the probe at a path the
/// user never saw and then PERSIST it on success — silent, and persisted.
#[test]
fn the_ca_path_gate_expands_tilde_visibly_and_refuses_other_shell_tokens() {
    let js = ui_file("main.js");
    let start = js
        .find("function caPathGateCheck(")
        .expect("the CA path gate exists");
    let body = &js[start..];
    let end = body.find("\n}\n").expect("gate body end");
    let body = &body[..end];
    assert!(
        body.contains("homeDir === \"\""),
        "an unresolvable $HOME must REFUSE, never guess at a path nobody typed"
    );
    assert!(
        body.contains("Use the full path"),
        "the ruled refusal string is missing"
    );
    // The expansion must be written back into the field before the probe.
    assert!(
        js.contains(r#"if (caGate.expanded) byId("relay-ca-path").value = caGate.value;"#),
        "the expansion must be VISIBLE in the field before any test runs"
    );
}

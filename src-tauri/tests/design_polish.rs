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

// ===========================================================================
// NA-0755 (D-0036) — INVITE LANE A: THE CREATE FLOW.
//
// These are the seals for behaviour the offline harness cannot drive, plus the
// character-for-character copy pins. What the harness DOES drive lives in
// `f_k_invite_create.json`; what real IPC already pins lives in
// `na0751_gateway_surface.rs`, which drove `invite_create`/`invite_revoke`/
// `invite_list` through the mock runtime BEFORE this lane and is deliberately
// not duplicated here.
// ===========================================================================

/// Whitespace-collapsing containment: HTML wraps a sentence across source lines,
/// and a copy pin must survive that without becoming a pin on the indentation.
fn html_says(hay: &str, needle: &str) -> bool {
    let flat: String = hay.split_whitespace().collect::<Vec<_>>().join(" ");
    let n: String = needle.split_whitespace().collect::<Vec<_>>().join(" ");
    flat.contains(&n)
}

/// Z5 — THE MOCKUP-VERBATIM COPY, from the blob at 142c1eb6.
///
/// ⚠ THIS IS THE SOURCE PIN AND IT IS THE ONE THAT MATTERS. The scenario's
/// `read_text` reads WebDriver's RENDERED text, which applies `text-transform` —
/// measured at build time when `.tag` came back `INVITE — STEP 1`. Rendered text
/// pins what the user sees; only this pin holds the STRING.
///
/// ⚠ MUST GO RED IF: any of these sentences is reworded. They are the ratified
/// mockup's own bytes, and the brief's S6 transcription of them DIVERGED IN SIX
/// PLACES — which is why the mockup, not the brief, is quoted here.
#[test]
fn invite_modal_copy_is_mockup_verbatim() {
    let html = ui_file("index.html");
    // ⚠⚠ RE-SCOPED AT v2. The v1 form pinned mockup-14's TWO-STEP copy, including the
    // "Invite — step 1" / "step 2" tags. The v2 design is a SINGLE VIEW WITH NO EXPLAINER
    // STEP, so those tags are gone BY DESIGN, not by drift — pinning them would have held the
    // surface to a shape the operator's own flight retired. What survives is the copy the v2
    // bank still carries, and it is still checked character-for-character.
    assert!(html_says(&html, "Invite someone"), "the mint heading");
    // ⚠⚠ THE EXPLAINER PARAGRAPH IS DELIBERATELY GONE AT v2, and this is where that is
    // recorded. mockup-14's step-1 body ("Create a one-time invite code and send it to the
    // person you want to add …") was an EXPLAINER STEP, and the v2 bank rules the mint view
    // "the landing — '+' and the welcome button open straight here; NO EXPLAINER STEP".
    // Pinning it would hold the surface to the shape the operator's flight retired.
    //
    // ⚠ The ACCEPTED COST is stated rather than left implicit: the "share it through a channel
    // you trust" guidance no longer greets the user BEFORE minting. It survives where it is
    // actionable — in the callout beside the live code, asserted below — which is the moment
    // the advice can still change what they do.
    // ⚠⚠ v4 — THE OPERATOR-AUTHORED WARNING, PINNED CHARACTER-FOR-CHARACTER. It supersedes the
    // ratified mockup-14 sentence, and the mockup-refresh note rides the records. The leading
    // clause is BOLD, the ellipsis is the HOUSE character (`&hellip;`, never three dots), and
    // the apostrophe is the house curly form.
    assert!(
        html_says(
            &html,
            "<strong>Only send this code to the person it&rsquo;s intended for&hellip;</strong> \
             over a secure channel that you fully trust such as a text message, a call, or in \
             person. It works only once, then dies. If unused, it expires on its own in"
        ),
        "the operator-authored warning, verbatim, with the bold lead and the house ellipsis"
    );
    assert!(
        !html.contains("for..."),
        "the ellipsis must be the house character, never three dots"
    );
    // v2's single commit, and the label field the mockup always had.
    assert!(
        html_says(&html, "Activate &amp; Copy"),
        "the single-commit button"
    );
    // ⚠ NA-0766 (`D-0043`) RE-AIM — ITEMS 8 and 9. The caption takes the blessed v9 form: the
    // question, four NON-BREAKING spaces (four literal spaces collapse to one in HTML, so this
    // is the only form that RENDERS the drawn gap), and the local-only assurance called out in
    // the accent token. The separate private-note hint line is DELETED — the parenthetical says
    // the same thing where the eye already is — so its pin retires with it rather than being
    // re-aimed at nothing.
    assert!(
        html_says(
            &html,
            "Who is this invite for?&nbsp;&nbsp;&nbsp;&nbsp;<span class=\"field-note-accent\">(stored only on this device)</span>"
        ),
        "the recipient-label caption, layout-verbatim, with the assurance in the accent token"
    );
    assert!(html_says(&html, "Invite code"), "the code field label");
    // ⚠⚠⚠ NA-0766 (`D-0043`), RULING Q5 = (B) — AN ASSERTION IS REMOVED HERE, AND IT IS NOT A
    // WEAKENING. THIS COMMENT IS THE RECORD OF WHY, SO NO LATER READER CAN MISTAKE IT FOR A
    // CONVENIENCE DELETION.
    //
    // What stood here asserted that the mid-mint cancel control's label was PRESENT and was
    // "the ONLY post-activate action". v4 REMOVED that control. The assertion nevertheless went
    // on passing for two lanes, because `html_says` collapses whitespace but does NOT strip HTML
    // comments, and the label's only remaining occurrence in the markup was inside the comment
    // that RECORDED THE REMOVAL. Proven both arms at NA-0766 STOP 1 and confirmed independently
    // by the Director: raw index.html -> present (exactly one occurrence); comments stripped ->
    // ABSENT (zero). Positive control: real markup survives stripping. Negative control: a string
    // present nowhere is absent in both.
    //
    // Meanwhile `the_cancel_invite_control_is_gone_and_revoke_is_the_single_kill`, 700-odd lines
    // below, asserted the EXACT OPPOSITE — that the control is gone — and also passed, because
    // ITS needles are on the SHIPPED FORMS (a rendered text node, an id) which a comment cannot
    // produce. Two green seals, contradicting each other, and the contradiction was invisible
    // because one of them could no longer fail.
    //
    // Sec 8's "no test weakened, skipped or deleted" protects assertions that can CATCH
    // something. This one could not: it had been unfalsifiable since v4. Deleting a disproven
    // assertion while its true counterpart stands RAISES coverage rather than lowering it.
    //
    // THE GENERAL PROPERTY, which is worth more than the fix and is now itself sealed in
    // `na0766_invite_flow.rs::na0766_a_comment_cannot_satisfy_a_copy_seal`:
    //   A COMMENT THAT DOCUMENTS A REMOVAL RE-PLANTS THE REMOVED THING'S NEEDLE.
    // Any seal built on bare-word presence can be held green by the explanation of its own
    // subject's deletion. The cure is the one the sibling test already used — needle on the
    // SHIPPED FORM, never the bare word.
    // ⚠ v2 retires "New code" and "Copy code" as BUTTONS: re-minting is a fresh activation and
    // copy-again is a glyph, whose own click is its own gesture. Pinned NEGATIVELY so a
    // re-introduction is a deliberate act, not a drift.
    assert!(!html_says(&html, ">New code<"), "v2 has no New code button");
    assert!(
        !html_says(&html, ">Copy code<"),
        "v2 has no Copy code button"
    );
    // Δ5 — the callout. The mockup's first sentence, NOT the brief's invented
    // "Treat the code like a house key while it's live."
    // ⚠⚠ SUPERSEDED AT v4, and the supersession is stated rather than silently dropped. This
    // pinned mockup-14's callout sentence ("Share it through a channel you trust — a text, a
    // call, or in person…"). The operator AUTHORED a replacement across five mockup rounds, and
    // the v4 bank rules it final: "This supersedes the ratified mockup-14 sentence — the
    // mockup-refresh note rides the records."
    //
    // ⚠ The mockup file itself is NOT edited (it is forbidden this lane), so the tree now holds
    // a ratified mockup whose callout copy the shipped surface deliberately does not match.
    // That divergence is recorded in D-0036 rather than left for a future reader to find as
    // drift. The replacement is pinned character-for-character above.
    assert!(
        !html_says(
            &html,
            "Share it through a channel you trust &mdash; a text, a call, or in person."
        ),
        "the superseded mockup-14 callout must not ship alongside its replacement"
    );
    assert!(
        !html_says(&html, "house key"),
        "the brief's 'house key' sentence appears nowhere in the ratified mockup and MUST NOT ship (R380 §6)"
    );
}

/// Δ5 — THE CALLOUT USES `.callout`, NEVER THE MOCKUP'S `.warn`.
///
/// ⚠ MUST GO RED IF: someone copies the mockup's class name across. `style.css:277`
/// records the ruling that renamed it: "a class named `warn` rendering in accent is
/// a lie the next reader has to decode". Adopting the mockup's name would re-open
/// a defect the tree already closed by name.
#[test]
fn the_invite_callout_uses_the_renamed_class_not_the_mockups_warn() {
    let html = ui_file("index.html");
    let start = html
        .find(r#"id="invite-overlay""#)
        .expect("the modal exists");
    let modal = &html[start..];
    let end = modal.find("<script src=").unwrap_or(modal.len());
    let modal = &modal[..end];
    assert!(
        modal.contains(r#"class="callout""#),
        "the modal uses the shipped .callout"
    );
    assert!(
        !modal.contains(r#"class="warn""#),
        "the mockup's `.warn` name must not be adopted — style.css:277 renamed it because it lied"
    );
}

/// Z4 — THE ONE-TIME BOUNDARY, and it is STRUCTURAL rather than remembered.
///
/// ⚠ MUST GO RED IF: `show()` stops closing the modal. The overlay is deliberately
/// NOT a `SCREENS` member, so the screen loop cannot hide it; without this call an
/// autolock firing with the modal open leaves a live one-time code rendered over
/// the unlock screen. There are eight `show()` call sites and this covers all of
/// them, including ones not yet written.
#[test]
fn every_screen_transition_closes_the_invite_modal() {
    let js = ui_file("main.js");
    let start = js.find("function show(id) {").expect("show() exists");
    // ⚠ NA-0756: the slice now runs to the ACTUAL end of `show()` instead of a fixed
    // 1400-byte window. The old bound was an INSTRUMENT limit, not the property: adding one
    // documented line to the function pushed its closing brace outside the window and the
    // seal failed to FIND the body rather than failing to find the call. A pin whose reach
    // depends on how much comment a function carries measures the wrong thing.
    let body = &js[start..];
    let end = body.find("\n}\n").expect("show() ends");
    let body = &body[..end];
    assert!(
        body.contains("closeInviteModal()"),
        "show() MUST close the invite modal — the overlay is not a SCREENS member, so \
         nothing else can, and the autolock path (`show(\"scr-unlock\")`) is one of the \
         call sites this protects"
    );
    // NA-0756 (D-0037): the redeem overlay is held to the SAME rule and the seal now measures
    // BOTH. A pasted invite code is a one-time capability exactly as a minted one is, and the
    // second overlay would otherwise inherit the protection by luck rather than by check.
    assert!(
        body.contains("closeRedeemModal()"),
        "show() MUST close the redeem overlay too — a pasted code is one-time capability and \
         an autolock firing with the redeem surface open would leave it over the unlock screen"
    );
    // And the overlays are genuinely NOT in SCREENS: if either were added there, this
    // seal would be measuring a redundancy instead of the real boundary.
    let screens_start = js.find("const SCREENS = [").expect("SCREENS exists");
    let screens = &js[screens_start..screens_start + 200];
    assert!(
        !screens.contains("invite-overlay"),
        "the overlay must stay OUT of SCREENS — putting it in would make it a navigation \
         destination and silently defeat the reason show() closes it"
    );
    assert!(
        !screens.contains("redeem-overlay"),
        "the redeem overlay must stay OUT of SCREENS for the same reason"
    );
}

/// Z4 — THE CODE IS NEVER PERSISTED UI-SIDE, and closing clears it.
///
/// ⚠ MUST GO RED IF: the code is written to storage, to settings, or to a variable
/// that outlives the modal. The recorded NA-0751 boundary is that the full code
/// appears exactly once, at mint; `InviteDto` carries no `code` field, so the only
/// way it could leak is through this file.
#[test]
fn the_one_time_code_is_never_persisted_by_the_front_end() {
    let js = ui_file("main.js");
    for banned in ["localStorage", "sessionStorage", "indexedDB"] {
        assert!(
            !js.contains(banned),
            "`{banned}` must not appear in main.js — the one-time code has no UI-side store"
        );
    }
    let start = js
        .find("// ---- NA-0755")
        .expect("the invite module exists");
    let module = &js[start..];
    let end = module
        .find("// ---- boot")
        .expect("the module ends before boot");
    let module = &module[..end];
    assert!(
        !module.contains("settings_set"),
        "the invite module must never write settings — nothing about a one-time code is configuration"
    );
    // closeInviteModal empties the node the code lives in.
    let close = module
        .find("function closeInviteModal()")
        .expect("closeInviteModal exists");
    let close_body = &module[close..];
    let close_end = close_body.find("\n}\n").expect("closeInviteModal ends");
    let close_body = &close_body[..close_end];
    // ⚠ NA-0766 (`D-0043`) RE-AIM. The property is unchanged and the pin is STRONGER: the code
    // box is no longer merely emptied, it is RESET to the item-7 empty state. A one-time code
    // cannot survive a close either way, but now the seal also fails if the reset path stops
    // being shared — which is the defect v3 had to fix once already at two call sites.
    assert!(
        close_body.contains("inviteResetSlot();"),
        "closing the modal MUST reset the code slot"
    );
    let reset = module
        .find("function inviteResetSlot()")
        .expect("the one reset path exists");
    let reset_body = &module[reset..];
    let reset_end = reset_body.find("\n}\n").expect("inviteResetSlot ends");
    assert!(
        reset_body[..reset_end].contains("box.textContent = INVITE_SLOT_EMPTY;"),
        "and that reset REPLACES the code with the empty-slot sentence, so it cannot survive"
    );
    assert!(
        close_body.contains("inviteId = null;"),
        "closing the modal MUST forget the displayed invite's id"
    );
}

/// Z3's FRONT-END HALF — the arg shapes this file really emits are the ones
/// `na0751_gateway_surface.rs` already pinned through real IPC.
///
/// ⚠ MUST GO RED IF: a key is renamed on either side. The gateway test drives
/// `{"selfLabel": null, "relay": …, "ttlSecs": …}` and `{"inviteId": …}`; if the
/// front end emitted different keys, that test would still pass while the app
/// broke — which is exactly the gap this seal closes.
#[test]
fn the_invite_calls_emit_the_arg_shapes_the_gateway_test_pins() {
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"invoke("invite_create", {"#)
            && js.contains("selfLabel: null, relay: relayUrl, ttlSecs: INVITE_TTL_SECS,"),
        "invite_create must emit selfLabel/relay/ttlSecs"
    );
    // ⚠ RE-AIMED AT v4. The old needle matched `{ inviteId }` — an expression that existed
    // only in the "Cancel Invite" handler, which v4 removed. The list's call is
    // `{ inviteId: rev }`: the same KEY, a different identifier. The property worth pinning is
    // the key the command receives, so the needle matches that and survives a caller moving.
    assert!(
        js.contains(r#"invoke("invite_revoke", { inviteId: rev })"#),
        "invite_revoke must emit the inviteId key"
    );
    assert!(
        js.contains(r#"invoke("invite_clear", { inviteId: clr })"#),
        "invite_clear must emit the inviteId key"
    );
    assert!(
        js.contains(r#"invoke("invite_list")"#),
        "invite_list takes no args"
    );
    // The TTL is the CLI's own default, adopted rather than invented.
    assert!(
        js.contains("const INVITE_TTL_SECS = 259200;"),
        "the requested TTL is qsc's own `default_value_t` (72h); changing it here without \
         changing that reference makes the two front ends ask for different things"
    );
}

/// R380 §2 — THE `relay_rejected` SENTENCE NAMES BOTH PROVENANCES.
///
/// ⚠ MUST GO RED IF: it is reworded to claim the relay was unreachable. Every
/// non-TLS send failure on the create path returns the caller's own fallback
/// (`relay_send_outcome_from_parts`), so unreachable and refused arrive as the SAME
/// code — measured live at STOP 1. A sentence that picks one is a claim the app
/// has not measured, which is the defect class NA-0754 removed from the relay pane.
#[test]
fn the_create_failure_sentence_does_not_claim_a_cause_it_cannot_know() {
    let js = ui_file("main.js");
    assert!(
        js.contains("The relay didn't create the invite"),
        "the ruled banner"
    );
    assert!(
        js.contains(
            "Nothing was created — the relay couldn't be reached, or it refused the request. \
             Check Settings → Relay; its Test connection button can tell which."
        ),
        "the ruled detail, naming BOTH provenances"
    );
    let start = js
        .find(r#"if (c === "relay_rejected")"#)
        .expect("the arm exists");
    let arm = &js[start..start + 900];
    assert!(
        !arm.contains("Couldn't reach the relay"),
        "the relay-pane's unreachable banner must NOT be reused here: this code cannot \
         distinguish unreachable from refused"
    );
}

/// R380 §5 — THE TWO SECURITY TELLS ARE PREPARED, DISTINCT, AND UNSEALED.
///
/// ⚠ This seal pins that the two arms remain DISTINCT FROM EACH OTHER and from the
/// generic. It does NOT assert either is reachable — both are produced only inside
/// `verify_redeemed_bundle`, on the redeem/accept path, so no Lane A call can
/// render them. A seal on their reachability could not fail, and a seal that
/// cannot fail is not a seal.
#[test]
fn the_two_security_tells_stay_distinct_from_each_other() {
    let js = ui_file("main.js");
    let commitment = js
        .find("This invite's keys don't match")
        .expect("commitment tell");
    let signature = js
        .find("This invite has been altered")
        .expect("signature tell");
    assert_ne!(
        commitment, signature,
        "the two tells must not collapse into one sentence"
    );
    // The const doc's distinction — substituted KEYS vs tampered FIELDS — is what
    // makes two arms worth having; both must say someone may be interfering.
    assert_eq!(
        js.matches("Someone may be interfering.").count(),
        2,
        "both security tells carry the interference statement, and only they do"
    );
}

/// R380 §3 — THE NO-RELAY GATE DISABLES RATHER THAN LETS THE USER FAIL.
///
/// ⚠ MUST GO RED IF: the gate becomes advisory. `relay_config_get()` returns
/// `{relay_url: ""}` on a fresh profile (measured), so with no relay the create
/// cannot succeed; an enabled button whose only outcome is an error is the
/// control-that-cannot-succeed shape.
#[test]
fn create_is_disabled_when_no_relay_is_configured() {
    let js = ui_file("main.js");
    let start = js
        .find("async function inviteRefresh()")
        .expect("the refresh-on-open gate exists");
    let body = &js[start..];
    let end = body.find("\n}\n").expect("open ends");
    let body = &body[..end];
    assert!(
        body.contains(r#"inviteNoRelay = relayUrl === "";"#),
        "the gate reads the empty string"
    );
    // ⚠ v2 gates on BOTH conditions: no relay, and the soft cap. The bank's cap-full state is
    // "Activate disabled with the TRUE message" — the message the operator never saw because
    // v1 routed the cap outcome through the wrong arm.
    assert!(
        body.contains("inviteCapFull = live >= INVITE_SOFT_CAP;"),
        "the cap is part of the gate, not only an error arm"
    );
    // ⚠⚠ NA-0766 (`D-0043`) RE-AIM, NOT A WEAKENING. The decision MOVED but did not soften: the
    // two conditions this seal was born to pin are now two of FOUR causes in a single
    // assignment, and the seal follows them there. It gained the name gate (item 10) and the
    // one-invite-per-window latch (item 12), so this pin is STRICTLY STRONGER than the one it
    // replaces — it now fails if any of four causes is dropped, where it used to fail on two.
    // The Z6 precedent: the equality stays EXACT.
    assert!(
        body.contains("inviteSyncActivate();"),
        "the gate routes through the ONE decision"
    );
    assert!(
        js.contains(
            r#"  byId("btn-invite-activate").disabled = inviteNoRelay || inviteCapFull || !nameOk || inviteMinted;"#
        ),
        "and that decision DISABLES create on all four causes"
    );
    assert!(
        body.contains(r#"invoke("relay_config_get")"#),
        "refresh-on-open: the gate re-reads configuration when the modal opens"
    );
}

/// THE BANK'S "NO POLLING" DECISION, held structurally.
///
/// ⚠ MUST GO RED IF: a timer is added to the invite module. Refresh-on-open is the
/// blessed design; a background check is FILED as a candidate, not built.
#[test]
fn the_invite_module_adds_no_timer() {
    let js = ui_file("main.js");
    let start = js
        .find("// ---- NA-0755")
        .expect("the invite module exists");
    let module = &js[start..];
    let end = module.find("// ---- boot").expect("the module ends");
    let module = &module[..end];
    assert!(
        !module.contains("setInterval"),
        "no polling — the bank's decision 1 is refresh-on-open, with the background check FILED"
    );
    // setTimeout IS used, once, for the transient "Copied" label. That is a
    // one-shot label reset, not a poll of backend state; pinning the count keeps
    // the distinction honest rather than banning the primitive outright.
    // v3 has TWO one-shots and neither is a poll. Pinning the COUNT keeps the distinction
    // honest: a third would have to be justified, and `setInterval` stays banned outright.
    //   1. the copy link's "copied" → "copy code" revert
    //   2. the revoke flip-then-leave pause — the row shows "Revoked" where the user is looking
    //      before it goes, because a row that simply vanished is indistinguishable from a bug
    assert_eq!(
        module.matches("setTimeout").count(),
        2,
        "exactly two one-shot timers — the copy revert and the revoke flip pause; never a poll"
    );
}

/// ⚠ THE CODE BOX WRAPS, AND IT IS NOT THE VERIFICATION CODE'S BOX.
///
/// MUST GO RED IF: `.code-box` gains `nowrap`, or the modal switches to
/// `.verify-code`. An invite code is 133-154 characters; `.verify-code` is
/// `white-space: nowrap; overflow: hidden` with fitCode(), and its own comment
/// records the re-flight where that pair produced a SILENT CLIP. Reusing it here
/// would reproduce NA-0753's defect at twice the string length.
#[test]
fn the_invite_code_box_wraps_and_is_selectable() {
    let css = ui_file("style.css");
    let start = css.find(".code-box {").expect(".code-box exists");
    let rule = &css[start..];
    let end = rule.find('}').expect("the rule closes");
    let rule = &rule[..end];
    assert!(
        rule.contains("overflow-wrap: anywhere"),
        ".code-box must wrap"
    );
    assert!(
        rule.contains("word-break: break-all"),
        ".code-box must break long tokens"
    );
    assert!(
        !rule.contains("nowrap"),
        ".code-box must never be nowrap — that is the silent-clip pair"
    );
    // `body` sets `user-select: none`, so the box must re-enable it or the manual
    // fallback (select the code by hand) is unavailable.
    assert!(
        rule.contains("user-select: text"),
        ".code-box must re-enable selection"
    );
    let html = ui_file("index.html");
    let m = html
        .find(r#"id="invite-overlay""#)
        .expect("the modal exists");
    let modal = &html[m..modal_end(&html, m)];
    // ⚠ THE NEEDLE MATCHES USAGE, NOT MENTION — and it was rebuilt here after the
    // first draft caught its OWN AUTHOR. That draft searched for the bare class
    // name and fired on the modal's explanatory comment, which names the class in
    // order to say it is NOT used: the "documenting a removal re-plants it" hazard,
    // arriving inside the seal written to prevent the defect. A seal must test what
    // the markup DOES, so it matches the attribute, and the comment stays.
    assert!(
        !modal.contains(r#"class="verify-code""#)
            && !modal.contains(r#"class="code-box verify-code""#),
        "the modal must not APPLY .verify-code — naming it in a comment is fine and is why \
         this needle matches the attribute rather than the word"
    );
    // ⚠ v2 gives the minted code a second class (`code-box minted` — the one-time accent
    // border), so the needle matches the class TOKEN inside the attribute rather than the whole
    // attribute value. Matching the whole value would have gone red on a purely additive
    // change, which is a pin that punishes the wrong thing.
    assert!(
        modal.contains(r#"class="code-box"#),
        "the code box carries .code-box"
    );
    // ⚠ NA-0766 (`D-0043`) RE-AIM. Item 6 makes the slot ship PRESENT AND EMPTY, so the markup
    // now carries `code-box empty` and the minted accent is applied at RUNTIME when a code
    // actually lands. The property — a minted code wears its one-time accent class — is pinned
    // where it now lives, in the mint path, rather than deleted.
    assert!(
        modal.contains("code-box empty"),
        "the slot ships in its empty state, present from the moment the window opens"
    );
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"box.classList.add("minted");"#),
        "and a landed code carries its one-time accent class"
    );
    assert!(
        js.contains(r#"box.classList.remove("empty");"#),
        "swapping the empty treatment off in the same act, so the two can never both apply"
    );
}

/// R380 §4, RE-AIMED BY NA-0764 (`D-1405`) — THE LAST TWO REVEALERS RETIRE,
/// BECAUSE THE PANE THEY WERE WAITING FOR NOW EXISTS.
///
/// ⚠⚠ THIS SEAL WENT RED ON LANE C'S FIRST EDIT, AND THAT IS THE SEAL WORKING.
/// Its v1 doc said in its own words: *"The Contacts pane is Lane C and is still
/// unbuilt, so on those two paths the stub is still the truth."* Lane C built
/// the pane, so both rail buttons now open it and the revealer count moves
/// **2 -> 0**. Following the Z6 precedent (NA-0763), the seal is **RE-AIMED to
/// what is measured, never weakened and never deleted**: the count stays an
/// EXACT equality, so an unaccounted revealer reappearing still fails it, and
/// the property that replaces it — both rail buttons reach the real pane — is
/// pinned here rather than left to prose.
///
/// ⚠ THE ELEMENT AND ITS COPY SURVIVE UNTOUCHED. "Retire the revealers" and
/// "delete the message" are different acts, and only the first is this lane's.
/// Nothing else in the app claims contacts are unbuilt.
///
/// ⚠ MUST GO RED IF: a revealer is reintroduced, `#stub-note` is deleted, or
/// either rail button stops opening the Contacts pane.
#[test]
fn the_stub_survives_and_lane_c_retired_its_last_revealer() {
    let js = ui_file("main.js");
    let html = ui_file("index.html");
    // ⚠ NA-0756 (D-0037, R387 §S4) — RETARGETED, and the pin moves WITH the change rather
    // than being deleted. Item 1 orders both contact-making entries onto the chooser, so the
    // welcome button now opens THAT; the property this seal exists for is unchanged — the
    // button reaches a real flow and not the stub — and the load-bearing half below (exactly
    // TWO stub revealers survive for Lane C) is untouched.
    assert!(
        js.contains(
            r#"byId("btn-add-contact").addEventListener("click", () => openRedeemChooser());"#
        ),
        "the welcome button opens the real flow — the chooser, since NA-0756"
    );
    // The element and BOTH Lane-C revealers are still here.
    assert!(
        html.contains(r#"id="stub-note""#),
        "the stub element survives for Lane C"
    );
    assert_eq!(
        js.matches(r#"byId("stub-note").classList.remove("hidden");"#)
            .count(),
        0,
        "ZERO revealers remain — Lane C built the pane both rail buttons were waiting for. \
         An EXACT equality, deliberately: a revealer reappearing must fail this, not be \
         absorbed by a >= comparison"
    );
    // The property that REPLACES the retired one: both rail buttons reach the
    // real surface. Pinned here so the retirement above cannot be satisfied by
    // simply deleting the handlers.
    assert!(
        js.contains(
            r#"byId("btn-rail-contacts").addEventListener("click", () => showContactsPane());"#
        ),
        "the main rail's Contacts button opens the pane"
    );
    assert!(
        js.contains(r#"byId("btn-rail-contacts-s").addEventListener("click", async () => {"#),
        "the settings rail's Contacts button returns to main and opens the pane"
    );
    // And the stub's own copy is untouched — retiring a path is not deleting a message.
    assert!(
        html.contains("Adding contacts arrives in a future update."),
        "the stub sentence survives verbatim"
    );
}

/// SEVERITY: THE MODAL IS ACCENT, NEVER RED.
///
/// ⚠ MUST GO RED IF: a danger banner is introduced. Red is reserved for the
/// armed-erasure state (setBanner's own note at main.js:152), and qsc's const doc
/// says the same of the invite arms: "Severity is accent, never red".
#[test]
fn the_invite_modal_never_renders_danger_severity() {
    let js = ui_file("main.js");
    let start = js
        .find("// ---- NA-0755")
        .expect("the invite module exists");
    let module = &js[start..];
    let end = module.find("// ---- boot").expect("the module ends");
    let module = &module[..end];
    assert!(
        !module.contains(r#""danger""#),
        "the invite modal must never use the danger tier — red is the vault-loss reservation"
    );
    assert!(
        module.contains(r#"setBanner(box.querySelector(".status-banner"), "accent""#),
        "the modal renders through the shipped banner helper at accent severity"
    );
    let html = ui_file("index.html");
    let m = html
        .find(r#"id="invite-overlay""#)
        .expect("the modal exists");
    let modal = &html[m..modal_end(&html, m)];
    assert!(
        !modal.contains("danger"),
        "no danger chrome in the modal markup"
    );
}

fn modal_end(html: &str, from: usize) -> usize {
    let tail = &html[from..];
    from + tail.find("<script src=").unwrap_or(tail.len())
}

// ===========================================================================
// NA-0755 v2 — THE SINGLE-VIEW MINT. Seals for the reshape, and for the one
// mechanism the whole design rests on.
// ===========================================================================

/// ⛳⛳ THE CLIPBOARD MECHANISM — the measurement that saved the design, held in place.
///
/// The v2 bank assumed a "~4 s user-activation timeout" and specified a fallback for it.
/// MEASURED in this webview: a plain `await` then `writeText` RESOLVES at 750 ms and
/// **REJECTS at 1000 ms**. A create needs two network round-trips, so on that route
/// "Activate & Copy" would have failed EVERY time. `ClipboardItem` built SYNCHRONOUSLY around a
/// pending promise **RESOLVED at 4000 ms** — that is why the single gesture works.
///
/// ⚠ MUST GO RED IF: the item stops being constructed inside the handler, or the code is
/// awaited BEFORE the clipboard write. Either change silently reintroduces the failure the
/// measurement found, and it would look correct in review.
///
/// ⚠ CLAIM BOUNDARY, carried verbatim from the measurement: WebKitGTK under X11 on the build
/// box. macOS and Windows are unmeasured; the fallback below is what covers them.
#[test]
fn the_single_gesture_copy_builds_its_clipboard_item_before_awaiting_the_create() {
    let js = ui_file("main.js");
    let start = js
        .find(r#"byId("btn-invite-activate").addEventListener"#)
        .expect("the activate handler exists");
    let body = &js[start..];
    let end = body.find("\n});\n").expect("the handler ends");
    let body = &body[..end];

    let mint = body
        .find("const mint = invoke(\"invite_create\"")
        .expect("the create is started");
    let item = body
        .find("new ClipboardItem({")
        .expect("the item is constructed");
    let write = body
        .find("navigator.clipboard.write([item])")
        .expect("the write happens");
    let awaited = body
        .find("code = await mint;")
        .expect("the code is awaited");

    assert!(
        mint < item,
        "the create promise must exist BEFORE the item wraps it"
    );
    assert!(item < write, "the item is built before the write");
    assert!(
        write < awaited,
        "⚠ THE WRITE MUST PRECEDE `await mint`. Awaiting the code first spends the user \
         activation — measured to expire between 750ms and 1000ms — and the write then rejects."
    );
    assert!(
        body.contains("mint.then((code) => new Blob([code]"),
        "the item's payload is the PENDING promise, not a resolved string"
    );
}

/// The fallback is a CAPABILITY TEST, never a timeout guess.
///
/// ⚠ MUST GO RED IF: it becomes a timer, a try/catch on duration, or a platform sniff. The bank's
/// rule is that the label must never promise what the platform refuses — so the label is derived
/// from the capability, and both branches are pinned.
#[test]
fn the_clipboard_fallback_is_a_capability_test_and_relabels_the_button() {
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"const HAS_CLIPBOARD_ITEM = typeof ClipboardItem !== "undefined""#),
        "the fallback is decided by capability"
    );
    assert!(
        js.contains(r#"HAS_CLIPBOARD_ITEM ? "Activate & Copy" : "Activate""#),
        "the button must not promise a copy the platform will refuse"
    );
    // v3: the in-box icon is gone; ONE text link below the box is both the re-copy control
    // and the fallback's recovery path, so the recovery line names THAT.
    assert!(
        js.contains(r#""Copy didn't complete — use the copy code link below.""#),
        "the recovery line names the text link that is actually there"
    );
    assert!(
        !js.contains("use the copy icon"),
        "the retired icon must not survive in copy the user reads"
    );
    assert!(
        !js.contains("setTimeout(() => navigator.clipboard"),
        "the fallback must never be a timeout guess"
    );
}

/// B-1 — THE WORD "SAFE" DOES NOT SHIP, and the chip says what is true.
///
/// ⚠ MUST GO RED IF: "safe to clear" reappears. `Creating` does not mean the relay never
/// confirmed — the relay may hold the slot, un-revocably, because the token was dropped
/// unpersisted. Calling that "safe" is a claim about the relay that nothing established.
#[test]
fn the_failed_row_never_calls_itself_safe() {
    let js = ui_file("main.js");
    // v3 splits what v2 crammed into one chip: the CHIP states the fact, and the row's own
    // muted line carries the honesty about the relay. Both are pinned.
    assert!(
        js.contains(r#"chip.textContent = "Didn't finish";"#),
        "the v3 chip states the fact"
    );
    assert!(
        js.contains("didn't finish — if the relay registered it, that slot expires on its own and can't be revoked from here"),
        "the row line carries the honesty about the relay"
    );
    for banned in ["safe to clear", "Safe to clear", "safe to remove"] {
        assert!(
            !js.contains(banned),
            "the word `safe` must not ship: `{banned}`"
        );
    }
}

/// The FE emits `recipientLabel` LAST, which is what the gateway test's arg-shape pin expects.
///
/// ⚠ MUST GO RED IF: the key is inserted between `selfLabel` and `relay`. That ordering is not
/// cosmetic — `na0751_gateway_surface.rs` pins the emitted shape, and a same-typed neighbour to
/// `self_label` is the transposition hazard the SR-15 read measured as failing open.
#[test]
fn the_front_end_emits_the_recipient_label_last() {
    let js = ui_file("main.js");
    let i = js
        .find("invoke(\"invite_create\"")
        .expect("the create call exists");
    let call = &js[i..i + 260];
    let self_l = call.find("selfLabel").expect("selfLabel present");
    let relay = call.find("relay:").expect("relay present");
    let ttl = call.find("ttlSecs").expect("ttlSecs present");
    let recip = call.find("recipientLabel").expect("recipientLabel present");
    assert!(
        self_l < relay && relay < ttl && ttl < recip,
        "recipientLabel must be LAST — never adjacent to selfLabel: {call}"
    );
}

/// R381 §1 — the self-diagnosing vault arm renders its provenance as a SUBDUED SUFFIX.
///
/// ⚠ MUST GO RED IF: the suffix is dropped, or the sentence starts telling the user to unlock.
/// `vault_unavailable` carries THREE provenances — locked mid-operation, vault DAMAGE, or a
/// key-source failure — so "unlock it" is wrong in two of them.
#[test]
fn the_vault_arm_carries_its_source_code_and_never_says_unlock_it() {
    let js = ui_file("main.js");
    let i = js
        .find(r#"if (c === "vault_unavailable")"#)
        .expect("the arm exists");
    let arm = &js[i..i + 900];
    assert!(
        arm.contains("const suffix = d ? ` (${d})` : \"\";"),
        "the payload rides as a suffix"
    );
    assert!(
        arm.contains("The vault couldn't be read."),
        "the three-provenance sentence"
    );
    // ⚠⚠ THE NEEDLE MATCHES THE RENDERED COPY, NOT THE ARM — and it was rebuilt here after
    // catching its OWN AUTHOR. The first draft searched the whole arm for "unlock it" and fired
    // on the COMMENT that says the copy must not contain it: the "documenting a removal
    // re-plants it" hazard, arriving inside the seal written to prevent the defect — for the
    // second time in this lane. A seal must test what SHIPS, so it reads the `detail:` string.
    let detail_start = arm.find("detail: `").expect("the rendered detail exists");
    let detail = &arm[detail_start..arm[detail_start..].find("` };").unwrap() + detail_start];
    assert!(
        !detail.contains("unlock"),
        "the rendered copy must not tell the user to unlock — that is true in only one of the \
         three provenances this code carries: {detail}"
    );
}

/// ⚠⚠ v3 — THE LABEL-CLEAR TRIPWIRE. The operator's flight found that one typed label SILENTLY
/// RODE EVERY LATER MINT: the field was cleared when the surface opened but NOT when the user
/// came back to it from the list, so the second invite inherited the first invite's note
/// without anyone touching the box.
///
/// ⚠ WHY A SOURCE SEAL AND NOT ONLY A DRIVEN ONE: the harness drives the field too (scenario
/// `f_k`), but the defect was a MISSING CALL on one of two paths, and the durable property is
/// that BOTH paths go through the one function that owns "entering the mint fresh". Two call
/// sites that must each remember to clear is how this happened in the first place.
///
/// ⚠ MUST GO RED IF: either entry path stops calling `inviteEnterMintFresh`, or that function
/// stops clearing the field.
#[test]
fn entering_the_mint_fresh_always_clears_the_recipient_label() {
    let js = ui_file("main.js");

    // The one owner exists and actually clears.
    let f = js
        .find("function inviteEnterMintFresh() {")
        .expect("the single owner of `entering the mint fresh` exists");
    let body = &js[f..];
    let end = body.find("\n}\n").expect("it ends");
    let body = &body[..end];
    assert!(
        body.contains(r#"byId("invite-label").value = "";"#),
        "entering the mint fresh MUST clear the recipient label"
    );
    // ⚠ NA-0766 (`D-0043`) RE-AIM: the clear routes through the ONE reset path (item 7), so the
    // markup's empty sentence and the code's reset cannot drift into two different states.
    assert!(
        body.contains("inviteResetSlot();"),
        "and the one-time code, which must never survive into a new mint"
    );
    assert!(
        body.contains("byId(\"invite-label\").readOnly = false;"),
        "ITEM 12: and the field is UNLOCKED again, or a fresh mint would open read-only"
    );

    // BOTH entry paths route through it — the fix is structural, not a habit at two call sites.
    let open = js
        .find("async function openInviteModal() {")
        .expect("open exists");
    let open_body = &js[open..open + 400];
    assert!(
        open_body.contains("inviteEnterMintFresh();"),
        "opening the surface enters the mint fresh"
    );
    // ⚠⚠ NA-0766 (`D-0043`) RE-AIM, AND THE COUNT OF ENTRY PATHS DROPPED FROM TWO TO ONE.
    // This seal was born because a typed label SILENTLY RODE EVERY LATER MINT: the field was
    // cleared when the surface opened but NOT when the user came back to the mint from the list,
    // so a second invite inherited the first one's note. That second path was the fresh-mint
    // button in the list head, and item 14 RETIRES IT — the only way back to a fresh mint is now
    // Close then "+", which is `openInviteModal` and therefore the path already pinned above.
    // ⚠ THE DEFECT'S PREMISE STOPS BEING TRUE EXACTLY WHEN ITS CURE IS REMOVED, which is the
    // NA-0765 `D-0036` shape. So the seal is re-aimed to the STRUCTURAL fact that makes the
    // regression unreachable — exactly ONE caller of the fresh-entry function — rather than
    // deleted, and it still fails if a second entry path is ever added without routing through it.
    assert_eq!(
        js.matches("inviteEnterMintFresh();").count(),
        1,
        "exactly ONE call site enters the mint fresh, so the two paths cannot drift apart again"
    );
}

// ===========================================================================
// NA-0755 v4 — THE POLISH PASS. Ratified across five mockup rounds.
// ===========================================================================

/// ⚠⚠ ONE SOURCE, TWO DISPLAYS — the property, not just the strings.
///
/// The meta row's expiry and the warning's closing figure are two renderings of ONE fact: this
/// invite's remaining life. A warning reading "3 days" above a code the meta row says expires
/// in 2 is a lie the user has no way to resolve, and nothing on screen would contradict it.
///
/// ⚠ MUST GO RED IF: either display is fed from anywhere but the single computed value — a
/// literal, a second calculation, or the requested TTL (which the relay clamps).
///
/// The seal reads the one writer and asserts BOTH displays are written inside it, from the same
/// parameter. Its control mutates one display's source and is shown RED.
#[test]
fn the_expiry_figure_has_one_source_and_two_displays() {
    let js = ui_file("main.js");
    let f = js
        .find("function inviteWriteExpiry(secondsLeft) {")
        .expect("the single writer of the expiry figure exists");
    let body = &js[f..];
    let end = body.find("\n}\n").expect("it ends");
    let body = &body[..end];

    assert!(
        body.contains("const human = secondsLeft > 0 ? humanDuration(secondsLeft) : \"—\";"),
        "ONE value is computed, once, from the invite's own remaining seconds"
    );
    assert!(
        body.contains(r#"byId("invite-meta-expiry").textContent"#),
        "display 1 — the meta row — is written here"
    );
    assert!(
        body.contains(r#"byId("invite-warn-days").textContent = human;"#),
        "display 2 — the warning's figure — is written HERE, from the SAME `human` value"
    );
    // And nowhere else writes the warning figure, or the single-source property is a fiction.
    assert_eq!(
        js.matches(r#"byId("invite-warn-days")"#).count(),
        1,
        "exactly ONE writer of the warning figure — a second would let the two displays diverge"
    );
    // The figure is READ BACK from the invite, never printed from the TTL we asked for.
    assert!(
        !body.contains("INVITE_TTL_SECS"),
        "the displayed life must come from the invite, not from the TTL the relay clamps"
    );
}

/// v4 — "Cancel Invite" IS GONE, and Revoke in the list is the single kill.
///
/// ⚠ MUST GO RED IF: it returns. Two kill controls in two places is the ambiguity this removal
/// exists to end — one word, one place. Mid-mint regret is Review invites → Revoke: one extra
/// click for a rare case, and that trade was chosen rather than lost.
#[test]
fn the_cancel_invite_control_is_gone_and_revoke_is_the_single_kill() {
    let html = ui_file("index.html");
    let js = ui_file("main.js");
    // ⚠⚠ THE NEEDLES MATCH WHAT SHIPS, NOT THE BARE WORDS — and they were rebuilt after the
    // first draft caught ITS OWN AUTHOR for the SIXTH time in this lane: the comments that
    // RECORD the removal name the control, so a bare-word needle fires on the explanation.
    // A label ships as a TEXT NODE (`>Cancel Invite<`) and a handler ships as a registration;
    // neither form can be produced by a comment. Same cure as `.verify-code`, the vault arm and
    // the tier scanner: test the shipped form.
    assert!(
        !html.contains(r#"id="btn-invite-cancel""#),
        "the Cancel control is gone from the markup"
    );
    assert!(
        !html.contains(">Cancel Invite<"),
        "and so is its rendered label"
    );
    assert!(
        !js.contains(r#"byId("btn-invite-cancel")"#),
        "and its handler went with it"
    );
    // The kill that remains is the list's, and it is still there.
    assert!(
        js.contains(r#"invoke("invite_revoke", { inviteId: rev })"#),
        "Revoke in the list is the single kill mechanism"
    );
}

/// v4 — the copy link is PLAIN and its acknowledgement is GREEN.
///
/// ⚠ MUST GO RED IF: the underline returns, or "copied" loses its own colour. The link reuses
/// the shipped `a.rm` text-link style and modifies ONLY the underline — re-minting the whole
/// look would have been an invented control.
#[test]
fn the_copy_link_is_plain_and_its_acknowledgement_is_green() {
    let css = ui_file("style.css");
    let js = ui_file("main.js");
    assert!(
        css.contains("a.rm.plain { text-decoration: none; }"),
        "plain: never underlined"
    );
    assert!(
        css.contains("a.rm.plain.copied { color: var(--ok); }"),
        "the acknowledgement is green"
    );
    assert!(
        js.contains(r#"link.classList.add("copied");"#),
        "the class is applied on success"
    );
    assert!(
        js.contains(r#"link.classList.remove("copied");"#),
        "and removed when it reverts"
    );
    let html = ui_file("index.html");
    assert!(
        html.contains(r#"class="rm plain""#),
        "the markup carries both classes"
    );
}

/// v4 — the structural pins the harness can read: the widened surface and the one-line button.
///
/// ⚠ MUST GO RED IF: the width narrows back or the button loses `nowrap`. At the old width the
/// New-invite label WRAPPED. The width fixes the symptom; `nowrap` fixes the property, so a
/// future narrowing cannot quietly reintroduce it.
#[test]
fn the_surface_widened_and_the_new_invite_button_is_one_line() {
    let css = ui_file("style.css");
    let html = ui_file("index.html");
    assert!(
        css.contains("max-width: 500px;"),
        "the ratified width, expressed once"
    );
    // ⚠ v5 RE-AIM: 580 → the 500 class, against the rendered mockups. Expressed exactly ONCE
    // in the stylesheet, so all three states share it — three states that could drift apart
    // into different widths is what the uniformity rule exists to prevent.
    assert_eq!(
        css.matches("max-width: 500px;").count(),
        1,
        "ONE expression of the surface width, shared by pre-mint, post-mint and the list"
    );
    // ⚠ ITS NAME IS NOW STALE AND IS DELIBERATELY LEFT SO. The test is not renamed, because a
    // rename shows in `EXPECTED_TEST_INVENTORY.txt` as a REMOVAL plus an ADDITION, and a pin whose
    // whole job is to catch a test disappearing should not be handed a false positive by a
    // cosmetic edit. The repo already carries this precedent (`na0700_ipc_replay`'s
    // `all_27_registered_commands_...`, whose set is 42): a stale NAME is REPORTED, not quietly
    // fixed mid-lane. Reported in this lane's records.
    // ⚠⚠ NA-0766 (`D-0043`) RE-AIM. Item 14 retires the list head's fresh-mint button, and its
    // one-line rule retires WITH it — a rule whose only element is gone is DEAD CSS, the very
    // artifact NA-0765 used to prove a blessed row had been styled but never built. The half of
    // this seal that is about WIDTH is untouched above and still EXACT; the half that pinned a
    // control this lane deleted is re-aimed to assert the retirement is COMPLETE on both sides.
    let nowrap_rule = format!("button.{} {{ white-space: nowrap; }}", "nowrap");
    assert!(
        !css.contains(nowrap_rule.as_str()),
        "the retired button's rule went with the button, leaving no dead CSS behind"
    );
    let retired = format!("id=\"btn-invite-{}\"", "back");
    assert!(
        !html.contains(retired.as_str()),
        "and the button itself is gone from the list head — Close only"
    );
}

/// v4 — the meta row sits ABOVE the code, and names what the code is.
///
/// ⚠ MUST GO RED IF: the row moves below the box again, or stops naming the note. The left
/// cell answers "what is this?" before the user reads 150 characters of base64.
#[test]
fn the_meta_row_sits_above_the_code_and_names_it() {
    let html = ui_file("index.html");
    let island = html.find("code-island").expect("the island exists");
    let meta = html[island..]
        .find("invite-meta-note")
        .expect("the meta row is inside it");
    let box_ = html[island..]
        .find("invite-code")
        .expect("the code box is inside it");
    assert!(meta < box_, "the meta row must render ABOVE the code box");
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"row.label ? "Invite for: " + row.label : "Invite code""#),
        "the left cell names the note when there is one, else says plainly what it is"
    );
}

/// v5 — REVOKE IS PLAIN RED, ON THE SHIPPED DANGER-LINK TOKEN.
///
/// ⚠ MUST GO RED IF: the underline returns, or the colour is repainted to the link blue, or a
/// literal hex replaces the token. Revoke destroys something at the relay; Remove does not, and
/// the two must not look alike.
///
/// ⚠ The token choice is `--danger-link`, NOT `--danger-text`: the former was minted for this
/// exact shape (E.6, the "Delete vault?" link) and carries its own hover target. Reusing the
/// pairing is what keeps the two danger surfaces consistent.
#[test]
fn revoke_is_plain_red_on_the_shipped_danger_link_token() {
    let css = ui_file("style.css");
    let js = ui_file("main.js");
    assert!(
        css.contains("a.rm.plain.link-danger { color: var(--danger-link); }"),
        "the colour comes from the shipped token, never a literal"
    );
    assert!(
        css.contains("a.rm.plain.link-danger:hover { color: var(--danger-text); }"),
        "the hover target is E.6's own pairing, reused rather than re-decided"
    );
    // No hex may appear on this control — tokens are the colour authority.
    let rule_start = css
        .find("a.rm.plain.link-danger {")
        .expect("the rule exists");
    // ⚠ CLAMPED: this rule is the last block in the stylesheet, so an unclamped end index
    // runs past EOF and panics instead of asserting.
    let rule = &css[rule_start..(rule_start + 120).min(css.len())];
    assert!(!rule.contains('#'), "no hex on the danger link: {rule}");
    // The markup carries plain (no underline) AND the danger class.
    assert!(
        js.contains(r#"b.className = "rm plain link-danger";"#),
        "Revoke is plain AND danger-coloured"
    );
    // ⚠ And Remove is NOT: it clears a local row that can never become actionable.
    // ⚠ Remove stays NEUTRAL. The needle reads the Remove branch's OWN assignment rather than
    // searching the file, so it cannot be satisfied by the Revoke branch above it.
    let rm = js
        .find(r#"b.textContent = "Remove";"#)
        .expect("the Remove control exists");
    let rm_assign = js[..rm]
        .rfind("b.className =")
        .expect("its class assignment");
    assert_eq!(
        &js[rm_assign..rm_assign + 25],
        r#"b.className = "rm plain";"#,
        "Remove stays neutral — painting it red would claim it destroys something at the relay"
    );
}

/// v5 — THE CHIP AND ITS ACTION SHARE ONE VISUAL ROW.
///
/// ⚠ MUST GO RED IF: they stack again. The v3 build rendered them in a column, diverging from
/// the v3 bank's own reference markup — a divergence no seal could see, because nothing
/// measured the rendered geometry. This one does, in the harness; the source side is pinned
/// here so the two halves cannot drift apart.
#[test]
fn the_chip_and_its_action_are_laid_out_on_one_row() {
    let css = ui_file("style.css");
    let start = css
        .find(".invite-row-side {")
        .expect("the cluster rule exists");
    // ⚠ CLAMPED for the same reason the danger rule above is: an unclamped end index panics
    // instead of asserting the moment this rule becomes the last block in the stylesheet.
    let rule = &css[start..(start + 200).min(css.len())];
    assert!(
        rule.contains("flex-direction: row;"),
        "one visual row, not a column: {rule}"
    );
    assert!(
        rule.contains("align-items: center;"),
        "vertically centred against the text block"
    );
    assert!(rule.contains("justify-content: flex-end;"), "right-aligned");
    // And the row itself centres its two blocks against each other.
    let row = css.find(".invite-row {").expect("the row rule exists");
    assert!(
        css[row..(row + 220).min(css.len())].contains("align-items: center;"),
        "the row centres the text block and the cluster against each other"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// NA-0756 (D-0037, R387) — INVITE LANE B: THE REDEEM FLOW
//
// These are SOURCE-side seals. The rendered half is driven in `f_l_invite_redeem` against
// the real webview; what lives here is what a running app cannot show — the shipped BYTES
// of blessed copy, the ABSENCE of controls that must never exist, and the places where a
// future edit could quietly undo a ruling without changing anything visible.
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Z1 — THE BLESSED COPY, PINNED IN ITS SHIPPED SOURCE FORM.
///
/// ⚠ MUST GO RED IF: one byte of the operator's authored text changes. The rendered pins in
/// the scenario prove what the user SEES; this proves what SHIPS, and they are different
/// checks — a renderer that computed the sentence would pass the first and fail this.
///
/// ⚠ THE ENTITY FORM IS PART OF THE PIN, and it is the MEASURED local one. `R387` §S5 voided
/// the brief's numeric-decimal sentence; Lane A's own block uses the named forms and this
/// block matches it.
///
/// ⚠⚠ THE NEGATIVE PIN BELOW READS THE WHOLE FILE, NOT THE MODAL SLICE, AND THAT WIDENING IS
/// ITSELF A FINDING. On its first draft the pin sliced from the overlay's id and PASSED while
/// the file carried the numeric forms nine lines above it, in a comment that spelled them out
/// in order to say they were absent — the plant hazard NA-0754 recorded three times, firing
/// on its fourth. A source-text scanner cannot tell a comment from markup (`ENG-0235`), so
/// the only durable cure is to DESCRIBE a retired construct and never spell it, and the only
/// honest scope for the pin is the entire shipped file.
/// tree. Lane A's own block uses `&mdash;`/`&hellip;`/`&rsquo;` and this block matches it.
#[test]
fn na0756_the_redeem_copy_ships_in_its_blessed_form() {
    let html = ui_file("index.html");
    let start = html
        .find(r#"id="redeem-overlay""#)
        .expect("the redeem overlay exists");
    let modal = &html[start..];

    // ⚠ v2 — THE OPERATOR'S OWN CHOOSER SENTENCE, RE-AUTHORED AND TRIMMED TO TWO BY HIS OWN
    // ORDER. The v1 pin held the v1 wording; the bank's v2 copy section supersedes it and the
    // pin MOVES with the blessing rather than being deleted, so the sentence stays pinned
    // across the change instead of going unpinned for a release.
    assert!(
        modal.contains("Invitations are how contacts are added. One person creates an invite code and then shares it with the person they want to add."),
        "the chooser's blessed v2 sentence ships verbatim"
    );
    // ⚠⚠ v2 — EACH ROW IS PINNED PER LINE. A two-line row pinned as one string would stay green
    // while either line drifted, and the subtitle is the half that carries the explanation the
    // whole pass exists to add. Four pins where v1 had two: the pin is STRENGTHENED, not moved.
    for needle in [
        r#"<span class="choose-row-title">Invite someone</span>"#,
        r#"<span class="choose-row-sub">Create a one-time code to send to a person you trust</span>"#,
        r#"<span class="choose-row-title">I have a code</span>"#,
        r#"<span class="choose-row-sub">Enter an invite code someone sent you</span>"#,
    ] {
        assert!(
            modal.contains(needle),
            "the blessed v2 row copy ships verbatim: `{needle}`"
        );
    }
    // The way out of the surface, which v1 did not offer at all.
    assert!(
        modal.contains(r#"<button id="btn-choose-close" class="secondary full">Close</button>"#),
        "the chooser's Close ships in the Lane A full-width secondary idiom"
    );
    // State 1: the intro, the name field's caption, and the standing hint.
    assert!(
        modal.contains("Paste the invite code they sent you, and choose what to call them. You&rsquo;ll connect through their relay."),
        "state 1's intro ships verbatim, in the house entity form"
    );
    // ⚠ RE-AIMED at NA-0765 (`D-0042`, R-4 row 1): the operator picked the SHORT form,
    // "Their name", for every place this label appears. The pin is STRENGTHENED in the
    // same move — it was a bare phrase and is now the whole element, so the same two
    // words appearing in some unrelated node cannot satisfy it. Its can-fail proof is
    // the lane's control C-9, which restores the retired parenthetical and observes red.
    assert!(
        modal.contains(r#"<span class="field-label" id="redeem-name-caption">Their name</span>"#),
        "the name caption ships verbatim, as its whole element"
    );
    // ⚠ RE-AIMED at NA-0765 (`D-0042`, R-4 row 2): the operator picked the FULLEST truth
    // — the sentence the Settings pane already shipped — as the single wording everywhere.
    // C1 had to CURE the inconsistency, not relocate it, so this asserts the count across
    // the whole file: the first-run pane, Settings, and the code-entry hint. An exact
    // equality, so a fourth site drifting in fails here rather than passing quietly.
    assert_eq!(
        html.matches("Stored only on this device — never sent anywhere.")
            .count(),
        3,
        "one sentence, byte-identical, in all three places the app says it"
    );
    assert!(
        modal.contains("Stored only on this device — never sent anywhere."),
        "the standing hint ships verbatim"
    );
    // State 2: the body and the accent callout.
    assert!(
        modal.contains(
            "When they approve you on their device, the connection completes automatically."
        ),
        "state 2's body ships verbatim"
    );
    assert!(
        modal.contains("Until you both verify by comparing codes,"),
        "state 2's callout ships verbatim"
    );
    // The security-failure state's title and callout.
    assert!(
        modal.contains("<h2>Couldn&rsquo;t make a secure connection</h2>"),
        "the failure title ships verbatim, apostrophe in the house entity form"
    );
    assert!(
        modal.contains("This code failed its security checks &mdash; that can mean a bug, or that someone tampered with the invite. Nothing was set up, and this code can no longer be used. Reach them another way and ask for a fresh invite."),
        "the operator's blessed failure callout ships VERBATIM — R387 §S2a composed rather \
         than replaced it, so this text is untouched"
    );
    // ⚠ The numeric entity forms must appear NOWHERE IN THE FILE — not in the markup, and
    // not in a comment describing them either. The needle is BUILT rather than written as a
    // literal, so this test file cannot itself become the thing that plants them.
    for n in ["8212", "8230", "8217"] {
        let form = format!("&#{n};");
        assert!(
            !html.contains(form.as_str()),
            "the numeric entity forms are VOID per R387 §S5, and they must not be SPELLED even \
             to say they are absent: a source-text scanner cannot tell a comment from markup \
             (ENG-0235), which is exactly how this pin was defeated on its first draft"
        );
    }
}

/// Z4 — NO RETRY EXISTS IN THE SECURITY-FAILURE STATE, PINNED AS AN ABSENCE IN THE SOURCE.
///
/// ⚠ MUST GO RED IF: anyone adds one. This is not caution — it is FACT. The capability burns
/// at `invite/mod.rs:1081`, the instant the relay answers, and the verification that produces
/// these two arms runs at `:1101`, AFTER. A retry would return `already_redeemed`, so a Retry
/// button here would be a control whose only possible outcome is a second, more confusing
/// error.
#[test]
fn na0756_the_security_failure_state_offers_no_retry() {
    let html = ui_file("index.html");
    let start = html
        .find(r#"id="redeem-failed""#)
        .expect("the state exists");
    let state = &html[start..];
    let end = state.find("</div>\n    </div>").unwrap_or(state.len());
    let state = &state[..end];
    let lowered = state.to_lowercase();
    assert!(
        !lowered.contains("retry") && !lowered.contains("try again"),
        "NO Retry control may exist in the security-failure state — the code is already burned"
    );
    // The two buttons that DO exist are pinned positively, so the absence above cannot be
    // satisfied by an empty state.
    assert!(
        state.contains(r#"id="btn-redeem-copydetails""#)
            && state.contains(r#"id="btn-redeem-close2""#),
        "the ruled pair — Copy details / Close — is present, so the Retry absence is measured \
         against a state that really has controls"
    );
}

/// Z3 — THE ADMISSIBILITY GATE MIRRORS THE ENGINE'S OWN SET, EXACTLY.
///
/// ⚠ MUST GO RED IF: the JS predicate drifts from `channel_label_ok`
/// (qsc `lib.rs:2568-2573`: non-empty AND every char in `[A-Za-z0-9_#-]`). A drift in the
/// PERMISSIVE direction is the dangerous one — it re-opens the burn-before-validation defect
/// this gate exists to defend against, and it would do so silently, because the failure only
/// appears after a real invite is destroyed.
///
/// ⚠ The engine gap itself is ENG-0236, filed and NOT patched: the front end being careful
/// does not make the engine safe, and any other caller of `invite_redeem` still loses the
/// invite on a space.
#[test]
fn na0756_the_name_gate_mirrors_the_engine_predicate() {
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"const REDEEM_NAME_RE = /^[A-Za-z0-9_#-]+$/;"#),
        "the JS predicate must be the engine's set character-for-character"
    );
    // The gate is consulted in BOTH places that can commit: the enable/disable path AND the
    // handler. A gate that lives only in the first is one keyboard event from being bypassed.
    assert!(
        js.contains("byId(\"btn-redeem-connect\").disabled = !(code !== \"\" && nameOk);"),
        "the button arms only on code non-empty AND an admissible name"
    );
    assert!(
        js.contains(r#"if (code === "" || !redeemNameOk(name)) { redeemSyncConnect(); return; }"#),
        "and the commit handler refuses independently of the button's state"
    );
}

/// Z5 — THE RULED COPY SET IS PRESENT, AND THE REDEEM PATH CANNOT SPEAK THE CREATE VERB.
///
/// ⚠ MUST GO RED IF: a redeem failure falls through to "Couldn't create the invite", or to a
/// detail sentence that says something false on this path. Measured at STOP 002: 21 of the 35
/// redeem-reachable wire codes had NO copy and rendered a raw engine token, and THREE shipped
/// rows stated something false on a redeem.
#[test]
fn na0756_the_redeem_copy_set_is_ruled_and_verb_true() {
    let js = ui_file("main.js");
    // The six arms that had no copy at all (R387 §S2b).
    for (code, banner) in [
        ("malformed", "That code isn't readable"),
        ("expired", "This invite has expired"),
        ("expired_at_relay", "This invite has expired"),
        ("already_used", "This code has already been used"),
        ("already_redeemed", "You've already used this code"),
        ("revoked", "This invite was cancelled"),
    ] {
        assert!(
            js.contains(&format!("c === \"{code}\"")),
            "the `{code}` arm must be named"
        );
        assert!(
            js.contains(banner),
            "the ruled banner for `{code}` must ship verbatim"
        );
    }
    // The residual's verb switch gained a redeem arm (R387 §S2c).
    assert!(
        js.contains(r#": verb === "redeem" ? "Couldn't add the contact""#),
        "the residual must name the REDEEM verb, not fall through to the create string"
    );
    // ⚠ And create's ruled copy is UNTOUCHED — the rewording is verb-CONDITIONAL, which is
    // the whole reason it is safe. If this disappears, the redeem fix has been applied
    // globally and Lane A's ruled copy has been silently overwritten.
    assert!(
        js.contains(r#"banner: "The relay didn't create the invite""#),
        "create's ruled relay_rejected copy must survive untouched"
    );
    assert!(
        js.contains(r#"detail: "Nothing was added — their relay couldn't be reached, or it refused the request. Check the code is complete, and try again in a moment." };"#),
        "and the redeem path must have its OWN, verb-true sentence"
    );
    // The ENG-0228 both-positions guard is REUSED, not re-derived: the same string is a
    // `code` for one input and a `detail` for another, and position is all that separates them.
    assert_eq!(
        js.matches("INVITE_ENDPOINT_DETAILS.includes(c) || (c === \"other\" && INVITE_ENDPOINT_DETAILS.includes(d))")
            .count(),
        2,
        "both the create and redeem endpoint branches use the SAME both-positions guard"
    );
}

/// Z6 — THE TRIGGERS COMPARE BY EQUALITY, AND THE TIMER CENSUS IS EXACT.
///
/// ⚠ MUST GO RED IF: a `contains`/`includes` creeps into the state comparison, a trigger
/// stops feeding the one handler, or a timer appears or vanishes without this count being
/// updated deliberately. `connect_status.state` is a CLOSED SET OF TWO — "active" |
/// "inactive" — so equality is available and a substring test would be the 187-day prefix
/// lesson waiting to happen (`established` versus `established_recv_only`).
///
/// ⚠⚠ v2, NA-0763 (`D-0040`) — THIS SEAL'S v1 NAME AND HEADING BECAME FALSE, AND THAT IS
/// CORRECTED HERE RATHER THAN QUIETLY DELETED, following the pattern the neighbouring
/// selectors seal set when its own v1 heading promised a check it never made.
/// v1 was called `..._add_no_timer` and asserted "NOTHING POLLS" with `setInterval` pinned at
/// THREE — an honest self-binding by NA-0756, whose lane genuinely added none. NA-0763 IS the
/// lane that adds one: the liveness tick, rung 1 of the delivery ladder. So the count moves
/// 3 -> 4 and the fourth is NAMED. ⚠ Nothing is weakened: the assertion stays an EXACT
/// equality rather than relaxing to `>=`, so an unaccounted FIFTH timer still fails it — which
/// is the property the census existed to protect all along.
///
/// The two trigger needles also move from `inviteFinishScanPending(<why>)` to the one
/// handler's `relayScan({ source: <why>, at: ... })`. The triggers themselves are unchanged:
/// same call sites, same order, same awaited-ness — only the entry point is consolidated.
#[test]
fn na0756_the_triggers_use_equality_and_the_timer_census_is_exact() {
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"if (st.state !== "inactive") continue;"#),
        "the pending predicate must be an EQUALITY test on the extracted value"
    );
    assert!(
        js.contains("if (done === true) marks.finished += 1;"),
        "the finish outcome must be compared by equality, never by truthiness"
    );
    // Both triggers exist, and trigger (b) is on the CHOOSER's opener — R387 §S6. Item 1
    // retargets both entries there, so a trigger left on the create modal would cover only
    // half the doors.
    assert!(
        js.contains(r#"await relayScan({ source: "unlock", at: Date.now() });"#),
        "trigger (a) rides the unlock landing, through the ONE handler"
    );
    let start = js
        .find("async function openRedeemChooser()")
        .expect("the chooser opener exists");
    let body = &js[start..];
    let end = body.find("\n}\n").expect("it ends");
    assert!(
        body[..end].contains(r#"await relayScan({ source: "surface_open", at: Date.now() });"#),
        "trigger (b) rides the CHOOSER's opener, which is where BOTH retargeted entries land"
    );
    // ⚠ THE TIMER CENSUS, COUNTED AND NOT EYEBALLED. v1 pinned THREE and NA-0756 added none.
    // NA-0763 adds exactly ONE — the liveness tick — so the exact count is now FOUR. It stays
    // an EXACT equality on purpose: an unaccounted fifth timer must still fail this arm.
    assert_eq!(
        js.matches("setInterval(").count(),
        4,
        "the timer census is EXACT — the four that exist are the unlock countdown, the erase \
         countdown, the idle autolock and NA-0763's liveness tick"
    );
}

/// Z2 — THE LANE'S NEW SELECTORS ARE TOKEN-ONLY AND NONE OF THEM INVENTS A WIDTH.
///
/// ⚠ MUST GO RED IF: one of the named rules disappears, a literal hex enters them, or a second
/// width value is minted. The width must come from `.modal`, which expresses 500px EXACTLY ONCE
/// for the whole app — the v5 cure for a surface that resized as the user moved through one flow.
///
/// ⚠⚠ THE v1 HEADING OF THIS SEAL SAID SOMETHING THE SEAL DID NOT CHECK, AND THAT IS WORTH
/// RECORDING RATHER THAN QUIETLY DELETING. It read "THE TWO NEW SELECTORS ARE THE CLOSED SET"
/// and promised to go red "if a third selector appears" — but the body only ever asserted that
/// two NAMED rules exist, that one of them is token-only, and that the width is stated once. A
/// third selector would have sailed straight through it. The comment was a DESCRIPTION; the
/// assertions were the record, and they disagreed. v2 adds the third selector this pass needs,
/// so the heading is corrected to what is actually measured and the new rule is brought under
/// the same two disciplines instead of being left outside them.
#[test]
fn na0756_the_new_selectors_are_token_only_and_add_no_width() {
    let css = ui_file("style.css");
    assert!(
        css.contains(".callout.warning {"),
        "the warning modifier exists"
    );
    assert!(
        css.contains(".modal textarea {"),
        "the modal textarea rule exists"
    );
    // ⚠ v2 — the chooser's two-line row. ONE class, and the two lines it owns.
    for rule in [
        ".choose-row {",
        ".choose-row .choose-row-title {",
        ".choose-row .choose-row-sub {",
    ] {
        assert!(css.contains(rule), "the v2 row rule exists: `{rule}`");
    }
    // ⚠⚠ AND IT MINTS NO WIDTH. The rows take their width from the shipped `button.full`; a
    // `width` or `max-width` inside this class is how a surface starts resizing between its own
    // states again, which is precisely what v5 cured for Lane A.
    let rstart = css.find(".choose-row {").expect("found");
    let rend = rstart + css[rstart..].find('}').expect("rule ends");
    let row_rule = &css[rstart..rend];
    assert!(
        !row_rule.contains("width"),
        "the row class must not mint a width — `button.full` already supplies it"
    );
    assert!(
        !row_rule.contains('#'),
        "no literal hex may enter the row class — tokens are the authority"
    );
    // Tokens are the colour authority — never the mockup's hex.
    let wstart = css.find(".callout.warning {").expect("found");
    let wend = wstart + css[wstart..].find('}').expect("rule ends");
    let warn_rule = &css[wstart..wend];
    assert!(
        warn_rule.contains("var(--warn-bg)") && warn_rule.contains("var(--warn-border)"),
        "the warning callout uses the SHIPPED --warn-* tokens, which style.css:286 kept \
         defined for exactly this"
    );
    assert!(
        !warn_rule.contains('#'),
        "no literal hex may enter it — tokens are the authority"
    );
    // ⚠ The surface width is still expressed exactly ONCE in the whole stylesheet.
    assert_eq!(
        css.matches("max-width: 500px").count(),
        1,
        "the shared width must stay expressed exactly once — a second copy is how the states \
         drift apart again"
    );
}

/// v2 — THE CHOOSER STACKS AT FULL WIDTH IN THE SOURCE, AND THE RETIRED v1 FORM IS GONE.
///
/// ⚠ MUST GO RED IF: the chooser's controls go back onto one line, a row loses its tier or its
/// full-width class, Close disappears, or the v1 button form returns.
///
/// ⚠⚠ THE NEGATIVE NEEDLE IS THE SHIPPED FORM, NEVER A BARE WORD. "Invite someone" occurs three
/// times in this file for three legitimate reasons — the rail tip, the mint's heading, and this
/// row's own title — so a word-level pin would be either vacuous or wrong. The needle is the
/// exact v1 TAG: a tier-only class with the label as the button's own text. It occurred exactly
/// ONCE at the v1 head, which is what makes the zero below a measurement rather than an
/// accident of phrasing.
///
/// ⚠⚠ AND THE HALF THAT CAN BE WHOLE-FILE IS WHOLE-FILE. A slice cannot see a comment that sits
/// ABOVE its start — that is exactly how the plant hazard passed once already in this lane
/// (ENG-0235), and a scanner cannot tell a comment from markup. The v1 tag form is unique enough
/// to pin file-wide, so it is. The side-by-side-row half CANNOT be file-wide, because three
/// legitimate surfaces use that row, so that half is slice-scoped — and the chooser block's
/// prose is deliberately kept OUTSIDE the slice so the slice is markup only.
#[test]
fn na0756_v2_the_chooser_stacks_full_width_and_the_inline_form_is_gone() {
    let html = ui_file("index.html");

    // WHOLE-FILE negatives — the retired v1 tag form, both rows.
    for gone in [
        r#"class="secondary">Invite someone</button>"#,
        r#"class="secondary">I have a code</button>"#,
    ] {
        assert!(
            !html.contains(gone),
            "the retired v1 chooser button form must not return: `{gone}`"
        );
    }

    // The slice: markup only, from the view's own id to the next state's banner.
    let start = html
        .find(r#"<div id="choose-view">"#)
        .expect("the chooser exists");
    let rest = &html[start..];
    let end = rest
        .find("STATE 1: ADD A CONTACT")
        .expect("state 1 follows the chooser");
    let view = &rest[..end];

    assert!(
        !view.contains("btnrow"),
        "no side-by-side row may exist on this surface — every control stacks"
    );

    // Positives, so the absence above is measured against a surface that really has controls
    // rather than against an empty one.
    for needle in [
        r#"<button id="btn-choose-create" class="secondary full choose-row">"#,
        r#"<button id="btn-choose-redeem" class="secondary full choose-row">"#,
        r#"<button id="btn-choose-close" class="secondary full">Close</button>"#,
    ] {
        assert!(
            view.contains(needle),
            "the v2 control ships in its blessed form: `{needle}`"
        );
    }

    // Both halves of a row's geometry travel together: the TIER makes it a button the design
    // system recognises, the FULL class makes it span the content box. Counted, not eyeballed.
    assert_eq!(
        view.matches(r#"class="secondary full choose-row""#).count(),
        2,
        "exactly two two-line rows, each carrying its tier AND its full-width class"
    );

    // The Close wiring exists and reuses the shipped dismissal, which is what makes "fires no
    // invite call" structural rather than remembered — `closeRedeemModal` never invites.
    let js = ui_file("main.js");
    assert!(
        js.contains(r#"byId("btn-choose-close").addEventListener("click", closeRedeemModal);"#),
        "Close reuses the one shipped dismissal path rather than minting a second one"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NA-0765 (`D-0042`) — THE LANE C ACCEPTANCE REPAIRS
// ═══════════════════════════════════════════════════════════════════════════════════

/// **I7 — ONE NAME FOR ONE THING.**
///
/// The shipped Settings > Identity pane called the user's own code "Verification
/// code" before Lane C existed. Lane C introduced a second term for the same object,
/// and a lane that invents a word for a thing the app already names creates a second
/// vocabulary that every later screen has to choose between.
///
/// ⚠ MUST GO RED IF: the retired term returns anywhere under `ui/`, or the label
/// stops being the app's own word.
///
/// ⚠⚠ THE NEEDLE IS BUILT SO IT CANNOT FIRE ON ITS OWN RATIONALE. A test that bans a
/// phrase and then SPELLS that phrase in its own prose is self-falsifying — this file
/// records the same lesson at `strip_html_comments`, and `ENG-0235` is it firing for
/// real. So the banned phrase is never written here or in the sources: it is
/// ASSEMBLED at runtime from two halves that never sit adjacent in any file.
#[test]
fn na0765_the_verification_code_naming_is_singular() {
    let retired = format!("{} {}", "identity", "code");
    for name in ["index.html", "main.js", "style.css"] {
        let body = ui_file(name);
        assert_eq!(
            body.to_lowercase().matches(retired.as_str()).count(),
            0,
            "the retired term must not survive anywhere in `ui/{name}` — comments included, \
             because a second vocabulary spreads from prose into copy"
        );
    }
    // And the surviving word is the app's own, in BOTH places that render a code.
    let html = ui_file("index.html");
    let js = ui_file("main.js");
    assert!(
        html.contains(r#"<span class="field-label">Verification code</span>"#),
        "Settings keeps the label this lane standardised on"
    );
    assert!(
        js.contains(
            r#"label(d, ui === "changed" ? "Verification code (new)" : "Verification code");"#
        ),
        "the contact detail uses the same word, with the changed-key variant the layout draws"
    );
}

/// **A1 — THE RAIL CAN GO BACK, AND THE HIGHLIGHT FOLLOWS THE PANE.**
///
/// The shipped main rail's Chats button carried NO id and NO listener, and a
/// HARD-CODED `active` that nothing ever moved — so Contacts was a one-way door and
/// the rail kept selecting the pane you had left.
///
/// ⚠ MUST GO RED IF: the button loses its id or its listener, or the highlight mover
/// stops being called from either pane function.
#[test]
fn na0765_the_main_rail_switches_both_ways_and_the_highlight_follows() {
    let html = ui_file("index.html");
    let js = ui_file("main.js");

    // ⚠ THE ID IS `-m`, NOT THE BARE NAME: `btn-rail-chats` is already the SETTINGS
    // rail's button and is pinned by two scenarios. Asserted here so the asymmetry is
    // deliberate and documented rather than looking like a typo to the next reader.
    assert!(
        html.contains(r#"id="btn-rail-chats-m" data-tip="Chats""#),
        "the MAIN rail's Chats button carries its own id"
    );
    assert!(
        html.contains(r#"id="btn-rail-chats" data-tip="Chats""#),
        "the SETTINGS rail's Chats button keeps the bare name it was pinned under"
    );
    assert!(
        js.contains(
            r#"byId("btn-rail-chats-m").addEventListener("click", () => showChatsPane());"#
        ),
        "the main rail's Chats button opens the Chats pane"
    );

    // The highlight mover exists and is called from BOTH pane functions — which is what
    // makes the settings rail get it for free.
    assert!(
        js.contains("function railSelect(id) {"),
        "the highlight mover exists"
    );
    assert_eq!(
        js.matches("railSelect(\"btn-rail-contacts\");").count(),
        1,
        "showContactsPane selects Contacts, exactly once"
    );
    assert_eq!(
        js.matches("railSelect(\"btn-rail-chats-m\");").count(),
        1,
        "showChatsPane selects Chats, exactly once"
    );
}

/// **A2 — THE DETAIL PANE HAS THE PADDING THE BLESSED LAYOUT DRAWS.**
///
/// It shipped flush against the divider because `.content-pane` sets no padding and
/// only the `.welcome` modifier saved the one pane anybody looked at.
///
/// ⚠ MUST GO RED IF: the padded rule disappears, or it stops excluding `.welcome`
/// (which centres itself and must keep doing so in an unpadded box).
#[test]
fn na0765_the_detail_pane_is_padded_and_the_welcome_pane_is_not() {
    let css = ui_file("style.css");
    assert!(
        css.contains(
            ".content-pane:not(.welcome) { padding: var(--sp-x20) var(--sp-5); overflow: auto; }"
        ),
        "the detail pane is padded in SHIPPED tokens, and the welcome pane is excluded"
    );
    // The dead styling that let a string-shaped review believe the v5 button row had
    // shipped is gone; the hairline now comes from the shipped section idiom.
    assert_eq!(
        css.matches("contact-detail-divider").count(),
        0,
        "the divider class was declared once and used zero times — dead styling for a row \
         that was never built, and exactly what made the gap invisible to review"
    );
}

/// **I2 — THE DETAIL RENDERS ITS PARTS, AND `Block` IS DELIBERATELY ABSENT.**
///
/// ⚠⚠ THIS IS THE INSTRUMENT WHOSE ABSENCE LET THE GAP SHIP. The review that passed
/// the defective build checked for STRINGS where the claim was ELEMENTS. The
/// element-level half runs in the gui-driver against the RENDERED DOM; this half pins
/// the renderer's own structure, which a string check cannot fake.
///
/// ⚠ MUST GO RED IF: the rename control, the code card, the Devices projection or the
/// shipped section idiom leaves the renderer — or if a Block control appears while the
/// verb that would make its blessed copy true is still unreachable.
#[test]
fn na0765_the_contact_detail_is_complete_and_block_is_deferred() {
    let js = ui_file("main.js");
    for needle in [
        r#"form.className = "pane-form";"#,
        r#"d.className = "pane-sect";"#,
        r#"l.className = "field-label";"#,
        r#"rowEl.className = "ctlrow";"#,
        r#"card.className = "contact-code-card";"#,
        r#"save.id = "btn-contact-rename";"#,
        r#"input.id = "contact-rename-input";"#,
        r#"label(d, "Their name");"#,
        r#"label(d, "Connection");"#,
    ] {
        assert!(
            js.contains(needle),
            "the detail renders its blessed part: `{needle}`"
        );
    }

    // ⚠ THE DEFERRAL IS ASSERTED, NOT ASSUMED. R-1 ruled the blocking controls OUT of
    // this lane because the only reachable verb is one-way and destructive, so the
    // blessed "restores the connection you already had" would be FALSE.
    //
    // ⚠⚠ THE NEEDLE IS STRUCTURAL, NOT A WORD — AND THE FIRST DRAFT OF THIS VERY
    // ASSERTION PROVED WHY. It banned the two English words, and the paragraph
    // explaining the ban contained both of them, so the seal fired on its own
    // rationale. That is `ENG-0235` a third time, caught here before it ran. Ids and a
    // verb name cannot appear in prose by accident.
    for banned in [
        "btn-contact-block",
        "btn-contact-unblock",
        "contact_request_block",
    ] {
        assert_eq!(
            js.matches(banned).count(),
            0,
            "no blocking control ships while the honest symmetric pair is absent from the \
             facade — and in particular the app must not reach for the destructive \
             request-family verb instead (ENG-0248)"
        );
    }
}

/// **B1 — THE CHATS "+" IS GONE, AND THE FLOW IT CARRIED IS NOT.**
///
/// ⚠ MUST GO RED IF: the retired element returns, or the entries that replaced it
/// stop reaching the chooser.
#[test]
fn na0765_the_chats_plus_is_retired_and_the_flow_still_has_two_entries() {
    let html = ui_file("index.html");
    let js = ui_file("main.js");
    assert_eq!(
        html.matches("btn-invite-open").count(),
        0,
        "adding people is a Contacts act — the Chats header carries no entry any more"
    );
    assert_eq!(
        js.matches("btn-invite-open").count(),
        0,
        "and its listener retires with it, rather than dangling on a missing node"
    );
    // The replacements, measured rather than assumed — this is what makes the removal a
    // RETIREMENT rather than a hole.
    assert!(
        js.contains(
            r#"byId("btn-contacts-add").addEventListener("click", () => openRedeemChooser());"#
        ),
        "the Contacts pane's own + reaches the chooser"
    );
    assert!(
        js.contains(
            r#"byId("btn-add-contact").addEventListener("click", () => openRedeemChooser());"#
        ),
        "and so does the welcome panel, which this lane keeps on screen"
    );
}

/// **B4, RE-AIMED BY NA-0766 (`D-0043`) — BOTH FLOWS STILL HAVE A VISIBLE WAY
/// OUT, AND NOW IT IS THE SAME ONE ON EVERY SURFACE.**
///
/// NA-0765 gave the code-entry view its first visible exit — an X and a Back —
/// because it had NONE and the only ways out were Escape and the scrim. The
/// property that mattered was never "an X exists": it was **a visible exit that
/// shares the overlay's one dismissal**. NA-0766 replaces the X and the Back
/// with a single full-width Close on every invite surface, so the seal follows
/// the property rather than the control that used to carry it.
///
/// ⚠ THIS SEAL WENT RED ON THIS LANE'S FIRST EDIT, AND THAT IS THE SEAL WORKING
/// — its own heading said "MUST GO RED IF ... either Back disappears", and a
/// Back disappeared. Re-aimed, never weakened: the count of exits per surface is
/// still EXACT, so a surface losing its exit still fails, and so does one growing
/// a second (the Z6 precedent).
#[test]
fn na0765_both_modal_flows_have_a_visible_way_out() {
    let html = ui_file("index.html");
    let js = ui_file("main.js");
    // Every invite surface reaches a visible Close.
    for needle in [
        r#"id="btn-invite-close""#,
        r#"id="btn-choose-close""#,
        r#"id="btn-redeem-close3""#,
    ] {
        assert!(html.contains(needle), "the visible exit ships: `{needle}`");
        assert_eq!(
            html.matches(needle).count(),
            1,
            "exactly one of it: `{needle}`"
        );
    }
    // And each reuses its overlay's ONE dismissal — the same function Escape and
    // the scrim take — so the visible and invisible exits cannot drift apart.
    assert!(
        js.contains(
            r#"byId("btn-invite-close").addEventListener("click", () => closeInviteModal());"#
        ),
        "the invite Close reuses that overlay's one dismissal, which is also Escape's"
    );
    assert!(
        js.contains(r#"byId("btn-choose-close").addEventListener("click", closeRedeemModal);"#),
        "the chooser Close reuses the redeem overlay's one dismissal"
    );
    assert!(
        js.contains(r#"byId("btn-redeem-close3").addEventListener("click", closeRedeemModal);"#),
        "and so does the code-entry Close — the view that had no exit at all before NA-0765"
    );
}

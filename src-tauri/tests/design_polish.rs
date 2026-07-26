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

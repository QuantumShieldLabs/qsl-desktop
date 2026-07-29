//! D609 slice-B Server pane — additive structural + claim-discipline guards,
//! REVISED IN LOCKSTEP with the NA-0674 / D610 redesign.
//! design_round2.rs / design_system.rs / design_round3.rs stay byte-frozen;
//! these pin the Server-pane surface so a later edit cannot silently drift it.
//!
//! ⚠ D610 C5: this file — NOT design_round3.rs — is the Server pane's
//! frozen-needle home. The lane intent's G3 named design_round3.rs, which has
//! no server-pane coupling at all; these needles are the ones that actually
//! move when the pane's markup moves, and they moved in the same commit.

use std::fs;
use std::path::Path;

fn repo_file(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}
fn ui(name: &str) -> String {
    repo_file(&format!("ui/{name}"))
}

#[test]
fn server_pane_has_the_connectivity_controls() {
    let html = ui("index.html");
    for needle in [
        r#"id="relay-url""#,
        r#"id="relay-token""#,
        r#"id="relay-ca-path""#,
        r#"id="btn-relay-test""#,
        r#"id="btn-relay-save""#,
        r#"id="relay-results""#,
        // D610 redesign: the state-communication surfaces that REPLACED the
        // per-field buttons — one swapping token helper line (R-E1/E2/E3), the
        // CA status line (R-E4), the dirty helper (R-E5).
        r#"id="relay-token-help""#,
        r#"id="relay-ca-status""#,
        r#"id="relay-dirty""#,
    ] {
        assert!(html.contains(needle), "server pane missing {needle}");
    }
    // the slice-A placeholder copy is gone
    assert!(
        !html.contains("makes no network connections at all"),
        "the #pane-server placeholder copy must be replaced by the pane"
    );
}

#[test]
fn the_four_per_field_commit_buttons_are_gone_and_stay_gone() {
    // ⚠ NEGATIVE PIN (D610 R-A1). The [F.1-COMMIT] reversal REMOVED these four
    // controls; one unified Save commits everything and each field offers a
    // "remove it" link instead. A pin file that merely LOST its assertions
    // would document nothing and would not notice them coming back — so the
    // removal itself is asserted.
    //
    // Two of these were ENG-0073: `btn-relay-token-clear` and
    // `btn-relay-ca-clear` were adjacent controls BOTH labelled "Clear", and
    // the NA-0673 acceptance flight mis-clicked between them twice, each time
    // producing a plausible-looking wrong result card. The redesign removes
    // the confusion by removing the controls.
    let html = ui("index.html");
    let js = ui("main.js");
    for banned in [
        "btn-relay-token-set",
        "btn-relay-token-clear",
        "btn-relay-ca-set",
        "btn-relay-ca-clear",
    ] {
        assert!(
            !html.contains(banned),
            "removed per-field commit control reappeared in index.html: {banned}"
        );
        assert!(
            !js.contains(banned),
            "removed per-field commit control reappeared in main.js: {banned}"
        );
    }
    // R-F1: results state 8 ("Not saved yet") is gone as a results-block state;
    // its job folded into the dirty helper.
    assert!(
        !html.contains("Not saved yet"),
        "state 8 must be gone from the results block (R-F1)"
    );
    assert!(
        !html.contains("relay-saved-note"),
        "the state-8 note element must be gone (R-F1)"
    );
}

#[test]
fn three_sections_two_hairlines() {
    // R-D1: THREE sections in the form column, hairlines BETWEEN them only.
    // The adjacent-sibling CSS rule means the hairline count FOLLOWS the
    // section count — pinning both is what makes "exactly two" durable.
    let html = ui("index.html");
    let css = ui("style.css");
    assert_eq!(
        html.matches(r#"class="srv-sect""#).count(),
        3,
        "the Server pane must have exactly three sections (R-D1)"
    );
    assert!(
        css.contains(".srv-sect + .srv-sect { border-top: 1px solid var(--border); }"),
        "the hairline must come from the adjacent-sibling rule, so N sections give N-1 rules"
    );
    // F2R (operator-ruled): 30px had no nearest --sp step — it sat exactly
    // between --sp-x28 and --sp-6, and --sp-6 was ruled. No new token.
    assert!(
        css.contains(".srv-sect { padding: var(--sp-6) 0; }"),
        "section padding must be the ruled --sp-6 (F2R)"
    );
    assert!(
        !css.contains("--sp-x30"),
        "F2R rejected adding a new spacing token"
    );
    // The results block sat inside its own bordered container before; that
    // border would now read as a THIRD hairline.
    assert!(
        !html.contains(r#"id="relay-results" class="hidden" style="border-top"#),
        "the results block must not carry its own border-top (it would be a third hairline)"
    );
}

#[test]
fn removal_is_a_link_per_field_not_a_shared_button_label() {
    // R-A1/R-E1/R-E4 + F1R. Each field's removal affordance lives INSIDE the
    // line that describes that field's state, so the two can never be
    // confused for one another the way two "Clear" buttons were.
    let js = ui("main.js");
    let css = ui("style.css");
    assert!(
        js.contains(r#"a.className = "rm""#),
        "the removal affordance must be the prose link (R-A1)"
    );
    assert!(
        js.contains(r#"a.textContent = "remove it""#),
        "the removal link copy is 'remove it' (R-E1/R-E4)"
    );
    assert!(
        js.contains(r#"removalLink("relay-token-remove""#)
            && js.contains(r#"removalLink("relay-ca-remove""#),
        "each field gets its OWN removal link, distinctly identified"
    );
    assert!(
        css.contains("a.rm {"),
        "the removal link needs its own style"
    );
    // R-E3: a pending removal is CANCELLED by typing.
    assert!(
        js.contains("tokenPendingRemoval = false;") && js.contains("caPendingRemoval = false;"),
        "typing must cancel a pending removal (R-B3/R-E3)"
    );
}

#[test]
fn test_saves_first_and_a_failed_commit_never_probes() {
    // R-A2 is the whole reason this lane exists: on the shipped pane, typing a
    // token and pressing Test probed the OLD token, because the typed one had
    // never been committed. Test now commits first and probes what it saved.
    let js = ui("main.js");
    let test_handler = js
        .split(r#"byId("btn-relay-test").addEventListener"#)
        .nth(1)
        .expect("the Test handler must exist");
    let commit_at = test_handler
        .find("commitServerSettings()")
        .expect("Test must commit before probing (R-A2)");
    let probe_at = test_handler
        .find(r#"invoke("relay_test""#)
        .expect("Test must probe");
    assert!(
        commit_at < probe_at,
        "TEST SAVES FIRST: the commit must precede the probe (R-A2)"
    );
    // R-B2/R-B1: a failed commit abandons the remainder AND the probe.
    assert!(
        test_handler.contains("if (fail) {"),
        "a failed commit must short-circuit the Test handler"
    );
    // R-F2: the new state 14 exists and says the probe did not run.
    assert!(
        js.contains(r#"setBanner(byId("relay-status"), "accent", "Couldn't save settings")"#),
        "state 14 must render (R-F2)"
    );
    assert!(
        js.contains("no connection test was run"),
        "state 14 must say plainly that the probe did not run (R-F2)"
    );
}

#[test]
fn the_commit_order_is_fixed_and_settings_validation_gates_the_vault() {
    // D610 C2, as CORRECTED at implementation: neither the URL nor the CA path
    // can be validated without writing — the crate exposes no validate-only
    // command, and `relay_config_set` / `relay_ca_file_set` each validate BY
    // writing. R-B2 ("nothing persists" on a malformed address, on Save AND on
    // Test) therefore forces the address to be committed FIRST, inverting
    // R-B1's "settings.json last". Pinned so the order cannot drift back
    // silently while the deviation is on the record.
    let js = ui("main.js");
    let commit = js
        .split("async function commitServerSettings()")
        .nth(1)
        .expect("the unified commit must exist (R-A1)");
    let url_at = commit
        .find(r#"invoke("relay_config_set""#)
        .expect("URL step");
    let token_at = commit
        .find(r#"invoke("relay_token_set""#)
        .expect("token step");
    let ca_at = commit
        .find(r#"invoke("relay_ca_file_set""#)
        .expect("CA step");
    assert!(
        url_at < token_at && token_at < ca_at,
        "the commit order is address -> token -> CA (C2 as corrected)"
    );
    // R-B3: a blank token field KEEPS the stored token — it must never be
    // committed as an empty replacement.
    assert!(
        commit.contains(r#"byId("relay-token").value !== """#),
        "a blank token field must mean 'keep', not 'replace' (R-B3)"
    );
}

#[test]
fn dirty_helper_is_re_evaluated_after_the_normalized_echo() {
    // REGRESSION (D-0011, found by the NA-0674 acceptance flight).
    // renderDirty() ran inside refreshServerState(), i.e. BEFORE the R-B5 echo
    // wrote the normalized URL back into the field. When normalization changed
    // the string -- `https://192` -> `https://0.0.0.192`, a trailing slash, an
    // uppercase host -- the field still held the RAW text while savedRelayUrl
    // held the normalized one, so the pane read as dirty and the helper claimed
    // "Settings changed — not saved." about settings that HAD just been saved.
    // The echo then fixed the field, but nothing re-evaluated the helper.
    //
    // Pin the ORDER in both commit handlers: the echo, then renderDirty().
    let js = ui("main.js");
    for (handler, label) in [
        (r#"byId("btn-relay-test").addEventListener"#, "Test"),
        (r#"byId("btn-relay-save").addEventListener"#, "Save"),
    ] {
        let body = js.split(handler).nth(1).expect("handler must exist");
        let echo = body
            .find(r#"byId("relay-url").value = savedRelayUrl"#)
            .unwrap_or_else(|| panic!("{label}: the R-B5 echo must exist"));
        let dirty = body
            .find("renderDirty();")
            .unwrap_or_else(|| panic!("{label}: renderDirty must be called after a commit"));
        assert!(
            dirty > echo,
            "{label}: renderDirty() must run AFTER the R-B5 echo, or the dirty \
             helper reports 'not saved' for settings that were just saved"
        );
    }
}

#[test]
fn the_inline_validation_path_awaits_nothing_before_clearing() {
    // REGRESSION (D-0011, found by the NA-0674 acceptance flight).
    // The failed-commit path did `await refreshServerState()` BEFORE clearing
    // the results panel. refreshServerState() reaches relay_token_show /
    // relay_ca_file_show, and BOTH run on the process-wide SERIAL blocking
    // gate -- so a probe still in flight against a dead address parked the
    // await for the whole TCP timeout, the clear never ran, and a stale
    // "Testing…" banner sat under the new inline error, claiming a test was
    // running when none had been attempted.
    //
    // It was also wrong on principle: C2(b) requires re-reading after a PARTIAL
    // commit because something landed. R-B2 guarantees a validation failure
    // persists NOTHING, so this branch has nothing to re-read.
    let js = ui("main.js");
    let handler = js
        .split("function handleFailedCommit(")
        .nth(1)
        .expect("the shared failed-commit handler must exist");
    let inline_branch = handler
        .split("if (fail.inline) {")
        .nth(1)
        .and_then(|s| s.split("return;").next())
        .expect("the inline branch must exist");
    assert!(
        inline_branch.contains("clearServerResults()"),
        "the inline branch must clear the results panel"
    );
    assert!(
        !inline_branch.contains("refreshServerState"),
        "the inline branch must NOT re-read live state: nothing persisted (R-B2), \
         and the read is gated -- it can park behind an in-flight probe"
    );
    assert!(
        !inline_branch.contains("await"),
        "the inline branch must await NOTHING before the panel is cleared"
    );
}

#[test]
fn commit_failure_prose_never_opens_with_a_raw_error_code() {
    // REGRESSION (D-0011, found by the NA-0674 acceptance flight).
    // mapErr() falls through to String(e) when a code has no mapping, and the
    // result was concatenated onto the front of a sentence -- so state 14 read
    // "vault_write_failed The access token wasn't saved...". State 14 is only
    // ever reached AFTER something has gone wrong; leading with an internal
    // identifier is the worst moment for it. A friendly sentence leads; the
    // code stays, in parentheses, at the end.
    let js = ui("main.js");
    let commit = js
        .split("async function commitServerSettings()")
        .nth(1)
        .and_then(|s| s.split("function handleFailedCommit").next())
        .expect("the unified commit must exist");
    assert!(
        !commit.contains("mapErr(e, { relay_token_missing"),
        "the token-set failure must not lead its message with mapErr's raw-code fallback"
    );
    assert!(
        commit.contains(r#""The access token couldn't be saved to your vault (""#),
        "the token-set failure must lead with prose and carry the code in parentheses"
    );
}

#[test]
fn no_secret_is_written_outside_the_qsc_vault_trios() {
    // Unchanged boundary, re-pinned because the commit path moved: the token
    // and the CA path go to the vault through the qsc trios, the URL goes to
    // settings.json, and nothing crosses.
    let js = ui("main.js");
    for trio in [
        r#"invoke("relay_token_set""#,
        r#"invoke("relay_token_clear""#,
        r#"invoke("relay_ca_file_set""#,
        r#"invoke("relay_ca_file_clear""#,
    ] {
        assert!(js.contains(trio), "the vault trio call {trio} must remain");
    }
    assert!(
        !js.contains("secret_set"),
        "secrets go through the qsc trios, never vault::secret_set directly"
    );
    // The token is a BARE BOOL by design; the pane must not render a length.
    assert!(
        js.contains(r#"input.placeholder = "••••••••""#),
        "R-E1: a FIXED eight dots — never the real token length"
    );
}

#[test]
fn results_reuse_the_shipped_status_banner_no_invented_classes() {
    let html = ui("index.html");
    let js = ui("main.js");
    // the results headline IS the shipped §2 status-banner component
    assert!(html.contains(r#"id="relay-status" class="status-banner"#));
    // the connected state uses the shipped `neutral` kind
    assert!(js.contains(r#"setBanner(status, "neutral", "Connected")"#));
    // R7: red (status-danger) is RESERVED for vault-danger; the results never
    // use it, and NO new status-* colour class is invented (no mockup palette).
    for banned in ["status-ok", "status-bad", "status-warn", "status-success"] {
        assert!(
            !html.contains(banned) && !js.contains(banned),
            "invented status class {banned} — R7 forbids new colour classes"
        );
    }
}

#[test]
fn no_bypass_control_anywhere() {
    // R8: the GUI face of NA-0663's hard boundary. No connect-anyway / trust-
    // this-cert affordance exists; the ONLY remedy for an untrusted cert is the
    // operator CA file. Guard the copy so a later edit cannot smuggle a bypass.
    let html = ui("index.html").to_lowercase();
    let js = ui("main.js").to_lowercase();
    for banned in [
        "connect anyway",
        "trust this certificate",
        "trust anyway",
        "ignore certificate",
        "proceed anyway",
        "skip verification",
        "disable verification",
    ] {
        assert!(
            !html.contains(banned),
            "bypass affordance in index.html: {banned}"
        );
        assert!(
            !js.contains(banned),
            "bypass affordance in main.js: {banned}"
        );
    }
}

#[test]
fn claim_discipline_five_surfaces_swept() {
    // D609 R4: five surfaces edited; the two COMPOUND claims kept their
    // surviving true clause. Stale "no network / serverless" claims are retired.
    let html = ui("index.html");
    let js = ui("main.js");
    let lib = repo_file("src-tauri/src/lib.rs");
    let cmds = repo_file("src-tauri/src/commands.rs");

    // stale claims RETIRED (full phrases, so the explanatory comments — which
    // quote the retired clause — do not trip the guard)
    assert!(
        !html.contains("serverless skeleton"),
        "stub-note still says serverless skeleton"
    );
    assert!(
        !html.contains("server setup arrives in a future update"),
        "footer still says server setup arrives in a future update"
    );
    assert!(
        !js.contains("makes no network connections"),
        "main.js About still says makes no network connections"
    );
    assert!(
        !lib.contains("makes no network connections"),
        "lib.rs About still says makes no network connections"
    );
    assert!(
        !cmds.contains("serverless skeleton"),
        "app_info slice string still says serverless skeleton"
    );

    // A2 (D611): the README is a claim surface too. The stale sentence survived
    // NA-0673 and NA-0674 because this block covered the app and not the page
    // about the app. repo_file() panics if the path does not resolve, so this
    // cannot silently pass by reading nothing.
    let readme = repo_file("README.md");
    assert!(
        !readme.contains("makes no network connections at all"),
        "README status section still says makes no network connections at all"
    );

    // surviving TRUE clauses KEPT (the two compound surfaces)
    assert!(
        html.contains("Adding contacts arrives in a future update"),
        "stub-note lost the still-true contacts clause"
    );
    assert!(
        js.contains("no security-assurance claims"),
        "About in-app lost the surviving no-assurance clause"
    );
    assert!(
        lib.contains("no security-assurance claims"),
        "About native menu lost the surviving no-assurance clause"
    );
}

/// NA-0686 / D-1325 (ENG-0088) — THE NEEDLE GAP, closed.
///
/// ⚠ THE DEFECT WAS THE NEEDLE SET, NOT THE WORD. `claim_discipline_five_surfaces_swept`
/// retired "serverless skeleton" from `ui/index.html` and `src-tauri/src/commands.rs`
/// — and stopped there. Two published surfaces kept the retired claim for two
/// more slices: the crate DESCRIPTION in `Cargo.toml` (which reaches package
/// registries and any bundle manifest) and the MODULE DOC in `lib.rs:1` (which
/// reaches `cargo doc`).
///
/// The sharper reading, measured rather than inherited from the filing: `lib.rs`
/// WAS already in the older guard's needle set — but only for the phrase "makes
/// no network connections", never for "serverless skeleton". So the gap was not
/// "a file nobody looked at"; it was **a file looked at for the wrong needle**,
/// which is harder to spot and is exactly why a sweep that stops at the surfaces
/// a previous lane happened to name keeps missing the ones nobody thought of.
///
/// Slice B shipped relay connectivity. "Serverless skeleton" is no longer true
/// of the crate it describes, and a published crate description that states
/// something the product outgrew is a claim-discipline failure whatever file it
/// lives in.
#[test]
fn claim_discipline_covers_cargo_metadata_and_module_docs() {
    let cargo_toml = repo_file("src-tauri/Cargo.toml");
    let lib = repo_file("src-tauri/src/lib.rs");

    // (a) the retired claim is absent from BOTH newly covered surfaces.
    for (surface, body) in [
        ("src-tauri/Cargo.toml", &cargo_toml),
        ("src-tauri/src/lib.rs", &lib),
    ] {
        assert!(
            !body.contains("serverless skeleton"),
            "{surface} still carries the retired 'serverless skeleton' claim"
        );
        assert!(
            !body.contains("slice A:"),
            "{surface} still describes the crate as slice A only; slice B shipped relay connectivity"
        );
    }

    // (b) both surfaces carry the claim boundary. A description that merely drops
    //     the false clause is not the same as one that states the real limit —
    //     "research-stage, no security-assurance claims" is the standing posture
    //     and it must survive here as it does in the in-app About.
    for (surface, body) in [
        ("src-tauri/Cargo.toml", &cargo_toml),
        ("src-tauri/src/lib.rs", &lib),
    ] {
        assert!(
            body.contains("Research-stage") || body.contains("research-stage"),
            "{surface} must state the research-stage boundary"
        );
        assert!(
            body.contains("no security-assurance claims"),
            "{surface} must keep the no-security-assurance claim boundary"
        );
    }

    // (c) both describe what the crate ACTUALLY contains now.
    assert!(
        cargo_toml.contains("relay connectivity"),
        "Cargo.toml description must reflect slice B's relay connectivity"
    );
    assert!(
        lib.contains("relay connectivity"),
        "lib.rs module doc must reflect slice B's relay connectivity"
    );
}

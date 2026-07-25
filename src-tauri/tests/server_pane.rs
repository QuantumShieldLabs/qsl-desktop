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

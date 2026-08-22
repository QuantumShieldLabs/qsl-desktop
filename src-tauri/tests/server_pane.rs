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
        r#"id="relay-results""#,
        r#"id="relay-dirty""#,
        // NA-0754 (D-0035): the two clear controls that REPLACED the prose
        // "remove it" links. A stored secret must always be deletable even with
        // no relay reachable, so the affordance survives the helper lines that
        // used to host it — as an icon-only control that deletes immediately.
        r#"id="relay-token-clear""#,
        r#"id="relay-ca-clear""#,
    ] {
        assert!(html.contains(needle), "server pane missing {needle}");
    }
    // the slice-A placeholder copy is gone
    assert!(
        !html.contains("makes no network connections at all"),
        "the #pane-server placeholder copy must be replaced by the pane"
    );
}

/// ⚠ NEGATIVE PIN (NA-0754 / D-0035, design bank v2 items 1-2). The
/// test-and-save-on-proof model REMOVED the Save button and the two per-field
/// helper lines; this asserts the removal itself, because a pin file that merely
/// LOST its assertions would document nothing and would not notice them coming
/// back. Same reasoning as the four-buttons pin below, whose shape this follows.
///
/// WHY EACH ONE WENT:
///   `btn-relay-save` — its whole job was to persist an UNTESTED configuration,
///     which is precisely what the invariant ("what is persisted has connected at
///     least once") forbids. Removing it strengthens the invariant; it is UI
///     chrome, not a safety mechanism.
///   `relay-token-help` / `relay-ca-status` — the per-field status sentences. The
///     FIELDS now carry stored state (the token's fixed dots, the CA's stored
///     marker in its placeholder) and the results banner carries connection truth,
///     so a sentence narrating what the field already shows is clutter.
///
/// THE REGRESSION THIS MUST CATCH: a later lane re-adding a Save button, which
/// would silently restore the ability to persist a configuration that has never
/// connected — the exact defect this lane exists to make structurally impossible.
/// NA-0754 COPY RIDER (Bank F3, operator-blessed 2026-08-22) — THE RELAY PANE'S
/// HEADER COPY, PINNED VERBATIM.
///
/// ⚠ THIS STRING IS A TESTABLE CLAIM SET, NOT DECORATION, which is why it is
/// pinned character-for-character rather than by keyword. It asserts four things
/// about the product: messages are sealed client-side before transmission; names
/// are never present on the relay; delivery runs on anonymous codes; and the relay
/// observes traffic-flow only. This test pins the WORDS. It cannot and does not
/// verify the claims — the threat-model documentation owes that alignment, filed
/// as an open documentation candidate. Any future change to what the relay can
/// actually see must re-open this copy BEFORE it ships.
///
/// ⚠ MUST GO RED IF: a single character drifts. That is the point — a claim set
/// that can be quietly reworded is a claim set nobody is accountable for.
#[test]
fn the_relay_pane_header_copy_is_the_blessed_claim_set() {
    let html = ui("index.html");
    let blessed = "Assume the relay is hostile — this app already does. It's built so a compromised relay still learns almost nothing: your messages are sealed before they leave your device, your name and your contacts' names never exist on it, and delivery runs on anonymous codes. The most it can ever see is that sealed traffic is flowing.";
    assert!(
        html.contains(blessed),
        "the blessed F3 relay-pane copy must ship VERBATIM — no drift, house em-dash"
    );
    // The em-dash is house typography; the bank's `--` was transport armor only.
    assert!(
        blessed.contains('\u{2014}') && !blessed.contains("--"),
        "the shipped string carries the house em-dash, never the armor form"
    );
    // The superseded one-liner is retired in the same stroke, so the pane cannot
    // carry both the old understated claim and the new explicit one.
    assert!(
        !html.contains("The relay carries your encrypted messages."),
        "the retired one-liner must not survive beside its replacement"
    );
}

#[test]
fn the_save_button_and_the_per_field_helper_lines_are_gone_and_stay_gone() {
    let html = ui("index.html");
    for needle in [
        r#"id="btn-relay-save""#,
        r#"id="relay-token-help""#,
        r#"id="relay-ca-status""#,
    ] {
        assert!(
            !html.contains(needle),
            "{needle} must stay removed (NA-0754: test-and-save-on-proof)"
        );
    }
    // And the retired per-field status sentences themselves, so the copy cannot
    // come back on a differently-named element.
    for copy in [
        "A token is set",
        "Required only if the operator set one",
        "CA certificate file set",
        "No CA file set",
    ] {
        assert!(
            !ui("main.js").contains(copy),
            "the retired helper sentence {copy:?} must stay removed"
        );
    }
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
fn removal_is_a_distinctly_named_control_per_field_not_a_shared_label() {
    // R-A1's PURPOSE, carried forward through a changed mechanism (NA-0754 /
    // design bank v2 item 2). ENG-0073 was two ADJACENT controls both labelled
    // "Clear": the NA-0673 flight mis-clicked between them twice, each time
    // producing a plausible-looking wrong result. The fix was never "use a link";
    // it was "make the two removals impossible to confuse". D610 achieved that
    // with prose links inside each field's status line. Those status lines are
    // gone, so the affordance is now an icon control per field — and the
    // anti-confusion property is re-pinned on what actually carries it: each
    // control names ITS OWN field in its accessible name, and the two names
    // differ.
    //
    // ⚠ MUST GO RED IF: the two controls ever share a label, which is the exact
    // condition that produced ENG-0073.
    let html = ui("index.html");
    let token_label = "Remove stored token";
    let ca_label = "Remove stored certificate authority file";
    assert!(
        html.contains(&format!(r#"aria-label="{token_label}""#)),
        "the token clear must name its own field"
    );
    assert!(
        html.contains(&format!(r#"aria-label="{ca_label}""#)),
        "the CA clear must name its own field"
    );
    assert_ne!(
        token_label, ca_label,
        "the two removals must never share a label (ENG-0073)"
    );
    // Icon-only controls: an accessible name is the ONLY thing a screen reader
    // has here, so a missing one would make the affordance unreachable.
    for id in ["relay-token-clear", "relay-ca-clear"] {
        let at = html
            .find(&format!(r#"id="{id}""#))
            .expect("control present");
        let tag_end = html[at..].find('>').expect("tag closes") + at;
        assert!(
            html[at..tag_end].contains("aria-label="),
            "{id} is icon-only and MUST carry an accessible name"
        );
    }
}

#[test]
fn test_probes_first_and_a_failed_probe_never_persists() {
    // ⚠ THIS TEST IS THE INVERSE OF THE ONE IT REPLACES, AND THAT IS THE LANE.
    // R-A2 ruled TEST-SAVES-FIRST because nothing could be validated without
    // being written: the pane committed everything and then probed what it had
    // just saved. It fixed a real trap (a typed token that had never been
    // committed was not the token the probe used) and introduced a worse one —
    // a FAILED test overwrote a working configuration, which the operator met
    // in flight. The v2 design bank supersedes R-A2 with test-and-save-on-proof,
    // so the order pinned here is REVERSED on purpose: probe, then persist, and
    // only on a Connected result.
    let js = ui("main.js");
    let test_handler = js
        .split(r#"byId("btn-relay-test").addEventListener"#)
        .nth(1)
        .expect("the Test handler must exist");
    let probe_at = test_handler
        .find(r#"invoke("relay_probe""#)
        .expect("Test must probe with the TYPED values");
    let persist_at = test_handler
        .find("persistProvenSettings(proven)")
        .expect("Test must persist through the ruled order");
    assert!(
        probe_at < persist_at,
        "TEST-AND-SAVE-ON-PROOF: the probe must precede every write"
    );
    // The persist is GATED on the accepting outcome, not merely sequenced after
    // the probe — sequence alone would still save on a failed rung.
    let gate_at = test_handler
        .find(r#"res.kind === "reachable""#)
        .expect("the persist must be gated on Connected");
    assert!(
        gate_at < persist_at,
        "the write must be gated on a Connected result"
    );
    // And the user is TOLD nothing was saved — the invariant's user-facing half.
    assert!(
        js.contains("Nothing saved — your previous settings are unchanged."),
        "a failed test must say plainly that nothing was persisted"
    );
}

#[test]
fn the_persist_order_is_fixed_and_r_b1s_original_order_is_restored() {
    // D610 C2 ruled "URL -> token -> CA -> settings.json LAST", then implementation
    // measured its premise false: no command was validate-only, so validating the
    // address MEANT writing it, and R-B2's "nothing persists on a malformed
    // address" forced the address to commit FIRST — inverting R-B1.
    //
    // ⛳ NA-0754 dissolves the forcing rather than picking a side. `relay_probe`
    // validates an explicit triple while persisting nothing, so nothing has to be
    // written in order to be checked, and R-B1's ORIGINAL order is restored:
    // vault token -> vault CA -> settings.json LAST. Pinned so it cannot drift
    // back, because settings.json's `relay_url` is the OBSERVABLE configuration
    // (the status footer and relaunch both read it) — writing it last is what
    // keeps the surviving configuration coherent when a vault write fails.
    let js = ui("main.js");
    let persist = js
        .split("async function persistProvenSettings(")
        .nth(1)
        .expect("the persist function must exist");
    let token_at = persist
        .find(r#"invoke("relay_token_set""#)
        .expect("token step");
    let ca_at = persist
        .find(r#"invoke("relay_ca_file_set""#)
        .expect("CA step");
    let url_at = persist
        .find(r#"invoke("relay_config_set""#)
        .expect("settings step");
    assert!(
        token_at < ca_at && ca_at < url_at,
        "the restored order is token -> CA -> settings.json LAST (R379 Q2)"
    );
    // The superseded entry point must be gone, not merely bypassed.
    assert!(
        !js.contains("async function commitServerSettings"),
        "the write-then-probe commit path must not survive alongside its replacement"
    );
    // R-B3 STANDS: a blank token field KEEPS the stored token. Under the new model
    // that is expressed by sending `null` to the probe, meaning "use what's stored".
    assert!(
        js.contains(r#"token: typedToken !== "" ? typedToken : null"#),
        "a blank token field must mean 'keep', never 'replace' (R-B3)"
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
    //
    // NA-0754: the Save handler is GONE, so this now pins the one remaining
    // commit handler. The regression it guards is unchanged and still reachable —
    // Test still echoes the normalized URL back into the field after persisting,
    // and renderDirty() must still run after that echo.
    let js = ui("main.js");
    assert!(
        !js.contains(r#"byId("btn-relay-save")"#),
        "the Save handler is retired; this pin must not silently keep passing on a ghost"
    );
    // One handler now, not two — the loop collapses with the Save button.
    {
        let (handler, label) = (r#"byId("btn-relay-test").addEventListener"#, "Test");
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
    // NA-0754: the same discipline, re-anchored on the function that replaced
    // commitServerSettings(). The failure prose it guards is unchanged.
    let commit = js
        .split("async function persistProvenSettings(")
        .nth(1)
        .and_then(|s| s.split("function handleFailedCommit").next())
        .expect("the persist function must exist");
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

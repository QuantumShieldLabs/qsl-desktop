//! NA-0764 (`D-1405`) — THE CONTACTS SURFACE'S STRUCTURAL SEALS.
//!
//! What lives HERE is what only the SOURCE can answer: an invariant about which
//! string reaches a command, a ban on persistence, and the ORDER of a precedence
//! chain. What needs the mapping actually EXECUTED lives in the gui-driver
//! scenario `f_n_contacts_autoconnect`, because a structural test cannot tell a
//! chain that is present from a chain that is correct.
//!
//! ⚠ The desktop harness has NO FIXTURE RELAY (`ENG-0226`, open), so nothing here
//! or in `f_n` completes a real handshake. The auto-connect PATH is proven
//! against the engine at `na0764_m3_empty_slot_accept` in qsl-protocol, and end
//! to end only by the operator's two-machine flight. Stated, not hidden.

use std::fs;
use std::path::PathBuf;

fn ui_file(name: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("ui");
    p.push(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Comments stripped, so a rule's own EXPLANATION cannot satisfy the rule.
fn strip_line_comments(js: &str) -> String {
    js.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// **M-1 / C-24 / order 2(g) — THE ALIAS INVARIANT, SEALED STRUCTURALLY.**
///
/// Two names per contact now exist. `alias` is the KEY — it keys
/// `ContactsStore.peers`, `identity_read_pin(peer)` and
/// `qsp_session_for_channel(channel)` — while `display_name` is a label the user
/// chose. A UI that DISPLAYS `display_name` and PASSES `alias` is correct; one
/// that passes `display_name` is a silent break reaching identity pins and live
/// sessions, and it would look completely reasonable in review.
///
/// ⚠ MUST GO RED IF: `display_name` appears in ANY invoke-argument position.
/// The ruled count is ZERO. Its can-fail proof is the control test below, which
/// plants one and observes the count move.
#[test]
fn display_name_never_reaches_a_command() {
    let code = strip_line_comments(&ui_file("main.js"));
    assert_eq!(
        count_display_name_in_invoke_args(&code),
        0,
        "`display_name` must never be passed to a command — it is a LOCAL LABEL, and every \
         verb keys on `alias`. Rendering it is correct; passing it is a silent break that \
         reaches identity pins and sessions"
    );
    // And it IS rendered, or the field is dead weight and this seal is vacuous.
    assert!(
        code.contains("row.display_name ? row.display_name : row.alias"),
        "the display name must actually be RENDERED, falling back to the alias"
    );
}

/// The counter the seal above depends on, factored so its own control can drive
/// it. An invoke argument object is `invoke("cmd", { ... })`; this looks inside
/// those braces only.
fn count_display_name_in_invoke_args(code: &str) -> usize {
    let mut n = 0;
    let mut rest = code;
    while let Some(i) = rest.find("invoke(") {
        rest = &rest[i + "invoke(".len()..];
        // ⚠ BOUND EACH CALL TO ITS OWN STATEMENT. The first version searched for
        // the next `{` anywhere, so `invoke("contact_list")` — which takes NO
        // argument object — reached forward into unrelated code and counted a
        // render site as an argument. It reported 1 against a file that passes
        // display_name nowhere. A scanner that over-reaches is a false positive
        // generator, and a seal built on one gets deleted by the next lane.
        let stmt = match rest.find(';') {
            Some(end) => &rest[..end],
            None => rest,
        };
        let Some(open) = stmt.find('{') else { continue };
        if stmt[open..].contains("display_name") {
            n += 1;
        }
    }
    n
}

/// **THE CAN-FAIL PROOF for the alias invariant.** A seal that counts to zero is
/// worthless unless the counter can reach one.
#[test]
fn the_alias_invariant_counter_can_actually_count() {
    // One violation, one clean call, and one RENDER site. The count must be
    // exactly 1: the counter has to see the violation AND ignore both the
    // argument-less call and the legitimate render.
    let planted = r#"
        await invoke("invite_finish", { selfLabel: null, alias: row.display_name, max: 1 });
        await invoke("contact_list");
        const shown = row.display_name ? row.display_name : row.alias;
    "#;
    assert_eq!(
        count_display_name_in_invoke_args(planted),
        1,
        "the counter must SEE a display_name in an argument object, and must NOT count an \
         argument-less invoke or a render site — if it cannot do both, the zero it reports on \
         the real file means nothing"
    );
}

/// **L1 / ruling sec 5 — THE BADGE ACK ADDS NO PERSISTENCE.**
///
/// "No new persistence for the badge ack." The badge is a NUDGE; the future
/// verification lane is what makes "verified" a durable fact. A badge that
/// survived a restart would be quietly claiming to be that record.
///
/// ⚠ MUST GO RED IF: the badge set is written to settings, to a file, or to any
/// storage at all.
#[test]
fn the_badge_ack_adds_no_persistence() {
    let code = strip_line_comments(&ui_file("main.js"));
    let decl = code
        .find("const contactsNewBadge")
        .expect("the badge set must exist");
    let body = &code[decl..];
    let scope = &body[..body.len().min(4000)];
    for banned in [
        "settings_set",
        "localStorage",
        "sessionStorage",
        "indexedDB",
        "writeTextFile",
    ] {
        assert!(
            !scope.contains(banned),
            "the badge ack must not persist through `{banned}` — sec 5 forbids new persistence \
             for it"
        );
    }
    assert!(
        code.contains("contactsNewBadge.delete(contactsSelected)"),
        "opening a contact must CLEAR its badge — a nudge that never clears is a permanent \
         decoration, not an acknowledgment"
    );
}

/// **F1 — THE LANE'S MOST IMPORTANT SINGLE ARM, STRUCTURALLY.**
///
/// `missing_seed` must NOT be in the fault list. The desktop never sets
/// `QSC_QSP_SEED`, so every not-yet-connected contact answers it; classifying it
/// as a fault would tell every establishing contact it has a storage problem and
/// would leave "Connecting…" with no reachable member. The shipped footer
/// already ruled this once (D-0033).
///
/// ⚠ MUST GO RED IF: `missing_seed` or `no_session` is added to the fault list.
#[test]
fn missing_seed_is_not_classified_as_a_fault() {
    let code = strip_line_comments(&ui_file("main.js"));
    let start = code
        .find("const CONTACT_FAULT_REASONS")
        .expect("the fault list must exist");
    let body = &code[start..];
    let list = &body[..body.find("];").expect("the list must terminate")];

    assert!(
        !list.contains("missing_seed"),
        "`missing_seed` is the ORDINARY ESTABLISHING STATE in this app, not a fault. Listing \
         it here ships 'This connection has a storage problem' on the commonest state and \
         leaves Connecting… unreachable"
    );
    assert!(
        !list.contains("no_session"),
        "`no_session` is unreachable in the desktop (QSC_QSP_SEED is never set) and is not a \
         fault either"
    );
    // The four that ARE faults, named — so the list cannot be emptied to pass.
    for r in [
        "session_invalid",
        "unsafe_parent",
        "missing_home",
        "channel_invalid",
    ] {
        assert!(list.contains(r), "`{r}` must be classified as a fault");
    }
}

/// **F2/F3 — THE PRECEDENCE ORDER, WHICH IS THE WHOLE MAPPING.**
///
/// R4 rules two dominations: CHANGED dominates Active, and blocked dominates
/// CHANGED. In an ordered if-chain the ORDER *is* the semantics, so this asserts
/// positions rather than presence.
///
/// ⚠ MUST GO RED IF: the arms are reordered. A CHANGED contact rendering as
/// "Connected" is the MITM tell being hidden by the state it warns about.
#[test]
fn the_state_precedence_is_blocked_then_changed_then_new() {
    let code = strip_line_comments(&ui_file("main.js"));
    let start = code
        .find("function contactUiState")
        .expect("the mapping must be ONE named function");
    let body = &code[start..];
    let body = &body[..body.find("\n}").expect("function end")];

    let blocked = body.find("row.blocked").expect("blocked arm");
    let changed = body.find(r#""CHANGED""#).expect("CHANGED arm");
    let badge = body.find("contactsNewBadge.has").expect("badge arm");
    let active = body.find(r#""active""#).expect("active arm");

    assert!(
        blocked < changed,
        "blocked must DOMINATE changed (R4): a blocked contact is blocked whatever its key did"
    );
    assert!(
        changed < badge && changed < active,
        "CHANGED must DOMINATE Active (R4): the peer-key-changed signal is the MITM tell, and \
         a connected-looking row would bury it"
    );
    assert!(
        badge < active,
        "the verify-first badge outranks plain Connected — an auto-created contact must not \
         look settled before anyone compared a code"
    );
}

/// **I4's STRUCTURAL HALF — the footer's outage arm sits LAST.**
///
/// The behaviour proof (an unknown reason WHILE the tick is in trouble) lives in
/// `f_n_contacts_autoconnect`, because only execution can prove what the chain
/// RETURNS. This proves what a textual seal can: the arm's POSITION.
///
/// ⚠ MUST GO RED IF: the outage arm is moved above any reason arm. There it
/// would mask the storage line, the locked line, and the please-report tripwire
/// — and a relay outage and a storage fault are correlated, so the masking would
/// happen exactly when it hurts.
#[test]
fn the_footer_outage_arm_sits_below_every_reason_arm() {
    let code = strip_line_comments(&ui_file("main.js"));
    let start = code
        .find("function statusFooterLine")
        .expect("the footer mapping must be ONE named function");
    let body = &code[start..];
    let body = &body[..body.find("\n}").expect("function end")];

    // ⚠ ANCHOR ON THE ARM, NOT THE NAME. `tickTrouble` is also the third
    // PARAMETER, and the signature precedes every arm — so searching for the
    // bare name reported the outage arm as first and failed against a chain
    // that was in fact correct.
    let trouble = body.find("if (tickTrouble)").expect("the outage arm");
    for (token, what) in [
        ("missing_home", "storage"),
        ("vault_locked", "locked"),
        ("unrecognized", "the please-report tripwire"),
    ] {
        let at = body
            .find(token)
            .unwrap_or_else(|| panic!("`{token}` must still be matched inside the footer"));
        assert!(
            at < trouble,
            "the outage arm must sit BELOW {what} — placed above it, an outage silently \
             replaces a problem the user can actually act on"
        );
    }
    assert!(
        body.find("Ready. Relay:").expect("the ready arm") > trouble,
        "the outage arm renders only in the otherwise-Ready case, so it sits ABOVE Ready"
    );
}

//! NA-0766 (`D-0043`) — THE INVITE FLOW'S STRUCTURAL SEALS.
//!
//! What lives HERE is what only the SOURCE can answer: which CONTROLS a surface
//! carries, which closer they share, and whether a decision is expressed in ONE
//! place. What needs the app actually RUNNING — a disabled state, a visible
//! element set, a copy link's revert, a slot count that must not move — lives in
//! the gui-driver scenarios `f_k_invite_create` and `f_l_invite_redeem`, because
//! a structural test cannot tell a control that is present from a control that
//! is enabled.
//!
//! ⚠⚠ EVERY NEEDLE FOR A RETIRED THING IS ASSEMBLED AT RUNTIME from halves that
//! never sit adjacent in this file. That is not decoration: `ENG-0235` is the
//! hazard where a seal's own prose satisfies the seal, and this lane's ruling Q5
//! is a live instance of its mirror image — an assertion held GREEN for two lanes
//! by the comment that documented its subject's deletion. A seal that spells what
//! it bans cannot fail.

use std::fs;
use std::path::PathBuf;

fn ui_file(name: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("ui");
    p.push(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Comments stripped, so a rule's own EXPLANATION cannot satisfy the rule. The
/// JS twin of this already lives in `na0764_contacts_surface.rs`; the HTML side
/// is what ruling Q5 showed was missing.
fn strip_html_comments(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(i) = rest.find("<!--") {
        out.push_str(&rest[..i]);
        match rest[i..].find("-->") {
            Some(j) => rest = &rest[i + j + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// The markup slice belonging to ONE view, from its id to the id that follows it.
/// Element-level, never string-level: the point of `I1` is to count CONTROLS.
fn view_slice<'a>(html: &'a str, start_id: &str, end_id: &str) -> &'a str {
    let a = html
        .find(&format!("id=\"{start_id}\""))
        .unwrap_or_else(|| panic!("view `{start_id}` not found"));
    let b = html[a..]
        .find(&format!("id=\"{end_id}\""))
        .unwrap_or_else(|| panic!("terminator `{end_id}` not found after `{start_id}`"));
    &html[a..a + b]
}

/// Count opening tags of `tag` inside a slice — elements, not mentions.
fn count_elements(slice: &str, tag: &str) -> usize {
    slice.matches(&format!("<{tag} ")).count() + slice.matches(&format!("<{tag}>")).count()
}

// ─────────────────────────────────────────────────────────────────────────────
// I1 — ONE EXIT PER MODAL, BY ELEMENT
// ─────────────────────────────────────────────────────────────────────────────

/// **I1 — EVERY INVITE MODAL HAS EXACTLY ONE EXIT CONTROL, AND IT IS A CLOSE.**
///
/// The NA-0765 lesson, applied as the brief ordered it: ASSERT THE ELEMENT,
/// NEVER THE STRING. That lane's PR review checked that blessed strings were
/// present and passed three ELEMENT-level defects straight through.
///
/// ⚠ MUST GO RED IF: any X returns, any Back returns, or any of the four modals
/// stops carrying exactly one full-width Close.
#[test]
fn na0766_i1_one_exit_per_modal_by_element() {
    let html = strip_html_comments(&ui_file("index.html"));

    // Assembled at runtime — see the module note.
    let x_invite = format!("id=\"btn-invite-{}\"", "x");
    let x_redeem = format!("id=\"btn-redeem-{}\"", "x");
    let back_chooser = format!("id=\"btn-invite-back-{}\"", "chooser");
    let back_redeem = format!("id=\"btn-redeem-{}\"", "back");
    let new_invite = format!("id=\"btn-invite-{}\"", "back");

    for needle in [
        &x_invite,
        &x_redeem,
        &back_chooser,
        &back_redeem,
        &new_invite,
    ] {
        assert!(
            !html.contains(needle.as_str()),
            "a retired control is back in the markup: `{needle}`"
        );
    }

    // The four modals the blessed layout draws, and the ONE Close each one reaches.
    // `#invite-mint` and `#invite-list-view` are served by the overlay-level footer,
    // which is a sibling of both — so the footer's Close is counted once for the
    // overlay, and neither view may carry a second.
    let mint = view_slice(&html, "invite-mint", "invite-list-view");
    let list = view_slice(&html, "invite-list-view", "btn-invite-close");
    let chooser = view_slice(&html, "choose-view", "redeem-form");
    let entry = view_slice(&html, "redeem-form", "redeem-sent");

    assert_eq!(
        html.matches("id=\"btn-invite-close\"").count(),
        1,
        "the invite overlay has exactly ONE Close element, shared by its two views"
    );
    assert!(
        html.contains(r#"<div class="block"><button class="secondary full" id="btn-invite-close">Close</button></div>"#),
        "and it is FULL WIDTH, alone in its row"
    );
    for (name, slice) in [("#invite-mint", mint), ("#invite-list-view", list)] {
        assert_eq!(
            slice.matches("id=\"btn-invite-close\"").count(),
            0,
            "{name} must not carry a Close of its own — the footer serves both"
        );
    }

    assert!(
        chooser.contains(r#"<button id="btn-choose-close" class="secondary full">Close</button>"#),
        "the chooser's single full-width Close is unchanged"
    );
    assert_eq!(
        chooser.matches("id=\"btn-choose-close\"").count(),
        1,
        "exactly one, on the chooser"
    );

    assert!(
        entry.contains(r#"id="btn-redeem-close3" class="secondary full stacked""#),
        "the code-entry view's Close is full width and stacked beneath Connect"
    );
    assert_eq!(
        entry.matches("id=\"btn-redeem-close3\"").count(),
        1,
        "exactly one, on the code-entry view"
    );

    // The mint's foot is Activate then the footer's Close, and Activate is the LAST
    // child of the mint — the only arrangement satisfying items 11 and 14 together.
    let act = mint
        .find("id=\"btn-invite-activate\"")
        .expect("Activate lives on the mint");
    let island = mint.find("code-island").expect("the island is on the mint");
    assert!(
        act > island,
        "ITEM 11: Activate & Copy sits BELOW the code island, at the bottom of the window"
    );
    assert!(
        count_elements(mint, "button") == 1,
        "the mint carries exactly ONE button element — Activate; Close is the footer's"
    );
}

/// **I2 — ESCAPE AND THE SCRIM STILL CLOSE, THROUGH THE SAME ONE CLOSER.**
///
/// Shipped behaviour this lane must NOT regress (brief sec 3 and item 4). The
/// visible exit and the two invisible ones share one function per overlay, which
/// is what makes it structurally impossible for them to disagree.
///
/// ⚠ MUST GO RED IF: either overlay's Escape or scrim stops reaching its one
/// closer, or a Close is wired to a second, bespoke dismissal.
#[test]
fn na0766_i2_escape_and_scrim_keep_their_one_closer() {
    let js = ui_file("main.js");
    for (overlay, closer) in [
        ("invite-overlay", "closeInviteModal()"),
        ("redeem-overlay", "closeRedeemModal()"),
    ] {
        let scrim = format!(
            "byId(\"{overlay}\").addEventListener(\"click\", (ev) => {{\n  if (ev.target === byId(\"{overlay}\")) {closer};\n}});"
        );
        assert!(
            js.contains(&scrim),
            "{overlay}'s scrim still reaches its one closer"
        );
    }
    // NA-0778 (004c / RULING_NA0778_008 R54): the close confirmation of 004a was RETIRED by the operator's
    // flight ruling, and the needles return to the ONE closer -- Escape, the scrim and Close reach
    // `closeInviteModal()` directly, as they did before 004. What 004a added is kept where it still
    // holds: a re-open of a live mint (the keyboard's route behind the scrim) is a NO-OP, never a
    // reset -- pinned below as the guard's first statement.
    assert!(
        js.contains("if (ev.key === \"Escape\") closeInviteModal();"),
        "Escape still closes the invite overlay, through the one closer"
    );
    let open = js
        .find("async function openInviteModal() {")
        .expect("the mint opener exists");
    let open_body = &js[open..open + 400];
    assert!(
        open_body.contains(
            r#"if (inviteMinted && !byId("invite-overlay").classList.contains("hidden")) return;"#
        ),
        "a re-open while a code is on screen is a NO-OP: the open mint stays exactly as it is"
    );
    assert!(
        !js.contains("inviteRequestClose") && !js.contains("invite-close-confirm"),
        "the retired question leaves no needle behind (a comment cannot re-plant it)"
    );
    assert!(
        js.contains("if (ev.key === \"Escape\") closeRedeemModal();"),
        "Escape still closes the redeem overlay"
    );
    // The visible exits reuse the SAME closers, never a second one.
    assert!(
        js.contains(
            r#"byId("btn-invite-close").addEventListener("click", () => closeInviteModal());"#
        ),
        "the invite Close reuses that overlay's one dismissal"
    );
    assert!(
        js.contains(r#"byId("btn-redeem-close3").addEventListener("click", closeRedeemModal);"#),
        "the code-entry Close reuses that overlay's one dismissal"
    );
}

/// **I7 — "review invites" IS A LINK, NOT A BUTTON, AND CARRIES NO COUNT.**
///
/// ⚠ MUST GO RED IF: the control becomes a button element, gains an underline,
/// stops reading lowercase, or the outstanding-count line returns.
#[test]
fn na0766_i7_review_invites_is_a_link_not_a_button() {
    let html = strip_html_comments(&ui_file("index.html"));
    let css = ui_file("style.css");
    let js = ui_file("main.js");

    // NA-0778 (`D-0047`) RE-AIM AT MOCKUP 17: the single link became a NON-CLICKABLE label with
    // three links beneath it. The property is unchanged -- anchors on the shipped plain idiom,
    // lowercase, no count -- and the label is pinned as a paragraph, not an anchor.
    assert!(
        html.contains(
            r#"<p id="contacts-invitations-label" class="contacts-inv-label">Invitations</p>"#
        ),
        "the label is a paragraph, not a link: blue is for what is clickable"
    );
    for (id, text) in [
        ("btn-contacts-review", "review"),
        ("btn-contacts-redeem", "redeem"),
        ("btn-contacts-send", "send"),
    ] {
        assert!(
            html.contains(&format!(
                r#"<a class="rm plain" id="{id}" role="button" tabindex="0">{text}</a>"#
            )),
            "the `{text}` link ships as an ANCHOR on the shipped plain text-link idiom, lowercase, no count"
        );
    }
    // NA-0778 (004a / F-07): the no-badge check reads the BLOCK's own slice, so it discriminates --
    // the first form tested a selector string that cannot occur in markup, and a whole-file needle.
    let block = view_slice(&html, "contacts-invitations", "contacts-rows");
    assert!(
        block.contains(r#"id="btn-contacts-send""#),
        "non-vacuity: the slice is the invitations block"
    );
    assert!(
        !block.contains(r#"class="count""#) && !block.contains("<span class=\"count"),
        "no count badge inside the block (mockup 17 v2)"
    );
    // By ELEMENT: there is no button element carrying this id anywhere.
    assert!(
        !html.contains("<button class=\"rm plain\" id=\"btn-contacts-review\"")
            && !html.contains("id=\"btn-contacts-review\"></button>"),
        "it is not a button element"
    );
    assert!(
        css.contains("a.rm.plain { text-decoration: none; }"),
        "the plain idiom removes the underline, and is REUSED rather than re-minted"
    );
    // The retired hint: its class, its element and its copy are all gone.
    let hint_class = format!("contacts-{}", "hint");
    assert!(
        !html.contains(hint_class.as_str()),
        "the outstanding-count element is gone from the pane"
    );
    assert!(
        !css.contains(hint_class.as_str()),
        "and its rule went with it — a rule whose only element is gone is DEAD CSS, which is \
         exactly the artifact NA-0765 used to prove a blessed row had been styled but never built"
    );
    let outstanding = format!("invites {}", "outstanding");
    assert!(
        !js.contains(outstanding.as_str()),
        "and the pane no longer renders a count"
    );
}

/// **I8 — THE CODE-ENTRY VIEW CARRIES CONNECT AND CLOSE, BOTH FULL WIDTH.**
///
/// This screen is the reason items 2, 3 and 15 land together: before NA-0765 it
/// had no exit at all, and NA-0765 gave it an X and a Back which this lane
/// removes. Remove those without adding the Close and the screen is a trap again.
///
/// ⚠ MUST GO RED IF: either control loses `full`, the Close disappears, or a
/// third control joins the foot.
#[test]
fn na0766_i8_code_entry_carries_connect_and_close() {
    let html = strip_html_comments(&ui_file("index.html"));
    let entry = view_slice(&html, "redeem-form", "redeem-sent");
    assert!(
        entry.contains(
            r#"<button id="btn-redeem-connect" class="primary full" disabled>Connect</button>"#
        ),
        "Connect runs FULL WIDTH"
    );
    assert!(
        entry.contains(
            r#"<button id="btn-redeem-close3" class="secondary full stacked">Close</button>"#
        ),
        "with a full-width Close directly beneath it"
    );
    assert_eq!(
        count_elements(entry, "button"),
        2,
        "exactly two controls at this foot — no X, no Back, nothing else"
    );
    // Ruling Q1's one line: the failure view's foot takes the same shape.
    let failed = view_slice(&html, "redeem-failed", "btn-redeem-copydetails");
    let _ = failed; // the slice proves the view still exists and is unrenamed
    assert!(
        html.contains(
            r#"<button id="btn-redeem-close2" class="secondary full stacked">Close</button>"#
        ),
        "ruling Q1: the failure view's Close is full width, with Copy details above it"
    );
}

/// **ITEM 10 — THE NAME GATE LIVES IN THE ONE ASSIGNMENT THAT DECIDES ACTIVATE.**
///
/// The shape is forced rather than chosen (ruling sec 2(c)). `inviteRefresh()`
/// runs on open AND after every mint and ASSIGNS the disabled flag outright, so a
/// name gate placed beside it would be silently overwritten by the next refresh.
///
/// ⚠ MUST GO RED IF: the name term leaves the decision, the trim leaves the
/// emptiness test, or a second site starts computing the same flag.
#[test]
fn na0766_the_name_gate_lives_in_the_single_assignment() {
    let js = ui_file("main.js");
    assert!(
        js.contains(
            r#"  byId("btn-invite-activate").disabled = inviteNoRelay || inviteCapFull || !nameOk || inviteMinted;"#
        ),
        "the ONE decision carries all four causes, the name among them"
    );
    // NA-0778 (004d / RULING_NA0778_009 R61): the name term is the REDEEM side's grammar -- the trim
    // still comes first (whitespace alone is empty) and the SAME regex constant the redeem gate
    // uses decides legality, so the two fields cannot drift apart. The typed value is never
    // rewritten by the gate.
    assert!(
        js.contains(r#"const name = byId("invite-label").value.trim();"#)
            && js.contains(r#"const nameOk = name !== "" && REDEEM_NAME_RE.test(name);"#),
        "the name term trims first and applies the redeem side's grammar through the shared constant"
    );
    let gate = js
        .find("function inviteSyncActivate() {")
        .expect("the one gate exists");
    let gate_body = &js[gate..];
    let gate_end = gate_body.find("\n}\n").expect("it ends");
    assert!(
        !gate_body[..gate_end].contains(r#"byId("invite-label").value ="#),
        "the gate never rewrites what the user typed"
    );
    assert!(
        js.contains(
            r#"if (label === "" || !REDEEM_NAME_RE.test(label)) { inviteSyncActivate(); return; }"#
        ),
        "and the commit handler refuses an illegal name independently of the button"
    );
    assert_eq!(
        js.matches(r#"byId("btn-invite-activate").disabled ="#)
            .count(),
        1,
        "exactly ONE site computes it — a second would drift from the first"
    );
    // ITEM 12: the latch that makes a second mint deliberate, and the read-only field.
    assert!(
        js.contains("  inviteMinted = true;"),
        "a successful mint latches: one invite per window"
    );
    assert!(
        js.contains(r#"byId("invite-label").readOnly = true;"#),
        "and the name field becomes read-only showing what was minted"
    );
}

/// **ITEM 6 — THERE IS NO BEFORE/AFTER PAIR LEFT TO SWAP.**
///
/// The structural half of the no-transform property. The half that matters to a
/// user — that nothing APPEARS or MOVES — is measured on the running app in
/// `f_k_invite_create`, because visibility is not a source-level fact.
///
/// ⚠ MUST GO RED IF: either container returns, or a swap function returns.
#[test]
fn na0766_the_window_has_no_before_after_pair() {
    let html = strip_html_comments(&ui_file("index.html"));
    let js = ui_file("main.js");
    let pre = format!("id=\"invite-{}\"", "pre");
    let post = format!("id=\"invite-{}\"", "post");
    assert!(!html.contains(pre.as_str()), "the before container is gone");
    assert!(!html.contains(post.as_str()), "the after container is gone");
    let swap = format!("inviteShow{}", "Post");
    assert!(
        !js.contains(swap.as_str()),
        "and the function that swapped them is gone"
    );
    // The code slot ships PRESENT and EMPTY, with its sentence in it.
    assert!(
        html.contains(r#"<div class="code-box empty" id="invite-code" tabindex="0">Your invite code will appear here after you activate.</div>"#),
        "ITEM 7: the slot is present from open and says what will land in it"
    );
    assert!(
        js.contains(
            r#"const INVITE_SLOT_EMPTY = "Your invite code will appear here after you activate.";"#
        ),
        "and the reset path shares ONE definition of that sentence with the markup"
    );
}

/// **RULING Q5, AS A GENERAL PROPERTY — A COMMENT CANNOT SATISFY A COPY SEAL.**
///
/// The finding this lane was ruled on: an assertion that a control's label was
/// PRESENT had been unfalsifiable since v4, passing only on the HTML comment that
/// recorded the control's REMOVAL — while its true counterpart, asserting the
/// same control was GONE, also passed. Both green, contradicting each other.
///
/// The general property is worth more than the fix: A COMMENT THAT DOCUMENTS A
/// REMOVAL RE-PLANTS THE REMOVED THING'S NEEDLE. Any seal built on bare-word
/// presence can be held green by the explanation of its own subject's deletion.
///
/// This seal makes that impossible to reintroduce: for every control this lane
/// retired, the name must be absent from the RAW source too — not merely from the
/// markup — so no future comment can resurrect it as a needle.
///
/// ⚠ MUST GO RED IF: any retired control's name reappears anywhere in `ui/`,
/// including inside a comment.
#[test]
fn na0766_a_comment_cannot_satisfy_a_copy_seal() {
    let html_raw = ui_file("index.html");
    let js_raw = ui_file("main.js");
    let html_stripped = strip_html_comments(&html_raw);

    // The stripper must be PROVEN LIVE on this very file, or every assertion below
    // is vacuous: real markup must survive it, and it must actually remove something.
    assert!(
        html_stripped.contains("Who is this invite for?"),
        "control: real markup survives comment-stripping"
    );
    assert!(
        html_stripped.len() < html_raw.len(),
        "control: the stripper actually removed comment bytes from this file"
    );

    // Assembled at runtime, halves never adjacent — this seal must not plant what it bans.
    let retired = [
        format!("btn-invite-{}", "x"),
        format!("btn-redeem-{}", "x"),
        format!("btn-invite-{}", "back"),
        format!("btn-invite-{}", "review"),
        format!("invite-{}", "count"),
        format!("Your private {}", "note"),
        format!("Cancel {}", "Invite"),
    ];
    for needle in &retired {
        assert!(
            !html_raw.contains(needle.as_str()),
            "a retired name is back in index.html — even a COMMENT counts, because a \
             source-text scanner cannot tell a comment from markup: `{needle}`"
        );
        assert!(
            !js_raw.contains(needle.as_str()),
            "a retired name is back in main.js — even a COMMENT counts: `{needle}`"
        );
    }
}

/// **RULING Q4 — THE CAP EXPLANATION IS DECIDED ONCE, AT OPEN, AND NEVER AGAIN.**
///
/// This is the one boundary at which the invite window used to MOVE.
/// `inviteRefresh()` runs on open AND after every mint, and it used to recompute
/// the cap and toggle the explanation line — so minting the TENTH invite made a
/// line APPEAR after activation, in direct contradiction of item 6's "nothing
/// appears ... on activation".
///
/// ⚠⚠ THIS SEAL EXISTS BECAUSE THE RUNTIME INSTRUMENT CANNOT REACH THE REAL
/// BOUNDARY, AND THAT LIMIT IS STATED RATHER THAN PAPERED OVER. Reaching it for
/// real needs TEN live invites, which needs a successful `invite_create`, which
/// needs a relay — and the desktop harness has no fixture relay (`ENG-0226`,
/// open). `f_k` therefore drives the adopt path with the cap latched and proves
/// the window does not move; what it CANNOT do is prove that no OTHER code path
/// re-shows the line. That is a structural fact, so it is sealed structurally:
/// the toggle exists EXACTLY ONCE and lives in the OPENER.
///
/// ⚠ MUST GO RED IF: a second toggle appears anywhere, or the one toggle moves
/// back into the refresh — which is precisely the shape of the retired defect.
#[test]
fn na0766_the_cap_line_is_decided_once_at_open() {
    let js = ui_file("main.js");
    // Assembled at runtime: this seal must not be satisfiable by its own prose.
    let toggle = format!("byId(\"invite-cap-{}\").classList.toggle", "full");

    assert_eq!(
        js.matches(toggle.as_str()).count(),
        1,
        "the cap explanation is toggled in EXACTLY ONE place — a second site is how it \
         starts appearing as a RESULT of activation again"
    );

    fn body<'a>(js: &'a str, sig: &str) -> &'a str {
        let a = js.find(sig).unwrap_or_else(|| panic!("`{sig}` not found"));
        let rest = &js[a..];
        let e = rest.find("\n}\n").expect("function ends");
        &rest[..e]
    }
    assert!(
        body(&js, "async function openInviteModal() {").contains(toggle.as_str()),
        "and that one place is the OPENER, so the decision is made once per window"
    );
    assert!(
        !body(&js, "async function inviteRefresh() {").contains(toggle.as_str()),
        "and it is NOT in the refresh, which runs again after every mint — that is the \
         exact code path by which the tenth invite used to make the window grow"
    );
}

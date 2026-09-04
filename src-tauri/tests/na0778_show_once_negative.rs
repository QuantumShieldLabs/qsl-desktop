//! NA-0778 (`D-0047`) -- THE SHOW-ONCE NEGATIVE ARM (STOP 003 sec 3.3's enumeration, ordered by
//! RULING_NA0778_004 R28). The one-time invite code is shown ONCE, in the creation dialog, and
//! never on the Invitations page. A page cannot be pinned NOT to show something by reading the
//! page; what CAN be pinned is the set of places a code can come from and land, as EXACT counts:
//!   (1) the front end asks for a code in exactly ONE place        `invoke("invite_create"`
//!   (2) the gateway hands a code out of exactly ONE command       `qsc::facade::invite_create(`
//!   (3) the code lands in exactly ONE DOM slot with TWO writers   adopt, and the reset
//!   (4) the slot is READ in exactly ONE place                     the copy link
//!   (5) the Invitations page's module and markup touch neither the slot nor the verb.
//! ⚠ MUST GO RED IF: a second mint call, a second gateway path, a second slot, a second writer or a
//! second reader appears -- each is a new emission site. Exact equalities, deliberately: a `>=`
//! would absorb precisely the regression this arm exists to catch. The red arm was shown by
//! planting a second `invoke("invite_create"` in a scratch copy of main.js (banked in STOP 004).

use std::fs;
use std::path::PathBuf;

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .join(rel);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}

/// (1) the front end mints in exactly one place.
#[test]
fn na0778_the_front_end_mints_in_exactly_one_place() {
    let js = repo_file("ui/main.js");
    assert_eq!(
        js.matches("invoke(\"invite_create\"").count(),
        1,
        "exactly ONE mint call in the front end -- a second is a second emission site"
    );
}

/// (2) the gateway hands a code out of exactly one command.
#[test]
fn na0778_the_gateway_emits_the_code_from_exactly_one_command() {
    let rs = repo_file("src-tauri/src/commands.rs");
    assert_eq!(
        rs.matches("qsc::facade::invite_create(").count(),
        1,
        "exactly ONE gateway path obtains a code from the engine"
    );
    assert_eq!(
        rs.matches("pub async fn invite_create(").count(),
        1,
        "and exactly ONE command carries it out"
    );
}

/// (3) + (4) the slot: one element, two writers (adopt and reset), one reader (the copy link).
#[test]
fn na0778_the_code_slot_has_two_writers_and_one_reader() {
    let js = repo_file("ui/main.js");
    let html = repo_file("ui/index.html");
    assert_eq!(
        html.matches("id=\"invite-code\"").count(),
        1,
        "ONE slot in the markup"
    );
    assert_eq!(
        js.matches("byId(\"invite-code\")").count(),
        3,
        "the slot is touched in exactly three places: the adopt writer, the reset writer, the copy reader"
    );
    assert_eq!(
        js.matches("box.textContent = code;").count(),
        1,
        "the adopt writer"
    );
    assert_eq!(
        js.matches("box.textContent = INVITE_SLOT_EMPTY;").count(),
        1,
        "the reset writer"
    );
    assert_eq!(
        js.matches("byId(\"invite-code\").textContent").count(),
        1,
        "the copy link is the ONE reader"
    );
}

/// (5) the Invitations page never touches the code or the verb -- the page reviews, it never shows.
#[test]
fn na0778_the_invitations_page_never_touches_the_code() {
    let js = repo_file("ui/main.js");
    let start = js
        .find("// ---- NA-0778")
        .expect("the invitations module exists");
    let end = js[start..]
        .find("// ---- NA-0755")
        .expect("it ends before the invite module")
        + start;
    let module = &js[start..end];
    // non-vacuity: the slice is the real module
    assert!(
        module.contains("function invitationsRender()"),
        "the renderer lives in the slice"
    );
    for needle in [
        "invite-code",
        "invite_create",
        "code-box",
        "INVITE_SLOT_EMPTY",
    ] {
        assert!(
            !module.contains(needle),
            "the page module must not touch `{needle}`"
        );
    }
    let html = repo_file("ui/index.html");
    let ps = html
        .find("id=\"pane-invitations\"")
        .expect("the pane exists");
    let pe = html[ps..]
        .find("id=\"pane-appearance\"")
        .expect("the next pane")
        + ps;
    let pane = &html[ps..pe];
    assert!(
        pane.contains("invitations-sent-rows"),
        "the slice is the real pane"
    );
    for needle in ["code-box", "invite-code"] {
        assert!(
            !pane.contains(needle),
            "the page markup must not carry `{needle}`"
        );
    }
}

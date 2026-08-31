//! NA-0774 (`D-0045`) — E5: A THROWN `identity_show` AND AN ABSENT IDENTITY ARE
//! DIFFERENT FACTS AND MUST NOT SHARE A SCREEN.
//!
//! Before this lane `refreshIdentityPane` swallowed any error from
//! `identity_show` and fell into its `!rec` branch, which reveals
//! `#identity-empty`: "No identity exists yet — finish setup to create one." A
//! user who HAS an identity was told they had none, and invited to an action
//! that is wrong for their actual state.
//!
//! ⚠⚠ WHY THIS IS A UNIT SEAL AND NOT A DRIVER FLOW, STATED PLAINLY BECAUSE IT
//! IS A REAL LIMIT: the error path is NOT DRIVEN END-TO-END anywhere. Making
//! `identity_show` throw needs the IPC layer, and NA-0768 measured that
//! `window.__TAURI__.core` is FROZEN — `invoke` is non-writable and
//! non-configurable, so a scenario cannot patch it at any delay. What these
//! seals prove is that BOTH BRANCHES EXIST and are DISTINCT in the shipped
//! bytes. What no test in this repository proves is the rendered result of a
//! real `identity_show` rejection.
//!
//! ⚠ Markup needles run on COMMENT-STRIPPED html. The comment that documents
//! this fix quotes both strings, so an unstripped needle would pass on its own
//! rationale — the `ENG-0235` hazard, and this file would be its next instance.

use std::fs;
use std::path::Path;

fn repo_file(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}

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

/// The error state is a REAL element in the markup, not a reused one, and its
/// copy names retry rather than setup.
#[test]
fn na0774_identity_error_element_exists_with_retry_copy() {
    let html = strip_html_comments(&repo_file("ui/index.html"));
    let line = html
        .lines()
        .find(|l| l.contains(r#"id="identity-read-error""#))
        .expect("the identity read-error element exists in the markup");
    assert!(
        line.contains("Couldn't read your identity."),
        "the error names the cause: {line}"
    );
    assert!(
        line.contains("Try again."),
        "the error names RETRY, which is the action that fits the cause: {line}"
    );
    // The whole point of the fix: this state must NOT send the user to setup.
    assert!(
        !line.contains("finish setup"),
        "the error state must never invite setup — that is the falsehood: {line}"
    );
    assert!(
        line.contains("hidden"),
        "it arrives hidden, like the absent state beside it: {line}"
    );
}

/// The two causes are still two elements — the fix is not a reworded single state.
#[test]
fn na0774_absent_and_error_are_distinct_elements() {
    let html = strip_html_comments(&repo_file("ui/index.html"));
    assert!(
        html.contains(r#"id="identity-empty""#),
        "the ABSENT state survives unchanged"
    );
    assert!(
        html.contains(r#"id="identity-read-error""#),
        "the ERROR state exists"
    );
    let empty = html
        .lines()
        .find(|l| l.contains(r#"id="identity-empty""#))
        .expect("absent state");
    assert!(
        empty.contains("No identity exists yet"),
        "the absent state keeps its own copy: {empty}"
    );
}

/// The JS distinguishes the causes: a throw is recorded, not swallowed into
/// the same branch as a null record.
#[test]
fn na0774_refresh_identity_pane_separates_throw_from_absent() {
    let js = repo_file("ui/main.js");
    let start = js
        .find("async function refreshIdentityPane()")
        .expect("refreshIdentityPane exists");
    let body = &js[start..];
    let body = &body[..body.find("\n}\n").expect("function body end")];

    assert!(
        body.contains("readFailed = true"),
        "a thrown identity_show is RECORDED, not swallowed"
    );
    assert!(
        body.contains(r#"byId("identity-read-error")"#),
        "the pane reaches the error element"
    );
    // Both states are driven from the SAME condition, so they cannot both show
    // and cannot both hide.
    assert!(
        body.contains(r#"empty.classList.toggle("hidden", readFailed)"#),
        "the absent state hides exactly when the read failed"
    );
    assert!(
        body.contains(r#"readError.classList.toggle("hidden", !readFailed)"#),
        "the error state shows exactly when the read failed"
    );
    // And the success path clears BOTH, or a stale error would sit above a
    // populated pane.
    assert!(
        body.contains(r#"readError.classList.add("hidden")"#),
        "the success path clears the error state too"
    );
}

//! NA-0768 (`D-1409`, `RULING_012`) — THE REPAINT SIGNAL, PINNED AT THE SOURCE.
//!
//! ## WHAT WENT WRONG, SO THE PIN IS READ AS GUARDING SOMETHING REAL
//! On the E4 completing path the fan-out consumes the peer's A2 and commits the inviter's
//! session — and then `invite_finish` returns **`Ok(false)`** (`invite/mod.rs:1649`), because
//! `Ok(true)` is reserved for the selected invite-RESP path, which an inviter never has. So
//! `marks.finished` stayed 0 while a contact had genuinely gone live, `recordScanOutcome`'s
//! gate never fired, and the screen kept reading "Connecting…" over a live session. Three AWS
//! flights showed it; STOP 012 diagnosed it to file:line.
//!
//! ## ⚠⚠ WHY EVERY NEEDLE IS ASSEMBLED AT RUNTIME AND EVERY COMMENT IS STRIPPED
//! The fix's own source comments name `marks.changed`, the gate, and `refreshContacts` many
//! times over. A seal that searched the raw file would be satisfied **by the explanation of the
//! thing it is checking for** — `ENG-0235`, the hazard where a seal's own prose satisfies the
//! seal, and this repo has been bitten by its mirror image too. So: comments are stripped
//! first, and no needle appears as a literal in this file.
//!
//! ## WHAT THIS CANNOT DO
//! It is a SOURCE pin. It cannot prove the screen repaints — only that the signal the repaint
//! depends on is computed from the state and that the gate reads it. Proving the pixels is the
//! gui-driver's job, and no scenario can reach a completed handshake without a fixture relay
//! (`ENG-0226`). **The flight is the acceptance instrument; this pin is the regression guard.**

use std::fs;
use std::path::PathBuf;

fn ui_file(name: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("ui");
    p.push(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Strip `//` line comments and `/* */` blocks. Not a JS parser: it is deliberately
/// conservative and is only ever used to make a seal HARDER to satisfy, never easier.
fn strip_js_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let b = src.as_bytes();
    let (mut i, n) = (0usize, b.len());
    let (mut in_line, mut in_block) = (false, false);
    while i < n {
        if in_line {
            if b[i] == b'\n' { in_line = false; out.push('\n'); }
        } else if in_block {
            if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' { in_block = false; i += 1; }
        } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'/' {
            in_line = true; i += 1;
        } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
            in_block = true; i += 1;
        } else {
            out.push(b[i] as char);
        }
        i += 1;
    }
    out
}

/// The body of a named function, from its declaration to the next top-level `\n}`.
fn fn_body<'a>(js: &'a str, decl: &str) -> &'a str {
    let a = js.find(decl).unwrap_or_else(|| panic!("declaration not found: {decl}"));
    let rest = &js[a..];
    let b = rest.find("\n}").unwrap_or_else(|| panic!("no terminator after: {decl}"));
    &rest[..b]
}

#[test]
fn na0768_r1_the_scan_counts_a_status_change_not_only_the_verbs_bool() {
    let js = strip_js_comments(&ui_file("main.js"));
    let body = fn_body(&js, &format!("async function finishScan{}(marks)", "Class"));

    // The verb's own bool is NOT enough — the scan must ALSO observe the state.
    let status_call = format!("invoke(\"connect_{}\"", "status");
    let reads = body.matches(status_call.as_str()).count();
    assert!(
        reads >= 2,
        "the scan must read connect_status TWICE per attempted row — once before the finish and \
         once after — because the verb returns Ok(false) on the completing path and cannot be \
         the repaint signal. Found {reads} read(s)."
    );

    // The counter must be incremented from a COMPARISON, not unconditionally.
    let counter = format!("marks.{} += 1", "changed");
    assert!(
        body.contains(counter.as_str()),
        "the scan must increment the changed-counter when a row's state moves"
    );
    let compare = format!("after.{} !== st.{}", "state", "state");
    assert!(
        body.contains(compare.as_str()),
        "the changed-counter must be driven by an EQUALITY comparison of the extracted state \
         before and after, never by the attempt itself"
    );
}

#[test]
fn na0768_r2_the_repaint_gate_reads_the_change_counter() {
    let js = strip_js_comments(&ui_file("main.js"));
    let finished = format!("marks.{} > 0", "finished");
    let changed = format!("marks.{} > 0", "changed");
    let gate = format!("if ({finished} || {changed})");
    assert!(
        js.contains(gate.as_str()),
        "the repaint gate must fire on EITHER a completed handshake OR an observed state change"
    );

    // ⚠ NEGATIVE CONTROL: the pre-fix gate must be GONE, not merely accompanied. Without this
    // the seal would pass on a file that kept the old narrow gate beside a new wide one.
    let old_gate = format!("if ({finished}) {{", finished = finished);
    assert!(
        !js.contains(old_gate.as_str()),
        "the NARROW gate must not survive anywhere: it is the defect"
    );
}

#[test]
fn na0768_r3_unlock_repaints_contacts_after_its_scan() {
    let js = strip_js_comments(&ui_file("main.js"));
    let body = fn_body(&js, &format!("async function enter{}()", "Main"));
    let scan = format!("relayScan({{ source: \"{}\"", "unlock");
    let repaint = format!("refresh{}()", "Contacts");
    let a = body.find(scan.as_str()).expect("the unlock scan must still run");
    let b = body
        .find(repaint.as_str())
        .expect("unlock must repaint contacts: a session that completed while locked renders \
                 stale on the first screen the user sees");
    assert!(
        b > a,
        "the repaint must follow the unlock scan, not precede it — before the scan it would \
         publish the pre-scan state and undo its own purpose"
    );
}

//! Desktop lock follow-up F1 and T5 (SR-15 of PR #63, findings L1, L6-a and L4): text seals over
//! ui/main.js. Neither is ignored, so both run in the required rust job.
//!
//! F1 (delta symbol F1-SEALED): the boot's seal must be load-bearing. PR #63 put
//! `try { await invoke("lock_now"); } catch (_) {}` before `await route();`, and the catch
//! swallowed an IPC-level rejection, so route() drew the unlock screen over an unlocked engine
//! (fail OPEN; the SR-15 reproduced it with lock_now unregistered). RED at 8b6f18f7: there is no
//! sealed flag. GREEN when a rejected seal writes one fixed line, draws no screen and returns.
//!
//! T5 (delta symbol T5-ARM-LEAD): the wipe ARM handler printed a bare error code for anything
//! but an out-of-range limit (R-17: never a bare code). RED at 8b6f18f7: the handler uses
//! mapErr. GREEN with the disarm handler's plainError lead.

use std::path::PathBuf;

fn main_js() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("ui/main.js");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// The text from `start` up to the first `end` after it.
fn region<'a>(src: &'a str, start: &str, end: &str) -> &'a str {
    let i = src
        .find(start)
        .unwrap_or_else(|| panic!("premise moved: {start:?} not found in ui/main.js"));
    let rest = &src[i..];
    let j = rest[start.len()..]
        .find(end)
        .unwrap_or_else(|| panic!("premise moved: {end:?} not found after {start:?}"));
    &rest[..start.len() + j]
}

fn pos(hay: &str, needle: &str, what: &str) -> usize {
    hay.find(needle)
        .unwrap_or_else(|| panic!("{what}: {needle:?} not found in the boot block"))
}

const FIXED_LINE: &str = "This window could not be secured. Quit and relaunch.";

#[test]
fn f1_boot_draws_nothing_unless_the_seal_resolved() {
    let js = main_js();
    let boot = region(&js, "// ---- boot ---", "})();");

    let flag = pos(
        boot,
        "let sealed = false;",
        "the boot must carry a sealed flag",
    );
    let seal = pos(boot, r#"await invoke("lock_now");"#, "the boot must seal");
    let set = pos(
        boot,
        "sealed = true;",
        "sealed may be set only after the seal resolved",
    );
    let gate = pos(boot, "if (!sealed)", "a failed seal must be handled");
    let draw = pos(
        boot,
        "await route();",
        "premise moved: the boot no longer routes",
    );
    assert!(
        flag < seal && seal < set && set < gate && gate < draw,
        "order must be: flag, awaited lock_now, sealed = true, if (!sealed), route() -- got {flag} {seal} {set} {gate} {draw}"
    );

    let branch = &boot[gate..draw];
    assert!(
        branch.contains(FIXED_LINE),
        "a failed seal must say: {FIXED_LINE}"
    );
    assert!(
        branch.contains("return;"),
        "a failed seal must return before route()"
    );
    for banned in [
        "show(",
        "route(",
        "showUnlockScreen(",
        "\"error\"",
        "danger",
    ] {
        assert!(
            !branch.contains(banned),
            "the failed-seal branch must draw no screen and use no danger style: found {banned:?}"
        );
    }
    assert!(
        !js.contains("lock_now has no Err arm"),
        "the comment is false at the IPC boundary and must go"
    );

    // A guard, not a delta: this lane does not move the seal into showUnlockScreen (F3 is
    // carried to F13 -- an extra lock_now rewrites the debug log's lock cause).
    let sus = region(&js, "async function showUnlockScreen(next) {", "\n}\n");
    assert!(
        !sus.contains("lock_now"),
        "showUnlockScreen must not seal in this lane (F3 is ruled to F13)"
    );
}

#[test]
fn t5_wipe_arm_handler_maps_a_refusal_to_plain_words() {
    let js = main_js();
    let arm = region(
        &js,
        r#"byId("btn-wipe-arm").addEventListener("click""#,
        r#"byId("btn-wipe-disarm")"#,
    );
    assert!(
        arm.contains("plainError(") && !arm.contains("mapErr("),
        "the arm handler must map errors with plainError, never mapErr's bare code"
    );
    assert!(
        arm.contains(r#""That setting wasn't changed.""#),
        "the arm handler's lead must be the disarm handler's: That setting wasn't changed."
    );
    assert!(
        arm.contains(r#"wipe_limit_out_of_bounds: "Limit must be between 1 and 100.""#),
        "the out-of-range mapping must be kept"
    );
}

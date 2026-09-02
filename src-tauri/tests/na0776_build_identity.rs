//! NA-0776 (spec v2 sec 3.4) -- ENG-0275: app_info reports WHICH BUILD this is.
//! Arms per the cold read's MAJOR-6 and MAJOR-10: 40-hex-or-unknown, the clean-tree
//! equality that reds on a STALE stamp, and a forced-absent arm that is actually
//! drivable because the fallback is a pure function rather than an `env!`.

use qsl_desktop_app::commands::{app_info, build_commit_or_unknown};
use std::process::Command;

fn is_40_hex(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// A1 -- the field is either a 40-hex commit or exactly "unknown". Nothing else, ever:
/// no empty string, no "HEAD", no fabricated value.
#[test]
fn build_commit_is_40_hex_or_exactly_unknown() {
    let v = app_info().build_commit;
    assert!(
        is_40_hex(v) || v == "unknown",
        "build_commit is {v:?} -- neither a 40-hex commit nor the literal \"unknown\""
    );
    assert!(!v.is_empty(), "build_commit is empty: an absent value must read \"unknown\"");
}

/// A2 -- THE STALE-STAMP ARM. The stamp must equal the commit this tree is on. If
/// build.rs failed to re-run after a commit, the binary carries an older sha and this
/// goes red -- which is the whole point of declaring the rerun triggers.
#[test]
fn stamped_commit_equals_head() {
    let out = Command::new("git").args(["rev-parse", "HEAD"]).output();
    let head = match out {
        Ok(o) if o.status.success() => String::from_utf8(o.stdout).unwrap().trim().to_string(),
        // No git here: the contract is then that the field says "unknown", and A1
        // already pins the shape. Stated rather than skipped silently.
        _ => {
            assert_eq!(app_info().build_commit, "unknown",
                "git is unavailable, so the stamp must be \"unknown\"");
            return;
        }
    };
    assert_eq!(
        app_info().build_commit, head,
        "STALE STAMP: the binary reports a different commit than the tree is on"
    );
}

/// A3 -- THE FORCED-ABSENT ARM, drivable because the fallback is pure. With `env!` this
/// arm could not exist: an absent variable is a compile error, not a red assertion.
#[test]
fn an_absent_stamp_reads_unknown() {
    assert_eq!(build_commit_or_unknown(None), "unknown");
}

/// A3b -- a malformed stamp is not believed. This is the branch that keeps a truncated
/// or garbage value from reaching a flight record as if it were a commit.
#[test]
fn a_malformed_stamp_reads_unknown() {
    for bad in ["", "HEAD", "0123456789", "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
                "0123456789012345678901234567890123456789a"] {
        assert_eq!(
            build_commit_or_unknown(Some(bad)), "unknown",
            "a malformed stamp {bad:?} was accepted as a commit"
        );
    }
    // and a well-formed one passes through unchanged
    let good = "0123456789abcdef0123456789abcdef01234567";
    assert_eq!(build_commit_or_unknown(Some(good)), good);
}

/// The two fields MAJOR-6 argued out of existence stay out: a reader must not find a
/// `dirty` or build-time field on the DTO and believe it.
#[test]
fn no_dirty_flag_and_no_build_timestamp_on_the_dto() {
    let v = serde_json::to_value(app_info()).unwrap();
    let obj = v.as_object().expect("AppInfoDto serializes as an object");
    for absent in ["dirty", "build_dirty", "build_utc", "build_time", "built_at"] {
        assert!(!obj.contains_key(absent),
            "AppInfoDto carries {absent:?}: MAJOR-6 ruled it out as believed-and-wrong");
    }
    assert!(obj.contains_key("build_commit"), "build_commit is missing from the DTO");
}

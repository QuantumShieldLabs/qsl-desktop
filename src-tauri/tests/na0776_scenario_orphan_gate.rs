//! NA-0776 (spec v2 sec 3.7 / cold read BLOCKER-3b) -- THE ORPHAN GATE.
//!
//! Scenarios are NOT auto-discovered: `gui_driver.rs` builds the path from a NAME passed
//! in, so a scenario JSON dropped into the directory with no corresponding dispatch call
//! NEVER RUNS -- and nothing catches it. `scripts/ci/test_inventory.sh` cannot: it allows
//! growth and fires only on shrinkage, and a scenario that never runs is not a test at
//! all, so it never enters the enumeration. An unrun acceptance arm reports nothing and
//! looks exactly like a passing suite. That is the vacuous-pass family this house names.
//!
//! ⚠ THE GATE MUST COUNT BOTH DISPATCH FORMS. Measured while building it: sixteen
//! scenarios dispatch via `run_scenario("...")` and ONE -- `f_m_liveness_tick` -- via
//! `run_scenario_with_env("...", ...)`. A needle keying only on the first form reports a
//! FALSE ORPHAN and reds a correct tree. The union is the instrument.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn scenario_files() -> BTreeSet<String> {
    let dir = repo_root().join("src-tauri/tests/harness/scenarios");
    std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("scenarios dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter_map(|n| n.strip_suffix(".json").map(str::to_owned))
        .collect()
}

/// Both dispatch forms, taken as a UNION. Deliberately string-scanned rather than
/// parsed: the property is "a call exists in the source a human will read", and a
/// scanner that agrees with the eye is the right instrument here.
fn dispatched_names() -> BTreeSet<String> {
    let src = std::fs::read_to_string(repo_root().join("src-tauri/tests/gui_driver.rs"))
        .expect("gui_driver.rs");
    let mut out = BTreeSet::new();
    // ⚠ WHITESPACE-TOLERANT BY NECESSITY, and this was found by the gate reddening on a
    // correct tree: `run_scenario_with_env` is often written multi-line, with the name on
    // the line AFTER the paren. A needle demanding `("` immediately adjacent misses those
    // and reports a FALSE ORPHAN -- the same class of too-narrow instrument this gate
    // exists to catch, met in the gate itself.
    for marker in ["run_scenario", "run_scenario_with_env"] {
        let mut rest = src.as_str();
        while let Some(i) = rest.find(marker) {
            rest = &rest[i + marker.len()..];
            let after = rest.trim_start();
            if let Some(inner) = after.strip_prefix('(') {
                let inner = inner.trim_start();
                if let Some(q) = inner.strip_prefix('"') {
                    if let Some(end) = q.find('"') {
                        out.insert(q[..end].to_string());
                    }
                }
            }
        }
    }
    out
}

#[test]
fn every_scenario_is_dispatched_and_every_dispatch_has_a_scenario() {
    let files = scenario_files();
    let calls = dispatched_names();

    let orphans: Vec<_> = files.difference(&calls).cloned().collect();
    let dangling: Vec<_> = calls.difference(&files).cloned().collect();

    assert!(
        orphans.is_empty(),
        "ORPHAN SCENARIO(S) -- these JSON files exist but NOTHING RUNS THEM, and a suite \
         with an unrun acceptance arm looks exactly like a passing one: {orphans:?}"
    );
    assert!(
        dangling.is_empty(),
        "DANGLING DISPATCH -- gui_driver.rs runs scenario(s) with no JSON file, which \
         fails only when that test is actually executed: {dangling:?}"
    );
    assert_eq!(files, calls, "the two sets must be equal, not merely non-empty");
    // non-vacuity: an empty directory would satisfy set equality trivially
    assert!(files.len() >= 17, "only {} scenarios found -- the gate may be looking in the \
        wrong place, which would make its equality vacuous", files.len());
}

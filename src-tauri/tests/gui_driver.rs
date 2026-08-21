//! NA-0701 — THE GUI INPUT DRIVER (D636 as amended; spine D-1341, desktop D-0026).
//!
//! M = 6 `#[ignore]`-marked tests, one per flow. Ignored-by-default is the honest
//! local shape: the rows are VISIBLE as "ignored" in every plain `cargo test`,
//! never fake-green. Their real execution is
//! `cargo test --test gui_driver -- --ignored --test-threads=1` (SERIALIZED —
//! one app instance at a time, the NA-0700 C4 race lesson), which is exactly
//! what the `gui-driver` CI job runs.
//!
//! Each test invokes the python harness runner (std::process::Command — no new
//! dependencies, src-tauri/Cargo.toml deliberately untouched per A1.5) for its
//! scenario file and asserts the verdict JSONL with the consumer contract
//! validated both directions at Phase 0 (STOP_NA0701_007 P0.5): every step row
//! carries "verdict":"PASS", and the terminal row carries "result":"PASS" with
//! a "steps" count reconciling the step-row count.
//!
//! QSLD_CONTINUE_ON_FAIL (the perturbation-measurement facility, R171 3.4) is
//! STRUCTURALLY unreachable through this wrapper: the env var is removed from
//! the child environment below, so no CI or local `cargo test` invocation can
//! weaken a real-flow run into continue-on-fail mode.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/src-tauri (the only workspace member).
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

/// Minimal JSONL field extractor — the runner emits flat json.dumps rows, so a
/// key search with string/number handling is exact for this producer (the
/// format pair is co-designed and was validated against samples both ways).
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":");
    let i = line.find(&pat)? + pat.len();
    let rest = line[i..].trim_start();
    if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped.find('"')?;
        Some(&stripped[..end])
    } else {
        let end = rest.find([',', '}']).unwrap_or(rest.len());
        Some(rest[..end].trim())
    }
}

fn run_scenario(scenario: &str) {
    let root = repo_root();
    let runner = root.join("src-tauri/tests/harness/runner.py");
    let scenario_file = root.join(format!("src-tauri/tests/harness/scenarios/{scenario}.json"));
    assert!(runner.is_file(), "runner missing: {}", runner.display());
    assert!(
        scenario_file.is_file(),
        "scenario missing: {}",
        scenario_file.display()
    );
    let out = Command::new("python3")
        .arg(&runner)
        .arg(&scenario_file)
        .current_dir(&root)
        .env_remove("QSLD_CONTINUE_ON_FAIL") // R171 3.4: not reachable from here
        .output()
        .expect("failed to spawn the harness runner (python3 required)");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let run_dir = stdout
        .lines()
        .find_map(|l| l.strip_prefix("RUN_DIR="))
        .unwrap_or_else(|| panic!("runner printed no RUN_DIR; stdout:\n{stdout}"))
        .to_string();
    let verdict_path = PathBuf::from(&run_dir).join("verdict.jsonl");
    let text = std::fs::read_to_string(&verdict_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", verdict_path.display()));
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "verdict file needs >=1 step row + terminal row, got {} lines",
        lines.len()
    );
    let (steps, terminal) = lines.split_at(lines.len() - 1);
    let terminal = terminal[0];
    let mut failed: Vec<String> = Vec::new();
    for l in steps {
        match field(l, "verdict") {
            Some("PASS") => {}
            _ => failed.push((*l).to_string()),
        }
    }
    let result = field(terminal, "result").unwrap_or("<missing>");
    let n: Option<usize> = field(terminal, "steps").and_then(|s| s.parse().ok());
    assert!(
        failed.is_empty(),
        "scenario {scenario}: {} step row(s) not PASS (evidence: {}):\n{}",
        failed.len(),
        run_dir,
        failed.join("\n")
    );
    assert_eq!(
        n,
        Some(steps.len()),
        "scenario {scenario}: terminal steps count does not reconcile ({})",
        run_dir
    );
    assert_eq!(
        result, "PASS",
        "scenario {scenario}: terminal result={result} ({run_dir})"
    );
    assert!(
        out.status.success(),
        "scenario {scenario}: runner exit={:?} with an all-PASS verdict file — investigate ({run_dir})",
        out.status.code()
    );
}

#[test]
#[ignore]
fn na0701_gui_a_read_census() {
    run_scenario("f_a_read_census");
}

#[test]
#[ignore]
fn na0701_gui_b_onboarding() {
    run_scenario("f_b_onboarding");
}

#[test]
#[ignore]
fn na0701_gui_c_lock_unlock() {
    run_scenario("f_c_lock_unlock");
}

#[test]
#[ignore]
fn na0701_gui_d_settings_persistence() {
    run_scenario("f_d_settings_persistence");
}

#[test]
#[ignore]
fn na0701_gui_e_erase_ceremony() {
    run_scenario("f_e_erase_ceremony");
}

#[test]
#[ignore]
fn na0701_gui_f_menu_event_substitutes() {
    run_scenario("f_f_menu_event_substitutes");
}

// NA-0750 (D-0031): the on-screen half of W1. The harness proves SHAPE and PRESENCE —
// 30 ASCII digits and 64 lowercase hex by anchored regex — plus the ABSENCE of the
// retired grouped form. The VALUE is proven by the in-crate test in commands.rs; the
// harness cannot assert it, because a scenario compares against literals and the
// harness vault's identity is generated fresh on every run (R365 §3).
//
// ⚠ CLAIM BOUNDARY: this proves the value's shape ON SCREEN, never its legibility —
// `textContent` reads the same whether the element is clipped or not.
#[test]
#[ignore]
fn na0750_gui_g_fingerprint_two_tier() {
    run_scenario("f_g_fingerprint_two_tier");
}

// NA-0752 (D-0033): the status footer reports the desk's typed state. The
// harness drives the TWO states it genuinely can — no relay configured, and a
// relay configured — and asserts the footer's EXACT ruled copy by equality on
// extracted text. The three undrivable rows are presence-asserted in
// design_polish.rs (seal F1b), which says in its own doc that presence is not
// behaviour.
#[test]
#[ignore]
fn na0752_gui_h_status_footer_truth() {
    run_scenario("f_h_status_footer_truth");
}

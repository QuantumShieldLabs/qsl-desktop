//! REVIEW-desktop RD-05: the README is a claim surface (the D611 precedent in server_pane.rs).
//! While the front end carries a background scan, the README must not say the app never touches
//! the network on a timer, and it must say that it does. Delta symbol: README.md's network-contact
//! bullet (the retired sentence "Nothing reaches the network at launch, in the background, or on a
//! timer.").

use std::path::PathBuf;

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Line breaks and runs of spaces collapse to one space, so a re-wrapped sentence is still found.
fn flat(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn rd05_readme_states_the_background_scan_it_ships() {
    let js = repo_file("ui/main.js");
    let readme = flat(&repo_file("README.md"));
    // The instrument's premise, checked rather than assumed: the tick and the unlock scan exist.
    assert!(
        js.contains(r#"relayScan({ source: "tick""#)
            && js.contains(r#"relayScan({ source: "unlock""#),
        "premise moved: the front end no longer carries the tick or the unlock scan"
    );
    for retired in [
        "Nothing reaches the network at launch, in the background, or on a timer.",
        "Every connection is the direct result of a button you pressed.",
    ] {
        assert!(!readme.contains(retired), "README still says: {retired}");
    }
    assert!(
        readme.contains("checks in the background"),
        "README must say the app checks in the background while unlocked"
    );
}

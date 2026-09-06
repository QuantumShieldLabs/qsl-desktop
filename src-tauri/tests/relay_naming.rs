//! NA-0683 / D618 — the RELAY naming guard.
//!
//! The operator ruled 2026-07-27 that the user-facing term is **Relay**, never
//! **Server**: "relay" teaches the security model — a dumb pipe forwarding
//! opaque bytes, not a trust-holding service — matches the protocol docs and the
//! invite system, and suits the audience. This file is that ruling's guard.
//!
//! It is the idiom `server_pane.rs::claim_discipline_five_surfaces_swept`
//! already uses for retired claims: pin what each surface now says, and assert
//! that what it used to say stays gone. A rename with no guard is a rename that
//! comes back one edit at a time.
//!
//! ⚠ THIS FILE DELIBERATELY SPELLS THE RETIRED WORD. A guard that cannot name
//! what it forbids cannot be read, and the property has to live somewhere a
//! human will look. The lane's sweep instrument therefore counts these
//! occurrences, and the acceptance arithmetic states them explicitly rather than
//! absorbing them — see D618 §5.
//!
//! ⚠ IDENTIFIERS ARE NOT IN SCOPE AND MUST NOT BE "FIXED" BY A LATER LANE:
//! `data-pane="server"`, `#pane-server`, `.server-form`, `.srv-sect`,
//! `commitServerSettings`, `RelayServerInfoOutcome`, `GET /v1/server-info` and
//! this repo's `server_pane.rs` all stay. The ruling is about what a user reads.

use std::fs;
use std::path::Path;

fn repo_file(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}
fn ui(name: &str) -> String {
    repo_file(&format!("ui/{name}"))
}

#[test]
fn the_pane_is_named_relay_everywhere_it_is_shown() {
    let html = ui("index.html");
    // The nav item's TEXT is renamed; its data-pane KEY is not -- the key is an
    // identifier that main.js and three tests match on.
    assert!(
        html.contains(r#"<button data-pane="server" class="cat">Relay</button>"#),
        "the settings nav item must read Relay while keeping its data-pane key"
    );
    assert!(
        html.contains("<h2>Relay</h2>"),
        "the pane heading must read Relay"
    );
    // NA-0779 (18c L6): the status bar's static default -- state words, never an address.
    assert!(
        html.contains("Relay: not configured &middot; Vault: unlocked &middot; Auto-lock: 60 min"),
        "the main-window status bar's default must read the relay as 'not configured'"
    );
}

#[test]
fn the_status_and_result_copy_reads_relay() {
    let js = ui("main.js");
    for needle in [
        "const STATUS_FOOTER_NO_RELAY = \"not configured\";",
        r#"docRow("Relay version""#,
        "Couldn't reach the relay",
        "This relay presented a certificate",
        "the relay's certificate",
    ] {
        assert!(
            js.contains(needle),
            "renamed user-facing copy missing: {needle}"
        );
    }
}

#[test]
fn the_about_pane_slice_string_reads_relay_connectivity() {
    // Rendered by main.js's About pane via app_info().slice -- a user-facing
    // string that lives in Rust, which is why a sweep of ui/ alone would miss it.
    let cmds = repo_file("src-tauri/src/commands.rs");
    assert!(
        cmds.contains("B (relay connectivity:"),
        "app_info's slice string must say relay connectivity"
    );
}

#[test]
fn the_readme_reads_relay() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("Settings › Relay pane"),
        "the README must name the Settings › Relay pane"
    );
    assert!(
        readme.contains("relay configuration"),
        "the README status heading must say relay configuration"
    );
}

#[test]
fn the_retired_server_wording_stays_gone() {
    // ⚠ NEGATIVE PINS. These are the exact strings NA-0683 replaced. A file that
    // merely LOST its old assertions would document nothing and would not notice
    // the word coming back, so the removal itself is asserted.
    let html = ui("index.html");
    let js = ui("main.js");
    let cmds = repo_file("src-tauri/src/commands.rs");
    let readme = repo_file("README.md");

    for banned in [
        ">Server</button>",
        "<h2>Server</h2>",
        "No server configured",
    ] {
        assert!(
            !html.contains(banned),
            "retired wording back in index.html: {banned}"
        );
    }
    for banned in [
        "No server configured",
        "Settings › Server",
        r#"docRow("Server version""#,
        "Couldn't reach the server",
        "This server presented",
        "the server's certificate",
    ] {
        assert!(
            !js.contains(banned),
            "retired wording back in main.js: {banned}"
        );
    }
    // ⚠ PIN THE STRING, NOT THE WORD. A bare `!cmds.contains("server
    // connectivity")` failed on its first run against the CORRECT tree: the
    // phrase also appears in the section comment at commands.rs:319, which is
    // internal prose the ruling deliberately leaves alone. A needle that cannot
    // tell a rendered string from a comment about it is testing the mechanism
    // rather than the property.
    assert!(
        !cmds.contains(r#"slice: "B (server connectivity:"#),
        "app_info's slice string is back to server connectivity"
    );
    for banned in ["Settings › Server pane", "server configuration"] {
        assert!(
            !readme.contains(banned),
            "retired wording back in README.md: {banned}"
        );
    }
}

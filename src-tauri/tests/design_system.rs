//! D596 design-system source disciplines (additive; the slice-A test files
//! are byte-frozen). These pin the token layer, the button tiers, the
//! display-name binding, and the Appendix A verbatim copy so a later edit
//! cannot silently drift out of the system. D597 item 3 removed the
//! common-passwords check (the ONE sanctioned amendment of this file);
//! the round-2 pins live in tests/design_round2.rs.

use std::fs;
use std::path::Path;

fn ui_file(name: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ui")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}

fn manifest_file(name: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    fs::read_to_string(&p).unwrap_or_else(|_| panic!("read {}", p.display()))
}

/// style.css split into (the :root token block, everything outside it).
fn split_root_block(css: &str) -> (String, String) {
    let start = css
        .find(":root {")
        .expect("style.css must define the :root token block");
    let end = css[start..].find('}').expect("unterminated :root block") + start;
    let root = css[start..=end].to_string();
    let mut outside = String::new();
    outside.push_str(&css[..start]);
    outside.push_str(&css[end + 1..]);
    (root, outside)
}

/// Item 6: the display name binds to the window title (and About via
/// APP_DISPLAY_NAME); the identifier is the data-dir anchor and must never
/// change (a D596 STOP condition).
#[test]
fn display_name_and_identifier_binding() {
    let conf = manifest_file("tauri.conf.json");
    assert!(
        conf.contains(r#""title": "QuantumShield Chat""#),
        "window title must be QuantumShield Chat"
    );
    assert!(
        conf.contains(r#""identifier": "org.quantumshieldlabs.qsldesktop""#),
        "the identifier must not change (it anchors the app data dir)"
    );
    let html = ui_file("index.html");
    assert!(
        html.contains("<title>QuantumShield Chat</title>"),
        "the static page title must match the display name"
    );
}

/// Item 1: every font-size outside the :root token block references the
/// type scale (no bespoke sizes per screen).
#[test]
fn font_sizes_only_from_scale_tokens() {
    let (_, outside) = split_root_block(&ui_file("style.css"));
    for line in outside.lines() {
        if let Some(idx) = line.find("font-size:") {
            let value = &line[idx..];
            assert!(
                value.contains("var(--fs-"),
                "font-size outside the scale tokens: `{}`",
                line.trim()
            );
        }
    }
}

/// Item 2: every padding/margin/gap declaration outside :root uses the
/// spacing scale (or 0/auto) — one rhythm for cards, fields, and buttons.
#[test]
fn spacing_only_from_scale_tokens() {
    let (_, outside) = split_root_block(&ui_file("style.css"));
    let props = [
        "padding",
        "padding-left",
        "padding-right",
        "padding-top",
        "padding-bottom",
        "margin",
        "margin-left",
        "margin-right",
        "margin-top",
        "margin-bottom",
        "gap",
        "row-gap",
        "column-gap",
    ];
    for chunk in outside.split([';', '{', '}']) {
        let c = chunk.trim();
        for prop in props {
            if let Some(rest) = c.strip_prefix(prop) {
                let rest = rest.trim_start();
                if let Some(value) = rest.strip_prefix(':') {
                    for token in value.split_whitespace() {
                        assert!(
                            token == "0" || token == "auto" || token.starts_with("var(--sp-"),
                            "spacing value off the scale in `{c}` (token `{token}`)"
                        );
                    }
                }
            }
        }
    }
}

/// Item 4: color literals live ONLY in the :root token block — the accent
/// discipline (one accent; the red family separate; status colors named).
#[test]
fn colors_only_in_token_block() {
    let (_, outside) = split_root_block(&ui_file("style.css"));
    let bytes = outside.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'#' {
            let hex_run = bytes[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_hexdigit())
                .count();
            assert!(
                hex_run < 3,
                "hex color literal outside :root near byte {i}: `{}`",
                &outside[i..(i + 9).min(outside.len())]
            );
        }
    }
    assert!(
        !outside.contains("rgb("),
        "rgb() literal outside the :root token block"
    );
    assert!(
        !outside.contains("rgba("),
        "rgba() literal outside the :root token block"
    );
}

/// Item 3: every button in the markup carries exactly one action tier
/// (primary / secondary / danger) OR is one of the two navigation roles
/// (rail-btn / cat), never both and never neither.
#[test]
fn every_button_is_tiered_or_nav() {
    let html = ui_file("index.html");
    let mut rest = html.as_str();
    let mut seen = 0;
    while let Some(pos) = rest.find("<button") {
        rest = &rest[pos..];
        let tag_end = rest.find('>').expect("unterminated button tag");
        let tag = &rest[..tag_end];
        let classes: Vec<&str> = tag
            .split_once("class=\"")
            .map(|(_, after)| after.split('"').next().unwrap_or(""))
            .unwrap_or("")
            .split_whitespace()
            .collect();
        let tiers = ["primary", "secondary", "danger"]
            .iter()
            .filter(|t| classes.contains(&&***t))
            .count();
        let nav = classes.contains(&"rail-btn") || classes.contains(&"cat");
        assert!(
            (tiers == 1 && !nav) || (tiers == 0 && nav),
            "button is not exactly one tier or one nav role: `{tag}`"
        );
        seen += 1;
        rest = &rest[tag_end..];
    }
    assert!(
        seen >= 15,
        "expected the full button inventory, found {seen}"
    );
}

/// Appendix A verbatim copy (claim-discipline binding). If one of these
/// strings drifts, the approved wording drifted.
#[test]
fn appendix_a_copy_verbatim() {
    let html = ui_file("index.html");
    for needle in [
        "Length matters most — a few random words beat a short complex password.",
        "You don't need to write this down — view it anytime in Settings.",
        "Requires your passphrase. Permanently erases this vault — this cannot be undone.",
        ">Destroy vault</h3>",
        "Type <code>destroy my vault</code> to confirm",
        ">Destroy permanently</button>",
        "What should this device call you?",
    ] {
        assert!(html.contains(needle), "missing verbatim copy: `{needle}`");
    }
    assert!(
        !html.contains("Danger zone"),
        "the destroy heading replaced Danger zone (item 12)"
    );
    let commands = manifest_file("src/commands.rs");
    for needle in [
        r#"pub const APP_DISPLAY_NAME: &str = "QuantumShield Chat";"#,
        "Designed to stay secure even against future quantum computers.",
    ] {
        assert!(
            commands.contains(needle),
            "missing binding constant: `{needle}`"
        );
    }
}

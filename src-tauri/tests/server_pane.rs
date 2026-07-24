//! D609 slice-B Server pane — additive structural + claim-discipline guards.
//! design_round2.rs / design_system.rs stay byte-frozen; these are NEW and
//! pin the slice-B surface so a later edit cannot silently drift it.

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
fn server_pane_has_the_connectivity_controls() {
    let html = ui("index.html");
    for needle in [
        r#"id="relay-url""#,
        r#"id="relay-token""#,
        r#"id="relay-ca-path""#,
        r#"id="btn-relay-test""#,
        r#"id="btn-relay-save""#,
        r#"id="btn-relay-token-set""#,
        r#"id="btn-relay-token-clear""#,
        r#"id="btn-relay-ca-set""#,
        r#"id="btn-relay-ca-clear""#,
        r#"id="relay-results""#,
    ] {
        assert!(html.contains(needle), "server pane missing {needle}");
    }
    // the slice-A placeholder copy is gone
    assert!(
        !html.contains("makes no network connections at all"),
        "the #pane-server placeholder copy must be replaced by the pane"
    );
}

#[test]
fn results_reuse_the_shipped_status_banner_no_invented_classes() {
    let html = ui("index.html");
    let js = ui("main.js");
    // the results headline IS the shipped §2 status-banner component
    assert!(html.contains(r#"id="relay-status" class="status-banner"#));
    // the connected state uses the shipped `neutral` kind
    assert!(js.contains(r#"setBanner(status, "neutral", "Connected")"#));
    // R7: red (status-danger) is RESERVED for vault-danger; the results never
    // use it, and NO new status-* colour class is invented (no mockup palette).
    for banned in ["status-ok", "status-bad", "status-warn", "status-success"] {
        assert!(
            !html.contains(banned) && !js.contains(banned),
            "invented status class {banned} — R7 forbids new colour classes"
        );
    }
}

#[test]
fn no_bypass_control_anywhere() {
    // R8: the GUI face of NA-0663's hard boundary. No connect-anyway / trust-
    // this-cert affordance exists; the ONLY remedy for an untrusted cert is the
    // operator CA file. Guard the copy so a later edit cannot smuggle a bypass.
    let html = ui("index.html").to_lowercase();
    let js = ui("main.js").to_lowercase();
    for banned in [
        "connect anyway",
        "trust this certificate",
        "trust anyway",
        "ignore certificate",
        "proceed anyway",
        "skip verification",
        "disable verification",
    ] {
        assert!(
            !html.contains(banned),
            "bypass affordance in index.html: {banned}"
        );
        assert!(
            !js.contains(banned),
            "bypass affordance in main.js: {banned}"
        );
    }
}

#[test]
fn claim_discipline_five_surfaces_swept() {
    // D609 R4: five surfaces edited; the two COMPOUND claims kept their
    // surviving true clause. Stale "no network / serverless" claims are retired.
    let html = ui("index.html");
    let js = ui("main.js");
    let lib = repo_file("src-tauri/src/lib.rs");
    let cmds = repo_file("src-tauri/src/commands.rs");

    // stale claims RETIRED (full phrases, so the explanatory comments — which
    // quote the retired clause — do not trip the guard)
    assert!(
        !html.contains("serverless skeleton"),
        "stub-note still says serverless skeleton"
    );
    assert!(
        !html.contains("server setup arrives in a future update"),
        "footer still says server setup arrives in a future update"
    );
    assert!(
        !js.contains("makes no network connections"),
        "main.js About still says makes no network connections"
    );
    assert!(
        !lib.contains("makes no network connections"),
        "lib.rs About still says makes no network connections"
    );
    assert!(
        !cmds.contains("serverless skeleton"),
        "app_info slice string still says serverless skeleton"
    );

    // surviving TRUE clauses KEPT (the two compound surfaces)
    assert!(
        html.contains("Adding contacts arrives in a future update"),
        "stub-note lost the still-true contacts clause"
    );
    assert!(
        js.contains("no security-assurance claims"),
        "About in-app lost the surviving no-assurance clause"
    );
    assert!(
        lib.contains("no security-assurance claims"),
        "About native menu lost the surviving no-assurance clause"
    );
}

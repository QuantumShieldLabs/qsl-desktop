//! NA-0779 (spine `D-1422`; desktop `D-0048`) STOP 004 -- THE DEBUG LOG's DESKTOP ARMS, RED FIRST
//! (`STOP_NA0779_002` sec 9; kickoff L5). The ring, the levels, the lock, the erase, the cap, the
//! export and its digest, the disposal event, the desktop vocabulary's refusals, and the
//! redaction at every desktop source -- through the REAL engine choke point (`qsc::output::
//! emit_marker`) with the REAL sink installed where the arm needs it.
//!
//! The sink slot and the global log are process-wide, so every test here serialises on ARM_LOCK
//! and leaves the global log OFF with the sink removed. The pure arms use their own `DebugLog`
//! instances. The indicator (the pill) and the live list are the GUI driver's arm
//! (`f_v_debug_log_pane`), not this file's.

use qsc::output::event::{Event, Level, Outcome, Source};
use qsc::output::{emit_marker, set_marker_routing, MarkerRouting};
use qsl_desktop_app::debug_log::{
    desktop_event, install_sink, sha256_hex, sink_installed, uninstall_sink, verify_export,
    DebugLog, Refusal, COMMAND_NAMES, DESKTOP_EVENTS, QSC_PIN, RING_CAP,
};
use qsl_desktop_app::gateway::CoreGateway;
use qsl_desktop_app::settings::{self, AppSettings, DebugLogLevel, DebugLogSetting};
use std::sync::{Mutex, MutexGuard, OnceLock};

static ARM_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn arm_lock() -> MutexGuard<'static, ()> {
    ARM_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

/// A fresh, ON, detailed log with a session open.
fn fresh(level: DebugLogLevel) -> DebugLog {
    let log = DebugLog::new();
    log.on_unlock(true, level);
    log
}

fn export_of(log: &DebugLog) -> String {
    log.export_text("testbuild", "0123456789abcdef", "2026-09-05T20:00:00.000Z")
}

/// THE SEVEN PLANTS (kickoff L5; STOP 002 sec 9): an alias, an invite code, a bearer token, a
/// route token, a relay URL, key material, an identity-derived hash.
const PLANTS: [&str; 7] = [
    "PLANT-alias-dana-oncology",
    "PLANT-QSLI-1-invitecode0123456789",
    "PLANT-bearer-Zx9Qw8Er7Ty6Ui5Op4As3Df2Gh1Jk0",
    "PLANT-route-token-0f1e2d3c4b5a69788796a5b4c3d2e1f0",
    "https://plant.example.invalid/v1/attachments",
    "PLANT-key-MCowBQYDK2VwAyEAplantplantplantplantplant",
    "PLANT-hash8-deadbeef",
];

fn plants_in(text: &str) -> usize {
    PLANTS.iter().filter(|p| text.contains(*p)).count()
}

/// RED: the ring and the export are TRANSPARENT -- an event that carries a plant shows it. The
/// only way to build such an event is to leak a String into a `&'static str` (which no shipped
/// constructor does); the arm proves the pipe would show a plant if a constructor ever let one
/// through. Then GREEN: the same seven plants, each at a REAL desktop source, reach no line.
#[test]
fn a1_redaction_red_then_green_at_every_desktop_source() {
    let _g = arm_lock();
    // RED -- 7 of 7 through the pipe with a hand-built (leaked) event.
    let red = fresh(DebugLogLevel::Detailed);
    for p in PLANTS.iter() {
        let leaked: &'static str = Box::leak(p.to_string().into_boxed_str());
        red.push(&Event {
            level: Level::Events,
            source: Source::Ui,
            name: "ui.guard_refused",
            code: None,
            outcome: None,
            reason: None,
            duration_ms: None,
            ints: Vec::new(),
            bools: Vec::new(),
            enums: vec![("guard", leaked)],
        });
    }
    assert_eq!(
        plants_in(&export_of(&red)),
        7,
        "RED ARM: the pipe must show every plant it is handed"
    );

    // GREEN 1 -- the engine source: the real choke point with the real sink into the global log.
    let log = DebugLog::global();
    log.on_unlock(true, DebugLogLevel::Detailed);
    set_marker_routing(MarkerRouting::InApp);
    let sites: [(&str, &str, &str); 7] = [
        ("contacts_add", "peer", PLANTS[0]),
        ("invite_cleared", "invite_id", PLANTS[1]),
        ("relay_token_set", "token", PLANTS[2]),
        ("recv_start", "mailbox", PLANTS[3]),
        ("relay_server_info", "attachments_service_url", PLANTS[4]),
        ("outbox_contact", "peer_key", PLANTS[5]),
        ("recv_start", "mailbox_hash", PLANTS[6]),
    ];
    for (name, key, value) in sites.iter() {
        emit_marker(name, None, &[(key, value)]);
    }
    // GREEN 2 -- the gateway source: a command's ARGUMENT is a plant; gw.command carries the name only.
    let gw = CoreGateway::default();
    let secret = PLANTS[2].to_string();
    let r = tauri::async_runtime::block_on(gw.call_named("relay_token_set", move || {
        let carried = secret.len(); // USED inside the closure; it never leaves it
        if carried > 0 {
            Ok::<(), String>(())
        } else {
            Err("empty".to_string())
        }
    }));
    assert!(r.is_ok());
    // GREEN 3 -- the ui source: a plant in a field is REFUSED, not stored.
    for (name, k, v) in [
        ("ui.guard_refused", "guard", PLANTS[0]),
        ("ui.surface", "screen", PLANTS[4]),
        ("ui.tick_beat", "note", PLANTS[1]),
        ("gw.command", "name", PLANTS[5]),
        ("ui.made_up", "x", PLANTS[6]),
    ] {
        assert!(
            log.push_from_ui(name, &[(k, v)]).is_err(),
            "{name} {k} must be refused"
        );
    }
    set_marker_routing(MarkerRouting::Stdout);
    let text = export_of(log);
    log.on_erase();
    assert!(!sink_installed());
    assert_eq!(
        plants_in(&text),
        0,
        "GREEN ARM: 0 of 7 plants may reach the export: {text}"
    );
    for p in PLANTS.iter() {
        let frag = &p[..12];
        assert!(
            !text.contains(frag),
            "a plant fragment {frag:?} reached the export"
        );
    }
    assert!(
        text.contains("ev=contacts_add"),
        "the engine events themselves arrived"
    );
    assert!(text.contains("ev=gw.command") && text.contains("c.name=relay_token_set"));
    assert!(
        text.contains("ev=gw.log") && text.contains("c.action=refused"),
        "each refusal left its notice"
    );
    assert!(text.is_ascii());
}

/// The level arm, by list: a detailed-only engine event is absent at "events" and present at
/// "detailed"; an events-level one is present at both; a desktop event follows its own level.
#[test]
fn a2_level_by_list() {
    let _g = arm_lock();
    let log = DebugLog::global();
    for (level, expect_detailed) in [
        (DebugLogLevel::Events, false),
        (DebugLogLevel::Detailed, true),
    ] {
        log.on_unlock(true, level);
        set_marker_routing(MarkerRouting::InApp);
        emit_marker("vault_unlock", None, &[("ok", "true")]); // events level
        emit_marker("relay_pull_diagnostic", None, &[("count", "1")]); // detailed only
        set_marker_routing(MarkerRouting::Stdout);
        log.push_desktop("ui.tick_gate", &[("gate", "open")])
            .unwrap(); // lvl d
        log.push_desktop("ui.tick_beat", &[("source", "tick")])
            .unwrap(); // lvl e
        let text = export_of(log);
        log.on_erase();
        assert!(text.contains("ev=vault_unlock"));
        assert!(text.contains("ev=ui.tick_beat"));
        assert_eq!(
            text.contains("ev=relay_pull_diagnostic"),
            expect_detailed,
            "{level:?}: {text}"
        );
        assert_eq!(
            text.contains("ev=ui.tick_gate"),
            expect_detailed,
            "{level:?}"
        );
    }
}

/// The lock arm: after a lock the ring holds EXACTLY one event, gw.lock with its cause, at seq 1;
/// the sink is gone; the next unlock continues with gw.unlock at seq 2.
#[test]
fn a3_lock_keeps_only_the_cause_and_restarts_seq() {
    let _g = arm_lock();
    let log = DebugLog::global();
    log.on_unlock(true, DebugLogLevel::Detailed);
    assert!(sink_installed());
    for i in 0..5 {
        log.push_desktop("ui.scan_rerun", &[("count", &i.to_string())])
            .unwrap();
    }
    assert!(log.read(0, 100).buffered >= 6);
    log.on_lock("autolock");
    assert!(!sink_installed(), "the slot is None while locked");
    let r = log.read(0, 100);
    assert_eq!(r.buffered, 1);
    assert_eq!(r.lines[0].seq, 1);
    assert!(
        r.lines[0].line.contains("ev=gw.lock") && r.lines[0].line.contains("c.cause=autolock"),
        "{}",
        r.lines[0].line
    );
    log.on_unlock(true, DebugLogLevel::Detailed);
    let r = log.read(0, 100);
    assert_eq!(r.buffered, 2);
    assert!(
        r.lines[1].line.starts_with("seq=2 ") && r.lines[1].line.contains("ev=gw.unlock out=ok")
    );
    // an unknown cause reads as `user`, never as the offered text
    log.on_lock("PLANT-cause");
    assert!(log.read(0, 10).lines[0].line.contains("c.cause=user"));
    log.on_erase();
}

/// The erase arm: gw.erase, then the ring is EMPTY, the switch reads off, the sink is gone.
#[test]
fn a4_erase_wipes_and_switches_off() {
    let _g = arm_lock();
    let log = DebugLog::global();
    log.on_unlock(true, DebugLogLevel::Detailed);
    log.push_desktop("ui.scan_busy", &[]).unwrap();
    log.on_erase();
    let r = log.read(0, 10);
    assert_eq!(r.buffered, 0);
    assert!(!r.on);
    assert!(!sink_installed());
    assert!(
        !log.push_desktop("ui.scan_busy", &[]).unwrap(),
        "off: nothing enters"
    );
}

/// The cap arm: a flood of 20000 events leaves buffered=8192 and dropped=11808, the oldest gone,
/// and ONE drop notice per episode (the notice itself is evicted by a flood this large -- the
/// header's dropped count is what survives; a small overflow shows the notice as its last line).
#[test]
fn a5_cap_drops_oldest_counts_and_notifies_once_per_episode() {
    let _g = arm_lock();
    let log = fresh(DebugLogLevel::Detailed);
    for i in 0..20_000u32 {
        log.push_desktop("ui.scan_rerun", &[("count", &i.to_string())])
            .unwrap();
    }
    let r = log.read(0, 10);
    assert_eq!(r.buffered, RING_CAP);
    // 20000 pushed + 1 gw.unlock + 1 drop notice = 20002 entered; 8192 kept.
    assert_eq!(r.dropped, 20_002 - RING_CAP as u64);
    assert_eq!(
        log.drop_notices(),
        1,
        "one notice per episode, not per event"
    );
    assert!(r.lines[0].seq > 1, "the oldest is gone");
    let header = export_of(&log);
    // the export pushes its OWN gw.log event first, which evicts one more: the header's figure
    // is the ring's figure AFTER that push, read back rather than derived twice.
    let after = log.read(0, 1);
    assert_eq!(after.dropped, r.dropped + 1);
    assert!(
        header.contains(&format!("dropped={} ", after.dropped)),
        "{}",
        header.lines().nth(1).unwrap()
    );
    // the small overflow: the notice is the newest line
    let small = fresh(DebugLogLevel::Detailed);
    for i in 0..RING_CAP {
        small
            .push_desktop("ui.scan_rerun", &[("count", &i.to_string())])
            .unwrap();
    }
    let r = small.read(0, 1);
    let last = small.read(r.seq_last - 1, 5);
    assert!(
        last.lines.last().unwrap().line.contains("ev=gw.log")
            && last.lines.last().unwrap().line.contains("c.action=drop")
    );
    assert_eq!(small.drop_notices(), 1);
    // a clear closes the episode; the next overflow notifies again
    small.clear();
    for i in 0..(RING_CAP + 2) {
        small
            .push_desktop("ui.scan_rerun", &[("count", &i.to_string())])
            .unwrap();
    }
    assert_eq!(small.drop_notices(), 2);
}

/// The export and digest arm: the header's fields, one line per event, the footer's sha256 equal
/// to the digest of every byte before it; one flipped byte breaks the check; a truncated file
/// breaks it. Written to a directory: created new at 0600, named by time and label.
#[test]
fn a6_export_header_lines_footer_and_the_digest_breaks_on_one_byte() {
    let _g = arm_lock();
    let log = fresh(DebugLogLevel::Events);
    log.push_desktop("gw.lock", &[("cause", "user")]).unwrap();
    let text = export_of(&log);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with("# app=qsl-desktop version="));
    assert!(
        lines[0].contains(&format!("qsc_pin={QSC_PIN}"))
            && lines[0].contains("build_commit=testbuild")
    );
    assert!(lines[1].starts_with("# level=events exported_utc=2026-09-05T20:00:00.000Z cap=8192 buffered=3 dropped=0 seq_first=1 seq_last=3 label=0123456789abcdef format=qsl-debug-log/1"), "{}", lines[1]);
    assert_eq!(
        lines[2],
        "# digest=sha256-of-all-bytes-before-the-footer-line"
    );
    assert!(lines[3].starts_with("seq=1 ") && lines[3].contains("ev=gw.unlock"));
    assert!(
        lines[4].starts_with("seq=2 ")
            && lines[4].contains("ev=gw.lock")
            && lines[4].contains("c.cause=user")
    );
    assert!(
        lines[5].starts_with("seq=3 ")
            && lines[5].contains("ev=gw.log")
            && lines[5].contains("c.action=export"),
        "the export contains its own event: {}",
        lines[5]
    );
    let footer = lines[6];
    assert!(footer.starts_with("# sha256=") && footer.len() == 9 + 64);
    let body = &text[..text.rfind("# sha256=").unwrap()];
    assert_eq!(&footer[9..], sha256_hex(body.as_bytes()));
    assert!(verify_export(&text));
    assert!(text.is_ascii());
    // one flipped byte inside the body
    let mut flipped = text.clone().into_bytes();
    let i = text.find("ev=gw.lock").unwrap();
    flipped[i] = b'E';
    assert!(!verify_export(std::str::from_utf8(&flipped).unwrap()));
    // a truncated file
    let cut = &text[..text.len() - 20];
    assert!(!verify_export(cut));
    // to a directory
    let dir = tempfile::tempdir().unwrap();
    let dto = log.export_to_dir(dir.path(), "testbuild").unwrap();
    let written = std::fs::read_to_string(&dto.path).unwrap();
    assert!(verify_export(&written));
    assert_eq!(dto.sha256, sha256_hex(written.as_bytes()));
    assert_eq!(dto.bytes, written.len());
    assert_eq!(dto.label.len(), 16);
    let name = std::path::Path::new(&dto.path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        name.starts_with("qsl-desktop-debug-log-")
            && name.ends_with(&format!("-{}.txt", dto.label)),
        "{name}"
    );
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&dto.path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(
        log.export_to_dir(&dir.path().join("absent"), "testbuild")
            .is_err(),
        "no directory, no file"
    );
}

/// The disposal arm (A3's surface): a declined frame's marker reaches the export with its class
/// and disposition, through the real choke point.
#[test]
fn a7_disposal_event_reaches_the_export() {
    let _g = arm_lock();
    let log = DebugLog::global();
    log.on_unlock(true, DebugLogLevel::Detailed);
    set_marker_routing(MarkerRouting::InApp);
    emit_marker(
        "recv_frame_skipped",
        None,
        &[
            ("class", "message"),
            ("disposition", "left_leased"),
            ("reason", "peer_unconfirmed"),
            ("id", "PLANT-frame-id"),
        ],
    );
    set_marker_routing(MarkerRouting::Stdout);
    let text = export_of(log);
    log.on_erase();
    assert!(
        text.contains("ev=recv_frame_skipped")
            && text.contains("c.class=message")
            && text.contains("c.disposition=left_leased"),
        "{text}"
    );
    assert!(!text.contains("PLANT-frame-id"));
}

/// The desktop vocabulary: every spec name is unique; ui.* is the only door for the front end;
/// unknown keys, non-members and non-integers are refused with a closed reason; the reason
/// field softens to `?` because it relays engine codes.
#[test]
fn a8_desktop_vocabulary_is_closed() {
    let _g = arm_lock();
    let mut names: Vec<&str> = DESKTOP_EVENTS.iter().map(|s| s.name).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), DESKTOP_EVENTS.len());
    assert!(names
        .iter()
        .all(|n| n.starts_with("ui.") || n.starts_with("gw.")));
    assert_eq!(
        desktop_event("ui.made_up", &[], true).unwrap_err(),
        Refusal::UnknownName
    );
    assert_eq!(
        desktop_event("gw.lock", &[("cause", "user")], true).unwrap_err(),
        Refusal::NotAUiName
    );
    assert!(desktop_event("gw.lock", &[("cause", "user")], false).is_ok());
    assert_eq!(
        desktop_event(
            "ui.surface",
            &[("screen", "scr-main"), ("extra", "x")],
            true
        )
        .unwrap_err(),
        Refusal::UnknownKey
    );
    assert_eq!(
        desktop_event("ui.surface", &[("screen", "scr-secret")], true).unwrap_err(),
        Refusal::NotInVocabulary
    );
    assert_eq!(
        desktop_event(
            "ui.autolock",
            &[("decision", "fired"), ("idle_s", "soon")],
            true
        )
        .unwrap_err(),
        Refusal::NotAnInteger
    );
    let ok = desktop_event(
        "ui.tick_beat",
        &[
            ("source", "tick"),
            ("out", "ok"),
            ("dur_ms", "42"),
            ("contacts", "3"),
        ],
        true,
    )
    .unwrap();
    assert_eq!(ok.to_line(7, 0), "seq=7 utc=1970-01-01T00:00:00.000Z lvl=e src=ui ev=ui.tick_beat out=ok dur=42 n.contacts=3 c.source=tick");
    let gw = desktop_event(
        "gw.unlock",
        &[("out", "fail"), ("reason", "PLANT-free-text")],
        false,
    )
    .unwrap();
    assert_eq!(
        gw.reason,
        Some("?"),
        "a reason outside the engine's vocabulary becomes ?"
    );
    let gw2 = desktop_event(
        "gw.unlock",
        &[("out", "fail"), ("reason", "peer_mismatch")],
        false,
    )
    .unwrap();
    assert_eq!(gw2.reason, Some("peer_mismatch"));
    assert!(
        COMMAND_NAMES.contains(&"debug_log_export") && COMMAND_NAMES.contains(&"unlock_attempt")
    );
}

/// The switch persists: settings round-trip with the log on / detailed; the default is omitted
/// (the file's key set is the prior one); the DTO names are the wire names the pane reads.
#[test]
fn a9_the_switch_is_a_stored_setting_that_round_trips() {
    let _g = arm_lock();
    let dir = tempfile::tempdir().unwrap();
    let s = AppSettings {
        debug_log: DebugLogSetting {
            on: true,
            level: DebugLogLevel::Detailed,
        },
        ..Default::default()
    };
    settings::save(dir.path(), &s).unwrap();
    let raw = std::fs::read_to_string(qsl_desktop_app::paths::settings_file(dir.path())).unwrap();
    assert!(
        raw.contains("\"debug_log\"") && raw.contains("\"detailed\""),
        "{raw}"
    );
    assert_eq!(settings::load(dir.path()), s);
    settings::save(dir.path(), &AppSettings::default()).unwrap();
    let raw = std::fs::read_to_string(qsl_desktop_app::paths::settings_file(dir.path())).unwrap();
    assert!(!raw.contains("debug_log"), "the default is omitted: {raw}");
    // no environment variable is read anywhere for the switch (the tree's own rule): the module
    // names none, and the only `env::var` in it is the OS RNG's absence.
    let src =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/debug_log.rs")).unwrap();
    assert!(
        !src.contains("env::var("),
        "no env switch in the log's module"
    );
}

/// The Clear button: the ring empties, seq CONTINUES, the drop count resets with the ring.
#[test]
fn a10_clear_empties_and_seq_continues() {
    let _g = arm_lock();
    let log = fresh(DebugLogLevel::Detailed);
    for i in 0..3 {
        log.push_desktop("ui.scan_rerun", &[("count", &i.to_string())])
            .unwrap();
    }
    let before = log.read(0, 10).seq_last;
    log.clear();
    let r = log.read(0, 10);
    assert_eq!(r.buffered, 1, "the clear's own gw.log event");
    assert!(r.lines[0].line.contains("c.action=clear"));
    assert_eq!(r.lines[0].seq, before + 1);
    assert!(!r.reset, "a clear is not a lock");
}

/// The read's contract for a live viewer: since_seq filters; a ring restart reports reset.
#[test]
fn a11_read_since_seq_and_reset() {
    let _g = arm_lock();
    let log = fresh(DebugLogLevel::Detailed);
    for i in 0..5 {
        log.push_desktop("ui.scan_rerun", &[("count", &i.to_string())])
            .unwrap();
    }
    let all = log.read(0, 100);
    assert_eq!(all.buffered, 6);
    let tail = log.read(4, 100);
    assert_eq!(tail.lines.len(), 2);
    assert!(tail.lines.iter().all(|s| s.seq > 4));
    let capped = log.read(0, 2);
    assert_eq!(capped.lines.len(), 2);
    log.on_lock("user");
    let after = log.read(6, 100);
    assert!(
        after.reset,
        "the caller held seq 6; the ring restarted at 1"
    );
    assert_eq!(after.lines.len(), 1);
    // the gateway's outcome trait through a real Result
    let gw = CoreGateway::default();
    let out: Result<(), String> = tauri::async_runtime::block_on(
        gw.call_named("relay_probe", || Err("relay_unreachable".to_string())),
    );
    assert!(out.is_err());
    let _ = uninstall_sink;
    let _ = install_sink;
    let _ = Outcome::Ok;
}

//! NA-0779 (spine `D-1422`; desktop `D-0048`) -- THE DEBUG LOG's DESKTOP HALF.
//!
//! The engine half (qsl-protocol `#1819`, pinned here at `4e03092f`) offers every marker to an
//! optional sink as a TYPED `qsc::output::event::Event` built through an allowlist: a key outside
//! the allowlist is dropped without its value being read. This module HOSTS that sink:
//!
//!   * THE RING -- 8192 events, memory only, oldest dropped and counted (one `gw.log action=drop`
//!     notice per overflow episode, never per event), cleared on lock keeping the lock's cause as
//!     the new ring's first event, wiped by erase. No path is opened by the ring.
//!   * THE SWITCH's runtime -- a stored setting (`settings.json`: `debug_log = { on, level }`,
//!     the existing 0600 writer) read at unlock; NEVER an environment variable.
//!   * THE DESKTOP's OWN CLOSED VOCABULARY -- `gw.*` from the gateway, `ui.*` from the front end
//!     through ONE command (`debug_log_event`) that REFUSES any name or field outside the
//!     vocabulary and says so with a `gw.log action=refused` notice of its own.
//!   * THE EXPORT -- one ASCII file: a header, one line per event (the SAME line the viewer
//!     shows), a `# sha256=` footer over every byte before it. The only disk write of the log.
//!
//! The model is `STOP_NA0779_002` sec 3-7 as the operator blessed it
//! (`RBANK_debug_log_event_model_blessed_20260905`) with `RULING_NA0779_002` R2 (a)(b) and R3.
//!
//! WHAT NEVER ENTERS: a value. `Event` holds only `&'static str` MEMBERS of closed tables,
//! integers and bools, and the desktop's own events are built the same way (`desktop_event`),
//! so an alias, an invite code, a token, a URL, a path, a passphrase or free text has no field
//! to land in. Command ARGUMENTS never enter `gw.command`: it carries the command's NAME (a
//! member of the registered set), its outcome, a reason code from the engine's closed reason
//! vocabulary (else `?`) and a duration.

use crate::settings::DebugLogLevel;
use qsc::output::event::{self as engine, Event, Level, Outcome, Source, ENUM_KEYS};
use serde::Serialize;
use std::collections::VecDeque;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// The ring's cap. STOP 002 sec 5: at "detailed" the ordinary beat costs under 30 events, the
/// bounded worst case under 500; at the INSTANT tempo the ordinary case fills 8192 in ~90 min.
pub const RING_CAP: usize = 8192;
/// The export format's name, printed in every header.
pub const FORMAT: &str = "qsl-debug-log/1";
/// The digest construction, printed in every header so the receiver can recompute it:
/// `head -n -1 <file> | sha256sum`.
pub const DIGEST_CONSTRUCTION: &str = "sha256-of-all-bytes-before-the-footer-line";
/// The qsc pin this build carries (`RULING_NA0779_003` R4 (3): `#1819`'s MERGE COMMIT, no later
/// bump). A test asserts it equals `src-tauri/Cargo.toml`'s `rev`.
pub const QSC_PIN: &str = "4e03092fb14a6129065b711f8c17fc7252965e07";
/// The literal an unlisted reason or command name becomes. Never a copy of the input.
pub const UNLISTED: &str = "?";

/// The registered command set (`lib.rs` `generate_handler!`). `gw.command`'s `c.name` is a
/// member of this table or `?`; a test pins the two against each other.
pub const COMMAND_NAMES: &[&str] = &[
    "app_info",
    "cli_vault_present",
    "connect_status",
    "contact_list",
    "contact_request_accept",
    "contact_request_block",
    "contact_request_ignore",
    "contact_requests",
    "contact_set_display_name",
    "core_busy",
    "debug_log_control",
    "debug_log_event",
    "debug_log_export",
    "debug_log_read",
    "destroy_vault",
    "erase_all",
    "home_dir",
    "identity_ensure",
    "identity_show",
    "invite_accept",
    "invite_clear",
    "invite_create",
    "invite_finish",
    "invite_list",
    "invite_redeem",
    "invite_revoke",
    "launch_state",
    "lock_now",
    "marker_stats",
    "notice_dismiss",
    "notice_list",
    "protection_status",
    "relay_ca_file_clear",
    "relay_ca_file_set",
    "relay_ca_file_show",
    "relay_config_get",
    "relay_config_set",
    "relay_probe",
    "relay_test",
    "relay_token_clear",
    "relay_token_set",
    "relay_token_show",
    "restart_app",
    "settings_get",
    "settings_set",
    "ui_surface_changed",
    "unlock_attempt",
    "vault_create",
    "wipe_arm",
    "wipe_disarm",
];

/// The seven screen ids (`ui/main.js` `SCREENS`).
pub const UI_SCREENS: &[&str] = &[
    "scr-wizard-vault",
    "scr-wizard-identity",
    "scr-unlock",
    "scr-erase",
    "scr-wiped",
    "scr-main",
    "scr-settings",
];

/// What a desktop event may carry, per name: its level and source, its INT keys, its ENUM keys
/// with CLOSED vocabularies, and whether `out`, `reason` and `dur_ms` are accepted. Anything
/// else is refused. `ui.*` names are the ONLY ones the front end may send; `gw.*` are emitted by
/// this crate alone.
pub struct DesktopSpec {
    pub name: &'static str,
    pub level: Level,
    pub source: Source,
    pub ints: &'static [&'static str],
    pub enums: &'static [(&'static str, &'static [&'static str])],
    pub out: bool,
    pub reason: bool,
    pub dur: bool,
}

pub const DESKTOP_EVENTS: &[DesktopSpec] = &[
    DesktopSpec {
        name: "ui.surface",
        level: Level::Detailed,
        source: Source::Ui,
        ints: &[],
        enums: &[("screen", UI_SCREENS)],
        out: false,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "ui.tick_gate",
        level: Level::Detailed,
        source: Source::Ui,
        ints: &[],
        enums: &[
            ("gate", &["open", "closed"]),
            ("reason", &["locked", "no_relay", "none"]),
        ],
        out: false,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "ui.tick_beat",
        level: Level::Events,
        source: Source::Ui,
        ints: &["contacts"],
        enums: &[("source", &["tick", "unlock", "surface_open", "manual"])],
        out: true,
        reason: false,
        dur: true,
    },
    DesktopSpec {
        name: "ui.scan_rerun",
        level: Level::Detailed,
        source: Source::Ui,
        ints: &["count"],
        enums: &[],
        out: false,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "ui.scan_busy",
        level: Level::Detailed,
        source: Source::Ui,
        ints: &[],
        enums: &[],
        out: false,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "ui.guard_refused",
        level: Level::Detailed,
        source: Source::Ui,
        ints: &[],
        enums: &[(
            "guard",
            &[
                "invite_live",
                "invite_in_flight",
                "redeem_in_flight",
                "name_grammar",
                "cap_full",
                "no_relay",
            ],
        )],
        out: false,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "ui.autolock",
        level: Level::Events,
        source: Source::Ui,
        ints: &["idle_s"],
        enums: &[("decision", &["fired", "zero_disabled", "off_surface"])],
        out: false,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "gw.command",
        level: Level::Detailed,
        source: Source::Gateway,
        ints: &[],
        enums: &[("name", COMMAND_NAMES)],
        out: true,
        reason: true,
        dur: true,
    },
    DesktopSpec {
        name: "gw.lock",
        level: Level::Events,
        source: Source::Gateway,
        ints: &[],
        enums: &[("cause", &["user", "autolock", "erase", "restart"])],
        out: false,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "gw.unlock",
        level: Level::Events,
        source: Source::Gateway,
        ints: &[],
        enums: &[],
        out: true,
        reason: true,
        dur: false,
    },
    DesktopSpec {
        name: "gw.erase",
        level: Level::Events,
        source: Source::Gateway,
        ints: &[],
        enums: &[],
        out: true,
        reason: false,
        dur: false,
    },
    DesktopSpec {
        name: "gw.log",
        level: Level::Events,
        source: Source::Gateway,
        ints: &[],
        enums: &[(
            "action",
            &[
                "on",
                "off",
                "level_events",
                "level_detailed",
                "export",
                "clear",
                "drop",
                "refused",
            ],
        )],
        out: false,
        reason: false,
        dur: false,
    },
];

/// Why a desktop event was refused. Each variant is a closed reason, never the offending text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    UnknownName,
    NotAUiName,
    UnknownKey,
    NotInVocabulary,
    NotAnInteger,
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        match self {
            Refusal::UnknownName => "unknown_name",
            Refusal::NotAUiName => "not_a_ui_name",
            Refusal::UnknownKey => "unknown_key",
            Refusal::NotInVocabulary => "not_in_vocabulary",
            Refusal::NotAnInteger => "not_an_integer",
        }
    }
}

fn spec_for(name: &str) -> Option<&'static DesktopSpec> {
    DESKTOP_EVENTS.iter().find(|s| s.name == name)
}

fn member(table: &'static [&'static str], s: &str) -> Option<&'static str> {
    table.iter().copied().find(|m| *m == s)
}

/// The engine's closed `reason` vocabulary (REJECT_* codes, literal marker codes and reason
/// literals), the SHARED one for `code` and `reason`. A string outside it becomes `?`.
pub fn engine_reason(s: &str) -> &'static str {
    ENUM_KEYS
        .iter()
        .find(|(k, _)| *k == "reason")
        .and_then(|(_, vocab)| member(vocab, s))
        .unwrap_or(UNLISTED)
}

fn outcome_member(s: &str) -> Option<Outcome> {
    match s {
        "ok" => Some(Outcome::Ok),
        "fail" => Some(Outcome::Fail),
        "refused" => Some(Outcome::Refused),
        "skipped" => Some(Outcome::Skipped),
        _ => None,
    }
}

/// THE DESKTOP CONSTRUCTOR. Builds a typed `Event` for a desktop name from `(key, value)` pairs,
/// REFUSING (never copying) anything outside the name's spec: an unknown name, a `gw.*` name
/// from the front end (`ui_only`), an unknown key, an enum value outside its closed vocabulary,
/// an int that does not parse. `reason` is looked up in the ENGINE's closed vocabulary and
/// becomes `?` outside it -- the one field that softens, because its values come from engine
/// error codes the desktop merely relays.
pub fn desktop_event(name: &str, fields: &[(&str, &str)], ui_only: bool) -> Result<Event, Refusal> {
    let spec = spec_for(name).ok_or(Refusal::UnknownName)?;
    if ui_only && !name.starts_with("ui.") {
        return Err(Refusal::NotAUiName);
    }
    let mut ev = Event {
        level: spec.level,
        source: spec.source,
        name: spec.name,
        code: None,
        outcome: None,
        reason: None,
        duration_ms: None,
        ints: Vec::new(),
        bools: Vec::new(),
        enums: Vec::new(),
    };
    for (k, v) in fields {
        if spec.out && *k == "out" {
            ev.outcome = Some(outcome_member(v).ok_or(Refusal::NotInVocabulary)?);
            continue;
        }
        if spec.reason && *k == "reason" {
            ev.reason = Some(engine_reason(v));
            continue;
        }
        if spec.dur && *k == "dur_ms" {
            ev.duration_ms = Some(v.trim().parse::<u32>().map_err(|_| Refusal::NotAnInteger)?);
            continue;
        }
        if let Some(key) = member(spec.ints, k) {
            ev.ints.push((
                key,
                v.trim().parse::<i64>().map_err(|_| Refusal::NotAnInteger)?,
            ));
            continue;
        }
        if let Some((key, vocab)) = spec.enums.iter().find(|(key, _)| key == k) {
            let m = member(vocab, v).ok_or(Refusal::NotInVocabulary)?;
            ev.enums.push((key, m));
            continue;
        }
        // Every other key is REFUSED. Nothing about its value is read past the lookup.
        return Err(Refusal::UnknownKey);
    }
    Ok(ev)
}

fn gw_log(action: &'static str) -> Event {
    desktop_event("gw.log", &[("action", action)], false).expect("gw.log's own vocabulary")
}

/// One stored event: the host's `seq` and `utc_ms` plus the rendered line (ASCII by
/// construction; the SAME bytes the viewer shows and the export carries).
#[derive(Clone, Debug, Serialize)]
pub struct Stored {
    pub seq: u64,
    pub utc_ms: u64,
    pub line: String,
}

struct Inner {
    on: bool,
    level: DebugLogLevel,
    ring: VecDeque<Stored>,
    next_seq: u64,
    dropped: u64,
    drop_episode_open: bool,
    drop_notices: u64,
    unlocked_at: Option<Instant>,
}

pub struct DebugLog {
    inner: Mutex<Inner>,
}

impl Default for DebugLog {
    fn default() -> Self {
        Self::new()
    }
}

/// The viewer's read: the switch, the counts and the lines since a sequence number.
#[derive(Clone, Debug, Serialize)]
pub struct ReadDto {
    pub on: bool,
    pub level: DebugLogLevel,
    pub buffered: usize,
    pub dropped: u64,
    pub seq_first: u64,
    pub seq_last: u64,
    pub next_seq: u64,
    /// True when the ring restarted since the caller's `since_seq` (a lock or an erase): the
    /// caller must drop what it holds and take `lines` as the whole.
    pub reset: bool,
    pub since_unlock_ms: Option<u64>,
    pub lines: Vec<Stored>,
}

/// What an export produced.
#[derive(Clone, Debug, Serialize)]
pub struct ExportDto {
    pub path: String,
    pub bytes: usize,
    pub sha256: String,
    pub label: String,
}

static GLOBAL: OnceLock<DebugLog> = OnceLock::new();

impl DebugLog {
    pub fn new() -> Self {
        DebugLog {
            inner: Mutex::new(Inner {
                on: false,
                level: DebugLogLevel::Events,
                ring: VecDeque::new(),
                next_seq: 1,
                dropped: 0,
                drop_episode_open: false,
                drop_notices: 0,
                unlocked_at: None,
            }),
        }
    }

    /// The process's one log: the sink closure and the gateway reach it here.
    pub fn global() -> &'static DebugLog {
        GLOBAL.get_or_init(DebugLog::new)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn is_on(&self) -> bool {
        self.lock().on
    }

    pub fn level(&self) -> DebugLogLevel {
        self.lock().level
    }

    /// The switch's RUNTIME half. Persisting it is the command's job (settings.rs, 0600).
    pub fn set_on(&self, on: bool) {
        self.lock().on = on;
    }

    pub fn set_level(&self, level: DebugLogLevel) {
        self.lock().level = level;
    }

    /// THE ONE INTAKE. Off -> nothing. At "events", a detailed-only event -> nothing. Otherwise
    /// the host's `seq` and `utc_ms` are assigned, the line rendered, and the ring advanced;
    /// an overflow drops the OLDEST, counts it, and opens a drop episode with ONE notice.
    pub fn push(&self, ev: &Event) -> bool {
        let mut g = self.lock();
        if !g.on {
            return false;
        }
        if g.level == DebugLogLevel::Events && ev.level == Level::Detailed {
            return false;
        }
        let seq = g.next_seq;
        g.next_seq += 1;
        let utc_ms = now_ms();
        let line = ev.to_line(seq, utc_ms);
        debug_assert!(line.is_ascii(), "an event line is ASCII by construction");
        let overflowed = g.ring.len() >= RING_CAP;
        if overflowed {
            g.ring.pop_front();
            g.dropped += 1;
        }
        g.ring.push_back(Stored { seq, utc_ms, line });
        let notify = overflowed && !g.drop_episode_open;
        if notify {
            g.drop_episode_open = true;
            g.drop_notices += 1;
        }
        drop(g);
        if notify {
            // The episode is open now, so this push cannot notify again: no recursion.
            self.push(&gw_log("drop"));
        }
        true
    }

    /// A desktop event through the closed constructor, from this crate's own sites.
    pub fn push_desktop(&self, name: &str, fields: &[(&str, &str)]) -> Result<bool, Refusal> {
        let ev = desktop_event(name, fields, false)?;
        Ok(self.push(&ev))
    }

    /// The front end's ONE door: `ui.*` names only; a refusal is answered with a `gw.log
    /// action=refused` notice in the ring (so the log shows that something was refused, never
    /// what) and the closed reason to the caller.
    pub fn push_from_ui(&self, name: &str, fields: &[(&str, &str)]) -> Result<bool, Refusal> {
        match desktop_event(name, fields, true) {
            Ok(ev) => Ok(self.push(&ev)),
            Err(r) => {
                self.push(&gw_log("refused"));
                Err(r)
            }
        }
    }

    /// `gw.command`: the NAME (a member of the registered set or `?`), the outcome, the reason
    /// from the engine's closed vocabulary (or `?`), the duration. Arguments never reach here.
    pub fn gw_command(&self, name: &str, outcome: Outcome, reason: Option<&str>, dur_ms: u128) {
        let name = member(COMMAND_NAMES, name).unwrap_or(UNLISTED);
        let ev = Event {
            level: Level::Detailed,
            source: Source::Gateway,
            name: "gw.command",
            code: None,
            outcome: Some(outcome),
            reason: reason.map(engine_reason),
            duration_ms: Some(dur_ms.min(u32::MAX as u128) as u32),
            ints: Vec::new(),
            bools: Vec::new(),
            enums: vec![("name", name)],
        };
        self.push(&ev);
    }

    /// `ui.surface` from the surface-change command (the seven screen ids; anything else is
    /// refused and never stored).
    pub fn ui_surface(&self, screen: &str) {
        let _ = self.push_desktop("ui.surface", &[("screen", screen)]);
    }

    /// AT LOCK: the ring is emptied and the lock's own event becomes the FIRST event of the new,
    /// otherwise empty ring; `seq` restarts at 1; the sink is removed (the slot is `None` while
    /// locked). The cause survives; the session's history does not.
    pub fn on_lock(&self, cause: &str) {
        let cause = member(&["user", "autolock", "erase", "restart"], cause).unwrap_or("user");
        {
            let mut g = self.lock();
            g.ring.clear();
            g.next_seq = 1;
            g.dropped = 0;
            g.drop_episode_open = false;
            g.unlocked_at = None;
        }
        uninstall_sink();
        let _ = self.push_desktop("gw.lock", &[("cause", cause)]);
    }

    /// AT UNLOCK: the stored switch is applied; with the log on the sink is installed and
    /// `gw.unlock` is the session's next event (after the lock's cause, if any).
    pub fn on_unlock(&self, on: bool, level: DebugLogLevel) {
        {
            let mut g = self.lock();
            g.on = on;
            g.level = level;
            g.unlocked_at = Some(Instant::now());
        }
        if on {
            install_sink();
            let _ = self.push_desktop("gw.unlock", &[("out", "ok")]);
        } else {
            uninstall_sink();
        }
    }

    /// AT ERASE: `gw.erase` is emitted, then the ring is WIPED and the switch returns to off
    /// with the rest of the settings (the file is gone); the sink is removed.
    pub fn on_erase(&self) {
        let _ = self.push_desktop("gw.erase", &[("out", "ok")]);
        {
            let mut g = self.lock();
            g.ring.clear();
            g.next_seq = 1;
            g.dropped = 0;
            g.drop_episode_open = false;
            g.on = false;
            g.level = DebugLogLevel::Events;
            g.unlocked_at = None;
        }
        uninstall_sink();
    }

    /// The Clear button: the ring is emptied; `seq` CONTINUES (a clear is not a lock); the
    /// drop count resets with the ring it counted.
    pub fn clear(&self) {
        {
            let mut g = self.lock();
            g.ring.clear();
            g.dropped = 0;
            g.drop_episode_open = false;
        }
        let _ = self.push_desktop("gw.log", &[("action", "clear")]);
    }

    /// The switch's four actions from the pane, applied at runtime (the caller persists them).
    pub fn apply_action(&self, action: &str) -> bool {
        match action {
            "on" => {
                self.set_on(true);
                install_sink();
                let _ = self.push_desktop("gw.log", &[("action", "on")]);
                true
            }
            "off" => {
                let _ = self.push_desktop("gw.log", &[("action", "off")]);
                self.set_on(false);
                uninstall_sink();
                true
            }
            "level_events" => {
                self.set_level(DebugLogLevel::Events);
                let _ = self.push_desktop("gw.log", &[("action", "level_events")]);
                true
            }
            "level_detailed" => {
                self.set_level(DebugLogLevel::Detailed);
                let _ = self.push_desktop("gw.log", &[("action", "level_detailed")]);
                true
            }
            "clear" => {
                self.clear();
                true
            }
            _ => false,
        }
    }

    pub fn drop_notices(&self) -> u64 {
        self.lock().drop_notices
    }

    /// The viewer's read. `since_seq` = the last seq the caller holds (0 for none); `max` bounds
    /// one read. A ring restart (lock / erase) is reported as `reset` with the whole ring.
    pub fn read(&self, since_seq: u64, max: usize) -> ReadDto {
        let g = self.lock();
        let reset = since_seq >= g.next_seq && since_seq != 0;
        let lines: Vec<Stored> = g
            .ring
            .iter()
            .filter(|s| reset || s.seq > since_seq)
            .take(max.max(1))
            .cloned()
            .collect();
        ReadDto {
            on: g.on,
            level: g.level,
            buffered: g.ring.len(),
            dropped: g.dropped,
            seq_first: g.ring.front().map(|s| s.seq).unwrap_or(0),
            seq_last: g.ring.back().map(|s| s.seq).unwrap_or(0),
            next_seq: g.next_seq,
            reset,
            since_unlock_ms: g.unlocked_at.map(|t| t.elapsed().as_millis() as u64),
            lines,
        }
    }

    /// THE EXPORT's BYTES: the header, one line per event, the digest footer. Pure ASCII by
    /// construction. `gw.log action=export` is pushed FIRST so the export contains its own
    /// event. The same bytes serve the file, the Copy button and the harness capture.
    pub fn export_text(&self, build_commit: &str, label: &str, exported_utc: &str) -> String {
        let _ = self.push_desktop("gw.log", &[("action", "export")]);
        let g = self.lock();
        let level = match g.level {
            DebugLogLevel::Events => "events",
            DebugLogLevel::Detailed => "detailed",
        };
        let mut body = String::new();
        body.push_str(&format!(
            "# app=qsl-desktop version={} build_commit={} qsc_pin={}\n",
            env!("CARGO_PKG_VERSION"),
            ascii_token(build_commit),
            QSC_PIN
        ));
        body.push_str(&format!(
            "# level={} exported_utc={} cap={} buffered={} dropped={} seq_first={} seq_last={} label={} format={}\n",
            level,
            exported_utc,
            RING_CAP,
            g.ring.len(),
            g.dropped,
            g.ring.front().map(|s| s.seq).unwrap_or(0),
            g.ring.back().map(|s| s.seq).unwrap_or(0),
            ascii_token(label),
            FORMAT
        ));
        body.push_str(&format!("# digest={}\n", DIGEST_CONSTRUCTION));
        for s in g.ring.iter() {
            body.push_str(&s.line);
            body.push('\n');
        }
        drop(g);
        let digest = sha256_hex(body.as_bytes());
        body.push_str(&format!("# sha256={}\n", digest));
        debug_assert!(body.is_ascii());
        body
    }

    /// THE ONLY DISK WRITE of the log: one file, created new at 0600 in the directory the
    /// operator chose, named `qsl-desktop-debug-log-<utc>-<label>.txt`.
    pub fn export_to_dir(&self, dir: &Path, build_commit: &str) -> Result<ExportDto, String> {
        if !dir.is_dir() {
            return Err("export_dir_not_a_directory".to_string());
        }
        let label = random_label()?;
        let now = now_ms();
        let exported_utc = engine::utc_rfc3339_ms(now);
        let text = self.export_text(build_commit, &label, &exported_utc);
        let name = format!(
            "qsl-desktop-debug-log-{}-{}.txt",
            compact_utc(&exported_utc),
            label
        );
        let path: PathBuf = dir.join(name);
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .map_err(|e| format!("export_create_failed: {e}"))?;
        f.write_all(text.as_bytes())
            .map_err(|e| format!("export_write_failed: {e}"))?;
        f.sync_all()
            .map_err(|e| format!("export_sync_failed: {e}"))?;
        Ok(ExportDto {
            path: path.to_string_lossy().into_owned(),
            bytes: text.len(),
            sha256: sha256_hex(text.as_bytes()),
            label,
        })
    }
}

/// Verify an export's footer against its own construction: `sha256(all bytes before the
/// footer line)` must equal the footer's hex. One flipped byte anywhere before the footer, or
/// a truncated file, fails here.
pub fn verify_export(text: &str) -> bool {
    let Some(idx) = text.trim_end_matches('\n').rfind('\n') else {
        return false;
    };
    let (body, footer) = text.split_at(idx + 1);
    let footer = footer.trim_end_matches('\n');
    let Some(hex) = footer.strip_prefix("# sha256=") else {
        return false;
    };
    hex.len() == 64 && sha256_hex(body.as_bytes()) == hex
}

/// Install the engine sink: every marker the engine emits reaches `DebugLog::global()` as a
/// typed event. The slot is process-global (one sink); the closure captures nothing.
pub fn install_sink() {
    engine::set_event_sink(Some(Box::new(|ev: &Event| {
        DebugLog::global().push(ev);
    })));
}

pub fn uninstall_sink() {
    engine::set_event_sink(None);
}

pub fn sink_installed() -> bool {
    engine::event_sink_installed()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// `2026-09-05T15:44:01.140Z` -> `20260905T154401Z`: the file name's stamp.
pub fn compact_utc(rfc3339_ms: &str) -> String {
    let mut out = String::new();
    for c in rfc3339_ms.chars() {
        match c {
            '-' | ':' => {}
            '.' => break,
            c => out.push(c),
        }
    }
    if !out.ends_with('Z') {
        out.push('Z');
    }
    out
}

/// A header token is printed as-is only if it is a plain ASCII word; anything else (which no
/// caller produces) collapses to `?` so the header stays ASCII and single-token by construction.
fn ascii_token(s: &str) -> &str {
    if !s.is_empty()
        && s.is_ascii()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'))
    {
        s
    } else {
        UNLISTED
    }
}

/// The export label: 16 hex from the OS RNG, minted per export, the ONLY correlator (never an
/// identity). Linux-only app (D-A / L9): `/dev/urandom`, no crate.
fn random_label() -> Result<String, String> {
    use std::io::Read;
    let mut f =
        std::fs::File::open("/dev/urandom").map_err(|e| format!("os_rng_unavailable: {e}"))?;
    let mut b = [0u8; 8];
    f.read_exact(&mut b)
        .map_err(|e| format!("os_rng_short_read: {e}"))?;
    Ok(hex(&b))
}

fn hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 15) as usize] as char);
    }
    s
}

// ---- SHA-256 (FIPS 180-4), self-contained: this crate declares no hashing dependency and the
// ---- engine re-exports none. Tested against the published vectors below.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().enumerate().take(16) {
            *word = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

pub fn sha256_hex(data: &[u8]) -> String {
    hex(&sha256(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIPS 180-4 vectors: the empty message and "abc".
    #[test]
    fn sha256_matches_the_published_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn compact_utc_strips_punctuation_and_millis() {
        assert_eq!(compact_utc("2026-09-05T15:44:01.140Z"), "20260905T154401Z");
        assert_eq!(compact_utc("1970-01-01T00:00:00.000Z"), "19700101T000000Z");
    }

    #[test]
    fn the_pin_constant_equals_the_manifest_rev() {
        let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .expect("Cargo.toml");
        let needle = format!("rev = \"{}\"", QSC_PIN);
        assert!(
            manifest.contains(&needle),
            "src-tauri/Cargo.toml must pin qsc at QSC_PIN ({QSC_PIN})"
        );
    }

    #[test]
    fn the_command_table_equals_the_registered_set() {
        let lib = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
            .expect("lib.rs");
        let block = &lib[lib.find("generate_handler!").expect("handler list")..];
        let block = &block[..block.find("])").expect("handler list end")];
        let mut registered: Vec<String> = block
            .lines()
            .filter_map(|l| {
                let t = l.trim().trim_end_matches(',');
                let t = t.strip_prefix("commands::").unwrap_or(t);
                (!t.is_empty()
                    && t.chars()
                        .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()))
                .then(|| t.to_string())
            })
            .collect();
        registered.sort();
        let mut table: Vec<String> = COMMAND_NAMES.iter().map(|s| s.to_string()).collect();
        table.sort();
        assert_eq!(
            table, registered,
            "COMMAND_NAMES must equal lib.rs's registered set"
        );
        assert!(
            COMMAND_NAMES.windows(2).all(|w| w[0] < w[1]),
            "COMMAND_NAMES is sorted"
        );
    }
}

//! Startup rule (b): the qsc marker queue is unbounded (investigation R1), so
//! the app drains it completely after EVERY core call into this bounded
//! buffer. Overflow drops the oldest line and counts the drop visibly —
//! honest, never silent.

use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub const MARKER_BUFFER_CAP: usize = 1024;

/// The test-only marker injection seam's env name. `QSLD_` family, harness-set, never
/// IPC-reachable. ⚠ OBSERVATION FOR THE RECORD: this is a FOURTH production-compiled
/// test-seam environment variable, and `ENG-0127` already files that family for
/// release-build gating as a CLASS. It joins that class rather than escaping it.
pub const INJECT_ENV: &str = "QSLD_INJECT_MARKER";

/// NA-0776 (spec v2 3.3) -- THE BINDING WHITELIST. A POSITIVE allowlist over the ~358
/// `emit_marker` call sites in 15 qsc modules: a marker added upstream reaches the UI
/// only by an explicit edit here. Ruled a PRIVACY PROPERTY of the surface, not a style
/// choice (RULING_003 R5 / cold read NOTE-3).
pub const NOTICE_KINDS: &[&str] = &["invite_finish_hs_unconsumed"];

/// The classifier: PURE, and unit-tested against REAL lines from BOTH format arms
/// (cold read MAJOR-8). The return type is `&'static str` taken FROM `NOTICE_KINDS`:
/// the VALUE the UI receives is always a whitelist member, never text carried out of
/// the marker line.
pub fn classify(line: &str) -> Option<&'static str> {
    let event: String = if let Some(rest) = line.strip_prefix("QSC_MARK/1 event=") {
        rest.split_whitespace().next().unwrap_or("").to_string()
    } else if line.starts_with('{') {
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => {
                let e = v.get("event").and_then(|e| e.as_str())?;
                e.to_string()
            }
            Err(_) => return None,
        }
    } else {
        return None;
    };
    NOTICE_KINDS.iter().copied().find(|k| **k == *event)
}

pub struct MarkerBuffer {
    buf: Mutex<VecDeque<String>>,
    dropped: AtomicU64,
    cap: usize,
    /// NA-0776 (3.3 / cold read MAJOR-2): a PER-KIND MONOTONIC counter, incremented as
    /// each line arrives. The ring buffer is a DIAGNOSTIC structure -- 1024 slots shared
    /// with ~358 marker kinds, dropping the oldest -- and a user-facing tally read off it
    /// would under-report systematically and could DECREASE between two polls. These
    /// counters are exact and eviction-immune.
    counts: Mutex<BTreeMap<&'static str, u64>>,
    /// Per-kind DISMISS WATERMARK. Displayed = count - watermark. It lives on the Rust
    /// side deliberately: a front-end watermark sits at main.js module scope and is
    /// destroyed by `window.location.reload()` (main.js:628, :1151), so the notice would
    /// resurrect after every erase or destroy (cold read MINOR-11). Consuming buffer
    /// entries instead would change `marker_stats.buffered` -- one command's meaning
    /// altered by another's UI gesture -- so `marker_stats` is left UNTOUCHED and
    /// ENG-0121's limbs stay dormant (R5).
    dismissed: Mutex<BTreeMap<&'static str, u64>>,
}

impl Default for MarkerBuffer {
    fn default() -> Self {
        Self::with_cap(MARKER_BUFFER_CAP)
    }
}

impl MarkerBuffer {
    pub fn with_cap(cap: usize) -> Self {
        MarkerBuffer {
            buf: Mutex::new(VecDeque::new()),
            dropped: AtomicU64::new(0),
            cap,
            counts: Mutex::new(BTreeMap::new()),
            dismissed: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn push(&self, line: String) {
        // Count BEFORE storing: the tally must not depend on whether the line survives
        // the ring buffer's eviction.
        if let Some(kind) = classify(&line) {
            let mut c = self.counts.lock().unwrap_or_else(|p| p.into_inner());
            *c.entry(kind).or_insert(0) += 1;
        }
        let mut g = self.buf.lock().unwrap_or_else(|p| p.into_inner());
        if g.len() >= self.cap {
            g.pop_front();
            self.dropped.fetch_add(1, Ordering::SeqCst);
        }
        g.push_back(line);
    }

    /// Drain qsc's queue completely into this buffer.
    pub fn drain_from_core(&self) {
        let q = qsc::output::marker_queue();
        let mut g = q.lock().unwrap_or_else(|p| p.into_inner());
        while let Some(line) = g.pop_front() {
            drop(g);
            self.push(line);
            g = q.lock().unwrap_or_else(|p| p.into_inner());
        }
    }

    /// The surface: kinds whose count exceeds their dismiss watermark, with the
    /// UNDISMISSED remainder. `{kind, count}` and nothing else -- no timestamps, which
    /// have no source here and would be per-attempt timing metadata acquired as a DTO
    /// side effect (cold read BLOCKER-4 / NOTE-4).
    pub fn notices(&self) -> Vec<(&'static str, u64)> {
        let c = self.counts.lock().unwrap_or_else(|p| p.into_inner());
        let d = self.dismissed.lock().unwrap_or_else(|p| p.into_inner());
        c.iter()
            .filter_map(|(k, n)| {
                let seen = d.get(k).copied().unwrap_or(0);
                n.checked_sub(seen).filter(|r| *r > 0).map(|r| (*k, r))
            })
            .collect()
    }

    /// Dismiss: the watermark moves to the current count. Anything that arrives after
    /// this call is shown again. A kind outside the whitelist is ignored rather than
    /// stored, so this cannot be used to grow the map from the front end.
    pub fn dismiss(&self, kind: &str) {
        let Some(k) = NOTICE_KINDS.iter().copied().find(|k| **k == *kind) else {
            return;
        };
        let c = self.counts.lock().unwrap_or_else(|p| p.into_inner());
        let now = c.get(&k).copied().unwrap_or(0);
        drop(c);
        let mut d = self.dismissed.lock().unwrap_or_else(|p| p.into_inner());
        d.insert(k, now);
    }

    /// NA-0776 (3.3) -- THE TEST-ONLY INJECTION SEAM, and why it is an env var rather
    /// than a `#[cfg(test)]` method: the driver arm drives the REAL BINARY, where a
    /// cfg-test method does not exist. This follows the `QSLD_TICK_MS` precedent
    /// (settings.rs) exactly -- a `QSLD_`-family env the harness sets per launch -- and
    /// it is NOT reachable from the front end: no command reads or writes it, which is
    /// the constraint commands.rs:813 places on the clock seam.
    pub fn inject_from_env(&self) {
        if let Ok(line) = std::env::var(INJECT_ENV) {
            if !line.is_empty() {
                self.push(line);
            }
        }
    }

    pub fn stats(&self) -> (usize, u64) {
        let g = self.buf.lock().unwrap_or_else(|p| p.into_inner());
        (g.len(), self.dropped.load(Ordering::SeqCst))
    }

    #[cfg(test)]
    pub fn snapshot(&self) -> Vec<String> {
        let g = self.buf.lock().unwrap_or_else(|p| p.into_inner());
        g.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_drops_oldest_and_counts() {
        let b = MarkerBuffer::with_cap(3);
        for i in 0..5 {
            b.push(format!("m{i}"));
        }
        let (len, dropped) = b.stats();
        assert_eq!(len, 3);
        assert_eq!(dropped, 2);
        assert_eq!(b.snapshot(), vec!["m2", "m3", "m4"]);
    }
}

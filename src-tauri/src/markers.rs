//! Startup rule (b): the qsc marker queue is unbounded (investigation R1), so
//! the app drains it completely after EVERY core call into this bounded
//! buffer. Overflow drops the oldest line and counts the drop visibly —
//! honest, never silent.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub const MARKER_BUFFER_CAP: usize = 1024;

pub struct MarkerBuffer {
    buf: Mutex<VecDeque<String>>,
    dropped: AtomicU64,
    cap: usize,
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
        }
    }

    pub fn push(&self, line: String) {
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

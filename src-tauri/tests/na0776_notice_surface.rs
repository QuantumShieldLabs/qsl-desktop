//! NA-0776 (spec v2 sec 3.3) -- ENG-0274: the declined-frame notice.
//! Arms per the cold read: BLOCKER-4 (the DTO carries no timestamps), MAJOR-2 (the count
//! is monotonic and eviction-immune), MAJOR-8 (the classifier is driven with REAL lines
//! from BOTH format arms, not hand-typed ones), MINOR-11 (the watermark), and R5/NOTE-3
//! (the whitelist is a privacy property).

use qsl_desktop_app::commands::NoticeDto;
use qsl_desktop_app::markers::{classify, MarkerBuffer, NOTICE_KINDS};

const KIND: &str = "invite_finish_hs_unconsumed";

/// Produce a marker line through QSC'S OWN FORMATTER in the named format, so the
/// classifier is tested against what actually arrives rather than against a string this
/// test made up. Both arms run inside ONE test: the format is process-global env, and
/// mutating it from parallel tests would race.
fn real_line(format: &str, event: &str) -> String {
    std::env::set_var("QSC_MARK_FORMAT", format);
    qsc::output::init_output_policy(false);
    qsc::output::set_marker_routing(qsc::output::MarkerRouting::InApp);
    let q = qsc::output::marker_queue();
    q.lock().unwrap_or_else(|p| p.into_inner()).clear();
    qsc::output::emit_marker(event, None, &[("candidates", "2")]);
    let mut g = q.lock().unwrap_or_else(|p| p.into_inner());
    g.pop_front().expect("qsc emitted no line")
}

/// A3 -- the arm the spec's v1 injection test was structurally blind to: injection
/// bypasses the formatter, so only this can catch a format the classifier cannot read.
#[test]
fn classifier_reads_real_lines_from_both_format_arms() {
    let plain = real_line("plain", KIND);
    assert!(plain.starts_with("QSC_MARK/1 event="), "not the plain form: {plain}");
    assert_eq!(classify(&plain), Some(KIND), "plain arm not classified: {plain}");

    let jsonl = real_line("jsonl", KIND);
    assert!(jsonl.starts_with('{'), "not the jsonl form: {jsonl}");
    assert_eq!(classify(&jsonl), Some(KIND), "jsonl arm not classified: {jsonl}");

    // and a non-whitelisted event, through the real formatter, in both forms
    for f in ["plain", "jsonl"] {
        let other = real_line(f, "relay_ack");
        assert_eq!(classify(&other), None, "a non-whitelisted marker classified: {other}");
    }
    std::env::set_var("QSC_MARK_FORMAT", "plain");
}

/// A2 / R5 -- the whitelist is POSITIVE and the classifier cannot return input text.
/// The returned &'static str is the whitelist's OWN entry: proven by pointer identity,
/// so no slice of the line can ever reach the UI even by mistake.
#[test]
fn the_returned_kind_is_the_whitelists_own_static_and_never_input_text() {
    let line = format!("QSC_MARK/1 event={KIND} candidates=7");
    let got = classify(&line).expect("whitelisted kind must classify");
    assert_eq!(got, KIND);
    assert!(NOTICE_KINDS.contains(&got), "the returned kind is not a whitelist member");
    // The property is "NOT A SLICE OF THE INPUT". Address-range containment tests that
    // directly.
    // ⚠ An earlier version of this arm asserted pointer EQUALITY against the test
    // crate's own `NOTICE_KINDS`. That was the INSTRUMENT being wrong, not the code:
    // `NOTICE_KINDS` is a `const`, so it is inlined per use site and the test crate's
    // copy need not share an address with the library's. Recorded rather than quietly
    // rewritten.
    let (lo, hi) = (line.as_ptr() as usize, line.as_ptr() as usize + line.len());
    let got_at = got.as_ptr() as usize;
    assert!(got_at < lo || got_at >= hi,
        "the classifier returned a slice of the INPUT line -- raw marker text could reach the UI");
    for junk in ["", "not a marker", "QSC_MARK/1 event=", "{", "{\"event\":123}",
                 "QSC_MARK/1 event=some_other_kind x=1"] {
        assert_eq!(classify(junk), None, "junk classified: {junk:?}");
    }
}

/// MAJOR-2 -- the count is MONOTONIC and immune to the ring buffer's eviction. Read off
/// the buffer instead, it would under-report and could DECREASE between two polls.
#[test]
fn count_is_exact_under_eviction() {
    let b = MarkerBuffer::with_cap(3);
    for _ in 0..5 {
        b.push(format!("QSC_MARK/1 event={KIND} candidates=1"));
    }
    let (buffered, _dropped) = b.stats();
    assert_eq!(buffered, 3, "fixture must actually evict, or this arm proves nothing");
    assert_eq!(b.notices(), vec![(KIND, 5)], "the tally followed the buffer, not the truth");
}

/// MINOR-11 -- dismiss is a Rust-side WATERMARK: it hides what has been seen, and a NEW
/// arrival brings the notice back. A front-end watermark would be destroyed by the
/// reload that erase and destroy both perform.
#[test]
fn dismiss_watermarks_and_later_arrivals_reappear() {
    let b = MarkerBuffer::with_cap(64);
    for _ in 0..3 {
        b.push(format!("QSC_MARK/1 event={KIND} candidates=1"));
    }
    assert_eq!(b.notices(), vec![(KIND, 3)]);
    b.dismiss(KIND);
    assert!(b.notices().is_empty(), "dismiss did not clear the surface");
    b.push(format!("QSC_MARK/1 event={KIND} candidates=1"));
    assert_eq!(b.notices(), vec![(KIND, 1)], "a NEW arrival must resurface the notice");
    // marker_stats' meaning is untouched by the UI gesture: nothing was consumed
    let (buffered, _) = b.stats();
    assert_eq!(buffered, 4, "dismiss consumed buffer entries -- marker_stats would drift");
}

#[test]
fn dismiss_ignores_kinds_outside_the_whitelist() {
    let b = MarkerBuffer::with_cap(8);
    b.push(format!("QSC_MARK/1 event={KIND} candidates=1"));
    b.dismiss("relay_ack");
    b.dismiss("");
    assert_eq!(b.notices(), vec![(KIND, 1)], "an unrelated dismiss altered the surface");
}

/// BLOCKER-4 / NOTE-4 -- the DTO is {kind, count}. The two time fields were specified in
/// v1, have no source, and are per-attempt timing metadata; their ABSENCE is pinned so a
/// later refactor cannot quietly reintroduce them.
/// ⚠ BOUNDARY: a tripwire over the named fields, not a universal ban -- a differently
/// named timestamp would pass, and adding one is a ruling, not a slip this can catch.
#[test]
fn the_dto_is_kind_and_count_only() {
    let v = serde_json::to_value(NoticeDto { kind: KIND, count: 3 }).unwrap();
    let obj = v.as_object().expect("NoticeDto serializes as an object");
    let mut keys: Vec<_> = obj.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, vec!["count", "kind"], "the DTO grew a field");
    for absent in ["first_seen_ms", "last_seen_ms", "ts", "at", "when"] {
        assert!(!obj.contains_key(absent), "the DTO carries {absent:?}");
    }
}

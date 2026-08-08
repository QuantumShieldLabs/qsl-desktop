# The GUI input driver (NA-0701 / D636-as-amended · spine D-1341 · desktop D-0026)

WebDriver clicks, keystrokes, and verbatim reads over the RUNNING desktop app,
so a behavioral GUI defect fails a test instead of waiting for the operator's
live flight. It caught one before it ever landed: the erase-form error write
skips the R-14 window resize, leaving both ceremony buttons unclickable after a
wrong phrase (ENG row on the spine ledger; found by the F-E scenario's original
in-place shape, which now relaunches instead — the fix-lane restores it).

## What runs

Six `#[ignore]`-marked tests in `src-tauri/tests/gui_driver.rs`
(`na0701_gui_{a..f}_*`, pinned BY NAME in `scripts/ci/EXPECTED_TEST_INVENTORY.txt`),
each invoking `src-tauri/tests/harness/runner.py` on its scenario file in
`src-tauri/tests/harness/scenarios/`. Plain `cargo test` shows them honestly as
`ignored`; their real execution is:

    cargo test --test gui_driver -- --ignored --test-threads=1

SERIALIZED — one app instance at a time (the NA-0700 C4 race lesson). That is
exactly what the `gui-driver` CI job runs (non-required at landing, the NA-0698
pattern; `timeout-minutes: 30` is the D636 STOP threshold with teeth).

## The recipe is PER-PRODUCER (SR-20 ext)

Every element below is a MEASURED fact on the producer it runs on, never an
assumption carried across producers. The box record lives in the NA-0701
operator record (STOP_NA0701_002/007); the CI runner re-measures everything and
**its first green is a finding** — diff the job's producer-identity step
against the box figures:

| element | box (measured) | CI (re-measured every run) |
|---|---|---|
| rustc | 1.95.0 (59807616e) | producer-identity step |
| tauri-driver | **2.0.6, PINNED `--locked`** (a bump is a lane, not a drift) | same pin, cold install |
| webkit2gtk-driver | 2.52.3-0ubuntu0.24.04.1 | apt-resolved, recorded |
| display | xvfb-run -a -s "-screen 0 1280x800x24" | same |
| session bus | dbus-run-session | same |
| backend | GDK_BACKEND=x11 | same |
| automation env | **NEITHER is set by the harness** (R164 §3: tauri-driver v2.0.6 itself injects `TAURI_AUTOMATION` + `TAURI_WEBVIEW_AUTOMATION`, `webdriver.rs:50-51`; the runner additionally `env`-removes both) | same fact, re-measured via the pin |
| ports | two INDEPENDENT port-0 probes, both passed explicitly as `--port`/`--native-port`; 3-attempt bind retry, each recorded | same mechanism |

## Harness duties (enforced in `runner.py`, in this order)

1. **Run root (A2.1):** `QSLD_GUI_RUN_ROOT`, default
   `<repo-root>/target/gui_driver_runs/<utc>/` — repo-relative, gitignored by
   the `/target` rule. The shared cargo target cache is NEVER a data or
   evidence home (A2.2); only the app binary is read from `CARGO_TARGET_DIR`.
2. **The standing isolation bracket (A1.1):** every invocation writes a
   real-`$HOME` candidate-dir census pre and post; the two must be
   BYTE-IDENTICAL; destructive steps (the erase legs) are REFUSED if the
   pre-census is absent. F-E additionally asserts `vault.qsv` +
   `settings.json` PRESENT in the per-run profile before `#link-forgot` —
   presence-before/absence-after, so an isolation lapse reds a presence row
   instead of greening a residue row.
3. **Bounded polls only.** No sleeps anywhere: every wait is a 1s-interval
   bounded poll with its count recorded in the verdict row (the 30s erase
   countdown is POLLED on `#countdown-number`; a real run observes ~26
   distinct values).
4. **Teardown on EVERY exit path (A1.15):** DELETE /session → SIGTERM the
   MEASURED pgid (the runner starts the stack with `start_new_session`, so the
   leader's pgid is measured from the live process) → comm-name pgrep census
   (`Xvfb|tauri-driver|WebKitWebDriver|qsl-desktop`) must be EMPTY.
5. **Verdicts:** json.dumps JSONL — step rows
   `{"step","expected","measured","verdict","evidence"}` + one terminal row
   `{"scenario","result","steps"}`; the cargo wrapper asserts the
   Phase-0-validated consumer contract. The verdict log is FROZEN first, the
   run log next, `MANIFEST.json` (sha256 per artifact) is written LAST
   (R164 §5).
6. **Liveness every run (P9):** an absent selector must yield
   `no such element`, and a deliberately wrong expected text must miscompare —
   the harness proves it can go red before any green is trusted.
7. `QSLD_CONTINUE_ON_FAIL=1` is a PERTURBATION-MEASUREMENT facility only
   (records every row instead of aborting at the first red; launch/session
   rows stay hard). It is structurally unreachable from `cargo test`: the
   wrapper `env_remove`s it (R171 3.4).

## ⚠ THE REBUILD BRACKET (R171 §2 — binding on every ui/ perturbation in this program)

`tauri.conf.json` sets `frontendDist: "../ui"`, so `generate_context!` EMBEDS
the ui assets into the binary at BUILD time (brotli — a plaintext grep of the
binary proves nothing). **An edit to `ui/` does NOT reach a running test until
the binary is rebuilt.** Empirically proven in the lane (the `dev_embed_probe`:
perturbed file on disk, unperturbed literal served). Every scenario-control
perturbation therefore runs the full bracket:

    edit → cargo build → run → git checkout (restore) → cargo build

Without the bracket, a ui/ perturbation is a silent no-op and its control
"passes" while testing nothing — the gate-that-cannot-go-red class.

## ⚠ THE CONSUMER-CENSUS INSTRUMENT (R172 2.2 — before predicting any persistence-perturbation red set)

Before committing the red set of a perturbation that changes a PERSISTED
ARTIFACT, enumerate every call site of that artifact's accessor
(tooling-extracted, whole-tree) and walk each site into the flows; zero
unwalked sites is the gate. Worked example from this lane: `settings_file` has
THREE production consumers — `settings.rs` load/save, the destroy/erase
removals in `commands.rs`, and **`state.rs:75`, where `settings.json`
existence is a CONJUNCT of the S2 launch state** — the third is why breaking
`saveSettings()` reds F-C (its relaunch boots S1 and the correct unlock
re-offers the identity step) and not just F-D/F-E. Two independent derivations
missed it; the census finds it mechanically.

## Running locally

    # tauri-driver 2.0.6 on PATH (or QSLD_TAURI_DRIVER=<abs path>)
    cargo build
    cargo test --test gui_driver -- --ignored --test-threads=1

Evidence lands under `target/gui_driver_runs/<utc>/<scenario>/` — verdict.jsonl,
run_log.txt, http_transcript.jsonl, censuses, screenshots, MANIFEST.json.

## Not claimed (A1.22)

Full-surface behavioral coverage (the census reads everything; only the six
flows go deep) · `scr-wiped`'s display path (armed wipe — a filed successor,
as is `destroy_vault` end-to-end) · native GTK menu items / WM behavior
(operator-flown; F-F proves the FE listen handlers + `app.emit` plumbing via
execute/sync with ZERO IPC change — commands stay 27) · perceptual diff
(deferred severable) · macOS/Windows · required-status promotion (the
operator's later branch-protection act).

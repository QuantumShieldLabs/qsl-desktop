# DECISIONS (qsl-desktop)

This log records repo-local decisions only. All directive, queue, and
decision AUTHORITY lives in the qsl-protocol governance spine (see
CLAUDE.md); spine decisions about this repository are recorded in the spine's
DECISIONS.md (registration: D-1279; bootstrap: D-1280).

- **ID:** D-0001
  - **Status:** Accepted
  - **Date:** 2026-07-19
  - **Decision:** Bootstrap this repository as a SATELLITE of the
    qsl-protocol spine per the spine's D-1279 registration (DOC-PROG-004
    v0.2.0 step 3): land the single required `rust` CI gate
    (.github/workflows/ci.yml; branch protection follows the first merge as
    the operator's companion step), the community-health set (LICENSE,
    NOTICE, README, SECURITY, CODE_OF_CONDUCT, CONTRIBUTING), the pointer
    CLAUDE.md (the repository's root commit), this DECISIONS log, and a
    minimal zero-dependency placeholder crate (`qsl-desktop`, version-line
    binary + one unit test + committed Cargo.lock) so the gate checks
    something real. No Tauri, no GUI code, no qsc dependency — the GUI
    skeleton is a future spine-governed lane.
  - **Rationale:** The repository's first commits must be governed work; the
    CI pipeline must be proven before the application exists (prove the
    pipeline, not the app).
  - **References:** spine NA-0657 (directive D593); spine D-1279 (the
    registration and its owed list) and D-1280 (the bootstrap closeout);
    spine D-1278/NA-0655 (community-health forms); spine D-1265/D578 (the
    qsl-server satellite pattern).

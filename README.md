# qsl-desktop

Desktop client (GUI) for the QSL protocol — in development; nothing runnable
yet.

## What this is
- The desktop client repository for QSL, a research-stage post-quantum
  secure messaging protocol (see the
  [qsl-protocol](https://github.com/QuantumShieldLabs/qsl-protocol) spine).
- Planned shape: a Tauri v2 shell with the `qsc` client core linked
  in-process as a rev-pinned Rust library (no sidecar).
- v1 targets Linux only (roadmap decision D-A / locked decision L9); macOS
  is the first post-skeleton platform lane, Windows sits at a later horizon.

## What this is not (yet)
- There is no GUI, no messaging functionality, and no release.
- The only binary here is a bootstrap placeholder that prints a version
  line; it exists so the CI gate checks something real.
- No security, privacy, or availability claims are made for anything in
  this repository.

## Status
Bootstrap stage: repository infrastructure only (CI gate, community-health
and governance files, placeholder crate). The GUI skeleton is a future,
separately-governed lane.

## Governance
This repository is a satellite of
[qsl-protocol](https://github.com/QuantumShieldLabs/qsl-protocol); all
directive, queue, and decision authority lives in that spine (see CLAUDE.md
and DECISIONS.md). Changes land only through spine-governed lanes.

## License
AGPL-3.0-only; see LICENSE and NOTICE.

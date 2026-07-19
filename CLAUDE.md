# CLAUDE.md — qsl-desktop (thin pointer; governance is authoritative)

Goals: G4 (primary), supports G1–G5

This repository is a satellite of qsl-protocol. All directive, queue, and
decision authority lives in the qsl-protocol governance spine; work here
occurs only when a qsl-protocol NA directive explicitly authorizes it.

Read first: README.md and DECISIONS.md in this repo, then the qsl-protocol
spine (START_HERE.md, GOALS.md, AGENTS.md, CODEX_RULES.md, NEXT_ACTIONS.md,
docs/ops/DIRECTOR_OPERATIONS.md). Every rule addressed to "Codex" binds you.

Hard boundaries for this repo:
- qsl-desktop is the DESKTOP CLIENT (GUI) satellite. v1 is Linux-only
  (roadmap decision D-A / locked decision L9).
- qsc will be consumed as a REV-PINNED git dependency once the GUI skeleton
  lane introduces it; pin advances are deliberate spine-governed bump lanes.
  No qsc dependency exists at bootstrap.
- Fail-closed everywhere; no best-effort parsing.
- No dependency/lockfile/workflow mutation unless the active qsl-protocol
  NA lane explicitly authorizes it.
- Merge commits only; no squash/rebase/force-push/amend after PR creation.
- Publish class summaries only; raw private values remain proof-root-only.
- No public/production/security-completion claims; the README's
  nothing-runnable-yet posture holds until a spine lane changes it.

Precedence: this file is a pointer only; the governance spine wins on any
conflict.

# qsl-desktop

Desktop client (GUI) for the QSL protocol — in development, pre-release.

## What this is
- The desktop client repository for QSL, a research-stage post-quantum
  secure messaging protocol (see the
  [qsl-protocol](https://github.com/QuantumShieldLabs/qsl-protocol) spine).
- A Tauri v2 shell with the `qsc` client core linked in-process as a
  rev-pinned Rust library (no sidecar), with a static vanilla HTML/CSS/JS
  frontend (no npm, no node, no JS dependencies).
- v1 targets Linux only (roadmap decision D-A / locked decision L9); macOS
  is the first post-skeleton platform lane, Windows sits at a later horizon.

## Status: local lifecycle, relay configuration, invites; no messaging
This build contains the local vault/identity/unlock lifecycle —
onboarding (create vault, identity display), the unlock screen with its
escalating-delay protection display, idle autolock, and the Vault &
Security settings — plus the Settings › Relay pane, where a relay
address, an access token, and a CA certificate file are configured. One
**Save** commits the pane; **Test connection** saves first and then
reports what the relay actually answered. (That commit model is a
revision of the pane's first version, which committed each field through
its own button.)

Since PR #36 the app can also **create, review and revoke invites**: one
button mints a single-use code with a 3-day expiry, shown once, with an
optional local-only note naming who it is for; a list shows the live
invites (10 maximum) and their state, including when one has been
redeemed. Since PR #37 it can also **redeem** one: paste a code, name
the contact, and the app completes the handshake and records the peer.
Creating, revoking and redeeming an invite all contact the configured
relay.

**The app connects only when you ask it to** — pressing Test
connection, or creating, revoking or redeeming an invite. Nothing
connects at launch, in the background, or on a timer.

There is still **no messaging** — no sending, no receiving — and no
release. An invite can be created, reviewed, revoked and redeemed;
redeeming one completes a handshake and records the contact, but there
is no contact list to browse and nothing can be sent to it yet.

No security, privacy, or availability claims are made for anything in this
repository beyond factual feature description; the app's status line shows
only what is actually true.

## One profile, one program (R8)
Do not run the qsc CLI and this app against the same profile. The app
keeps its own application-scoped data directory, separate from the
CLI's default configuration directory, which makes collision unlikely
by default — but nothing synchronizes concurrent writers, so: one
profile, one program.

## Building (Linux)
Requires stable Rust and the Tauri v2 system libraries (Ubuntu 24.04
package names): `libwebkit2gtk-4.1-dev libgtk-3-dev
libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev
libjavascriptcoregtk-4.1-dev`. Then `cargo build` / `cargo test` at the
repository root. The binary is `qsl-desktop`.

## Governance
This repository is a satellite of
[qsl-protocol](https://github.com/QuantumShieldLabs/qsl-protocol); all
directive, queue, and decision authority lives in that spine (see CLAUDE.md
and DECISIONS.md). Changes land only through spine-governed lanes.

## License
AGPL-3.0-only; see LICENSE and NOTICE.

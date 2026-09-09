# qsl-desktop

Goals: G4

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

## Local self-invitation rejection (NA-0780)
Connect checks the submitted invitation locally before redemption. An invitation
owned by this app is refused with the banner “You can’t connect to yourself”
and helper “Please share this invitation code with your intended contact.”
Locked, unavailable invitation storage, and invalid-code errors also stop the
attempt, with separate explanations.
The existing redemption handler repeats ownership and lock checks; preflight
success does not authenticate the sender or authorize a later attempt.

Changing either field or closing the window invalidates an outstanding preflight.
Only one Connect attempt runs at a time, and late results cannot repaint a closed
or reopened window. Closing after redemption has started does not undo the request.

The encrypted vault retains recoverable ownership through visible-history clearing,
restart and identity rotation. Ownership deleted before the engine upgrade cannot
be recovered; restoring an older vault backup restores its older history. Full vault
erasure removes that history. Automated fixtures do not establish two-machine
acceptance or resolve the lane's other invitation-reliability findings.

### Operator check
Build the PR with `cargo build --locked` using the repository's pinned toolchain,
then run the resulting `qsl-desktop` binary on each Linux machine. In the managed
build tree, export `QBUILD_ROOT` as the absolute path to the build-tree root (the
directory containing `state/tools/env_qbuild.sh`). Keep its machine-specific value
in your local shell. Select the shared target and build in a child shell that exits
on failure before attempting to launch:

```bash
bash -c 'set -e
: "${QBUILD_ROOT:?Export QBUILD_ROOT as the absolute build-tree root}"
[[ "$QBUILD_ROOT" = /* ]]
source "$QBUILD_ROOT/state/tools/env_qbuild.sh"
qbuild_export_repo_env qsl-desktop
: "${CARGO_TARGET_DIR:?Environment selection did not set CARGO_TARGET_DIR}"
[[ "$CARGO_TARGET_DIR" = /* ]]
cargo build --locked
exec "$CARGO_TARGET_DIR/debug/qsl-desktop"'
```

Run from the checked-out PR repository. On another Linux build host, with the
required Tauri libraries and pinned toolchain already available, use
`cargo build --locked` and `./target/debug/qsl-desktop` from that checkout (or the
binary under its configured `CARGO_TARGET_DIR`). Use a separate profile on each
machine and the same reviewed desktop commit.

1. Unlock the app and configure/test the intended relay on both machines. Enable
   the existing detailed debug log before the test if diagnostic exports are needed.
2. On A, create an invitation for B. Paste it into A's Connect window, give it a
   valid local name, and press Connect. Verify the exact self-invitation message,
   no pending contact, and no redemption call in the local debug log. Repeat with
   an older invitation after closing and restarting A. A background scan may log
   independently; it is not a redemption caused by this refusal.
3. Send A's invitation privately to B. On B, paste it into Connect, name A, and
   press Connect once. Keep both apps unlocked; follow the existing incoming
   request/approval flow on A. Verify that both contact states reach connected.
   “Request sent” alone does not establish connection completion.
4. Record the desktop commit, engine revision, each observed result and both
   privacy-safe debug-log exports. Keep invitation codes and private profile files
   out of reports. A failure to complete remains an observation for the lane's
   separate reliability analysis, not an acceptance result.

CI waits use the installed `state/tools/bin/qci-wait` helper, pinned to the exact
repository, PR and full head SHA, with each known check passed via `--expect`.
Keep its new log in lane evidence; use its default 60-second interval and 60-minute
limit. Pending, missing, failed, cancelled and skipped checks stay distinct. Never
rerun jobs or merge automatically.

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

[README.md](https://github.com/user-attachments/files/32136468/README_qsl-desktop.md)

# QSL Desktop

**The desktop client for QSL — post-quantum secure messaging with no phone number and no
server you have to trust.**

> [!WARNING]
> **Pre-release. Not independently audited. There is no messaging yet.**
> This app can create a vault, mint and redeem invitations, and record contacts. It cannot
> send or receive messages. Do not rely on it for anything real. If you need a secure
> messenger today, use [Signal](https://signal.org).

---

## What this is

A desktop application for the [QSL protocol](https://github.com/QuantumShieldLabs/qsl-protocol),
a research-stage secure messaging protocol built so that conversations stay private even
against an adversary who records them now and waits for a quantum computer.

**Design choices that distinguish it:**

- **No phone number, no account.** Your identity is a keypair you generate locally. Nothing is
  tied to a SIM or a carrier.
- **Self-hosted relays.** You point the app at a relay you choose, and the protocol does not
  assume that relay is trustworthy.
- **It connects only when you ask it to.** Nothing reaches the network at launch, in the
  background, or on a timer. Every connection is the direct result of a button you pressed.
- **Local-only contact names.** The name you give a contact lives on your machine. No display
  name is ever accepted from the network.
- **Your vault, your keys.** Secrets live in an encrypted local vault with idle autolock,
  escalating unlock delays, and a destroy path that actually destroys.

## The stack

- **Tauri v2** shell with the `qsc` client core linked **in-process** as a rev-pinned Rust
  library — no sidecar process, no IPC to a separate binary.
- **Vanilla HTML, CSS, and JavaScript.** Zero npm, zero Node, zero JavaScript dependencies.
  The entire frontend is three files you can read.
- **v1 targets Linux.** macOS is the first platform lane after the skeleton is complete;
  Windows sits at a later horizon.

The dependency choice is deliberate. A messenger whose frontend pulls hundreds of transitive
packages has a supply chain nobody can audit. This one does not have that problem.

---

## What works today

**Vault and identity**
Onboarding and vault creation, identity display, the unlock screen with its escalating-delay
protection, idle autolock, and the Vault & Security settings including destroy and erase.

**Relay configuration**
Settings › Relay takes a relay address, an access token, and a CA certificate file. One
**Save** commits the pane. **Test connection** saves first, then reports what the relay
actually answered — not what the app hoped it would say.

**Invitations**
Mint a single-use invitation code with a short expiry, shown once, with an optional local-only
note naming who it is for. Review live invitations and their state, including when one has
been redeemed. Revoke one. Redeem someone else's: paste the code, name the contact, and the
app completes the handshake and records the peer.

**Contacts**
Contacts are listed, incoming contact requests can be accepted, ignored, or blocked, and
display names can be set locally.

**Safety rails on connecting**
An invitation you own is refused before redemption with a plain explanation rather than a
cryptic failure. Locked vaults, unavailable invitation storage, and malformed codes each stop
the attempt with their own message. Changing a field or closing the window invalidates an
outstanding preflight, only one connect attempt runs at a time, and a late result cannot
repaint a window that has closed.

## What does not work yet

**Messaging.** There is no sending and no receiving. You can establish a contact and the
handshake completes, but nothing can be sent to that contact.

That is the next major body of work, and it is larger than it looks: the messaging surface is
the first place the interface speaks to the protocol's message plane at all.

There is also no release build, no packaging, and no update channel.

No security, privacy, or availability claims are made for anything here beyond factual
feature description. The app's own status line shows only what is actually true.

---

## Build and run

Requires a Rust toolchain and the Tauri v2 system dependencies for your distribution.

```bash
git clone https://github.com/QuantumShieldLabs/qsl-desktop
cd qsl-desktop
cargo tauri dev
```

There is no npm install step, because there is no npm.

To exercise anything that touches the network you will need a relay. Run one locally from
[qsl-server](https://github.com/QuantumShieldLabs/qsl-server), then point Settings › Relay at
it.

---

## Status and honest limits

- **Not independently audited.** No third party has reviewed this application or the
  cryptography beneath it.
- **Pre-release.** Interfaces, on-disk state, and wire formats are still changing. Assume a
  vault created today may not open tomorrow.
- **Not externally reviewed, not production-ready, and not a release.**
- **Linux only** for v1.
- Open defects exist and are tracked in the open.

**Do not use this to protect anyone whose safety depends on it.** When that changes, it will
change because an external audit says so.

---

## Security reporting

Please do **not** file security-sensitive reports in public issues. Use GitHub private
vulnerability reporting on this repository, or follow [`SECURITY.md`](SECURITY.md). If private
reporting is unavailable, open a minimal public issue with **no exploit details**, stating
that you can share specifics privately.

## Related repositories

- [**qsl-protocol**](https://github.com/QuantumShieldLabs/qsl-protocol) — specifications,
  conformance vectors, and the reference implementation this client links
- [**qsl-server**](https://github.com/QuantumShieldLabs/qsl-server) — the relay
- [**qsl-attachments**](https://github.com/QuantumShieldLabs/qsl-attachments) — encrypted
  attachment plane

## License

`AGPL-3.0-only` — see [`LICENSE`](LICENSE). Any future commercial services or support
offerings are separate from this repository and do not replace the AGPL terms on the source
published here.

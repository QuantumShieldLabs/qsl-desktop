# Appendix F — Server pane & connection-results taxonomy (slice B, operator-approved 2026-07-24)

Companion to QSC_DESIGN_SPEC_v1_round2.md (§1 tokens govern values; §2 status
banner component) and Appendices D/E. This appendix BINDS the slice-B Server
pane (spine lane NA-0673, directive D609). Where F and an earlier appendix
disagree ON THE SERVER PANE, F governs — it is the newer decision. F changes
nothing about the pre-main screens, the Vault pane, or the token values.

The Server pane is the app's FIRST network surface: it points the app at a
relay and tests the connection. Scope is server CONNECTIVITY only — not
contacts, not messaging, not the Logs pane, not the rail toggle.

## F.1 — Layout & structure

The pane lives in the existing Settings shell (icon rail → settings-rail →
detail pane); the "Server" nav item and `#pane-server` already existed as a
placeholder. Structure, top to bottom, inside a form column capped at **470px**
so inputs do not stretch across a wide monitor (the pane widens; the form does
not — the mockup's cap, and the mockup is the LAYOUT authority):

1. Lede: "The relay carries your encrypted messages. It can't read them."
2. **Relay address** — a monospace text input (`#relay-url`) + an inline error
   slot (`#relay-url-error`).
3. **Access token** — a password input (`#relay-token`), a helper ("Required
   only if the operator set one. Stored in your vault, not in settings."), a
   **Set token / Clear** control row, and a status line (`#relay-token-status`).
4. **Certificate authority (optional)** — a `<details>` disclosure holding a
   monospace path input, an inline error slot, a **Set CA file / Clear** row,
   and a status line.
5. **Test connection** (primary) + **Save** (secondary), side by side.
6. **Results panel** (`#relay-results`, hidden until a test runs): the shipped
   status-banner component + a detail paragraph + a document rows container +
   the "Not saved yet" note.

**[F.1-COLOUR] Colour comes from the shipped tokens, never the mockup.** The
reference markup `02-settings-server-pane.html` is authoritative for LAYOUT AND
STRUCTURE only; its palette (`qsl-tokens.css`) differs from the shipped tokens
and is NOT binding. `design_round2.rs` pins the values. Reading a hex out of
the mockup into the build is a STOP (D609 R7).

**[F.1-BANNER] The results reuse the §2 status-banner component with only
`status-neutral` and `status-accent`.** RED (`status-danger`) is RESERVED by §2
for the vault-danger surfaces (armed erasure, autolock-0, ceremony phrases, the
"Delete vault?" link). A CONNECTION FAILURE IS NOT A VAULT DANGER, so it uses
`status-accent` (the app's "needs attention" treatment), never red — the
MESSAGE carries the severity. Connected uses `status-neutral` (calm, verified).
The mockup's red "bad" / amber "warn" coding is deliberately not reproduced;
copying it would be reading a mockup colour.

**[F.1-COMMIT] Save persists ONLY the relay URL; the token and CA commit
through their own controls.** The URL is NON-SECRET and lives in
`settings.json` (added to the allowlist test deliberately, D609 R6). The bearer
token and the CA-file path are SECRET and live in the qsc vault, written via the
`relay_token_set/_clear` and `relay_ca_file_set/_clear` trios — never
`vault::secret_set` directly, and never `settings.json`. Because the probe reads
the token and CA FROM THE VAULT (env→vault→file), they must be committed before
a test can exercise them; each therefore has its own Set/Clear, and "Save"
governs the URL alone. (The mockup drew the token as a bare input; the
directive's "Save persists ONLY the URL" governs, so the token gained explicit
controls.)

## F.2 — The results states (the full enumeration)

"8 states" means the eight results-panel CARDS: SEVEN probe outcomes plus the
one save-state. Two further UI states (idle, clear-on-edit) and three local-
error states are named here so "8" is never read as "8 probe outcomes."

SEVEN PROBE OUTCOMES (the qsc `RelayServerInfoOutcome`, classified in the crate
and rendered here — never re-classified):

| # | State | Trigger | Banner | Headline |
|---|-------|---------|--------|----------|
| 1 | Reachable / Bearer | 200 + `auth.mode=bearer` + token accepted | neutral | "Connected" — "Token required — accepted. Certificate trusted." |
| 2 | Reachable / Open | 200 + `auth.mode=open` | neutral | "Connected" — "Open relay — anyone who can reach this address can use it…" |
| 3 | Cert not trusted | TLS refused a readable cert | accent | "Certificate not trusted" (…"also what an interception attack looks like") |
| 4 | Unreachable | conn/DNS/timeout | accent | "Couldn't reach the server" |
| 5 | Token rejected | 401 + QSL challenge, token WAS sent | accent | "Token rejected" |
| 6 | Token required | 401 + QSL challenge, token was NOT sent | accent | "This relay requires an access token" |
| 7 | Not a QSL relay | answered, no parseable `auth.mode` | accent | "Not a QSL relay" |

For a Reachable outcome the panel also renders the REAL `ServerInfoDoc` fields
(relay name, certificate = Trusted, access mode, retention, max message size,
server version). The mockup's `0.9.2` / `7 days` / `1 MB` / `inspiron-lan-relay`
are placeholders; the pane shows what the probe returns.

THE SAVE-STATE:

| 8 | Not saved yet | a successful test whose URL is not the saved one | — | "Not saved yet." + Save takes the accent (primary) treatment |

TWO NON-CARD STATES:

| 9  | Idle / never-tested | pane opened, no test run | the results panel is HIDDEN |
| 10 | Clear-on-edit | any field edited after a test | the results panel is CLEARED — asserting "Connected" for a configuration that no longer exists is a false claim |

THREE LOCAL-ERROR STATES (the probe's `Err` channel — a request was never
formed; see F.6):

| 11 | Bad address | `normalize_relay_endpoint` rejected the URL | INLINE field validation under the address, NOT a results card |
| 12 | CA file unreadable | configured CA missing/unreadable/not-a-cert | its OWN accent results line — EXPLICITLY NOT "Certificate not trusted" |
| 13 | Test couldn't start | client build failure / other | a generic accent line: "Couldn't start the connection test" |

## F.3 — The two-message 401 rule

States 5 and 6 come from the SAME byte-identical relay `401` — the relay does
not, and must not, reveal which. The CLIENT distinguishes them by whether IT
sent a token (`token_was_sent`). Both messages are phrased as LOCAL
OBSERVATIONS about what this app did, NEVER as server verdicts: "the one this
app sent" / "this app sent no token." The app never claims the server said
anything it did not say.

## F.4 — Save/Test independence, no auto-save, clear-on-edit

- **Independent.** Test is never required before Save; Save is never gated on a
  passing Test. Either can be used alone.
- **No auto-save.** A successful Test never writes anything. Test is a probe;
  Save is a commitment. After a good Test with an unsaved URL, the panel shows
  "Not saved yet." and Save takes the accent treatment (state 8) — it does not
  save itself.
- **Clear-on-edit.** Editing any field (URL, token, CA path), or committing a
  new token/CA, after a Test CLEARS the results panel (state 10).

## F.5 — The no-bypass boundary

There is NO "connect anyway" and NO "trust this certificate" control anywhere on
the pane. This is the GUI face of NA-0663's hard boundary, and it agrees with
the crate BY CONSTRUCTION: `qsc`'s `relay_http_client()` builds its trust store
as webpki roots ∪ OS-native roots ∪ the operator CA file, fail-closed, with NO
bypass path of any kind. The only remedy the pane offers for an untrusted
certificate is to add the operator's CA file (an explicit trust anchor), never
to disable verification. A future lane cannot add a convenience bypass to this
pane without contradicting the crate — the crate would still refuse.

## F.6 — CA-file-unreadable is NOT CertNotTrusted

State 12 (a configured CA file that is missing, unreadable, or not a
certificate) is a LOCAL CONFIGURATION problem: the request was never formed,
because the client could not be built. It is rendered as its own results line
with a file-path remedy. It is EXPLICITLY NOT state 3 (Certificate not
trusted), which means TLS refused a READABLE-but-untrusted certificate presented
by the server. Conflating them would send a private-CA self-hoster debugging a
server certificate problem that does not exist — the wrong-error-mapping class,
on precisely the private-CA surface NA-0663 built for and NA-0672 first
exercised live (the CA pair). The two states carry different remedies and must
stay visibly distinct.

## F.7 — The locked-vault CA false-negative (latent lie, recorded)

`relay_ca_file_show()` (and `relay_token_show()`) resolve through
`vault::secret_get`, which fails CLOSED when the vault is locked. A locked vault
therefore reports `configured = false` — reading as "no CA file / no token set"
rather than "unknown." This is SAFE ONLY because the Settings surface is
unlock-gated by construction: the Server pane is reachable only from an unlocked
session, so the vault is always unlocked when the pane reads it. **It becomes a
lie the moment any future lane exposes a Settings pane (or this status) in a
locked state.** Any such lane must resolve the locked case to an explicit
"unknown / locked" rendering, not to `configured = false`. Recorded here so the
constraint travels with the design authority, not just the code.

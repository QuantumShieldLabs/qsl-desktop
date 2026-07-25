# Appendix F — Server pane & connection-results taxonomy (slice B, operator-approved 2026-07-24; REVISED 2026-07-25 by the NA-0674 redesign)

Companion to QSC_DESIGN_SPEC_v1_round2.md (§1 tokens govern values; §2 status
banner component) and Appendices D/E. This appendix BINDS the Server pane
(spine lanes NA-0673 / directive D609, and NA-0674 / directive D610). Where F
and an earlier appendix disagree ON THE SERVER PANE, F governs — it is the
newer decision. F changes nothing about the pre-main screens, the Vault pane,
or the token values.

> **REVISION NOTE — 2026-07-25, NA-0674 (D610), desktop D-0010.**
> The commit model recorded in the original **[F.1-COMMIT]** was REVERSED one
> lane after it shipped. Passages this revision supersedes are **MARKED IN
> PLACE and left legible**, never deleted or quietly reworded: a design
> authority whose history is edited away cannot be reasoned with, and the
> reasoning that made the old model right at the time is exactly what a future
> reader needs in order to understand why it stopped being right. Superseded
> passages carry a `⛔ SUPERSEDED` banner and a forward pointer; the replacing
> text follows immediately. **F.3, F.5, F.6, F.7 and [F.1-COLOUR] are
> UNCHANGED by that revision.**

The Server pane is the app's FIRST network surface: it points the app at a
relay and tests the connection. Scope is server CONNECTIVITY only — not
contacts, not messaging, not the Logs pane, not the rail toggle.

## F.1 — Layout & structure

The pane lives in the existing Settings shell (icon rail → settings-rail →
detail pane); the "Server" nav item and `#pane-server` already existed as a
placeholder. Structure, top to bottom, inside a form column capped at **470px**
so inputs do not stretch across a wide monitor (the pane widens; the form does
not — the mockup's cap, and the mockup is the LAYOUT authority).

**THREE SECTIONS** (D610 R-D1), separated by hairlines (`1px solid
var(--border)`) **BETWEEN sections only — two rules, never three**. The rule is
drawn by an adjacent-sibling selector (`.srv-sect + .srv-sect`), so the
hairline count follows the section count structurally and cannot drift.
Vertical padding above and below each hairline is **`var(--sp-6)` (32px)**: the
reference markup used 30px, which sits exactly between the shipped `--sp-x28`
and `--sp-6` with no nearer step, and `--sp-6` was ruled (D610 F2R) as a
canonical scale step rather than an escape-hatch value. The 2px difference from
the mockup is a sanctioned deviation, not a drift.

0. Lede: "The relay carries your encrypted messages. It can't read them."

**Section 1 — address and token**
1. **Relay address** — a monospace text input (`#relay-url`) + an inline error
   slot (`#relay-url-error`).
2. **Access token** — a password input (`#relay-token`) at full column width,
   and **one** helper line (`#relay-token-help`) that **SWAPS by state and
   never stacks** (F.2a below).

**Section 2 — certificate authority**
3. **Certificate authority (only needed for self-hosted certificates)** — a
   `<details>` disclosure, collapsed by default, holding the explanatory hint,
   a monospace path input at full column width, an inline error slot, and the
   CA status line (`#relay-ca-status`).
   The summary wording is D610 R-D3's. The reference markup still reads
   "(optional)" because it was captured before the relabel was ruled; **R-D3
   governs** — a specific ruling beats a generic layout authority.
   **The path input renders EMPTY even when a CA IS set**, and this is forced,
   not chosen: `relay_ca_file_show()` returns `{configured, path_hash}` and the
   PATH IS NEVER RETURNED (the CA path is public so it is hashed; the token is
   secret so it is a bare bool — the asymmetry is deliberate, see F.7). The
   reference markup draws the input populated; the app cannot know that value,
   and echoing it back would require a qsc API change. The status line carries
   the stored state instead.

**Section 3 — actions and results**
4. **Test connection** (primary) then **Save** (secondary), side by side
   (R-D4).
5. **Dirty helper** (`#relay-dirty`) directly under the actions row — see
   F.2b.
6. **Results panel** (`#relay-results`, hidden until a test runs): the shipped
   status-banner component + a detail paragraph + a document rows container.
   It no longer carries a border of its own — that border would now read as a
   third hairline — and it no longer holds a save-note (state 8 is gone, F.2).

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

**THE REASON (ratified 2026-07-24) — recorded so a future lane cannot re-open
it.** §2 reserves red for VAULT DANGER: irreversible-loss states — armed
erasure, autolock-0, the destroy ceremony. A relay you cannot reach is an
INCONVENIENCE: nothing is lost, nothing is at risk; you fix the address and
retry. If "certificate not trusted" rendered in the same red as "this will
erase your vault," the palette would stop meaning anything — the user learns
red is merely "something's off," and red on the destroy ceremony then carries no
more weight than a typo'd hostname. Severity belongs to the message, not the
colour. Declining to copy the mockup's red is R7 working as intended: the mockup
is the LAYOUT authority; its palette is not.

### ⛔ SUPERSEDED 2026-07-25 by [F.1-COMMIT-v2] below (NA-0674, D610, D-0010)

> The two paragraphs that follow are the ORIGINAL [F.1-COMMIT], recorded
> 2026-07-24 and reversed 2026-07-25. They are kept verbatim and legible
> BECAUSE they were reasoned, not guessed: the split-commit shape really was
> the only one consistent with both standing rulings *given the affordances
> that existed at the time*, and reading the reversal without them makes it
> look like drift. What the original could not see is stated in
> [F.1-COMMIT-v2]. **The storage rule it states — secrets to the vault via the
> trios, URL to `settings.json` — is UNCHANGED and still binds.** Only the
> user-facing commit surface changed.

**~~[F.1-COMMIT] Save persists ONLY the relay URL; the token and CA commit
through their own controls.~~** The URL is NON-SECRET and lives in
`settings.json` (added to the allowlist test deliberately, D609 R6). The bearer
token and the CA-file path are SECRET and live in the qsc vault, written via the
`relay_token_set/_clear` and `relay_ca_file_set/_clear` trios — never
`vault::secret_set` directly, and never `settings.json`. Because the probe reads
the token and CA FROM THE VAULT (env→vault→file), they must be committed before
a test can exercise them; each therefore has its own Set/Clear, and "Save"
governs the URL alone. (The mockup drew the token as a bare input; the
directive's "Save persists ONLY the URL" governs, so the token gained explicit
controls.)

~~This is a RULING-REFINEMENT, not a deviation (ratified 2026-07-24). "Save
persists only the URL" was correct — secrets belong in the vault, never in
`settings.json` — but on its own it left the token with nowhere to be
committed. Own Set/Clear controls via the vault trios is the ONLY shape
consistent with BOTH standing rulings ("URL to settings, secrets to the vault"),
and it matches the CA disclosure's existing shape.~~

### [F.1-COMMIT-v2] ONE Save commits everything; Test saves first

**Save commits the whole pane: the URL to `settings.json`, the token and the
CA path to the vault through the same trios as before. Test, on a dirty pane,
COMMITS FIRST and then probes the just-saved state. The four per-field
Set/Clear buttons are REMOVED; each field offers a "remove it" prose link
instead.** (D610 R-A1, R-A2, R-A3.)

**What the original could not see.** The split model was locally coherent and
globally a trap. The probe reads the token FROM THE VAULT, so a user who typed
a new token and pressed Test — the obvious gesture, and the one the layout
invited — got a result computed against the OLD token, because the typed one
had never been committed. The pane then reported that result **truthfully**,
which is what makes the trap dangerous: nothing looked wrong. "Token rejected"
for a token the user believed they had just replaced is indistinguishable, on
screen, from a genuinely bad token.

The original could have been patched with a warning — "press Set token before
testing." **Removing the trap beats warning about it.** A warning puts the
burden on the user to remember an ordering the interface itself created; the
unified commit deletes the ordering. That is the whole reversal.

**Two-button rationale, retained.** Test and Save remain separate because they
answer different questions: Save is "remember this," Test is "and does it
work?" Save stays independently clickable for the configure-offline case
(R-A3), and a clean pane's Test commits nothing.

**The commit is ORDERED, and it is NOT atomic.** Validate-then-write cannot be
separated with the affordances that exist: `relay_config_set` runs
`normalize_relay_endpoint` **and writes** in one call, exactly as
`relay_ca_file_set` validates **by writing**. There is no validate-only
command for either field, and this lane adds none. The order is therefore:

1. **the relay address** — validated-and-written first, so a malformed address
   blocks the entire commit with **nothing persisted** and **no probe**
   (R-B2), rendering as inline field validation (state 11), never a card;
2. **the token** — vault, via the trio (blank keeps, typed replaces, pending
   removal deletes — R-B3);
3. **the CA path** — vault, via the trio.

A failure at any step **abandons the remainder** and renders **state 14**,
naming which part failed. Steps that already succeeded STAY committed: the
commit is a sequence, not a transaction, and D610 R-B1 concedes this
explicitly. After any failed commit the pane **re-reads live state** and
re-renders the helper lines, so it can never describe state a partial commit
has already changed.

> **⚠ RECORDED DEVIATION FROM D610 C2, raised for ruling (D-0010).** C2 ordered
> "validate the URL → token → CA → `settings.json` LAST", on the premise that
> the URL — unlike the CA path — could be validated without writing. **That
> premise is false**, and was verified false during implementation: the crate
> registers nine relay commands and none is validate-only. That leaves R-B1
> ("vault writes first, settings.json last") and R-B2 ("nothing persists" on a
> malformed address, on Save AND on Test) in direct conflict. **R-B2 is
> implemented**, because it is absolute, is stated for both buttons, and is
> the one a user can observe; R-B1's ordering carries no stated rationale and
> R-B1 already concedes the commit is non-atomic, so inverting the order costs
> a preference rather than a guarantee. The cost is real and stated: if a
> vault write fails, the address has already been saved.

**Removal is pending, not immediate.** "remove it" marks the field for
deletion on the NEXT commit and marks the pane dirty; typing in the field
cancels it (R-B3, R-E3). Nothing is deleted the moment the link is clicked —
the same Save/Test gesture that commits everything else commits the removal.

**A failed probe never rolls back a commit** (R-B4). The commit outcome and
the probe outcome are independent verdicts, and the pane reports both.

## F.2 — The results states (the full enumeration)

**REVISED 2026-07-25 (D610 R-F1..R-F4).** The results block now carries SEVEN
cards, not eight: state 8 ("Not saved yet") is REMOVED, its job folded into the
dirty helper (F.2b). State **14** is ADDED — a commit that failed. States 1–7,
9, 11, 12 and 13 are UNCHANGED in trigger and in wording; state 10's trigger is
BROADENED (F1R). The enumeration below is the whole set, so "seven cards" is
never read as "seven states."

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

⛔ **THE SAVE-STATE — REMOVED 2026-07-25 (R-F1).**

> | ~~8~~ | ~~Not saved yet~~ | ~~a successful test whose URL is not the saved one~~ | ~~—~~ | ~~"Not saved yet." + Save takes the accent (primary) treatment~~ |
>
> State 8 existed because a successful Test did not save, so the panel had to
> say so. Under [F.1-COMMIT-v2] a Test **does** save, so a successful Test can
> no longer leave unsaved changes and the state is unreachable. Its real job —
> "there is uncommitted work on this pane" — is now the dirty helper's (F.2b),
> where it belongs: that condition is a property of the FORM, not a property
> of a probe result, and it should be visible before any test is ever run.

TWO NON-CARD STATES:

| 9  | Idle / never-tested | pane opened, no test run | the results panel is HIDDEN |
| 10 | Cleared by a change | **any change to what the app will use** — a field edit, a "remove it" click, or cancelling a pending removal by typing | the results panel is CLEARED — asserting "Connected" for a configuration that no longer exists is a false claim |

> **State 10's trigger was BROADENED 2026-07-25 (F1R).** It read "any field
> edited after a test." That left a gap: "remove it" is a link, not a field
> edit, so a pending removal would have left a stale results card standing.
> The rule is now one sentence — *any change to what the app will use clears
> the results* — which is both the honest rule and the implementable one.
> A consequence worth stating: this makes the reference mockups' composite
> view (dirty helper AND a Connected card, drawn together) **unreachable by
> construction**. The mockups are part catalogues, not screenshots of a
> reachable state, and should be compared per-section.

FOUR LOCAL-ERROR STATES (no probe was performed; see F.6):

| 11 | Bad address | `normalize_relay_endpoint` rejected the URL | INLINE field validation under the address, NOT a results card |
| 12 | CA file unreadable | configured CA missing/unreadable/not-a-cert | its OWN accent results line — EXPLICITLY NOT "Certificate not trusted" |
| 13 | Test couldn't start | client build failure / other | a generic accent line: "Couldn't start the connection test" |
| 14 | **Couldn't save settings** (NEW, R-F2) | a commit step failed | an accent line, sibling of 13. Names WHICH part failed — the vault or the settings file — and states plainly that **the probe did not run**. Steps that already succeeded stay committed; the pane re-reads live state so the helper lines stay true. |

## F.2a — The token and CA state lines

The token field carries **exactly one** helper line, which SWAPS by state and
**never stacks** (R-E1, R-E2, R-E3):

| Token state | Input | Helper line |
|---|---|---|
| set | EMPTY, placeholder `••••••••` — **always eight, never the real length** | "A token is set — leave blank to keep it, or *remove it*." |
| not set | empty, no placeholder dots | "Required only if the operator set one. Stored in your vault, not in settings." |
| removal pending | empty, no dots | "Token will be removed when you save or test." |

The eight dots are FIXED because `relay_token_show()` returns a bare bool —
the app does not know the token's length and **must not appear to**. A
variable-length mask would leak exactly the thing the bare bool protects.

The CA status line follows the same pattern (R-E4): "CA certificate file set
(`<hash>`) — *remove it*." when configured; "No CA file set — the app uses your
computer's trusted certificates." when not; and "Certificate authority file
will be removed when you save or test." when removal is pending. The **hash**
appears, never the path (F.7, and F.1 §3).

*remove it* is a link in both lines, per field. It replaced the four Set/Clear
buttons — two of which were adjacent controls both labelled "Clear", which the
NA-0673 acceptance flight mis-clicked twice, each time producing a
plausible-looking wrong result card (ENG-0073, superseded by this redesign).
A removal affordance that lives inside the sentence describing its own field
cannot be confused with its neighbour.

## F.2b — The dirty helper

"**Settings changed — not saved.**" — rendered under the actions row while any
field differs from stored state or a removal is pending; cleared by a
successful commit from either button.

**Accent, NEVER red.** An unsaved relay address is inconvenience-class loss:
nothing is destroyed and nothing is at risk. Red is reserved for vault danger
([F.1-BANNER]), and spending it here would devalue it there.

**No confirm dialog on navigate-away** (R-B6). A dirty pane left behind is
discarded silently. Severity discipline: inconvenience-class loss gets a
helper, not a modal — and a modal that fires on every incidental navigation
trains the user to dismiss modals.

## F.2c — In-flight

While a commit or a probe is running, **both buttons are disabled** and the
results area shows a neutral, accent-free line until resolution. No re-entry
(R-C1).

The line reads "Testing…" on the Test path and "Saving…" on the Save path.
D610 R-C1 wrote "Testing…" for both; a Save performs no probe, so saying
"Testing…" during one would state something untrue about what the app is
doing — the precise class of claim this project sweeps for elsewhere. The
mechanism R-C1 specifies is implemented exactly; only the Save path's label
differs, and it differs in order to be accurate.

## F.3 — The two-message 401 rule

States 5 and 6 come from the SAME byte-identical relay `401` — the relay does
not, and must not, reveal which. The CLIENT distinguishes them by whether IT
sent a token (`token_was_sent`). Both messages are phrased as LOCAL
OBSERVATIONS about what this app did, NEVER as server verdicts: "the one this
app sent" / "this app sent no token." The app never claims the server said
anything it did not say.

## F.4 — Save/Test relationship

### ⛔ SUPERSEDED 2026-07-25 by F.4-v2 below (NA-0674, D610, D-0010)

> The "no auto-save" bullet is the one this lane reverses; it is kept here
> because it names the principle the reversal had to weigh — *Test is a probe,
> Save is a commitment* — and that principle survives in a modified form. The
> independence bullet survives intact; the clear-on-edit bullet survives with
> a broadened trigger (F1R).

- ~~**Independent.** Test is never required before Save; Save is never gated on a
  passing Test. Either can be used alone.~~
- ~~**No auto-save.** A successful Test never writes anything. Test is a probe;
  Save is a commitment. After a good Test with an unsaved URL, the panel shows
  "Not saved yet." and Save takes the accent treatment (state 8) — it does not
  save itself.~~
- ~~**Clear-on-edit.** Editing any field (URL, token, CA path), or committing a
  new token/CA, after a Test CLEARS the results panel (state 10).~~

### F.4-v2 — Test saves first

- **Save is never gated on a passing Test.** Unchanged, and it matters more
  than before: Save alone is the whole configure-offline path (R-A3).
- **Test COMMITS, then probes** (R-A2). On a dirty pane Test writes everything
  and probes the just-saved state; on a clean pane it writes nothing. The
  probe therefore always describes **what the app will actually use** — which
  is the property the old model could not offer.
- **The commitment principle is preserved, not abandoned.** "Test is a probe,
  Save is a commitment" was protecting the user from writing something they
  had not chosen to write. But the user who types a token and presses Test
  *has* chosen it — the typing was the choice; the button press was the
  confirmation. What the old model actually delivered was not caution but a
  silent divergence between what the pane showed and what the vault held.
- **A commit is announced.** On the test-committed path the detail line
  appends "**Settings saved.**" (R-E6, implemented rather than optional —
  D610 C6). A dirty helper merely *disappearing* is absence-of-signal, not
  confirmation, and a Test that writes must say that it wrote.
- **Cleared by a change.** Any change to what the app will use — a field edit,
  a "remove it" click, or cancelling a pending removal by typing — clears the
  results panel (state 10, trigger broadened by F1R).

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

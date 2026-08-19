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

- **ID:** D-0002
  - **Status:** Accepted
  - **Date:** 2026-07-19
  - **Decision:** Land GUI slice A — the serverless skeleton — per spine
    directive D595 (QSL-DIR-2026-07-19-595, as amended at approval; spine
    decision D-1282; lane NA-0659): the Tauri v2 scaffold replaces the
    placeholder crate (src-tauri joins the bootstrap's empty [workspace];
    the placeholder binary is retired); qsc is consumed as a rev-pinned git
    dependency at spine main `81143dcd3b4a7beead7d0f4e742717a4310e2409`;
    the CI gate gains the Tauri system-dependency apt step with the
    required context name kept exactly `rust`; the frontend is static
    vanilla HTML/CSS/JS with zero npm/node (operator rationale: no JS
    supply chain in a security product's build); the core-call layer
    encodes the four startup rules (env+policy+InApp routing once before
    threads; drain-after-every-call into a bounded buffer; lock state only
    via the qsc NA-0658 one-call surface; strictly-serial single-flight
    spawn_blocking), each test-proven; the S0/S1/S2 launch state machine
    uses the app-level `vault.qsv` existence probe (spine F2 ruling; the
    filename coupling is recorded on the spine ledger); wizard steps 1–2,
    the unlock screen rendering the typed retry-after/attempts values, the
    two-step app-level forgotten-passphrase erase (file removal only),
    idle autolock (on, 15 min default), the empty three-pane main window
    ("no server configured"), and Settings Vault/Security bound to the
    real qsc protection surface (identity export absent by design).
    Slice A contains ZERO networking code; the server-connectivity surface
    (wizard step 3, error taxonomy, Settings Server pane, four-state
    status line) is slice B — OWED, the committed successor concern.
  - **Rationale:** House lane sizing (one concern per lane) split step 5 on
    the network/zero-network line; the launch state machine derives
    everything from what exists on disk, so slice-A installations sit on
    the deferred path in S2 until slice B lands — no migration, no wizard
    re-entry.
  - **References:** spine D595 (the directive, as amended at approval,
    sha256 d94f2b7b…); spine D-1282 (the lane closeout); spine D-1281 /
    NA-0658 (the vault-protection surface this slice binds to); spine
    D-1272 / NA-0649 (the GUI-surface functions); the 2026-07-16
    GUI-readiness investigation (the four startup rules, §4.1 system
    dependencies).

- **ID:** D-0003
  - **Status:** Accepted
  - **Date:** 2026-07-19
  - **Decision:** Land the GUI slice-A design pass per spine directive
    D596 (QSL-DIR-2026-07-19-596, as amended at approval, sha256
    508ac660…; spine decision D-1283; lane NA-0660) — presentation and
    the acknowledgment rule ONLY; every state-machine, wizard, autolock,
    wipe, destroy, and erase SEMANTIC is byte-for-byte the NA-0659
    behavior (the two slice-A test files are byte-identical to the D-0002
    tree and green). THE DESIGN SYSTEM (built once, applied app-wide):
    one `:root` token layer in `ui/style.css` — the type scale
    (hero/title/glyph/section/body/hint), the spacing scale (--sp-1..6),
    ONE accent carrying primary actions, focus states, and the active
    rail item, the destructive red family kept separate, greens/ambers as
    status colors only, color literals confined to the token block —
    plus the button tiers (primary filled-accent / secondary outline /
    destructive red, every button exactly one tier or a named nav role)
    and the NO-SILENT-STATE-CHANGES rule as ONE shared `acknowledge()`
    helper (momentary "✓ Saved"-style flash on the control + the
    section's persistent status line updated; applied to autolock save,
    wipe arm/disarm, and self-alias save; binding on all future
    settings). THE SCREEN WORK: display name "QuantumShield Chat" bound
    to the window title + About line ONLY (identifier/productName/binary
    unchanged — the identifier anchors the app data dir); the passphrase
    step's honest checklist (12+ characters / match / not a
    commonly-used password from a built-in ≥100-entry sorted array)
    gating Create UI-side with the strength meter and no-recovery box
    retained and zero composition theater; the "This is you" redesign
    (verification-code hero, optional local-only self-alias stored as a
    skip-when-empty `settings.json` key — non-secret by ruling, the
    fresh-profile key set unchanged — the approved identity-safety copy
    with the plain-English post-quantum line, the fingerprint +
    mechanism naming behind "Show technical details", the Settings
    reassurance line); the DEDICATED Identity pane FIRST in the Settings
    rail (operator F1) fed exclusively by the existing
    identity_show/settings_get surface — zero new core calls, command
    count unchanged at 17 — with the alias editable there (operator F2)
    and a rail identity dot (alias initial; "Y" for the empty default)
    above the gear; rail hover tooltips (one pattern) and unmistakable
    accent active states on both rails; the Vault & Security restructure
    (controls first, prose trimmed; the failed-attempts line SILENT at
    zero and rendered as an amber "N failed unlock attempts since your
    last unlock" alert from the value CAPTURED AT UNLOCK-SCREEN ENTRY —
    never a post-unlock read, the core resets its counter on success —
    with an app-local dismiss); the shared destroy/erase confirmation
    pattern (heading "Destroy vault", the one-sentence prose, the
    one-line Type-to-confirm instruction above the input, [Destroy
    permanently] [Cancel]; the erase screen inherits the identical form
    with its own phrase and no passphrase field); and the main-window
    empty state's honest warmth (inline SVG glyph, one line, one CTA;
    "no server configured" untouched). Additive tests pin the
    disciplines (tests/design_system.rs: password-list soundness, title
    + identifier binding, type/spacing/color token discipline, button
    tiers, Appendix A verbatim copy); settings tests extend for the
    alias key only. ZERO dependency/lockfile/workflow motion; the qsc
    pin stays `81143dcd…`; zero networking code (the scan stays green);
    slice B remains OWED and untouched.
  - **Rationale:** The operator's first-flight design review: the slice-A
    surfaces become designed rather than assembled, and slice B's
    surfaces will be born INTO the system instead of retrofitted; the
    one functional gap the review found (identity visibility after
    onboarding) closes with zero new core surface.
  - **References:** spine D596 (the directive, as amended at approval);
    spine D-1283 (the lane closeout); D-0002 (the slice-A landing this
    pass restyles); spine D-1281 / NA-0658 (the protection surface whose
    reset-on-success semantics dictate the capture rule).

- **ID:** D-0004
  - **Status:** Accepted
  - **Date:** 2026-07-19
  - **Decision:** Land the GUI slice-A design pass ROUND 2 per spine
    directive D597 (QSL-DIR-2026-07-19-597, as amended at approval,
    sha256 `0bdde81a…`; spine decision D-1284; lane NA-0661) — the
    operator's post-merge review of the D-0003 build: presentation/copy,
    the item-13 webview state reset, the full-bleed shell, and the
    native-menu wiring ONLY; what destroy, erase, unlock, wipe-after-N,
    and autolock DO — and the S0/S1/S2 machine and wizard order — are
    byte-for-byte the D-0003 behavior (the two slice-A test files are
    byte-identical to the D-0003 tree and green). THE DESIGN AUTHORITY
    LANDS IN-REPO: `docs/DESIGN_SPEC.md` (the operator-approved round-2
    spec, byte-exact from the directive's Appendix C, sha256
    `34ced51b…`) + `docs/DESIGN_SPEC_AppendixD.md` (the operator's
    reference markup, byte-exact from Appendix D, sha256 `a7d45a0a…`)
    — BINDING on this and all future GUI lanes until revised; the
    `:root` token layer migrates to the spec §1 values (page #1D1D1F,
    card #252528, field #1A1A1C, hairline #3A3A3E; text #E8E8E8 /
    #A8A8A8 / #7A7A7A; the accent/danger/success/neutral role trios;
    17px/600 titles, 13px body; radius 12 cards / 8 controls) with the
    D596 discipline greps kept green. THE FIFTEEN ITEMS: (1) Confirm
    passphrase directly below Passphrase; (2) the strength meter
    REMOVED; (3) the checklist = exactly two checks — the
    common-password check, its 149-entry list, and its
    design_system.rs soundness test REMOVED (the one sanctioned test
    amendment); (4) the step-2 heading "Your identity"; (5) the
    verification code on ONE line, never wrapping (17px mono
    shrink-to-fit, shared by wizard + Identity pane); (6) the §5
    ceremony pattern on destroy AND erase — the instruction as its own
    one line, the erase screen's extra prose deleted, no passphrase
    field on erase (unchanged semantics); (7) the autolock helper
    verbatim with no number restated; (8) Arm = destructive tier /
    Disarm = secondary; (9) the duplicated guest-warning paragraph
    deleted; (10) the true disabled tier (#2A2A2E + muted text, never
    dimmed accent); (11) the rail identity dot REMOVED (rail = Chats,
    Contacts, gear; the Identity pane stays first in Settings — the dot
    half of the D596 F1 ruling is superseded by the operator's round-2
    item 11, recorded not relitigated); (12) both Vault and Security
    status lines become the spec-§2 status-banner component (ARMED =
    danger + warning icon; OFF = neutral + shield icon; AUTOLOCK =
    accent + lock icon; red RESERVED for the armed-erasure state; the
    attempts alert stays amber); (13) the STATE-HYGIENE FIX (the
    operator-verified defect: the destroy success path never cleared or
    collapsed the ceremony and no reload existed, so the prior vault's
    typed passphrase + phrase — and in-memory alias/alert state —
    survived into the next session): destroy/erase completion now
    performs a FULL webview reload (F2 default; provable by
    construction — all durable state is backend-side), PLUS the §5
    ceremony rules independent of the reload (every screen transition
    clears all six sensitive fields and collapses the ceremony; pane
    navigation resets the destroy flow; the wizard never pre-fills a
    prior alias). BINDING RULE RECORDED: no secret or prior-vault value
    may cross a destroy/erase boundary. HONEST RESIDUE: `destroy_vault`
    leaves `settings.json` (autolock minutes + alias, both non-secret)
    on disk by landed D-0002 semantics — changing what destroy removes
    was out of this lane's scope; surfaced to the operator as a
    semantics question; (14) the FULL-BLEED SHELL per Appendix D.1–D.3:
    no outer padding or inset frame, panes meet the window edges, 1px
    hairlines only, grid 52px | 210px | 1fr with the status bar as the
    full-width last row; SETTINGS IS A VIEW, NOT A MODAL — the same
    shell (52px | 160px | 1fr) with the icon rail live (gear active,
    Chats returns to main); the wizard card (max-width 440) stays the
    one centered exception; (15) the NATIVE MENU via the pinned
    tauri 2 core menu API ONLY (zero new crates/features): File
    (Settings, Lock now, Quit), Edit (Cut, Copy, Paste, Select all —
    native predefined), View (Reload — the item-13 reset mechanism;
    Full screen), Help (About — native, factual name/version + the
    honesty line); WORKING ENTRIES ONLY, and per R1 the two
    state-dependent File entries are live-DISABLED unless an unlocked
    surface shows (the frontend reports surface changes through the new
    app-layer `ui_surface_changed` command; zero qsc symbols, zero
    marker strings). Appendix-D tensions dispositioned as pre-ruled:
    About stays FUNCTIONAL (not muted-unbuilt); the D.2 search
    affordance lands as-shown, non-interactive, claiming nothing; the
    D.6 wizard-card background resolves to the §1 card value #252528
    per the appendix's own precedence sentence (layout from D, values
    from §1). Round-2 pins live in the additive
    tests/design_round2.rs. ZERO dependency/lockfile/workflow/
    tauri.conf.json motion; the qsc pin stays `81143dcd…`; zero
    networking code (the scan stays green); slice B remains OWED and
    untouched.
  - **Rationale:** The operator flew the D-0003 build and returned
    fifteen findings; the spec they approved becomes the repo's living
    design authority so round-3 and slice B are corrected against a
    written standard, not memory. The one state-hygiene defect the
    review found (typed secrets surviving destroy/erase in the living
    webview) closes with a mechanism that is provable by construction.
  - **References:** spine D597 (the directive, as amended at approval);
    spine D-1284 (the lane closeout); D-0003 (the round-1 pass this
    corrects); D-0002 (the slice-A semantics re-proven byte-frozen);
    spine D-1281 / NA-0658 (the protection surface; the capture rule
    unchanged); docs/DESIGN_SPEC.md + docs/DESIGN_SPEC_AppendixD.md
    (the landed design authority).

- **ID:** D-0005
  - **Status:** Accepted
  - **Date:** 2026-07-20
  - **Decision:** Land the GUI slice-A design pass ROUND 3 per spine
    directive D598 (QSL-DIR-2026-07-19-598, approved+amended, sha256
    `bb9dc338…`, 1073 lines; spine decision D-1285; lane NA-0662) — the
    operator's flight review of the merged NA-0661 build: presentation,
    window-sizing behavior, the autolock 60/0 semantics, and the
    30-second erase countdown gate ONLY. THE DESIGN AUTHORITY BECOMES
    THREE FILES: `docs/DESIGN_SPEC_AppendixE.md` lands BYTE-EXACT
    (cmp-proven against the directive extract; sha256 `5175f3bc…`, 128
    lines) and GOVERNS where it disagrees with the landed files; item 12
    amends the landed files minimally against the enumerated
    contradiction set, each amendment citing its [E.x]:
    `docs/DESIGN_SPEC.md` `34ced51b…` (113 l) → `074244be…` (143 l);
    `docs/DESIGN_SPEC_AppendixD.md` `a7d45a0a…` (222 l) → `5f5d3a2e…`
    (244 l); no contradiction needle survives (grep-pinned). THE TWO
    SANCTIONED BEHAVIOR DELTAS, exactly: (a) item 2 — autolock default
    60 with 0 VALID meaning never-auto-lock (settings.rs is exactly the
    item-2 set: AUTOLOCK_DEFAULT_MINUTES 15→60, the save() 0-reject and
    its `autolock_minimum_one_minute` error REMOVED, the in-file tests
    amended to pin default-60 + zero-valid, the doc comment updated;
    per the F2 DEFAULT no backend upper bound exists — the 0-1440 range
    is UI-side VISIBLE validation), superseding the 15-minute default
    (operator decision; the DOC-PROG-004 roadmap-revision note is filed
    in the spine ledger at this lane's closeout); the idle timer gains
    the BINDING never-fire guard at 0 (`if (autolockMinutes === 0)
    return;` before the elapsed comparison — at 0 the minutes*60000
    path would lock immediately; source-pinned) and the banner state
    machine renders accent/lock above 0 and the VERBATIM danger banner
    at 0; (b) item 11b — the 30-second countdown GATE on erase: on a
    validated confirm the form is REPLACED by the E.5 countdown panel,
    30→0, Cancel (the only action), closing the window, or ANY state
    transition aborts with nothing erased, and `erase_all` — its phrase
    check and scope byte-untouched (commands.rs untouched in the diff)
    — is invoked ONLY at countdown zero; the gate changes WHEN the
    erase commits, never what it erases. ITEMS 1-11 as landed: (1) no
    native number inputs (type=text + inputmode=numeric; the E.2
    appearance/spin-button CSS; ~64px centered fields; invalid entries
    BLOCKED with inline message + danger border via the shared
    validateNum; erase Limit 1-100, autolock 0-1440, the landed 720
    attr gone); (3) the danger phrases render QUOTED in danger mono —
    the quotes are CSS content on the `.instruction code` element (the
    round-2 byte-frozen markup needles bind the markup form; the
    `.phrase` selector of E.4 lands as `.instruction code` per the
    spec's adaptation rule; the typed phrase values stay unquoted); (4)
    both destroy fields full width, `.field-label` above (the
    label-wrap gone); (5) ONE `.ceremony-card` treatment on BOTH
    destructive surfaces (E.4 tokens: page bg, danger border, radius
    12, padding 20/22; `.ceremony-head` 17/600 danger; the settings
    destroy head stays an h3 — `.danger-h ceremony-head` — because the
    round-2 `>Destroy vault</h3>` needle is byte-frozen; verbatim
    landed copy survives; collapsed/expanded mechanics and the D597
    state-hygiene rules unchanged); (6) the arm checkbox 16px with the
    one-line clickable label; (7) the helper DIRECTLY under the
    autolock banner, the error surface moved to the validation slot
    above it, the reserved `.error`/`.feedback` min-heights removed so
    empty surfaces collapse (no voids); (8) `#settings-code` max-width
    420px centered; (9) rail icons 21px (svg + --fs-glyph together) and
    the status bar 12px #A8A8A8; (10) TWO WINDOW MODES resized on MODE
    transition riding ui_surface_changed — wizard 560x660, unlock/erase
    460x420, main+Settings 1024x700 (min 800x600 restored), each
    centered, set_min_size→set_size→center from the pinned tauri 2 core
    window API; scr-wiped (unlisted in E.1, reachable only FROM unlock)
    inherits the CompactGate class — recorded as the total-mapping
    completion; MENU VISIBILITY BY ATTACHMENT: compact modes
    remove_menu(), the full mode re-attaches the app-wide menu +
    show_menu() — chosen because tao's Linux set_visible(true) is gtk
    show_all(), which RESURRECTS a merely-hidden menubar on the F1
    deferred first show (found in the rig: hide_menu() returned Ok and
    is_menu_visible read false, yet the bar re-rendered; the
    attachment mechanism is deterministic and stays inside the pinned
    tauri 2 core menu API — zero dependency motion, manifests
    byte-untouched); the compact card FILLS its window (page padding
    20px, stretch); keyboard shortcuts and the webview's native
    right-click context menu remain available on compact screens
    (operator eyes; no input driver on the rig); (11a) `#link-forgot`
    renamed "Delete vault?" as `.link-danger` (12px #C87A7A tokenized
    as --danger-link, underlined; the landed destination unchanged;
    "Forgot your passphrase?" removed everywhere; the unlock error
    renders inline only when present). F1 RESOLUTION AS EXECUTED (the
    operator-ruled ALTERNATIVE): tauri.conf.json windows[0] ONLY — the
    exact key-level diff: `visible: (absent)→false`, `width: 1024→560`,
    `height: 700→660` (the compact initial size, wizard class);
    minWidth/minHeight and every other key in the file BYTE-IDENTICAL
    (they clamp only the never-shown pre-paint window; the backend
    applies per-mode minimums before the first show); the window is
    SHOWN by the backend on the first sized surface report — no
    1024x700→compact snap ever renders — with a 5-second fail-open
    fallback show in setup (an invisible app on a frontend boot fault
    is the worse failure; recorded honestly). R2 EXTENSION RECORDED
    (operator decision, not re-argued): the danger palette extends to
    the autolock-0 banner, the quoted ceremony phrases, and the
    "Delete vault?" link; red otherwise stays reserved for
    armed-erasure. PROOF: suite 56 passed / 0 failed / 1 ignored (lib
    5 + design_round2 17 + design_round3 16 NEW + design_system 6 +
    slice_a_flows 7+1 + slice_a_rules 5); the four existing test files
    BYTE-IDENTICAL to base and green; fmt/clippy -D warnings/metadata
    --locked clean; audit exit 0 on the unchanged lock; qsc-symbol set
    head == base (23/23); zero new markers; zero-networking scan
    green; publication scan class pass, zero overclaims; headless
    xwininfo geometry proof 560x660 / 460x420 per mode with NO menu
    bar on the compact captures.
  - **References:** spine D598 (the directive, as approved+amended);
    spine D-1285 (the lane closeout); D-0004 (the round-2 pass this
    corrects); D-0002 (the slice-A semantics re-proven byte-frozen);
    docs/DESIGN_SPEC.md + docs/DESIGN_SPEC_AppendixD.md +
    docs/DESIGN_SPEC_AppendixE.md (the three-file living design
    authority).

- **ID:** D-0006
  - **Status:** Accepted
  - **Date:** 2026-07-22
  - **Decision:** Land GUI ROUND 4a — pre-main screen chrome — per spine
    directive D601 (QSL-DIR-2026-07-22-601, approved+amended, sha256
    `2d80a72a…`, 966 lines; spine decision D-1291; lane NA-0665).
    Presentation and window-sizing behavior ONLY: no security property,
    no crypto, no vault internal, no protocol surface, no core call.
    `src-tauri/src/commands.rs` and `src-tauri/src/gateway.rs` are
    BYTE-ABSENT from the diff, as are `settings.rs`, `design_round2.rs`,
    `design_system.rs`, `Cargo.toml` and `Cargo.lock` (zero dependency
    motion). Visual authority: the operator's reference markup
    `01-pre-main-screens.html` (sha256 `316f8613…`, 148 l) for LAYOUT AND
    STRUCTURE ONLY — **never for color**. `qsl-tokens.css` (sha256
    `b9edb8f9…`, 353 l) is CONTEXT ONLY and is on a different palette than
    the build ships; the `#22262c` / `#16181c` values in the lane intent
    were handed in error and appear nowhere in the repository. NO TOKEN
    VALUE CHANGED: `--bg: #1D1D1F` and `--bg-raised: #252528` stand, and
    `design_round2.rs` is untouched (F3, STOP-class).
    **THE F2 OVERRIDE, RECORDED EXPLICITLY BECAUSE THE RULING CHANGED
    MID-LANE:** F2 as ruled at readback kept the E.4 red ceremony chrome
    on the destructive pre-main screens and stripped the neutral outer
    `.card` only. The operator REVISED that at census review: strip ALL
    card chrome from all five pre-main screens INCLUDING the danger
    border, with RED TEXT as the sole danger signal. This decision
    records the revision, not the readback ruling, and the build
    implements the revision. WHAT LANDED: (A) all five pre-main screens
    (wizard 1, wizard 2, unlock, erase, wiped) lose background, border,
    radius and card padding; the screen carries a uniform 28px content
    padding (`--sp-x28`, a new SPACING token — no color token added or
    changed) and content sits directly on `var(--bg)`. The flex column
    the `.card` provided is RE-HOMED onto the same element rather than
    lost. The strip is ID-SCOPED, so the bare `.ceremony-card` rule
    survives intact and the SETTINGS destroy ceremony (`#pane-vault`)
    keeps its card — it is not a pre-main screen. (B) F1 per-surface
    window LITERALS: `WindowMode` goes from three variants to SIX, one
    per pre-main surface plus `Full`, on the same single shared path —
    `apply_window_mode`, the changed-guard and the NA-0662 deferred-show
    sequence are untouched. **360px is the READING WIDTH**, shared by all
    five pre-main surfaces — the operator's chosen measure, found by
    hand-resizing the identity window until the copy composed correctly;
    the round-3 560/460 widths let body text run too long. WIDTH AND
    HEIGHT ARE COUPLED: at 360 the copy wraps into more lines, so the
    heights are MEASURED AT 360 AND ARE NOT VALID AT ANY OTHER WIDTH.
    The table is WizardVault 360x585, WizardIdentity 360x625, Unlock
    360x255, Erase 360x275 (sized to the TALLER of its form 273 and
    countdown 253 states), Wiped 360x220, Full 1024x700. The heights were
    MEASURED headlessly in WebKit2 4.1 — the same engine tauri uses on
    Linux — against the real `ui/index.html`, with `fitCode`'s
    shrink/wrap replicated so the verification code's rendered size is
    included; each is the natural content height plus the 28px top and
    bottom padding, rounded up to the next multiple of 5 so a sub-pixel
    difference cannot clip the last element or trip the card's overflow
    scrollbar (measured → landed: 583→585, 620→625, 250→255, 273→275,
    217→220). INDEPENDENT CORROBORATION: the operator hand-measured the
    identity window at 360 wide as needing 621; the headless measurement
    returned 620, a 1px agreement that confirms the measurement viewport
    matches the real window. An earlier DERIVED (unmeasured) table —
    560x500 / 560x520 / 460x250 / 460x280 / 460x210 — was replaced by
    this measured one after operator visual review found the wide
    literals let text run long and CLIPPED the wizard screens' bottom
    content. The compact minimum is a single floor (360x200, at-or-below
    the shortest window so `set_min_size` cannot re-impose it) instead of
    "minimum == initial", so the pre-main windows stay resizable.
    `tauri.conf.json` windows[0] initial size — `width: 560→360` and
    `height: 660→585`, the wizard-1 literal — are the ONLY keys touched. (C) F4 — the
    verification code can no longer clip silently: `fitCode` still
    shrinks 17px→11px, but at the floor it now adds `.verify-code.wrapped`
    (`white-space: normal; overflow: visible; overflow-wrap: anywhere`)
    so the code WRAPS at a group boundary — the operator's ruled
    preference — instead of being cut off by `overflow: hidden`; and a
    debounced `resize` listener refits BOTH call sites, where before this
    lane `ui/` contained ZERO resize listeners of any kind and a code
    fitted at render was never refitted. The `.wrapped` rule is placed
    AFTER the base block: ordering is load-bearing, because the frozen
    needle in `design_round2.rs` slices from the FIRST `.verify-code` to
    the next `}` and requires `white-space: nowrap` inside it — the base
    block keeps it, so no frozen assertion breaks and the flagged
    horizontal-scroll fallback was NOT needed. (D) the Settings
    destroy-vault ceremony REPLACES its trigger button rather than
    sitting below it; Cancel restores it, and so does any state
    transition via `clearCeremonyState`. Behavior only — the passphrase,
    typed-phrase and tokened-core-call gates are byte-unchanged.
    APPENDIX E amended minimally, each edit citing its section: [E.1]
    (the size table, the wiped notice named for the first time, the
    compact floor, and the WINDOW-IS-THE-CARD rule replacing a round-3
    formulation that was satisfiable by stretching the card — which kept
    the void and merely moved it inside) and [E.4] (the ceremony card
    treatment now applies to the SETTINGS surface only; red text carries
    danger on the pre-main screens). MEASURED AT PHASE 1 on main
    `8db2b2a5`, before any edit, and what justified the sizing work:
    trailing void 153px = 23.2% of the wizard-step-1 window and 164px =
    39.0% of the unlock window, with per-screen `xwininfo` geometry
    confirming the round-3 table was real (560x660 / 460x420).
    FINDING C2 SETTLED EMPIRICALLY at the same time: there is NO white
    menu strip on the pre-main screens at `8db2b2a5` — the top rows
    measure exactly RGB(29,29,31) = `--bg` — so the operator's uploaded
    screenshots predate NA-0662 and are superseded. Work item C (the
    native menubar's color) stays DROPPED: it is a Tao/GTK widget
    outside the DOM that no frontend change can reach; theming or hiding
    it is platform-specific work owned by the eventual Appearance-pane /
    dark-frame story, NOT by any frontend lane.
  - **References:** spine D601 (the directive, as approved+amended);
    spine D-1291 (the lane closeout); D-0005 (the round-3 pass this
    builds on); docs/DESIGN_SPEC.md + docs/DESIGN_SPEC_AppendixD.md +
    docs/DESIGN_SPEC_AppendixE.md (the three-file living design
    authority, Appendix E amended here at [E.1] and [E.4]).

- **ID:** D-0007
  - **Status:** Accepted
  - **Date:** 2026-07-24
  - **Decision:** GATE 1 of GUI slice B (server connectivity) — bump the
    qsc pin ONLY, proved before any pane code exists — per spine directive
    D609 (QSL-DIR-2026-07-24-609, approved, sha256 `eb6f9da0…`, 678 lines;
    spine lane NA-0673; GATE-1 result class GUI_SLICE_B_PIN_BUMP_PASS).
    `src-tauri/Cargo.toml` qsc git `rev` `81143dcd` → `ab5041cd`
    (`ab5041cdc8e1d1f8a311303160060a4d708eb48d`) and `Cargo.lock`; NOTHING
    ELSE. No pane, no commands, no `settings.rs`, no claim-surface edits,
    no Appendix F — all GATE 2. The bump crosses NA-0663's TLS-trust
    surface, so it introduces the D599-sanctioned native-roots union: the
    lock gains exactly FIVE crates — `rustls-native-certs` 0.8.4,
    `openssl-probe` 0.2.1 (Linux verifier), `security-framework` 3.7.0 +
    `security-framework-sys` 2.17.0 (macOS), `schannel` 0.1.29 (Windows) —
    and the `qsc` + `quantumshield_refimpl` rev lines move to `ab5041cd`;
    the 32 other resolved deps and all 12 RustCrypto pins are UNCHANGED (no
    cargo-1.95 resolver drift manifested; verified against a before/after
    `Cargo.lock` diff). rustls stays on the ring backend
    (`default-features=false`) — `aws-lc-rs` is ABSENT from the lock, the
    precise failure GATE 1 exists to catch.
  - **Rationale:** GUI slice B's Server pane calls qsc's server-info
    consumer and the token/CA trio, which do not exist at the slice-A pin
    `81143dcd` (it predates NA-0663 / NA-0670 / NA-0672). Proving the pin
    bump ALONE — the native-roots union compiles, no aws-lc-rs, no pin
    drift, the suite green — before any pane code means a lock-alignment
    failure surfaces here, not while debugging a UI. This PR changes
    `Cargo.lock` and the build, so the `rust` CI gate actually runs and its
    green carries the evidence (the opposite of a docs_only PR).
  - **References:** spine D609 (the directive); spine lane NA-0673, GATE 1;
    spine D599 (the qsc client TLS-trust surface, the sanctioned
    native-roots transitive set, and the aws-lc-rs trap ruling); the qsc
    server-info consumer + token/CA trio landed at spine NA-0672 (rev
    `ab5041cd`); D-0006 (the round-4a pass this builds on). GATE 2 (the
    pane) and GATE 3 (the spine closeout) follow under D609.

- **ID:** D-0008
  - **Status:** Accepted
  - **Date:** 2026-07-24
  - **Decision:** Land GATE 2 of GUI slice B — the Server pane — per spine
    directive D609 (QSL-DIR-2026-07-24-609, approved, sha256 `eb6f9da0…`,
    678 lines; spine lane NA-0673; result class
    GUI_SLICE_B_SERVER_PANE_PASS). `#pane-server` becomes the full pane:
    relay-address + access-token + CA inputs, Test/Save, a results panel.
    Backend: thin `relay_*` Tauri commands forwarding onto the qsc surface
    NA-0672 shipped, EVERY qsc call through the serial blocking gate
    (`gateway.rs`) — the pane constructs NO HTTP client and never touches
    `relay_server_info_from_parts` (R1: an out-of-gate fetch is a runtime
    panic). `settings.rs` gains `relay_url` (the `self_alias` pattern —
    serde default + skip-when-empty; the allowlist test extended to it; the
    `deny_unknown_fields` downgrade property KNOWINGLY untouched, R6). The
    Test button maps qsc's PRE-CLASSIFIED outcome to the 13 results states
    (R3): 7 probe outcomes + the "Not saved yet" save-state + idle +
    clear-on-edit + the 3 `Err`-channel states. R2 `Err` mapping: a bad
    address → INLINE field validation (never a card); an unreadable
    configured CA → its OWN line, EXPLICITLY NOT CertNotTrusted (R2b); a
    client build failure → a generic line. The two 401 messages are LOCAL
    observations, never server verdicts (R3). No connect-anyway control
    (R8). The five-surface claim-discipline sweep (R4): About in-app
    (`ui/main.js` + the `commands.rs` slice string), About native menu
    (`lib.rs`), footer (`index.html` + `main.js`), welcome stub
    (`index.html`) — the two COMPOUND surfaces edited surgically, the
    surviving true clauses kept ("no security-assurance claims"; "Adding
    contacts arrives in a future update"). Appendix F
    (`docs/DESIGN_SPEC_AppendixF.md`) is the new design authority for the
    pane.
  - **Design calls recorded (see Appendix F):** (a) R7 — the results reuse
    the shipped §2 status-banner component with only `neutral`/`accent`;
    RED (`status-danger`) is RESERVED for the vault-danger surfaces
    (DESIGN_SPEC §2), so a connection FAILURE is `accent` (attention), not
    red — the message carries the severity. The mockup's red "bad" / amber
    "warn" coding is deliberately NOT copied (reading a mockup colour is a
    STOP). (b) "Save persists ONLY the URL" (directive) → the token and CA
    commit through their OWN Set/Clear controls (the vault trios), not Save;
    the probe reads them from the vault, so they must be committed to be
    exercised.
  - **Scope + two necessary deviations, recorded:** touched `ui/index.html`,
    `ui/main.js`, `src-tauri/src/commands.rs`, `src-tauri/src/settings.rs`,
    `src-tauri/src/lib.rs`, `src-tauri/tests/server_pane.rs` (new, additive),
    `src-tauri/tests/slice_a_rules.rs` + `slice_a_flows.rs` (the slice-A
    "zero networking" invariant slice B necessarily breaks — see below),
    `docs/DESIGN_SPEC_AppendixF.md` (new), `DECISIONS.md`. `gateway.rs`,
    `design_round2.rs`, `design_system.rs` BYTE-UNCHANGED (STOP-class); no
    dependency motion (the pin bump was GATE 1). **(i)** The directive's
    GATE-2 MAY-touch names `lib.rs` "About menu comment ONLY", but the 9 new
    Tauri commands MUST be registered in `generate_handler` (also in
    `lib.rs`) or they cannot be invoked — an unavoidable structural
    consequence of adding commands, not a discretionary expansion. **(ii)**
    `ui/style.css` was NOT in the GATE-2 MAY-touch list; the pane's few
    structural needs (the 470px form cap, the results layout) were met with
    inline styles in `index.html` (colours are shipped tokens only, no
    mockup hex) rather than editing `style.css`, to stay within scope.
    **(iii)** the slice-A `zero_networking_in_src_and_ui` test asserted an
    invariant slice B is defined to break; it was REFINED (not deleted) to
    the surviving, meaningful R1 invariant — the desktop crate builds no
    `reqwest`/`hyper` client of its own; all networking goes through qsc.
  - **References:** spine D609 (the directive, R1–R8); spine lane NA-0673,
    GATE 2; the qsc server-info consumer + token/CA trios at spine NA-0672
    (rev `ab5041cd`, pinned in D-0007); D-0007 (the GATE-1 pin bump this
    builds on); docs/DESIGN_SPEC_AppendixF.md (the pane's design authority).
    GATE 3 (the spine governance closeout) follows.

- **ID:** D-0009
  - **Status:** Accepted
  - **Date:** 2026-07-24
  - **Decision:** Amend `docs/DESIGN_SPEC_AppendixF.md` to record the RATIFIED
    REASONING for the two GATE-2 design calls (operator, 2026-07-24), so a
    future lane cannot re-open them. (a) [F.1-BANNER] gains the REASON red is
    not used for connection results: §2 reserves red for irreversible
    vault-loss (armed erasure, autolock-0, the destroy ceremony); a relay you
    cannot reach is an INCONVENIENCE, not a danger; if failures rendered in the
    same red, the palette would stop meaning anything and red on the destroy
    ceremony would carry no more weight than a typo'd hostname; severity belongs
    to the MESSAGE; declining the mockup's red is R7 working (the mockup is the
    layout authority, not the palette). (b) [F.1-COMMIT] is reframed as a
    RULING-REFINEMENT, not a deviation: "Save persists only the URL" left the
    token with nowhere to commit, so own Set/Clear controls via the vault trios
    is the only shape consistent with both standing rulings. Doc-only (Appendix
    F + this log); no code, no test, no behaviour change.
  - **Rationale:** The reasoning arrived at ratification, AFTER GATE 2 (D-0008)
    merged, so it could not ride that PR. Recording WHY — not merely the
    decision — is what stops a later lane re-litigating the palette or the
    commit model.
  - **References:** spine D609 (R7, the design authority); D-0008 (GATE 2, which
    landed Appendix F and made the two design calls); DESIGN_SPEC §2 (the red
    reservation this reason cites).

- **ID:** D-0010
  - **Status:** Accepted
  - **Date:** 2026-07-25
  - **Decision:** Redesign the Server pane's interaction model and layout,
    REVERSING **[F.1-COMMIT]** (recorded in D-0008, reasoned in D-0009 — one
    lane old). (a) **ONE unified Save commits everything**: the relay URL to
    `settings.json`, the token and CA path to the qsc vault through the
    EXISTING `relay_token_set/_clear` and `relay_ca_file_set/_clear` trios.
    (b) **Test saves first**: on a dirty pane Test commits, then probes the
    just-saved state; a clean pane's Test commits nothing. (c) The four
    per-field **Set token / Clear / Set CA file / Clear buttons are REMOVED**,
    replaced by per-field "remove it" prose links whose removal is PENDING
    until the next commit and is cancelled by typing. (d) Layout becomes
    **three sections** separated by exactly two hairlines at `var(--sp-6)`.
    (e) Results **state 8 ("Not saved yet") is REMOVED** — its job folds into
    a new dirty helper, "Settings changed — not saved." (accent, never red) —
    and **state 14 ("Couldn't save settings") is ADDED**. (f) State 10's
    trigger broadens from "any field edited" to "any change to what the app
    will use." States 1-7, 9, 11, 12, 13 are UNCHANGED in trigger and wording.
    UI + tests + docs only: no `src-tauri/src/**` change, no new backend
    command, no qsc API change, no dependency motion, no colour or token
    change.
  - **Rationale:** The split commit model was locally coherent and globally a
    trap. The probe reads the token FROM THE VAULT, so typing a new token and
    pressing Test — the obvious gesture, and the one the layout invited —
    probed the OLD token, and then reported that result TRUTHFULLY. That is
    what made it dangerous: "Token rejected" for a token the user believed
    they had just replaced is indistinguishable on screen from a genuinely bad
    token. The model could have been patched with a warning ("press Set token
    before testing"); **removing the trap beats warning about it**, because a
    warning puts the burden on the user to remember an ordering the interface
    itself created. "Secrets to the vault, URL to settings" is UNCHANGED and
    still binds — only the user-facing commit surface unified. D-0009's
    reasoning was correct for the affordances that existed when it was
    written; it is marked superseded in Appendix F, not deleted, because a
    reversal without the original reasoning reads as drift.
  - **Operator rulings folded (2026-07-25):** **F1R** "remove it" also clears
    the results block, so state 10's rule is one sentence and the reference
    mockups' dirty-helper-plus-results composite is unreachable by
    construction. **F2R** the hairline padding is `--sp-6` (32px): the
    mockup's 30px sat exactly between `--sp-x28` and `--sp-6` with no nearer
    step, and no new token was added; the 2px is a sanctioned deviation.
    **F3R** the reference mockups commit SANITIZED — this repository is
    PUBLIC and the captured markup used a private host as its illustrative
    example, so exactly two values were replaced with the placeholders the
    shipped pane already uses (`https://relay.example.net`, `/path/to/ca.pem`)
    and nothing else differs. The originals are deliberately NOT restated in
    the committed files or in this entry: restating them would republish
    precisely what the substitution removes. Committed under `docs/mockups/`:
    `06e-server-pane.html` sha256
    `07b0400076af8991127d745632512f707ac81bc2bfc7407bec71a8caec39c359`,
    `06e2-server-pane-no-token.html` sha256
    `184cdf3871a29240765c4c06c4fc21b3ce2fa1d336e793a180da41cfefd92836`.
  - **⚠ AMENDMENT TO R-B1 — Director ruling 2026-07-25, UPHELD as
    implemented.** D610's **C2** fixed the commit order as "validate the URL →
    token → CA → `settings.json` LAST", on the premise that the URL, unlike
    the CA path, could be validated WITHOUT writing. **That premise is false**,
    and was found false during implementation: the app registers nine relay
    commands and none is validate-only — `relay_config_set` runs
    `normalize_relay_endpoint` and writes in the same call, exactly as
    `relay_ca_file_set` validates BY writing. Neither field can be checked
    without committing it. That put two rulings in direct conflict: **R-B1**
    wants vault writes first and `settings.json` LAST; **R-B2** wants a
    malformed address to block the ENTIRE commit with NOTHING persisted, on
    Save AND on Test. Honouring R-B1's order lands the vault writes before the
    bad address is rejected (R-B2 broken); honouring R-B2 forces the address
    to commit first (R-B1's ordering inverted).
    **RULED (Director, 2026-07-25): R-B2's guarantee GOVERNS. R-B1's
    vault-first ordering is AMENDED to address-first.** Rationale, as given:
    *an absolute stated guarantee — nothing persists on validation failure —
    outranks unexplained write ordering; and partial-commit-on-vault-failure
    is acceptable because state 14 reports it honestly and a re-test heals
    it.* The implementation stands unchanged; this entry records the amendment
    rather than a deviation.
    Consequences, recorded so they are not rediscovered: the commit is a
    SEQUENCE, not a transaction; if a vault write fails the address has
    ALREADY been saved; state 14 names which part failed; the remainder is
    abandoned; the probe does not run; and the pane re-reads live state after
    any failed commit so it never describes state a partial commit has already
    changed. **The healing path is a re-test** — fix the failing field, press
    Test again, and the commit completes from where it stopped.
    **D610's C2 text is NOT rewritten.** The directive is sha-pinned in the
    spine queue block (`6b8e8ac1…`), so amending it in place would break that
    pin and quietly rewrite an approved document; the same mark-don't-rewrite
    discipline this lane applies to Appendix F applies to the directive. C2
    stands as approved and is superseded HERE, with the pointer recorded in
    the spine closeout (D-1304/D-1305).
  - **Two further deviations, both small and both to avoid asserting something
    untrue:** (i) R-C1 specified the in-flight line as "Testing…" on BOTH
    paths; Save performs no probe, so the Save path reads "Saving…" — the
    mechanism R-C1 specifies (both buttons disabled, neutral treatment, no
    re-entry) is implemented exactly. (ii) R-E6's optional "Settings saved."
    is implemented as MANDATORY on the test-committed path: under
    Test-saves-first the commit is otherwise silent, and a dirty helper merely
    disappearing is absence-of-signal, not confirmation.
  - **Census corrections carried from D610:** the CA path input renders EMPTY
    even when a CA is set, because `relay_ca_file_show()` returns
    `{configured, path_hash}` and the path is never retrievable — the mockups
    draw it populated and the app genuinely cannot know that value (C3); the
    disclosure summary uses R-D3's wording, not the mockups' "(optional)",
    which predates the relabel (C4); the Server pane's frozen needles live in
    `src-tauri/tests/server_pane.rs`, NOT `design_round3.rs` as the lane
    intent's G3 stated — `server_pane.rs` pinned the four removed buttons and
    was updated in the same commit as the markup, with NEGATIVE pins added so
    their removal cannot silently regress (C5).
  - **ENG-0073 is discharged by construction.** The finding (two adjacent
    controls both labelled "Clear", mis-clicked twice during the NA-0673
    acceptance flight, each time producing a plausible-looking wrong result
    card) was superseded rather than fixed: the controls it describes no
    longer exist. A removal affordance that lives inside the sentence
    describing its own field cannot be confused with its neighbour.
  - **References:** spine D610
    (`QSL-DIR-2026-07-25-610_server_pane_redesign.md`, APPROVED 2026-07-25,
    sha256 `6b8e8ac11d9375e53b8362335b812ce68fa4419f9655c16593392bd60a3516ed`)
    and the operator-approved lane intent it was formalized from (sha256
    `a3113bae67e4e9e1473c756720753773a3e5ab089075ead85a46a3c30addc42d`);
    D-0008 (the pane this replaces) and D-0009 (the reasoning now marked
    superseded); `docs/DESIGN_SPEC_AppendixF.md` [F.1-COMMIT-v2] and F.4-v2;
    spine ENG-0073 (superseded) and ENG-0072 (the seat-identity fix that ran
    as this lane's setup step). The live operator acceptance flight is OWED
    and is the evidence for the interaction model; CI green is not.

- **ID:** D-0011
  - **Status:** Accepted
  - **Date:** 2026-07-25
  - **Decision:** Fix three defects in the D-0010 Server pane, **all three found
    by the operator's live acceptance flight and none by the 70-test suite**,
    and add a regression pin for each. (a) **The dirty helper claimed "not
    saved" about settings that were saved.** `renderDirty()` ran inside
    `refreshServerState()`, i.e. BEFORE the R-B5 echo wrote the normalized URL
    back into the field; when normalization changed the string the field still
    held the raw text while `savedRelayUrl` held the normalized one, so the pane
    read as dirty. The echo then corrected the field and nothing re-evaluated
    the helper. Fixed by calling `renderDirty()` AFTER the echo in both commit
    handlers. (b) **A stale "Testing…" banner persisted under the inline
    address error.** The failed-commit path did `await refreshServerState()`
    BEFORE clearing the results panel; that call reaches `relay_token_show` /
    `relay_ca_file_show`, both of which run on the process-wide SERIAL blocking
    gate, so a probe still in flight against a dead address parked the await
    for the whole TCP timeout and the clear never ran. Fixed by extracting
    `handleFailedCommit()`, whose inline branch clears FIRST and awaits
    NOTHING. (c) **State 14 opened with a raw error code** — `mapErr` falls
    through to `String(e)` when a code has no mapping, and that was
    concatenated onto the front of the sentence, yielding "vault_write_failed
    The access token wasn't saved…". Fixed so prose leads and the code stays in
    parentheses at the end, matching the other four commit-failure messages.
  - **Rationale:** (b) is the substantive one and it was a reasoning error, not
    a typo: C2(b) requires re-reading live state after a **partial** commit,
    because something landed. R-B2 guarantees a validation failure persists
    **nothing** — so that branch has nothing to re-read, and applying the
    obligation to a branch it does not cover is what put a gated call in the
    way. The visible symptom was the pane asserting a test was running when
    none had been attempted, on the exact surface whose purpose is to never
    describe a state that isn't real. (a) is the same class one layer down: an
    ordering assumption ("refreshServerState re-renders everything") that held
    only while the typed and normalized forms happened to match. (c) is
    cosmetic but lands in the one message a user reaches ONLY after something
    has already gone wrong, which is the worst moment to show an internal
    identifier where a sentence belongs. **No design ruling changed** —
    Appendix F already specified the correct behaviour in all three cases;
    these were deviations from it, so the spec needs no revision.
  - **On the tests:** all three defects sat behind 70 passing assertions. (a)
    is invisible unless the typed address differs from its normalized form —
    `https://192` expanding to `https://0.0.0.192` under WHATWG IPv4 shorthand
    is what exposed it. (b) needs a *slow* probe still holding the serial gate
    when the next action starts, i.e. real network latency plus an impatient
    human. Neither is reachable by a socket-free structural test, which is
    precisely the argument for the live flight: **CI green was never the
    acceptance, and here is the proof.** The three new pins were verified as a
    POSITIVE CONTROL — run against the merged (defective) `main`, exactly the
    three fail and the other ten pass; against the fix, all thirteen pass. A
    pin that also passes on the buggy code documents nothing.
  - **References:** D-0010 (the pane these defects are in); spine D610
    (R-B2, R-B5, R-E5, R-F2, and C2(b) — the obligation misapplied in (b));
    `docs/DESIGN_SPEC_AppendixF.md` F.2b (the dirty helper), F.2c (in-flight),
    F.2 state 11 and state 14; the flight evidence in
    `/srv/qbuild/evidence/NA-0674/flight/`.

- **ID:** D-0012
  - **Status:** Accepted
  - **Date:** 2026-07-25
  - **Decision:** Correct the README's status section, which still told readers
    this build "makes no network connections at all" — false since D-0007/D-0008
    shipped the Server pane and D-0010/D-0011 redesigned it. The replacement
    states the shipped boundary and nothing beyond it: the local
    vault/identity/unlock lifecycle **and** the Settings › Server pane (relay
    address, access token, CA certificate file) with one Save committing the
    pane and Test connection saving first, then **"the app opens a network
    connection only when you press Test connection"**, and finally what is still
    absent — **no messaging** (no sending, no receiving, no contacts) and no
    release. The claim-boundary paragraph beneath it is byte-unchanged. The
    redesign is noted **by its behaviour** rather than by lane number: this is a
    public page, and a reader who has never heard of NA-0674 learns more from
    "Save commits the pane; Test saves first" than from an identifier (Director
    ruling, 2026-07-25).
  - **The claim was proven before it was written.** `grep -c 'invoke("relay_test"'
    ui/main.js` == 1, and that single call site sits inside the Test-connection
    handler; every other frontend `invoke` is local. The sentence follows the
    measurement.
  - **Anti-regression pin added** to `server_pane.rs`'s
    `claim_discipline_five_surfaces_swept`: `repo_file("README.md")` must not
    contain "makes no network connections at all". **The stale sentence survived
    two lanes that retired it everywhere else precisely because that block
    covered the app surfaces and nothing covered the page about the app.**
    `repo_file()` panics when a path does not resolve, so the pin cannot pass by
    silently reading nothing. **Proved by positive control:** it FAILS against
    the uncorrected README and passes after the correction
    (`/srv/qbuild/evidence/NA-0675/a2_pin_positive_control.txt`) — a pin that has
    never failed is not known to pin anything.
  - **Deliberately NOT changed:** `src-tauri/src/**` is byte-untouched, and
    `settings.rs`'s 60-minute idle-autolock default in particular. A review note
    proposed reverting it to 15; the census found 60 is the **sanctioned** value,
    having superseded 15 by the operator's own decision recorded at D-0005
    ("autolock default 60 with 0 VALID meaning never-auto-lock … superseding the
    15-minute default"). The live residue — the roadmap doc still saying "~15
    min" — is already filed as spine WF-0024 and belongs to its own micro-lane.
  - **Goals:** G4 (truthfulness of the published claim boundary). Tests 73 pass
    / 1 ignored; fmt and clippy clean.
  - **References:** spine D611 (APPROVED 2026-07-25, sha256
    `e2087a656570e6f3d2d3ac88f603f701e056ff4452a89e20703a7f67418c5b78`, 240
    lines; census corrections C1–C4, flags A1/A2 both ruled YES) and spine
    NA-0675; D-0007/D-0008 (the pane), D-0010/D-0011 (its redesign), D-0005 (the
    autolock supersession); `/srv/qbuild/evidence/NA-0675/`.

- **ID:** D-0014
  - **Status:** Accepted
  - **Date:** 2026-07-25
  - **Decision:** Land the operator-infrastructure literal gate, the advisories
    gate, and `--all-targets` clippy in this repository, per spine **D613**
    (NA-0677). **⚠ The intent this lane came from said "port the spine's
    public-safety job". There was nothing to port:** that job scans for private
    keys and cloud tokens and has never contained an address, path or host
    pattern — which is exactly why it ran green on every pull request that
    published a private LAN address. The failure was the pattern set, not the
    scan's scope. `scripts/ci/infra_literal_scan.py` is that missing pattern set,
    adapted from the operator-side scanner whose class vocabulary it keeps
    (provenance recorded in the file header and the workflow).
  - **Two tiers, and one deliberately absent.** TIER 1 (network-identifying
    literals and personal identity) is scanned over the **whole tracked tree** and
    fails on any hit. TIER 2b (low-frequency private names) is scanned over
    **added lines only**, so pre-existing occurrences are left alone and every new
    one fails. TIER 2a (the build-root and home paths) is **not scanned at all**:
    the governance convention cites directives and lane intents by absolute path,
    so those literals are added by roughly 60% of governance commits and a gate on
    them would be switched off within a week.
  - **The private names are stored as salted SHA-256 digests, not as text.** This
    file and the scanner are public. A pattern file spelling the names out would
    republish exactly what the sanitize lane removed — and the Tier-1 whole-tree
    scan would then **hit its own pattern file**, so the gate would fail itself on
    the day it landed. That was verified, not assumed. Structural classes
    (RFC-1918 address forms, a mail-provider domain, the tailnet-hostname form)
    keep literal regexes, because they describe a shape rather than a name.
  - **⚠ Anchoring: no naked word boundaries, and the first design was rejected by
    its own control.** `\b` does not match inside `HOST_NAME` — the underscore is
    a word character and kills the boundary — which is the exact case the previous
    lane was blind to. Raw substring matching fixes that but fires on every
    identifier that merely *spans* a camelCase seam: a 7-character host name sits
    inside `setServerBusy` and `commitServerSettings`, and the first control run
    buried the real hit under 11 false positives from this repository's own UI
    code. **Matching is therefore TOKEN-WISE**, splitting on non-alphanumeric
    characters *and* camelCase transitions, so `HOST_NAME`, `SOME_NAME_THING` and
    `name-lan-relay` all hit while `setServerBusy` does not. Residual, stated
    rather than hidden: a name written with no delimiter and no case change is one
    token and will not match.
  - **Advisories: `cargo audit --deny warnings` with every waived ID named
    individually** in `.cargo/audit.toml` — **no blanket waivers**, because
    dropping `--deny warnings` would accept every future unmaintained or unsound
    crate silently. **`RUSTSEC-2024-0429` is dispositioned separately and
    explicitly as an UNSOUNDNESS waiver, not an unmaintained one** — an earlier
    lane record described all 17 findings as "the gtk3 unmaintained family", which
    is wrong twice: six are not GTK bindings at all, and that one is an
    unsoundness in `glib::VariantStrIter` reaching this crate through
    tauri → wry → webkit2gtk. Waiving it is a real risk acceptance and is written
    as a sentence a reader can disagree with. A dated re-check is owed.
  - **Clippy `--all-targets`** replaces the lib+bin-only invocation, with the five
    findings it exposes fixed in the same commit (the flag and the fix must land
    together or the PR is red by construction). All five were
    `field_reassign_with_default` in test code; no production code changed.
  - **⚠ These two jobs are ADVISORY, not blocking.** This repository requires
    exactly one status context, `rust`. `public-safety` and `advisories` run and
    report on every PR but **cannot block a merge** until the operator adds them
    to the required set — a branch-protection change, which is the operator's act.
    Green is not the same as blocking and this decision does not claim otherwise.
  - **A pre-commit call site** (`scripts/hooks/pre-commit`, opt-in via
    `git config core.hooksPath scripts/hooks`) runs the **same** instrument over
    the staged set. The pattern set is not forked. It is a convenience that saves
    a round trip; **CI is the enforcement**, since hooks are not cloned.
  - **Every control was run RED first.** The embedded-literal control forced the
    matcher redesign above; the Tier-1 control caught an invalid test of its own
    (an unstaged seed is not tracked, so `--mode tree` correctly did not see it);
    the waiver control confirmed the gate still fails on an unwaived advisory.
    Evidence: `/srv/qbuild/evidence/NA-0677/gate_positive_control.txt`.
  - **Goals:** G4. Tests 73 pass / 1 ignored; fmt, `clippy --all-targets` and
    `cargo audit --deny warnings` all clean.
  - **References:** spine D613 (APPROVED 2026-07-25, amended after Lane B, sha256
    `22b3b509…5927655e62f39499`, 407 lines) and spine NA-0677; NA-0676/D-1307 (the
    sanitize that made a whole-tree tier adoptable, and whose closeout found the
    anchoring gap); D-0012 (the claim lane that preceded this one).

- **ID:** D-0015
  - **Date:** 2026-07-26
  - **Lane:** NA-0680 (spine), directive **D615** (`QSL-DIR-2026-07-26-615_onboarding_settings_polish.md`, APPROVED 2026-07-26, sha256 `32a15f3f9bb2542b2d9117d1ef72c8b6d158dc316c79bfc32c9bda7195de8e9c`, 399 lines) — **GATE 1 of 3**.
  - **Decision:** the app-wide focus ring and the two onboarding steps are re-landed per the
    operator-approved polish rulings **R-1, R-6, R-7, R-8, R-9**, with the reference mockups
    committed under `docs/mockups/` as the layout authority they were used as.
  - **⚠ R-1's premise was wrong and the correction is the whole change.** Mockup 10 described the
    shipped focus treatment as `border-color + box-shadow`. **There is no focus `box-shadow` in
    this repository and never was** — what shipped is `outline: 2px` at `outline-offset: 1px`,
    which reads as a detached ring. So the change is **outline → border**, and a lane that
    faithfully "removed the glow" would have edited nothing and shipped the 2px outline intact.
    Inputs now take a true 1px accent border with the outline suppressed (mockup 10's `.proposed`
    rule); every other focusable takes a flush 1px accent outline, because `.settings-rail .cat`
    is `border: none` and the prose links and `details summary` have no border at all, so "1px
    accent border" is not directly expressible there. **No colour is introduced** —
    `--accent-fill` is the shipped token and is exactly mockup 10's `--border-focus`.
  - **R-8:** ONE merged intro carries both good-passphrase recipes and the anti-pattern; the
    separate "Length matters most…" hint is REMOVED. Two surfaces stating the same rule is where
    the weaker wording survives a later edit. `length` is bold, not caps.
  - **R-9:** the no-recovery box is recoloured amber → **accent**, and the class is renamed
    `.warn` → `.callout` — a class named "warn" rendering in accent is a lie the next reader has
    to decode. Full prominence kept (bordered, whole first sentence bold). **The `--warn-*` and
    `--amber*` tokens stay defined**: `.alert-amber` still consumes them and is out of this
    lane's scope, so deleting them would be a second, unruled change.
  - **R-6/R-7:** the identity step follows mockup 07B — heading **"This is you"**, name field
    first, then the verification code, then the technical disclosure, then ONE **Continue**. The
    name is **required** (trimmed non-empty) to leave the wizard; no error text. ⚠ **Onboarding
    only** (D615 F4) — Settings still accepts an empty name and falls back to "You", because
    profiles created before this gate existed have one and must not be held hostage by a new rule.
  - **⚠ Two frozen needles were AMENDED, not deleted.** `design_round2::step2_heading_is_your_identity`
    had its assertion **inverted** (the heading is still pinned, at its new value, and the old
    wording is now what must not survive) — deleting it would have left the heading unpinned,
    which is exactly what the round-2 pin existed to prevent. `design_system::appendix_a_copy_verbatim`
    lost only the one line R-8 deletes; the replacement copy is pinned in the new
    `design_polish.rs`, so the claim did not become unpinned — it moved.
  - **⚠ Every authored needle ships with a proof it can fail** (the operator's standing rule).
    Eight controls were run: each pinned property was broken deliberately, the test observed RED,
    the break reverted, and the suite observed GREEN again at 79 passed / 1 ignored.
    **Two of the eight caught defects in the needles themselves before any of them could give a
    false assurance:** a blanket `!css.contains("box-shadow")` fired on
    `.settings-rail .cat.active`'s inset active-nav bar, which has nothing to do with focus and
    would have forced an unrelated control to be rewritten to satisfy a focus rule; and a bare
    `!html.contains("Your identity")` fired on mockup 07B's own subtitle ("Your identity was
    created and is stored in your vault"), i.e. a substring ban would have forbidden the copy the
    mockup specifies. Both were narrowed to the property actually being pinned.
  - **Not in this gate:** the Identity and Vault & Security panes, the content-driven window
    sizing (R-14), and R-17/R-18 — all GATE 2. **R-16 is untouched by design:** the redirect is
    implemented and does cover Settings, so it is ruled not a code defect and waits on the
    operator's live rig reproduction before GATE 2 closes. **R-19 is not in this lane at all.**
  - **Goals:** G1. Tests 79 pass / 1 ignored / 0 fail; `cargo fmt --check` and
    `clippy --all-targets -D warnings` both clean.
  - **References:** spine D615 and NA-0680; D-0010/D-0011 (the Server pane this lane must not
    touch); D-0014 (the CI gate whose positive-control discipline this lane reuses).

- **ID:** D-0016
  - **Date:** 2026-07-26
  - **Lane:** NA-0680, directive **D615** (sha256 `32a15f3f9bb2542b2d9117d1ef72c8b6d158dc316c79bfc32c9bda7195de8e9c`, 399 lines) — **GATE 2 of 3**.
  - **Decision:** the Identity and Vault & Security panes are re-landed per **R-2, R-4, R-5,
    R-10–R-13**; window sizing becomes **content-driven** (R-14); errors get **per-site** plain
    language (R-17); the guarded-unlock dead-field defect is fixed (R-18); and the armed state
    shows the **remaining** attempt count (R-11/R-15).
  - **⚠ F1 IS A REFINEMENT, NOT A REVERSAL, AND THE DISTINCTION IS TEXT vs CHROME.** Persistent
    state moves from filled banners to quiet status lines; danger-coloured **text** is permitted
    on danger-class state, while danger-coloured **chrome** (borders, fills, card backgrounds)
    stays absolute to the destroy ceremony. `design_round3::autolock_banner_state_machine` was
    **amended, not deleted**: the property it encodes — 0 gets danger treatment, >0 does not — is
    unchanged and re-pinned against the status-line renderer. Deleting it because its old
    selector disappeared would have silently unpinned the never-locks warning, which is the one
    autolock state that actually endangers the vault. The banner component itself SURVIVES and is
    still asserted: it narrowed to one consumer (the Server pane's results), it did not go away.
  - **⚠ R-14 IS FIXED AS A CLASS.** Every pre-main window height was measured once, headlessly,
    against the EMPTY state of that surface's conditional elements — the unlock window's 255 had
    no room for the "Locked after inactivity." line, and with `overflow-y: auto` the
    "Delete vault?" link fell below the fold. Wizard step 1 (`#cli-notice`) and the erase screen
    (`#erase-error`) carry the same latent defect. **The table is now a FLOOR**: the frontend
    reports its measured content height on the existing `ui_surface_changed` carrier and the
    window takes `max(table, measured + padding)`. **No height literal changed** — `lib.rs`,
    `design_round3.rs:322-331` and the `tauri.conf.json` 585 all keep their values; only their
    meaning becomes "minimum", which is why the ruled fix is *less* invasive to the frozen
    needles than three hand-measured bumps would have been.
  - **⚠ THE ORDERING TRAP, AND WHY ITS TEST IS THE ONE THAT MATTERS.** The autolock path calls
    `show("scr-unlock")` and writes the feedback line **afterwards**, so a sync wired only to the
    surface change would miss the very content R-14 exists for — passing its own test while the
    window still clips. The sync therefore runs at `show()`, **after any write to a conditional
    element**, and on the existing resize listener; and the negative control **removes the sync
    that follows the autolock write specifically** and observes RED.
  - **R-17 maps BY SITE, not by code.** In the destroy pane `vault_locked` means WRONG
    PASSPHRASE — Settings is unlock-gated, so the vault is demonstrably unlocked and the generic
    locked-vault wording would have been FALSE at the one site the finding named as its example.
    Recorded as Appendix **F.8**. `mapErr`'s bare fall-through — the mechanism behind NA-0674's
    naked `vault_write_failed` — is replaced everywhere by a lead sentence with the code in
    parentheses.
  - **R-18's dead field was real but not where the ruling looked.** `countdownTimer` was cleared
    but never nulled at natural expiry, so after the first countdown of a session the re-enable
    predicate fell through to comparing the feedback element's class string — and the `catch`
    branch sets `"feedback reject"`, leaving Unlock **permanently disabled** with a raw error
    above it. The handle is now nulled and the predicate is state-driven.
  - **F7 as ratified:** Disarm is `class="danger danger-outline"` — **the danger tier token is
    mandatory and outline is only a modifier.** A bare `danger-outline` scores ZERO tier tokens
    and fails `design_system::every_button_is_tiered_or_nav`, established by RUNNING that test
    against both spellings rather than reading it. **F7a: Arm is unchanged.**
  - **⚠ The Server pane is untouched.** `.pane-sect` is a NEW class mirroring `.srv-sect`'s
    idiom; the Server pane's own rules are asserted byte-intact by `design_polish.rs`. The
    duplicated declarations are the deliberate cost of not disturbing NA-0674's shipped
    acceptance evidence.
  - **⚠ Seventeen negative controls across the two gates** (8 in GATE 1, 9 here), each breaking
    the pinned property, observing RED, reverting, and observing GREEN. **Three caught defects in
    the NEEDLES rather than the code** — all three the same shape: a substring ban applied too
    widely. A blanket `box-shadow` ban fired on the active-nav bar; a bare `"Your identity"` ban
    fired on the mockup's own subtitle; and a `"copy"` ban fired on **the comment explaining that
    there is no copy button**. The third produced a shared `strip_html_comments` helper, because
    a needle that bans a substring across a region must exclude the prose documenting the ban.
  - **Filed, not fixed (operator-ruled):** **ENG-0075** — `cargo test -q` in the desktop CI hides
    WHICH tests ran, so a deleted test file can stay green at a lower total nobody compares. It
    rides a CI/tooling lane. Filed in the GATE-3 spine closeout.
  - **R-16 is untouched by design.** No redirect code exists; the redirect is implemented and does
    cover Settings, and the operator's live rig reproduction decides its disposition.
  - **Goals:** G1. Tests 88 pass / 1 ignored / 0 fail; `cargo fmt --check` and
    `clippy --all-targets -D warnings` clean.
  - **References:** D-0015 (GATE 1); spine D615/NA-0680; D-0010/D-0011 (the Server pane this lane
    must not touch); Appendix F.8 (the new wordings); Appendix E [E.1] (sizes → minima).

- **ID:** D-0017
  - **Date:** 2026-07-26
  - **Lane:** NA-0680, directive **D615** — **acceptance-flight fixes** (GATE 2 reopened).
  - **Decision:** the operator's live flight produced findings CI cannot reach. Findings 1–4 land
    here; **Finding 5 is HELD** (below).
  - **⚠ FINDING 1 — window sizing was not content-driven, in EITHER direction, app-wide.** Seven
    instances, one root cause. **My own floor caused six of them:** `height_for` returned
    `max(table, measured)` on my reasoning that the per-surface table encoded a chosen reading
    composition. **That was an inference the operator never stated**, and a floor holds a window
    open when its content is shorter — which is exactly "too tall". The unlock window is the proof
    in one surface: it clipped, then over-corrected, so it was never tracking content either way.
    **The floor is removed**; the measurement governs and the only clamp is the mode's absolute
    minimum, which exists so a window cannot become un-draggable. The table height survives ONLY
    as the pre-first-report fallback.
  - **Finding 1, instance 4 (Settings too wide) — diagnosed as a BAD DEFAULT, not remembered
    geometry.** There is **no window-state plugin**, so nothing persists across launches. `Full`
    was a hard-coded 1024 while the Settings content needs 812. The "sometimes" was deterministic:
    `mode_for_surface` mapped **both** `scr-main` and `scr-settings` to `Full`, so opening Settings
    never resized anything. **`Full` is SPLIT into `Settings` and `Main`**, and the Settings width
    is **DERIVED** from the layout caps it must contain (52 + 160 + 560 + 40), asserted as that sum
    rather than as a literal. The `tauri.conf.json` `minWidth: 800 / minHeight: 600` creation floor
    is **removed** — left in place it would silently re-impose itself over any content-driven size.
  - **⚠ FINDING 4 — the answer was NEITHER option the operator offered.** The section-padding token
    is CORRECT at `--sp-6` (32px) on both `.pane-sect` and `.srv-sect`. The extra height came from
    **`.pane h3`'s 12px TOP margin stacking on it** → 44px where mockup 09 draws 32. **Blast radius,
    counted:** Vault has 4 `h3`s; Identity/Server/About/Appearance/Notifications have **zero**. The
    rule is shared, the effect is one pane — so the fix is scoped to **`.pane-sect h3`**, leaving
    `.pane h3` alone so no future pane is silently restyled.
  - **Finding 3 (operator ruling):** the technical-details disclosure is REMOVED from onboarding and
    KEPT in Settings › Identity. Nothing is being verified yet at that point in the flow, so the
    fingerprint and mechanism line are premature there even collapsed; identity detail belongs where
    it is acted on. Mockup 07B updated in lockstep. The needle asserts **both** directions.
  - **Finding 2:** the verification code clipped its own glyph bottoms intermittently. `overflow:
    hidden` is load-bearing for the shrink-to-fit logic and stays, so the fix is an **explicit
    `line-height`** that scales with whatever size `fitCode` lands on, plus a re-order so `fitCode`
    runs BEFORE the height measurement — the window was being sized against a code about to change
    size. ⚠ **This is a PIXEL defect and cannot be verified headlessly; it needs the re-flight.**
  - **⚠ FINDING 5 — HELD, NOT LANDED. See ENG-0076.** The operator ruled `settings.json`'s
    EXISTENCE as the resume signal (correctly — the `self_alias`-absent alternative was withdrawn
    because `skip_serializing_if` omits an empty alias, making key-absent indistinguishable from a
    cleared name and from every pre-R-7 profile, including the operator's own). **Implementing it
    breaks SEVEN behavioural tests in `slice_a_flows.rs`**, a file design_round3's header declares
    **byte-frozen**, because `d_interruption_matrix` *already encodes this exact scenario* and
    asserts **S2**: create vault → `identity_ensure` → no settings write → **S2**. That is D595's
    normative S1/S2 discriminator, and it was correct when written — a nameless identity was the
    sanctioned default until R-7 made a name mandatory. **The change is therefore a normative
    amendment to D595's launch-state machine, not the one-line resolver fix it was ruled as.**
    Reverted from this commit and reported for a ruling rather than landed silently.
  - **⚠ Five negative controls, all RED**, including reinstating the floor (the exact defect the
    flight found) and returning Settings to the shared mode. ⚠ A **fourth** substring-ban defect
    surfaced: the resolver's own comment recording why `self_alias` was ruled out tripped two
    `self_alias` bans, producing a `strip_rust_comments` helper alongside the HTML one.
  - **Goals:** G1. Tests 92 pass / 1 ignored / 0 fail; fmt and clippy `--all-targets` clean.
  - **References:** D-0015, D-0016; spine D615/NA-0680; **ENG-0076** (the held Finding-5
    regression); ENG-0075 (the `-q` CI gap, also filed at closeout).

- **ID:** D-0018
  - **Date:** 2026-07-26
  - **Lane:** NA-0680 — **a D595 CONTRACT REVISION riding a polish lane.** Recorded loudly, not
    folded in: same discipline as the `[F.1-COMMIT]` reversal.
  - **Decision:** D595's **S1/S2 discriminator is revised**. S2 was "vault and identity exist";
    it becomes "vault exists AND the onboarding identity step FINISHED", where finished is
    signalled by `settings.json` existing beside the identity record.
  - **Why (and why it is a supersession, not a break):** D595's definition was **correct when
    written** — a nameless identity was a valid completed state, and the wizard's own copy said
    "leave empty to be shown as You". **R-7 (D615, this lane) made the name MANDATORY** to leave
    the identity step, superseding that premise without updating the definition. The gap is a real
    regression, tracked as **ENG-0076**: the identity record is written when the step OPENS
    (`identity_ensure` → `identity_self_kem_keypair` → `identity_write_public_record`), so killing
    the app between the step opening and Continue left a keypair on disk with no name, resume
    resolved S2, and the user landed in main with R-7's gate silently bypassed, shown as "You".
  - **⚠ THE SIGNAL, and why not the obvious one.** `self_alias`-absent was ruled out:
    `skip_serializing_if = "String::is_empty"` OMITS an empty alias, so "key absent" is
    indistinguishable from "name cleared in Settings" **and** matches every profile created before
    R-7 — which D615's F4 exists to protect. The operator's own live profile is an instance
    (`{"autolock_minutes":1}`, no alias key, identity present); that signal would have re-routed a
    completed profile through onboarding. **`settings.json`'s EXISTENCE is unambiguous**, because
    NO write path precedes Continue — traced, not assumed: `vault_create` writes no settings,
    `settings_get` is read-only, the boot path never saves, and the alias/autolock/relay writers
    are reachable only after main. That trace is PINNED by
    `design_polish::no_settings_write_precedes_onboarding_continue`, because a future pre-Continue
    write would break the signal with no other test failing.
  - **⚠ THE FROZEN-FILE UNFREEZE, with the scope corrected from 7 to 2.** The change was first
    reported as breaking SEVEN `slice_a_flows.rs` tests. **Only TWO actually fail**
    (`d_interruption_matrix`, `c_prime_deferred_path_to_honest_disconnected`); the other five pass
    in isolation and were failing as a **cascade** — the file's shared `env_lock()` `Mutex` is
    POISONED by the first panic, so every later `.lock().unwrap()` panics too. **The suite's
    failure list overstated the blast radius 3.5×**, and the operator's stop condition ("if any
    amendment encodes something R-7 did NOT supersede, STOP") is exactly what forced checking each
    one individually rather than trusting the count.
  - **Both real amendments ARE R-7 supersessions**, so no stop condition fired.
    `d_interruption_matrix` models this precise interruption and sanctioned S2 for it;
    `c_prime`'s core sequence stops at identity GENERATION, which post-R-7 is mid-step. Both now
    assert **S1 until the step finishes**, then S2 — so the tests still pin the discriminator,
    at its revised value, rather than being deleted.
  - **Negative controls:** making a settings write reachable before Continue → **RED**; reverting
    the resolver to "identity exists ⇒ S2" → **RED** on the amended contract test.
  - **Goals:** G1. Tests 93 pass / 1 ignored / 0 fail; fmt and clippy `--all-targets` clean.
  - **References:** **ENG-0076** (the R-7 regression this fixes); D-0015/D-0016/D-0017 (this
    lane); D595 (the revised contract); D615 F4 (why the alias signal was rejected).

- **ID:** D-0019
  - **Date:** 2026-07-26
  - **Lane:** NA-0680 — **re-flight fixes.** The operator flew the staged checklist; these are the
    outcomes CI could not reach.
  - **⚠ THE FOUR SIZING FINDINGS COLLAPSE TO TWO CSS FACTS, NEITHER OF THEM WRITTEN BY THIS LANE.**
    - **FACT 1 — the card is STRETCHED to the window.** `.screen` is
      `position:absolute; inset:0; display:flex` with `align-items: stretch` (round 4a), so the
      card's height IS the window's. A stretched box whose content is shorter reports its OWN
      height from `scrollHeight`, so the measurement was **self-referential** —
      `measured = window_height` — and the window could grow but never shrink. It is also why two
      different surfaces reported an identical 388×765: **the size was inherited, not computed.**
      Fixed by releasing the stretch (`align-self: flex-start`) for the duration of the read.
    - **FACT 2 — the card's children SHRINK.** Nothing in the stylesheet ever set `flex-shrink`,
      so every child of the flex column carried the default `1`. A window shorter than its content
      **squashed** the children instead of the card scrolling, and the code box's `overflow:
      hidden` turned that squash into a clipped glyph. Fixed with `flex-shrink: 0`.
  - **⚠ FACT 2 REFRAMES THE ORIGINAL R-14 DEFECT.** "Delete vault?" was never below a scroll fold —
    it was squashed. That is why raising the window height *appeared* to fix it, and why the code
    box clipped again on the resume path where the window arrives short (592 inner vs 700). One
    cause, two symptoms, two lanes apart.
  - **⚠ THE D-0016 `line-height` "FIX" WAS AIMED AT THE WRONG MECHANISM.** It is KEPT as deliberate
    headroom but **re-labelled**: it passed on the fresh path only because the window happened to
    be tall enough for nothing to shrink. The comment claiming it was the remedy is corrected
    rather than left to mislead the next reader into thinking the box is already defended.
  - **⚠ AND THE D-0017 SIZING TEST WAS A HOLLOW PROOF — MINE.**
    `every_window_tracks_its_content_in_both_directions` feeds `height_for` SYNTHETIC values, so it
    passed green on a build whose windows were visibly wrong in six places: **the defect was in
    what reached the function.** Its doc now states that scope explicitly, and the pipeline is
    pinned separately by `measurement_releases_the_stretch_before_reading`. I named this failure
    mode three times in this lane before shipping an instance of it.
  - **Instance 4 corrected:** the Settings width mechanism worked (840 = 812 + 28 chrome, matching
    the pre-main 388 = 360 + 28), but was derived off `.pane`'s 560 cap instead of `.pane-form`'s
    520 — 40px too wide, visible as **asymmetric insets** (20px left, 60px right). The hairlines
    span `.pane-form`, so `.pane-form` decides the width. Now `52 + 160 + 520 + 40 = 772`.
  - **Copy corrected:** the autolock-exemption note is **dropped**, not relocated. This lane first
    read R-12's "moves to the wizard" as "needs a home there"; flown, it reads as noise where the
    user has no autolock context. `autolock_helper_verbatim` amended again to assert its absence.
  - **⚠ A DOCUMENTED HAZARD IN THIS FILE WAS TRIPPED ANYWAY.** `style.css` already warned that
    `design_round2`'s frozen needle slices from the FIRST `.verify-code` in the file, so no earlier
    comment may name that selector. The new FACT-2 comment named it and moved the slice off the
    rule. Reworded, and the warning is now restated at the site that tripped it.
  - **Verified live (operator flight):** ENG-0076 / D-0018 **PASSES** — no `settings.json` after a
    kill before Continue, and the relaunch resumes AT the identity step. Unlock sizing correct.
    Fresh-path code box clean. Onboarding copy, name gate and disclosure placement all correct.
  - **Four negative controls, all RED:** removing the un-stretch; dropping `flex-shrink: 0`;
    reverting the width derivation; restoring the dropped note.
  - **Goals:** G1. Tests 96 pass / 1 ignored / 0 fail; fmt and clippy `--all-targets` clean.
  - **References:** D-0015/D-0016/D-0017/D-0018 (this lane); R-14 (the defect this finally
    diagnoses); ENG-0076 (verified live here).

- **ID:** D-0020
  - **Date:** 2026-07-26
  - **Lane:** NA-0680 — **round-2 re-flight fix.** R-14, third occurrence.
  - **⚠ THE ORIGINAL R-14 DEFECT REAPPEARED, ON THE MOST COMMON PATH IN THE APP.** The operator's
    round-2 flight found "Delete vault?" vanishing the moment a wrong passphrase is entered:
    the feedback text appears, content grows, the window does not, and the link is pushed out of
    view — enlarging the window by hand brings it back, which is the signature.
  - **⚠ WHY IT SURVIVED TWO FIXES: I WROTE THE RULE AND IMPLEMENTED THE INSTANCE.** D615 says the
    height sync runs "after **ANY** write to a conditional element". D-0017 wired it at the ONE
    write the finding happened to name — the autolock notice — plus the CLI notice and the identity
    error. The unlock feedback line has **six** writers (the empty reset, the empty-passphrase
    guard, the rejected-attempt line, the countdown tick, the countdown expiry, the error path) and
    **none of them resized**. The single most frequently hit conditional element in the app was the
    one left out.
  - **THE FIX IS STRUCTURAL, NOT ANOTHER REMINDER.** All six writers are routed through one
    `setUnlockFeedback(kind, text)` helper that writes **and** resizes in the same call. There is
    now exactly ONE way to write that element and it cannot forget.
  - **⚠ AND THE TEST PINS THE CLASS, NOT THE INSTANCE — which is the actual lesson.** Asserting
    that the six known writers call the sync would repeat the original mistake: it can only ever
    cover the writers that exist today. `unlock_feedback_has_exactly_one_writer_and_it_resizes`
    instead asserts there is exactly ONE reference to `#unlock-feedback` in code and that it
    resizes, so a **new** writer added later cannot reintroduce the defect without failing.
    `window_height_syncs_on_the_autolock_path_not_just_surface_change` is amended to pin the
    autolock path through the new writer, since the operator ruled that path must stay covered.
  - **⚠ A TEST ANCHOR WAS WRONG IN THE SAME "FIRST MATCH" SHAPE AS THE `.verify-code` SLICE.**
    `showUnlockScreen("main")` appears three times (route, the idle timer, the menu Lock-now) and a
    bare `find` returned route's. Re-anchored on `autolockMinutes * 60 * 1000`, which is unique to
    the idle timer.
  - **⚠ ONE NEGATIVE CONTROL SILENTLY NO-OPPED AND REPORTED GREEN.** The control removing the
    resize from the helper did not match its target, so the test "passed" — indistinguishable from
    a test that cannot fail. Re-run with the edit **asserted applied first**, it goes RED correctly.
    Same family as the `-q` and failure-list findings: **a control is itself an instrument, and an
    unverified one proves nothing.**
  - **Round-2 flight results, all verified live:** instance 1 **649** and instance 2 **636** —
    different heights where round 1 had both pinned at 765, confirming the measurement now computes
    rather than inherits; Settings **800 wide with even insets** (772 derived + 28 chrome); the
    resume-path code box **636, identical to the fresh path** — path-independent, which is what
    content-driven means; erase confirm and countdown clean.
  - **⚠ INSTANCE 7 WAS A CHECKLIST ERROR, NOT AN APP DEFECT.** The "Vault erased" screen is reached
    only from the armed-wipe path (`unlock_attempt` returning `wiped`); the manual erase reloads to
    S0 → Create vault, which is correct — the user CHOSE to erase, and that notice exists to explain
    an erasure they did not initiate. The armed-wipe terminal was verified working in round 1.
  - **Goals:** G1. Tests 97 pass / 1 ignored / 0 fail; fmt and clippy `--all-targets` clean.
  - **References:** D-0019 (Facts 1 and 2); D-0017 (the instance-scoped fix this supersedes);
    R-14 (the defect, now diagnosed three times to two causes).

- **ID:** D-0021
  - **Date:** 2026-07-29
  - **Lane:** NA-0683 — **the naming sweep.** Spine directive **D618** (sha256
    `48d77b12…f9d8b390`, 445 lines, all six flags ruled); spine decisions **D-1320** (the
    ruling itself) and **D-1321** (closeout).
  - **THE RULING (operator, 2026-07-27):** the user-facing term is **"Relay", never
    "Server"**. The rationale is the part that must survive: *relay* **teaches the security
    model** — a dumb pipe forwarding opaque bytes, not a trust-holding service — it matches
    the protocol docs and the invite system, and it suits a privacy-conscious audience.
    That first clause is why the sweep stops where it does: the word is doing security-model
    work **on surfaces a user reads**, and nowhere else.
  - **⚠ THE WORD WAS ALREADY ~90% SHIPPED.** "Relay address", "Relay name", "Open relay",
    "Not a QSL relay", "This relay requires an access token" and `Relay: {url}` shipped
    before this lane — **in the same pane** as "No server configured" and "Couldn't reach
    the server". This lane finishes a word the product had already chosen; it does not
    introduce one.
  - **WHAT CHANGED — 16 lines, 8 files, one word each:** the settings nav item and the pane
    heading (`Server` → `Relay`), the main-window status line, the two "Settings › Server"
    strings, the `Server version` result row, the "Couldn't reach the server" banner, the
    cert-not-trusted sentence, the CA-unreadable sentence, `app_info().slice`'s "server
    connectivity", two README lines, and the rendered text of both reference mockups. Plus
    **F1**: 14 live normative lines in `DESIGN_SPEC.md` / Appendix D / Appendix F, each
    file carrying **exactly one** dated revision line.
  - **⚠ WHAT DELIBERATELY DID NOT CHANGE, AND MUST NOT LATER:** `data-pane="server"`,
    `#pane-server`, `.server-form`, `.srv-sect`, `serverBusy`, `commitServerSettings`,
    `refreshServerState`, `ServerInfoDocDto`, `RelayServerInfoOutcome`, `relay_server_info`,
    `GET /v1/server-info`, and the test file name `server_pane.rs`. **No user reads any of
    them**, and renaming a key, field, route or identifier costs compatibility to buy
    nothing. The settings key is `relay_url` and was already correct.
  - **⚠ ONE LINE WAS RULED UNFIXABLE IN ONE WORD (F2).**
    `DESIGN_SPEC_AppendixD.md:60` reads "No server configured — server setup arrives in a
    future update". The substitution would produce a sentence that is **false** — relay
    setup shipped in slice B, and `server_pane.rs` asserts that clause is gone from the app.
    Correcting it means deciding what the appendix should now say, which is a design
    decision wearing a naming decision's clothes. **Left, and filed to the Slice 4 spec
    refresh.**
  - **⚠ ONE STALE CLAIM WAS RULED LEFT (F5), AND THE REAL DEFECT IS THE GAP AROUND IT.**
    `src-tauri/Cargo.toml:6` still describes the crate as "slice A: serverless skeleton".
    `server_pane.rs::claim_discipline_five_surfaces_swept` asserts that phrase is gone from
    `ui/index.html` and `src-tauri/src/commands.rs` — **Cargo metadata and module docs are
    outside every needle**, and `src-tauri/src/lib.rs:1` carries the same phrase. The naming
    gate prints it as its single `RULED-LEAVE` rather than hiding it. Filed to the
    CI/tooling lane with OBS-1: **the needle gap, not the word, is the defect.**
  - **THE GUARD (F4).** `src-tauri/tests/relay_naming.rs` pins the renamed surfaces and
    asserts the retired literals stay gone — the idiom
    `claim_discipline_five_surfaces_swept` already uses. It was proven **red-capable**
    before being trusted: one retired string reintroduced, the guard observed RED, the file
    restored byte-identically, green again. A rename with no guard is a rename that comes
    back one edit at a time.
  - **⚠ THE CENSUS CORRECTED FOUR OF THE LANE INTENT'S OWN PREMISES**, each measured, none
    inferred: the repo **does** have a public-safety scan (it simply cannot block, since the
    one required context is `rust`); the config-key stop condition **cannot fire** here
    (`relay_url`); **no existing test asserts any string this lane changes**, so the
    "update the test expectations" work set was **empty**; and one user-facing string lives
    in **Rust, not `ui/`** (`commands.rs` → rendered by `main.js`), which a sweep of `ui/`
    alone would have missed.
  - **Goals:** G4 (primary), supports G1. **Evidence lives in the spine** — the census
    table, the cross-repo enumeration and the gate output are full of the retired word and
    would put hundreds of hits into the tree the gate measures.
  - **References:** spine D618 / D-1320 / D-1321; D-0010/D-0011 (the pane this renames);
    D615 (the polish lane that last touched these surfaces).

- **ID:** D-0022
  - **Status:** Accepted
  - **Date:** 2026-07-29
  - **Goals:** G4
  - **Lane:** NA-0684 — the infra-hostname sanitization micro-lane. Spine directive **D619**
    (sha256 `a8dab7f1…7ea65092`, 539 lines, all seven flags ruled plus two post-ruling
    findings ruled); spine decisions **D-1322** (implementation) and **D-1323** (closeout).
  - **Decision:** `docs/DESIGN_SPEC_AppendixF.md` takes **both** edits ruled for its one
    line: the mockup's example relay name becomes `<lan-relay-host>`, and the
    `ServerInfoDoc` field list reads **relay version**. This is **NA-0683's fourteenth F1
    line** — the one that lane could not land.
  - **⚠ WHY IT COULD NOT LAND THEN, AND CAN NOW.** The line carried an
    operator-infrastructure literal **already present on `main`**, and that literal class
    fires on **added lines only**. So the tree read clean until a lane *touched* the line;
    the one-word edit re-added it and the pre-commit gate refused the commit. NA-0683
    **reverted rather than worked around it**, because redacting a hostname inside a naming
    PR would have pre-empted this lane. Removing the literal is what makes the commit
    possible — the two edits are not merely adjacent, they are **mutually enabling**.
  - **⚠ The line number moved between the ruling and the execution** — `:239` → `:241`,
    pushed down by NA-0683's own revision line. The lane anchored on **content**, and the
    directive says so in as many words: *anchor on content, never on a number.*
  - `ServerInfoDoc` is an **identifier and is untouched**, per D-1320's named boundary: the
    naming ruling does user-facing work only, and *a lane that cites it to rename an
    identifier has misread it.*
  - **This file's revision note is one dated line**, per the F1 convention NA-0683
    established; every `⛔ SUPERSEDED` block is untouched.
  - **References:** spine D619 / D-1322 / D-1323; **D-1320's follow-up map**, which records
    this line as F1's deferred fourteenth and is cited rather than re-derived; D-0021 (the
    naming sweep); **ENG-0089** (the instrument finding this line produced).

- **ID:** D-0023
  - **Status:** Accepted
  - **Date:** 2026-07-29
  - **Decision:** Close the **claim-discipline needle gap** (spine ENG-0088) and
    fold in the **desktop suite's enumeration remedy** (spine ENG-0075), under
    spine lane **NA-0686 / D-1325**.
    - `src-tauri/Cargo.toml` `description` and `src-tauri/src/lib.rs`'s module doc
      now read the operator-pre-approved wording: *"QSL desktop client — slices
      A–B: vault, identity and unlock lifecycle, plus relay connectivity.
      Research-stage; makes no security-assurance claims."* Slice B shipped relay
      connectivity, so the previous slice-A-only description had been false for
      two slices.
    - ⚠ **The needles were extended FIRST, because the defect was the needle set
      and not the word.** `claim_discipline_covers_cargo_metadata_and_module_docs`
      now covers Cargo metadata (which reaches package registries and any bundle
      manifest) and module docs (which reach `cargo doc`), asserting both the
      ABSENCE of the retired claim and the PRESENCE of the research-stage and
      no-security-assurance boundaries.
    - ⚠ **The filed description of the gap was slightly wrong, and the correction
      sharpens it.** ENG-0088 recorded that the old guard reads `ui/index.html`
      and `src-tauri/src/commands.rs` *"and nowhere else"*. Measured: it reads
      **five** files, `lib.rs` among them — but only for a DIFFERENT phrase. The
      gap was **a file looked at for the wrong needle**, which is harder to spot
      than a file nobody looked at, and a stronger argument for the same
      conclusion.
    - ⚠ **The new guard caught this very change writing the defect back in:** the
      first draft of the `lib.rs` explanatory note QUOTED the retired claim and
      failed the needle. Recorded because it is direct evidence the guard works on
      live content rather than only on the case it was written for.
    - **ENG-0075 (narrow authorised fold):** CI loses `cargo test -q`, which hid
      which binaries ran and let a deleted test file stay green at a lower total
      nobody compared. `scripts/ci/test_inventory.sh` pins every test NAME in
      `scripts/ci/EXPECTED_TEST_INVENTORY.txt` and fails the build when one
      disappears — **by name, not as a number that moved**, because printing is
      not checking. Baseline **103 tests** (102 passed + 1 ignored, 11 binaries).
      Growth is allowed; disappearance is not.
    - The `infra-literal-scan` self-test runs ahead of the scan, so a broken
      instrument fails before it can report clean.
  - **Controls:** reintroducing the retired phrase into either surface turns the
    new guard red (both directions, both restored byte-identical); deleting
    `src-tauri/tests/relay_naming.rs` drops the enumeration 103 → 98 and fails
    naming the five missing tests.
  - **References:** spine NA-0686 / D-1325; ENG-0088; ENG-0075; D-0021 (the naming
    lane that filed the claim as LEAVE-and-FILE); `server_pane.rs`'s existing
    `claim_discipline_five_surfaces_swept`.

- **ID:** D-0024
  - **Status:** Accepted
  - **Date:** 2026-08-06
  - **Lane:** spine **NA-0697 / D631 as twice amended** (post-A2 sha `010943c9…592e`;
    spine decision **D-1337**; ENG-0048, filed on the protocol ledger by NA-0661).
  - **Decision:** **A tokened DESTROY now removes the app-level `settings.json`
    AND its `settings.json.tmp` staging sibling** (Shape A, ratified at the
    Director's STOP-002 ruling). `destroy_vault`'s gateway closure delegates to
    the new `pub fn destroy_vault_impl(data_dir, passphrase)` — the
    byte-equivalent tokened core destroy followed by the removal loop, each path
    exists-then-`remove_file` with erase's error mapping — the erase-mirror
    testability shape (`erase_all` / `erase_all_impl` precedent), ratified as
    D631 Amendment 2 after NA-0697 STOP 004 proved the originally ruled
    "inline block" shape untestable (the new test must compile at base AND
    exercise the edit; only a pub non-`State` fn satisfies both).
  - **Why (the artifact-level reading, CITING spine D-1337):** the D-1336
    boundary rule classifies ARTIFACTS; the artifact here is **the file**, the
    level at which qsc's own `config.txt`/`store.meta` were classified — and
    `settings.json` is vault-lifecycle-coupled through its D-0018 role: its
    EXISTENCE is the per-profile "identity step finished" signal. "Vault-keyed
    joins the gone-set" reads at the artifact level as semantic keying to the
    profile's lifecycle; the survivor clause protects vault-independent CONFIG
    ARTIFACTS, and a dual-role file whose signal role is load-bearing is not
    one. Field verdicts BY NAME (ruled, Amendment 1): `autolock_minutes`
    survives-as-field · `relay_url` survives-as-field · `self_alias`
    DIES-as-field (profile-scoped; its resurfacing under the next identity IS
    the ENG-0048 complaint) — all three die WITH the file under Shape A, the
    stated cost accepted deliberately (destroy is profile-ending; erase already
    resets both survivors, unfiled as defect).
  - **The §5 finding, closed BY CONSTRUCTION (citing NA-0697 STOP 001):** a
    surviving `settings.json` FORGES the D-0018 S2 signal for the next profile —
    destroy → re-onboard → step 2 opens (identity record written) → kill before
    Continue → relaunch resolves S2 off the OLD profile's file — the precise
    ENG-0076 regression, reintroduced for every post-destroy re-onboarding.
    With the file dead at destroy, the forged-signal path no longer exists.
  - **Instrument:** NEW `src-tauri/tests/eng0048_destroy_boundary.rs::
    destroy_residue_set_enumerated_by_name` — ks1 vault + a REAL settings.json
    (alias + relay non-default) → destroy via `destroy_vault_impl` → `data_dir`
    listing EQUALITY == `["qsc"]` (never a count) + `resolve_launch_state == S0`
    + `!vault_unlocked()`. Deliberately does NOT pin the `qsc/` interior (the
    pinned library owns that boundary). Inventory re-pinned 104 → 105 BY NAME,
    deliberate (D631 R7).
  - **Controls (the A2 mechanism-identical pair):** Control 1′ = base +
    factoring + test, NO removal → exactly this one test red at the listing
    (measured `["qsc", "settings.json"]`), all others green (factoring
    neutrality inside the control); Control 2′ = final tree with the removal
    loop reverted → the identical one-test red; both restores cmp-identical.
  - **⚠ The pin-advance inheritance, BY NAME:** the qsc pin stays `ab5041cd`
    (pre-D-1336). The future pin-advance lane inherits the
    `DestroyConfirmToken::confirm_with_passphrase` → `confirm(typed)` rename at
    the desktop's **three measured constructor call sites**
    (`commands.rs::destroy_vault_impl` + `slice_a_flows.rs:352/:356`; the
    directive's "two call sites" was its enumeration — this record states the
    measured set), plus the D-1336 gone-set/ceremony semantics arriving with
    head qsc. Until then nothing at this pin contradicts the boundary rule (the
    desktop creates ks1 vaults, for which D-1336 keeps destroy byte-unchanged).
  - **Cross-references:** spine D-1337 (the interpretive precedent + the
    cross-repo mechanics this lane sets); **ENG-0119** (erase's own crash-window
    `.tmp` residue — `settings.json.tmp` survives erase if a crash lands between
    settings.rs:59's write and :62's rename; FILED on the protocol ledger by
    this lane, deliberately NOT fixed here); D-0018 (the signal role); D-0002 /
    D-0003 (the settings-file and alias semantics this supersedes at the destroy
    boundary); Slice 4's inherited copy obligation (the destroy copy must state
    device preferences — auto-lock, relay address, display name — are reset).

- **ID:** D-0025
  - **Status:** Accepted
  - **Date:** 2026-08-07
  - **Lane:** spine **NA-0700 / D634 as amended (A2-FINAL)** (directive
    `QSL-DIR-2026-08-07-634_polish_gui_ipc_boundary_closure_A2FINAL.md`, sha256
    `dedc4b374949de059954bf9110c196df6a64dbde088f281f5e3996c5a48c045d`; spine
    decision **D-1340**).
  - **Decision:** **The desktop's IPC arg/DTO boundary is pinned by an in-repo
    replay harness** (`src-tauri/tests/na0700_ipc_replay.rs`): every one of the
    27 registered commands is invoked through tauri's REAL IPC ingestion on the
    mock runtime — real serde arg decoding, real camelCase→snake_case mapping,
    real State injection, the REAL compiled ACL — with the arg-key sets
    HARVESTED from the 31 `main.js` call sites and replayed literally
    (`confirmPhrase`, `autolockMinutes`, `selfAlias`, `contentHeight` among
    them), DTO wire shapes pinned as serialized (the `kind` strings the FE
    string-matches: `unlocked`/`rejected`/`unreachable` pinned live), and the 2
    registered-and-dormant commands (`marker_stats`, `core_busy`) invoked
    registration-level. Enablers, both behaviour-identical to the shipped app:
    the tauri `test` dev-feature (dev-dependencies re-declaration; **measured
    Cargo.lock delta: NONE** — the feature resolves inside the already-locked
    tauri 2.11.5) and the `configure_builder` composition point extracted from
    `run()` (managed state + the one `generate_handler!` list; `run()` composes
    through it, so the harness registers exactly the run-path set;
    `ui_surface_changed`/`apply_window_mode`/`MenuHandles` genericized over
    `tauri::Runtime` for the mock runtime, Wry behaviour unchanged).
  - **Claim boundary (R108, verbatim on this record):** this harness does NOT
    click, type, or read the interface, and closes only the IPC half of the
    GUI-test blindness; the interface half belongs to NA-0701. The desktop
    consumes qsc at pin `ab5041cd` — measured an ancestor of spine base
    `a6abd911`, so it predates the qsc output-routing edit and this harness
    tests the desktop against its pinned qsc: the two NA-0700 halves are
    independent by measurement (R115), desktop-first merge order. The routing
    improvement itself arrives only at the future pin-advance lane (D-0024's
    ceremony; R120(a): the boundary is closed AT qsc HEAD, not in the shipped
    desktop).
  - **Harness findings, recorded so no future seat re-derives them:** (1) the
    mock context's EMPTY capability set rejects every command — the compiled
    ACL from `capabilities/default.json` is part of the boundary, so the
    harness builds with the real `generate_context!()` on the mock runtime;
    (2) config-declared windows are a run()-phase creation — the harness builds
    the `main` webview itself under the ACL's label and uses the webview's own
    origin for the local-scope match; (3) `bootstrap()`'s panic-redaction hook
    would redact every test-assert panic, so the harness replicates bootstrap's
    env/policy/routing steps and omits only the hook (qsc-owned production
    behaviour, tested there).
  - **Instrument:** inventory re-pinned **105 → 106 BY NAME**, deliberate — the
    one added name is
    `all_27_registered_commands_invoke_through_real_ipc_with_fe_arg_shapes`.
    In-harness red-capability: a deliberate missing-required-arg invoke is
    asserted REJECTED at the boundary.
  - **Control (SR-06, one edit, exact red set):** removing the single entry
    `commands::app_info,` from `configure_builder`'s handler list reds exactly
    `{all_27_registered_commands_invoke_through_real_ipc_with_fe_arg_shapes}`
    and nothing else; restore proven cmp-identical.
  - **References:** spine D-1340 (the routing half + the claim sentence); D-0024
    (pin-advance ceremony this record defers to); D-0022/ENG-0075 (the
    inventory-by-name discipline); R108/R109/R111 (harness joins the existing
    `rust` job, no new CI context), R115, R119–R122, R130–R158 via the banked
    NA-0700 record.

- **ID:** D-0026
  - **Status:** Accepted
  - **Date:** 2026-08-08
  - **Lane:** spine **NA-0701 / D636 as amended (A1 + A2)** (directive
    `QSL-DIR-2026-08-08-636_na0701_gui_input_driver_lane.md`, sha256
    `4999eeb147c79d68cf3bcd44ca24b56feffd38b42c1f390cea2bd1f5bc82c9c5`, 140
    lines = base + A1; amendment A2
    `QSL-DIR-2026-08-08-636_na0701_gui_input_driver_lane_A2.md`, sha256
    `4223768ea7fc0642f6e980885667b212b50f1c9b78996e5996ac3ed9ce2bf556`; spine
    decision **D-1341**).
  - **Decision:** **The desktop gains a rendered-DOM GUI input driver**:
    WebDriver clicks, keystrokes, and verbatim reads over the RUNNING app —
    real `POST /session` against the debug binary through
    `xvfb-run → dbus-run-session → tauri-driver` (PINNED `--version 2.0.6
    --locked`) — cargo-wrapped as six `#[ignore]`-marked tests in
    `src-tauri/tests/gui_driver.rs` (`na0701_gui_{a..f}_*`; inventory
    106 → 112 BY NAME) executed for real by the new NON-REQUIRED `gui-driver`
    CI job (`timeout-minutes: 30`; evidence uploaded EVERY run via the one
    new major-pinned `actions/upload-artifact@v4`). The harness
    (`src-tauri/tests/harness/`: the probe's client EVOLVED per R112 +
    `runner.py` + six scenario seeds quoting source BYTES as
    selector+literal pairs) carries as numbered duties: the A2.1 run root
    (`QSLD_GUI_RUN_ROOT`, default repo-local `target/gui_driver_runs/<utc>/`;
    the shared cargo cache is never a data home), the A1.1 standing isolation
    bracket (pre/post real-`$HOME` census byte-identical; destructive steps
    REFUSED without the pre-census), two independent port-0 probes with a
    recorded 3-attempt retry, NEITHER automation env (R164 §3 — the pinned
    driver injects both), bounded polls only (zero sleeps; the erase
    countdown is POLLED, 26 distinct values observed), teardown on every
    exit path via the MEASURED pgid with a comm-name survivor census, and
    json.dumps verdict JSONL frozen before the manifest (R164 §5).
  - **What it caught before landing (findings FILED, not fixed):** the
    erase-form error write skips the R-14 window resize — after a wrong
    ceremony phrase both `#btn-erase` and `#btn-erase-cancel` click centers
    fall outside the card clip and WebDriver measures
    element-not-interactable (ENG row on the spine ledger; F-E therefore
    runs the R170 Option-1 third-launch shape, and the fix-lane's acceptance
    includes restoring the in-place wrong→correct click).
  - **Controls (SR-06; six perturbations, red observed, cmp-identical
    restores under the REBUILD BRACKET — `frontendDist: "../ui"` embeds the
    assets, so a ui/ perturbation without a rebuild is a silent no-op, R171
    §2):** F-A h1 one-char → {F-A} · F-B :263 gate flip → {ALL SIX} (the
    shared A1.11 preamble propagates; ratified R171 3.1) · F-C :465 template
    one-char → {F-C} · F-D :207 arg-key one-char → **{F-C, F-D, F-E}**
    (settings.json is an S2 CONJUNCT at `state.rs:75`; ratified R172 2.1
    after the consumer-census instrument, R172 2.2) · F-E :522 operator flip
    → {F-E} with the full 8-row set matching its written prediction at zero
    delta · F-F :1405 event-name one-char → {F-F settings leg}. Plus the P9
    liveness pair every run.
  - **Claim boundary (A1.22):** real click/type/read over the six flows and
    the measured census (7 screens · 30 buttons · 14 inputs · 6 panes),
    reaching 6 of 7 screens; `scr-wiped` presence-asserted only; native GTK
    menus/WM stay operator-flown (F-F proves the FE listen handlers +
    `app.emit` plumbing via execute/sync, ZERO IPC change, commands stay 27);
    `destroy_vault` end-to-end and armed-wipe are FILED successors;
    perceptual diff deferred severable; macOS/Windows not claimed;
    required-status promotion is the operator's later act.
  - **References:** spine D-1341 (the lane's record half + the ENG filing);
    D-0025/ENG-0075 (inventory-by-name discipline); the NA-0701 sealed
    record: STOP_002 (probe, R164), STOP_007 (Phase 0, R169), STOP_008 (the
    R-14 catch, R170), STOP_009/STOP_010 (the red-set corrections, R171/R172);
    rulings R163–R172 banked as RBANK_NA0701_002–013.

- **ID:** D-0027
  - **Status:** Accepted
  - **Date:** 2026-08-08
  - **Lane:** spine **NA-0702 / D637 as amended (A1)** (directive
    `QSL-DIR-2026-08-08-637_na0702_eng0123_erase_error_resize_fix.md`, sha256
    `4000ae6ace0a9aa99272c43f0a530222124c67eb321940b763f5c8966218a97c`, 88
    lines = base + Amendment A1, A1 governs on conflict; spine decision
    **D-1342**).
  - **Decision:** **`#erase-error` becomes a ONE-RESIZING-WRITER element**
    (ENG-0123 — R-14 class, FOURTH occurrence, the GUI driver's first machine
    catch: after a wrong ceremony phrase the error write skipped the resize
    and BOTH `#btn-erase` and `#btn-erase-cancel` fell outside the card's
    clip on the app's most stressful screen). The `setUnlockFeedback`
    PROPERTY applied verbatim — STRUCTURAL, not a reminder: one TOTAL helper
    `setEraseError(text)` (null-guard retained, resize unconditional) is the
    only way to write the element, its body writes AND calls
    `syncWindowHeight()` in the same operation, and
    `design_polish.rs::erase_error_has_exactly_one_writer_and_it_resizes`
    COUNTS THE REFERENCES (comment-stripped == 1) so a second writer cannot
    appear silently. ALL FOUR base sites absorbed (:488 entry-clear ·
    :515–:516 abort-clear · :520–:523 handler clear + wrong-phrase write ·
    :549 catch write); `syncWindowHeight` call sites 7 → 8; COUNTDOWN CODE
    BYTE-UNTOUCHED (measured FORK B at formalization: the countdown block is
    SHORTER than the form it replaces, cancel bottom 225.0 vs clip 245.0 =
    20.0px margin; the NAMED near-miss is FILED on the spine ledger with its
    numbers per A1.1, not fixed). F-E (`f_e_erase_ceremony.json`, 45 → 51
    steps, existing ops only — runner.py untouched): leg B gains the REAL
    Cancel click at the error state (`#btn-erase-cancel` → scr-unlock; the
    driver's own in-view-centre refusal is the instrument that caught
    ENG-0123, so acceptance is that instrument passing); leg C RETAINED as
    the R170 boot-to-unlock nothing-was-erased instrument with the workaround
    property REMOVED — it deliberately re-enters the error state and carries
    the RESTORED in-place wrong→correct click; the in-file note updated,
    STOP_NA0701_008 kept as origin. Inventory 112 → 113 BY NAME.
  - **The three-point ordering proof (A1.2, executed in this order):**
    (1) the corrected scenario RED against the UNFIXED app — under cargo,
    first-red abort at EXACTLY the leg-B Cancel row (rc=2); under the
    perturbation facility, EXACTLY the pair {leg-B Cancel click, leg-C
    in-place click} both rc=2 `element not interactable` + 7
    consequence-class rows with NOTHING ERASED positively measured
    (vault.qsv present); the born-red control red at the same base (no
    helper; base reference count 4). (2) AFTER the fix, the C2 regression
    (the resize line deleted) reds the SAME pair row-for-row — red for the
    RIGHT REASON, not for the new rows' existence. (3) GREEN with the fix
    via the real consumers: `cargo test --test gui_driver -- --ignored
    --test-threads=1` 6/6, full bare suite 113 names 106/0/7 exit 0.
  - **Controls (SR-06, both under the REBUILD BRACKET — `frontendDist:
    "../ui"` embeds the assets — with cmp-identical restores and re-greens):**
    C1 reinserted direct write at the `#link-forgot` handler →
    EXACTLY {the new control} at count 2, F-E green (an entry-time clear is
    height-neutral) · C2 resize-line deletion → EXACTLY {the control's body
    assertion (b), na0701_gui_e_erase_ceremony per the pair above}.
    Evidence root:
    `target/gui_driver_runs/na0702_execution_20260808T162637Z/`
    (ORDERING_PROOF.md; every red and green run preserved).
  - **Claim boundary (§9):** after a wrong ceremony phrase BOTH ceremony
    buttons are interactable at the measured geometry, proven by the
    driver's own real clicks on the box and on the CI producer as separately
    measured. NOT claimed: every conditional element resize-safe (the sweep
    lane); the countdown near-miss fixed (FILED with its numbers); any
    Slice-4 screen; native GTK menus/WM; macOS/Windows; required-status
    promotion; the promotion-three count (this lane's spine PR is
    record-only).
  - **References:** spine D-1342 + the two IMPROVEMENT_LEDGER rows (ENG-0123
    → resolved; the countdown near-miss filing); D-0026/D-1341 (the driver
    that caught the defect); D-0025/ENG-0075 (inventory-by-name discipline);
    the NA-0702 record chain: RBANK_NA0702_001 (brief) → STOP_NA0702_001
    (formalization; the FORK B measurement) → RBANK_NA0702_002 (R174) →
    STOP_NA0702_002 (promotion PR #1716) → this lane's execution stop.

- **ID:** D-0028
  - **Status:** Accepted
  - **Date:** 2026-08-08
  - **Lane:** spine **NA-0703 / D638 as amended (A1, A2-as-corrected)**
    (directive `QSL-DIR-2026-08-08-638_na0703_mockup_refresh_ratified_designs.md`,
    sha256 `570bfcce61951fdd9550590b26c340dd10fcd2778e9654e9583aa246725943d3`,
    124 lines = 84-line base (cmp-verified) + Amendment A2-as-corrected per
    R177/R178; spine decision **D-1343**).
  - **Decision:** **The ratified GUI designs land in `docs/mockups/` as repo
    truth — 10 adopted, 5 replaced, indexed by a new README, every file
    provenance-marked and sanitized.** The OPERATOR REVIEWED EVERY COMMITTED
    MOCKUP INDIVIDUALLY (A1 review gate: exact bytes in a review directory,
    per-file verdicts, bytes frozen at approval by manifest sha; R179/R180).
    Adopted: mockups 11/12/13/13a/14/15 (chat era), 08b, the
    channel-established banner (States 0/1/2, ratified 2026-08-01, success
    path only — its header points at the companion), the channel-establishment
    FAILURE STATES S-F1..S-F5 (ruled 2026-08-08), and the fingerprint
    two-tier RATIFIED reference. Replaced (each delta measured and named in
    its commit): 07 (superseded `QSCFP-` form → two-tier), 07b (HYBRID, see
    below), 09 (emoji lock → inline SVG), 06e/06e2 (relay-explainer paragraph
    + placeholder example values; D610/D-0010 lineage carried in the headers).
    Unchanged and current: 08, 10. The operator-side early files 01–05 are
    SUPERSEDED-HISTORY and never ship (02/03/05 carry live tailnet literals).
  - **⚠ NA-0680 FINDING 3 STANDS (this entry records the resolution):** the
    2026-07-26 operator ruling (this file, the D-0016 lane record at
    DECISIONS.md:1025) that the "Show technical details" disclosure is
    REMOVED from onboarding is REAFFIRMED — the operator's F5 verdict at
    NA-0703 R179 was HOLD on the delivered 07b's disclosure restoration. The
    2026-08-01 mockup ratification advanced the fingerprint FORMAT ONLY
    (two-tier voice form + 256-bit hex reference). The committed 07b is a
    HYBRID built on the CURRENT repo Finding-3 form with exactly four change
    regions (provenance header · one `.fp` CSS rule · two-tier code card ·
    `e.g. Alex`), its rejection of the disclosure MEASURED: every
    disclosure-identifying needle 0 over the hybrid with each needle proven
    able to fire on the delivered copy first (STOP_NA0703_004 §4; R180).
  - **The four-site operator-name sanitization (A2.1' as corrected by R178
    §3), base → after:** the whole-tree tracked case-insensitive
    operator-name count (the A2·2 needle) moves **4 → 0** — the four sites:
    `docs/mockups/mockup-07-identity-pane.html:50` (a filled field, now
    `value="Alex"`) · `docs/mockups/mockup-07b-onboarding-identity.html:46`
    and `ui/index.html:55` (placeholders, now `placeholder="e.g. Alex"`) ·
    `src-tauri/tests/design_polish.rs:251` (the same placeholder string
    inside the R-6 assertion; string literal only, the assertion's property
    and message untouched). ⚠ The before-literals are deliberately NOT
    respelled here — prose spelling a literal that a whole-tree gate counts
    would trip the gate it records (the D625 §0b.3 class); the literal
    base values are preserved in the operator-side NA-0703 record
    (STOP_003 §4, STOP_004). The ui and test edits land in ONE commit (they pin each
    other); the C-PIN control ran pre-commit as ordered: ui edit alone →
    `identity_step_orders_name_before_code` RED at exactly the :251
    assertion (24 passed / 1 failed, exit 101, log preserved) → test edit →
    25/25 green. Honest limit (R177 §3.5, stated in the PR body): the name
    is removed from CURRENT truth; the string remains in public commit
    history, which no lane rewrites.
  - **Sanitization delta (recorded in full at STOP_NA0703_003 §4):** invite
    code tail → same-length `Example…` string (14/15) · `Dana Krol` → `Dana`
    + avatar `DK` → `D` (banner, failure-states) · the four operator-name sites
    above · KEPT with explicit class calls: Ben/Maria (first-name-class),
    message bodies (fictional), fingerprints (fabricated per the RATIFIED
    artifact's own note), `relay.example.org:8443`, `/home/user/
    relay-ca-root.crt`, the 08b example passphrase.
  - **Claim boundary:** the committed set is the ratified design as of this
    lane, sanitized with its delta recorded, indexed, provenance-marked; the
    repo no longer carries a mockup contradicting ratified design (scoped to
    the classified set). NOT claimed: mockups match any shipped screen
    (layout authority only; tokens are the colour authority); Slice-4 design
    settled; any behavioural change (the one ui/ line is a placeholder
    string); the promotion-three count (this lane's spine PR is
    record-only). Mockup 03's empty-states content has no successor in the
    delivered set — filed as a design gap, not this lane's work; the
    rail-toggle pattern of operator-side mockup 02 is filed on the spine
    ledger (R180 §2.5) so it is not lost when 02 stays operator-side.
  - **References:** spine D-1343; NA-0680 Finding 3 (DECISIONS.md:1025,
    STANDING); D610/D-0010 (06e lineage); the NA-0703 record chain:
    RBANK_NA0703_001 (brief) → RBANK_002 (A1) → STOP_001 (formalization; the
    packet-absent and repo-newer premise corrections) → RBANK_003 (R177) →
    STOP_002 (the R16 fourth-site catch) → RBANK_004 (R178: A2' ratified,
    F8 corrected) → STOP_003 (Phase A: 22 classified, review directory) →
    RBANK_005 (R179: operator verdicts, F5 HOLD) → STOP_004 (the 07b
    hybrid) → RBANK_006 (R180: hybrid COMMIT, Phase B) → this lane's
    execution stop.

- **ID:** D-0029
  - **Status:** Accepted
  - **Date:** 2026-08-09
  - **Lane:** desktop **NA-0705 / D640 as amended (A1, A2)**
    (directive `QSL-DIR-2026-08-09-640_na0705_qsc_pin_bump_A2.md`,
    sha256 `4c5ae91a733823225a2574bbc138b710b3bd46daf7a55b6977f2e25058925a21`,
    236 lines = base + A1 + A2, precedence A2 > A1 > base; spine decision
    **D-1344**).
  - **Decision:** **The desktop's `qsc` pin advances `ab5041cd` → `32e572c7`,
    and because that crosses the `QSCV01` → `QSCV02` vault-format HARD BREAK
    (no migration, no dual-format read — spine D628 Ruling 2), the desktop
    now CLASSIFIES the on-disk envelope BEFORE it touches the guarded vault
    paths, on BOTH doors.**
  - **Why it is not optional.** `qsc` names the refusal correctly
    (`vault_version_unsupported`) at both of its parse sites, but
    `vault/protection.rs:156` collapses every `Err` into one branch, counts a
    failed attempt at `:175`, and at an armed wipe-after-N limit erases the
    vault at `:180`. Measured on a real build at the new pin, before the
    remediation: three CONSECUTIVE **CORRECT** passphrases against a `QSCV01`
    vault produced `rejected`, `rejected`, `wiped` with `vault_exists=false`
    and `failed_unlocks` 0→1. The user is told "Wrong passphrase" while
    entering the right one, and the vault is destroyed.
  - **What was built.** `commands.rs::vault_version_state()` classifies through
    `qsc::adversarial::vault_format::classify_vault_magic` (qsc's one owner of
    magic recognition), reading only the 6 magic bytes of
    `paths::vault_file(data_dir)`. On `KnownOld`, **`unlock_attempt_impl` and
    `destroy_vault_impl` return the refusal WITHOUT CALLING the guarded or
    destroy path at all** — the classification GATES the call, it does not
    interpret its result, because the counting and the wipe happen inside the
    call. New `UnlockDto::VersionUnsupported`; `ui/main.js` gains one honest
    state per door and never says "Wrong passphrase" for this cause.
    `unlock_attempt_impl` was extracted first as a behaviour-preserving seam
    (body moved verbatim) so the instrument had something to be red against.
  - **Both doors, because destroy reaches it independently.** At the new pin
    `destroy_with_passphrase` peeks the envelope through the same parser
    BEFORE examining the passphrase, so a pre-flight on unlock alone would have
    closed one of two. Found by the commissioned SR-15 read (F-2), not by this
    seat.
  - **The instrument (`src-tauri/tests/na0705_qscv01_refusal.rs`, 5 tests).**
    A `QSCV02` vault with its 6-byte magic rewound to `QSCV01` — faithful,
    because the version arm sits before any key derivation. Asserts, for BOTH
    doors: the refusal is named and distinct, `failed_unlocks` does NOT
    increment, and with wipe armed at N, N correct-passphrase attempts do NOT
    wipe. **Watched RED before the remediation existed** (pre-registered red
    set 3 red / 2 green-at-base, measured exactly; the destroy pins were green
    at base and are recorded as regression pins rather than manufactured red),
    then green. ⚠ No other gate in this lane can fail on this defect: the
    compile passes (no signature moved), and the suite, the gui-driver flows
    and the rig e2e walk all create FRESH vaults.
  - **`confirm(typed)` was READ, not renamed into.** `confirm_with_passphrase`
    no longer exists at the new pin (the compile break, `commands.rs:265` +
    `tests/slice_a_flows.rs:352`/`:356`). The replacement constructor is
    value-neutral; what the commitment must equal is decided at the destroy
    site by a runtime branch on the peeked `key_source`. The desktop passes the
    passphrase, and that is correct ONLY because it can never hold a keychain
    vault (no `features` key, qsc `default = []`, `keyring` absent from
    `Cargo.lock`). **That precondition rides as a comment at the call site**, as
    does `vault_create`'s S0 file-existence precondition (`state.rs:71`), which
    is why that guard call needs no pre-flight.
  - **Also carried.** `commands.rs`' "1:1 rendering" comment corrected —
    `ServerInfoDoc` gained three invite limits the DTO does not surface (filed,
    not built). `na0700_ipc_replay.rs` creates its dirs at **0700**, matching
    `create_private_dir`: it claimed to replicate `bootstrap()` and did not, and
    under `umask 0002` its 0775 dirs were refused by `enforce_safe_parents` at
    the new pin. The product was right; the replication was not.
  - **Claim boundary.** The desktop links `qsc` at `32e572c7` and builds; the
    suite is green BY NAME at 118 / 111-0-7. **NOT claimed:** any GUI messaging
    capability (still zero messaging commands), a receive loop, any Slice-4 UI,
    or that the suite covers the bumped messaging code. The keychain-addressing
    break at the new pin is real for the spine and **nil for the desktop**
    (compiled out) — an explicit non-claim.
  - **Record chain:** brief RBANK_001 → STOP_001 (formalization + divergence
    table) → RBANK_002 (R185) → RBANK_003 (R186) → the commissioned SR-15 read
    (FINDINGS, 14 findings, banked RBANK_005/006) → RBANK_004 (R187: 14/14
    dispositioned, A2) → STOP_002 (the A2 fold) → RBANK_007 (R188: execute) →
    RBANK_008 (R189: rig authority) → STOP_003 (halt at the edit-set boundary)
    → RBANK_009 (R190: one file admitted) → this lane's execution stop.

- **ID:** D-0030
  - **Status:** Accepted
  - **Date:** 2026-08-19
  - **Lane:** desktop **NA-0748 / Phase 1 Lane 1** — the `qsc` pin bump, executing the
    Director's ruling **`R360`** of 2026-08-19 (banked verbatim under SR-14 as this turn's
    FIRST ACT at `RULING_NA0748_R360_SR15_DISPOSED_SEALS_V2_BUILD_AUTHORIZED_20260819.md`,
    sha256 `d623d743452ab1e27b4f9fee989e47fd34d88fa21f2cfe1e4b8d329367cb55b3`, 94 lines /
    7526 bytes, mode 444), which disposed the commissioned SR-15 cold read
    (`FINDINGS_SR15_NA0748_READ_20260819T051013Z.md`, sha256
    `b24ad2911739dc716e70a96018c26290d1a92f8441a85fb5a25db5f6d65459ba`, 593 lines — sha
    verified by this seat against the ruling's citation) and amended the seals to **v2**.
    Spine decision **D-1389** (qsl-protocol `e917e7e8`).
  - **Decision:** **The desktop's `qsc` pin advances `32e572c7` → `e917e7e8`, crossing 154
    commits / 10.33 days of the invite–handshake–transport repair arc, with ZERO `.rs`
    edits.** The whole edit is one `rev` value in `src-tauri/Cargo.toml` and the
    **root** `Cargo.lock` regenerated. `ENG-0207` is the ruled disposition (BUMP-THE-PIN);
    the fingerprint format is **NOT** changed here — that is the named successor `qsc` lane,
    and **V3 exists to prove this lane did not move it**.
  - **⚠ The path in the governing brief did not exist.** The brief's §1.8 and V1's first seal
    named `src-tauri/Cargo.lock`, which has **never existed in this repo's history** (0
    commits; positive control: the root `Cargo.lock`, 4 commits). The read caught it as its
    single BLOCKER; `R360` §1 accepted it as the Director's own and amended item 8 to the
    **ROOT** lock. Re-verified independently by this seat before acting.
  - **The seals, v2 — every arm measured.**
    - **V1 BUILD+LOCK — HIT.** `cargo build` and `cargo test --no-run` both exit 0 at the
      target pin. The lock diff is **+2/−2**, exactly the `qsc` and `quantumshield_refimpl`
      `source` lines. Package census **461 → 461: 0 added, 0 removed, 0 version-changed** —
      so the F-4 hazard (the last two bumps each moved transitive dependencies) **did not
      materialise**, reported as a measurement rather than assumed. `aws-lc-rs` **ABSENT
      before and after** (D-0007's gate class), against a positive control confirming `ring`
      present in both.
    - **V2 VAULT CONTINUITY — HIT.** A vault **created at the old pin** (zero failed
      attempts; passphrase from a file, so a typo could not void the ceremony) **unlocks at
      the new pin**: `event=vault_unlock ok=true state=unlocked`, rc 0. Magic bytes read
      **directly from disk** are byte-identical either side — `QSCV02` (`51 53 43 56 30 32
      01 10 …`) — as is `vault status` (`present=true key_source=passphrase`) and the unlock
      output. **Negative control:** a wrong passphrase is **refused** (rc 1), proving the
      unlock validates; the vault re-unlocks cleanly afterwards, so the control did not
      damage the evidence. **Tamper control refuted** (last-byte mutation). The old-pin vault
      is preserved as evidence. ⚠ Per `R360` §4, `vault_version_state` was **not** the
      capture — it has no route (F-9); the magic bytes are the classifier's own input.
    - **V3 FINGERPRINT INVARIANCE — the fingerprint arm HIT; the verification-code arm a
      MISS OF THE ANTECEDENT, stated.** Identity was **ensured at the old pin before
      capture**, and the capture is **PRESENT**: `identity_fp=QSCFP-df7a1df77a49335cbd1e142a2eed24bd`
      (38 chars, non-empty). Re-read at the new pin on the **same vault**: byte-identical —
      indeed the **entire `identity_show` output** compares identical, tamper control
      refuted. ⚠ **The verification code could not be captured at all**: its only in-crate
      callers are `handshake` paths requiring a peer, the CLI never emits it, and the one
      desktop test that computes it builds a **fresh tempdir identity per run**, so it is not
      comparable across pins. Per `R360` §4's own instruction this is recorded as a **MISS of
      the antecedent**, not papered over. **Compensating evidence, with its limits stated:**
      `format_verification_code_from_fingerprint` is byte-identical at both revs and is a
      **pure function of the fingerprint string**, whose only dependency `IDENTITY_FP_PREFIX`
      is also identical — so an identical fingerprint necessarily yields an identical code.
      That is a proof about the mechanism, **not** a captured value.
    - **V4 HARNESS — HIT.** All six `na0701_gui_a..f` **PASS** at the new pin via the
      documented runner, 82.95 s, corroborated from the harness's **own artifacts** rather
      than the cargo summary. **Per-scenario step counts 96 / 20 / 28 / 25 / 52 / 21 = 242**,
      against the 242-step baseline — **deviation 0** (F-17's requirement). `gui_c_lock_unlock`
      passing is the separately-named **desktop-path** unlock evidence `R360` §4 V2 requires.
      ⚠ One difference outside the seal, reported: `gui_c`'s MANIFEST artifact count is **56**
      where Phase 0 recorded **57**; steps (28) and jsonl rows (29) are unchanged.
    - **V5 SUITE — HIT.** The old-pin baseline ran **FIRST**, per the ordering clause, and
      captured the **TEST-NAME LIST** (F-7), not counts: **118 names**. At the new pin the
      list is **118**, with **0 baseline names missing** and 0 added. Compared **per NAME**,
      every verdict is identical (`diff` empty): **111 ok / 7 ignored / 0 failed** at both
      pins, suite rc 0 both times. The already-red branch did not apply.
    - **V6 IDS — HIT.** `D-0030` derived at the edit: max `D-0029`, **0** declarations and
      **0** mentions repo-wide, positive control `D-0029` = 1, negative control `D-0031` = 0.
    - **V7 GATES — the audit review discharged; the PR contexts are recorded at the stop.**
  - **The `.cargo/audit.toml` review, owed by the file's own header — DISCHARGED, with
    before/after (`R360` §3).** `cargo audit` at the **old-pin** lock: 518 dependencies, **0
    vulnerabilities**, **17** distinct advisory IDs firing. At the **regenerated** lock: 518
    dependencies, **0 vulnerabilities**, **17** firing. The waiver list carries **17** IDs.
    **Symmetric difference EMPTY in both directions, before and after** — no waived ID has
    stopped appearing (so nothing is deleted) and no new advisory appeared (so no waiver is
    added, and no STOP is triggered). `cargo audit --deny warnings`, exactly as CI runs it,
    exits **0**. ⇒ **`.cargo/audit.toml` is NOT edited by this lane**, because the only edit
    `R360` §3 authorises is the deletion of a stale ID and there is none.
    ⚠ **Flagged, not done:** the file's header still reads *"Reviewed: 2026-07-25 (NA-0677).
    Next review owed: at the next qsl-desktop dependency bump"*, and that bump is this one.
    Refreshing that date is **outside** the scoped authorisation, which permits deletion
    only; `R360` §3 places the review's record in **this** D-record instead. The header is
    left to the Director rather than self-authorised.
  - **Facts of record carried from the read (`R360` §5).** `decode_failed` no longer
    collapses distinct reject reasons, and the `--self-label` → `--as` CLI rename landed in
    the arc — **both measured unreachable from this consumer** (the desktop imports no
    `qsc::invite::*` and no `qsc::handshake::*` path), and both strengthen `ENG-0206`'s
    typed-surface case. **Nine truly-`pub fn` signatures CHANGED** in the bump, every one
    `self_label: &str` → `Option<&str>`, all in `invite`/`handshake` and therefore
    unreachable here; **0 removed**; and the four items an earlier instrument called "public
    additions" are **`pub(crate)`**, i.e. not public surface at all.
  - **Claim boundary.** The desktop links `qsc` at `e917e7e8` and builds; the suite is green
    **by name** at 118 / 111-0-7 and the GUI harness at 6/6 / 242 steps. **NOT claimed:** any
    GUI messaging capability, any Slice-4 UI, any receive loop, or that this suite exercises
    the bumped invite/handshake/transport code — it does not reach it. The fingerprint
    **format** is unchanged and provably so; `ENG-0205`'s repair is the successor lane.
    `ENG-0207` closes with this bump; `ENG-0202`..`0206`, `ENG-0142`'s remainder, `ENG-0194`
    and `ENG-0197`..`0199` stay OPEN. Zero `.rs` edits; `gui_driver`'s `#[ignore]` untouched
    (7 markers, unchanged); no cargo feature enabled; no test added, edited or weakened;
    `EXPECTED_TEST_INVENTORY.txt` unchanged and its gate green (the read proved it
    growth-only-safe and this lane moves no test).
  - **Record chain:** the Director's brief (banked, sha `89f165c4…`) → STOP 001 (the
    promotion PR, cleared, merged as `e917e7e8`) → STOP 002 (the formalization package,
    seals v1) → the commissioned SR-15 read (**FINDINGS**, 17 findings, 1 BLOCKER) →
    **`R360`** (findings disposed, enumeration amended, seals **v2**, the build authorised)
    → this lane's execution stop.

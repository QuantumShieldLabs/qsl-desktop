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

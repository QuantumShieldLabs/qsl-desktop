# Appendix E — Round-3 reference markup & rule updates (operator-approved 2026-07-19)

Companion to QSC_DESIGN_SPEC_v1_round2.md (§1 tokens still govern values) and
Appendix D (shell rules). This appendix BINDS the round-3 items. Where E and D
disagree, E governs — it is the newer decision.

## E.1 — Window sizing (item 10) — REPLACES the "one window" assumption

The app runs in ONE window mode PER PRE-MAIN SURFACE plus the full mode; the
window is resized on state transition:

| Screen                         | Window size (approx) | Menu bar |
|--------------------------------|----------------------|----------|
| Wizard step 1 (S0)             | 360 x 585, centered  | HIDDEN   |
| Wizard step 2 (S1)             | 360 x 625, centered  | HIDDEN   |
| Unlock (S1/S2 gate)            | 360 x 255, centered  | HIDDEN   |
| Erase everything               | 360 x 275, centered  | HIDDEN   |
| Vault erased (wiped notice)    | 360 x 220, centered  | HIDDEN   |
| Main window + Settings (S2)    | full/default size    | VISIBLE  |

**[E.1] AMENDED round 4a (D601/F1).** Round 3 gave five pre-main surfaces TWO
shared sizes, which is what produced the dead space it forbade: whichever
screen was shorter than its class got the surplus. Each pre-main surface now
carries its OWN size, and the wiped notice is named in the table for the first
time.

**360px is the READING WIDTH** — the operator's chosen measure, found by
hand-resizing the identity window until the copy composed correctly. The
earlier 560/460 widths let body text run too long. **Width and height are
COUPLED:** at 360 the copy wraps into more lines, so every height above is
MEASURED AT 360 AND IS NOT VALID AT ANY OTHER WIDTH — changing the width
invalidates all five heights.

The heights were measured headlessly in WebKit2 4.1 (the same engine tauri
uses on Linux) against the real `ui/index.html`, with `fitCode`'s shrink/wrap
replicated so the verification code's rendered size is included; each is the
natural content height plus the 28px top and bottom padding, rounded up to the
next multiple of 5 so a sub-pixel difference cannot clip the last element.
The erase window is sized to the TALLER of its two states (form 273,
countdown 253), since one window serves both without a resize.

The compact minimum is a single floor (360 x 200) rather than "minimum ==
initial", so the pre-main windows stay resizable. Re-picking a height when
copy changes is an accepted one-line follow-up — but it must be re-measured
at 360, not estimated.

Rules:
- **[E.1] AMENDED round 4a: the WINDOW IS THE CARD.** No container chrome on
  any pre-main screen — see [E.4]. The SCREEN carries the uniform 28px content
  padding, content sits directly on `var(--bg)`, and the window is sized so
  the content ends at that padding. The round-3 formulation ("the card FILLS
  its compact window (page padding 20-24px), no vertical void, no
  centered-card-in-a-void") was satisfiable by STRETCHING the card, which kept
  the void and merely moved it inside the card; sizing the window per surface
  is what actually removes it.
- Keyboard shortcuts and the right-click context menu remain available on
  compact screens (paste must still work without the menu bar).
- Resizing happens on the state transition, not per-render.

## E.2 — Number inputs (item 1)

- No native spinner arrows anywhere (`appearance: textfield`, and
  `::-webkit-outer-spin-button`/`::-webkit-inner-spin-button {display:none}`).
- Styled per §1 field tokens; width ~64px for these two; text centered.
- Validation: numeric only; erase Limit 1-100; autolock 0-1440. Invalid input
  is blocked with VISIBLE feedback (inline message + field border danger) —
  never silently clamped, never silently ignored.

## E.3 — Idle autolock 60 / 0 (item 2)

- Default 60 minutes.
- 0 is VALID and means NEVER AUTO-LOCK.
- Banner state machine for the autolock status banner:
  - value > 0  -> `status-accent` + lock icon: "Locks after {N} minutes of inactivity"
  - value == 0 -> `status-danger` + warning icon, bold:
    "Never locks — anyone with access to this device can open your vault"
- The helper line sits DIRECTLY under the banner (item 7):
  "On by default. Applies to the main window and settings; the setup wizard is exempt."

## E.4 — Ceremony rules (items 3, 4, 5)

**[E.4] AMENDED round 4a (D601/F2 as revised at census review).** The shared
ceremony treatment below now applies to the SETTINGS destroy ceremony ONLY.
On the PRE-MAIN screens — Erase everything, and the wiped notice — ALL
container chrome is removed with every other pre-main card: no background, no
border (including the red one), no radius, no card padding. **Danger on those
screens is carried by the RED TEXT alone** — the `.ceremony-head` /
`.link-danger` colors and the danger copy, all unchanged. This supersedes the
round-3 rule that the red ceremony chrome appears on BOTH destructive
surfaces; it now appears on the Settings one only. No token value changes:
this is the deletion of a border rule, not a recolor.

Both destructive surfaces still share the ceremony's STRUCTURE and its gates
(typed phrase, passphrase where required, the quoted-phrase rendering). The
card treatment below is the Settings rendering:

```html
<div class="ceremony-card">        <!-- bg #1D1D1F, border 1px #8A3A3A, radius 12, padding 20px 22px -->
  <h2 class="ceremony-head">Destroy vault</h2>          <!-- 17px/600 #F0A0A0, mb 8px -->
  <p class="body">Requires your passphrase. Permanently erases this vault —
  this cannot be undone.</p>                            <!-- 13px #A8A8A8, mb 10px -->

  <label class="field-label">Current passphrase</label> <!-- destroy only -->
  <input class="text-input full" type="password">       <!-- FULL WIDTH (item 4) -->

  <p class="body">Type <span class="phrase">"destroy my vault"</span> to confirm</p>
  <!-- .phrase = mono, color #F0A0A0, quotes INCLUDED in the rendered text (item 3) -->
  <input class="text-input full">                       <!-- SAME width as above -->

  <div class="row">
    <button class="btn-destructive">Destroy permanently</button>
    <button class="btn-secondary">Cancel</button>
  </div>
</div>
```

The erase screen is identical minus the passphrase field, with
`"erase everything"` as the phrase and "Erase everything" as the button.

## E.5 — Erase countdown (item 11b)

On confirm, the form is REPLACED (not merely disabled) by:

```html
<div class="ceremony-card">
  <h2 class="ceremony-head">Erase everything</h2>
  <div class="countdown-panel">      <!-- bg #3A1D1D, border 1px #8A3A3A, radius 8,
                                          padding 14px, centered, mb 12px -->
    <p class="countdown-number">30</p>          <!-- 26px/600 #F0A0A0 -->
    <p class="countdown-label">Erasing in 30 seconds…</p>   <!-- 12px #F0A0A0 -->
  </div>
  <p class="body center">Close this window or press Cancel to stop.</p>
  <button class="btn-secondary full">Cancel</button>   <!-- the ONLY action -->
</div>
```

- Counts down 30 -> 0, updating both the number and the label.
- Cancel or closing the window ABORTS (nothing erased).
- The erase executes only on completion.

## E.6 — Unlock screen (item 11a)

```html
<div class="unlock-card">                       <!-- fills the compact window -->
  <h1>Unlock</h1>                               <!-- 17px/600, mb 14px -->
  <label class="field-label">Passphrase</label>
  <input class="text-input full" type="password">
  <!-- error message renders INLINE here ONLY when present; no reserved gap -->
  <p class="error">Wrong passphrase. Failed attempts: 2.</p>   <!-- 12px #F0A0A0 -->
  <!-- delay state: "Too many failed attempts (5). Try again in 15 seconds." -->
  <button class="btn-primary full">Unlock</button>             <!-- mt 12px -->
  <p class="center"><a class="link-danger">Delete vault?</a></p>
  <!-- .link-danger = 12px #C87A7A, underlined; opens Erase everything directly.
       The old "Forgot your passphrase?" wording is REMOVED. -->
</div>
```

## E.7 — Small corrections

- Item 6: the arm checkbox gets a larger hit area (>=16px box, label clickable)
  and the label sits on ONE line.
- Item 7: the gaps around both Vault-and-Security banners come onto the §1
  spacing scale (12/16/24) — no oversized voids.
- Item 8: the Settings verification-code box gets `max-width: 420px; margin:
  0 auto;` so it matches the wizard's proportions.
- Item 9: rail icons ~21px (from ~19px) and the status-bar text 12px
  #A8A8A8 (from 11px #7A7A7A).

## E.8 — Acceptance

Screenshot each surface beside its E.x block; structure, tokens, spacing, and
copy must match. Additionally prove: the countdown aborts and completes; the
autolock 0 danger banner appears AND the app never auto-locks at 0; invalid
number entries are visibly rejected; the compact windows carry no menu bar and
no vertical void, and the window resizes on entering the main view.

# Appendix E — Round-3 reference markup & rule updates (operator-approved 2026-07-19)

Companion to QSC_DESIGN_SPEC_v1_round2.md (§1 tokens still govern values) and
Appendix D (shell rules). This appendix BINDS the round-3 items. Where E and D
disagree, E governs — it is the newer decision.

## E.1 — Window sizing (item 10) — REPLACES the "one window" assumption

The app runs in TWO window modes; the window is resized on state transition:

| Screen                         | Window size (approx) | Menu bar |
|--------------------------------|----------------------|----------|
| Wizard step 1 / step 2 (S0/S1) | 560 x 660, centered  | HIDDEN   |
| Unlock (S1/S2 gate)            | 460 x 420, centered  | HIDDEN   |
| Erase everything               | 460 x 420, centered  | HIDDEN   |
| Main window + Settings (S2)    | full/default size    | VISIBLE  |

Rules:
- The card FILLS its compact window (page padding 20-24px), no vertical void,
  no centered-card-in-a-void.
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

Both destructive surfaces (Settings > Destroy vault, and the Erase everything
screen) share ONE treatment:

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

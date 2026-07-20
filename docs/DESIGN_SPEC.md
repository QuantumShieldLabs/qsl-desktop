# QSC Desktop Design Spec v1 — Round-2 Reference (operator-approved 2026-07-19)

This file is the DESIGN AUTHORITY for the round-2 lane and all future GUI
lanes until revised. Prose in the intent describes intent; THIS FILE defines
pixels. Where the build and this spec disagree, the build is wrong.
Adapt selectors/ids to the app's wiring; copy the visual DNA verbatim.

Round-3 note (2026-07-19): Appendix E (DESIGN_SPEC_AppendixE.md) is the
NEWER decision — where it and this file disagree, Appendix E governs; §1
tokens still govern values. The round-3 amendments below are marked [E.x].

## 1. Tokens (define once as CSS custom properties, use everywhere)

Spacing scale (the ONLY gaps allowed):  4 / 8 / 12 / 16 / 24 / 32 px
Radius: 8px controls, 12px cards
Type scale (dark theme, sizes in px / weight):
  - page-title    17 / 600
  - section-head  14 / 600
  - body          13 / 400
  - hint          12 / 400, color: text-muted
  - mono-code     15-19 / 500, monospace, letter-spacing 1-2px
Text colors: text-primary #E8E8E8 · text-secondary #A8A8A8 · text-muted #7A7A7A
Surfaces: page #1D1D1F · card #252528 · field #1A1A1C · hairline #3A3A3E
Semantic roles (bg / border / text):
  danger  #3A1D1D / #8A3A3A / #F0A0A0
  accent  #1C2A3E / #2E5A8E / #8FBAF0   (primary buttons fill #3D7BC4, text #FFF)
  success #1D3325 / #2E6E45 / #8FDCA8
  neutral-status bg #2A2A2E, icon+text text-secondary
Button tiers:
  primary     = accent fill, white text (ONE per screen)
  secondary   = transparent, 1px hairline border, text-primary
  destructive = danger bg + danger border + danger text
  disabled    = #2A2A2E bg, text-muted, no border emphasis (never dimmed-accent)

## 2. Status banner component (item 12 — approved mockup)

Structure (all status lines in Vault & Security use exactly this shape):

<div class="status-banner status-danger|status-accent|status-neutral">
  <i class="icon"></i><span class="status-text">MESSAGE</span>
</div>

CSS:
.status-banner { display:flex; align-items:center; gap:8px;
  padding:10px 14px; border-radius:8px; font-size:14px; }
.status-danger  { background:#3A1D1D; border:1px solid #8A3A3A;
  color:#F0A0A0; font-weight:600; }   /* icon: warning triangle */
.status-accent  { background:#1C2A3E; border:1px solid #2E5A8E;
  color:#8FBAF0; font-weight:600; }   /* icon: lock */
.status-neutral { background:#2A2A2E; border:none;
  color:#A8A8A8; font-weight:400; }   /* icon: shield-check */

Usage:
  ARMED    -> status-danger : "Armed — erases after {N} failed attempts"
  OFF      -> status-neutral: "Off — wrong attempts never erase the vault"
  AUTOLOCK (value > 0)  -> status-accent : "Locks after {N} minutes of
    inactivity"; (value == 0) -> status-danger + warning icon, bold:
    "Never locks — anyone with access to this device can open your
    vault" [E.3]
Red is RESERVED for the armed-erasure state and, by the round-3 operator
decision, the autolock-0 danger banner, the quoted ceremony phrases, and
the "Delete vault?" link [E.3–E.6]. Icons ~17px, inline SVG or
the app's existing icon approach; outline style.

## 3. Passphrase step (items 1-3, 10 — approved layout)

Order, single column, 12px gaps:
  [label] Passphrase        [field]
  [label] Confirm passphrase[field]        <- directly below, nothing between
  checklist (two rows, 13px):
     ○/✓ 12+ characters
     ○/✓ Passphrases match
     unmet = text-muted circle ○ ; met = success-text ✓
  hint line (12px, text-muted):
     "Length matters most — a few random words beat a short complex password."
  no-recovery warning box (unchanged from current build)
  [Create vault]  primary when both checks met; DISABLED TIER otherwise
NO strength meter. NO third check. Nothing else on the card.

## 4. Verification code display (item 5)

One line, always: font-size chosen so the full code + dashes fits the card
at min window width WITHOUT wrapping (start at 17px mono, shrink to fit;
white-space:nowrap; the format/characters are untouched). Centered, inside
the existing bordered code box. Applies to wizard step 2 AND the Identity
pane (shared style). In the Settings rendering the code box additionally
gets max-width: 420px; margin: 0 auto — matching the wizard's
proportions [E.7].

## 5. Confirmation ceremony pattern (items 6, 13 — destroy AND erase)

Both destructive surfaces sit in the shared red-bordered ceremony card
(bg #1D1D1F, border 1px #8A3A3A, radius 12, padding 20px 22px;
ceremony-head 17px/600 #F0A0A0) [E.4].

Collapsed state: heading "Destroy vault" (ceremony-head, danger text) +
one sentence body: "Requires your passphrase. Permanently erases this
vault — this cannot be undone." + [Destroy vault…] destructive-tier button.

Expanded (after click), 12px gaps:
  [label] Current passphrase   [field, FULL width of the card]
  ONE instruction line (13px): Type "destroy my vault" to confirm — the
    phrase mono, danger text #F0A0A0, the QUOTES INCLUDED in the rendered
    text; the typed phrase value stays unquoted [E.4]
  [field, SAME full width]
  [Destroy permanently] destructive tier   [Cancel] secondary tier
No other prose in the expanded form. The erase-everything screen uses the
IDENTICAL pattern with its phrase, minus the passphrase field; its confirm
is gated by the 30-second countdown panel (the erase commits only at
zero; Cancel or closing the window aborts) [E.5].

STATE RULE (item 13): ceremony forms never persist — every entry to the
pane/screen starts collapsed with ALL fields empty; passphrase fields
cleared on collapse, cancel, completion, and app-state transitions;
destroy/erase completion performs a full webview state reset (reload or
provably-complete clear) so nothing typed survives into the next session.

## 6. Small items

- Item 4: wizard step-2 heading text = "Your identity"
- Round 3 [E.1]: the app runs in TWO window modes, resized on state
  transition — wizard 560x660, unlock/erase 460x420 (both centered, menu
  bar HIDDEN, the card FILLS the compact window, page padding 20-24px);
  main window + Settings full/default size with the menu bar VISIBLE.
- Round 3 [E.2]: number inputs carry no native spinner arrows, are ~64px
  wide, text centered, with VISIBLE validation (erase Limit 1-100;
  autolock 0-1440; invalid input blocked with an inline message + danger
  field border — never silently clamped or ignored).
- Round 3 [E.3]: autolock default 60 minutes; 0 is VALID and means never
  auto-lock (the danger banner renders; the idle timer never fires).
- Item 7: autolock helper = "On by default. Applies to the main window and
  settings; the setup wizard is exempt." (no number restated); the helper
  sits DIRECTLY under the autolock banner [E.3]
- Item 8: Arm = destructive tier; Disarm = secondary tier (per §1)
- Item 9: the duplicated "Off by default. A guest — or a child…" paragraph
  below the status banner is DELETED (the checkbox line carries the warning)
- Item 11: no identity dot in the main-window rail (Chats, Contacts, gear)

## 7. Acceptance standard

Per item: build screenshot beside this spec's definition — matches within
reason (exact tokens, structure, and copy; minor anti-aliasing/font-metric
drift acceptable). Deviations require a recorded reason in the response.

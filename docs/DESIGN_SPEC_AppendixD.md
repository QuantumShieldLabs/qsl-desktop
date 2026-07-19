# Appendix D — Reference markup (operator-approved 2026-07-19)

Companion to QSC_DESIGN_SPEC_v1_round2.md. This appendix BINDS item 14
(full-bleed shell) and supplies the visual DNA for the round-2 surfaces.
Adapt ids/selectors/wiring to the app; copy structure, tokens, spacing,
and copy verbatim. Where this appendix and the spec prose disagree, this
appendix governs for layout; §1 tokens still govern values.

## D.1 — Shell rule (item 14)

- The app shell has NO outer padding, NO inset frame, NO rounded inner
  container. Panes meet the window edges.
- Grid: `52px | list-or-nav column | 1fr`, full window height.
- Separation is `1px solid #3A3A3E` hairlines ONLY.
- The status bar is the full-width last row, `border-top: 1px solid #3A3A3E`,
  padding `6px 14px`, 11px text-muted.
- SETTINGS IS A VIEW, NOT A MODAL: identical shell, icon rail still present
  with the gear active; nav column 160px; content runs to the right edge.
- The onboarding wizard is the ONE exception: a centered card
  (max-width ~440px) on the full-bleed page background.

## D.2 — Main window

```html
<div class="shell">                                  <!-- grid 52px 210px 1fr -->
  <nav class="rail">                                  <!-- bg #1A1A1C, border-right hairline -->
    <button class="rail-item active">                 <!-- 34x34, radius 8, bg #2E3B4E when active -->
      <i class="icon-message"></i>                    <!-- 19px, #8FBAF0 active / #7A7A7A idle -->
    </button>
    <button class="rail-item"><i class="icon-users"></i></button>
    <div class="rail-spacer"></div>                   <!-- flex:1 -->
    <button class="rail-item"><i class="icon-settings"></i></button>
  </nav>

  <section class="list-pane">                         <!-- border-right hairline -->
    <div class="list-header">                         <!-- padding 12px 14px 10px -->
      <h2>Chats</h2>                                  <!-- 16px/500 #E8E8E8 -->
      <div class="search">                            <!-- bg #252528, radius 8, padding 6px 10px -->
        <i class="icon-search"></i><span>Search</span> <!-- 14px icon, 12px #7A7A7A -->
      </div>
    </div>
    <p class="empty-note">No conversations yet.</p>   <!-- 12px #7A7A7A, padding 0 14px -->
  </section>

  <section class="content-pane welcome">              <!-- centered column -->
    <i class="icon-shield-lock"></i>                  <!-- 38px #3D7BC4, margin-bottom 12px -->
    <p class="welcome-title">Welcome to QuantumShield Chat</p>   <!-- 15px/500 #E8E8E8 -->
    <p class="welcome-sub">Add a contact to start your first conversation.</p>
                                                      <!-- 12px #A8A8A8, centered, mb 14px -->
    <button class="btn-primary">Add your first contact</button>
  </section>
</div>
<div class="status-bar">No server configured — server setup arrives in a future update.</div>
```

## D.3 — Settings view

```html
<div class="shell settings">                          <!-- grid 52px 160px 1fr -->
  <nav class="rail">…same rail, gear item active…</nav>

  <nav class="settings-nav">                          <!-- border-right hairline, padding 12px 0 -->
    <h2>Settings</h2>                                 <!-- 16px/500, padding 0 14px, mb 10px -->
    <a class="nav-item active">Identity</a>           <!-- active: bg #1C2A3E, border-left 2px #3D7BC4,
                                                            color #8FBAF0, padding 7px 14px, 13px -->
    <a class="nav-item">Server</a>                    <!-- idle: color #A8A8A8 -->
    <a class="nav-item">Vault and Security</a>
    <a class="nav-item muted">Appearance</a>          <!-- unbuilt: #7A7A7A -->
    <a class="nav-item muted">Notifications</a>
    <a class="nav-item muted">About</a>
  </nav>

  <section class="settings-content">…pane content, padding 16px 20px…</section>
</div>
```

## D.4 — Identity pane content

```html
<h1>Identity</h1>                                     <!-- 17px/500, mb 14px -->
<label class="field-label">What should this device call you?</label>  <!-- 12px #7A7A7A -->
<div class="row">                                     <!-- gap 8px, mb 8px -->
  <input class="text-input" value="Matthew" style="width:190px">
  <button class="btn-secondary">Save</button>
</div>
<p class="hint">Shown as: Matthew (local only)</p>    <!-- 12px #7A7A7A, mb 18px -->

<p class="code-caption">Your verification code</p>    <!-- 12px #7A7A7A, centered, mb 6px -->
<div class="code-box">                                <!-- bg #1A1A1C, 1px #3A3A3E, radius 8,
                                                           padding 13px, centered, mb 12px -->
  <span class="code">QSCF-P26A-0C0B-3B40-4</span>     <!-- mono 17px, letter-spacing 1.5px,
                                                           #E8E8E8, white-space: nowrap -->
</div>
<p class="body">Verification codes exist so you and a contact can confirm you're
really talking to each other — they catch man-in-the-middle substitution.</p>
                                                      <!-- 13px #A8A8A8, line-height 1.5, mb 8px -->
<p class="body">Designed to stay secure even against future quantum computers.</p>
<a class="disclosure">▸ Show technical details</a>    <!-- 13px #8FBAF0 -->
```

## D.5 — Vault and Security pane content

```html
<h1>Vault and Security</h1>                           <!-- 17px/500, mb 16px -->

<h3>Unlock protection</h3>                            <!-- 14px/500 #E8E8E8, mb 4px -->
<p class="hint-block">Wrong attempts are slowed automatically: two free tries,
then a doubling delay capped at 5 minutes. An offline copy of the vault file is
protected only by your passphrase strength.</p>       <!-- 12px #7A7A7A, lh 1.5, mb 20px -->

<h3>Erase vault after failed attempts</h3>            <!-- mb 8px -->
<div class="row">                                     <!-- gap 8px, mb 8px -->
  <span class="inline-label">Limit</span>             <!-- 13px #A8A8A8 -->
  <input class="text-input num" value="10">           <!-- width 56px, centered -->
  <button class="btn-destructive">Arm</button>        <!-- bg #3A1D1D, 1px #8A3A3A, #F0A0A0 -->
  <button class="btn-secondary">Disarm</button>
</div>
<label class="checkline">                             <!-- 13px #A8A8A8, gap 8px, mb 10px -->
  <input type="checkbox"> Reaching the limit erases the vault permanently — I understand
</label>
<div class="status-banner status-danger">             <!-- per spec §2 -->
  <i class="icon-alert-triangle"></i>
  <span>Armed — erases after 10 failed attempts</span>
</div>
<!-- when OFF, the SAME slot renders:
<div class="status-banner status-neutral">
  <i class="icon-shield-check"></i>
  <span>Off — wrong attempts never erase the vault</span>
</div> -->
<!-- NOTHING else here: the duplicated "Off by default. A guest…" prose is deleted (item 9) -->

<h3>Idle autolock</h3>                                <!-- mt 20px, mb 8px -->
<div class="row">
  <span class="inline-label">Lock after</span>
  <input class="text-input num" value="15">
  <span class="inline-label">minutes</span>
  <button class="btn-secondary">Save</button>
</div>
<div class="status-banner status-accent">
  <i class="icon-lock"></i>
  <span>Locks after 15 minutes of inactivity</span>
</div>
<p class="hint">On by default. Applies to the main window and settings;
the setup wizard is exempt.</p>                       <!-- NO number restated (item 7) -->

<h3 class="danger-head">Destroy vault</h3>            <!-- 14px/500 #F0A0A0, mt 20px, mb 4px -->
<p class="body">Requires your passphrase. Permanently erases this vault —
this cannot be undone.</p>                            <!-- 13px #A8A8A8, mb 10px -->
<button class="btn-destructive full">Destroy vault…</button>
<!-- expanded form per spec §5; ALWAYS starts collapsed and empty (item 13) -->
```

## D.6 — Wizard: Create your vault

```html
<div class="wizard-card">                             <!-- max-width 440px, centered on page bg,
                                                           bg #1D1D1F, radius 12, padding 22px 24px -->
  <p class="steps">● ○  Step 1 of 2 — Create your vault</p>   <!-- 11px #7A7A7A, mb 10px -->
  <h1>Create your vault</h1>                          <!-- 17px/500, mb 6px -->
  <p class="body-sm">Your vault protects everything this app stores on this
  computer. It is encrypted with the passphrase you choose.</p>  <!-- 12px #A8A8A8, mb 16px -->

  <label class="field-label">Passphrase</label>
  <input class="text-input full" type="password">     <!-- mb 12px -->
  <label class="field-label">Confirm passphrase</label>
  <input class="text-input full" type="password">     <!-- DIRECTLY below; mb 12px -->

  <p class="check unmet">○ 12+ characters</p>         <!-- 13px; unmet #7A7A7A, met #8FDCA8 "✓" -->
  <p class="check unmet">○ Passphrases match</p>      <!-- mb 10px -->
  <!-- NO strength meter. NO third check. -->

  <p class="hint">Length matters most — a few random words beat a short
  complex password.</p>                               <!-- 12px #7A7A7A, mb 12px -->

  <div class="warn-box">                              <!-- bg #2A2418, 1px #6E5A2E, radius 8,
                                                           padding 10px 12px, mb 14px -->
    <p>There is <strong>no recovery</strong>. If you forget this passphrase,
    no one — not you, not us — can recover your data. The only way back is to
    erase everything and start over.</p>              <!-- 12px #E8C87A, lh 1.5 -->
  </div>

  <button class="btn-disabled full">Create vault</button>
  <!-- disabled: bg #2A2A2E, color #7A7A7A, no border emphasis (item 10)
       enabled:  bg #3D7BC4, color #FFFFFF, 13px/500 -->
</div>
```

## D.7 — Wizard: Your identity

```html
<div class="wizard-card">
  <p class="steps"><span class="done">●</span> ●  Step 2 of 2 — Your identity</p>
                                                      <!-- done dot #8FDCA8 -->
  <h1>Your identity</h1>                              <!-- item 4 rename -->
  <p class="body-sm">This identity was created on this device and belongs to
  this vault. The private part of it never leaves this device.</p>

  <p class="code-caption">Your verification code</p>
  <div class="code-box"><span class="code">QSCF-P26A-0C0B-3B40-4</span></div>
                                                      <!-- one line, never wraps (item 5) -->

  <p class="body">Verification codes exist so you and a contact can confirm
  you're really talking to each other — they catch man-in-the-middle
  substitution.</p>                                   <!-- mb 14px -->

  <label class="field-label">What should this device call you?</label>
  <input class="text-input full">                     <!-- mb 4px -->
  <p class="hint">Local only — leave empty to be shown as "You".</p>   <!-- mb 14px -->

  <p class="body">Designed to stay secure even against future quantum computers.</p>
  <a class="disclosure">▸ Show technical details</a>
  <p class="hint">You don't need to write this down — view it anytime in Settings.</p>
                                                      <!-- mb 16px -->
  <button class="btn-primary full">Done</button>
</div>
```

## D.8 — Acceptance

Screenshot each surface beside its D.x block. Structure, tokens, spacing,
and copy must match; anti-aliasing and font-metric drift are acceptable.
Any deviation requires a recorded reason in the response.

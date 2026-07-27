// qsl-desktop slice A frontend. Static vanilla JS (F3: zero npm/node — no JS
// supply chain). All state lives in the backend; this file renders it.
// D598 round-3 design pass: presentation, window sizing, the autolock 60/0
// semantics, and the erase countdown gate only — the countdown changes WHEN
// the erase commits, never what it erases; every other backend semantic is
// byte-for-byte the NA-0661 behavior.
"use strict";

const tauriInvoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);

// Busy wrapper: the UI reflects core in-flight state (rule d).
let pendingCalls = 0;
function invoke(cmd, args) {
  pendingCalls += 1;
  byId("busy-indicator").classList.remove("hidden");
  return tauriInvoke(cmd, args).finally(() => {
    pendingCalls -= 1;
    if (pendingCalls === 0) byId("busy-indicator").classList.add("hidden");
  });
}

const byId = (id) => document.getElementById(id);
const SCREENS = [
  "scr-wizard-vault", "scr-wizard-identity", "scr-unlock", "scr-erase",
  "scr-wiped", "scr-main", "scr-settings",
];
let currentScreen = null;
// ---- R-14 (NA-0680): content-driven window height -----------------------
// The pre-main surfaces. Everything else is `Full` and is not measured.
const PRE_MAIN_SCREENS = [
  "scr-wizard-vault", "scr-wizard-identity", "scr-unlock", "scr-erase", "scr-wiped",
];

// Measure the ACTIVE pre-main surface: the card's CONTENT height plus the
// screen's own vertical padding.
//
// ⚠ FACT 1 (NA-0680 re-flight) — WHY THE UN-STRETCH IS LOAD-BEARING.
// `.screen` is `position:absolute; inset:0; display:flex` with
// `align-items: stretch`, so the card is STRETCHED to fill the window. A
// stretched box whose content is shorter reports its OWN height from
// `scrollHeight`, not its content's. The measurement was therefore
//
//     measured = (window_height - 56) + 56 = window_height
//
// — the window measuring ITSELF. It could grow (content exceeding the box)
// but could never shrink, which is exactly the "clipped, then over-corrected
// to too-tall, never content-driven in either direction" the acceptance
// flight found. It is also why two different surfaces reported an IDENTICAL
// 388x765: the size was inherited, not computed.
//
// `align-self: flex-start` releases the stretch for the duration of the read,
// so the card collapses to its content and `scrollHeight` reports what we
// actually want. It is restored before returning; nothing observes the
// intermediate state because the read is synchronous.
function measurePreMainHeight() {
  if (!PRE_MAIN_SCREENS.includes(currentScreen)) return null;
  const screen = byId(currentScreen);
  const card = screen && screen.querySelector(".card");
  if (!card) return null;
  const cs = getComputedStyle(screen);
  const pad = parseFloat(cs.paddingTop || 0) + parseFloat(cs.paddingBottom || 0);
  const prevAlignSelf = card.style.alignSelf;
  card.style.alignSelf = "flex-start";
  const content = card.scrollHeight; // forces layout with the stretch released
  card.style.alignSelf = prevAlignSelf;
  return Math.ceil(content + pad);
}

// ⚠ THE ORDERING TRAP THIS EXISTS FOR. Measuring only on a surface change is
// NOT enough: the autolock path calls `show("scr-unlock")` and THEN writes
// "Locked after inactivity." into `#unlock-feedback`. A sync that ran only at
// `show()` would miss the very line that motivated R-14, pass its own test,
// and still clip "Delete vault?" below the fold. So this is called at `show()`
// AND after any write to a conditional element AND on resize.
function syncWindowHeight() {
  if (!currentScreen) return;
  invoke("ui_surface_changed", {
    surface: currentScreen,
    contentHeight: measurePreMainHeight(),
  }).catch(() => {});
}

function show(id) {
  // Item 13 (§5 STATE RULE): every screen transition clears the ceremony
  // and passphrase fields — no typed secret survives a state transition.
  clearCeremonyState();
  for (const s of SCREENS) byId(s).classList.toggle("hidden", s !== id);
  currentScreen = id;
  // Item 15 (R1): the backend disables the state-dependent menu entries
  // (File > Settings / Lock now) unless an unlocked surface is showing.
  // R-14: the measurement rides the same carrier.
  syncWindowHeight();
}

// Item 13 (§5 STATE RULE, F2): ceremony forms never persist. Fields are
// cleared on collapse, cancel, completion, and every state transition; the
// destroy/erase COMPLETION paths additionally perform a full webview
// reload (see their handlers) so nothing typed — and no in-memory value —
// survives into the next session.
function clearCeremonyState() {
  for (const id of [
    "vault-pass", "vault-confirm", "unlock-pass",
    "erase-phrase", "destroy-pass", "destroy-phrase",
  ]) {
    const el = byId(id);
    if (el) el.value = "";
  }
  const flow = byId("destroy-flow");
  if (flow) flow.classList.add("hidden");
  // Item D (round 4a): the ceremony REPLACES its trigger, so collapsing it
  // anywhere must put the trigger back — including on a state transition,
  // not only on Cancel.
  const dopen = byId("btn-destroy-open");
  if (dopen) dopen.classList.remove("hidden");
  const derr = byId("destroy-error");
  if (derr) derr.textContent = "";
  // Item 11b (E.5): a running erase countdown dies on ANY state
  // transition — the armed gate never survives leaving the screen.
  eraseCountdownAbort();
  updateReqs();
}
function resetDestroyFlow() {
  byId("destroy-flow").classList.add("hidden");
  // Item D (round 4a): Cancel restores the trigger the ceremony replaced.
  byId("btn-destroy-open").classList.remove("hidden");
  byId("destroy-pass").value = "";
  byId("destroy-phrase").value = "";
  byId("destroy-error").textContent = "";
}

// ---- the NO-SILENT-STATE-CHANGES rule, ONE implementation ---------------
// Every state-changing control acknowledges in two places: a momentary
// "✓ Saved"-style flash ON the control, and the section's persistent
// status (line or banner) updated to the new reality. The flash is
// presentation; the status is the durable truth. Microcopy stays factual.
function acknowledge(btn, flashText, statusEl, statusText) {
  if (statusEl && statusText !== undefined) statusEl.textContent = statusText;
  const original = btn.textContent;
  btn.textContent = flashText;
  btn.classList.add("acked");
  btn.disabled = true;
  setTimeout(() => {
    btn.textContent = original;
    btn.classList.remove("acked");
    btn.disabled = false;
  }, 1400);
}

// ---- item 12: the status banner component (spec §2) ----------------------
// One helper owns the banner: class, icon, and message swap together.
// Red is RESERVED for the armed-erasure state (R2).
const BANNER_ICONS = {
  danger:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m10.29 3.86-8.2 14.14A2 2 0 0 0 3.82 21h16.36a2 2 0 0 0 1.73-3l-8.2-14.14a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
  accent:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
  neutral:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="m9 12 2 2 4-4"/></svg>',
};
function setBanner(el, kind, message) {
  el.className = "status-banner status-" + kind;
  el.querySelector(".icon").innerHTML = BANNER_ICONS[kind];
  el.querySelector(".status-text").textContent = message;
}

// ---- R-12/F1 (NA-0680): the quiet status LINE ----------------------------
// Persistent state renders as a line, not a banner. Banners stay reserved for
// the Server pane's test RESULTS — an outcome the user just asked for, rather
// than standing state that is simply true.
//
// ⚠ F1 PRECISION, and it is the whole point: `danger` here means danger TEXT.
// Danger CHROME — borders, fills, card backgrounds — remains ABSOLUTE to the
// destroy ceremony. The reservation recorded in DESIGN_SPEC §2 is REFINED by
// the text-vs-chrome distinction, not reversed: the armed-erasure state keeps
// its severity because the words and the colour carry it, and the destroy card
// keeps being the only red box on the surface.
function setStatusLine(el, kind, message) {
  el.className = "status-line-quiet" + (kind === "danger" ? " is-danger" : "");
  el.querySelector(".icon").innerHTML = BANNER_ICONS[kind];
  el.querySelector(".status-text").textContent = message;
}

// ---- item 1 (E.2): visible numeric validation — invalid entries are
// BLOCKED with an inline message + the danger field border; never
// silently clamped, never silently ignored ---------------------------------
function validateNum(inputEl, errEl, min, max, message) {
  const raw = inputEl.value.trim();
  const ok = /^\d+$/.test(raw) && Number(raw) >= min && Number(raw) <= max;
  inputEl.classList.toggle("invalid", !ok);
  if (!ok) {
    errEl.textContent = message;
    return null;
  }
  return Number(raw);
}
for (const id of ["wipe-limit", "autolock-min"]) {
  byId(id).addEventListener("input", () => byId(id).classList.remove("invalid"));
}

// ---- GUI-local non-secret settings (autolock + self-alias) ----------------
let currentSettings = { autolock_minutes: 60, self_alias: "" };
function aliasDisplay() {
  return currentSettings.self_alias.trim() === "" ? "You" : currentSettings.self_alias.trim();
}
async function saveSettings() {
  await invoke("settings_set", {
    autolockMinutes: currentSettings.autolock_minutes,
    selfAlias: currentSettings.self_alias,
  });
}
function adoptSettings(cfg) {
  currentSettings.autolock_minutes = cfg.autolock_minutes;
  currentSettings.self_alias = cfg.self_alias || "";
  autolockMinutes = cfg.autolock_minutes;
}

// ---- failed-attempts capture (binding D596 rule, unchanged) --------------
// A successful unlock RESETS the core counter, so the "since your last
// unlock" value is captured AT UNLOCK-SCREEN ENTRY (and updated from the
// typed outcomes seen in-session) — never read back after unlock. Dismissal
// is app-local acknowledgment only; no core state is touched.
let observedFailedUnlocks = 0;
let vaultAlertCount = 0;
let vaultAlertDismissed = false;

async function showUnlockScreen(next) {
  unlockNext = next;
  try {
    const s = await invoke("protection_status");
    observedFailedUnlocks = s.failed_unlocks;
  } catch (_) { /* fail-quiet: the alert simply stays silent */ }
  show("scr-unlock");
}

// unlock routing: where a successful unlock goes (S1 → wizard identity,
// S2 → main window). The wizard NEVER appears again once identity exists.
let unlockNext = "main";

async function route() {
  const st = await invoke("launch_state");
  if (st === "s0") {
    const cliVault = await invoke("cli_vault_present");
    byId("cli-notice").classList.toggle("hidden", !cliVault);
    show("scr-wizard-vault");
    syncWindowHeight(); // R-14: the notice is a conditional element too
  } else if (st === "s1") {
    await showUnlockScreen("wizard-identity");
  } else {
    await showUnlockScreen("main");
  }
}

// ---- wizard step 1: vault (items 1-3, 10 — spec §3 / D.6) ----------------
// Exactly TWO checks: length and match. No strength meter, no third check.
function updateReqs() {
  const p = byId("vault-pass").value;
  const c = byId("vault-confirm").value;
  const okLen = p.length >= 12;
  const okMatch = p.length > 0 && p === c;
  byId("req-len").classList.toggle("ok", okLen);
  byId("req-match").classList.toggle("ok", okMatch);
  // Create gates on BOTH green (a UI gate; the core contract is unchanged).
  byId("btn-vault-create").disabled = !(okLen && okMatch);
}
byId("vault-pass").addEventListener("input", updateReqs);
byId("vault-confirm").addEventListener("input", updateReqs);

byId("btn-vault-create").addEventListener("click", async () => {
  const pass = byId("vault-pass").value;
  const confirm = byId("vault-confirm").value;
  const err = byId("vault-error");
  err.textContent = "";
  if (!pass) { err.textContent = "Enter a passphrase."; return; }
  if (pass !== confirm) { err.textContent = "Passphrases do not match."; return; }
  const btn = byId("btn-vault-create");
  btn.disabled = true;
  try {
    await invoke("vault_create", { passphrase: pass, confirm });
    // Step 1 is DONE and not revisitable (no false Back — un-creating a
    // vault is not a navigation action). Straight into step 2 (show()
    // clears both passphrase fields on the transition):
    await showIdentityStep();
  } catch (e) {
    err.textContent = mapErr(e, {
      empty_passphrase: "Enter a passphrase.",
      mismatch: "Passphrases do not match.",
      vault_exists: "A vault already exists; restart the app.",
    });
    // R-17: mapErr falls through to the BARE code; never show one.
    if (err.textContent === String(e)) {
      err.textContent = `Your vault couldn't be created. (${String(e)})`;
    }
    btn.disabled = false;
  }
});

// ---- wizard step 2: "Your identity" (items 4-5 — D.7) --------------------
// Item 5 (§4): the verification code renders on ONE line, never wrapping —
// start at the mono token size and shrink to fit. Shared by both surfaces.
function fitCode(el) {
  if (!el) return;
  // Round 4a (F4): re-measure from a clean slate every time — this now runs
  // on resize too, so a code shrunk at one width must be able to grow back.
  el.classList.remove("wrapped");
  el.style.fontSize = "";
  let px = 17;
  while (el.scrollWidth > el.clientWidth && px > 11) {
    px -= 1;
    el.style.fontSize = px + "px";
  }
  // At the floor and STILL overflowing: the old code clipped here, silently,
  // because .verify-code is `overflow: hidden`. Wrap to a second line at a
  // group boundary instead — the operator's ruled preference, and the code
  // stays fully legible at 11px rather than disappearing off the edge.
  if (el.scrollWidth > el.clientWidth) {
    el.classList.add("wrapped");
  }
}
// Round 4a (F4): the window is resizable (no `resizable` key in
// tauri.conf.json, so it defaults true) and BEFORE this lane there was not a
// single resize listener in ui/ — a code fitted at render was never refitted,
// so any drag narrower clipped it. Refit both call sites on resize, debounced
// to one pass per frame-ish so a drag does not thrash the layout.
let codeFitTimer = null;
window.addEventListener("resize", () => {
  if (codeFitTimer) clearTimeout(codeFitTimer);
  codeFitTimer = setTimeout(() => {
    codeFitTimer = null;
    for (const id of ["identity-code", "settings-code"]) {
      const el = byId(id);
      if (el && el.textContent) fitCode(el);
    }
    syncWindowHeight(); // R-14 path (c): re-fit and re-measure together
  }, 60);
});
// R-7 (NA-0680): the name is REQUIRED to leave the wizard. Continue is
// disabled until the field holds a non-empty TRIMMED value — whitespace is not
// a name. No error text: a disabled button beside an empty required field is
// self-explanatory, and an error message for a state the user has not left yet
// is noise.
//
// ONBOARDING-ONLY (F4). Settings keeps accepting an empty name and
// `aliasDisplay()` still falls back to "You", because profiles created before
// this gate existed have one and must not be held hostage by a new rule.
function updateIdentityContinue() {
  const el = byId("alias-input");
  byId("btn-identity-done").disabled = el.value.trim() === "";
}
byId("alias-input").addEventListener("input", updateIdentityContinue);

async function showIdentityStep() {
  const errEl = byId("identity-error");
  errEl.textContent = "";
  try {
    const d = await invoke("identity_ensure");
    byId("identity-code").textContent = d.verify_code;
    byId("identity-purpose").textContent = d.purpose_line;
    byId("identity-pq").textContent = d.pq_line;
    // Item 13 (F2 inventory): a NEW identity starts with an EMPTY alias —
    // the wizard never pre-fills a prior value; Settings is the edit
    // surface (D596 F2).
    byId("alias-input").value = "";
    updateIdentityContinue(); // R-7: an empty field must arrive DISABLED
    show("scr-wizard-identity");
    // ⚠ ORDERING (Findings 1+2): fitCode CHANGES the code's rendered size, so
    // it must run BEFORE the window is measured — `show()` already synced once
    // against the pre-fit size. Re-sync after fitting or the window is sized to
    // a code that is about to shrink.
    fitCode(byId("identity-code"));
    syncWindowHeight();
  } catch (e) {
    errEl.textContent = plainError(e, {}, "Your identity couldn't be set up.");
    updateIdentityContinue();
    show("scr-wizard-identity");
    syncWindowHeight(); // R-14 path (b): the error line is conditional
  }
}

byId("btn-identity-done").addEventListener("click", async () => {
  // The optional self-alias (local-only; empty → "You"). Saving it is part
  // of completing the step; the editable control lives in Settings (F2).
  currentSettings.self_alias = byId("alias-input").value.trim();
  try { await saveSettings(); } catch (_) { /* non-fatal: defaults stand */ }
  enterMain();
});

// ---- unlock ---------------------------------------------------------------
let countdownTimer = null;
function startCountdown(seconds, failed) {
  const fb = byId("unlock-feedback");
  const btn = byId("btn-unlock");
  let left = seconds;
  btn.disabled = true;
  clearInterval(countdownTimer);
  const tick = () => {
    if (left <= 0) {
      clearInterval(countdownTimer);
      // ⚠ R-18 (NA-0680): NULL THE HANDLE. It was cleared but left truthy, and
      // the `finally` below re-enables the button only when
      // `!countdownTimer || className === "feedback"`. After the first
      // countdown of a session the first term was permanently false, so the
      // re-enable depended entirely on a className string — and the catch
      // branch sets "feedback reject", which left Unlock PERMANENTLY DISABLED
      // with a raw error above it. That is the "dead field then a raw lock"
      // the finding describes, reached by a path the finding did not name.
      // `eraseCountdownAbort` nulls its handle correctly; this one did not.
      countdownTimer = null;
      btn.disabled = false;
      fb.className = "feedback";
      fb.textContent = failed > 0 ? `Failed attempts: ${failed}. You can try again now.` : "";
      return;
    }
    fb.className = "feedback reject";
    fb.textContent =
      `Too many failed attempts (${failed}). Try again in ${left} second${left === 1 ? "" : "s"}.`;
    left -= 1;
  };
  tick();
  countdownTimer = setInterval(tick, 1000);
}

byId("btn-unlock").addEventListener("click", async () => {
  const pass = byId("unlock-pass").value;
  const fb = byId("unlock-feedback");
  fb.className = "feedback";
  fb.textContent = "";
  if (!pass) { fb.className = "feedback reject"; fb.textContent = "Enter your passphrase."; return; }
  const btn = byId("btn-unlock");
  btn.disabled = true;
  try {
    const r = await invoke("unlock_attempt", { passphrase: pass });
    if (r.kind === "unlocked") {
      byId("unlock-pass").value = "";
      // The captured pre-unlock count becomes the alert value; the core
      // counter has just reset itself (the binding capture rule).
      vaultAlertCount = observedFailedUnlocks;
      vaultAlertDismissed = false;
      observedFailedUnlocks = 0;
      if (unlockNext === "wizard-identity") await showIdentityStep();
      else enterMain();
    } else if (r.kind === "rejected") {
      observedFailedUnlocks = r.failed_unlocks;
      fb.className = "feedback reject";
      if (r.retry_after_s > 0) startCountdown(r.retry_after_s, r.failed_unlocks);
      else fb.textContent = `Wrong passphrase. Failed attempts: ${r.failed_unlocks}.`;
    } else if (r.kind === "delayed") {
      observedFailedUnlocks = r.failed_unlocks;
      startCountdown(r.retry_after_s, r.failed_unlocks);
    } else if (r.kind === "wiped") {
      show("scr-wiped");
    }
  } catch (e) {
    fb.className = "feedback reject";
    // R-17: a lead SENTENCE, never a bare code.
    fb.textContent = unlockErrorText(e);
  } finally {
    // ⚠ R-18: STATE-DRIVEN, not className-driven. The old predicate asked
    // whether the feedback element's class string happened to equal
    // "feedback", which coupled "is the button usable" to "what does the text
    // look like" — two unrelated things. A countdown is the ONLY reason to
    // leave Unlock disabled, and `countdownTimer` is now nulled when one ends,
    // so it is the whole answer.
    if (countdownTimer === null) btn.disabled = false;
  }
});

byId("link-forgot").addEventListener("click", (ev) => {
  ev.preventDefault();
  byId("erase-error").textContent = "";
  show("scr-erase"); // deliberate step 2 of 2; show() clears the phrase field
});

// ---- erase (app-level file removal ONLY) ---------------------------------
// Item 11b (E.5): the 30-second countdown GATE. It gates WHEN the erase
// commits, never what it erases — erase_all, its phrase check, and its
// scope are byte-untouched; the command is invoked ONLY at countdown zero.
// Cancel, closing the window, or any state transition ABORTS with nothing
// erased.
let eraseCountdownTimer = null;
let eraseCountdownLeft = 0;
function renderEraseCountdown() {
  byId("countdown-number").textContent = String(eraseCountdownLeft);
  byId("countdown-label").textContent = `Erasing in ${eraseCountdownLeft} seconds…`;
}
function eraseCountdownAbort() {
  if (eraseCountdownTimer !== null) {
    clearInterval(eraseCountdownTimer);
    eraseCountdownTimer = null;
  }
  const cd = byId("erase-countdown");
  if (cd) cd.classList.add("hidden");
  const form = byId("erase-form");
  if (form) form.classList.remove("hidden");
  const phrase = byId("erase-phrase");
  if (phrase) phrase.value = "";
  const err = byId("erase-error");
  if (err) err.textContent = "";
}
byId("btn-erase").addEventListener("click", () => {
  const phrase = byId("erase-phrase").value;
  const err = byId("erase-error");
  err.textContent = "";
  if (phrase !== "erase everything") {
    err.textContent = 'Type exactly: erase everything';
    return;
  }
  // The typed phrase already satisfies the landed gate; the form is
  // REPLACED (not merely disabled) by the countdown panel, the field
  // cleared immediately (§5 hygiene). The validated phrase rides the
  // closure to the zero-commit.
  byId("erase-phrase").value = "";
  byId("erase-form").classList.add("hidden");
  byId("erase-countdown").classList.remove("hidden");
  eraseCountdownLeft = 30;
  renderEraseCountdown();
  eraseCountdownTimer = setInterval(async () => {
    eraseCountdownLeft -= 1;
    renderEraseCountdown();
    if (eraseCountdownLeft > 0) return;
    clearInterval(eraseCountdownTimer);
    eraseCountdownTimer = null;
    try {
      await invoke("erase_all", { confirmPhrase: phrase });
      // Item 13 (F2): completion performs a FULL webview state reset — the
      // document reloads, so no typed value and no in-memory state survives
      // into the next session. Boot lands in S0 via route().
      window.location.reload();
    } catch (e) {
      eraseCountdownAbort();
      byId("erase-error").textContent = plainError(e, {
        erase_refused_cli_dir: "Nothing was erased — this app refused to touch the command-line tool's data folder.",
      }, "Nothing was erased.");
    }
  }, 1000);
});
byId("btn-erase-countdown-cancel").addEventListener("click", () => eraseCountdownAbort());
byId("btn-erase-cancel").addEventListener("click", () => showUnlockScreen(unlockNext));
byId("btn-wiped-restart").addEventListener("click", () => route());

// ---- main window ----------------------------------------------------------
async function enterMain() {
  show("scr-main");
  // Slice B (D609 R4): the footer reflects the ACTUAL relay config, not a
  // "future update" claim that is now false. It shows the configured endpoint,
  // or points at Settings when none is set.
  try {
    const cfg = await invoke("relay_config_get");
    byId("status-line").textContent = cfg.relay_url
      ? `Relay: ${cfg.relay_url}`
      : "No server configured — add one in Settings › Server.";
  } catch (_) {
    byId("status-line").textContent = "No server configured — add one in Settings › Server.";
  }
}
byId("btn-add-contact").addEventListener("click", () => {
  byId("stub-note").classList.remove("hidden"); // honest stub (lane 2 lands the flow)
});
byId("btn-rail-contacts").addEventListener("click", () => {
  byId("stub-note").classList.remove("hidden");
});

// ---- settings (item 14: a VIEW in the same shell; the icon rail is live) --
async function openSettings(pane) {
  show("scr-settings");
  selectPane(pane);
  await refreshIdentityPane();
  await refreshVaultPane();
  await refreshServerPane();
  const info = await invoke("app_info");
  byId("about-name").textContent = `${info.display_name} (qsl-desktop ${info.version})`;
  // Slice B (D609 R4): the "no network connections" clause is retired — the app
  // now reaches a relay — but the surviving TRUE clause STAYS: no
  // security-assurance claims. Only the network clause changed.
  byId("about-text").textContent =
    `Slice ${info.slice}. This build makes no security-assurance claims.`;
}
byId("btn-settings").addEventListener("click", () => openSettings("identity"));
byId("btn-rail-chats").addEventListener("click", () => enterMain());
byId("btn-rail-contacts-s").addEventListener("click", () => {
  enterMain();
  byId("stub-note").classList.remove("hidden");
});

function selectPane(name) {
  // Item 13 (§5): pane navigation is a state transition — the ceremony
  // always returns to collapsed and empty.
  resetDestroyFlow();
  for (const b of document.querySelectorAll(".settings-rail .cat[data-pane]")) {
    b.classList.toggle("active", b.dataset.pane === name);
  }
  for (const p of ["identity", "server", "vault", "appearance", "notifications", "about"]) {
    byId("pane-" + p).classList.toggle("hidden", p !== name);
  }
}
for (const b of document.querySelectorAll(".settings-rail .cat[data-pane]")) {
  b.addEventListener("click", () => selectPane(b.dataset.pane));
}

// ---- the Identity pane (existing identity_show surface ONLY) -------------
async function refreshIdentityPane() {
  let rec = null;
  try {
    rec = await invoke("identity_show");
  } catch (_) { /* treated as absent below */ }
  const empty = byId("identity-empty");
  const body = byId("identity-body");
  if (!rec) {
    empty.classList.remove("hidden");
    body.classList.add("hidden");
    return;
  }
  empty.classList.add("hidden");
  body.classList.remove("hidden");
  byId("settings-code").textContent = rec.verify_code;
  byId("settings-fp").textContent = rec.fingerprint;
  // R-4: ONE merged explainer. The pane previously carried the purpose line
  // and the post-quantum line as two separate paragraphs stacked under the
  // code; they answer the same question ("what is this for, and is it
  // strong?") and read better as one.
  byId("settings-explainer").textContent = rec.purpose_line + " " + rec.pq_line;
  byId("settings-mech").textContent = rec.mechanism_line;
  byId("settings-alias").value = currentSettings.self_alias;
  byId("alias-status").textContent = `Shown as: ${aliasDisplay()} (local only)`;
  fitCode(byId("settings-code"));
  syncWindowHeight();
}

byId("btn-alias-save").addEventListener("click", async () => {
  currentSettings.self_alias = byId("settings-alias").value.trim();
  try {
    await saveSettings();
    acknowledge(byId("btn-alias-save"), "✓ Saved", byId("alias-status"),
      `Shown as: ${aliasDisplay()} (local only)`);
  } catch (e) {
    byId("alias-status").textContent = plainError(e, {}, "Your name wasn't saved.");
  }
});

// ---- Vault and Security (items 7-9, 12 — D.5; controls first) ------------
function renderAttemptsAlert() {
  const box = byId("attempts-alert");
  if (vaultAlertCount > 0 && !vaultAlertDismissed) {
    byId("attempts-alert-text").textContent =
      `${vaultAlertCount} failed unlock attempt${vaultAlertCount === 1 ? "" : "s"} since your last unlock`;
    box.classList.remove("hidden");
  } else {
    box.classList.add("hidden");
  }
}
byId("btn-attempts-dismiss").addEventListener("click", () => {
  vaultAlertDismissed = true; // app-local acknowledgment only
  renderAttemptsAlert();
});

// R-11/R-12/R-15 (NA-0680): the erase-after-N state as a quiet status line,
// with the CONTEXTUAL controls and the remaining count.
//
// ⚠ R-15's Phase-0 answer is what makes the counter honest: failed passphrases
// in the DESTROY pane never reach this counter — `unlock_guarded` is the only
// ingress and `destroy_with_passphrase` does not route through it. So the only
// attempts that walk toward erasure are unlock attempts, and "N remaining" is
// exactly `wipe_after - failed_unlocks` on the DTO the pane already fetches.
// No new command, no DTO change, no qsc change.
function remainingBeforeWipe(s) {
  if (s.wipe_after === null) return null;
  return Math.max(0, s.wipe_after - s.failed_unlocks);
}
function renderWipeState(s) {
  const el = byId("wipe-state");
  const armed = s.wipe_after !== null;
  // R-11: Disarm was a DEAD control while off, and Arm is meaningless while
  // armed. Hidden via `.hidden`, never DOM removal, so the tier needles keep
  // reading a tiered button.
  byId("btn-wipe-arm").classList.toggle("hidden", armed);
  byId("btn-wipe-disarm").classList.toggle("hidden", !armed);
  if (!armed) {
    setStatusLine(el, "neutral", "Off — wrong attempts never erase the vault");
    return;
  }
  const left = remainingBeforeWipe(s);
  setStatusLine(el, "danger",
    `Armed — erases after ${s.wipe_after} failed attempt${s.wipe_after === 1 ? "" : "s"}` +
    ` · ${left} remaining`);
}
// Item 2c (E.3): the autolock banner state machine — value > 0 renders the
// accent lock banner; value == 0 renders the DANGER banner (the recorded
// R2 extension: red covers the never-locks state by operator decision).
// R-12 + F1: ONE line, and the two-branch PROPERTY survives the component
// change. `design_round3.rs`'s state machine still holds — the 0 state carries
// danger treatment, the >0 state does not — but the treatment is now danger
// TEXT on a status line rather than a filled red banner. The test's assertion
// MECHANISM moves with it; the property it encodes does not change, which is
// why that needle is amended rather than deleted.
function renderAutolockState(minutes) {
  const el = byId("autolock-status");
  if (minutes === 0) {
    setStatusLine(el, "danger",
      "Never locks — anyone with access to this device can open your vault");
  } else {
    setStatusLine(el, "neutral",
      `Locks after ${minutes} minute${minutes === 1 ? "" : "s"} of inactivity. 0 = never.`);
  }
}
async function refreshVaultPane() {
  renderAttemptsAlert();
  const s = await invoke("protection_status");
  renderWipeState(s);
  byId("wipe-limit").min = s.wipe_min;
  byId("wipe-limit").max = s.wipe_max;
  const cfg = await invoke("settings_get");
  adoptSettings(cfg);
  byId("autolock-min").value = cfg.autolock_minutes;
  renderAutolockState(cfg.autolock_minutes);
}

byId("btn-wipe-arm").addEventListener("click", async () => {
  const err = byId("wipe-error");
  err.textContent = "";
  const limit = validateNum(byId("wipe-limit"), err, 1, 100,
    "Enter a whole number from 1 to 100.");
  if (limit === null) return;
  if (!byId("wipe-ack").checked) {
    err.textContent = "Tick the confirmation first — arming can permanently erase the vault.";
    return;
  }
  try {
    await invoke("wipe_arm", { limit });
    byId("wipe-ack").checked = false;
    const s = await invoke("protection_status");
    renderWipeState(s);
    acknowledge(byId("btn-wipe-arm"), "✓ Armed");
  } catch (e) {
    err.textContent = mapErr(e, { wipe_limit_out_of_bounds: "Limit must be between 1 and 100." });
  }
});
byId("btn-wipe-disarm").addEventListener("click", async () => {
  byId("wipe-error").textContent = "";
  try {
    await invoke("wipe_disarm");
    const s = await invoke("protection_status");
    renderWipeState(s);
    acknowledge(byId("btn-wipe-disarm"), "✓ Off");
  } catch (e) {
    byId("wipe-error").textContent = plainError(e, {}, "That setting wasn't changed.");
  }
});

byId("btn-autolock-save").addEventListener("click", async () => {
  const err = byId("autolock-error");
  err.textContent = "";
  // Item 1/2 (E.2/E.3, F2): the 0-1440 range is UI-side visible
  // validation; 0 is valid and means never-auto-lock.
  const minutes = validateNum(byId("autolock-min"), err, 0, 1440,
    "Enter a whole number from 0 to 1440.");
  if (minutes === null) return;
  try {
    currentSettings.autolock_minutes = minutes;
    await saveSettings();
    autolockMinutes = minutes;
    renderAutolockState(minutes);
    acknowledge(byId("btn-autolock-save"), "✓ Saved");
  } catch (e) {
    err.textContent = plainError(e, {}, "The autolock setting wasn't saved.");
  }
});

// Item 6 (§5) + item 13: the destroy ceremony — always opens collapsed and
// empty; semantics unchanged (typed phrase + passphrase → the tokened core
// destroy).
byId("btn-destroy-open").addEventListener("click", () => {
  byId("destroy-flow").classList.remove("hidden");
  // Item D (round 4a): the ceremony REPLACES its trigger rather than sitting
  // below it — one destructive affordance on screen at a time. Cancel (and
  // any state transition, via clearCeremonyState) puts it back. Behavior
  // only: the passphrase + typed-phrase gates and the tokened core call are
  // byte-untouched below.
  byId("btn-destroy-open").classList.add("hidden");
  byId("destroy-pass").value = "";
  byId("destroy-phrase").value = "";
  byId("destroy-error").textContent = "";
});
byId("btn-destroy-cancel").addEventListener("click", () => resetDestroyFlow());
byId("btn-destroy").addEventListener("click", async () => {
  const err = byId("destroy-error");
  err.textContent = "";
  const phrase = byId("destroy-phrase").value;
  if (phrase !== "destroy my vault") {
    err.textContent = 'Type exactly: destroy my vault';
    return;
  }
  try {
    await invoke("destroy_vault", {
      passphrase: byId("destroy-pass").value,
      confirmPhrase: phrase,
    });
    // Item 13 (F2): the vault is gone — completion performs a FULL webview
    // state reset. The reloaded document boots into S0; the typed
    // passphrase, the phrase, the ceremony expansion, and every in-memory
    // value (alias, alert counters) die with this document.
    window.location.reload();
  } catch (e) {
    err.textContent = destroyErrorText(e); // R-17: `vault_locked` here = wrong passphrase
  }
});

// ---- the Server pane (slice B, REDESIGNED by NA-0674 / D610) -------------
// R1 (D609, still binding): every relay_* command runs qsc through the backend
// serial blocking gate; this file NEVER constructs an HTTP client and NEVER
// classifies a probe — it renders the pre-classified outcome the backend
// returns. This lane adds no call site; it re-orders existing ones.
//
// R7 COLOUR: the results reuse the shipped status-banner component with only
// `neutral` (connected / in flight) and `accent` (needs attention). RED
// (status-danger) is RESERVED for the vault-danger surfaces (DESIGN_SPEC §2),
// so a connection FAILURE is accent, not red — the message text carries the
// severity. The mockup's red "bad" treatment is deliberately NOT copied.
//
// THE COMMIT MODEL (R-A1..R-A4) REVERSES [F.1-COMMIT], which shipped one lane
// ago. The old split model gave the token and the CA their own Set/Clear
// buttons and let Save persist only the URL. That created a trap: type a new
// token, press Test, and the probe read the OLD token out of the vault,
// because the typed one had never been committed. The pane reported, truly,
// the result for a configuration the user believed they had replaced.
// Test-saves-first removes the trap rather than warning about it. "Secrets to
// the vault, URL to settings" is UNCHANGED — only the commit surface unifies.
let savedRelayUrl = "";
let tokenConfigured = false;
let caConfigured = false;
let caPathHash = "";
let tokenPendingRemoval = false;
let caPendingRemoval = false;
let serverBusy = false;

function clearServerResults() {
  byId("relay-results").classList.add("hidden");
  byId("relay-detail").textContent = "";
  byId("relay-doc").innerHTML = "";
}

// R-E5: dirty = any field differs from stored state, or a removal is pending.
// A blank token field is NOT dirty — blank means "keep what's there" (R-B3).
function urlDirty() {
  const v = byId("relay-url").value.trim();
  return v !== "" && v !== savedRelayUrl;
}
function tokenDirty() { return byId("relay-token").value !== "" || tokenPendingRemoval; }
function caDirty() { return byId("relay-ca-path").value.trim() !== "" || caPendingRemoval; }
function serverDirty() { return urlDirty() || tokenDirty() || caDirty(); }

// ⚠ ORDERING: after a commit this MUST run AFTER the R-B5 echo has written the
// normalized URL back into the field — never before it. Found by the NA-0674
// acceptance flight; see D-0011.
//
// `refreshServerState()` calls this internally, and the commit handlers used to
// rely on that call alone. But it fires while the input still holds the user's
// RAW text, and `savedRelayUrl` has by then been updated to the NORMALIZED
// form. When normalization changes the string — IPv4 shorthand (`https://192`
// -> `https://0.0.0.192`), a trailing slash, an uppercase host, a redundant
// default port — the two differ, the pane reads as dirty, and the helper says
// "Settings changed — not saved." about settings that WERE just saved. The
// echo then corrected the field but nothing re-evaluated the helper, so the
// false claim stood until the next keystroke.
//
// Type an address in its already-canonical form and the bug is invisible,
// which is exactly why 70 passing tests missed it and a human typing a
// shorthand IP found it in minutes.
function renderDirty() {
  byId("relay-dirty").classList.toggle("hidden", !serverDirty());
}

// F1R (operator-ruled): ANY change to what the app will use clears the results
// — a field edit, a "remove it" click, or cancelling a pending removal by
// typing. Results describing a configuration the user has already changed are
// stale-but-technically-true, which is the kind that gets misread; the dirty
// helper explains why they vanished.
function onServerChanged() {
  clearServerResults();
  renderDirty();
}

// R-A1: "remove it" replaces the per-field Clear BUTTONS. A prose link inside
// the line that describes the field's state cannot be confused with its
// neighbour the way two adjacent buttons both labelled "Clear" were
// (ENG-0073, surfaced by the NA-0673 acceptance flight and superseded here).
function removalLink(id, onPick) {
  const a = document.createElement("a");
  a.className = "rm";
  a.id = id;
  a.textContent = "remove it";
  a.setAttribute("role", "button");
  a.tabIndex = 0;
  a.addEventListener("click", onPick);
  a.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onPick(); }
  });
  return a;
}

// R-E1/E2/E3: ONE helper line that SWAPS by state — the two texts never stack.
function renderTokenHelp() {
  const p = byId("relay-token-help");
  const input = byId("relay-token");
  p.textContent = "";
  if (tokenPendingRemoval) {
    input.placeholder = "";
    p.textContent = "Token will be removed when you save or test.";
    return;
  }
  if (tokenConfigured) {
    // R-E1: FIXED eight dots, always. relay_token_show is a bare bool by
    // design — the app cannot know the token's length, and must not appear to.
    input.placeholder = "••••••••";
    p.appendChild(document.createTextNode("A token is set — leave blank to keep it, or "));
    p.appendChild(removalLink("relay-token-remove", () => {
      tokenPendingRemoval = true;
      renderTokenHelp();
      onServerChanged();
    }));
    p.appendChild(document.createTextNode("."));
    return;
  }
  input.placeholder = "";
  p.textContent = "Required only if the operator set one. Stored in your vault, not in settings.";
}

// R-E4: same link pattern, wording adapted. C3: the PATH is never shown —
// relay_ca_file_show() returns {configured, path_hash} and the path itself is
// deliberately not retrievable, so this line is the only place the stored
// state appears.
function renderCaStatus() {
  const p = byId("relay-ca-status");
  p.textContent = "";
  if (caPendingRemoval) {
    p.textContent = "Certificate authority file will be removed when you save or test.";
    return;
  }
  if (caConfigured) {
    p.appendChild(document.createTextNode("CA certificate file set (" + (caPathHash || "configured") + ") — "));
    p.appendChild(removalLink("relay-ca-remove", () => {
      caPendingRemoval = true;
      renderCaStatus();
      onServerChanged();
    }));
    p.appendChild(document.createTextNode("."));
    return;
  }
  p.textContent = "No CA file set — the app uses your computer's trusted certificates.";
}

// R-C1: no re-entry while a commit or probe is in flight; both buttons
// disabled; the results area shows a neutral, accent-free in-flight line.
//
// The label is the caller's because Save does not probe. D610 R-C1 wrote
// "Testing…" for both paths; showing "Testing…" while a Save runs would state
// something untrue about what the app is doing, which is exactly the class of
// claim this project sweeps for. The MECHANISM R-C1 specifies (both buttons
// disabled, neutral treatment, no re-entry) is implemented exactly; only the
// Save path's four-word label differs, and it differs so it can be accurate.
function setServerBusy(on, label) {
  serverBusy = on;
  byId("btn-relay-test").disabled = on;
  byId("btn-relay-save").disabled = on;
  if (!on) return;
  byId("relay-results").classList.remove("hidden");
  byId("relay-doc").innerHTML = "";
  byId("relay-detail").textContent = "";
  setBanner(byId("relay-status"), "neutral", label);
}

// ⚠ THE COMMIT ORDER — R-B1 AMENDED BY DIRECTOR RULING 2026-07-25.
//
// D610's C2 ruled the order "validate the URL → token → CA → settings.json
// LAST", on the premise that the URL (unlike the CA path) could be validated
// WITHOUT writing. That premise is FALSE, and it was verified false here: the
// crate exposes nine relay commands and NONE is validate-only.
// `relay_config_set` runs `normalize_relay_endpoint` and then writes
// settings.json in the same call, exactly as `relay_ca_file_set` validates by
// writing. Neither field can be checked without committing it.
//
// That put two rulings in direct conflict:
//   R-B1 wants vault writes first and settings.json LAST.
//   R-B2 wants a malformed address to block the ENTIRE commit with NOTHING
//        persisted — "on Save AND on Test."
// Honouring R-B1's order means the vault writes land and THEN the bad address
// is rejected, so something persisted: R-B2 broken. Honouring R-B2 means the
// address must be validated before any vault write, and since validating IS
// writing, the address commits first: R-B1's ordering inverted.
//
// RULED: R-B2's guarantee GOVERNS; R-B1's vault-first ordering is AMENDED to
// address-first. An absolute stated guarantee outranks unexplained write
// ordering — R-B2 says what the user is promised and can observe, while
// R-B1's ordering was a preference with no stated purpose, and R-B1 already
// conceded the commit is not atomic.
//
// THE ACCEPTED COST: if a vault write fails, the address is ALREADY saved.
// That is acceptable because state 14 reports it HONESTLY (it names the failed
// part and says the probe did not run) and because A RE-TEST HEALS IT — fix
// the failing field, press Test again, and the commit completes from where it
// stopped. A partial commit the user can see and finish is not the defect a
// partial commit that hides would be. See D-0010 and Appendix F.
//
// Returns null on success, or {part, message, inline} on the first failure —
// at which point the REMAINDER IS ABANDONED (R-B1) and the probe never runs.
async function commitServerSettings() {
  // (1) THE ADDRESS. Validated-and-persisted first so a malformed one blocks
  //     everything with nothing written (R-B2). Skipped when the field is
  //     empty or unchanged — an untouched field is not a validation failure.
  if (urlDirty()) {
    try {
      await invoke("relay_config_set", { url: byId("relay-url").value.trim() });
    } catch (e) {
      const code = String(e);
      if (RELAY_ENDPOINT_CODES.some((c) => code.includes(c))) {
        // State 11: INLINE field validation, never a results card — no probe
        // was attempted, so a card would assert something untrue.
        byId("relay-url").classList.add("invalid");
        byId("relay-url-error").textContent = "Enter a valid relay address, e.g. https://relay.example.net";
        return { part: "settings", message: "", inline: true };
      }
      return {
        part: "settings",
        message: "The relay address couldn't be saved to your settings (" + code + "). Nothing else was saved, and no connection test was run.",
        inline: false,
      };
    }
  }

  // (2) THE TOKEN — to the vault, through the qsc trio. R-B3: blank keeps,
  //     typed replaces, pending-removal deletes.
  if (tokenPendingRemoval) {
    try {
      await invoke("relay_token_clear");
    } catch (e) {
      return { part: "vault", message: "The access token couldn't be removed from your vault (" + String(e) + "). The relay address was saved; nothing after this was. No connection test was run.", inline: false };
    }
  } else if (byId("relay-token").value !== "") {
    try {
      await invoke("relay_token_set", { token: byId("relay-token").value });
    } catch (e) {
      // A FRIENDLY SENTENCE LEADS; the raw code survives in parentheses at the
      // end, matching the other four commit-failure messages. `mapErr` falls
      // through to the BARE error code when it has no mapping, and that was
      // being concatenated straight onto prose — so state 14 opened with a
      // naked `vault_write_failed`. State 14 is the one message a user only
      // ever reaches after something has already gone wrong, which is the
      // worst possible moment to show them an internal identifier where a
      // sentence belongs. Found by the NA-0674 flight; see D-0011.
      const code = String(e);
      const lead = code.includes("relay_token_missing")
        ? "Enter a token first."
        : "The access token couldn't be saved to your vault (" + code + ").";
      return { part: "vault", message: lead + " Nothing after it was saved, and no connection test was run.", inline: false };
    }
  }

  // (3) THE CA FILE — to the vault, through the qsc trio. C2: there is no
  //     validate-only call; relay_ca_file_set VALIDATES BY WRITING, and its
  //     error codes ARE the validation.
  if (caPendingRemoval) {
    try {
      await invoke("relay_ca_file_clear");
    } catch (e) {
      return { part: "vault", message: "The certificate authority file couldn't be removed from your vault (" + String(e) + "). Everything before it was saved. No connection test was run.", inline: false };
    }
  } else if (byId("relay-ca-path").value.trim() !== "") {
    try {
      await invoke("relay_ca_file_set", { path: byId("relay-ca-path").value.trim() });
    } catch (e) {
      byId("relay-ca-error").textContent = mapErr(e, {
        relay_ca_file_missing: "No file at that path.",
        relay_ca_file_unreadable: "That file can't be read.",
        relay_ca_file_invalid: "That doesn't look like a certificate file.",
      });
      return { part: "vault", message: "The certificate authority file couldn't be saved to your vault. Check the path under “Certificate authority” above. Everything before it was saved, and no connection test was run.", inline: false };
    }
  }
  return null;
}

// Both buttons take the SAME path out of a failed commit, and the two branches
// are not symmetric. Found by the NA-0674 acceptance flight; see D-0011.
//
// ⚠ THE INLINE BRANCH MUST NOT AWAIT ANYTHING BEFORE CLEARING THE PANEL.
// This originally read `await refreshServerState()` FIRST, then cleared. Two
// mistakes in one line:
//
//   1. `refreshServerState()` reaches `relay_token_show` / `relay_ca_file_show`,
//      and BOTH run through the process-wide SERIAL blocking gate. A probe
//      still in flight against a dead address holds that gate for the whole TCP
//      timeout, so the await parked, the clear never ran, and the panel sat
//      showing a stale "Testing…" banner UNDERNEATH the new inline error —
//      telling the user a test was running when none had been attempted.
//   2. The re-read does not belong on this branch AT ALL. C2(b) requires
//      re-reading live state after a PARTIAL commit, because something landed.
//      R-B2 guarantees a validation failure persists NOTHING, so there is
//      nothing to re-read. Applying the obligation to a branch it does not
//      cover is what put a gated call in the way.
//
// The partial-commit branch is the opposite: it renders FIRST (so the failure
// is on screen immediately) and re-reads AFTER, because there the state really
// did change under the pane.
function handleFailedCommit(fail) {
  if (fail.inline) {
    // State 11 already rendered inline, under the field. Nothing persisted.
    clearServerResults();
    renderDirty();
    return;
  }
  renderCommitFailure(fail);
  // C2(b): a partial commit DID land — the helper lines must stop describing
  // state it already changed. Safe to await here: the commit has finished, so
  // the gate is free.
  void refreshServerState();
}

// NEW state 14 (R-F2): a commit that failed. Accent, sibling of state 13,
// names WHICH part failed, and says plainly that the probe did not run.
function renderCommitFailure(fail) {
  byId("relay-results").classList.remove("hidden");
  byId("relay-doc").innerHTML = "";
  setBanner(byId("relay-status"), "accent", "Couldn't save settings");
  byId("relay-detail").textContent = fail.message;
}

function humanBytes(n) {
  if (!n) return "—";
  if (n >= 1048576) { const v = n / 1048576; return (Number.isInteger(v) ? v : v.toFixed(1)) + " MB"; }
  if (n >= 1024) return Math.round(n / 1024) + " KB";
  return n + " bytes";
}
function humanDuration(secs) {
  if (!secs) return "—";
  if (secs >= 86400) { const d = Math.round(secs / 86400); return d + " day" + (d === 1 ? "" : "s"); }
  if (secs >= 3600) { const h = Math.round(secs / 3600); return h + " hour" + (h === 1 ? "" : "s"); }
  const m = Math.round(secs / 60); return m + " minute" + (m === 1 ? "" : "s");
}
function docRow(label, value) {
  const row = document.createElement("div");
  row.style.cssText = "display:flex;gap:var(--sp-3);";
  const l = document.createElement("span");
  l.className = "hint"; l.style.cssText = "min-width:130px;color:var(--fg-muted);";
  l.textContent = label;
  const v = document.createElement("span");
  v.className = "hint"; v.style.color = "var(--fg-secondary)";
  v.textContent = value;
  row.appendChild(l); row.appendChild(v);
  return row;
}

// Results states 1-7 — the probe outcome (already classified by qsc).
// R-F4: trigger and wording are UNCHANGED from the shipped pane. This lane
// redesigns the commit surface and the layout, not the result copy.
function renderServerOutcome(res, committed) {
  const status = byId("relay-status");
  const detail = byId("relay-detail");
  const doc = byId("relay-doc");
  byId("relay-results").classList.remove("hidden");
  detail.textContent = "";
  doc.innerHTML = "";
  switch (res.kind) {
    case "reachable": {
      setBanner(status, "neutral", "Connected");
      const bearer = res.auth_mode === "bearer";
      detail.textContent = bearer
        ? "Token required — accepted. Certificate trusted."
        : "Open relay — anyone who can reach this address can use it. No access token needed. You still need an invite from someone to exchange messages.";
      const d = res.doc;
      if (d.name) doc.appendChild(docRow("Relay name", d.name));
      doc.appendChild(docRow("Certificate", "Trusted"));
      doc.appendChild(docRow("Access", bearer ? "Token required — accepted" : "Open — no token needed"));
      if (d.retention_ttl_secs) doc.appendChild(docRow("Message retention", humanDuration(d.retention_ttl_secs)));
      if (d.max_body_bytes) doc.appendChild(docRow("Max message size", humanBytes(d.max_body_bytes)));
      if (d.version) doc.appendChild(docRow("Server version", d.version));
      break;
    }
    case "auth_required":
      // The relay's 401 is byte-identical for both; the CLIENT distinguishes by
      // whether IT sent a token — a LOCAL observation, never a server verdict.
      setBanner(status, "accent", res.token_was_sent ? "Token rejected" : "This relay requires an access token");
      detail.textContent = res.token_was_sent
        ? "The relay requires an access token and didn't accept the one this app sent. Check it with the operator."
        : "This app sent no token, and the relay requires one. Ask the operator for one, set it above, and test again.";
      break;
    case "cert_not_trusted":
      setBanner(status, "accent", "Certificate not trusted");
      detail.textContent =
        "This server presented a certificate your computer doesn't recognise. That's expected if the operator runs their own certificate authority — and it's also what an interception attack looks like. Ask the operator for their CA certificate and add it above, or install it on this computer.";
      break;
    case "unreachable":
      setBanner(status, "accent", "Couldn't reach the server");
      detail.textContent =
        "Nothing answered at that address. Check the address, and check you're on the same network or VPN as the relay.";
      break;
    case "not_a_qsl_relay":
      setBanner(status, "accent", "Not a QSL relay");
      detail.textContent = "Something answered, but it isn't a QSL relay. Check the address.";
      break;
  }
  // C6 (R-E6, promoted from "may" to "does"): a Test that COMMITTED says so.
  // Under Test-saves-first the commit is otherwise silent, and the dirty
  // helper merely disappearing is absence-of-signal, not confirmation. Shown
  // only when something was actually committed.
  if (committed) {
    detail.textContent = (detail.textContent ? detail.textContent + " " : "") + "Settings saved.";
  }
}

// R2 — the probe's Err channel: LOCAL config problems, no request was formed.
// (11) malformed address -> INLINE field validation, never a card.
// (12) unreadable configured CA -> its OWN line, EXPLICITLY NOT cert-not-trusted
//      (that means TLS refused a READABLE cert; this is a local file problem).
// (13) client build failure / other -> a generic line.
const RELAY_ENDPOINT_CODES = ["relay_endpoint_missing", "relay_endpoint_invalid_host", "relay_endpoint_invalid_scheme", "relay_endpoint_invalid"];
const RELAY_CA_CODES = ["relay_ca_file_missing", "relay_ca_file_unreadable", "relay_ca_file_invalid"];
function renderServerError(codeStr) {
  if (RELAY_ENDPOINT_CODES.some((c) => codeStr.includes(c))) {
    byId("relay-url").classList.add("invalid");
    byId("relay-url-error").textContent = "Enter a valid relay address, e.g. https://relay.example.net";
    clearServerResults();
    return;
  }
  byId("relay-results").classList.remove("hidden");
  byId("relay-doc").innerHTML = "";
  if (RELAY_CA_CODES.some((c) => codeStr.includes(c))) {
    setBanner(byId("relay-status"), "accent", "Certificate authority file couldn't be read");
    byId("relay-detail").textContent =
      "The certificate authority file you configured couldn't be read. Check the path under “Certificate authority” above — this is a local file problem, not a problem with the server's certificate.";
  } else {
    setBanner(byId("relay-status"), "accent", "Couldn't start the connection test");
    byId("relay-detail").textContent = "The connection test couldn't be started (" + codeStr + ").";
  }
}

// Re-read the STORED state and re-render everything that describes it.
// C2(b): this runs after ANY failed commit too, so a partial commit can never
// leave the helper lines describing state the commit already changed.
async function refreshServerState() {
  try {
    const cfg = await invoke("relay_config_get");
    savedRelayUrl = cfg.relay_url || "";
  } catch (_) { savedRelayUrl = ""; }
  try {
    const s = await invoke("relay_token_show");
    tokenConfigured = !!s.configured;
    if (!tokenConfigured) tokenPendingRemoval = false;
  } catch (_) { tokenConfigured = false; }
  try {
    const s = await invoke("relay_ca_file_show");
    caConfigured = !!s.configured;
    caPathHash = s.path_hash || "";
    if (!caConfigured) caPendingRemoval = false;
  } catch (_) { caConfigured = false; caPathHash = ""; }
  renderTokenHelp();
  renderCaStatus();
  renderDirty();
}

// State 9 (idle): the results panel is hidden on pane open. R-B6: a dirty pane
// left by navigating away is discarded silently — an unsaved relay address is
// inconvenience-class loss, and severity discipline gives that a helper, not a
// modal.
async function refreshServerPane() {
  tokenPendingRemoval = false;
  caPendingRemoval = false;
  byId("relay-token").value = "";
  byId("relay-ca-path").value = "";
  byId("relay-url-error").textContent = "";
  byId("relay-ca-error").textContent = "";
  byId("relay-url").classList.remove("invalid");
  clearServerResults();
  await refreshServerState();
  byId("relay-url").value = savedRelayUrl;
  renderDirty();
}

for (const fid of ["relay-url", "relay-token", "relay-ca-path"]) {
  byId(fid).addEventListener("input", () => {
    if (fid === "relay-url") {
      byId("relay-url").classList.remove("invalid");
      byId("relay-url-error").textContent = "";
    }
    if (fid === "relay-ca-path") {
      byId("relay-ca-error").textContent = "";
      // R-B3/R-E3: typing CANCELS a pending removal and reverts the line.
      if (caPendingRemoval && byId("relay-ca-path").value !== "") {
        caPendingRemoval = false;
        renderCaStatus();
      }
    }
    if (fid === "relay-token" && tokenPendingRemoval && byId("relay-token").value !== "") {
      tokenPendingRemoval = false;
      renderTokenHelp();
    }
    onServerChanged();
  });
}

// R-A2: TEST SAVES FIRST. On a dirty pane Test commits everything, then probes
// the JUST-SAVED state — so the result always describes what the app will
// actually use. A clean pane commits nothing and probes what is stored.
byId("btn-relay-test").addEventListener("click", async () => {
  if (serverBusy) return;
  byId("relay-url-error").textContent = "";
  byId("relay-url").classList.remove("invalid");
  byId("relay-ca-error").textContent = "";
  setServerBusy(true, "Testing…");
  try {
    let committed = false;
    if (serverDirty()) {
      const fail = await commitServerSettings();
      if (fail) {
        handleFailedCommit(fail);
        return; // R-B2/R-B1: the probe does NOT run on a failed commit.
      }
      committed = true;
      byId("relay-token").value = "";
      byId("relay-ca-path").value = "";
      await refreshServerState();
      byId("relay-url").value = savedRelayUrl; // R-B5: the NORMALIZED form.
      renderDirty(); // MUST follow the echo — see renderDirty()'s ORDERING note.
    }
    renderServerOutcome(await invoke("relay_test", { url: savedRelayUrl }), committed);
  } catch (e) {
    // R-B4: a failed probe does NOT roll the commit back. The commit outcome
    // and the probe outcome are independent verdicts.
    renderServerError(String(e));
  } finally {
    setServerBusy(false);
  }
});

// R-A3: Save stays independently clickable — commit without probe, for the
// configure-now-connect-later case.
byId("btn-relay-save").addEventListener("click", async () => {
  if (serverBusy) return;
  byId("relay-url-error").textContent = "";
  byId("relay-url").classList.remove("invalid");
  byId("relay-ca-error").textContent = "";
  setServerBusy(true, "Saving…");
  try {
    const fail = await commitServerSettings();
    if (fail) {
      handleFailedCommit(fail);
      return;
    }
    byId("relay-token").value = "";
    byId("relay-ca-path").value = "";
    await refreshServerState();
    byId("relay-url").value = savedRelayUrl; // R-B5
    renderDirty(); // MUST follow the echo — see renderDirty()'s ORDERING note.
    clearServerResults();
    acknowledge(byId("btn-relay-save"), "✓ Saved");
  } finally {
    setServerBusy(false);
  }
});

// ---- idle autolock (ON by default at 60 min, adjustable; 0 = NEVER
// auto-lock; wizard exempt) -------------------------------------------------
let autolockMinutes = 60;
let idleSince = Date.now();
for (const ev of ["mousemove", "mousedown", "keydown", "wheel", "touchstart"]) {
  window.addEventListener(ev, () => { idleSince = Date.now(); }, { passive: true });
}
setInterval(async () => {
  const onLockedSurface = currentScreen === "scr-main" || currentScreen === "scr-settings";
  if (!onLockedSurface) return; // the wizard (and unlock itself) is exempt
  // Item 2b (E.3, BINDING encoded rule): at 0 the timer must NEVER fire —
  // without this guard the elapsed-time comparison below is satisfied
  // immediately and the vault would lock the moment it unlocked.
  if (autolockMinutes === 0) return;
  if (Date.now() - idleSince >= autolockMinutes * 60 * 1000) {
    idleSince = Date.now();
    await invoke("lock_now"); // the one-call NA-0658 lock()
    await showUnlockScreen("main");
    // R-14: accent severity, not red-adjacent — being locked by the timer is
    // the protection working, not a failure.
    byId("unlock-feedback").className = "feedback locked-notice";
    byId("unlock-feedback").textContent = "Locked after inactivity.";
    // ⚠ R-14 PATH (b): this write happens AFTER show(), so the surface-change
    // sync already ran against an EMPTY feedback line. Without this call the
    // window keeps the table height and "Delete vault?" clips — the exact
    // reported defect.
    syncWindowHeight();
  }
}, 5000);

// ---- item 15: native menu events (R1: backend gates the entries) ---------
if (window.__TAURI__.event && window.__TAURI__.event.listen) {
  window.__TAURI__.event.listen("menu-open-settings", () => {
    if (currentScreen === "scr-main" || currentScreen === "scr-settings") {
      openSettings("identity");
    }
  });
  window.__TAURI__.event.listen("menu-lock-now", async () => {
    if (currentScreen === "scr-main" || currentScreen === "scr-settings") {
      await invoke("lock_now");
      await showUnlockScreen("main");
    }
  });
}

// ---- R-17 (NA-0680): plain language, mapped BY SITE ----------------------
// ⚠ THE RULING THAT MATTERS: the same code means different things at
// different call sites, so a single code->text table would ship a false
// sentence. `vault_locked` is the proof. In the DESTROY pane it means WRONG
// PASSPHRASE — Settings is unlock-gated, so the vault is demonstrably
// unlocked and "your vault is locked, unlock it first" would be untrue at the
// one site the finding named as its example. Elsewhere the same code really
// does mean locked. Hence: per-site wordings, and `mapErr` never returns a
// bare code again.
//
// The generic fall-through is the mechanism behind NA-0674's naked
// `vault_write_failed`: `mapErr` returned the raw code when nothing matched,
// and callers concatenated it straight onto prose. Now a lead sentence always
// arrives first and the code survives only in parentheses, for a bug report.
function plainError(e, table, lead) {
  const s = String(e);
  for (const k of Object.keys(table)) if (s.includes(k)) return table[k];
  return `${lead} (${s})`;
}

function unlockErrorText(e) {
  return plainError(e, {
    vault_attempt_limit_io:
      "This vault's protection settings couldn't be read, so the unlock was refused rather than attempted. Check the app's data folder is readable.",
  }, "Your vault couldn't be unlocked.");
}

// The DESTROY pane. `vault_locked` here is a WRONG PASSPHRASE — see above.
function destroyErrorText(e) {
  return plainError(e, {
    vault_locked: "That passphrase doesn't match. Nothing was destroyed.",
    confirm_phrase_mismatch: "The confirmation phrase doesn't match. Nothing was destroyed.",
    vault_erase_failed:
      "The vault file couldn't be erased. Nothing was destroyed — check the app's data folder is writable.",
  }, "Your vault wasn't destroyed.");
}

function mapErr(e, table) {
  const s = String(e);
  for (const k of Object.keys(table)) if (s.includes(k)) return table[k];
  return s;
}

// ---- boot -----------------------------------------------------------------
(async () => {
  try {
    const cfg = await invoke("settings_get");
    adoptSettings(cfg);
  } catch (_) { /* defaults stand */ }
  await route();
})();

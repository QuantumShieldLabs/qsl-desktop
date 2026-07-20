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
function show(id) {
  // Item 13 (§5 STATE RULE): every screen transition clears the ceremony
  // and passphrase fields — no typed secret survives a state transition.
  clearCeremonyState();
  for (const s of SCREENS) byId(s).classList.toggle("hidden", s !== id);
  currentScreen = id;
  // Item 15 (R1): the backend disables the state-dependent menu entries
  // (File > Settings / Lock now) unless an unlocked surface is showing.
  invoke("ui_surface_changed", { surface: id }).catch(() => {});
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
  const derr = byId("destroy-error");
  if (derr) derr.textContent = "";
  // Item 11b (E.5): a running erase countdown dies on ANY state
  // transition — the armed gate never survives leaving the screen.
  eraseCountdownAbort();
  updateReqs();
}
function resetDestroyFlow() {
  byId("destroy-flow").classList.add("hidden");
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
    btn.disabled = false;
  }
});

// ---- wizard step 2: "Your identity" (items 4-5 — D.7) --------------------
// Item 5 (§4): the verification code renders on ONE line, never wrapping —
// start at the mono token size and shrink to fit. Shared by both surfaces.
function fitCode(el) {
  el.style.fontSize = "";
  let px = 17;
  while (el.scrollWidth > el.clientWidth && px > 11) {
    px -= 1;
    el.style.fontSize = px + "px";
  }
}
async function showIdentityStep() {
  const errEl = byId("identity-error");
  errEl.textContent = "";
  try {
    const d = await invoke("identity_ensure");
    byId("identity-code").textContent = d.verify_code;
    byId("identity-fp").textContent = d.fingerprint;
    byId("identity-purpose").textContent = d.purpose_line;
    byId("identity-pq").textContent = d.pq_line;
    byId("identity-mech").textContent = d.mechanism_line;
    // Item 13 (F2 inventory): a NEW identity starts with an EMPTY alias —
    // the wizard never pre-fills a prior value; Settings is the edit
    // surface (D596 F2).
    byId("alias-input").value = "";
    show("scr-wizard-identity");
    fitCode(byId("identity-code"));
  } catch (e) {
    errEl.textContent = "Identity setup failed: " + e;
    show("scr-wizard-identity");
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
    fb.textContent = "Unlock failed: " + e;
  } finally {
    if (!countdownTimer || byId("unlock-feedback").className === "feedback") btn.disabled = false;
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
      byId("erase-error").textContent = "Erase failed: " + e;
    }
  }, 1000);
});
byId("btn-erase-countdown-cancel").addEventListener("click", () => eraseCountdownAbort());
byId("btn-erase-cancel").addEventListener("click", () => showUnlockScreen(unlockNext));
byId("btn-wiped-restart").addEventListener("click", () => route());

// ---- main window ----------------------------------------------------------
function enterMain() {
  show("scr-main");
  // slice A: exactly one honest status; no capability data exists to go stale.
  byId("status-line").textContent =
    "No server configured — server setup arrives in a future update.";
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
  const info = await invoke("app_info");
  byId("about-name").textContent = `${info.display_name} (qsl-desktop ${info.version})`;
  byId("about-text").textContent =
    `Slice ${info.slice}. This build makes no network connections and no security-assurance claims.`;
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
  byId("settings-purpose").textContent = rec.purpose_line;
  byId("settings-pq").textContent = rec.pq_line;
  byId("settings-mech").textContent = rec.mechanism_line;
  byId("settings-alias").value = currentSettings.self_alias;
  byId("alias-status").textContent = `Shown as: ${aliasDisplay()} (local only)`;
  fitCode(byId("settings-code"));
}

byId("btn-alias-save").addEventListener("click", async () => {
  currentSettings.self_alias = byId("settings-alias").value.trim();
  try {
    await saveSettings();
    acknowledge(byId("btn-alias-save"), "✓ Saved", byId("alias-status"),
      `Shown as: ${aliasDisplay()} (local only)`);
  } catch (e) {
    byId("alias-status").textContent = "Not saved: " + e;
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

// Item 12: banner copy per spec §2 (red reserved for the armed state).
function renderWipeBanner(s) {
  const el = byId("wipe-state");
  if (s.wipe_after === null) {
    setBanner(el, "neutral", "Off — wrong attempts never erase the vault");
  } else {
    setBanner(el, "danger",
      `Armed — erases after ${s.wipe_after} failed attempt${s.wipe_after === 1 ? "" : "s"}`);
  }
}
// Item 2c (E.3): the autolock banner state machine — value > 0 renders the
// accent lock banner; value == 0 renders the DANGER banner (the recorded
// R2 extension: red covers the never-locks state by operator decision).
function renderAutolockBanner(minutes) {
  const el = byId("autolock-status");
  if (minutes === 0) {
    setBanner(el, "danger",
      "Never locks — anyone with access to this device can open your vault");
  } else {
    setBanner(el, "accent",
      `Locks after ${minutes} minute${minutes === 1 ? "" : "s"} of inactivity`);
  }
}
async function refreshVaultPane() {
  renderAttemptsAlert();
  const s = await invoke("protection_status");
  renderWipeBanner(s);
  byId("wipe-limit").min = s.wipe_min;
  byId("wipe-limit").max = s.wipe_max;
  const cfg = await invoke("settings_get");
  adoptSettings(cfg);
  byId("autolock-min").value = cfg.autolock_minutes;
  renderAutolockBanner(cfg.autolock_minutes);
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
    renderWipeBanner(s);
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
    renderWipeBanner(s);
    acknowledge(byId("btn-wipe-disarm"), "✓ Off");
  } catch (e) {
    byId("wipe-error").textContent = String(e);
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
    renderAutolockBanner(minutes);
    acknowledge(byId("btn-autolock-save"), "✓ Saved");
  } catch (e) {
    err.textContent = String(e);
  }
});

// Item 6 (§5) + item 13: the destroy ceremony — always opens collapsed and
// empty; semantics unchanged (typed phrase + passphrase → the tokened core
// destroy).
byId("btn-destroy-open").addEventListener("click", () => {
  byId("destroy-flow").classList.remove("hidden");
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
    err.textContent = "Destroy refused: " + e;
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
    byId("unlock-feedback").textContent = "Locked after inactivity.";
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

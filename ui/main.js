// qsl-desktop slice A frontend. Static vanilla JS (F3: zero npm/node — no JS
// supply chain). All state lives in the backend; this file renders it.
// D596 design pass: presentation + the no-silent-state-changes rule only —
// every backend semantic is byte-for-byte the NA-0659 behavior.
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
  for (const s of SCREENS) byId(s).classList.toggle("hidden", s !== id);
  currentScreen = id;
}

// ---- item 5: the NO-SILENT-STATE-CHANGES rule, ONE implementation --------
// Every state-changing control acknowledges in two places: a momentary
// "✓ Saved"-style flash ON the control, and the section's persistent status
// line updated to the new reality. The flash is presentation; the status
// line is the durable truth. Microcopy stays factual ("Saved", never more).
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

// ---- GUI-local non-secret settings (autolock + self-alias) ----------------
let currentSettings = { autolock_minutes: 15, self_alias: "" };
function aliasDisplay() {
  return currentSettings.self_alias.trim() === "" ? "You" : currentSettings.self_alias.trim();
}
function updateRailInitial() {
  byId("rail-initial").textContent = [...aliasDisplay()][0].toLocaleUpperCase();
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

// ---- item 11 (binding rule): failed-attempts capture ----------------------
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

// ---- wizard step 1: vault (item 7 — the honest checklist gates Create) ---
// BEGIN COMMON_PASSWORDS (tests/design_system.rs asserts: >=100 entries,
// lowercase, sorted, unique; membership is checked case-insensitively)
const COMMON_PASSWORDS = [
  "000000", "102030", "111111", "112233", "121212", "123123", "123321",
  "1234", "12345", "123456", "1234567", "12345678", "123456789",
  "1234567890", "123abc", "131313", "159753", "191919", "1q2w3e",
  "1q2w3e4r", "202020", "212121", "222222", "232323", "252525", "333333",
  "444444", "555555", "654321", "666666", "696969", "777777", "789456",
  "888888", "987654", "987654321", "999999", "aaaaaa", "abc123",
  "abcd1234", "abcdef", "access", "admin", "admin123", "alexander",
  "amanda", "andrea", "andrew", "angel", "anthony", "apple", "arsenal",
  "asdfgh", "ashley", "austin", "banana", "barcelona", "baseball",
  "basketball", "batman", "blink182", "bubbles", "buster", "butterfly",
  "charlie", "cheese", "chelsea", "chicken", "chocolate", "computer",
  "cookie", "corvette", "daniel", "dexter", "dragon", "elephant",
  "family", "ferrari", "flower", "football", "forever", "freedom",
  "friends", "george", "ginger", "hannah", "harley", "hello", "hockey",
  "hunter", "iloveyou", "jasmine", "jennifer", "jessica", "jordan",
  "jordan23", "joshua", "juventus", "killer", "letmein", "liverpool",
  "london", "lovely", "maggie", "master", "matrix", "matthew", "melissa",
  "michael", "michelle", "monkey", "mustang", "naruto", "nicole", "ninja",
  "orange", "password", "password1", "password123", "peanut", "pepper",
  "pokemon", "princess", "purple", "qazwsx", "qwerty", "qwerty123",
  "qwertyuiop", "ranger", "robert", "samantha", "samsung", "secret",
  "shadow", "snoopy", "soccer", "sophie", "spiderman", "starwars",
  "summer", "sunshine", "superman", "taylor", "thomas", "tigger",
  "trustno1", "welcome", "whatever", "william", "winter", "zaq12wsx",
  "zxcvbnm",
];
// END COMMON_PASSWORDS

function updateReqs() {
  const p = byId("vault-pass").value;
  const c = byId("vault-confirm").value;
  const okLen = p.length >= 12;
  const okMatch = p.length > 0 && p === c;
  const okCommon = p.length > 0 && !COMMON_PASSWORDS.includes(p.toLowerCase());
  byId("req-len").classList.toggle("ok", okLen);
  byId("req-match").classList.toggle("ok", okMatch);
  byId("req-common").classList.toggle("ok", okCommon);
  // Create gates on ALL green (a UI gate; the core contract is unchanged).
  byId("btn-vault-create").disabled = !(okLen && okMatch && okCommon);
}

function strengthEstimate(p) {
  if (!p) return null;
  let classes = 0;
  if (/[a-z]/.test(p)) classes++;
  if (/[A-Z]/.test(p)) classes++;
  if (/[0-9]/.test(p)) classes++;
  if (/[^a-zA-Z0-9]/.test(p)) classes++;
  const score = Math.min(4, Math.floor(p.length / 6) + (classes > 2 ? 1 : 0));
  const labels = ["very weak", "weak", "fair", "good", "strong"];
  return { label: labels[score], lvl: score, pct: [10, 25, 50, 75, 100][score] };
}
byId("vault-pass").addEventListener("input", () => {
  const est = strengthEstimate(byId("vault-pass").value);
  byId("strength").innerHTML = est
    ? `<span class="bar"><i class="lvl${est.lvl}" style="width:${est.pct}%"></i></span>` +
      `${est.label} <span class="hint">(guidance only)</span>`
    : "";
  updateReqs();
});
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
    byId("vault-pass").value = "";
    byId("vault-confirm").value = "";
    // Step 1 is DONE and not revisitable (no false Back — un-creating a
    // vault is not a navigation action). Straight into step 2:
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

// ---- wizard step 2: identity (item 8 — the verification code is the hero) -
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
    byId("alias-input").value = currentSettings.self_alias;
    show("scr-wizard-identity");
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
      // The captured pre-unlock count becomes the item-11 alert value; the
      // core counter has just reset itself (the binding capture rule).
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
  byId("erase-phrase").value = "";
  byId("erase-error").textContent = "";
  show("scr-erase"); // deliberate step 2 of 2: the typed phrase below
});

// ---- erase (forgotten passphrase; app-level file removal ONLY) -----------
byId("btn-erase").addEventListener("click", async () => {
  const phrase = byId("erase-phrase").value;
  const err = byId("erase-error");
  err.textContent = "";
  if (phrase !== "erase everything") {
    err.textContent = 'Type exactly: erase everything';
    return;
  }
  try {
    await invoke("erase_all", { confirmPhrase: phrase });
    await route();
  } catch (e) {
    err.textContent = "Erase failed: " + e;
  }
});
byId("btn-erase-cancel").addEventListener("click", () => showUnlockScreen(unlockNext));
byId("btn-wiped-restart").addEventListener("click", () => route());

// ---- main window ----------------------------------------------------------
function enterMain() {
  updateRailInitial();
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

// ---- settings -------------------------------------------------------------
// F1: the Identity pane is first in the Settings rail; the rail identity dot
// and the gear both open Settings (landing on the first pane, Identity).
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
byId("btn-rail-identity").addEventListener("click", () => openSettings("identity"));
byId("btn-settings-close").addEventListener("click", () => enterMain());

function selectPane(name) {
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

// ---- item 9: the Identity pane (existing identity_show surface ONLY) ------
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
}

byId("btn-alias-save").addEventListener("click", async () => {
  currentSettings.self_alias = byId("settings-alias").value.trim();
  try {
    await saveSettings();
    updateRailInitial();
    acknowledge(byId("btn-alias-save"), "✓ Saved", byId("alias-status"),
      `Shown as: ${aliasDisplay()} (local only)`);
  } catch (e) {
    byId("alias-status").textContent = "Not saved: " + e;
  }
});

// ---- item 11: Vault & Security (controls first; silent at zero) -----------
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

function wipeStatusText(s) {
  return s.wipe_after === null
    ? "Off — wrong attempts never erase the vault."
    : `Armed — erases after ${s.wipe_after} failed attempts.`;
}
async function refreshVaultPane() {
  renderAttemptsAlert();
  const s = await invoke("protection_status");
  byId("wipe-state").textContent = wipeStatusText(s);
  byId("wipe-limit").min = s.wipe_min;
  byId("wipe-limit").max = s.wipe_max;
  const cfg = await invoke("settings_get");
  adoptSettings(cfg);
  byId("autolock-min").value = cfg.autolock_minutes;
  byId("autolock-status").textContent =
    `Locks after ${cfg.autolock_minutes} minute${cfg.autolock_minutes === 1 ? "" : "s"} of inactivity.`;
}

byId("btn-wipe-arm").addEventListener("click", async () => {
  const err = byId("wipe-error");
  err.textContent = "";
  if (!byId("wipe-ack").checked) {
    err.textContent = "Tick the confirmation first — arming can permanently erase the vault.";
    return;
  }
  const limit = parseInt(byId("wipe-limit").value, 10);
  try {
    await invoke("wipe_arm", { limit });
    byId("wipe-ack").checked = false;
    const s = await invoke("protection_status");
    acknowledge(byId("btn-wipe-arm"), "✓ Armed", byId("wipe-state"), wipeStatusText(s));
  } catch (e) {
    err.textContent = mapErr(e, { wipe_limit_out_of_bounds: "Limit must be between 1 and 100." });
  }
});
byId("btn-wipe-disarm").addEventListener("click", async () => {
  byId("wipe-error").textContent = "";
  try {
    await invoke("wipe_disarm");
    const s = await invoke("protection_status");
    acknowledge(byId("btn-wipe-disarm"), "✓ Off", byId("wipe-state"), wipeStatusText(s));
  } catch (e) {
    byId("wipe-error").textContent = String(e);
  }
});

byId("btn-autolock-save").addEventListener("click", async () => {
  byId("autolock-error").textContent = "";
  const minutes = parseInt(byId("autolock-min").value, 10);
  try {
    currentSettings.autolock_minutes = minutes;
    await saveSettings();
    autolockMinutes = minutes;
    acknowledge(byId("btn-autolock-save"), "✓ Saved", byId("autolock-status"),
      `Locks after ${minutes} minute${minutes === 1 ? "" : "s"} of inactivity.`);
  } catch (e) {
    byId("autolock-error").textContent = mapErr(e, {
      autolock_minimum_one_minute: "The minimum autolock interval is 1 minute.",
    });
  }
});

// item 12: destroy — the shared confirmation pattern (semantics unchanged:
// typed phrase + passphrase → the tokened core destroy).
byId("btn-destroy-open").addEventListener("click", () => {
  byId("destroy-flow").classList.remove("hidden");
  byId("destroy-pass").value = "";
  byId("destroy-phrase").value = "";
  byId("destroy-error").textContent = "";
});
byId("btn-destroy-cancel").addEventListener("click", () =>
  byId("destroy-flow").classList.add("hidden"));
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
    await route(); // the vault is gone; the state machine lands in S0
  } catch (e) {
    err.textContent = "Destroy refused: " + e;
  }
});

// ---- idle autolock (ON by default, ~15 min, adjustable; wizard exempt) ----
let autolockMinutes = 15;
let idleSince = Date.now();
for (const ev of ["mousemove", "mousedown", "keydown", "wheel", "touchstart"]) {
  window.addEventListener(ev, () => { idleSince = Date.now(); }, { passive: true });
}
setInterval(async () => {
  const onLockedSurface = currentScreen === "scr-main" || currentScreen === "scr-settings";
  if (!onLockedSurface) return; // the wizard (and unlock itself) is exempt
  if (Date.now() - idleSince >= autolockMinutes * 60 * 1000) {
    idleSince = Date.now();
    await invoke("lock_now"); // the one-call NA-0658 lock()
    await showUnlockScreen("main");
    byId("unlock-feedback").textContent = "Locked after inactivity.";
  }
}, 5000);

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

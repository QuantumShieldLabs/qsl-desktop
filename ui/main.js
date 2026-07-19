// qsl-desktop slice A frontend. Static vanilla JS (F3: zero npm/node — no JS
// supply chain). All state lives in the backend; this file renders it.
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
    unlockNext = "wizard-identity";
    show("scr-unlock");
  } else {
    unlockNext = "main";
    show("scr-unlock");
  }
}

// ---- wizard step 1: vault -------------------------------------------------
function strengthEstimate(p) {
  if (!p) return null;
  let classes = 0;
  if (/[a-z]/.test(p)) classes++;
  if (/[A-Z]/.test(p)) classes++;
  if (/[0-9]/.test(p)) classes++;
  if (/[^a-zA-Z0-9]/.test(p)) classes++;
  const score = Math.min(4, Math.floor(p.length / 6) + (classes > 2 ? 1 : 0));
  const labels = ["very weak", "weak", "fair", "good", "strong"];
  const colors = ["#b0524d", "#b0524d", "#b08a4d", "#5d9e6f", "#5d9e6f"];
  const pct = [10, 25, 50, 75, 100][score];
  return { label: labels[score], color: colors[score], pct };
}
byId("vault-pass").addEventListener("input", () => {
  const est = strengthEstimate(byId("vault-pass").value);
  byId("strength").innerHTML = est
    ? `<span class="bar"><i style="width:${est.pct}%;background:${est.color}"></i></span>` +
      `${est.label} <span class="small">(guidance only — length matters most)</span>`
    : "";
});

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
  } finally {
    btn.disabled = false;
  }
});

async function showIdentityStep() {
  const errEl = byId("identity-error");
  errEl.textContent = "";
  try {
    const d = await invoke("identity_ensure");
    byId("identity-fp").textContent = d.fingerprint;
    byId("identity-code").textContent = d.verify_code;
    byId("identity-purpose").textContent = d.purpose_line;
    byId("identity-pq").textContent = d.pq_line;
    show("scr-wizard-identity");
  } catch (e) {
    errEl.textContent = "Identity setup failed: " + e;
    show("scr-wizard-identity");
  }
}

byId("btn-identity-done").addEventListener("click", () => enterMain());

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
      if (unlockNext === "wizard-identity") await showIdentityStep();
      else enterMain();
    } else if (r.kind === "rejected") {
      fb.className = "feedback reject";
      if (r.retry_after_s > 0) startCountdown(r.retry_after_s, r.failed_unlocks);
      else fb.textContent = `Wrong passphrase. Failed attempts: ${r.failed_unlocks}.`;
    } else if (r.kind === "delayed") {
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
byId("btn-erase-cancel").addEventListener("click", () => show("scr-unlock"));
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

// ---- settings -------------------------------------------------------------
byId("btn-settings").addEventListener("click", async () => {
  show("scr-settings");
  selectPane("vault");
  await refreshVaultPane();
  const info = await invoke("app_info");
  byId("about-text").textContent =
    `qsl-desktop ${info.version} — slice ${info.slice}. This build makes no network connections and no security-assurance claims.`;
});
byId("btn-settings-close").addEventListener("click", () => enterMain());

function selectPane(name) {
  for (const b of document.querySelectorAll(".settings-rail .cat[data-pane]")) {
    b.classList.toggle("active", b.dataset.pane === name);
  }
  for (const p of ["server", "vault", "appearance", "notifications", "about"]) {
    byId("pane-" + p).classList.toggle("hidden", p !== name);
  }
}
for (const b of document.querySelectorAll(".settings-rail .cat[data-pane]")) {
  b.addEventListener("click", () => selectPane(b.dataset.pane));
}

async function refreshVaultPane() {
  const s = await invoke("protection_status");
  byId("prot-status").textContent =
    `Failed unlock attempts: ${s.failed_unlocks}` +
    (s.retry_after_s > 0 ? ` — delay in force: ${s.retry_after_s}s` : "") +
    ` — vault ${s.locked ? "locked" : "unlocked"}`;
  byId("wipe-state").textContent = s.wipe_after === null
    ? "Currently: OFF (default)."
    : `Currently: ARMED — erases after ${s.wipe_after} failed attempts.`;
  byId("wipe-limit").min = s.wipe_min;
  byId("wipe-limit").max = s.wipe_max;
  const cfg = await invoke("settings_get");
  byId("autolock-min").value = cfg.autolock_minutes;
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
    await refreshVaultPane();
  } catch (e) {
    err.textContent = mapErr(e, { wipe_limit_out_of_bounds: "Limit must be between 1 and 100." });
  }
});
byId("btn-wipe-disarm").addEventListener("click", async () => {
  byId("wipe-error").textContent = "";
  try {
    await invoke("wipe_disarm");
    await refreshVaultPane();
  } catch (e) {
    byId("wipe-error").textContent = String(e);
  }
});

byId("btn-autolock-save").addEventListener("click", async () => {
  const minutes = parseInt(byId("autolock-min").value, 10);
  try {
    await invoke("settings_set", { autolockMinutes: minutes });
    autolockMinutes = minutes;
    await refreshVaultPane();
  } catch (e) {
    byId("wipe-error").textContent = mapErr(e, {
      autolock_minimum_one_minute: "The minimum autolock interval is 1 minute.",
    });
  }
});

// destroy: a deliberate multi-step flow (open → passphrase + typed phrase).
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
    unlockNext = "main";
    byId("unlock-feedback").textContent = "Locked after inactivity.";
    show("scr-unlock");
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
    autolockMinutes = cfg.autolock_minutes;
  } catch (_) { /* defaults stand */ }
  await route();
})();

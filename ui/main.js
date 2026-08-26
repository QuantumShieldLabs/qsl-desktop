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
  // NA-0755 (D-0036, R380 §7): the one-time invite code is held to the SAME
  // rule, and it is held STRUCTURALLY rather than by remembering.
  //
  // ⚠ Why it must live HERE and not at the call sites. The invite modal is a
  // body-level overlay and is deliberately NOT a `SCREENS` member, so the loop
  // below cannot hide it. There are eight `show()` call sites and one of them
  // is the autolock path (`:232`) — an autolock firing with the modal open
  // would otherwise leave a live one-time code rendered OVER the unlock screen.
  // One line here covers every transition, including ones not yet written; the
  // alternative is a habit that must be re-derived at each new call site, and a
  // habit that must be re-derived is not a control.
  closeInviteModal();
  // NA-0756 (D-0037): the redeem overlay is held to the SAME structural rule for the SAME
  // reason — a pasted invite code is a one-time capability, and an autolock firing with the
  // redeem surface open would otherwise leave it rendered over the unlock screen. One line
  // here covers every transition including ones not yet written.
  closeRedeemModal();
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
  // NA-0763 (`D-0040`): the stored tempo. ⚠ `settings_set` does NOT carry it
  // (see `saveSettings` above, still two fields) — nothing in this lane writes
  // the knob, so an absent key is the blessed default rather than a loss.
  tickTempo = TICK_TEMPO[cfg.tempo] ? cfg.tempo : TICK_DEFAULT;
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

// NA-0753 (R377 §1; design bank v2 item 2) — THE VERIFICATION CODE IS READ
// ALOUD, so it renders in six 5-digit groups on BOTH surfaces (onboarding
// "This is you" and Settings > Identity).
//
// PRESENTATION ONLY: the backend still returns, and every Rust value test
// still pins, the raw 30 digits. Nothing upstream changes.
//
// ⚠ ONE TEXT NODE, NOT A `<br>` SPLIT, AND THE REASON IS MEASURED. Ratified
// mockup-07:74 draws a fixed 3+3 two-line card, but `.verify-code` is
// `white-space: nowrap; overflow: hidden` with a one-line `line-height: 1.6`,
// and `fitCode()` only releases the clip (adding `.wrapped`) when
// `scrollWidth > clientWidth`. A `<br>` halves each line's width, so that
// escape could NEVER fire and the second line would clip SILENTLY — the exact
// class `verify_code_never_clips_silently` exists to prevent. Ruled at R377
// §1; the layout delta from the mockup is recorded in D-0034 with a
// mockup-refresh candidate, never left silent.
//
// ⛳ As a single spaced line this also makes fitCode's own promise TRUE for
// the first time: at the 11px floor `.wrapped` breaks the code at a GROUP
// BOUNDARY (a space), which against the previous 30 bare digits had no
// boundary to land on.
//
// A value that is not exactly 30 digits renders RAW rather than being
// silently regrouped — an honest fallback beats a tidy lie about an
// unexpected shape.
function groupedCode(code) {
  const raw = code == null ? "" : String(code);
  const digits = raw.replace(/\s+/g, "");
  if (digits.length !== 30) return raw;
  return digits.match(/.{5}/g).join(" ");
}

async function showIdentityStep() {
  const errEl = byId("identity-error");
  errEl.textContent = "";
  try {
    const d = await invoke("identity_ensure");
    byId("identity-code").textContent = groupedCode(d.verify_code);
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
// ⚠ THE UNLOCK FEEDBACK LINE IS A CONDITIONAL ELEMENT, AND WRITING IT MUST
// RESIZE THE WINDOW. This helper exists so that cannot be forgotten.
//
// The re-flight found "Delete vault?" disappearing the moment a wrong
// passphrase was entered — the feedback text appears, the content grows, the
// window does not, and the link is pushed out of view. That is the ORIGINAL
// R-14 defect, found a THIRD time.
//
// It survived because the previous fix wired the sync at the ONE write the
// finding happened to name (the autolock notice), while D615's own rule says
// "after ANY write to a conditional element". Five other writers — the empty
// reset, the empty-passphrase guard, the rejected-attempt line, the countdown
// tick and its expiry, the error path — each wrote and none resized.
//
// So the remedy is structural rather than another reminder: there is now ONE
// way to write this element and it resizes. `design_polish.rs` asserts no
// other writer exists, which is what makes this a control instead of a note.
function setUnlockFeedback(kind, text) {
  const fb = byId("unlock-feedback");
  fb.className = kind ? "feedback " + kind : "feedback";
  fb.textContent = text;
  syncWindowHeight();
}

let countdownTimer = null;
function startCountdown(seconds, failed) {
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
      setUnlockFeedback("", failed > 0 ? `Failed attempts: ${failed}. You can try again now.` : "");
      return;
    }
    setUnlockFeedback(
      "reject",
      `Too many failed attempts (${failed}). Try again in ${left} second${left === 1 ? "" : "s"}.`
    );
    left -= 1;
  };
  tick();
  countdownTimer = setInterval(tick, 1000);
}

byId("btn-unlock").addEventListener("click", async () => {
  const pass = byId("unlock-pass").value;
  setUnlockFeedback("", "");
  if (!pass) { setUnlockFeedback("reject", "Enter your passphrase."); return; }
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
      if (r.retry_after_s > 0) startCountdown(r.retry_after_s, r.failed_unlocks);
      else setUnlockFeedback("reject", `Wrong passphrase. Failed attempts: ${r.failed_unlocks}.`);
    } else if (r.kind === "version_unsupported") {
      // NA-0705: the vault was written by an older version. A distinct cause gets its
      // own words — never "Wrong passphrase", which is what the user would otherwise be
      // told while entering the correct one.
      setUnlockFeedback(
        "reject",
        "This vault was created by an older version of the app and can't be opened by this one. Your passphrase was not the problem.",
      );
    } else if (r.kind === "delayed") {
      observedFailedUnlocks = r.failed_unlocks;
      startCountdown(r.retry_after_s, r.failed_unlocks);
    } else if (r.kind === "wiped") {
      show("scr-wiped");
    }
  } catch (e) {
    // R-17: a lead SENTENCE, never a bare code.
    setUnlockFeedback("reject", unlockErrorText(e));
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
  setEraseError("");
  show("scr-erase"); // deliberate step 2 of 2; show() clears the phrase field
});

// ---- erase (app-level file removal ONLY) ---------------------------------
// Item 11b (E.5): the 30-second countdown GATE. It gates WHEN the erase
// commits, never what it erases — erase_all, its phrase check, and its
// scope are byte-untouched; the command is invoked ONLY at countdown zero.
// Cancel, closing the window, or any state transition ABORTS with nothing
// erased.
//
// ⚠ R-14, FOURTH OCCURRENCE (ENG-0123, NA-0702 / D-0027): the ceremony error
// line is a conditional element, and every writer used to skip the resize —
// after a wrong phrase the content grew past the card's clip and BOTH Erase
// and Cancel fell out of reach on the app's most stressful screen. Same
// remedy as setUnlockFeedback above: there is ONE way to write the element
// and that way resizes. `design_polish.rs` counts the references so a second
// writer cannot appear silently.
function setEraseError(text) {
  const el = byId("erase-error");
  if (el) el.textContent = text;
  syncWindowHeight();
}
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
  setEraseError("");
}
byId("btn-erase").addEventListener("click", () => {
  const phrase = byId("erase-phrase").value;
  setEraseError("");
  if (phrase !== "erase everything") {
    setEraseError('Type exactly: erase everything');
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
      setEraseError(plainError(e, {
        erase_refused_cli_dir: "Nothing was erased — this app refused to touch the command-line tool's data folder.",
      }, "Nothing was erased."));
    }
  }, 1000);
});
byId("btn-erase-countdown-cancel").addEventListener("click", () => eraseCountdownAbort());
byId("btn-erase-cancel").addEventListener("click", () => showUnlockScreen(unlockNext));
byId("btn-wiped-restart").addEventListener("click", () => route());

// ---- main window ----------------------------------------------------------

// NA-0752 (D-0033, ruled at R374): THE STATUS FOOTER TELLS THE TRUTH. It stops
// knowing one sentence and reports the desk's typed state.
//
// ⚠ TWO SOURCES ARE STRUCTURAL, NOT A CONVENIENCE. `qsp_status_tuple` never
// reads relay config — measured: `config_dir()` reads only env, `check_parent_
// safe` only filesystem permissions, `qsp_session_load` only the session blob —
// so the desk CANNOT say "no relay configured", and `relay_config_get` cannot
// say the store is unwell. Either source alone would ship a false line.
//
// PRECEDENCE IS WORST-FIRST and it is load-bearing: the first matching row
// wins, so a storage fault is never hidden behind a cheerful "Ready."
const STATUS_FOOTER_STORAGE = "Storage problem — check Settings › Vault.";
const STATUS_FOOTER_LOCKED = "Locked — unlock to connect.";
const STATUS_FOOTER_UNKNOWN = "Status unknown — please report this.";
const STATUS_FOOTER_NO_RELAY = "No relay configured — add one in Settings › Relay.";

// PURE AND TOTAL: every input maps to exactly one shipped sentence. `reason`
// is the desk's wire string, or `null` when the desk did not answer.
//
// ⚠ FIVE OF THE NINE REASONS ARE DELIBERATELY NON-SIGNALLING HERE, AND THAT IS
// A DECISION, NOT A GAP (D-0033). `handshake`, `no_session`, `missing_seed`,
// `session_invalid` and `channel_invalid` are PER-CONTACT — they describe one
// peer, not the app. A healthy fresh profile answers `missing_seed`, so a
// footer rendering it as a problem would call every new install broken. They
// fall through to the relay rows by design.
//
// ⚠ Two arms are RESIDUAL rather than routine, and are kept for honesty:
// `vault_locked` cannot normally be seen here at all (this footer lives inside
// `scr-main`, and every lock path navigates to `scr-unlock`), and
// `missing_home` is unreachable while `bootstrap()` sets QSC_CONFIG_DIR before
// the runtime exists. A footer that could not say "storage is wrong" when the
// desk says so is the dishonesty this lane exists to remove.
function statusFooterLine(reason, relayUrl) {
  if (reason === "missing_home" || reason === "unsafe_parent") {
    return STATUS_FOOTER_STORAGE;
  }
  if (reason === "vault_locked") {
    return STATUS_FOOTER_LOCKED;
  }
  // The honest tripwire: an EIGHTH upstream reason string, or a desk that did
  // not answer at all. A typed failure must never arrive as silence.
  if (reason === null || reason === "unrecognized") {
    return STATUS_FOOTER_UNKNOWN;
  }
  if (!relayUrl) {
    return STATUS_FOOTER_NO_RELAY;
  }
  return `Ready. Relay: ${relayUrl}`;
}

async function enterMain() {
  show("scr-main");
  // Slice B (D609 R4) kept: the footer reflects the ACTUAL relay config, never
  // a "future update" claim. NA-0752 adds the desk's answer beside it.
  //
  // The peer label is `peer-0` — the TREE'S OWN convention for an app-level
  // status read, hard-coded in qsc's own `status` verb (main.rs:95). ⚠ It is a
  // real, valid contact label; the hazard is recorded in D-0033 with a forward
  // trigger, because it is non-load-bearing only while the per-contact reasons
  // fall through.
  //
  // A rejection from EITHER command leaves `reason` null, which the mapping
  // renders as the honest tripwire rather than as a stale or empty line.
  let reason = null;
  let relayUrl = "";
  try {
    const cfg = await invoke("relay_config_get");
    relayUrl = cfg.relay_url || "";
    const status = await invoke("connect_status", { peer: "peer-0" });
    reason = status.reason;
  } catch (_) {
    reason = null;
  }
  byId("status-line").textContent = statusFooterLine(reason, relayUrl);
  // NA-0763: the tick's gate needs to know whether a relay exists at all, and
  // this function has just read it. R10: no relay configured => no ticks, which
  // is both correct product behaviour (there is nothing to pull) and politeness
  // to a small box.
  tickRelayConfigured = relayUrl !== "";
  // ⚠ NA-0756 (D-0037) — FINISH TRIGGER (a), at vault unlock: one bounded scan per contact
  // that is still establishing. It runs LAST so a finish failure cannot cost the footer, and
  // it is awaited rather than fired-and-forgotten so the harness can observe its outcome.
  // The wizard branch does not reach here, which is correct and measured: a vault being
  // created has no contacts to scan.
  // ⚠ NA-0763: re-routed through the ONE handler. The behaviour of this trigger is
  // unchanged — `relayScan` runs the same finish-scan class and still marks the redeem
  // slot for user-caused sources.
  await relayScan({ source: "unlock", at: Date.now() });
}
// NA-0755 (D-0036): UN-STUBBED. This button now opens the real invite flow.
//
// ⚠ `#stub-note` and its OTHER TWO revealers (`btn-rail-contacts` below and
// `btn-rail-contacts-s` in the settings rail) are deliberately LEFT IN PLACE:
// the Contacts pane is Lane C and is still unbuilt, so the honest stub is still
// the truth on those paths. Only THIS handler stops revealing it — which is why
// the seal is scoped to this handler and not to the element's existence.
// NA-0756 (D-0037, R387 §S4): RETARGETED to the chooser. Both contact-making acts now live
// behind one entry — "Invite someone" reaches the mint, "I have a code" reaches the redeem
// flow. The chooser is a STOPGAP: Lane C replaces it with the New-chat panel, and the panel
// IS the chooser plus the contacts list (design bank §1).
byId("btn-add-contact").addEventListener("click", () => openRedeemChooser());
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
  byId("settings-code").textContent = groupedCode(rec.verify_code);
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
let serverBusy = false;
// NA-0754: the ~ expansion must be VISIBLE in the field before the path is used,
// and the webview cannot resolve `~` on its own — $HOME is a process fact. Cached
// once per pane open from the `home_dir` command; "" means "could not resolve",
// and the gate then REFUSES `~` rather than guessing at a path nobody typed.
let homeDir = "";

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
// R-B3 STANDS (R379 Q1): a blank field means KEEP WHAT'S THERE. Removal is the x
// control's job now — it deletes immediately and works with no relay reachable —
// so a blank field is never a removal request and never dirty.
function tokenDirty() { return byId("relay-token").value !== ""; }
function caDirty() { return byId("relay-ca-path").value.trim() !== ""; }
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

// NA-0754 (design bank v2 item 2) — THE HELPER SENTENCES ARE GONE, AND THE FIELDS
// CARRY THE STATE. Six per-field status sentences were removed: the token's
// set/unset pair, the CA's set/unset pair, and the two pending-removal siblings
// that the immediate-x makes unreachable. The bottom status line and the results
// banner carry connection truth; these two fields carry stored-state, and nothing
// narrates.
//
// ⚠ THE SENTENCES ARE DESCRIBED HERE, NEVER QUOTED, and that is deliberate. The
// absence seal in design_polish.rs is a SOURCE-TEXT pin — it cannot tell a comment
// from live copy — so quoting a retired string to document its retirement would
// re-introduce it and turn the seal green-when-it-should-be-red. Recording a
// removal must not plant the thing removed; the retired wording lives in the
// lane's records, which no seal reads.
//
// ⚠ THE PLACEHOLDERS ARE OWNED BY JS, NOT BY THE MARKUP, and that is deliberate
// twice over. It is how each field reports stored state (the token's fixed eight
// dots; the CA's stored marker), AND it gives the harness a real transition to
// settle on — an attribute present in the HTML from the first byte can never
// signal that an async refresh has finished. The retired #relay-token-help was
// the old settle signal; this replaces it (see f_j / the re-aimed f_i note).
const CA_PLACEHOLDER_UNSET = "/path/to/ca.pem";

function renderFieldState() {
  // TOKEN. R-E1 unchanged: relay_token_show is a bare bool by design, so the app
  // cannot know the token's length and must not appear to — FIXED eight dots.
  const input = byId("relay-token");
  if (tokenConfigured) {
    input.placeholder = "••••••••";
  } else {
    input.placeholder = "";
  }
  byId("relay-token-clear").classList.toggle("hidden", !tokenConfigured);

  // CA. R379 Q3: qsc's path redaction STANDS — relay_ca_file_show() returns
  // {configured, path_hash} and never the path, so the field cannot echo the
  // value. It reports STORED-STATE instead, exactly as the token's dots do: one
  // word plus the hash8 marker the retired status line already displayed.
  byId("relay-ca-path").placeholder = caConfigured
    ? "Set · " + (caPathHash || "configured")
    : CA_PLACEHOLDER_UNSET;
  byId("relay-ca-clear").classList.toggle("hidden", !caConfigured);
}

// F1R (operator-ruled): ANY change to what the app will use clears the results.
// Results describing a configuration the user has already changed are
// stale-but-technically-true, which is the kind that gets misread.
function onServerChanged() {
  clearServerResults();
  renderDirty();
}

// NA-0754 — THE ALWAYS-DELETABLE AFFORDANCE, AND IT DELETES NOW.
//
// ⚠ WHY IMMEDIATE AND NOT PENDING. The old "remove it" link set a flag that only
// committed inside the old commitServerSettings(), whose FIRST step was the address. So a
// user with a mistyped address could not remove a stored token at all: the gate
// refused, the function returned, and the clear never ran — the house's own
// "a stored secret must ALWAYS be deletable" principle failing on a path nobody
// had measured (NA-0754 §5, filed and resolved as ENG-0225). Deleting on the
// click removes the dependency rather than documenting it.
//
// Neither clear touches the network: both resolve to a vault write of an empty
// value through the gateway, so they work with no relay reachable — which is the
// whole point of keeping the affordance.
//
// ⚠ The mechanism is described, never named in its qsc spelling: this file is
// scanned by `no_secret_is_written_outside_the_qsc_vault_trios`, a SOURCE-TEXT
// boundary test that cannot tell a comment from a call. Writing the forbidden
// construct here to explain it would trip the very seal it explains.
async function clearStoredSecret(which) {
  if (serverBusy) return;
  setServerBusy(true, which === "token" ? "Removing token…" : "Removing certificate…");
  try {
    // ⚠ THE TWO CALLS ARE WRITTEN OUT IN FULL, DELIBERATELY. A ternary picking the
    // command name would hide both from `no_secret_is_written_outside_the_qsc_vault_trios`,
    // which scans this file for the literal trio invocations. A boundary test that
    // cannot see a secret write is worse than no test, so the call sites stay greppable.
    if (which === "token") {
      await invoke("relay_token_clear");
    } else {
      await invoke("relay_ca_file_clear");
    }
    await refreshServerState();
    onServerChanged();
  } catch (e) {
    byId("relay-results").classList.remove("hidden");
    byId("relay-doc").innerHTML = "";
    setBanner(byId("relay-status"), "accent", "Couldn't remove it");
    byId("relay-detail").textContent =
      (which === "token"
        ? "The access token couldn't be removed from your vault ("
        : "The certificate authority file couldn't be removed from your vault (") +
      String(e) + "). Nothing else was changed.";
  } finally {
    setServerBusy(false);
  }
}

byId("relay-token-clear").addEventListener("click", () => clearStoredSecret("token"));
byId("relay-ca-clear").addEventListener("click", () => clearStoredSecret("ca"));

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
  // NA-0754: the Save button is gone; the two clear controls are the only other
  // controls that reach the vault, and they must not re-enter mid-flight either.
  byId("relay-token-clear").disabled = on;
  byId("relay-ca-clear").disabled = on;
  if (!on) return;
  byId("relay-results").classList.remove("hidden");
  byId("relay-doc").innerHTML = "";
  byId("relay-detail").textContent = "";
  setBanner(byId("relay-status"), "neutral", label);
}

// ⚠ THE WRITE ORDER — R-B1's ORIGINAL ORDER RESTORED (NA-0754, R379 §Q2).
//
// THE HISTORY, because the inversion is being undone rather than overridden.
// D610's C2 ruled "validate the URL → token → CA → settings.json LAST" on the
// premise that the URL could be validated WITHOUT writing. That premise was
// false: the crate exposed nine relay commands and NONE was validate-only —
// `relay_config_set` normalized and wrote settings.json in the same call, and
// `relay_ca_file_set` "validated by writing". Neither field could be checked
// without committing it. That put R-B1 (vault first, settings.json last) in
// direct conflict with R-B2 (a malformed address blocks the ENTIRE commit with
// nothing persisted), and R-B2 won: the address had to be validated before any
// vault write, so — validating being writing — the address committed FIRST.
//
// ⛳ THE V2 DESIGN BANK DISSOLVES THE FORCING RATHER THAN PICKING A SIDE. The
// probe now runs BEFORE every write, against explicitly-supplied field values
// (`relay_probe`), so nothing has to be persisted in order to be checked. With
// the forcing gone, R-B1's ordering becomes available again and is RESTORED:
//
//     (1) vault token  →  (2) vault CA path  →  (3) settings.json LAST
//
// WHY settings.json LAST, stated so it can be attacked: `relay_url` is the
// OBSERVABLE configuration — the status footer reads it and relaunch reads it.
// If a vault write fails, the address has not moved, so the app still points at
// the last proven-good relay and the surviving configuration stays COHERENT.
// Address-first leaves a NEW address paired with OLD credentials, which is the
// clobber shape itself.
//
// NO ROLLBACK, and none is needed. Each store's write is individually atomic —
// settings::save is tmp+rename, and qsc's vault secret writer holds an exclusive
// lock across the whole read-modify-write and then does tmp→fsync→rename. So a
// mid-sequence failure always leaves prefix-new/suffix-old, never a torn value,
// and the message NAMES what landed.
//
// ⚠ THE HONEST READING OF THE INVARIANT (ratified, D-1396): a green test that
// COMPLETES its writes persists exactly the tested triple; a partial write NAMES
// what landed; and everything persisted has still connected at least once —
// which is the guarantee that actually protects the user.
//
// Returns null on success, or {part, message, inline} on the first failure, at
// which point the remainder is ABANDONED.
async function persistProvenSettings(proven) {
  // (1) THE TOKEN — to the vault, through the qsc trio.
  if (proven.token !== null) {
    try {
      await invoke("relay_token_set", { token: proven.token });
    } catch (e) {
      const code = String(e);
      const lead = code.includes("relay_token_missing")
        ? "Enter a token first."
        : "The access token couldn't be saved to your vault (" + code + ").";
      return { part: "vault", message: lead + " Your settings were not changed.", inline: false };
    }
  }

  // (2) THE CA FILE — to the vault, through the qsc trio.
  if (proven.caPath !== null) {
    try {
      await invoke("relay_ca_file_set", { path: proven.caPath });
    } catch (e) {
      byId("relay-ca-error").textContent = mapErr(e, {
        relay_ca_file_missing: "No file at that path.",
        relay_ca_file_unreadable: "That file can't be read.",
        relay_ca_file_invalid: "That file isn't a certificate.",
      });
      return { part: "vault", message: "Your access token was saved. The certificate authority file couldn't be saved to your vault. Your relay address is unchanged.", inline: false };
    }
  }

  // (3) THE ADDRESS — settings.json, LAST.
  if (proven.address !== null) {
    try {
      await invoke("relay_config_set", { url: proven.address });
    } catch (e) {
      return {
        part: "settings",
        message: "Your token and certificate settings were saved. The relay address couldn't be saved (" + String(e) + "), so the app is still using the previous one.",
        inline: false,
      };
    }
  }
  return null;
}

// NA-0753 (R376 §3; design bank v2 item 4) — THE RELAY-ADDRESS GATE.
//
// ⚠ WHY IT EXISTS. The engine's `normalize_relay_endpoint` is
// `Url::parse().to_string()` minus a trailing slash (`qsc` route.rs:71-74),
// and `validate_relay_endpoint_url` (:50-69) applies NO host-shape check at
// all — it requires only that a host exists and the scheme is https. WHATWG
// URL parsing reads an ALL-DIGIT host as a packed IPv4 integer, so
// `https://1234` is ACCEPTED and becomes `https://0.0.4.210` — a real address
// nobody typed. The R-B5 echo then writes that back into the field, so a typo
// silently becomes a different server. Measured and driven at NA-0753 STOP 1;
// the ENGINE half is FILED as ENG-0218 for a guarded engine lane and is
// deliberately NOT patched here. This gate fronts it for every user-facing
// path: Test is now the only control that reaches the address, and it reaches
// it through this gate before any probe or any write.
//
// ⚠ THE GATE MUST NOT USE `new URL()`. The webview's URL parser performs the
// SAME WHATWG IPv4 expansion this gate exists to refuse, so the authority is
// split by hand.
//
// ZERO silent normalization. The ONLY transform is prepending `https://` when
// the scheme is omitted — never `http` — and it is written into the field
// BEFORE any test runs, so the user sees what will be used.
function relayGateCheck(raw) {
  const typed = String(raw == null ? "" : raw).trim();
  const EXAMPLE = "https://relay.example.org:8443";
  if (typed === "") {
    return { ok: false, message: "Enter your relay address, for example " + EXAMPLE };
  }
  let value = typed;
  let prepended = false;
  const scheme = value.match(/^([a-zA-Z][a-zA-Z0-9+.\-]*):\/\//);
  if (!scheme) {
    value = "https://" + value;
    prepended = true;
  } else if (scheme[1].toLowerCase() === "http") {
    return { ok: false, message: "Use https:// — a plain http:// address isn't accepted for a relay." };
  } else if (scheme[1].toLowerCase() !== "https") {
    return { ok: false, message: "The address must start with https:// — for example " + EXAMPLE };
  }
  const authority = value.slice("https://".length).split(/[/?#]/)[0];
  const at = authority.lastIndexOf("@");
  const hostport = at >= 0 ? authority.slice(at + 1) : authority;
  if (hostport === "") {
    return { ok: false, value, prepended, message: "Enter your relay address, for example " + EXAMPLE };
  }
  if (hostport.startsWith("[")) {
    // An IPv6 literal is outside the ruled shape; refuse rather than guess.
    return { ok: false, value, prepended, message: "Type the address your relay operator gave you, for example " + EXAMPLE };
  }
  const colon = hostport.lastIndexOf(":");
  const host = colon >= 0 ? hostport.slice(0, colon) : hostport;
  const port = colon >= 0 ? hostport.slice(colon + 1) : "";
  if (host === "") {
    return { ok: false, value, prepended, message: "Enter your relay address, for example " + EXAMPLE };
  }
  // THE INTEGER-IP TRAP. A dotless LAN name (`relaybox`) is VALID; a bare
  // number is not a server name at all.
  if (/^[0-9]+$/.test(host)) {
    return { ok: false, value, prepended, message: "That's a number, not a server name. Type the address your relay operator gave you, for example " + EXAMPLE };
  }
  if (port === "") {
    return { ok: false, value, prepended, message: "Add the port — for example :8443" };
  }
  if (!/^[0-9]+$/.test(port) || Number(port) < 1 || Number(port) > 65535) {
    return { ok: false, value, prepended, message: "That port isn't valid. Add the port — for example :8443" };
  }
  return { ok: true, value, prepended };
}

// NA-0754 (design bank v2 item 4) — THE CA-PATH GATE: `~` EXPANDED VISIBLY.
//
// Same shape as the address gate's https:// prepend and for the same reason: the
// ONLY transform is one the user can see, written into the field BEFORE the path
// is used, so nothing is silently sent at a path nobody typed.
//
// ⚠ THE EXPANSION NEEDS A PROCESS FACT THE WEBVIEW DOES NOT HAVE. `~` is not a
// filesystem entry; it is shell notation for $HOME, and JS in the webview cannot
// read the environment. `homeDir` is fetched from the `home_dir` command at pane
// open. If it is empty — HOME unset, or the fetch failed — the gate REFUSES `~`
// instead of guessing, because guessing here means probing a path the user never
// typed and then persisting it on success.
//
// Everything else that is shell notation rather than a path is refused with one
// message: `~user` (which needs a passwd lookup, not $HOME), $VAR and ${VAR},
// globs, and command substitution. qsc opens the path with fs::metadata, which
// expands none of them, so an unexpanded token would fail later as a confusing
// "no file at that path" instead of the truth.
function caPathGateCheck(raw) {
  const typed = String(raw == null ? "" : raw).trim();
  const EXAMPLE = "Use the full path — for example /home/you/ca.pem";
  if (typed === "") {
    return { ok: true, value: "", expanded: false };
  }
  let value = typed;
  let expanded = false;
  if (value === "~" || value.startsWith("~/")) {
    if (homeDir === "") {
      return { ok: false, value: typed, expanded: false, message: EXAMPLE };
    }
    const rest = value === "~" ? "" : value.slice(1);
    value = homeDir.replace(/\/+$/, "") + (rest === "" ? "" : rest);
    expanded = true;
  }
  if (/^~/.test(value) || /[$`*?]/.test(value)) {
    return { ok: false, value: typed, expanded: false, message: EXAMPLE };
  }
  return { ok: true, value, expanded };
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
      if (d.version) doc.appendChild(docRow("Relay version", d.version));
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
        "This relay presented a certificate your computer doesn't recognise. That's expected if the operator runs their own certificate authority — and it's also what an interception attack looks like. Ask the operator for their CA certificate and add it above, or install it on this computer.";
      break;
    case "unreachable":
      setBanner(status, "accent", "Couldn't reach the relay");
      detail.textContent =
        "Nothing answered at that address. Check the address, and check you're on the same network or VPN as the relay. " +
        "If your relay operator uses a non-standard port, include it — for example https://relay.example.org:8443.";
      break;
    case "not_a_qsl_relay":
      setBanner(status, "accent", "Not a QSL relay");
      detail.textContent = "Something answered, but it isn't a QSL relay. Check the address.";
      break;
  }
  // C6 (R-E6): a Test that COMMITTED says so — the commit is otherwise silent,
  // and a helper merely disappearing is absence-of-signal, not confirmation.
  //
  // NA-0754: and a test that did NOT commit says THAT, which is the half the old
  // pane could not say because it had already written. The red sentence is the
  // design bank's own verbatim direction, and it is the user-facing statement of
  // the invariant: a failed rung persists nothing, so what you had still stands.
  if (committed) {
    detail.textContent = (detail.textContent ? detail.textContent + " " : "") + "Settings saved.";
  } else {
    detail.textContent = (detail.textContent ? detail.textContent + " " : "") +
      "Nothing saved — your previous settings are unchanged.";
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
    byId("relay-url-error").textContent = "Enter a valid relay address, for example https://relay.example.org:8443";
    clearServerResults();
    return;
  }
  byId("relay-results").classList.remove("hidden");
  byId("relay-doc").innerHTML = "";
  if (RELAY_CA_CODES.some((c) => codeStr.includes(c))) {
    // NA-0754 (design bank v2 item 3): the failure must NAME WHICH CHECK FAILED.
    //
    // ⚠ THIS BRANCH IS WHERE THE CA IS NOW ACTUALLY VALIDATED, which is why the
    // three specific strings live here. Under the old model the CA path was
    // checked by being WRITTEN, so the specific sentences sat on the save path;
    // under test-and-save-on-proof nothing is written until the probe is green,
    // and the probe's own Err channel carries the verdict — `relay_http_client`
    // reads and PEM-parses the file before any socket is opened. Leaving the
    // sentences on the save path would have meant the pane naming the check on
    // the one route that no longer runs it, and saying only "couldn't be read"
    // on the route that does. Measured: scenario f_j caught exactly that.
    byId("relay-ca-error").textContent = mapErr(codeStr, {
      relay_ca_file_missing: "No file at that path.",
      relay_ca_file_unreadable: "That file can't be read.",
      relay_ca_file_invalid: "That file isn't a certificate.",
    });
    byId("relay-ca-path").classList.add("invalid");
    setBanner(byId("relay-status"), "accent", "Certificate authority file couldn't be read");
    byId("relay-detail").textContent =
      "The certificate authority file couldn't be read. Check the path under “Certificate authority” above — this is a local file problem, not a problem with the relay's certificate. Nothing saved — your previous settings are unchanged.";
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
  // NA-0763: the OTHER way the relay can appear or vanish (the Server pane's
  // commit path). Keeping the gate truthful here means a user who configures a
  // relay gets a live app without re-locking.
  tickRelayConfigured = savedRelayUrl !== "";
  try {
    const s = await invoke("relay_token_show");
    tokenConfigured = !!s.configured;
  } catch (_) { tokenConfigured = false; }
  try {
    const s = await invoke("relay_ca_file_show");
    caConfigured = !!s.configured;
    caPathHash = s.path_hash || "";
  } catch (_) { caConfigured = false; caPathHash = ""; }
  renderFieldState();
  renderDirty();
}

// State 9 (idle): the results panel is hidden on pane open. R-B6: a dirty pane
// left by navigating away is discarded silently — an unsaved relay address is
// inconvenience-class loss, and severity discipline gives that a helper, not a
// modal.
async function refreshServerPane() {
  // The ~ expansion needs $HOME and the webview cannot read it; fetched once per
  // pane open. On failure it stays "" and caPathGateCheck refuses `~` rather than
  // guessing at a path the user never typed.
  try { homeDir = await invoke("home_dir"); } catch (_) { homeDir = ""; }
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
      byId("relay-ca-path").classList.remove("invalid");
    }
    onServerChanged();
  });
}

// NA-0754 — TEST-AND-SAVE-ON-PROOF (design bank v2 item 1). ONE BUTTON, ONE RULE.
//
// The order is the whole point: GATE the two fields, PROBE with what was typed,
// and persist ONLY on a Connected result. A test that fails ANY rung —
// unreachable, certificate, token, CA — persists NOTHING, so the previous
// working configuration is untouched. That is the invariant in the record and in
// the pane's own copy: WHAT IS PERSISTED HAS CONNECTED AT LEAST ONCE.
//
// ⚠ WHAT THIS REPLACES, AND THE DEFECT IT KILLS. The old handler committed
// everything FIRST and then probed the just-saved state, because nothing could be
// validated without being written. So typing a broken address over a working one
// and pressing Test persisted the broken one and then reported that it did not
// work — a failed test clobbering a proven-good config. The operator met exactly
// that in flight. Inverting the order removes the class structurally rather than
// warning about it.
//
// ⚠ ACCEPTED COST, in the open (bank §1): there is no offline pre-configuration.
// A relay you cannot currently reach cannot be saved.
byId("btn-relay-test").addEventListener("click", async () => {
  if (serverBusy) return;
  byId("relay-url-error").textContent = "";
  byId("relay-url").classList.remove("invalid");
  byId("relay-ca-error").textContent = "";
  byId("relay-ca-path").classList.remove("invalid");

  // (1) THE ADDRESS GATE — before any invoke, so a refusal fires no probe and
  //     leaves the field byte-identical to what was typed.
  const typedUrl = byId("relay-url").value.trim();
  const useTypedUrl = typedUrl !== "" && typedUrl !== savedRelayUrl;
  let address = savedRelayUrl;
  if (useTypedUrl || savedRelayUrl === "") {
    const gate = relayGateCheck(byId("relay-url").value);
    if (gate.prepended) byId("relay-url").value = gate.value; // VISIBLE before any test.
    if (!gate.ok) {
      byId("relay-url").classList.add("invalid");
      byId("relay-url-error").textContent = gate.message;
      clearServerResults();
      return;
    }
    address = gate.value;
  }

  // (2) THE CA GATE — the ~ expansion is written back BEFORE the path is used.
  const caGate = caPathGateCheck(byId("relay-ca-path").value);
  if (caGate.expanded) byId("relay-ca-path").value = caGate.value; // VISIBLE before any test.
  if (!caGate.ok) {
    byId("relay-ca-path").classList.add("invalid");
    byId("relay-ca-error").textContent = caGate.message;
    clearServerResults();
    return;
  }

  // (3) THE PROBE, with the TYPED values and nothing persisted. `null` means
  //     "use whatever is stored" — R-B3's blank-means-keep, unchanged.
  const typedToken = byId("relay-token").value;
  const proven = {
    address: address !== savedRelayUrl ? address : null,
    token: typedToken !== "" ? typedToken : null,
    caPath: caGate.value !== "" ? caGate.value : null,
  };

  setServerBusy(true, "Testing…");
  try {
    const res = await invoke("relay_probe", {
      address,
      token: proven.token,
      caPath: proven.caPath,
    });

    // (4) PERSIST ONLY ON PROOF. Connected is the ONLY accepting outcome; every
    //     other variant is a rung that failed, and a failed rung saves nothing.
    if (res && res.kind === "reachable") {
      const fail = await persistProvenSettings(proven);
      if (fail) {
        handleFailedCommit(fail);
        return;
      }
      byId("relay-token").value = "";
      byId("relay-ca-path").value = "";
      await refreshServerState();
      byId("relay-url").value = savedRelayUrl; // R-B5: the NORMALIZED form.
      renderDirty(); // MUST follow the echo — see renderDirty()'s ORDERING note.
      renderServerOutcome(res, true);
    } else {
      renderServerOutcome(res, false);
    }
  } catch (e) {
    renderServerError(String(e));
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
    // the protection working, not a failure. The helper resizes: this write
    // lands AFTER show(), so the surface-change sync already ran against an
    // EMPTY feedback line.
    setUnlockFeedback("locked-notice", "Locked after inactivity.");
  }
}, 5000);

// ---- NA-0763 (`D-0040`; spine `D-1404`): THE LIVENESS TICK -----------------
// Rung 1 of the delivery ladder. While UNLOCKED and with a relay configured, a
// JITTERED background beat feeds the same one handler the unlock and
// surface-open triggers use, so an approval or an invite-finish lands on the
// other machine with zero human nudges.
//
// ⚠⚠ THE SHAPE IS THE AUTOLOCK'S ABOVE, DELIBERATELY — a short checker that
// fires when a due-time passes, rather than one standing interval OF LENGTH B.
// Two load-bearing reasons:
//   1. A fixed-period interval CANNOT re-jitter per beat. Jitter
//      is a DESIGN requirement and not polish: a fixed poll rhythm is a traffic
//      signature, the same reasoning that refused the auto-established message.
//   2. It makes "locked = no ticks" STRUCTURAL instead of a cleanup obligation.
//      There is no timer to clear on lock — the gate simply refuses and the
//      schedule resets — so a lock path written years from now cannot forget it.
//      The vault gates everything; probe results gate nothing.
//
// ⚠ THE NUMBERS ARE OPERATOR-BLESSED (2026-08-26, verbatim: "I recommend
// Instant to."), ruled at R10. They are not this file's to change.
const TICK_TEMPO = {
  instant:   { b:  20000, j:  5000 },
  private:   { b: 300000, j: 90000 },
  // Ladder 1.4, verbatim: "pull-only = the private tempo". At rung 1 everything
  // IS pull, so this is a real stored position that changes nothing until a
  // socket rung exists — plumbed now precisely so no migration is needed later.
  pull_only: { b: 300000, j: 90000 },
};
const TICK_DEFAULT = "instant";       // the blessed default, held in code
const TICK_CHECK_MS = 250;            // the checker's period => beat granularity
const TICK_BACKOFF_CEIL_MS = 900000;  // R7: 900 s
const TICK_FAIL_THRESHOLD = 3;        // R7: consecutive scan failures before we speak
const TICK_UNREACHABLE_COPY =
  "Can't reach the relay — retrying. Messages will arrive when it's back.";

let tickTempo = TICK_DEFAULT;
let tickOverrideMs = null;   // the TEST-ONLY seam; null in every ordinary run
let tickRelayConfigured = false;
let tickNextDueAt = null;
let tickCount = 0;
let tickFails = 0;

// The beat: B doubled per consecutive failure to a ceiling, then jittered.
// At `tickFails === 0` this is exactly B ± J, so backoff and the ordinary beat
// are ONE code path and the jitter cannot be lost on the failure branch (R7:
// "with the same per-beat jitter").
function tickIntervalMs() {
  // ⚠ The seam is deliberately UN-jittered so an instrument is deterministic.
  // That is an honest bound and it is stated in the records: the harness proves
  // the tick FIRES and is GATED; the jitter DISTRIBUTION is not harness-proven.
  if (tickOverrideMs !== null) return tickOverrideMs;
  const t = TICK_TEMPO[tickTempo] || TICK_TEMPO[TICK_DEFAULT];
  const grown = Math.min(t.b * Math.pow(2, tickFails), TICK_BACKOFF_CEIL_MS);
  const jitter = (Math.random() * 2 - 1) * t.j;   // uniform [-J, +J], redrawn EVERY beat
  return Math.max(1000, Math.round(grown + jitter));
}

// R10: unlocked AND a relay configured. `currentScreen` is assigned in exactly
// one place (`show`, :105) and the unlocked set is exactly these two of seven
// screens — the same gate the autolock uses, not a second opinion about it.
function tickGateOpen() {
  const unlocked = currentScreen === "scr-main" || currentScreen === "scr-settings";
  return unlocked && tickRelayConfigured;
}

// R2: the threshold status rides the R-12/F1 QUIET LINE and never
// `statusFooterLine` — that footer is NA-0752's ruled TWO-SOURCE truth line and
// a third source would re-open a settled ruling. `accent`, not `danger`: an
// unreachable relay needs attention, but the danger tier stays reserved.
// ⚠ `setStatusLine` rewrites `className` wholesale, so the hidden state is
// re-applied HERE, in the same helper — one place, never a habit re-derived at
// each call site.
function renderTickStatus() {
  const el = byId("tick-status");
  if (!el) return;
  if (tickFails >= TICK_FAIL_THRESHOLD) {
    setStatusLine(el, "accent", TICK_UNREACHABLE_COPY);
    el.classList.remove("hidden");
  } else {
    setStatusLine(el, "neutral", "");
    el.classList.add("hidden");
  }
}

// The tick's OWN observable counter (R6). Never touches `#redeem-overlay`.
function tickMark() {
  const el = byId("tick-status");
  if (!el) return;
  el.dataset.tickCount = String(tickCount);
  el.dataset.tickFails = String(tickFails);
  el.dataset.tickGate = tickGateOpen() ? "open" : "closed";
  el.dataset.scanBusyRejects = String(relayScanBusyRejects);
  el.dataset.scanRerun = String(relayScanRerunCount);
}

setInterval(async () => {
  if (!tickGateOpen()) {
    // Locked, or no relay: no beat, and the schedule RESETS so an unlock does
    // not inherit a due-time computed in a previous session.
    tickNextDueAt = null;
    return;
  }
  const now = Date.now();
  if (tickNextDueAt === null) {
    tickNextDueAt = now + tickIntervalMs();   // arm on the first eligible pass
    return;
  }
  if (now < tickNextDueAt) return;
  tickNextDueAt = now + tickIntervalMs();
  await relayScan({ source: "tick", at: now });
}, TICK_CHECK_MS);

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
    vault_version_unsupported:
      "This vault was created by an older version of the app, so this one can't open or destroy it. Nothing was destroyed.",
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

// ---- NA-0755 v2 (D-0036): THE INVITE SURFACE — THE SINGLE-VIEW MINT AND THE LIST ----
//
// Supersedes the v1 two-step modal, which the operator flew and which came back RED.
//
// ⚠ ZERO new commands beyond `invite_clear`. The other six invite verbs were already
// registered at NA-0751.

let inviteId = null;            // the DISPLAYED invite, for Cancel
let inviteRows = [];            // the last list read, for the list view
let inviteCopyTimer = null;

// The TTL this app requests. 259200 = 72h = the `qsc` CLI's own `default_value_t`, adopted
// rather than invented so both front ends ask for the same thing.
//
// ⚠ IT IS A REQUEST. `resolve_expiry` clamps it to the relay's advertised ceiling and
// subtracts a 300s skew margin, and a clamp is a NORMAL outcome. That is why every expiry the
// user sees is READ BACK from `invite_list`, never printed from this constant.
const INVITE_TTL_SECS = 259200;
const INVITE_SOFT_CAP = 10;

// ⚠⚠ THE CLIPBOARD, AND WHY IT IS SHAPED LIKE THIS — MEASURED, NOT ASSUMED.
//
// The design assumed a "~4 s user-activation timeout". Measured in this webview:
//     plain `await` then writeText  ->  RESOLVES at 750ms, REJECTS at 1000ms
// A create needs two network round-trips, so on that route "Activate & Copy" would have failed
// EVERY time, not occasionally. But:
//     ClipboardItem built SYNCHRONOUSLY around a pending promise -> RESOLVES at 4000ms
// So the single gesture is buildable: the item is constructed in the click handler, and its
// promise resolves to the code when `invite_create` returns.
//
// ⚠ The fallback is a CAPABILITY TEST, never a timeout guess. Where `ClipboardItem` is absent
// the button reads "Activate" and the copy glyph — its own gesture, always valid — is the
// recovery. The label must never promise what the platform refuses.
//
// ⚠ On a create FAILURE the promise rejects, the write rejects, and NOTHING is copied. That is
// correct: a code that does not exist must not reach the clipboard.
const HAS_CLIPBOARD_ITEM = typeof ClipboardItem !== "undefined" && !!(navigator.clipboard && navigator.clipboard.write);

// R380 §2 / §5 + R381 §1 — the ruled mapping from a typed facade failure onto modal copy.
// PURE and TOTAL: (code, detail) -> {banner, detail}. Severity is ACCENT, never red.
const INVITE_ENDPOINT_DETAILS = [
  "relay_endpoint_missing", "relay_endpoint_invalid",
  "relay_endpoint_invalid_host", "relay_endpoint_invalid_scheme",
];
function inviteErrorLine(code, detail, verb) {
  const c = String(code || "");
  const d = String(detail || "");
  // ── NA-0756 (D-0037, R387 §S2c) — THE REDEEM PATH'S VERB-CONDITIONAL ROWS. ──
  // THREE shipped rows state something FALSE on a redeem, so they are reworded HERE and
  // ONLY here; create's ruled copy below is untouched, which is what "verb-CONDITIONAL"
  // means. The falsehood is in the DETAIL for two of them and in BOTH halves for the third:
  //   relay_rejected  its banner names the CREATE verb outright, so the banner moves to the
  //                   ruled redeem one; nothing is invented, both halves are ruled strings
  //   not_found       "This app has no record" is backwards — on a redeem it is THEIR relay
  //                   that has no record; the banner stays true and only the detail moves
  //   endpoint family the "Settings → Relay" pointer is wrong where the address came from
  //                   the PASTED CODE, not from local configuration; banner stays true
  // ⚠ The endpoint test reuses the shipped BOTH-POSITIONS guard verbatim: the same string is
  // a `code` for one input and a `detail` for another (ENG-0228), and position is the only
  // thing that separates them.
  if (verb === "redeem") {
    if (c === "relay_rejected") {
      return { banner: "Couldn't add the contact",
        detail: "Nothing was added — their relay couldn't be reached, or it refused the request. Check the code is complete, and try again in a moment." };
    }
    if (c === "not_found") {
      return { banner: "That invite is already gone",
        detail: "Their relay has no record of this invite. It may have been revoked or already cleaned up — ask them for a fresh one." };
    }
    if (INVITE_ENDPOINT_DETAILS.includes(c) || (c === "other" && INVITE_ENDPOINT_DETAILS.includes(d))) {
      return { banner: "Relay address isn't usable",
        detail: "This code's relay address isn't usable, so the code may be damaged or incomplete. Ask them to send it again." };
    }
  }
  if (c === "relay_tls_untrusted") {
    return { banner: "Certificate not trusted",
      detail: "This relay presented a certificate your computer doesn't recognise. That's expected if the operator runs their own certificate authority — and it's also what an interception attack looks like. Add their CA certificate under Settings → Relay." };
  }
  if (c === "relay_ca_file") {
    return { banner: "Certificate authority file couldn't be read",
      detail: "The certificate authority file couldn't be read. Check the path under Settings → Relay — this is a local file problem, not a problem with the relay's certificate." };
  }
  if (c === "relay_unauthorized") {
    return { banner: "Token rejected",
      detail: "The relay requires an access token and didn't accept the one this app sent. Check it with the operator, then set it under Settings → Relay." };
  }
  if (INVITE_ENDPOINT_DETAILS.includes(c) || (c === "other" && INVITE_ENDPOINT_DETAILS.includes(d))) {
    return { banner: "Relay address isn't usable",
      detail: "Set a valid relay address under Settings → Relay, then try again." };
  }
  if (c === "soft_cap_reached") {
    return { banner: "Too many live invites",
      detail: "You already have 10 invites waiting. Revoke one you no longer need, then create another." };
  }
  if (c === "rate_limited") {
    return { banner: "The relay is rate-limiting",
      detail: "The relay asked this app to slow down. Wait a moment and try again." };
  }
  if (c === "relay_slots_full") {
    return { banner: "The relay has no invite slots free",
      detail: "The relay's invite table is full. Ask the operator, or try again later." };
  }
  if (c === "relay_rejected") {
    // R380 §2(A): BOTH provenances named, because the client cannot tell them apart — every
    // non-TLS send failure returns the caller's own fallback, so an unreachable relay and a
    // relay that refused arrive as the SAME code. Measured, 4112ms.
    return { banner: "The relay didn't create the invite",
      detail: "Nothing was created — the relay couldn't be reached, or it refused the request. Check Settings → Relay; its Test connection button can tell which." };
  }
  if (c === "not_found") {
    return { banner: "That invite is already gone",
      detail: "This app has no record of that invite any more." };
  }
  if (c === "revoke_invalid") {
    return { banner: "Couldn't revoke this invite",
      detail: "This invite has no revoke token stored, so it can't be revoked from here." };
  }
  if (c === "clear_refused") {
    // The SHORT wire discriminant. The engine const is `invite_clear_refused`; zero of the
    // other wire codes carry an `invite_` prefix and this one does not either.
    return { banner: "That invite can't be removed",
      detail: "Only an invite that didn't finish can be removed from the list. A live invite is revoked, not removed." };
  }
  if (c === "locked") {
    return { banner: "Vault is locked", detail: "Unlock to continue." };
  }
  if (c === "vault_unavailable") {
    // ⚠ MUST NOT say "unlock it": this code carries THREE provenances — locked mid-operation,
    // vault DAMAGE, or a key-source failure — and the copy must be true in all three.
    //
    // R381 §1: the arm is now SELF-DIAGNOSING. `detail` carries the underlying source code, and
    // it rides as a SUBDUED PARENTHETICAL so the sentence stays plain while the screenshot
    // names its own provenance. The payload is closed at seven static tokens — no user bytes.
    const suffix = d ? ` (${d})` : "";
    return { banner: "The vault couldn't be read",
      detail: `The vault couldn't be read. If this keeps happening, check Settings → Vault.${suffix}` };
  }
  // ── NA-0756 (D-0037, R387 §S2b) — SIX ARMS THAT HAD NO COPY AT ALL. ──
  // Measured at STOP 002: 21 of the 35 redeem-reachable wire codes fell through to the
  // residual, which rendered a raw engine token under a banner naming the wrong verb. These
  // six are the ORDINARY, EXPECTED outcomes of pasting a code — `malformed` alone is the
  // likeliest failure in the whole flow, since it is what a truncated paste produces — and
  // they are ruled here for EVERY invite verb, because a total map stays total.
  // ⚠ `malformed` is DUAL-PROVENANCE (local parse at invite/mod.rs:435-442, and the relay's
  // own ERR_INVITE_BAD_BODY at transport:4195). The facade cannot separate them, so the copy
  // must be true of both: it names the code, never the party.
  if (c === "malformed") {
    return { banner: "That code isn't readable",
      detail: "Check the whole code was pasted — it starts with QSLI-1-. If it looks complete, ask them to send it again." };
  }
  if (c === "expired") {
    return { banner: "This invite has expired",
      detail: "Invite codes only last a few days. Ask them for a fresh one." };
  }
  // ⚠ DELIBERATELY DISTINCT from `expired`: the relay clamps expiry against ITS clock, so a
  // local "alive" against a relay "expired" is a NORMAL outcome, and collapsing the two would
  // blame the user's clock for a relay ceiling (invite/mod.rs:113-116).
  if (c === "expired_at_relay") {
    return { banner: "This invite has expired",
      detail: "Their relay reports this invite expired. Ask them for a fresh one." };
  }
  // ⚠ `already_used` is the RELAY's claim; `already_redeemed` is THIS client's own record —
  // the arm that survives a hostile relay. Two different facts, two different sentences.
  if (c === "already_used") {
    return { banner: "This code has already been used",
      detail: "Their relay reports this invite was already used. If that wasn't you, ask them for a fresh invite through a channel you trust." };
  }
  if (c === "already_redeemed") {
    return { banner: "You've already used this code",
      detail: "This device already accepted this invite. If the contact hasn't appeared yet, they may not have approved it on their side." };
  }
  if (c === "revoked") {
    return { banner: "This invite was cancelled",
      detail: "The person who created it revoked this invite. Ask them for a new one." };
  }
  // ── PREPARED FOR LANE B BY R380 §5, AND LANE B IS THE LANE THAT REACHES THEM. ──
  // Both are produced only inside `verify_redeemed_bundle`, so NA-0755 could not seal them —
  // "a seal that cannot fail is not a seal". NA-0756 drives both from the facade, so they are
  // sealable at last. The strings below are UNCHANGED from R380 §5; the redeem flow ROUTES
  // these two codes to the dedicated security-failure STATE rather than this inline box
  // (R387 §S2a), and a total map stays total, so the arms remain here for every other caller.
  if (c === "commitment_mismatch") {
    return { banner: "This invite's keys don't match",
      detail: "The keys in this invite don't match what it commits to. Someone may be interfering. Ask the person who sent it for a new invite through a different channel." };
  }
  if (c === "signature_invalid") {
    return { banner: "This invite has been altered",
      detail: "This invite's signature doesn't check out, so its contents have been changed since it was made. Someone may be interfering. Ask for a new invite through a different channel." };
  }
  const shown = c === "other" && d ? d : c || "unknown";
  return { banner: verb === "revoke" ? "Couldn't revoke the invite"
         : verb === "clear" ? "Couldn't remove the invite"
         // NA-0756 (R387 §S2c): without this arm a redeem fell through to the CREATE string.
         : verb === "redeem" ? "Couldn't add the contact"
         : "Couldn't create the invite",
    detail: "The relay or this app reported: " + shown };
}

function renderInviteError(boxId, code, detail, verb) {
  const box = byId(boxId);
  const line = inviteErrorLine(code, detail, verb);
  setBanner(box.querySelector(".status-banner"), "accent", line.banner);
  box.querySelector(".hint").textContent = line.detail;
  box.classList.remove("hidden");
}
function clearInviteErrors() {
  for (const id of ["invite-error-mint", "invite-error-post", "invite-error-list"]) {
    byId(id).classList.add("hidden");
  }
}

// ⚠ The one-time boundary's local half: the code lives in the DOM node and nowhere else. No
// module variable holds it, nothing writes it to settings, and closing removes it. `inviteId`
// is NOT the code — it is the public slot id the list already publishes.
function closeInviteModal() {
  const ov = byId("invite-overlay");
  if (!ov || ov.classList.contains("hidden")) return;
  ov.classList.add("hidden");
  byId("invite-code").textContent = "";
  byId("invite-label").value = "";
  inviteId = null;
  clearInviteErrors();
  inviteShowMint();
}

function inviteShowMint() {
  byId("invite-mint").classList.remove("hidden");
  byId("invite-list-view").classList.add("hidden");
  byId("invite-pre").classList.remove("hidden");
  byId("invite-post").classList.add("hidden");
}

// ⚠⚠ v3 — ENTERING THE MINT FRESH. The operator's flight found that a typed label SILENTLY RODE
// EVERY LATER MINT: the field was cleared when the surface opened but not when the user came
// back to it from the list, so the second invite inherited the first invite's note without
// anyone touching the box. A silent wrong value is worse than a visible one.
//
// ⚠ ONE function owns this, and BOTH paths into the mint call it, so the two cannot drift apart
// again — which is how the defect existed in the first place.
function inviteEnterMintFresh() {
  byId("invite-label").value = "";
  byId("invite-code").textContent = "";
  byId("invite-meta-note").textContent = "Invite code";
  byId("invite-meta-expiry").textContent = "";
  byId("invite-copy-note").classList.add("hidden");
  inviteId = null;
  inviteShowMint();
}
function inviteShowPost() {
  byId("invite-pre").classList.add("hidden");
  byId("invite-post").classList.remove("hidden");
}
function inviteShowList() {
  byId("invite-mint").classList.add("hidden");
  byId("invite-list-view").classList.remove("hidden");
}

function humanWhen(unixSecs) {
  // F-3: `created` is an Option at the DTO precisely so the 1970 sentinel cannot arrive here.
  if (!unixSecs) return "—";
  return new Date(unixSecs * 1000).toLocaleDateString();
}

// Refresh-on-open (the bank's decision 1), never polling. A background check is FILED.
async function inviteRefresh() {
  try {
    inviteRows = await invoke("invite_list");
  } catch (_) {
    inviteRows = [];
  }
  const live = inviteRows.filter((r) => r.state === "active").length;
  byId("invite-count").textContent = String(live);
  byId("invite-slots").textContent = `${live} of ${INVITE_SOFT_CAP} slots used — codes expire on their own`;
  let relayUrl = "";
  try {
    const cfg = await invoke("relay_config_get");
    relayUrl = cfg.relay_url || "";
  } catch (_) { relayUrl = "" ; }
  const noRelay = relayUrl === "";
  const capFull = live >= INVITE_SOFT_CAP;
  // An enabled button whose only outcome is an error is the control-that-cannot-succeed shape.
  byId("btn-invite-activate").disabled = noRelay || capFull;
  byId("invite-no-relay").classList.toggle("hidden", !noRelay);
  byId("invite-cap-full").classList.toggle("hidden", !capFull || noRelay);
  return relayUrl;
}

async function openInviteModal() {
  clearInviteErrors();
  inviteEnterMintFresh();
  byId("btn-invite-activate").textContent = HAS_CLIPBOARD_ITEM ? "Activate & Copy" : "Activate";
  byId("invite-overlay").classList.remove("hidden");
  await inviteRefresh();
}

// `invite_create` returns the CODE, not the id Cancel needs, so the id is recovered by
// COMPOSITION from an `invite_list` diff — the same call that carries the REAL expiry.
// ⚠⚠ v4 — ONE SOURCE, TWO DISPLAYS. The meta row's expiry and the warning's closing figure
// are the SAME invite's remaining life, computed ONCE here and written to both. They are two
// renderings of one fact and must never be able to disagree: a warning that says "3 days" over
// a code the meta row says expires in 2 is a lie the user has no way to resolve.
//
// ⚠ The value is READ BACK from the invite, never printed from the TTL we requested — the
// relay clamps that, and a clamp is a normal outcome.
function inviteWriteExpiry(secondsLeft) {
  const human = secondsLeft > 0 ? humanDuration(secondsLeft) : "—";
  byId("invite-meta-expiry").textContent = secondsLeft > 0 ? "Expires in " + human : "";
  byId("invite-warn-days").textContent = human;
}

async function adoptMinted(before) {
  inviteId = null;
  byId("invite-meta-expiry").textContent = "";
  byId("invite-meta-note").textContent = "Invite code";
  try {
    inviteRows = await invoke("invite_list");
  } catch (_) {
    return;
  }
  const fresh = inviteRows.filter((r) => !before.includes(r.invite_id));
  if (fresh.length !== 1) return;      // ambiguous -> claim nothing
  const row = fresh[0];
  inviteId = row.invite_id;
  inviteWriteExpiry(row.expiry - Math.floor(Date.now() / 1000));
  // LEFT of the meta row names what this is: the note when one was given, else plain.
  byId("invite-meta-note").textContent = row.label ? "Invite for: " + row.label : "Invite code";
}

byId("btn-invite-activate").addEventListener("click", async (ev) => {
  clearInviteErrors();
  const label = byId("invite-label").value.trim();
  const btn = byId("btn-invite-activate");
  btn.disabled = true;

  let relayUrl = "";
  try {
    const cfg = await invoke("relay_config_get");
    relayUrl = cfg.relay_url || "";
  } catch (_) { relayUrl = ""; }
  let before = [];
  try { before = (await invoke("invite_list")).map((r) => r.invite_id); } catch (_) { before = []; }

  const mint = invoke("invite_create", {
    selfLabel: null, relay: relayUrl, ttlSecs: INVITE_TTL_SECS,
    recipientLabel: label === "" ? null : label,
  });

  let copied = false;
  if (HAS_CLIPBOARD_ITEM) {
    // ⚠ THE ITEM IS BUILT SYNCHRONOUSLY, INSIDE THE GESTURE. That is the whole mechanism: the
    // promise resolves later, when the create returns, and the write still succeeds.
    try {
      const item = new ClipboardItem({
        "text/plain": mint.then((code) => new Blob([code], { type: "text/plain" })),
      });
      await navigator.clipboard.write([item]);
      copied = true;
    } catch (_) {
      copied = false;   // a create failure rejects the promise; nothing is copied. Correct.
    }
  }

  let code;
  try {
    code = await mint;
  } catch (e) {
    btn.disabled = false;
    renderInviteError("invite-error-mint", e && e.code, e && e.detail, "create");
    await inviteRefresh();
    return;
  }
  byId("invite-code").textContent = code;
  // v3: on success the link itself is the affordance and no note is needed; the note exists
  // only to say when the single gesture could NOT copy, and it points at the link.
  const note = byId("invite-copy-note");
  note.textContent = copied ? "" : "Copy didn't complete — use the copy code link below.";
  note.classList.toggle("hidden", copied);
  inviteShowPost();
  await adoptMinted(before);
  await inviteRefresh();
});

// The copy glyph: its own click is its own gesture, so it always works — which is why it is the
// recovery path and why "Copy again" is not a button.
// v3: ONE text link is both the re-copy control and the fallback's recovery path. Its own
// click is its own gesture, which is precisely why it works when the single-gesture write did
// not — the user activation is fresh here and is never spent on an await.
async function inviteCopyLink() {
  const code = byId("invite-code").textContent;
  if (!code) return;
  const link = byId("btn-invite-copy");
  try {
    await navigator.clipboard.writeText(code);
    link.textContent = "copied";
    link.classList.add("copied");      // v4: green, per the ratified mockups
    if (inviteCopyTimer) clearTimeout(inviteCopyTimer);
    inviteCopyTimer = setTimeout(() => {
      link.textContent = "copy code";
      link.classList.remove("copied");
    }, 2500);
  } catch (_) {
    byId("invite-copy-note").textContent = "Copy didn't complete — select the code and copy it.";
    byId("invite-copy-note").classList.remove("hidden");
  }
}
byId("btn-invite-copy").addEventListener("click", () => inviteCopyLink());
// The precedent style (`a.rm`) is not focusable on its own; Enter/Space keep the control
// operable without a mouse.
byId("btn-invite-copy").addEventListener("keydown", (ev) => {
  if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); inviteCopyLink(); }
});

// ⚠ v4: the "Cancel Invite" handler is REMOVED with its button. The single kill mechanism is
// Revoke in the list — one word, one place. Mid-mint regret is Review invites → Revoke, one
// extra click for a rare case, and that trade is recorded rather than lost.

// ── the list view — THE REFERENCE MARKUP GOVERNS (v3) ─────────────────────
//
// ⚠⚠ VISIBILITY RULE: LIVE rows and FAILED rows only, NEWEST FIRST. Revoked and Expired
// records DO NOT RENDER — ever. They are inert, sealed and never counted; true vault deletion
// of dead records is the queued engine-hygiene lane, not this surface's job. The list answers
// "what is open", and an expired invite's answer is its absence.
//
// ⚠ NO HISTORY VIEW, deliberately (bank §5). The remedy for an expired invite is a fresh mint.
function inviteVisibleRows() {
  const now = Math.floor(Date.now() / 1000);
  return inviteRows
    .filter((r) => {
      if (r.state === "creating") return true;                 // FAILED row
      if (r.state === "redeemed") return true;                  // ACCEPTED row (interim)
      if (r.state === "active") return r.expiry > now;          // LIVE row
      return false;                                             // revoked / expired: never
    })
    .sort((a, b) => (b.created || 0) - (a.created || 0));       // newest first
}

function inviteRowEl(r, n) {
  const now = Math.floor(Date.now() / 1000);
  const row = document.createElement("div");
  row.className = "invite-row";
  row.dataset.inviteId = r.invite_id;

  const main = document.createElement("div");
  main.className = "invite-row-main";
  const l1 = document.createElement("div");
  l1.className = "invite-row-head";
  const label = r.label ? r.label : "(no label)";
  // The FAILED row is numbered "—": it never became one of your open invites.
  l1.textContent = `${r.state === "creating" ? "—" : n} — ${label}`;
  const l2 = document.createElement("div");
  l2.className = "invite-row-meta";
  if (r.state === "creating") {
    l2.textContent = "didn't finish — if the relay registered it, that slot expires on its own and can't be revoked from here";
  } else if (r.state === "redeemed") {
    l2.textContent = `created ${humanWhen(r.created)}`;
  } else {
    l2.textContent = `created ${humanWhen(r.created)} — expires in ${humanDuration(r.expiry - now)}`;
  }
  main.append(l1, l2);

  const side = document.createElement("div");
  side.className = "invite-row-side";
  const chip = document.createElement("span");
  chip.className = "invite-chip";
  if (r.state === "creating") {
    chip.className += " dim";
    chip.textContent = "Didn't finish";
    const b = document.createElement("a");
    // ⚠ Remove stays NEUTRAL, deliberately: it clears a local row that can never become
    // actionable. Painting it danger-red would claim it destroys something at the relay, which
    // is exactly the false promise the B-1 wording fight removed.
    b.className = "rm plain"; b.setAttribute("role", "button"); b.tabIndex = 0;
    b.textContent = "Remove"; b.dataset.clear = r.invite_id;
    side.append(chip, b);
  } else if (r.state === "redeemed") {
    // ACCEPTED — interim until Lane C, whose People pane is its permanent home.
    // ⚠ NO buttons: undeletable by the operator's own rule.
    chip.textContent = "✓ Accepted";
    side.append(chip);
  } else {
    chip.textContent = "Not yet accepted";
    const b = document.createElement("a");
    // v5: plain (never underlined) and the shipped danger-LINK colour — the token minted for
    // this exact shape. Revoke is destructive and now reads that way without shouting.
    b.className = "rm plain link-danger"; b.setAttribute("role", "button"); b.tabIndex = 0;
    b.textContent = "Revoke"; b.dataset.revoke = r.invite_id;
    side.append(chip, b);
  }
  row.append(main, side);
  return row;
}

async function renderInviteList() {
  const host = byId("invite-rows");
  host.innerHTML = "";
  const vis = inviteVisibleRows();
  byId("invite-list-empty").classList.toggle("hidden", vis.length > 0);
  vis.forEach((r, i) => host.appendChild(inviteRowEl(r, i + 1)));
}

// ⚠⚠ REVOKE: FLIP IN PLACE, THEN LEAVE. On success the chip flips to "Revoked" where the user
// is looking, for ~2s, and only then does the row go and the counter free — visible success,
// then tidy. A row that simply vanished would be indistinguishable from a bug.
//
// ⚠ ON FAILURE THE ROW DOES NOT CHANGE. A revoke that did not reach the relay did not happen,
// and the UI never pretends otherwise — the honest error line renders instead, in the reused
// vocabulary. This is the half the v1 silent-close got wrong.
byId("invite-rows").addEventListener("click", async (ev) => {
  const t = ev.target;
  const rev = t.dataset && t.dataset.revoke;
  const clr = t.dataset && t.dataset.clear;
  if (!rev && !clr) return;
  clearInviteErrors();
  const row = t.closest(".invite-row");
  try {
    if (rev) {
      await invoke("invite_revoke", { inviteId: rev });
      const chip = row.querySelector(".invite-chip");
      if (chip) { chip.textContent = "Revoked"; chip.className = "invite-chip dim"; }
      t.remove();
      await new Promise((r) => setTimeout(r, 2000));
    } else {
      await invoke("invite_clear", { inviteId: clr });
    }
    await inviteRefresh();
    await renderInviteList();
  } catch (e) {
    // The row is untouched: nothing flipped, nothing left.
    renderInviteError("invite-error-list", e && e.code, e && e.detail, rev ? "revoke" : "clear");
  }
});

byId("btn-invite-review").addEventListener("click", async () => {
  clearInviteErrors();
  await inviteRefresh();
  await renderInviteList();
  inviteShowList();
});
byId("btn-invite-back").addEventListener("click", async () => {
  clearInviteErrors();
  inviteEnterMintFresh();     // ⚠ the v3 fix: this path is the one that leaked the old label
  await inviteRefresh();
});
byId("btn-invite-open").addEventListener("click", () => openRedeemChooser());
byId("btn-invite-close").addEventListener("click", () => closeInviteModal());
byId("invite-overlay").addEventListener("click", (ev) => {
  if (ev.target === byId("invite-overlay")) closeInviteModal();
});
document.addEventListener("keydown", (ev) => {
  if (ev.key === "Escape") closeInviteModal();
});

// ═══════════════════════════════════════════════════════════════════════════════════════
// NA-0756 (D-0037, R387) — INVITE LANE B: THE REDEEM FLOW
// ═══════════════════════════════════════════════════════════════════════════════════════
//
// The app's SECOND contact-making act, and the first time anything has driven
// `invite_redeem` / `invite_accept` / `invite_finish` from a GUI. All three verbs were
// already registered at NA-0751/0755, so ZERO `.rs` product bytes are touched.
//
// ⚠⚠ THE SHAPE IS FORCED BY THE ENGINE, not chosen. Measured at qsc d3fefd12:
//   · `invite_redeem(code, alias, self_label)` PROVISIONS the contact inside the call and
//     the facade has NO rename verb  ⇒  the name is collected BEFORE Connect, not after.
//     mockup-15's two-step "connect, then name them" order is superseded by this fact.
//   · The capability BURNS at invite/mod.rs:1081, the instant the relay answers, and the
//     verification that can reject it runs at :1101 — AFTER. ⇒ Connect is irreversible and
//     there is no honest Retry anywhere in this flow.
//   · `invite_finish` scans the redeemer's OWN inbox, bounded (8 pulls × 16 frames), and
//     returns found / not-yet. "Not yet" is silent and normal, never an error.

// The four views of the one overlay. `null` is the closed state.
const REDEEM_VIEWS = ["choose-view", "redeem-form", "redeem-sent", "redeem-failed"];

// ⚠ THE ENGINE'S OWN PREDICATE, MIRRORED. `channel_label_ok` (qsc lib.rs:2568-2573) admits
// non-empty AND every char in [A-Za-z0-9_#-] — NO SPACES. The alias is NOT validated anywhere
// before the burn (its only uses in `invite_redeem_at` are the parameter at :1033 and the two
// consumers at :1107/:1122), so without this mirror a name like "Ben Smith" would destroy the
// user's one-time code and then report `other`/`contacts_alias_invalid`.
// This gate is DEFENCE. The engine gap itself is filed as ENG-0236 — the front end being
// careful does not make the engine safe, and the filing is the defect's record.
const REDEEM_NAME_RE = /^[A-Za-z0-9_#-]+$/;
function redeemNameOk(v) { return REDEEM_NAME_RE.test(v); }

const REDEEM_NAME_HINT_OK = "Stored only on this device.";
const REDEEM_NAME_HINT_BAD = "Names here can use letters, numbers, and - _ # — no spaces. It stays on your device.";

function redeemShow(view) {
  for (const v of REDEEM_VIEWS) byId(v).classList.toggle("hidden", v !== view);
}

// ⚠ STRUCTURAL, not remembered. Called from `show()` (see :98's sibling call) so every screen
// transition — including the autolock — clears a pasted code rather than leaving it rendered
// over the unlock screen. The pasted code is a one-time capability and is held to the same
// rule as the minted one.
function closeRedeemModal() {
  const ov = byId("redeem-overlay");
  if (!ov || ov.classList.contains("hidden")) return;
  ov.classList.add("hidden");
  byId("redeem-code").value = "";
  byId("redeem-name").value = "";
  byId("redeem-failed-detail").textContent = "";
  redeemLastDetail = "";
  redeemClearError();
  redeemSyncConnect();
  redeemShow("choose-view");
}

function redeemClearError() { byId("redeem-error").classList.add("hidden"); }

function redeemRenderError(code, detail) {
  const line = inviteErrorLine(code, detail, "redeem");
  const box = byId("redeem-error");
  setBanner(box.querySelector(".status-banner"), "accent", line.banner);
  box.querySelector(".hint").textContent = line.detail;
  box.classList.remove("hidden");
}

// ⚠ THE SINGLE-COMMIT GATE. Connect arms only when the code is non-empty AND the name is
// admissible to the engine's own set — R387 §S3 amended Z3 from "non-empty" to this. The hint
// slot carries the standing sentence while the field is empty or valid, and names the
// constraint in the user's words while it is not.
function redeemSyncConnect() {
  const code = byId("redeem-code").value.trim();
  const name = byId("redeem-name").value.trim();
  const nameOk = redeemNameOk(name);
  byId("btn-redeem-connect").disabled = !(code !== "" && nameOk);
  byId("redeem-name-hint").textContent =
    (name !== "" && !nameOk) ? REDEEM_NAME_HINT_BAD : REDEEM_NAME_HINT_OK;
}

// The two arms of the ruled security-failure state. R387 §S2a COMPOSED the two ruled texts:
// the operator's blessed callout is the constant in the markup, and this line beneath it
// carries the FIRST SENTENCE of the shipped R380 §5 copy for the SPECIFIC arm — so
// substituted KEYS stay distinguishable from altered FIELDS, which is the standing security
// principle. The remainder of those shipped sentences (interference, ask for a fresh invite)
// is already carried by the blessed callout, and duplication is noise.
const REDEEM_TELL_DETAIL = {
  commitment_mismatch: "The keys in this invite don't match what it commits to.",
  signature_invalid: "This invite's signature doesn't check out — its contents have been changed since it was made.",
};
let redeemLastDetail = "";

function redeemShowSecurityFailure(code) {
  redeemLastDetail = REDEEM_TELL_DETAIL[code] || "";
  byId("redeem-failed-detail").textContent = redeemLastDetail;
  // ⚠ The wire code rides the LOCAL diagnostic only. Nothing here is sent anywhere.
  byId("redeem-failed").dataset.arm = code;
  redeemShow("redeem-failed");
}

async function openRedeemChooser() {
  redeemClearError();
  byId("redeem-code").value = "";
  byId("redeem-name").value = "";
  redeemSyncConnect();
  redeemShow("choose-view");
  byId("redeem-overlay").classList.remove("hidden");
  // ⚠ TRIGGER (b), and it attaches HERE rather than to the create modal. R387 §S6: item 1
  // retargets BOTH entries to this chooser, so a trigger left on `openInviteModal` would fire
  // only on the create branch and the redeem branch would silently lose it.
  await relayScan({ source: "surface_open", at: Date.now() });
}

// ── THE TRIGGERS, AND THE ONE HANDLER THEY ALL FEED ───────────────────────────────────
// ⚠ NA-0763 (`D-0040`, spine `D-1404`) REWROTE THIS HEADER, and the two corrections are
// named rather than silently applied (ruling R11 / STOP 001 finding F3):
//   (i)  it said "the app's only standing interval is the idle autolock at :1685" — that
//        coordinate had already drifted past the autolock's own section, and there are now
//        TWO standing intervals: the idle autolock and this lane's tick. ⚠ This correction
//        deliberately names SECTIONS rather than line numbers: a coordinate is exactly what
//        went stale, and the whole-file timer census in `design_polish` is the instrument
//        that keeps the count honest instead;
//   (ii) it said "Once MESSAGING ships a jittered background pull, invite-finish rides that
//        same tick" — the delivery ladder sequences rung 1 BEFORE messaging, so the tick
//        arrived first and invite-finish rides it NOW. The prediction was right about the
//        shape and wrong about the order.
//
// P2, THE BANKED CONSTRAINT: every trigger — unlock, surface-open, the tick — feeds ONE
// handler that performs the SAME scan. Rungs are transport plugs; nothing above knows which
// rung is installed.
//
// ⚠ RUNG 1 INSTALLS THE FINISH-SCAN CLASS ONLY, and the bound is honest rather than
// accidental (ruled R1 on two measured grounds): there is NO handshake surface on this app
// to call — the desktop registers 44 commands and none is a handshake verb — and `ENG-0198`
// is open, where `handshake poll` returns rc 0 while completing nothing, which would defeat
// the fail-loud reporting below by never producing a failure to count. `SCAN_CLASSES` is a
// LIST so the second class is an addition, never a re-architecture.
//
// ⚠ EQUALITY, NEVER `contains`. `connect_status.state` is a CLOSED SET OF TWO — "active" |
// "inactive" (commands.rs:881-886) — and a pending contact reads "inactive"/"no_session"
// while a completed one reads "active"/"handshake". A substring test here is the 187-day
// prefix lesson waiting to happen.
//
// ⚠ `relay` IS OUR OWN RELAY, not the peer's. `invite_finish` pulls the REDEEMER's own inbox
// (`relay_inbox_pull(&relay_ep, &self_inbox, max)`, invite/mod.rs:1426), so the configured
// address is the right source — and it is the only one available, because `ContactDto`
// carries no relay endpoint at all.

// The finish-scan CLASS. Returns its own marks; it does not decide how they are surfaced —
// that is the handler's job, because the answer differs by SOURCE (see `relayScan`).
async function finishScanClass(marks) {
  let relayUrl = "";
  try {
    const cfg = await invoke("relay_config_get");
    relayUrl = cfg.relay_url || "";
  } catch (_) { relayUrl = ""; }
  if (relayUrl === "") return marks;

  let rows = [];
  try { rows = await invoke("contact_list"); } catch (_) { return marks; }

  for (const row of rows) {
    let st = null;
    try { st = await invoke("connect_status", { peer: row.alias }); } catch (_) { continue; }
    if (st.state !== "inactive") continue;   // EQUALITY on the extracted value
    marks.pending += 1;
    marks.attempted += 1;
    try {
      const done = await invoke("invite_finish", {
        selfLabel: null, alias: row.alias, relay: relayUrl, max: 1,
      });
      marks.scanned += 1;
      if (done === true) marks.finished += 1;   // EQUALITY, never truthiness
    } catch (_) {
      // ⚠ A finish failure must NEVER break the surface it rides on. Unlock still completes
      // and the chooser still opens; the contact simply stays pending, which is the honest
      // state. "Not yet" is not an error and does not land here at all — it is Ok(false).
      // ⚠ NA-0763 (ruled R7): it is now RECORDED as well as swallowed. Swallowing it was
      // right for the surface and wrong for the user — a relay that has been unreachable for
      // three consecutive scans must be SAID, and a counter is the only way to know.
      marks.scanned += 1;
      marks.failed += 1;
    }
  }
  return marks;
}

// Rung 1's class list. The handshake-poll class joins it when a GUI handshake surface exists
// and `ENG-0198` can report its own no-op; until then the omission is stated, not hidden.
const SCAN_CLASSES = [finishScanClass];

// ── THE ONE HANDLER (P2 / ladder 1.1) ─────────────────────────────────────────────────
// Single entry, event {source, at}; sources: unlock | surface_open | tick. Idempotent.
// ⚠ ONE SCAN IN FLIGHT: a trigger arriving mid-scan sets the pending slot and NEVER stacks —
// the last request wins and exactly one rerun follows.
//
// ⚠⚠ WHY THIS GUARD LIVES HERE AND NOT IN RUST, measured rather than assumed: `CoreGateway`
// IS a process-wide single-flight gate, but it serialises INDIVIDUAL CALLS and callers QUEUE
// on its mutex rather than fail. A scan is a multi-call SEQUENCE, so two overlapping scans
// would interleave their calls through that gate perfectly happily. `core_busy` is the same
// granularity and would read true during any unrelated call — a settings save, say — so it
// is the wrong instrument twice over. The scan is a unit only here.
let relayScanBusy = false;
let relayScanPending = null;
let relayScanRerunCount = 0;
let relayScanBusyRejects = 0;

async function relayScan(ev) {
  if (relayScanBusy) {
    relayScanPending = ev;          // never stacks: one rerun, last request wins
    relayScanBusyRejects += 1;
    tickMark();
    return null;
  }
  relayScanBusy = true;
  let marks = null;
  try {
    let cur = ev;
    for (;;) {
      marks = { why: cur.source, at: cur.at, scanned: 0, finished: 0, pending: 0,
                attempted: 0, failed: 0 };
      for (const cls of SCAN_CLASSES) marks = await cls(marks);
      recordScanOutcome(cur, marks);
      const next = relayScanPending;
      relayScanPending = null;
      if (!next) break;
      relayScanRerunCount += 1;
      cur = next;
    }
  } finally {
    relayScanBusy = false;
  }
  return marks;
}

// ⚠⚠ R6, REQUIRED, NOT OPTIONAL — THE MARKER SEPARATION.
// `redeemMark` is a SINGLE-SLOT, LAST-WRITE-WINS surface, and TWO committed assertions pin
// it by exact string INCLUDING `"why":"surface_open"`
// (tests/harness/scenarios/f_l_invite_redeem.json:101 and :493). A tick writing that slot
// would turn both red — and the no-relay short-circuit would not save them, because the old
// code marked even when it short-circuited. So the slot keeps LAST-USER-TRIGGER semantics
// permanently, and the tick keeps its OWN counter on `#tick-status`.
function recordScanOutcome(ev, marks) {
  if (ev.source === "tick") {
    tickCount += 1;
  } else {
    redeemMark(marks);              // user-caused triggers only — unchanged behaviour
  }
  // A scan that reached the relay and had EVERY attempt fail is a failure; one that reached
  // it and had any attempt succeed is a recovery; one that never reached it at all (nothing
  // pending, or no relay) is NEITHER and must not move the counter — otherwise an idle app
  // would report an outage it never observed.
  if (marks.attempted > 0) {
    if (marks.failed === marks.attempted) tickFails += 1;
    else tickFails = 0;
  }
  renderTickStatus();
  tickMark();
}

// The observable half of Z6: the outcome of the last scan, readable BY EQUALITY from the DOM.
// It is a measurement surface, not copy — nothing here is shown to the user.
function redeemMark(m) {
  const el = byId("redeem-overlay");
  if (!el) return;
  el.dataset.finishWhy = String(m.why);
  el.dataset.finishPending = String(m.pending);
  el.dataset.finishScanned = String(m.scanned);
  el.dataset.finishFinished = String(m.finished);
}

// ── THE ONE COMMIT ─────────────────────────────────────────────────────────────────────
async function redeemConnect() {
  const btn = byId("btn-redeem-connect");
  const code = byId("redeem-code").value.trim();
  const name = byId("redeem-name").value.trim();
  redeemClearError();
  // Belt and braces: the button cannot be armed otherwise, but the handler refuses anyway —
  // a gate that exists only in the enable/disable path is one keyboard event from being bypassed.
  if (code === "" || !redeemNameOk(name)) { redeemSyncConnect(); return; }
  btn.disabled = true;
  try {
    await invoke("invite_redeem", { code, alias: name, selfLabel: null });
    byId("redeem-sent-name").textContent = name;
    byId("redeem-sent-name2").textContent = name;
    redeemShow("redeem-sent");
  } catch (e) {
    const c = (e && e.code) ? String(e.code) : "";
    if (c === "commitment_mismatch" || c === "signature_invalid") {
      redeemShowSecurityFailure(c);
    } else {
      redeemRenderError(c, (e && e.detail) ? e.detail : "");
    }
  } finally {
    // ⚠ Re-armed only from the CURRENT field state, never unconditionally.
    redeemSyncConnect();
  }
}

// ── WIRING ─────────────────────────────────────────────────────────────────────────────
byId("btn-choose-create").addEventListener("click", async () => {
  byId("redeem-overlay").classList.add("hidden");
  await openInviteModal();
});
byId("btn-choose-redeem").addEventListener("click", () => {
  redeemClearError();
  redeemSyncConnect();
  redeemShow("redeem-form");
});
// ⚠ v2 — THE WAY OUT, AND IT REUSES THE ONE SHIPPED DISMISSAL. `closeRedeemModal` is already
// the path Escape and the scrim take, and it is already called from `show()` so every screen
// transition clears a pasted code. Wiring Close to a second, bespoke closer would be a second
// thing to keep in agreement with the one-time boundary; there is nothing here to keep in
// agreement because there is nothing here. ⚠ It fires NO invite call — the finish scan rides
// the chooser's OPENER, never its close.
byId("btn-choose-close").addEventListener("click", closeRedeemModal);
byId("redeem-code").addEventListener("input", redeemSyncConnect);
byId("redeem-name").addEventListener("input", redeemSyncConnect);
byId("btn-redeem-connect").addEventListener("click", redeemConnect);
byId("btn-redeem-close").addEventListener("click", closeRedeemModal);
byId("btn-redeem-close2").addEventListener("click", closeRedeemModal);
// ⚠ "Copy details" copies a LOCAL diagnostic and sends nothing. It carries the wire code so a
// screenshot-free user can report what happened; it never carries the code or the contact name.
byId("btn-redeem-copydetails").addEventListener("click", async () => {
  const arm = byId("redeem-failed").dataset.arm || "";
  const text = "QSL invite could not be verified — " + arm + (redeemLastDetail ? " — " + redeemLastDetail : "");
  try { await navigator.clipboard.writeText(text); } catch (_) { /* nothing is promised */ }
});
byId("redeem-overlay").addEventListener("click", (ev) => {
  if (ev.target === byId("redeem-overlay")) closeRedeemModal();
});
document.addEventListener("keydown", (ev) => {
  if (ev.key === "Escape") closeRedeemModal();
});

// ---- boot -----------------------------------------------------------------
(async () => {
  try {
    const cfg = await invoke("settings_get");
    adoptSettings(cfg);
  } catch (_) { /* defaults stand */ }
  // NA-0763 (ruling R4): the TEST-ONLY tempo seam. It rides `app_info` — a
  // Serialize-only DTO with no save path — precisely so that no code path can
  // round-trip a test tempo into `settings.json`. `null` in every ordinary run.
  try {
    const info = await invoke("app_info");
    tickOverrideMs =
      typeof info.tick_override_ms === "number" ? info.tick_override_ms : null;
  } catch (_) { /* no seam: the blessed tempo stands */ }
  await route();
})();

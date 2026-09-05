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
// NA-0774 -- FIX (b): THE BUSY INDICATOR IS QUIET UNDER A TICK.
// Depth counter, not a boolean, so an unbalanced entry can never wedge the
// indicator off; `relayScanOnce` raises it in a try/finally (:relayScanOnce)
// and nothing else touches it. Zero means "behave exactly as before", which is
// the whole of the user-sourced contract.
let tickQuietDepth = 0;
function invoke(cmd, args) {
  // A tick-sourced call does no indicator accounting AT ALL -- it neither shows
  // the indicator nor decrements a counter it never incremented, so a user call
  // already in flight keeps its own showing for its own full duration.
  if (tickQuietDepth > 0) return tauriInvoke(cmd, args);
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
      // NA-0776 (3.6-v3.1 cure B): RESTART, not reload. reload() does not reset the
      // WebContext -- it lives for the process -- so the webview directory cannot be
      // deleted under it, and module-scope state survives.
      try {
        await invoke("restart_app");
      } catch (_) {
        // TRIGGER: the restart command itself failed to dispatch (IPC rejected, or the
        // runtime refused). Falling back to a reload keeps the user out of a wiped
        // surface. ⚠ THE FALLBACK DOES NOT WEAKEN THE CURE: the wipe already set the
        // marker RUST-SIDE, so the webview directory is deleted at the next bootstrap
        // whether this reload or a later manual start gets there first.
        window.location.reload();
      }
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
// NA-0776 (3.6-v3.1 sec 5): "Start over" RESTARTS THE PROCESS. It used to call
  // route(), which walks to the wizard INSIDE THE LIVE PROCESS -- so a new vault was
  // created in a process still holding the wiped session's in-memory state, which is
  // ENG-0276's own reproduction reachable through the shipped UI. The restart also lets
  // bootstrap delete the webview directory, which cannot be done while the WebContext
  // is alive.
  byId("btn-wiped-restart").addEventListener("click", async () => {
    try { await invoke("restart_app"); } catch (_) { route(); }
  });

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

// NA-0764 (R5): the footer's last known inputs. The tick repaints the SAME line
// when its failure counter crosses the threshold, and it must not re-invoke the
// desk to do so — a status line that opens IPC calls on a timer is a second
// clock, which this lane is explicitly not adding.
let lastFooterReason = null;
let lastFooterRelayUrl = "";

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
// ⚠⚠ NA-0764 (`D-1405`, ruling R5) — REASON-FIRST PRECEDENCE, AND IT IS THE
// WHOLE POINT OF WHERE THE NEW ARM SITS.
//
// The reachability arm is NOT a `reason`. It is derived from `tickFails >=
// TICK_FAIL_THRESHOLD`, a global the tick owns, so it is keyed on a different
// source than the five arms above it. The natural place to put "trouble
// replaces normal" is at the TOP — and that is exactly wrong. Placed there, a
// relay outage would MASK the storage line, the locked line, and the
// please-report tripwire, all three of which are real problems the user can
// act on and none of which an outage cures. Worse, an outage and a storage
// fault are CORRELATED: both follow a bad restart or a moved vault.
//
// So the outage arm sits LAST, above `Ready` and below every real problem: it
// renders only in the otherwise-Ready case. The seal for this is a BEHAVIOUR
// test (`na0764_footer_precedence_behaviour`), because the presence seal
// beside it stays green through the whole defect — an arm reordered is still
// an arm present.
//
// ⚠ THE FLAT IF-CHAIN AND THE `const` SHAPE ARE HELD DELIBERATELY this lane.
// The neighbouring seal extracts this body by text and asserts the sentences
// exist as `const NAME = "...";` declarations, so the tempting refactor of a
// six-arm chain into a lookup table would either move the required tokens out
// of the body or truncate the extraction at a column-0 brace.
//
// ⚠ NO RECOVERY BLURB. Recovery is a simple return to normal — the counter
// resets and this function's last arm answers again. A "Reconnected" line was
// refused for this lane.
function statusFooterLine(reason, relayUrl, tickTrouble) {
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
  if (tickTrouble) {
    return TICK_UNREACHABLE_COPY;
  }
  return `Ready. Relay: ${relayUrl}`;
}

// The footer's TIER. Accent, never danger: an unreachable relay needs
// attention, but the danger tier stays reserved for the erasure ceremony —
// NA-0763's ruling, carried forward unchanged rather than reversed here.
function paintStatusFooter(reason, relayUrl) {
  const trouble = tickFails >= TICK_FAIL_THRESHOLD;
  const el = byId("status-line");
  el.textContent = statusFooterLine(reason, relayUrl, trouble);
  el.classList.toggle("is-trouble", trouble && !!relayUrl && isPlainReason(reason));
}

// The outage tier paints only where the outage arm can actually render — the
// otherwise-Ready case. Keeping this in ONE predicate stops the class and the
// text from drifting apart, which is how a line ends up accent-coloured while
// saying "Locked".
function isPlainReason(reason) {
  return (
    reason !== "missing_home" &&
    reason !== "unsafe_parent" &&
    reason !== "vault_locked" &&
    reason !== null &&
    reason !== "unrecognized"
  );
}

async function enterMain() {
  show("scr-main");
  refreshNotices();   // NA-0776 3.3: initial paint; not awaited, never blocking
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
  lastFooterReason = reason;
  lastFooterRelayUrl = relayUrl;
  paintStatusFooter(reason, relayUrl);
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
  // ⚠ NA-0768 (RULING_012 §1(u)): the unlock scan reaches `recordScanOutcome` like any
  //   other, so before this line an unlock repainted NOTHING unless that scan happened to
  //   trip the gate. A session that completed while the app was locked therefore rendered
  //   stale on the first screen the user sees. One call, once per unlock.
  await refreshContacts();
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
// NA-0764: the Contacts pane exists now, so this opens it instead of revealing
// the stub. The stub element and its copy survive untouched — nothing else in
// the app claims contacts are unbuilt, and deleting a message is not this
// lane's act.
byId("btn-rail-contacts").addEventListener("click", () => showContactsPane());
// NA-0765 (`D-0042`) — A1: THE RAIL CAN GO BACK. Until this lane the main rail's
// Chats button carried NO id and NO listener, so Contacts was a one-way door from
// this rail. ⚠ The id is `btn-rail-chats-m`, not `btn-rail-chats`: that name is
// already the SETTINGS rail's button and is pinned by two scenarios. The resulting
// asymmetry (bare = main for Contacts, bare = settings for Chats) is pre-existing and
// is recorded rather than fixed here.
byId("btn-rail-chats-m").addEventListener("click", () => showChatsPane());

// ── NA-0764: SWITCHING BETWEEN THE TWO LIST PANES ────────────────────────────
// Contacts is a PEER of Chats, not a replacement. The rail owns which is shown.
//
// ── NA-0765 (`D-0042`) — A1: THE HIGHLIGHT FOLLOWS THE PANE ──────────────────
// ⚠⚠ THE DEFECT WAS NOT "the highlight fails to move". The Chats button carried a
// HARD-CODED `active` in the markup and nothing ever moved it, so opening Contacts
// left the rail SELECTING THE PANE YOU WERE NO LONGER ON. Measured before the fix:
// `rail-btn` occurred ZERO times in this file (the same needle returns 9 in
// index.html, so it works), and this file's only `classList.*("active")` targets the
// settings rail's category list.
//
// ⚠ IT LIVES IN THE PANE FUNCTIONS, NOT THE LISTENERS. Every caller then gets it —
// including the SETTINGS rail, which reaches both panes through `enterMain()` — rather
// than two listeners each having to remember. It also leaves the listener line above
// byte-identical, so the design seal that pins that exact line stays green untouched
// instead of being re-aimed for a cosmetic reason.
function railSelect(id) {
  for (const b of document.querySelectorAll("#scr-main .rail .rail-btn")) {
    b.classList.toggle("active", b.id === id);
  }
}

function showContactsPane() {
  byId("pane-contacts").classList.remove("hidden");
  document.querySelector(".list-pane:not(#pane-contacts)").classList.add("hidden");
  railSelect("btn-rail-contacts");
  // NA-0765 (B3): the detail-vs-welcome choice belongs to ONE place — the renderer —
  // so opening the pane with nothing selected lands on the Welcome panel rather than
  // on a bare sentence. Called synchronously here so the pane is never briefly wrong
  // while the async refresh below is still in flight.
  renderContactDetail();
  // F4(i): the surface opening is a refresh trigger.
  refreshContacts();
}

function showChatsPane() {
  byId("pane-contacts").classList.add("hidden");
  byId("pane-contact-detail").classList.add("hidden");
  document.querySelector(".list-pane:not(#pane-contacts)").classList.remove("hidden");
  document.querySelector(".content-pane.welcome").classList.remove("hidden");
  railSelect("btn-rail-chats-m");
  renderWelcome();
}

// ── NA-0765 (`D-0042`) — B3: ONE WELCOME ELEMENT, REUSED ─────────────────────
// The button wording is the only difference between the two cases, exactly as the
// blessed layout says. ⚠ THE ALTERNATIVE WAS REFUSED ON MEASUREMENT: rendering welcome
// markup into the detail body would put a SECOND `.welcome-title` inside `#scr-main`,
// and the read census pins that node by exactly that selector — a duplicate would make
// that pin AMBIGUOUS rather than red, which is the worse failure of the two.
function renderWelcome() {
  const has = contactsRows.length > 0;
  byId("welcome-sub").textContent = has
    ? "Select a contact, or add another."
    : "Add a contact to start your first conversation.";
  byId("btn-add-contact").textContent = has ? "Add a contact" : "Add your first contact";
}

// The bare "+" — the EXISTING chooser, the same flow the welcome button uses.
// One entry point serves both, which was measured true before it was relied on.
byId("btn-contacts-add").addEventListener("click", () => openRedeemChooser());

// Row selection, and I8's badge clear. ⚠ THE ACK IS IN MEMORY ONLY (ruling
// sec 5). Opening the detail is the acknowledgment; the COMPARE NOTE stays
// until a future verification lane makes "verified" a recorded fact.
byId("contacts-rows").addEventListener("click", (ev) => {
  const row = ev.target.closest(".contact-row");
  if (!row) return;
  contactsSelected = row.dataset.alias;
  contactsNewBadge.delete(contactsSelected);
  renderContactsList();
  renderContactDetail();
});

// ---- settings (item 14: a VIEW in the same shell; the icon rail is live) --
async function openSettings(pane) {
  show("scr-settings");
  selectPane(pane);
  // NA-0778 (`D-0047`): the Invitations pane refreshes on open, before the other panes' reads.
  if (pane === "invitations") await refreshInvitationsPane();
  await refreshIdentityPane();
  await refreshVaultPane();
  await refreshServerPane();
  const info = await invoke("app_info");
  // NA-0776 (`ENG-0275`, RULING_015 sec 1): SHOW WHICH BUILD THIS IS. The DTO field
  // alone did not deliver the filing -- "a flight cannot state which build it flew" --
  // because a flight is the operator looking at the screen. The SHORT commit goes on
  // this line, the dynamic version-bearing one; `about-text` is left alone because it
  // carries the claim-discipline sentence. MEASURED before choosing: NEITHER About line
  // is pinned by any test.
  const buildShort =
    info.build_commit === "unknown" ? "unknown" : info.build_commit.slice(0, 8);
  byId("about-name").textContent =
    `${info.display_name} (qsl-desktop ${info.version}, build ${buildShort})`;
  // Slice B (D609 R4): the "no network connections" clause is retired — the app
  // now reaches a relay — but the surviving TRUE clause STAYS: no
  // security-assurance claims. Only the network clause changed.
  byId("about-text").textContent =
    `Slice ${info.slice}. This build makes no security-assurance claims.`;
}
byId("btn-settings").addEventListener("click", () => openSettings("identity"));
byId("btn-rail-chats").addEventListener("click", async () => {
  await enterMain();
  showChatsPane();
});
byId("btn-rail-contacts-s").addEventListener("click", async () => {
  await enterMain();
  showContactsPane();
});

function selectPane(name) {
  // Item 13 (§5): pane navigation is a state transition — the ceremony
  // always returns to collapsed and empty.
  resetDestroyFlow();
  for (const b of document.querySelectorAll(".settings-rail .cat[data-pane]")) {
    b.classList.toggle("active", b.dataset.pane === name);
  }
  for (const p of ["identity", "server", "vault", "invitations", "appearance", "notifications", "about"]) {
    byId("pane-" + p).classList.toggle("hidden", p !== name);
  }
  // NA-0778 (`D-0047`, RULING_NA0778_004 R22): selecting the Invitations pane puts it in its
  // LOADING state synchronously; only a completed refresh replaces that with rows.
  if (name === "invitations") invitationsSetLoading();
}
for (const b of document.querySelectorAll(".settings-rail .cat[data-pane]")) {
  b.addEventListener("click", () => {
    selectPane(b.dataset.pane);
    if (b.dataset.pane === "invitations") refreshInvitationsPane();
  });
}

// ---- the Identity pane (existing identity_show surface ONLY) -------------
// NA-0774 -- FIX (c) / E5: A THROWN `identity_show` AND AN ABSENT IDENTITY ARE
// DIFFERENT FACTS AND NO LONGER SHARE A SCREEN. The catch below used to swallow
// the error and fall into the `!rec` branch, which reveals `#identity-empty`:
// "No identity exists yet -- finish setup to create one." A user who HAS an
// identity was told they had none and invited to an action wrong for their
// state. `readFailed` separates the two causes; the error copy names RETRY.
async function refreshIdentityPane() {
  let rec = null;
  let readFailed = false;
  try {
    rec = await invoke("identity_show");
  } catch (_) { readFailed = true; }
  const empty = byId("identity-empty");
  const readError = byId("identity-read-error");
  const body = byId("identity-body");
  if (!rec) {
    // Exactly one of the two absent-body states, never both, never neither.
    empty.classList.toggle("hidden", readFailed);
    readError.classList.toggle("hidden", !readFailed);
    body.classList.add("hidden");
    return;
  }
  empty.classList.add("hidden");
  readError.classList.add("hidden");
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
    // NA-0776 (3.6-v3.1 cure B): RESTART, not reload -- same reasoning as the erase
    // path. The marker is already set Rust-side, so the webview deletion happens at the
    // next bootstrap even if this call fails.
    try {
        await invoke("restart_app");
      } catch (_) {
        // TRIGGER: the restart command itself failed to dispatch (IPC rejected, or the
        // runtime refused). Falling back to a reload keeps the user out of a wiped
        // surface. ⚠ THE FALLBACK DOES NOT WEAKEN THE CURE: the wipe already set the
        // marker RUST-SIDE, so the webview directory is deleted at the next bootstrap
        // whether this reload or a later manual start gets there first.
        window.location.reload();
      }
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
  "Can't reach the relay — still trying. Contacts may not finish connecting until it's back.";

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
// ⚠⚠ NA-0764 (`D-1405`, ruling R5) — THE VISIBLE ROLE MIGRATED; THE NODE STAYS.
//
// NA-0763 shipped this message on the quiet line ABOVE the footer, and the
// operator's own complaint was the resulting two-line pairing. F1 moves the
// message into the footer's closed reason set, so this function no longer
// writes any text.
//
// ⚠⚠ `#tick-status` IS NOT DELETED, AND DELETING IT IS FORBIDDEN. `tickMark()`
// still dereferences it every beat to write the tick's five `data-*` counters;
// removing the element would turn that into a TypeError inside the scan's hot
// path — an E3 break caused by the very edit E3 was cleared for. "Retire the
// visible role" and "delete the node" are different acts and only one of them
// is ruled.
function renderTickStatus() {
  const el = byId("tick-status");
  if (!el) return;
  // Hidden always: the footer now carries this state. The element survives as
  // the tick's observable carrier and nothing else.
  setStatusLine(el, "neutral", "");
  el.classList.add("hidden");
  // Repaint the ONE line that now speaks for reachability.
  paintStatusFooter(lastFooterReason, lastFooterRelayUrl);
}

// The tick's OWN observable counter (R6). Never touches `#redeem-overlay`.
// ---- NA-0776 (`ENG-0274`, spec v2 3.3): the declined-frame notice ----------
// The copy map is ALSO a whitelist: a kind with no entry here renders nothing, so the
// Rust allowlist and the UI agree by construction rather than by discipline. The DTO
// carries `{kind, count}` only -- no timestamps (BLOCKER-4 / NOTE-4).
const NOTICE_COPY = {
  invite_finish_hs_unconsumed: "A connection attempt was declined",
};

// ⚠ NO TIMER. This is called from places that ALREADY complete an invoke -- the scan
// pass and entering main -- because the app's tick was deliberately quieted by
// ENG-0271 and a polled notice would re-introduce exactly the periodic loop that cure
// removed (cold read MINOR-12).
async function refreshNotices() {
  const el = byId("notice-line");
  if (!el) return;
  let rows = [];
  try {
    rows = await invoke("notice_list");
  } catch (_) {
    return; // fail-quiet: a notice must never be the reason something else breaks
  }
  const shown = (rows || []).filter((r) => NOTICE_COPY[r.kind]);
  if (shown.length === 0) {
    el.className = "status-line-quiet hidden";
    delete el.dataset.noticeKind;
    return;
  }
  const r = shown[0];
  setStatusLine(el, "neutral", NOTICE_COPY[r.kind] + (r.count > 1 ? " (" + r.count + ")" : ""));
  el.dataset.noticeKind = r.kind;
  el.dataset.noticeCount = String(r.count);
}

byId("btn-notice-dismiss").addEventListener("click", async () => {
  const kind = byId("notice-line").dataset.noticeKind;
  if (!kind) return;
  try {
    await invoke("notice_dismiss", { kind });
  } catch (_) { /* fail-quiet, as above */ }
  await refreshNotices();
});

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

// ---- NA-0778 (`D-0047`): SETTINGS > INVITATIONS -- THE REVIEW SURFACE (mockup 16) ---------------
//
// REFRESH-ON-OPEN, never polling (the 08-31 bank's decision 1, the same rule the mint's list took).
// ⚠ RULING_NA0778_004 R22, BUILT IN RATHER THAN REMEMBERED: (1) the rows render ONLY after the data
// has arrived -- `invitationsSetLoading` is the pane's state the moment it is selected, and only
// `invitationsRender` replaces it; (2) the pane has NO editable field, so no loaded value can
// overwrite anything a user typed (the hazard E-4 names in the Vault pane); (3) every row action
// carries the invitation id in its own dataset and the handler reads THAT, never a row index, so a
// re-render between the click and the call cannot retarget it.
// ⚠ WHAT THE RECORD CAN SAY (RULING 004 R23, measured at the desktop's pinned qsc rev 63ece4fe, byte-identical to f32a4c20 for these verbs): Waiting (active, not yet
// expired), Expired (the facade's read-time overlay, or active past its expiry by this clock),
// Accepted (redeemed -- the operator's interim word), and the shipped "Didn't finish" for a
// `creating` record. NOT drawn, because nothing in the record carries them: which contact an
// invitation produced, whether that contact is verified, and when it connected. Revoked rows are
// not rendered. An expired row carries NO action: the engine's `invite_clear` accepts a `creating`
// record only (invite/mod.rs:985-999) and refuses every live state -- the mockup's "Clear" on an
// expired row has no verb behind it and is FILED, not faked. NO TIMER anywhere in this module.
// NA-0778 (`D-0047`, 004a / F-22): an anchor that plays a button answers Enter and Space. ONE binder for
// every such control this pane creates (Revoke, Clear, the chips, the Verify links, the create link)
// and for the rail links; the shipped copy link keeps its own handler.
function bindKeyClick(el) {
  el.addEventListener("keydown", (ev) => {
    if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); el.click(); }
  });
}

const INVITATIONS_STATE_TEXT = {
  waiting: "Waiting for reply", accepted: "Accepted", expired: "Expired", failed: "Didn't finish",
};
let invitationsFilter = "all";
let invitationsRows = [];      // the last invite_list read, as delivered
let invitationsContacts = [];  // {alias, name, ui, state} for the nudge, from the contacts refresh

function invitationsKind(r, now) {
  if (r.state === "creating") return "failed";
  if (r.state === "redeemed") return "accepted";
  if (r.state === "expired") return "expired";
  if (r.state === "active") return r.expiry > now ? "waiting" : "expired";
  return null;                                   // revoked: never rendered
}

function invitationsDate(unix) {
  if (!unix) return "—";
  const d = new Date(unix * 1000);
  const now = new Date();
  const sameDay = (a, b) =>
    a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
  const yday = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1);
  if (sameDay(d, now)) return "Today, " + d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  if (sameDay(d, yday)) return "Yesterday";
  const opts = { month: "short", day: "numeric" };
  if (d.getFullYear() !== now.getFullYear()) opts.year = "numeric";
  return d.toLocaleDateString([], opts);
}

function invitationsExpires(r, kind, now) {
  if (kind === "waiting") return "In " + humanDuration(r.expiry - now);
  if (kind === "expired") return invitationsDate(r.expiry);
  return "";
}

function invitationsSetLoading() {
  byId("invitations-loading").classList.remove("hidden");
  byId("invitations-body").classList.add("hidden");
  byId("invitations-nudge").classList.add("hidden");
  byId("invitations-error").classList.add("hidden");
}

async function refreshInvitationsPane() {
  invitationsSetLoading();
  let rows = null;
  let failure = null;
  try {
    rows = await invoke("invite_list");
  } catch (e) {
    failure = e;
  }
  // The nudge's source is the CONTACTS list, which carries the verified state; the shipped refresh
  // publishes it to `contactsRows` / `contactsStatus`, and this reads those after it returns.
  await refreshContacts();
  if (rows === null) {
    byId("invitations-loading").classList.add("hidden");
    // F-12: the shared vocabulary, so a code-less error cannot render as "[object Object]".
    renderInviteError("invitations-error", failure && failure.code, failure && failure.detail, "list");
    return;
  }
  invitationsRows = rows;
  invitationsContacts = contactsRows.map((row) => ({
    alias: row.alias,
    name: contactDisplayName(row),
    ui: contactUiState(row, contactsStatus[row.alias]),
    state: row.state,
  }));
  invitationsRender();
}

function invitationsRowEl(r, kind, now) {
  const tr = document.createElement("tr");
  tr.className = "invitations-row";
  tr.dataset.inviteId = r.invite_id;
  tr.dataset.kind = kind;
  const name = document.createElement("td");
  name.className = "invitations-name";
  name.textContent = r.label ? r.label : "(no name)";
  const state = document.createElement("td");
  const s = document.createElement("span");
  s.className = "invitations-state is-" + kind;
  const dot = document.createElement("span");
  dot.className = "invitations-dot";
  s.append(dot, document.createTextNode(INVITATIONS_STATE_TEXT[kind]));
  state.appendChild(s);
  const when = document.createElement("td");
  when.className = "invitations-when";
  when.textContent = r.created ? "Sent " + invitationsDate(r.created) : "—";
  const exp = document.createElement("td");
  exp.className = "invitations-when";
  exp.textContent = invitationsExpires(r, kind, now);
  const act = document.createElement("td");
  act.className = "invitations-actions";
  if (kind === "waiting" && r.revocable) {
    const a = document.createElement("a");
    // the shipped Revoke idiom: plain, and the danger-LINK token
    a.className = "rm plain link-danger"; a.setAttribute("role", "button"); a.tabIndex = 0;
    a.textContent = "Revoke"; a.dataset.revoke = r.invite_id;
    bindKeyClick(a);
    act.appendChild(a);
  } else if (kind === "failed") {
    const a = document.createElement("a");
    a.className = "rm plain"; a.setAttribute("role", "button"); a.tabIndex = 0;
    a.textContent = "Clear"; a.dataset.clear = r.invite_id;
    bindKeyClick(a);
    act.appendChild(a);
  }
  tr.append(name, state, when, exp, act);
  return tr;
}

// The rows the record yields, classified and ordered (newest first), with the clock they were
// classified against.
function invitationsClassified() {
  const now = Math.floor(Date.now() / 1000);
  return invitationsRows
    .map((r) => ({ r, kind: invitationsKind(r, now), now }))
    .filter((x) => x.kind !== null)
    .sort((a, b) => (b.r.created || 0) - (a.r.created || 0));     // newest first
}

// The count line, from the classified set -- the ONE writer of `#invitations-sent-meta`.
function invitationsRenderMeta(rows) {
  const counts = { waiting: 0, accepted: 0, expired: 0, failed: 0 };
  for (const x of rows) counts[x.kind] += 1;
  const meta = [];
  if (counts.waiting) meta.push(counts.waiting + " waiting");
  if (counts.accepted) meta.push(counts.accepted + " accepted");
  if (counts.expired) meta.push(counts.expired + " expired");
  if (counts.failed) meta.push(counts.failed + " didn't finish");
  byId("invitations-sent-meta").textContent = meta.join(" · ");
}

// NA-0778 (`D-0047`, 004a / RULING_NA0778_006 R38): A SUCCESSFUL REVOKE MOVES THE IN-MEMORY SET,
// not only the DOM. The cold read found the set left as delivered, so any chip click re-rendered
// the revoked row as "Waiting for reply" with a live Revoke. ONE function owns the update (the
// handler's success branch calls it; the arm can exec it): the record is marked revoked, the
// count line is re-rendered from the updated set, and the row -- already flipped to "Revoked"
// where the user is looking -- is dropped on the next render (revoked rows never render).
function invitationsMarkRevoked(id) {
  for (const r of invitationsRows) if (r.invite_id === id) r.state = "revoked";
  invitationsRenderMeta(invitationsClassified());
}

function invitationsRender() {
  const rows = invitationsClassified();
  invitationsRenderMeta(rows);
  const host = byId("invitations-sent-rows");
  host.innerHTML = "";
  const shown = rows.filter((x) => invitationsFilter === "all" || x.kind === invitationsFilter);
  for (const x of shown) host.appendChild(invitationsRowEl(x.r, x.kind, x.now));
  byId("invitations-sent-empty").classList.toggle("hidden", rows.length > 0);
  // R53: the Sent heading's own link shows ONLY while Sent has nothing to list.
  byId("invitations-sent-link").classList.toggle("hidden", rows.length > 0);
  // F-15a: a chip with no matching rows says so, instead of a header-only table.
  byId("invitations-filter-empty").classList.toggle("hidden", !(rows.length > 0 && shown.length === 0));
  byId("invitations-sent").classList.toggle("hidden", shown.length === 0);
  byId("invitations-filters").classList.toggle("hidden", rows.length === 0);
  for (const chip of document.querySelectorAll("#invitations-filters .invitations-chip")) {
    chip.classList.toggle("on", chip.dataset.filter === invitationsFilter);
  }
  invitationsRenderNudge();
  byId("invitations-loading").classList.add("hidden");
  byId("invitations-body").classList.remove("hidden");
}

function invitationsRenderNudge() {
  // "N connected contacts aren't verified yet": CONNECTED by the session status the contacts
  // refresh already reads (a new arrival counts -- it is connected and awaiting verification),
  // and NOT VERIFIED by the contact record's own state -- a join the engine feeds today (RULING
  // 004 R23). The Verify link lands on the contact's detail, which is the shipped verification
  // surface (the code card and its compare hint); no verify pop-up exists in this build, so
  // nothing pretends one does.
  const unverified = invitationsContacts.filter(
    (c) => (c.ui === "connected" || c.ui === "new") && c.state !== "verified"
  );
  const nudge = byId("invitations-nudge");
  if (unverified.length === 0) { nudge.classList.add("hidden"); return; }
  const n = unverified.length;
  byId("invitations-nudge-text").textContent =
    n + " connected contact" + (n === 1 ? " isn't" : "s aren't") +
    " verified yet. Verifying takes a minute on a call and is how you know it's really them.";
  const links = byId("invitations-nudge-links");
  links.innerHTML = "";
  unverified.forEach((c, i) => {
    if (i > 0) links.appendChild(document.createTextNode(" · "));
    const a = document.createElement("a");
    a.className = "rm plain"; a.setAttribute("role", "button"); a.tabIndex = 0;
    a.textContent = "Verify " + c.name; a.dataset.verify = c.alias;
    bindKeyClick(a);
    links.appendChild(a);
  });
  nudge.classList.remove("hidden");
}

// ⚠ THE ID, NEVER THE ROW (R22): the control carries the invitation id and the handler acts on
// that id; the row it repaints is looked up by the same id at that moment.
byId("invitations-sent-rows").addEventListener("click", async (ev) => {
  const t = ev.target;
  const rev = t.dataset && t.dataset.revoke;
  const clr = t.dataset && t.dataset.clear;
  if (!rev && !clr) return;
  byId("invitations-error").classList.add("hidden");
  const id = rev || clr;
  try {
    if (rev) {
      await invoke("invite_revoke", { inviteId: id });
      // FLIP IN PLACE, with no timer: the row reads "Revoked" where the user is looking and leaves
      // on the next refresh (revoked rows are never rendered). Visible success, then tidy.
      const row = document.querySelector('#invitations-sent-rows tr[data-invite-id="' + CSS.escape(id) + '"]');
      if (row) {
        const s = row.querySelector(".invitations-state");
        s.className = "invitations-state is-revoked";
        s.lastChild.textContent = "Revoked";
        row.dataset.kind = "revoked";
        t.remove();
      }
      invitationsMarkRevoked(id);
    } else {
      await invoke("invite_clear", { inviteId: id });
      invitationsRows = invitationsRows.filter((r) => r.invite_id !== id);
      invitationsRender();
    }
  } catch (e) {
    // The row is untouched: nothing flipped, nothing left. The shared vocabulary renders the line.
    renderInviteError("invitations-error", e && e.code, e && e.detail, rev ? "revoke" : "clear");
  }
});

byId("invitations-filters").addEventListener("click", (ev) => {
  const chip = ev.target.closest(".invitations-chip");
  if (!chip) return;
  invitationsFilter = chip.dataset.filter;
  invitationsRender();
});

byId("invitations-nudge-links").addEventListener("click", async (ev) => {
  const a = ev.target.closest("a[data-verify]");
  if (!a) return;
  await enterMain();
  showContactsPane();
  contactsSelected = a.dataset.verify;
  contactsNewBadge.delete(contactsSelected);
  renderContactsList();
  renderContactDetail();
});

// NA-0778 (004c / RULING_NA0778_008 R53): the groups' own links -- shown only while a group has nothing
// to list -- open the existing windows: "send invitation" the mint (painted, then the surface-open
// trigger, as the rail's send does), "redeem invitation" the code-entry view. The page-level
// "Create one from Contacts" link is retired with the mockup's explanation.
byId("btn-invitations-send").addEventListener("click", async () => {
  await openInviteModal();
  await relayScan({ source: "surface_open", at: Date.now() });
});
byId("btn-invitations-redeem").addEventListener("click", () => openRedeemEntry());
for (const chip of document.querySelectorAll("#invitations-filters .invitations-chip")) bindKeyClick(chip);
bindKeyClick(byId("btn-invitations-send"));
bindKeyClick(byId("btn-invitations-redeem"));

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
// NA-0766 (`D-0043`) -- ITEM 7. The empty slot's sentence lives HERE and is written into the box
// by every reset path, so the markup's initial text and the code's reset text cannot drift into
// two different sentences. ⚠ ONE invite per window (item 12): `inviteMinted` latches on a
// successful mint and only a fresh open clears it, which is what makes burning a second slot a
// deliberate act rather than a second click.
const INVITE_SLOT_EMPTY = "Your invite code will appear here after you activate.";
let inviteMinted = false;
// NA-0778 (004f / RULING_NA0778_011 R74 (b), RULING_NA0778_012 R79): a mint IN FLIGHT -- set at the
// Activate click after the label gate, cleared when the promise settles (adopt or failure). While it
// is set the openers and the three user closers are no-ops (the code always lands in an OPEN window)
// and Activate refuses a second mint. Bounded by the client's own timeout on the round trip.
let inviteInFlight = false;
// The window's GENERATION: advances at every open and every close. inviteAdoptCode writes only when the
// generation the mint started under is still the live one AND the window is open; otherwise the code
// is DISCARDED -- never written into a hidden node. The invitation then exists on the page as
// "Waiting for reply", where Revoke is one click.
let inviteGen = 0;
// The two conditions that are FIXED for the lifetime of one open window. `capFull` is deliberately
// NOT recomputed after a mint (ruling Q4 = A): recomputing it is what let the cap line APPEAR on
// activation at the tenth invite, which is the one boundary where the window used to move.
let inviteNoRelay = false;
let inviteCapFull = false;

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
         : verb === "list" ? "Couldn't read your invitations"
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
  inviteGen += 1;
  inviteResetSlot();
  // NA-0778 (004f / R74 (c)): the R53 page refresh lives in inviteUserClose(), the three user gestures'
  // closer, NOT here -- this structural closer is also show()'s, and a lock or a screen transition
  // must never call into a sealed vault.
  byId("invite-label").value = "";
  byId("invite-label").readOnly = false;
  inviteId = null;
  inviteMinted = false;
  inviteSyncActivate(); // NA-0778 (004f / R74 (d)): the hint follows the cleared field; nothing flashes on the next open
  clearInviteErrors();
  inviteShowMint();
}

// NA-0766 (`D-0043`) -- ITEM 7. The slot returns to its EMPTY state: the sentence back in the
// box, the minted border off, the empty treatment on. One function owns it and every reset path
// calls it, for the same reason `inviteEnterMintFresh` exists -- two paths that clear the same
// thing differently is precisely the defect v3 had to fix once already.
// ⚠⚠ NA-0766 (`D-0043`) -- EVERY DOM CHANGE A SUCCESSFUL MINT MAKES, IN ONE NAMED PLACE.
// This is the exact counterpart of `inviteResetSlot`, and it is a named function rather than a
// run of statements inside the click handler FOR A MEASURABLE REASON: the desktop harness has no
// fixture relay (`ENG-0226`, open), so a scenario cannot make `invite_create` succeed and could
// not otherwise reach the post-mint state at all. With the transition owned by one function, the
// gui-driver drives the PRODUCT'S OWN CODE with a synthetic code -- the same idiom `f_n` already
// uses to reach the contacts render -- instead of a test re-implementing what the product does,
// which would prove only that the test agrees with itself.
function inviteAdoptCode(code, gen) {
  // NA-0778 (004f / RULING_NA0778_012 R79): GENERATION-CHECKED. The code lands only in the window it
  // was minted in, and only while that window is open. A stale generation (the window closed, or
  // closed and re-opened, during the round trip) or a hidden overlay discards it: nothing is written
  // into a hidden node (the first reader's F-01).
  if (gen !== inviteGen || byId("invite-overlay").classList.contains("hidden")) return false;
  // ITEM 7: the code lands IN the slot that was already on screen. The box swaps `.empty` for
  // `.minted` -- a colour change, not a layout one, which is what lets item 6 hold.
  const box = byId("invite-code");
  box.textContent = code;
  box.classList.remove("empty");
  box.classList.add("minted");
  // ⚠ ITEM 12: ONE INVITE PER WINDOW, and this is the line that makes burning a second slot
  // deliberate. Before this lane the handler set `disabled = true` and then `inviteRefresh()`
  // RE-ASSIGNED it from the relay and cap alone -- so with a relay set and the cap unreached the
  // control came BACK, and a second press minted a second invite and burned a second slot. The
  // latch is read by the ONE decision, so no later refresh can undo it.
  inviteMinted = true;
  // ITEM 12: the field becomes read-only showing what the invite was actually minted with, so the
  // window keeps answering "who was this for?" without offering an edit that could no longer
  // change anything.
  byId("invite-label").readOnly = true;
  inviteSyncActivate();
  return true;
}

function inviteResetSlot() {
  const box = byId("invite-code");
  box.textContent = INVITE_SLOT_EMPTY;
  box.classList.add("empty");
  box.classList.remove("minted");
}

// NA-0766 (`D-0043`) -- ITEM 6. There is no pre/post pair to swap any more: the window renders
// its final shape from open. This function now only chooses BETWEEN the two views of the overlay
// (mint vs list), which is a different thing from transforming one of them.
function inviteShowMint() {
  byId("invite-mint").classList.remove("hidden");
  byId("invite-list-view").classList.add("hidden");
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
  byId("invite-label").readOnly = false;
  inviteMinted = false;
  inviteResetSlot();
  byId("invite-meta-note").textContent = "Invite code";
  byId("invite-meta-expiry").textContent = "";
  byId("invite-copy-note").classList.add("hidden");
  inviteId = null;
  inviteShowMint();
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
  // NA-0766 (`D-0043`) -- ITEM 5. The write to the pill's count span retires WITH the pill that
  // contained it. Keeping it would null-dereference and take the whole surface down, so the two
  // are one act, not two (ruling sec 2(b)). ⚠ The retired id is DESCRIBED, never spelled: a
  // comment recording a removal re-plants the removed thing's needle (`ENG-0235`, and this lane's
  // own ruling Q5).
  byId("invite-slots").textContent = `${live} of ${INVITE_SOFT_CAP} slots used — codes expire on their own`;
  let relayUrl = "";
  try {
    const cfg = await invoke("relay_config_get");
    relayUrl = cfg.relay_url || "";
  } catch (_) { relayUrl = "" ; }
  inviteNoRelay = relayUrl === "";
  // ⚠ NA-0766 (`D-0043`) -- RULING Q4 = (A). The cap is READ here but its EXPLANATION LINE is not
  // toggled here, and the `#invite-cap-full` line is never shown from inside a live window. The
  // old code recomputed it after every mint, so minting the TENTH invite made a line APPEAR after
  // activation -- the one boundary at which this window used to move. With item 12 disabling
  // Activate after a mint, the explanation is redundant at exactly the moment it would have
  // appeared, and the cap stays enforced by the disabled control rather than by prose.
  inviteCapFull = live >= INVITE_SOFT_CAP;
  inviteSyncActivate();
  byId("invite-no-relay").classList.toggle("hidden", !inviteNoRelay);
  return relayUrl;
}

// ⚠⚠ NA-0766 (`D-0043`) -- ITEM 10, AND ITS SHAPE IS FORCED (ruling sec 2(c)). This is the ONLY
// place in the file that decides whether Activate is enabled, and the name term is folded INTO
// that one decision rather than placed beside it. A separate name gate would be silently
// overwritten the next time `inviteRefresh()` ran -- which it does on open AND after every mint.
// Four causes, ONE assignment:
//   no relay        -- the control cannot succeed
//   cap reached     -- the control cannot succeed
//   name empty      -- ITEM 10: a name is REQUIRED, and the disabled control is the WHOLE
//                      enforcement. There is no error text, by design.
//   already minted  -- ITEM 12: one invite per window.
// ⚠ The emptiness test TRIMS first, so a field holding only spaces is empty. That matches how
// the label has always been READ at mint time, so the gate and the mint cannot disagree.
// NA-0778 (004d / RULING_NA0778_009 R61): the name term is the REDEEM side's grammar, not emptiness
// alone -- the engine's channel_label_ok refuses a label outside [A-Za-z0-9_#-] at the INVITER's
// accept (contacts_provision_from_invite), so a name with a space minted an invitation that could
// never be accepted, silently. The one decision keeps its shape (four causes, one assignment); the
// hint beneath the field shows only while the typed name is non-empty and illegal; the value the
// user typed is never rewritten here.
function inviteSyncActivate() {
  const name = byId("invite-label").value.trim();
  const nameOk = name !== "" && REDEEM_NAME_RE.test(name);
  byId("invite-label-hint").classList.toggle("hidden", !(name !== "" && !nameOk));
  byId("btn-invite-activate").disabled = inviteNoRelay || inviteCapFull || !nameOk || inviteMinted || inviteInFlight;
}

// NA-0778 (004c / RULING_NA0778_008 R54, completing R40): A LIVE MINT IS NEVER SILENTLY DISCARDED, and
// with the close confirmation retired by the operator's flight ruling the guard's action is a NO-OP:
// a re-open while a code is on screen (the keyboard's route behind the scrim) leaves the open mint
// exactly as it is -- no reset, no question. Comments stay ABOVE this function: a pin reads a fixed
// 400-byte window after its signature (D-05).
// NA-0778 (004f / RULING_NA0778_011 R74 (a)): ONE predicate for "a mint is live" -- a code on screen
// or a mint in flight, in a window that is open. It guards the no-op re-open below AND both redeem
// openers: while a code is on screen NO other opener does anything -- R54's principle extended to
// the second window (the second reader's F2-05: a redeem overlay stacked over a live mint let one
// Escape discard the code).
function inviteLive() {
  return (inviteMinted || inviteInFlight) && !byId("invite-overlay").classList.contains("hidden");
}

async function openInviteModal() {
  if (inviteLive()) return;
  inviteGen += 1;
  clearInviteErrors();
  inviteEnterMintFresh();
  byId("btn-invite-activate").textContent = HAS_CLIPBOARD_ITEM ? "Activate & Copy" : "Activate";
  byId("invite-overlay").classList.remove("hidden");
  await inviteRefresh();
  // RULING Q4 = (A): the cap explanation is decided ONCE, at open, and held for the lifetime of
  // this window. It can therefore never appear as a RESULT of activation, which is the property
  // item 6 states and `I4` measures at the tenth-invite boundary.
  byId("invite-cap-full").classList.toggle("hidden", !inviteCapFull || inviteNoRelay);
}

// ITEM 10: the gate follows the field, so the control answers the user as they type.
byId("invite-label").addEventListener("input", inviteSyncActivate);

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
  if (inviteInFlight) return; // NA-0778 (004f / R74 (b)): ONE mint at a time -- never a second invite_create in flight
  clearInviteErrors();
  const label = byId("invite-label").value.trim();
  // R61, belt and braces (the redeem handler's own shape): the commit refuses an illegal name
  // independently of the button's state -- a gate that lives only in the enable path is one
  // keyboard event from being bypassed.
  if (label === "" || !REDEEM_NAME_RE.test(label)) { inviteSyncActivate(); return; }
  const btn = byId("btn-invite-activate");
  inviteInFlight = true; // NA-0778 (004f / R74 (b)): in flight from here until the promise settles
  const gen = inviteGen;
  // In-flight guard. Unconditional and deliberately so -- it is not a DECISION about whether the
  // control may be enabled, which is `inviteSyncActivate`'s single job, but a latch for the
  // duration of one call.
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
    if (gen !== inviteGen) { inviteInFlight = false; return; } // NA-0778 (004g / RULING_NA0778_013 R85 (a)): a stale failure paints no error into a later window and calls nothing in the vault after a lock
    // A failed mint burns nothing, so the window returns to whatever the ONE decision says --
    // never to a bare `true`, which would have re-enabled it with an empty name.
    inviteSyncActivate();
    inviteInFlight = false;
    renderInviteError("invite-error-mint", e && e.code, e && e.detail, "create");
    await inviteRefresh();
    return;
  }
  inviteInFlight = false;
  if (!inviteAdoptCode(code, gen)) { inviteSyncActivate(); return; } // the window is gone (R79): discarded; the invitation is on the page; the next window's Activate re-decided (004g / R85 (b))
  // v3: on success the link itself is the affordance and no note is needed; the note exists
  // only to say when the single gesture could NOT copy, and it points at the link.
  const note = byId("invite-copy-note");
  note.textContent = copied ? "" : "Copy didn't complete — use the copy code link below.";
  note.classList.toggle("hidden", copied);
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

// ⚠ v4: the mid-mint cancel handler is REMOVED with its button. The single kill mechanism is
// Revoke in the list — one word, one place. Mid-mint regret is the invites list → Revoke, one
// extra click for a rare case, and that trade is recorded rather than lost.
// ⚠⚠ NA-0766 (`D-0043`, ruling Q5): this comment previously SPELLED the retired control's label.
// Its twin in the markup was the sole support for a copy assertion that had been unfalsifiable
// since v4 — passing on the explanation of the very deletion it denied. Both are now DESCRIBED
// rather than spelled, which is the general cure `ENG-0235` names.

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

// NA-0766 (`D-0043`) -- ITEMS 5 and 14. The slot-counter pill and the fresh-mint button are gone,
// and their handlers go with them. The list is now reached from the Contacts pane's link, and a
// fresh mint is Close then "+" -- navigation lives in ONE place instead of being duplicated inside
// the modals. ⚠ Their labels are DESCRIBED, never spelled (`ENG-0235` / ruling Q5).
// ⚠ `inviteEnterMintFresh` is NOT orphaned by this: `openInviteModal` still calls it, which is
// the path that keeps a typed label from riding a later mint (the v3 defect).

// ITEM 1: the Contacts pane's link is the ONE route to the list. It opens the overlay on the list
// view directly, so the user lands where the link said they would.
// NA-0778 (`D-0047`) -- THE INVITATIONS BLOCK'S THREE ENTRY POINTS (mockup 17, blessed 2026-09-01).
// ⚠ ENTRY POINTS, NOT SECOND FLOWS: each lands on a surface that already ships. `review` is a
// SCREEN transition to Settings > Invitations (so `show()` closes both overlays on the way, as it
// does for every transition); `redeem` opens the redeem overlay ON ITS CODE-ENTRY VIEW, through the
// chooser's own opener so the finish scan that rides that opener (R387 S6) still fires; `send`
// opens the mint through `openInviteModal`, the ONE path that enters the mint fresh.
// ⚠ NAMED CONSEQUENCE, not carried silently: the overlay's own list view is no longer reachable
// from any control -- the page is the review surface now. Its markup, its renderer and the pins on
// them are LEFT IN PLACE (outside this lane's ordered edit set); retiring them is a small item.
function railLinkKeys(id) { bindKeyClick(byId(id)); }
byId("btn-contacts-review").addEventListener("click", () => openSettings("invitations"));
// NA-0778 (`D-0047`, 004a / RULING_NA0778_006 R39): PAINT FIRST, THEN SCAN, for both links. The cold
// read found `redeem` awaiting the chooser's opener -- which paints the CHOOSER and only then awaits
// the surface-open scan -- so with a relay configured the chooser sat on screen for the scan's
// whole round trip; and `send` entered the mint WITHOUT the surface-open trigger "+" fires. Now
// `redeem` opens the code-entry view through its own opener (the view painted and the overlay
// shown BEFORE the await), and `send` paints the mint and THEN fires the same trigger, so P2's
// property -- every surface-open feeds the one handler -- is evenly true again.
byId("btn-contacts-redeem").addEventListener("click", () => openRedeemEntry());
byId("btn-contacts-send").addEventListener("click", async () => {
  await openInviteModal();
  await relayScan({ source: "surface_open", at: Date.now() });
});
railLinkKeys("btn-contacts-review");
railLinkKeys("btn-contacts-redeem");
railLinkKeys("btn-contacts-send");
// NA-0765 (`D-0042`): the Chats "+" and its listener retire together — adding people is
// a Contacts act. `#btn-contacts-add` and the welcome button carry the flow.
// NA-0766 (`D-0043`) -- ITEMS 2, 3 and 4. The corner X and the Back are gone from this overlay
// and this single full-width Close is its ONE exit control. It reuses `closeInviteModal`, which
// is also what Escape and the scrim call -- so the visible exit and the invisible ones cannot
// drift apart. NA-0765 wired the X to that same closer for exactly this reason; the property
// survives its control.
// NA-0778 (004c / RULING_NA0778_008 R54, the operator's flight ruling): THERE IS NO CLOSE CONFIRMATION.
// After a mint, Close, Escape and the scrim simply close the window through the ONE closer, as they
// did before 004. The Director's note stands in the record (an accidental close loses an unshared
// code); the operator flew it and ruled. 004f adds the in-flight no-op and the page refresh at the
// gesture (below), not at the closer.
// NA-0778 (004f / RULING_NA0778_011 R74 (c), RULING_NA0778_012 R79): the THREE user gestures -- Close,
// Escape, the scrim -- share this one gesture closer. A no-op while the window is hidden (Escape fires
// everywhere) or a mint is in flight (the code must land in an OPEN window; the round trip is bounded).
// Otherwise the ONE closer, THEN the R53 page refresh -- which lives here and not in closeInviteModal
// so that show()'s structural close during a lock or a screen transition never calls a sealed vault.
function inviteUserClose() {
  const ov = byId("invite-overlay");
  if (!ov || ov.classList.contains("hidden")) return;
  if (inviteInFlight) return;
  closeInviteModal();
  if (currentScreen === "scr-settings" && !byId("pane-invitations").classList.contains("hidden")) refreshInvitationsPane();
}
byId("btn-invite-close").addEventListener("click", () => inviteUserClose());
byId("invite-overlay").addEventListener("click", (ev) => {
  if (ev.target === byId("invite-overlay")) inviteUserClose();
});
document.addEventListener("keydown", (ev) => {
  if (ev.key === "Escape") inviteUserClose();
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

const REDEEM_NAME_HINT_OK = "Stored only on this device \u2014 never sent anywhere.";
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

// NA-0778 (`D-0047`, 004a / R39): the code-entry landing's OWN opener. It is the chooser opener with
// the view swapped, and it is a second function rather than a parameter BECAUSE the trigger-census
// pin reads `openRedeemChooser()` by its literal signature and requires the surface-open trigger
// inside that body; the two share four lines by design and fire the same trigger AFTER painting.
async function openRedeemEntry() {
  if (inviteLive()) return; // NA-0778 (004f / R74 (a)): refused while a mint is live
  redeemClearError();
  byId("redeem-code").value = "";
  byId("redeem-name").value = "";
  redeemSyncConnect();
  redeemShow("redeem-form");
  byId("redeem-overlay").classList.remove("hidden");
  await relayScan({ source: "surface_open", at: Date.now() });
}

async function openRedeemChooser() {
  if (inviteLive()) return; // NA-0778 (004f / R74 (a)): refused while a mint is live
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
        // ⚠⚠ NA-0768 (D-1409, RULING_012 §1(a)): THE REPAINT SIGNAL, AND WHY IT IS NOT
        //   `done`. On the E4 completing path the fan-out consumes the peer's A2 and
        //   commits her session, then `invite_finish` returns Ok(FALSE) at
        //   `invite/mod.rs:1649` -- because `Ok(true)` is reserved for the selected
        //   invite-RESP path, which an inviter never has. So `finished` stays 0 while a
        //   contact has genuinely gone live, `recordScanOutcome`'s gate never fires, and
        //   the screen keeps saying "Connecting…" over a live session. Measured on three
        //   AWS flights; diagnosed to file:line in STOP 012.
        // ⇒ THE HONEST SIGNAL IS THE STATE ITSELF. Re-read this one alias and compare BY
        //   EQUALITY with the value read at the top of this iteration.
        // ⚠ Deliberately NOT `attempted > 0`: a profile carrying permanently-pending
        //   contacts would then repaint every beat forever, which is the churn the gate
        //   exists to prevent.
        try {
          const after = await invoke("connect_status", { peer: row.alias });
          if (after.state !== st.state) marks.changed += 1;   // EQUALITY on the extracted value
        } catch (_) { /* a status re-read must never break the scan it rides on */ }
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

// ══════════════════════════════════════════════════════════════════════════════
// NA-0764 (`D-1405`) — THE CONTACTS SURFACE: auto-connect, and verify first.
// ══════════════════════════════════════════════════════════════════════════════

// R4's RULED MAPPING, as MEASURED at STOP 003 S1(e) — and the measurement
// REVERSED the cold read's classification, so the order of these arms is not a
// matter of taste.
//
// ⚠⚠ `missing_seed` IS "CONNECTING", NOT A FAULT, AND GETTING THIS BACKWARDS
// SHIPS FALSE COPY ON THE COMMONEST STATE. The desktop never sets
// `QSC_QSP_SEED`, so `qsp_status_tuple`'s no-session branch ALWAYS takes the
// `else` arm: `no_session` is unreachable in this app and every not-yet-
// connected contact answers `missing_seed`. Mapping it to "Needs attention"
// would tell every establishing contact it has a storage problem AND would
// leave "Connecting…" with no reachable member at all. The shipped footer
// already ruled this once (D-0033): "A healthy fresh profile answers
// missing_seed, so a footer rendering it as a problem would call every new
// install broken."
const CONTACT_FAULT_REASONS = [
  "session_invalid",   // a dead session is not "connecting"
  "unsafe_parent",     // reachable in the field: the qsc dir's perms change
  "missing_home",      // unreachable in-app (bootstrap sets QSC_CONFIG_DIR) — mapped for TOTALITY
  "channel_invalid",   // unreachable for listed rows (store keys passed channel_label_ok) — TOTALITY
];

// The badge set. ⚠ IN-MEMORY ONLY, by ruling sec 5: "No new persistence for the
// badge ack." It is a NUDGE, not a record — the future verification lane is
// what makes "verified" a durable fact.
const contactsNewBadge = new Set();

let contactsRows = [];
let contactsStatus = {};
let contactsSelected = null;
let contactsOutstanding = 0;

/// The six blessed states, resolved in RULED precedence order.
/// blocked > CHANGED > new-badge > connected > fault > connecting.
function contactUiState(row, st) {
  if (row.blocked) return "blocked";                       // dominates CHANGED (R4)
  // NA-0778 (004a / RULING_NA0778_006 R42, F-14): the WIRE FORM. The gateway emits the facade's
  // lowercase "changed" (commands.rs contact_state_wire); this arm compared the upstream
  // UPPERCASE and was unreachable from live data -- the MITM tell, dead since NA-0764. Every
  // comparison of a contact record's `state` in this file is censused in STOP 004a.
  if (row.state === "changed") return "changed";           // dominates Active (R4)
  if (contactsNewBadge.has(row.alias)) return "new";
  if (st && st.state === "active") return "connected";
  if (st && CONTACT_FAULT_REASONS.indexOf(st.reason) !== -1) return "attention";
  return "connecting";
}

// The ratified tier-1 verification code: 30 digits, grouped 6x5, read aloud and
// compared. ⚠ THE GROUPING IS THE RATIFIED FORM, not decoration — the mockup's
// `QF3K-92MB-7A` was a placeholder and is NOT a format (STOP 002 D-B).
function voiceGroups(voice) {
  if (typeof voice !== "string" || voice.length !== 30) return "";
  return voice.match(/.{5}/g).join(" ");
}

function contactDisplayName(row) {
  // display_name is RENDER-ONLY. `alias` is what every command receives.
  return row.display_name ? row.display_name : row.alias;
}

function renderContactsList() {
  const host = byId("contacts-rows");
  if (!host) return;
  host.innerHTML = "";
  for (const row of contactsRows) {
    const ui = contactUiState(row, contactsStatus[row.alias]);
    const el = document.createElement("div");
    el.className = "contact-row";
    if (ui === "new") el.classList.add("is-new");
    if (contactsSelected === row.alias) el.classList.add("is-selected");
    el.dataset.alias = row.alias;
    el.dataset.uiState = ui;

    const dot = document.createElement("span");
    dot.className = "contact-dot " + CONTACT_DOT[ui];
    const name = document.createElement("span");
    name.className = "contact-name";
    name.textContent = contactDisplayName(row);
    el.append(dot, name);

    const word = CONTACT_WORD[ui];
    if (word) {
      const w = document.createElement("span");
      w.className = "contact-word" + (CONTACT_WORD_TONE[ui] || "");
      w.textContent = word;
      el.appendChild(w);
    }
    host.appendChild(el);
  }
  byId("contacts-empty").classList.toggle("hidden", contactsRows.length > 0);

  // ⚠ NA-0766 (`D-0043`) -- ITEM 1. The outstanding-count line is GONE from the pane; the
  // "review invites" link stands in its place and carries NO count, by the operator's variant-B
  // ruling, taken knowingly. The link is ALWAYS VISIBLE (ruling Q2 = A) and therefore needs no
  // render pass here at all.
  // ⚠⚠ CONSEQUENCE, NAMED RATHER THAN CARRIED SILENTLY (ruling Q2): `contactsOutstanding` and the
  // `invite_list` call that computes it in `refreshContacts()` are now DEAD -- nothing reads them
  // and no live coupling depends on them. They are LEFT IN PLACE because removing them is outside
  // this lane's ordered edit set. This comment exists so the next reader does not mistake dead
  // code for a coupling worth preserving, and does not "discover" it as a defect: it is a
  // consequence of a blessed change, not a bug.
}

const CONTACT_DOT = {
  connected: "is-connected",
  connecting: "is-connecting",
  new: "is-connecting",
  changed: "is-warn",
  attention: "is-warn",
  blocked: "is-blocked",
};
const CONTACT_WORD = {
  connected: "",
  connecting: "…",
  new: "new — verify",
  changed: "check identity",
  attention: "needs attention",
  blocked: "",
};
const CONTACT_WORD_TONE = {
  new: " is-accent",
  changed: " is-warn",
  attention: " is-warn",
};

function renderContactDetail() {
  const body = byId("contact-detail-body");
  if (!body) return;
  body.innerHTML = "";
  const row = contactsRows.find((r) => r.alias === contactsSelected);
  const onContacts = !byId("pane-contacts").classList.contains("hidden");

  // NA-0765 (`D-0042`) — B3: NOTHING SELECTED IS THE WELCOME PANEL, not a bare
  // sentence. The blessed layout shows the same welcome the Chats pane shows; only
  // the button wording differs between the has-contacts and empty-list cases.
  if (!row) {
    renderWelcome();
    if (onContacts) {
      byId("pane-contact-detail").classList.add("hidden");
      document.querySelector(".content-pane.welcome").classList.remove("hidden");
    }
    return;
  }
  if (onContacts) {
    byId("pane-contact-detail").classList.remove("hidden");
    document.querySelector(".content-pane.welcome").classList.add("hidden");
  }

  const ui = contactUiState(row, contactsStatus[row.alias]);

  const name = document.createElement("div");
  name.className = "contact-detail-name";
  name.textContent = contactDisplayName(row);
  body.appendChild(name);

  const state = document.createElement("div");
  state.className = "contact-detail-state " + CONTACT_DETAIL_TONE[ui];
  state.textContent = CONTACT_DETAIL_STATE[ui];
  body.appendChild(state);

  // ── NA-0765 (`D-0042`) — A2/A3/B2: THE PANE ADOPTS THE SHIPPED SETTINGS>IDENTITY
  // IDIOM. `.pane-form` + `.pane-sect` + `.field-label` + `.ctlrow` + `.hint` all
  // ship and are globally scoped, so the structure the blessed layout draws costs
  // ZERO new classes; the adjacent-sibling rule on `.pane-sect` draws the hairlines,
  // so their count follows the section count and cannot drift.
  const form = document.createElement("div");
  form.className = "pane-form";
  body.appendChild(form);

  const sect = () => {
    const d = document.createElement("div");
    d.className = "pane-sect";
    form.appendChild(d);
    return d;
  };
  const label = (host, text) => {
    const l = document.createElement("span");
    l.className = "field-label";
    l.textContent = text;
    host.appendChild(l);
  };
  const hint = (host, text) => {
    const h = document.createElement("p");
    h.className = "hint";
    h.textContent = text;
    host.appendChild(h);
  };

  // The state's own explainer, as a box where the blessed layout draws one.
  // ⚠ THE BOX TIER IS THE SHIPPED ONE, NOT THE MOCKUP'S HEX. The layout authority
  // draws the two warning boxes in the danger tier; the shipped tokens are the
  // COLOUR authority and `.callout.warning` carries a ruling of its own —
  // "warning accent, NEVER red: red stays reserved for vault loss" — which
  // NA-0763 and NA-0764 both carried forward rather than reversed.
  const note = CONTACT_DETAIL_NOTE[ui];
  if (note) {
    const n = document.createElement("div");
    n.className = CONTACT_DETAIL_BOX[ui] || "contact-detail-note";
    n.textContent = ui === "new" ? note.replace("{name}", contactDisplayName(row)) : note;
    form.appendChild(n);
  }

  // ⚠ SECTION ORDER IS STATE-DEPENDENT, and that is the blessed layout's own
  // choice rather than an accident: a CONNECTED contact leads with the name you
  // gave them, a NEW one leads with the code you are being asked to compare.
  const codeFirst = ui === "new" || ui === "changed";
  const wantsName = ui !== "blocked" && ui !== "changed";

  const renderCode = () => {
    if (!row.fingerprint || !voiceGroups(row.fingerprint.voice)) return;
    const d = sect();
    label(d, ui === "changed" ? "Verification code (new)" : "Verification code");
    const card = document.createElement("div");
    card.className = "contact-code-card";
    const code = document.createElement("div");
    code.className = "contact-code";
    code.textContent = voiceGroups(row.fingerprint.voice);
    card.appendChild(code);
    d.appendChild(card);
    if (ui === "new" || ui === "connected") {
      hint(d, "If you can, compare this code with them over a call or in person. The full verification screen arrives in a later update.");
    }
  };

  // ── NA-0765 (`D-0042`) — A3: RENAME. `display_name` sits BESIDE the alias key and
  // is RENDER-ONLY; `alias` is what the verb receives, which is why the structural
  // seal counting `display_name` in invoke-argument positions stays at ZERO. An
  // empty box CLEARS the name — the engine normalises, so no caller special-cases "".
  const renderName = () => {
    const d = sect();
    label(d, "Their name");
    const rowEl = document.createElement("div");
    rowEl.className = "ctlrow";
    const input = document.createElement("input");
    input.type = "text";
    input.id = "contact-rename-input";
    input.className = "w-alias";
    input.maxLength = 32;
    input.autocomplete = "off";
    input.spellcheck = false;
    input.value = row.display_name ? row.display_name : "";
    input.placeholder = row.alias;
    const save = document.createElement("button");
    save.className = "secondary";
    save.id = "btn-contact-rename";
    save.textContent = "Save";
    rowEl.append(input, save);
    d.appendChild(rowEl);
    hint(d, "Stored only on this device — never sent anywhere.");
    const status = document.createElement("p");
    status.className = "hint";
    status.id = "contact-rename-status";
    d.appendChild(status);
    save.addEventListener("click", async () => {
      const typed = input.value.trim();
      const keyed = row.alias;
      status.textContent = "";
      try {
        await invoke("contact_set_display_name", { alias: keyed, displayName: typed === "" ? null : typed });
      } catch (_) {
        status.textContent = "That name could not be saved.";
        return;
      }
      await refreshContacts();
    });
  };

  if (codeFirst) renderCode();
  if (wantsName) renderName();
  if (!codeFirst) renderCode();

  // R3: DEVICES on the connected detail — and "Connected since" is DROPPED.
  // `seen_at` reads as LAST SEEN, not "connected since"; rendering it under that
  // label would show today's date for a contact made a year ago, and rendering it
  // truthfully would be a per-contact presence disclosure at a precision nobody
  // blessed (cold read C-20). The honest answer was to drop the line, not to
  // dress up the wrong field.
  //
  // ⚠ The count is a PROJECTION the facade computes; the device ARRAY never
  // crosses this boundary, because it carries device ids and key material.
  if (ui === "connected" && typeof row.device_count === "number") {
    const d = sect();
    label(d, "Connection");
    const dev = document.createElement("div");
    dev.className = "contact-detail-kv";
    dev.textContent = "Devices: ";
    const n = document.createElement("b");
    n.textContent = String(row.device_count);
    dev.appendChild(n);
    d.appendChild(dev);
    hint(d, "Messaging options will appear here when messaging ships.");
  }

  // ⚠ NO BLOCK CONTROL, IN ANY STATE, AND THAT IS A RULING RATHER THAN AN OMISSION
  // (NA-0765 R-1 = Option A). The blessed layout draws Block and an Unblock that
  // "restores the connection you already had". Measured at this pin: the honest
  // symmetric pair `contacts_block`/`contacts_unblock` exists in the engine and is
  // NOT in the facade, so the desktop cannot reach it; the one verb that IS
  // reachable is one-way AND sets the primary device to REVOKED, which no exposed
  // verb restores — so its blessed sentence would be false. A control whose copy is
  // measurably untrue does not ship. `ENG-0248` names the missing pair.
  byId("pane-contact-detail").dataset.uiState = ui;
}

const CONTACT_DETAIL_STATE = {
  connected: "✓ Connected",
  connecting: "Connecting…",
  new: "New contact — verify identity",
  changed: "Check identity",
  attention: "Needs attention",
  blocked: "Blocked",
};
const CONTACT_DETAIL_TONE = {
  connected: "is-connected",
  connecting: "is-connecting",
  new: "is-new",
  changed: "is-warn",
  attention: "is-warn",
  blocked: "is-blocked",
};
// NA-0765 (`D-0042`): which states render their explainer as a BOX, and in which
// shipped tier. States with no entry render the plain muted note they always did.
const CONTACT_DETAIL_BOX = {
  new: "callout",
  changed: "callout warning",
  attention: "callout warning",
};
const CONTACT_DETAIL_NOTE = {
  new: "Connected using your invite for {name}. Compare verification codes with them before sharing anything sensitive.",
  connecting:
    "Finishing automatically in the background — nothing for you to do. If it never completes, the status line below will say why.",
  changed:
    "Their verification code has changed. Don't share anything sensitive until you compare codes with them again.",
  attention: "This connection has a storage problem and can't finish on its own.",
  connected: "",
  blocked: "",
};

/// Read the contact surface. ⚠ A LOCKED VAULT MAKES `contact_list` REFUSE, and
/// that is NOT "no contacts": M7 measured `require_unlocked_here` running first,
/// so a locked vault yields Err and ZERO rows. Rendering the empty-state copy
/// there would tell the user their contacts are gone. The rows are left ALONE
/// on failure and the footer — which already says "Locked" — speaks instead.
// ⚠⚠ THE GENERATION GUARD IS NOT DECORATION — A LATE REFRESH MUST NOT OVERWRITE
// A NEWER ONE. This function awaits several IPC calls (contact_list, one
// connect_status PER ROW, invite_list), so two refreshes triggered close
// together — surface-open and a scan completion, say — overlap freely, and the
// one that STARTED FIRST can FINISH LAST. Without this guard it would then
// publish its stale rows over the newer result, and the pane would silently
// show a state the app had already moved past.
//
// The cold read named this exact hazard (C-12: "F4's refresh path is UNMEASURED
// for re-entrancy"), and the gui-driver then reproduced it: planted rows were
// published, asserted present, and were GONE two steps later when an in-flight
// refresh from the pane-open resolved on top of them. That was the product
// racing, not the scenario, so the cure is here.
//
// ⚠ ONE COUNTER, CHECKED AT EVERY PUBLICATION POINT — never merely at the end.
// The status loop awaits per row, so a newer refresh can start midway through
// it; a guard that only checked before the final render would still let a stale
// pass do all its work and win.
let contactsRefreshGen = 0;

async function refreshContacts() {
  const gen = ++contactsRefreshGen;
  let rows = null;
  try {
    rows = await invoke("contact_list");
  } catch (_) {
    return;                       // locked or unavailable: say nothing, change nothing
  }
  if (gen !== contactsRefreshGen) return;   // superseded before we published anything
  const status = {};
  for (const row of rows) {
    try {
      status[row.alias] = await invoke("connect_status", { peer: row.alias });
    } catch (_) { /* one peer's status must not cost the whole pane */ }
    if (gen !== contactsRefreshGen) return; // superseded mid-loop
  }
  let outstanding = contactsOutstanding;
  try {
    const invites = await invoke("invite_list");
    const now = Math.floor(Date.now() / 1000);
    outstanding = invites.filter(
      (i) => i.state === "active" && i.expiry > now
    ).length;
  } catch (_) { /* the hint simply does not move */ }
  if (gen !== contactsRefreshGen) return;   // superseded before publication
  // PUBLISH, all at once, only as the newest refresh.
  contactsRows = rows;
  contactsStatus = status;
  contactsOutstanding = outstanding;
  renderContactsList();
  renderContactDetail();
}

// ── R1: THE AUTO-CONNECT SCAN CLASS ──────────────────────────────────────────
//
// While ANY invite is outstanding, the beat that already runs also watches that
// invite's drop-box. An empty slot is a no-op — MEASURED at M3, on the real
// relay: it mutates nothing and takes no lease, which is what makes running it
// every beat safe rather than merely cheap.
//
// ⚠⚠ AN UNLABELLED INVITE CANNOT AUTO-CONNECT, AND THE SKIP IS COUNTED RATHER
// THAN SILENT. R1 says `alias := the invite's own label`, but the mint's own
// caption reads "Who is this invite for? (optional — stays on this device)", so
// a blessed GUI flow produces invites with NO label and therefore no alias to
// provision under. Those are skipped and tallied in `marks.unlabelled`, so the
// gap is observable instead of being a blessed flow that quietly never
// completes. The disposition is the operator's, and it is declared at STOP 2.
async function autoConnectClass(marks) {
  let relayUrl = "";
  try {
    const cfg = await invoke("relay_config_get");
    relayUrl = cfg.relay_url || "";
  } catch (_) { return marks; }
  if (relayUrl === "") return marks;

  let invites = [];
  try { invites = await invoke("invite_list"); } catch (_) { return marks; }

  const now = Math.floor(Date.now() / 1000);
  for (const inv of invites) {
    // EQUALITY on the extracted value, never a substring: `active` is not a
    // prefix of anything here today, and relying on that is how the 187-day
    // prefix lesson happens again. Revoked, redeemed, expired and creating
    // invites are never scanned — I2' is the arm that proves it.
    if (inv.state !== "active") continue;
    if (inv.expiry <= now) continue;
    if (!inv.label) { marks.unlabelled += 1; continue; }

    marks.pending += 1;
    marks.attempted += 1;
    try {
      const fp = await invoke("invite_accept", {
        selfLabel: null, inviteId: inv.invite_id, alias: inv.label, max: 1,
      });
      marks.scanned += 1;
      // `null` is the EMPTY-SLOT sentinel — nobody has redeemed yet. Only a
      // fingerprint means a handshake completed and a contact now exists.
      if (fp !== null && fp !== undefined) {
        marks.finished += 1;
        contactsNewBadge.add(inv.label);
      }
    } catch (_) {
      marks.scanned += 1;
      marks.failed += 1;
    }
  }
  return marks;
}

// Rung 1's class list. The handshake-poll class joins it when a GUI handshake surface exists
// and `ENG-0198` can report its own no-op; until then the omission is stated, not hidden.
const SCAN_CLASSES = [finishScanClass, autoConnectClass];

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

// NA-0774 -- FIX (b), THE SUPPRESSION POINT, AND WHY IT IS HERE AND NOT ROUND
// `relayScan`. A scan's RERUN can carry a DIFFERENT source than the pass that
// started it: `relayScan` stores the pending event and replays it through a
// second `relayScanOnce(next)`, so a tick-started scan can rerun for a
// user-sourced trigger and vice versa. A flag scoped to `relayScan` would
// silence that user rerun. Scoped to the PASS, quietness follows `ev.source`,
// which is the thing that actually decides it.
// ⚠ WHY A SCOPED FLAG AND NOT AN ARGUMENT ON EVERY CALL. The sec 2(d)
// enumeration is six commands (relay_config_get x2, contact_list, invite_list,
// connect_status, invite_finish, invite_accept) reached through two scan
// classes that receive `marks`, not `ev` -- threading the source to each call
// means changing both class signatures and every call site, and MISSING ONE IS
// SILENT. The flag covers every call reachable from the pass, including any
// added later, at one edit point. That is the "scoped flag" the brief offers.
// ⚠ STATED BOUND, NOT HIDDEN: `await` yields, so a user-sourced call made
// DURING one of this pass's awaits is also suppressed. The window is one scan
// pass. Eliminating it needs per-call context the platform does not give us
// without threading the source; it is recorded as a known bound rather than
// designed around.
async function relayScanOnce(ev) {
  let marks = { why: ev.source, at: ev.at, scanned: 0, finished: 0, pending: 0,
                attempted: 0, failed: 0, unlabelled: 0, changed: 0 };
  const quiet = ev && ev.source === "tick";
  if (quiet) tickQuietDepth += 1;
  try {
    for (const cls of SCAN_CLASSES) marks = await cls(marks);
    // NA-0776 (3.3): the notice refresh piggybacks on this pass -- no new timer. ⚠ IT
    // MUST SIT INSIDE THE QUIET SCOPE. Placed after the `finally` below, its `invoke`
    // runs with `tickQuietDepth` back at zero and the busy indicator flashes on every
    // tick -- which is ENG-0271's cure undone. Caught by f_p_tick_quiet_busy, the arm
    // that exists for exactly this, and it is the reason MINOR-12 warned about a notice
    // refresh riding the tick at all.
    await refreshNotices();
  } finally {
    if (quiet) tickQuietDepth -= 1;
  }
  recordScanOutcome(ev, marks);
  return marks;
}

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
    marks = await relayScanOnce(ev);
    // ⚠⚠ AT MOST ONE RERUN, AND THE BOUND IS THE WHOLE POINT — this is a rerun
    // BIT, not a queue. A first draft re-read the pending slot in a loop, which
    // LOOKED like the same thing and was not: whenever a beat is shorter than a
    // scan takes, the timer refills the slot faster than the loop drains it and
    // the loop NEVER TERMINATES. That is not hypothetical — it hung the harness
    // under the test seam's 250 ms beat, first on CI and then reproducibly here.
    // Production tempi (B >= 20 s) would never have shown it, which is precisely
    // why the seam exists. One extra pass is also all that is CORRECT: the rerun
    // re-reads current state, so it already covers every trigger that arrived
    // while the first pass ran.
    const next = relayScanPending;
    relayScanPending = null;
    if (next) {
      relayScanRerunCount += 1;
      marks = await relayScanOnce(next);
      // Triggers arriving during the rerun are DROPPED, deliberately: the rerun
      // has just re-read the same state they would ask about.
      relayScanPending = null;
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
  // ⚠ F4(ii)/I5: re-render ONLY when the scan actually changed contact state.
  // `marks.finished` counts completed handshakes — the one signal that means a
  // row's state moved. Re-rendering on every scan would repaint a static list
  // every beat; re-rendering on none would leave an auto-created contact
  // invisible until the user clicked something, which is the whole point of
  // the tick.
  if (marks.finished > 0 || marks.changed > 0) {
    refreshContacts();
  }
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
// ── NA-0766 (`D-0043`) — ITEMS 2, 3 AND 15: ONE EXIT, AND IT IS A CLOSE ──────────────
// The code-entry view's X and Back are gone and a full-width Close sits beneath a full-width
// Connect. ⚠ THE THREE ITEMS ARE LOAD-BEARING ON EACH OTHER: removing the X and the Back before
// this Close existed would have left this screen with NO visible exit at all -- which is the
// state NA-0765 found it in and cured with an X. Close reuses `closeRedeemModal`, the one
// dismissal Escape and the scrim already take, so all three agree by construction.
byId("btn-redeem-close3").addEventListener("click", closeRedeemModal);
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

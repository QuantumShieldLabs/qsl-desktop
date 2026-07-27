//! The launch state machine (D595, normative): on every launch the app
//! resolves exactly one state from its own app-scoped data dir. The wizard is
//! a STATE, not a flow; there is NO "onboarding complete" flag anywhere —
//! everything derives from what exists (vault creation is atomic;
//! identity_ensure is idempotent).
//!
//!   S0 — no vault exists            -> onboarding wizard, step 1
//!
//!   ⛔ SUPERSEDED 2026-07-26 by NA-0680 / D-0018:
//!   ~~S1 — vault exists, identity not -> UNLOCK first, wizard resumes step 2~~
//!   ~~S2 — vault and identity exist   -> UNLOCK, then the main window~~
//!
//!   S1 — vault exists AND the onboarding identity step is UNFINISHED
//!        -> UNLOCK first, then the wizard resumes at step 2.
//!        Unfinished means EITHER no identity record, OR an identity record
//!        with no `settings.json` beside it.
//!   S2 — vault exists AND the identity step FINISHED
//!        -> UNLOCK, then the main window
//!        (slice A: always honestly "no server configured")
//!
//! ⚠ WHY THE S1/S2 DISCRIMINATOR CHANGED — a ratified-contract revision,
//! authorised explicitly (D-0018), not a resolver tweak.
//!
//! D595 defined S2 as "vault and identity exist", and that was CORRECT WHEN
//! WRITTEN: a nameless identity was a valid completed state — the wizard's own
//! copy read "leave empty to be shown as You". **R-7 (D615, NA-0680) made the
//! name MANDATORY to leave the identity step.** That superseded the premise
//! without updating the definition, and the gap is a real regression: the
//! identity record is written when the step OPENS (`identity_ensure` ->
//! `identity_self_kem_keypair` -> `identity_write_public_record`), so killing
//! the app between the step opening and Continue left a keypair on disk with
//! no name, resume resolved S2, and the user landed in main with R-7's gate
//! silently bypassed, displaying as "You". Tracked as ENG-0076.
//!
//! ⚠ THIS STILL ADDS NO "ONBOARDING COMPLETE" FLAG. The state is derived from
//! what exists, exactly as D595 requires; only the set of things consulted grew
//! from {vault, identity} to {vault, identity, settings}. The alternative
//! signal — `self_alias` being absent — was RULED OUT: `skip_serializing_if =
//! "String::is_empty"` omits an empty alias, so key-absent also matches "name
//! cleared in Settings" and matches every profile created before R-7, which
//! D615's F4 forbids re-routing through onboarding.

use crate::paths::{settings_file, vault_file};
use std::path::Path;

pub const SELF_LABEL: &str = "self";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchState {
    S0,
    S1,
    S2,
}

impl LaunchState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LaunchState::S0 => "s0",
            LaunchState::S1 => "s1",
            LaunchState::S2 => "s2",
        }
    }
}

/// F2 (D595 as approved): the S0/S1 discriminator is an app-level existence
/// probe of the vault store file under the app-owned QSC_CONFIG_DIR; the
/// S1/S2 discriminator is the read-only, no-unlock identity public record.
/// An unreadable/corrupt identity record resolves to S1 (unlock first, then
/// the identity step surfaces the error honestly).
pub fn resolve_launch_state(data_dir: &Path) -> LaunchState {
    if !vault_file(data_dir).exists() {
        return LaunchState::S0;
    }
    match qsc::identity::identity_read_self_public(SELF_LABEL) {
        Ok(Some(_)) if settings_file(data_dir).exists() => LaunchState::S2,
        Ok(Some(_)) => LaunchState::S1,
        _ => LaunchState::S1,
    }
}

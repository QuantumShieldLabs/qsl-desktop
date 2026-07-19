//! The launch state machine (D595, normative): on every launch the app
//! resolves exactly one state from its own app-scoped data dir. The wizard is
//! a STATE, not a flow; there is NO "onboarding complete" flag anywhere —
//! everything derives from what exists (vault creation is atomic;
//! identity_ensure is idempotent).
//!
//!   S0 — no vault exists            -> onboarding wizard, step 1
//!   S1 — vault exists, identity not -> UNLOCK first, wizard resumes step 2
//!   S2 — vault and identity exist   -> UNLOCK, then the main window
//!        (slice A: always honestly "no server configured")

use crate::paths::vault_file;
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
        Ok(Some(_)) => LaunchState::S2,
        _ => LaunchState::S1,
    }
}

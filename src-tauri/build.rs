//! NA-0776 (spec v2 sec 3.4) -- the build-identity stamp, and the two fields it
//! deliberately does NOT carry.
//!
//! `QSLD_BUILD_COMMIT` is emitted as a rustc env so `app_info` can report which build a
//! flight flew (ENG-0275). It is read with `option_env!`, never `env!`: `env!` on an
//! absent variable is a COMPILE ERROR and so cannot be a red arm (cold read MAJOR-10).
//!
//! ⚠ WHY THERE IS NO `dirty` FLAG AND NO BUILD TIMESTAMP (cold read MAJOR-6). build.rs
//! re-runs only when a DECLARED trigger changes. `.git/HEAD` and the resolved ref say
//! nothing about the WORKING TREE, so a `dirty=false` stamped on a clean tree would
//! survive edits and ship over a dirty one -- a FALSE CLEAN, which is believed. Keeping
//! it truthful needs a trigger over the whole tree, which defeats build caching. A build
//! timestamp has the same defect and worse optics: it is when build.rs last RAN. A field
//! that is believed and can be wrong is worse than an absent field -- this cure's own
//! premise -- so neither is emitted.
//!
//! The commit field's realistic failure is OVER-REBUILDING, not silent staleness: cargo
//! re-runs a script whose declared path is absent (the packed-refs case).

use std::process::Command;

fn main() {
    stamp_build_commit();
    tauri_build::build()
}

/// Resolve a git path THROUGH git rather than by string-joining `.git`: in a worktree
/// `.git` is a FILE containing `gitdir:`, so `../.git/HEAD` is not a path at all
/// (cold read MAJOR-6). This house works in exactly such clones.
fn git_path(arg: &str) -> Option<String> {
    let out = Command::new("git").args(["rev-parse", "--git-path", arg]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let p = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if p.is_empty() { None } else { Some(p) }
}

fn stamp_build_commit() {
    // Re-run when HEAD moves. On a detached HEAD (what actions/checkout produces) HEAD
    // holds the raw sha and this trigger alone is sufficient; on a branch, HEAD never
    // changes across commits and the ref file does, so both are declared.
    if let Some(head) = git_path("HEAD") {
        println!("cargo:rerun-if-changed={head}");
        if let Ok(raw) = std::fs::read_to_string(&head) {
            if let Some(refname) = raw.strip_prefix("ref: ") {
                if let Some(refpath) = git_path(refname.trim()) {
                    println!("cargo:rerun-if-changed={refpath}");
                }
            }
        }
    }
    // Emit NOTHING when git is unavailable or this is not a repository: option_env!
    // then yields None and `app_info` reports the literal "unknown". Never a fabricated
    // or empty sha.
    if let Ok(out) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if out.status.success() {
            if let Ok(sha) = String::from_utf8(out.stdout) {
                let sha = sha.trim();
                if sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()) {
                    println!("cargo:rustc-env=QSLD_BUILD_COMMIT={sha}");
                }
            }
        }
    }
}

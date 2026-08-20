// HANDWRITE-BEGIN gap="missing-generator:logic:physical-filesystem-usage" tracker="#2947" reason="Portable, safe physical filesystem usage sampling for capacity-aware scheduling."
//! Physical filesystem usage for capacity-aware scheduling, via `statvfs`.
//!
//! The three numbers do not add up, on purpose. `used_bytes` is
//! `(f_blocks - f_bfree) * f_frsize` and `available_bytes` is
//! `f_bavail * f_frsize`, and `f_bfree` (free blocks) exceeds `f_bavail` (free to
//! an unprivileged writer) by whatever the filesystem holds in reserve. So
//! `used + available < total` on a filesystem with a reserve, and a scheduler
//! that computes headroom as `total - used` will promise space it cannot get.
//! **`available_bytes` is the only one of the three that answers "can I write
//! this?"**
//!
//! Every multiplication saturates, so a nonsensical `statvfs` clamps at
//! `u64::MAX` instead of wrapping into a small number that looks plausible.
//!
//! The sample describes the filesystem *carrying* `path`, not the subtree at
//! `path`: a quota or a bind mount below it is invisible here. The path must
//! exist -- `statvfs` on a missing path is an error, not a zeroed sample.
use std::path::Path;

use anyhow::{Context, Result};

/// A point-in-time physical filesystem usage sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilesystemUsage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

/// Sample physical filesystem usage for the filesystem carrying `path` using statvfs.
pub fn filesystem_usage(path: impl AsRef<Path>) -> Result<FilesystemUsage> {
    let stat = rustix::fs::statvfs(path.as_ref())
        .with_context(|| format!("statvfs failed for path {}", path.as_ref().display()))?;
    let total_bytes = stat.f_blocks.saturating_mul(stat.f_frsize);
    let used_bytes = stat
        .f_blocks
        .saturating_sub(stat.f_bfree)
        .saturating_mul(stat.f_frsize);
    let available_bytes = stat.f_bavail.saturating_mul(stat.f_frsize);
    Ok(FilesystemUsage {
        total_bytes,
        used_bytes,
        available_bytes,
    })
}
// HANDWRITE-END

// HANDWRITE-BEGIN gap="missing-generator:logic:physical-filesystem-usage" tracker="#2947" reason="Portable, safe physical filesystem usage sampling for capacity-aware scheduling."
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

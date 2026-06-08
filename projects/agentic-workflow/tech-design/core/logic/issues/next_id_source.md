---
id: sdd-logic-issues-next-id-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue/runtime boundary logic projects AW workflow state through configured external clients."
---

# Issue Next ID Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/next_id.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `allocate_next_id` | projects/agentic-workflow/src/issues/next_id.rs | function | pub | 30 | allocate_next_id(project_root: &Path, seed_floor: u64) -> Result<u64> |
| `read_counter` | projects/agentic-workflow/src/issues/next_id.rs | function | pub | 67 | read_counter(path: &Path) -> Result<Option<u64>> |
| `seed_counter` | projects/agentic-workflow/src/issues/next_id.rs | function | pub | 110 | seed_counter(project_root: &Path, seed_floor: u64) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/issues/next_id.rs -->
```rust
//! Phase B: monotonic id allocator backed by `.aw/issues/.next-id`.
//!
//! @spec projects/agentic-workflow/tech-design/core/logic/issues/slug-and-id.md#schema
//!
//! Used by the local lifecycle backend and by `aw wi migrate-slugs`
//! to mint stable u64 ids for issues that pre-date the
//! platform-id-as-canonical-key contract.
//!
//! The counter file is a single-line decimal u64. Allocation is
//! atomic via:
//!   1. fs2 file lock on `.aw/issues/.next-id.lock`
//!   2. read counter, increment, write back via temp+rename
//!
//! Concurrent processes serialise on the lock; the counter survives
//! process restarts.

use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Allocate the next id. Initialises the counter to `seed_floor + 1` on
/// first call (where `seed_floor` is the max id discovered by walking
/// existing issue files; pass 0 if no scan was performed).
///
/// Concurrent-safe via an exclusive file lock on a sibling `.lock` file.
/// @spec projects/agentic-workflow/tech-design/core/logic/issues/next_id_source.md#source
pub fn allocate_next_id(project_root: &Path, seed_floor: u64) -> Result<u64> {
    let issues_dir = project_root.join(".aw/issues");
    fs::create_dir_all(&issues_dir)
        .with_context(|| format!("creating {}", issues_dir.display()))?;

    let counter_path = issues_dir.join(".next-id");
    let lock_path = issues_dir.join(".next-id.lock");

    let lock_file = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("opening lock {}", lock_path.display()))?;

    fs2::FileExt::lock_exclusive(&lock_file)
        .with_context(|| format!("acquiring exclusive lock on {}", lock_path.display()))?;

    let next = (|| -> Result<u64> {
        let current = read_counter(&counter_path)?;
        let value = match current {
            Some(n) => n.checked_add(1).context(".next-id counter overflow")?,
            None => seed_floor
                .checked_add(1)
                .context("seed_floor + 1 overflows u64")?,
        };
        write_counter_atomic(&counter_path, value)?;
        Ok(value)
    })();

    let _ = fs2::FileExt::unlock(&lock_file);
    next
}

/// Read the current counter value, returning `None` if the file is missing
/// or empty.
/// @spec projects/agentic-workflow/tech-design/core/logic/issues/next_id_source.md#source
pub fn read_counter(path: &Path) -> Result<Option<u64>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("opening {}", path.display()))?;
    let mut body = String::new();
    file.read_to_string(&mut body)?;
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let n: u64 = trimmed
        .parse()
        .with_context(|| format!(".next-id at {} is not a u64: {trimmed:?}", path.display()))?;
    Ok(Some(n))
}

/// Write the counter atomically via temp+rename. Caller must hold the lock.
fn write_counter_atomic(path: &Path, value: u64) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp: PathBuf = parent.join(format!(".next-id.tmp.{}", std::process::id()));
    {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)
            .with_context(|| format!("creating temp {}", tmp.display()))?;
        writeln!(f, "{value}")?;
        f.flush()?;
        f.seek(SeekFrom::Start(0)).ok();
    }
    fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Initialise the counter to `seed_floor + 1` if it does not yet exist.
/// Idempotent — does nothing if the file already has a value.
/// @spec projects/agentic-workflow/tech-design/core/logic/issues/next_id_source.md#source
pub fn seed_counter(project_root: &Path, seed_floor: u64) -> Result<()> {
    let issues_dir = project_root.join(".aw/issues");
    fs::create_dir_all(&issues_dir)?;
    let counter_path = issues_dir.join(".next-id");
    if read_counter(&counter_path)?.is_some() {
        return Ok(());
    }
    write_counter_atomic(&counter_path, seed_floor + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn allocate_first_call_uses_seed_floor_plus_one() {
        let tmp = TempDir::new().unwrap();
        let id = allocate_next_id(tmp.path(), 100).unwrap();
        assert_eq!(id, 101);
    }

    #[test]
    fn allocate_two_sequential_calls_increment() {
        let tmp = TempDir::new().unwrap();
        let a = allocate_next_id(tmp.path(), 0).unwrap();
        let b = allocate_next_id(tmp.path(), 999).unwrap(); // seed_floor ignored on second call
        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }

    #[test]
    fn counter_survives_across_processes_simulated_via_reload() {
        let tmp = TempDir::new().unwrap();
        let _ = allocate_next_id(tmp.path(), 50).unwrap(); // 51
        let _ = allocate_next_id(tmp.path(), 0).unwrap(); // 52
                                                          // Read directly to simulate a separate process inspecting state.
        let counter = read_counter(&tmp.path().join(".aw/issues/.next-id"))
            .unwrap()
            .unwrap();
        assert_eq!(counter, 52);

        // Next allocation continues from 53, not from seed_floor.
        let next = allocate_next_id(tmp.path(), 0).unwrap();
        assert_eq!(next, 53);
    }

    #[test]
    fn seed_counter_idempotent() {
        let tmp = TempDir::new().unwrap();
        seed_counter(tmp.path(), 10).unwrap();
        let first = read_counter(&tmp.path().join(".aw/issues/.next-id"))
            .unwrap()
            .unwrap();
        seed_counter(tmp.path(), 999).unwrap(); // ignored — already seeded
        let second = read_counter(&tmp.path().join(".aw/issues/.next-id"))
            .unwrap()
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(first, 11);
    }

    #[test]
    fn missing_counter_file_reads_as_none() {
        let tmp = TempDir::new().unwrap();
        let counter = read_counter(&tmp.path().join("nope")).unwrap();
        assert!(counter.is_none());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/next_id.rs
    action: modify
    section: source
    impl_mode: codegen
    description: "Source template owns the full issue next-id allocator module."
```

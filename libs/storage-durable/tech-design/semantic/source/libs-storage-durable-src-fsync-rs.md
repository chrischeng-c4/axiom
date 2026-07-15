---
id: libs-storage-durable-src-fsync-rs
summary: Lossless rust-source-unit coverage for `libs/storage-durable/src/fsync.rs`.
capability_refs:
  - id: shared-storage-durable-contract
    role: primary
    gap: shared-storage-durable-contract
    claim: shared-storage-durable-contract
    coverage: full
    rationale: "The source unit implements the shared fsync policy."
fill_sections: [overview, source, changes]
---

# Standardized libs/storage-durable/src/fsync.rs

## Overview
<!-- type: overview lang: markdown -->

Lossless rust-source-unit coverage for `libs/storage-durable/src/fsync.rs`.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `FsyncPolicy` | libs/storage-durable/src/fsync.rs | enum | pub | Always, EverySec, Interval, Os |
| `should_sync_immediately` | libs/storage-durable/src/fsync.rs | method | pub | should_sync_immediately(self) -> bool |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
/// Shared flush policy for local durable files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FsyncPolicy {
    /// Flush and fsync at the durability boundary.
    #[default]
    Always,
    /// Flush writes and let the caller batch fsyncs on a timer.
    EverySec,
    /// Flush writes and let a higher-level storage owner decide the interval.
    Interval,
    /// Rely on the OS page cache.
    Os,
}

impl FsyncPolicy {
    pub fn should_sync_immediately(self) -> bool {
        matches!(self, Self::Always)
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/storage-durable/src/fsync.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/storage-durable/src/fsync.rs`.
```

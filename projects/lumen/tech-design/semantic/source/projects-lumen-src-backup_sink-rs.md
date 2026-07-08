---
id: projects-lumen-src-backup_sink-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/backup_sink.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/backup_sink.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `LocalFsSink` | projects/lumen/src/backup_sink.rs | struct | pub | 48 |  |
| `new` | projects/lumen/src/backup_sink.rs | function | pub | 55 | new(root: impl Into<PathBuf>, prefix: impl Into<String>) -> Result<Self> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-backup_sink-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Lumen backup compatibility exports.
//!
//! Lumen produces a consistent snapshot from its engine / raft state machine.
//! Destination schema, sink implementations, retention, and runner primitives
//! live in `libs/service-backup` so Lumen, Keep, Relay, and Loom share one
//! backup contract instead of each carrying a bespoke sink.
//!
pub use service_backup::{
    run_backup_once, sink_from_destination, BackupDestination, BackupObject, BackupPolicy,
    BackupRunResult, BackupSink, LocalFsSink, RetentionPolicy, UnsupportedCloudSink,
};
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/backup_sink.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/backup_sink.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```

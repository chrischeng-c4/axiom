---
id: projects-lumen-src-log_entry-rs
capability_refs:
  - id: "search"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit defines the committed-mutation write-log vocabulary that flows through the active lumen write path."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/log_entry.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/log_entry.rs` captured as a per-file rust-source-unit during lumen td_ast standardization.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `RaftLogEntry` | projects/lumen/src/log_entry.rs | enum | pub | |
| `RaftLogResponse` | projects/lumen/src/log_entry.rs | struct | pub | |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-log_entry-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The write-log entry vocabulary: the committed-mutation enum that flows
//! through the active write path (NATS/embedded WAL → `WriteCoordinator` →
//! `Engine::apply_raft_entry`) and the apply-step response marker.
//!
//! These are pure serde data types with no consensus dependency. They live in
//! their own always-compiled module so the load-bearing write path stays free of
//! any heavier replication machinery (NATS JetStream is the replication
//! substrate). The `RaftLogEntry` name is retained for historical continuity —
//! every variant still maps 1:1 to an `Engine::*` method.

use serde::{Deserialize, Serialize};

use crate::types::{CreateCollectionRequest, FieldSpec, IndexRequest};

/// One committed mutation against the lumen storage engine.
///
/// Every variant maps 1:1 to the matching `Engine::*` method; see
/// [`crate::storage::Engine::apply_raft_entry`].
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-log_entry-rs.md#source
pub enum RaftLogEntry {
    CreateCollection {
        collection_id: String,
        req: CreateCollectionRequest,
    },
    Index {
        collection_id: String,
        req: IndexRequest,
    },
    Delete {
        collection_id: String,
        external_id: String,
        /// `None` means "all fields for this `external_id`".
        field: Option<String>,
    },
    DropCollection {
        collection_id: String,
        force: bool,
    },
    AddField {
        collection_id: String,
        field_name: String,
        spec: FieldSpec,
    },
    DropField {
        collection_id: String,
        field_name: String,
    },
}

/// Response type returned by the state-machine apply step. The engine's RwLock
/// makes a write visible to subsequent reads, so nothing needs to come back —
/// this is a unit-shaped marker (the `R` of the consensus type config).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-log_entry-rs.md#source
pub struct RaftLogResponse;
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/log_entry.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/log_entry.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```

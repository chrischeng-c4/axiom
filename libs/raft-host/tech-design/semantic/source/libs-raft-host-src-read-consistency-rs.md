---
id: libs-raft-host-src-read-consistency-rs
summary: Lossless rust-source-unit coverage for `libs/raft-host/src/read_consistency.rs`.
capability_refs:
  - id: shared-raft-host-driver
    role: primary
    claim: shared-raft-host-driver-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Host library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/raft-host/src/read_consistency.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/raft-host/src/read_consistency.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `READ_CONSISTENCY_HEADER` | libs/raft-host/src/read_consistency.rs | const | pub | 14 | pub const READ_CONSISTENCY_HEADER: &str = "x-read-consistency"; |
| `ReadConsistency` | libs/raft-host/src/read_consistency.rs | enum | pub | 20 | pub enum ReadConsistency { |
| `from_header` | libs/raft-host/src/read_consistency.rs | function | pub | 33 | pub fn from_header(raw: Option<&str>) -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The `X-Read-Consistency` request-header contract.
//!
//! Every raft_core service's replica-read path parses the same header to
//! decide how stale a follower may be before it's allowed to answer a read:
//! `leader` (only the shard leader), `any` (potentially stale), or
//! `bounded(<ms>)` (a follower may answer if its replication lag is under the
//! bound). Centralized here so lumen/keep/relay/loom share one parser instead
//! of re-declaring the same fallback rules.

use serde::{Deserialize, Serialize};

/// The request header name every raft_core service reads to pick a read's
/// consistency requirement.
pub const READ_CONSISTENCY_HEADER: &str = "x-read-consistency";

/// Read-consistency requirement set by a client request via the
/// [`READ_CONSISTENCY_HEADER`] header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadConsistency {
    /// Default — only the shard leader may answer.
    Leader,
    /// A follower may answer if its replication lag is below the bound
    /// (ms). Carried in `bounded(ms)` form on the wire.
    Bounded(u64),
    /// Any replica is allowed (potentially stale).
    Any,
}

impl ReadConsistency {
    /// Parse a [`READ_CONSISTENCY_HEADER`] value. A missing or unrecognized
    /// value falls back to [`ReadConsistency::Leader`], the safest setting.
    pub fn from_header(raw: Option<&str>) -> Self {
        let Some(v) = raw else {
            return Self::Leader;
        };
        let v = v.trim().to_ascii_lowercase();
        if v == "leader" {
            Self::Leader
        } else if v == "any" {
            Self::Any
        } else if let Some(ms) = v
            .strip_prefix("bounded(")
            .and_then(|t| t.strip_suffix(')'))
            .and_then(|n| n.parse::<u64>().ok())
        {
            Self::Bounded(ms)
        } else {
            // Unknown values fall back to the safest setting.
            Self::Leader
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_consistency_from_header() {
        assert_eq!(
            ReadConsistency::from_header(Some("leader")),
            ReadConsistency::Leader
        );
        assert_eq!(
            ReadConsistency::from_header(Some("any")),
            ReadConsistency::Any
        );
        assert_eq!(
            ReadConsistency::from_header(Some("Bounded(250)")),
            ReadConsistency::Bounded(250)
        );
        assert_eq!(
            ReadConsistency::from_header(Some("gibberish")),
            ReadConsistency::Leader
        );
        assert_eq!(ReadConsistency::from_header(None), ReadConsistency::Leader);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/raft-host/src/read_consistency.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/raft-host/src/read_consistency.rs` captured during libs codegen standardization.
```

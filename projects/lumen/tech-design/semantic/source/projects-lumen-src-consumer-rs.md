---
id: projects-lumen-src-consumer-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/consumer.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/consumer.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `ShardRouter` | projects/lumen/src/consumer.rs | struct | pub |
| `index_url` | projects/lumen/src/consumer.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-consumer-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Consumer adapter glue.
//!
//! lumen does not own the source of truth and does not bundle an
//! event-pipeline subscriber. A consumer is whichever upstream the
//! caller wires up — AlloyDB CDC, Postgres logical replication, Kafka,
//! direct POST from an application — that ultimately calls the lumen
//! `POST /collections/{id}/index` endpoint.
//!
//! This module provides the shard-routing helper that any such consumer
//! needs: given a `collection_id`, compute which lumen pod to POST to.
//! Concrete adapter examples live under `examples/` (e.g.
//! `consumer_pg_logical.py` — Postgres logical replication → `POST /index`).
//!
//! A write is published to the configured log and folded in by serving nodes.
//! In primary-replica mode, clients should target the shard leader (or follow
//! the serving API's leader redirect/retry contract); in standalone mode the
//! single pod is the leader; in explicit broker mode any broker-connected pod
//! can accept the write.

use crate::routing::shard_index;

#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-consumer-rs.md#source
pub struct ShardRouter {
    pub shard_count: u32,
    pub lumen_host: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-consumer-rs.md#source
impl ShardRouter {
    /// URL of the `POST /index` endpoint for `collection_id` on the
    /// correct shard. In primary-replica mode this resolves the shard service;
    /// callers may need to follow the serving API's leader redirect/retry
    /// contract when the chosen pod is not currently leader.
    pub fn index_url(&self, collection_id: &str) -> String {
        let shard = shard_index(collection_id, self.shard_count);
        format!(
            "http://lumen-{shard}.{host}:8080/collections/{collection_id}/index",
            host = self.lumen_host
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_url_uses_pod_dns() {
        let r = ShardRouter {
            shard_count: 3,
            lumen_host: "lumen.svc.cluster.local".into(),
        };
        let url = r.index_url("users");
        assert!(url.starts_with("http://lumen-"));
        assert!(url.contains(".lumen.svc.cluster.local:8080/"));
        assert!(url.ends_with("/collections/users/index"));
    }
}
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/consumer.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/consumer.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```

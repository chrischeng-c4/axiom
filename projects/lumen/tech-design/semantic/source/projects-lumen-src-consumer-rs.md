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
//! A write is published to the log and folded in by every serving node;
//! there is no write leader to forward to (openraft was retired — NATS
//! JetStream is the replication substrate), so any replica of the target
//! shard accepts the `POST /index`.

use crate::routing::shard_index;

#[derive(Debug, Clone)]
pub struct ShardRouter {
    pub shard_count: u32,
    pub lumen_host: String,
}

impl ShardRouter {
    /// URL of the `POST /index` endpoint for `collection_id` on the
    /// correct shard. Any replica of that shard is fine — a write is
    /// published to the log, not applied at the receiving node, so there
    /// is no leader to forward to.
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

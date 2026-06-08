---
id: cclab-core-global-todos
title: cclab Core Global TODOs
status: active
component: roadmap
type: backlog
main_spec_ref: "cclab-core/logic/roadmap/global-todos.md"
fill_sections: [overview, requirements, scenarios, changes]
---

# Global TODOs

## Overview
<!-- type: overview lang: markdown -->

This backlog replaces the legacy `03-global-todos.md` rollup, which mixed
completed audit notes, PostgreSQL ORM roadmap items, KV roadmap items, and
merge-conflict residue. The active source of truth is now limited to open,
cross-cutting data-bridge and cclab-core follow-up work.

## Requirements
<!-- type: schema lang: yaml -->

```yaml
requirements:
  R1:
    text: "The backlog MUST track only open or intentionally retained follow-up work."
    priority: high
    status: active
  R2:
    text: "Completed audit and migration notes MUST remain summarized as history, not repeated as active tasks."
    priority: medium
    status: active
  R3:
    text: "KV persistence follow-up MUST cover server startup recovery, graceful shutdown, integration tests, and benchmarks."
    priority: high
    status: active
  R4:
    text: "Production-readiness follow-up SHOULD cover active TTL expiration, disk spillover, metrics, probes, password auth, and TLS."
    priority: medium
    status: active
  R5:
    text: "Roadmap specs MUST avoid unresolved merge-conflict markers and date-stale status blocks."
    priority: high
    status: active
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: kv_persistence_server_integration
    status: open
    priority: high
    tasks:
      - "Add server flags for data directory, persistence disabling, and fsync interval."
      - "Run recovery before accepting client connections."
      - "Flush and close persistence state during graceful shutdown."
      - "Add write-crash-recover integration coverage."
      - "Benchmark persistence overhead."

  - id: production_readiness
    status: open
    priority: medium
    tasks:
      - "Add active TTL expiration for expired entries."
      - "Add disk spillover or tiered storage for large datasets."
      - "Expose Prometheus metrics and health/readiness probes."
      - "Add Redis-style password authentication."
      - "Add TLS support for remote deployments."

  - id: completed_history
    status: archived
    priority: low
    summary:
      - "PostgreSQL audit items from 2025-12-30 are recorded as completed history."
      - "Relationship, migration, OpenTelemetry, loading-strategy, event, and inheritance roadmap notes are historical unless re-opened by a new issue."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/crates/cclab-core/logic/roadmap/global-todos.md
    action: modify
    section: requirements
    impl_mode: hand-written
    description: "Maintain the active cclab-core/data-bridge backlog without stale completed-task dumps."
  - path: .aw/tech-design/crates/cclab-core/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: "Link the normalized global TODO backlog from the development section."
```

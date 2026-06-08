---
title: Project Roadmap
status: planning
component: general
type: roadmap
main_spec_ref: "cclab-core/logic/roadmap/feature-roadmap.md"
fill_sections: [overview, schema, changes]
---

# Feature Roadmap

## Overview
<!-- type: overview lang: markdown -->

This roadmap tracks the major feature series for `data-bridge` and cclab-core.
Completed items are retained as historical milestones; open items belong in
the active backlog under `global-todos.md` or in dedicated issue specs.

## Schema
<!-- type: schema lang: yaml -->

```yaml
series:
  - id: 1xx
    name: Type Validation System
    status: completed
    focus: "Core type safety and state management with MongoDB focus."
    milestones:
      - "101: Copy-on-write state management"
      - "102: Lazy validation"
      - "103: Fast-path bulk operations"
      - "104: Rust query execution"
      - "105: Type schema extraction"
      - "106: Basic type validation"
      - "107: Complex type validation"
      - "108: Constraint validation"

  - id: 2xx
    name: Performance Optimization
    status: in_progress
    focus: "Maximize throughput and minimize latency."
    milestones:
      - "201+: Bulk operation improvements with Rayon parallelization"
      - "2xx: GIL release optimization"
      - "2xx: Zero-copy deserialization research"

  - id: 9xx
    name: Infrastructure
    status: completed
    focus: "Tooling and core utilities."
    milestones:
      - "901: HTTP client"
      - "902: Test framework"

future_solutions:
  postgres:
    status: planned
    scope:
      - "Async Rust driver for Postgres."
      - "Pydantic model mapping to SQL tables."
  kv_store:
    status: planned
    scope:
      - "Cloud-native simple KV store."
      - "Redis-style value types plus Decimal/Int/Float."
      - "Rust-based engine."
  redis:
    status: planned
    scope:
      - "High-performance caching layer."
      - "Queue implementation."

future_core_features:
  "3xx":
    name: Relations and References
    scope:
      - "Handle Link and BackLink efficiently."
      - "Pre-fetch relations in Rust."
  "4xx":
    name: Query Builder Enhancements
    scope:
      - "Support complex aggregation pipelines."
      - "Support geospatial queries."
  "5xx":
    name: Embedded Documents
    scope:
      - "Support deeply nested document structures."
      - "Support partial updates on nested fields."
  "6xx":
    name: Document Inheritance
    scope:
      - "Support polymorphic storage and retrieval."
  "7xx":
    name: Schema Migrations
    scope:
      - "Support declarative schema changes."
      - "Support Rust-powered data migration scripts."
  "8xx":
    name: Tooling and Developer Experience
    scope:
      - "Add CLI tools for scaffolding."
      - "Add IDE plugins and type stubs."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/crates/cclab-core/logic/roadmap/feature-roadmap.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Maintain cclab-core feature series, future solutions, and future core feature roadmap."
  - path: .aw/tech-design/crates/cclab-core/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: "Link to the normalized feature roadmap."
```

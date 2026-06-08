---
id: pylibs-refactor
type: proposal
version: 2
created_at: 2026-02-24T10:30:04.362295+00:00
updated_at: 2026-02-24T10:30:04.362295+00:00
iteration: 1
scope: minor
spec_plan:
  - id: queue-pyo3-refactor
    title: "Refactor cclab-queue PyO3 Bindings"
    depends: []
    context_refs:
      codebase: ["crates/cclab-queue/src/pyo3_bindings/mod.rs"]
      spec: ["nucleus-architecture", "scheduler-architecture"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 2 }
    affected_code: ["crates/cclab-queue/src/pyo3_bindings/mod.rs"]
  - id: mongo-pyo3-refactor
    title: "Refactor cclab-mongo PyO3 Bindings"
    depends: []
    context_refs:
      codebase: ["crates/cclab-mongo/src/pyo3_bindings/document.rs", "crates/cclab-mongo/src/pyo3_bindings/query.rs"]
      spec: ["nucleus-architecture"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 3 }
    affected_code: ["crates/cclab-mongo/src/pyo3_bindings/document.rs", "crates/cclab-mongo/src/pyo3_bindings/query.rs"]
  - id: titan-test-expansion
    title: "Expand cclab-titan Integration Tests"
    depends: []
    context_refs:
      codebase: ["crates/cclab-titan/tests/", "crates/cclab-titan/src/pool.rs", "crates/cclab-titan/src/error.rs"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 4 }
    affected_code: ["crates/cclab-titan/tests/", "crates/cclab-titan/src/pool.rs", "crates/cclab-titan/src/error.rs"]
  - id: quasar-pyo3-expansion
    title: "Expand cclab-quasar PyO3 Exports for FastAPI Parity"
    depends: []
    context_refs:
      codebase: ["crates/cclab-quasar/src/pyo3_bindings/"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 5 }
    affected_code: ["crates/cclab-quasar/src/pyo3_bindings/"]
  - id: shield-performance-opt
    title: "Optimize cclab-shield JSON-to-Model Performance"
    depends: []
    context_refs:
      codebase: ["crates/cclab-shield/src/"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 7 }
    affected_code: ["crates/cclab-shield/src/"]
  - id: fetch-migration-cleanup
    title: "Complete cclab-http to cclab-fetch Migration"
    depends: []
    context_refs:
      codebase: ["crates/cclab-http/", "crates/cclab-fetch/`Cargo.toml`"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 6 }
    affected_code: ["crates/cclab-http/", "crates/cclab-fetch/", "Cargo.toml"]
history:
  - timestamp: 2026-02-24T10:30:04.362295+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: pylibs-refactor

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((pylibs-refactor))  
    queue
    mongo
    titan
    quasar
    shield
    fetch
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  queue_pyo3_refactor["queue-pyo3-refactor\n codebase: crates/cclab-queue/src/pyo3_bindings/mod.rs\n gaps: codebase_spec#2"]
  mongo_pyo3_refactor["mongo-pyo3-refactor\n codebase: crates/cclab-mongo/src/pyo3_bindings/document.rs, crates/cclab-mongo/src/pyo3_bindings/query.rs\n gaps: codebase_spec#3"]
  titan_test_expansion["titan-test-expansion\n codebase: crates/cclab-titan/tests/, crates/cclab-titan/src/pool.rs, crates/cclab-titan/src/error.rs\n gaps: codebase_spec#4"]
  quasar_pyo3_expansion["quasar-pyo3-expansion\n codebase: crates/cclab-quasar/src/pyo3_bindings/\n gaps: codebase_spec#5"]
  shield_performance_opt["shield-performance-opt\n codebase: crates/cclab-shield/src/\n gaps: codebase_spec#7"]
  fetch_migration_cleanup["fetch-migration-cleanup\n codebase: crates/cclab-http/, crates/cclab-fetch/`Cargo.toml`\n gaps: codebase_spec#6"]

```

## Spec Execution Order

1. **fetch-migration-cleanup** — Complete cclab-http to cclab-fetch Migration
   - code: crates/cclab-http/, crates/cclab-fetch/, Cargo.toml
2. **mongo-pyo3-refactor** — Refactor cclab-mongo PyO3 Bindings
   - code: crates/cclab-mongo/src/pyo3_bindings/document.rs, crates/cclab-mongo/src/pyo3_bindings/query.rs
3. **quasar-pyo3-expansion** — Expand cclab-quasar PyO3 Exports for FastAPI Parity
   - code: crates/cclab-quasar/src/pyo3_bindings/
4. **queue-pyo3-refactor** — Refactor cclab-queue PyO3 Bindings
   - code: crates/cclab-queue/src/pyo3_bindings/mod.rs
5. **shield-performance-opt** — Optimize cclab-shield JSON-to-Model Performance
   - code: crates/cclab-shield/src/
6. **titan-test-expansion** — Expand cclab-titan Integration Tests
   - code: crates/cclab-titan/tests/, crates/cclab-titan/src/pool.rs, crates/cclab-titan/src/error.rs

</proposal>

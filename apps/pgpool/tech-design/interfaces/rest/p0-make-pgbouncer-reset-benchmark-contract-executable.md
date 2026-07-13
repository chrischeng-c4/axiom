---
id: '1617'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgbouncer-always-reset-benchmark-contract
entry: profile
nodes:
  profile: { kind: start, label: "Immutable simple protocol 64 clients 16 backends 30 seconds" }
  config: { kind: process, label: "Render PgBouncer transaction-pool configuration" }
  reset: { kind: process, label: "Set DISCARD ALL and server_reset_query_always 1" }
  pgbouncer: { kind: process, label: "Run complete PgBouncer benchmark leg" }
  pgpool: { kind: process, label: "Run complete pgpool reset-before-idle benchmark leg" }
  compare: { kind: terminal, label: "Compare no-error workloads" }
edges:
  - { from: profile, to: config }
  - { from: config, to: reset }
  - { from: reset, to: pgbouncer }
  - { from: profile, to: pgpool }
  - { from: pgbouncer, to: compare }
  - { from: pgpool, to: compare }
---
flowchart LR
  profile([immutable profile]) --> config[render PgBouncer config]
  config --> reset[DISCARD ALL plus always reset]
  reset --> pgbouncer[PgBouncer no-error leg]
  profile --> pgpool[pgpool reset-before-idle leg]
  pgbouncer --> compare([compare complete workloads])
  pgpool --> compare
```

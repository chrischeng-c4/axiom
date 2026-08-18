---
id: apps-tape-src-bin-tape-bench-rs
coverage_kind: semantic
capability_refs:
  - id: "competitor-performance"
    role: primary
    claim: "topic-replay-competitor-performance-baseline"
    gap: "topic-replay-competitor-performance-baseline"
    coverage: partial
    rationale: "The tape-bench CLI exposes the benchmark report and calibration status to agents and EC gates."
fill_sections: [overview, logic, changes]
---

# Tape Benchmark CLI

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/src/bin/tape-bench.rs` provides a small local benchmark CLI for
the first Tape replay slice. It prints either text or JSON and exits non-zero if
local regression budgets fail or if Tape overclaims external broker wins.

WI #3052 AC1 adds a `durable` subcommand that drives
`tape::bench::run_durable_benchmark` and prints the per-connection-count
samples plus the durable throughput scaling ratio, in the same text/JSON
convention as `run`.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    cli["tape-bench run"] --> report["tape::bench::run_benchmark"]
    report --> output["text or JSON report"]
    report --> verify["verify_report"]
    verify --> pass["exit 0"]
    verify --> fail["exit non-zero with budget/overclaim error"]

    durable_cli["tape-bench durable --events-per-connection --payload-bytes --connections 1,4,16"] --> durable_report["tape::bench::run_durable_benchmark"]
    durable_report --> durable_output["text or JSON: per-connection ops_per_sec + scaling_ratio"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/bin/tape-bench.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "CLI wrapper for Tape local benchmark and calibration-status reporting."
  - path: apps/tape/src/bin/tape-bench.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "WI #3052 AC1: durable subcommand surfaces run_durable_benchmark's per-connection samples and scaling ratio."
```

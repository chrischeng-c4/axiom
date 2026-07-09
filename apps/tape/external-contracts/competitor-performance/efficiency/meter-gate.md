---
id: tape-competitor-performance-meter-ec
summary: Tape's meter-owned performance gate wraps the existing perf_gate + NATS JetStream benchmark binaries in a vat-isolated meter dispatch, mirroring relay's meter-perf pattern.
fill_sections: [e2e-test, tool-contract]
---

# EC: Meter-Dispatched Competitive Performance Gate

Tape already has direct-cargo competitor-performance EC cases
(`tape-competitor-performance-claim-closure`,
`tape-competitor-performance-local-regression-and-calibration-ledger`,
`tape-competitor-performance-nats-jetstream-replay-win`) in `apps/tape/aw.toml`
that dispatch `cargo test` directly. This EC adds a meter-owned, vat-isolated
wrapper around the same `tape_perf_gate` and `tape_vs_nats_jetstream` test
binaries so the output is captured as runtime evidence through meter and the
test workspace/tool binaries stay isolated from the host checkout, matching
relay's `relay-competitor-performance-meter-gate` pattern layered on top of
pre-existing direct-cargo cases.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-meter-performance-gate
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: tape-meter-throughput-ratchet
    category: efficiency
    test_path: apps/tape/tests/benchmark_tape_meter_performance_gate.rs
    command: "cd apps/tape && ../../target/debug/vat run meter-perf"
    assertions:
      - "The local Tape performance regression gate passes for append, replay, and checkpoint operations."
      - "Tape's NATS JetStream local backlog replay win is backed by a real-service benchmark gate."
      - "The gate is executed by meter inside a vat workspace, not by a direct-cargo-only dispatch path."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: tape-meter-performance
    tool: meter
    manifest: meter-tape-performance.toml
    category: efficiency
    command: "cd apps/tape && ../../target/debug/vat run meter-perf"
    native:
      version: 1
      project: tape
      source_contract: tape-meter-performance-gate
      delegate_command: "cd apps/tape && ../../target/debug/vat run meter-perf"
```

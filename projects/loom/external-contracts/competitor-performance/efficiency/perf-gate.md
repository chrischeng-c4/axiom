---
id: loom-competitor-performance-ec
summary: Loom performance evidence is meter-owned and vat-isolated; Celery/Temporal comparison remains dogfood until it is a required gate.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive Workflow Performance

loom's production performance evidence is the meter-owned control-plane gate:
meter runs loom's scheduler / raft / DAG / API-idempotency behaviour suite and a
CPU-time or peak-RSS regression is a non-zero-exit finding. The gate runs inside
vat so report artifacts and transient state never mutate the host checkout.

Competitor comparison — loom vs **Celery** and **Temporal** as DAG orchestrators
(see `projects/loom/benchmark/`) — remains a dogfood work root until those
external services are part of the required gate. loom is a control plane: payload
bytes never traverse it (claim-check via keep), so its performance envelope is
scheduling latency + fan-in/fan-out throughput, not payload bandwidth.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: loom-competitor-performance-meter-gate
    capability_id: competitor-performance
    claim_id: vat-meter-runtime-gate
    contract_id: loom-meter-performance-report
    category: efficiency
    test_path: projects/loom/src/scheduler.rs
    command: "cd projects/loom && ../../target/debug/vat run meter-efficiency"
    assertions:
      - "meter owns the pass/fail evidence for loom's control-plane behaviour + engine gate."
      - "The gate runs inside vat so report artifacts and transient state do not mutate the host checkout."
      - "Celery/Temporal comparison remains dogfood until those peer services are required by the EC."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: loom-meter-performance
    tool: meter
    manifest: meter-loom-performance.toml
    category: efficiency
    delegate_command: "cd projects/loom && ../../target/debug/vat run meter-efficiency"
```

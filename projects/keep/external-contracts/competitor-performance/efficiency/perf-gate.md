---
id: keep-competitor-performance-ec
summary: Keep performance evidence is meter-owned and vat-isolated; Redis/Dragonfly comparison remains dogfood until it is a required gate.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive KV Performance

Keep's production performance evidence is the meter-owned engine/runtime gate.
The current vat runner delegates through meter and keeps the transient benchmark
workspace isolated. Redis/Dragonfly comparison remains a dogfood work root until
the external services are part of the required gate.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: keep-competitor-performance-meter-gate
    capability_id: competitor-performance
    claim_id: vat-meter-runtime-gate
    contract_id: keep-meter-performance-report
    category: efficiency
    test_path: projects/keep/tests/benchmark_keep_competitor_performance_meter_gate.rs
    command: "cd projects/keep && ../../target/debug/vat run meter-efficiency"
    assertions:
      - "meter owns the pass/fail evidence for Keep's performance-relevant API and engine gate."
      - "The gate runs inside vat so report artifacts and transient state do not mutate the host checkout."
      - "Redis/Dragonfly comparison remains dogfood until external peer services are required by the EC."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: keep-meter-performance
    tool: meter
    manifest: meter-keep-performance.toml
    category: efficiency
    command: "cd projects/keep && ../../target/debug/vat run meter-efficiency"
    native:
      version: 1
      project: keep
      source_contract: keep-competitor-performance-meter-gate
      delegate_command: "cd projects/keep && ../../target/debug/vat run meter-efficiency"
```

---
id: relay-competitor-performance-ec
summary: Relay performance evidence is meter-owned and vat-isolated; arena remains a legacy comparison spec, not the production EC dispatch path.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive Broker Performance

Relay's local performance contract is the repeatable work-queue throughput and
ratchet-decision gate. The production EC command runs through vat so the test
workspace and tool binaries are isolated, then delegates to meter so the output
is captured as runtime evidence. The external NATS/RabbitMQ/Redpanda arena spec
remains advisory until those native adapters are part of the required gate.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: relay-competitor-performance-meter-gate
    capability_id: competitor-performance
    claim_id: normalized-win-ratchet-decision-model
    contract_id: relay-meter-throughput-ratchet
    category: efficiency
    test_path: projects/relay/tests/benchmark_relay_competitor_performance_meter_gate.rs
    command: "cd projects/relay && ../../target/debug/vat run meter-perf"
    assertions:
      - "The normalized perf-gate ratchet fails on regression and must-beat loss."
      - "The small-scale append, broadcast, and work-queue lease/ack workloads complete correctly."
      - "The gate is executed by meter inside a vat workspace, not by a legacy arena-only dispatch path."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: relay-meter-performance
    tool: meter
    manifest: meter-relay-performance.toml
    category: efficiency
    command: "cd projects/relay && ../../target/debug/vat run meter-perf"
    native:
      version: 1
      project: relay
      source_contract: relay-competitor-performance-meter-gate
      delegate_command: "cd projects/relay && ../../target/debug/vat run meter-perf"
```

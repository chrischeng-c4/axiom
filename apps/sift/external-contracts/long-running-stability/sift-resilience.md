<!-- HANDWRITE-BEGIN gap="sift-resilience-external-contract" tracker="1607" reason="Declare bounded-ingest, drain, and recovery evidence for the stability gate." -->
---
id: sift-long-running-stability-resilience-ec
summary: Stability contract for bounded ingest, drain readiness, and durable Sift recovery.
fill_sections: [e2e-test, tool-contract]
---

# EC: Sift Long-Running Resilience

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: sift-long-running-stability-resilience
    capability_id: long-running-stability
    claim_id: ingest-query-replay-soak
    contract_id: sift.resilience.v1
    category: stability
    command: "cargo test -p sift --test stability_e2e -- --nocapture"
    assertions:
      - "A bounded burst of 128 valid events is durably acknowledged without an unbounded in-memory queue."
      - "Drain changes readiness to unavailable and the CRC-framed journal reopens with every acknowledged event."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: sift-rig-resilience
    tool: rig
    manifest: rig.toml
    category: stability
    command: "cd apps/sift && ../../target/debug/vat run ec-stability"
    native:
      version: 1
      project: sift
      source_contract: sift-long-running-stability-resilience
      scenarios_dir: apps/sift/e2e/rig/cases/resilience
  - id: sift-meter-stability
    tool: meter
    manifest: meter-stability.toml
    category: stability
    command: "target/debug/meter test -- -p sift --test stability_e2e -- --nocapture"
    native:
      version: 1
      project: sift
      source_contract: sift-long-running-stability-resilience
      delegate_command: "cargo test -p sift --test stability_e2e -- --nocapture"
```
<!-- HANDWRITE-END -->

---
id: lumen-long-running-stability-resilience-survival-ec
summary: Long-running stability contract for resilience and recovery evidence.
fill_sections: [e2e-test, tool-contract]
---

# EC: Stability Resilience Survival

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-long-running-stability-resilience-survival
    capability_id: long-running-stability
    claim_id: search-p99-survives-fault-and-recovers
    contract_id: search-p99-survives-fault-and-recovers
    category: stability
    command: "cd apps/lumen && ../../target/debug/vat run rig-resilience"
    assertions:
      - "packet_loss_p99 applies downstream toxiproxy timeout toxicity 0.05, requires 0 < loss_fail <= 30 and loss_p99 <= 2 * baseline_p99 + 20ms, then removes the toxic, records loss_recovered_recovered_secs <= 10, requires loss_recovery_fail == 0, and requires loss_recovery_p99 <= 2 * baseline_p99 + 1ms."
      - "partition_recovery requires partition_fail > 0 under a full downstream partition, records recovered_recovered_secs <= 10 after toxic removal, and requires recovery_p99 <= 2 * baseline_p99 + 1ms."
```
## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-rig-resilience
    tool: rig
    manifest: rig.toml
    category: stability
    command: "cd apps/lumen && ../../target/debug/vat run rig-resilience"
    native:
      version: 1
      project: lumen
      source_contract: lumen-long-running-stability-resilience-survival
      scenarios_dir: apps/lumen/tests/rig/cases/resilience
```

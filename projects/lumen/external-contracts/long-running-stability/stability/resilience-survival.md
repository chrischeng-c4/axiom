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
    command: "cargo test -p lumen --test drop_drain_e2e --test reindex_stream_e2e -- --nocapture"
    assertions:
      - "Search p99 stays within 2x baseline under 5% packet loss (toxiproxy timeout toxic; rig resilience scenario)."
      - "Search survives a full network partition and recovers within budget; post-recovery p99 stays within 2x baseline."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-rig-resilience
    tool: rig
    manifest: rig.toml
    category: stability
    command: "cd projects/lumen && ../../target/debug/vat run rig-resilience"
    native:
      version: 1
      project: lumen
      source_contract: lumen-long-running-stability-resilience-survival
      scenarios_dir: projects/lumen/tests/rig/cases/resilience
```

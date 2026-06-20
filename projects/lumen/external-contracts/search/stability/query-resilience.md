---
id: lumen-search-stability-ec
summary: Search stability — query p99 survives packet loss / partition; no RSS leak; (tracked gaps) overload backpressure, FD/thread leak, soak latency drift.
fill_sections: [e2e-test, tool-contract]
---

# EC: Search Stability (filtering · ranking · pagination)

Search must stay responsive under network fault and sustained load. rig drives
the fault scenarios (toxiproxy) and asserts the p99 SLAs; meter supplies the
peak-RSS ceiling (no leak). Three cells are tracked gaps (scenarios being added).
Because search is a Service capability, stability is production-required, so these
gaps block production until they pass.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-search-stability-resilience
    capability_id: search
    claim_id: search-p99-survives-fault-and-recovers
    contract_id: search-stability-fault-resilience
    category: stability
    test_path: projects/lumen/tests/stability_lumen_search_stability_resilience.rs
    command: "rig run --dir projects/lumen/tests/rig/cases/resilience"
    assertions:
      - "FILTERING/RANKING: under 5% packet loss (toxiproxy timeout toxic) search p99 stays <= 2x baseline_p99 + 20ms."
      - "ALL: after a full network partition, search recovers within 10s and post-recovery p99 stays <= 2x baseline_p99 + 1ms."
      - "RSS plateau: over the bounded-keyspace soak, window-2 RSS <= 1.10x window-1 RSS (no leak)."
  - id: lumen-search-stability-overload-backpressure
    capability_id: search
    claim_id: graceful-degradation-under-overload
    contract_id: search-stability-backpressure
    category: stability
    test_path: projects/lumen/tests/stability_lumen_search_stability_backpressure.rs
    command: "rig run --dir projects/lumen/tests/rig/cases/load --pins projects/lumen/tests/rig/config/pins"
    assertions:
      - "(d) Under 3x steady-state concurrent load the server stays up and bounded: error_rate <= 0.05 and p99 <= 250ms (rig load/backpressure_overload.toml + pins); no OOM/crash. Env-dependent (vat-provisioned lumen)."
  - id: lumen-search-stability-resource-leak
    capability_id: search
    claim_id: no-fd-socket-thread-leak
    contract_id: search-stability-resource-leak
    category: stability
    test_path: projects/lumen/tests/stability_lumen_search_stability_resource_leak.rs
    command: "rig run --dir projects/lumen/tests/rig/cases/endurance"
    assertions:
      - "(e) Open FD count after sustained index+search load <= 1.20x before + 16 (rig endurance/fd_leak.toml). Env-dependent (vat-provisioned lumen)."
  - id: lumen-search-stability-latency-drift
    capability_id: search
    claim_id: no-latency-drift-over-soak
    contract_id: search-stability-latency-drift
    category: stability
    test_path: projects/lumen/tests/stability_lumen_search_stability_latency_drift.rs
    command: "rig run --dir projects/lumen/tests/rig/cases/endurance"
    assertions:
      - "(f) search p99 per window over the soak drifts <= 1.10x + 6ms (rig endurance/soak_p99_drift.toml). Env-dependent (vat-provisioned lumen)."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-rig-search-stability
    tool: rig
    manifest: rig-search.toml
    category: stability
    command: "rig run --dir projects/lumen/tests/rig/cases/resilience"
    native:
      version: 1
      project: lumen
      source_contract: lumen-search-stability-resilience
      scenarios_dir: projects/lumen/tests/rig/cases/resilience
  - id: lumen-meter-search-stability
    tool: meter
    manifest: meter-search-stability.toml
    category: stability
    command: "target/debug/meter test -- -p lumen --test disk_scale_proof -- --ignored"
    native:
      version: 1
      project: lumen
      source_contract: lumen-search-stability-resilience
      delegate_command: "target/debug/meter test -- -p lumen --test disk_scale_proof -- --ignored"
```

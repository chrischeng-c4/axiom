---
id: lumen-long-running-stability-query-ec
summary: Long-running query stability — query p99 survives packet loss / partition; no RSS leak; overload backpressure, FD/thread leak, and soak latency drift remain explicit gates.
fill_sections: [e2e-test, tool-contract]
---

# EC: Long-Running Query Stability

Search must stay responsive under network fault and sustained load. Rig drives
the live toxiproxy fault, native overload, and bounded-keyspace endurance
scenarios and evaluates their captured latency, recovery, request-failure,
RSS, FD, socket, and thread measurements. The meter tool contract below remains
independent disk-scale capacity evidence; it is not the leak oracle. Because
Lumen is a long-running service, every stability case is production-required.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-long-running-stability-query-resilience
    capability_id: long-running-stability
    claim_id: search-p99-survives-fault-and-recovers
    contract_id: search-stability-fault-resilience
    category: stability
    test_path: apps/lumen/tests/stability_lumen_long_running_stability_query_resilience.rs
    command: "cd apps/lumen && ../../target/debug/vat run rig-resilience"
    assertions:
      - "packet_loss_p99 routes baseline and fault search samples through toxiproxy with downstream timeout toxicity 0.05, requires 0 < loss_fail <= 30 and loss_p99 <= 2 * baseline_p99 + 20ms, then removes the toxic, records loss_recovered_recovered_secs <= 10, requires loss_recovery_fail == 0, and requires loss_recovery_p99 <= 2 * baseline_p99 + 1ms."
      - "partition_recovery applies a full downstream partition, requires partition_fail > 0, records recovered_recovered_secs <= 10, and requires recovery_p99 <= 2 * baseline_p99 + 1ms after the toxic is removed."
  - id: lumen-long-running-stability-overload-backpressure
    capability_id: long-running-stability
    claim_id: graceful-degradation-under-overload
    contract_id: search-stability-backpressure
    category: stability
    test_path: apps/lumen/tests/stability_lumen_long_running_stability_overload_backpressure.rs
    command: "cd apps/lumen && ../../target/debug/vat run rig-load"
    assertions:
      - "backpressure_overload drives the live service at 600 offered QPS with 16 workers for 30s after warmup (3x the 200-QPS steady baseline), measures error_rate/p99_ms/achieved_qps, enforces error_rate <= 0.05 and p99_ms <= 250ms, and uses Rig's >=95% load-honesty gate so a crash, OOM, or collapsed request schedule cannot pass."
  - id: lumen-long-running-stability-resource-leak
    capability_id: long-running-stability
    claim_id: no-fd-socket-thread-leak
    contract_id: search-stability-resource-leak
    category: stability
    test_path: apps/lumen/tests/stability_lumen_long_running_stability_resource_leak.rs
    command: "cd apps/lumen && ../../target/debug/vat run rig-endurance"
    assertions:
      - "fd_leak resolves the unique Lumen listener PID, runs sustained bounded-keyspace index and search work, records independent fd/socket/thread counts before and after, requires zero request failures, and gates every after count at <= 1.20 * before + 16."
      - "soak_rss_plateau warms a bounded keyspace, runs two mixed workload windows with zero request failures, measures rss_w1/rss_w2 from the live Lumen PID, and requires rss_w2 <= 1.10 * rss_w1."
  - id: lumen-long-running-stability-latency-drift
    capability_id: long-running-stability
    claim_id: no-latency-drift-over-soak
    contract_id: search-stability-latency-drift
    category: stability
    test_path: apps/lumen/tests/stability_lumen_long_running_stability_latency_drift.rs
    command: "cd apps/lumen && ../../target/debug/vat run rig-endurance"
    assertions:
      - "soak_p99_drift warms a bounded keyspace, records p99_w1/p99_w2/p99_w3 over three 500-search windows with zero failures, and requires each adjacent window plus window 3 versus window 1 to stay <= 1.10x + 6ms."
```
## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-rig-search-stability
    tool: rig
    manifest: rig-search.toml
    category: stability
    command: "cd apps/lumen && ../../target/debug/vat run rig-resilience"
    native:
      version: 1
      project: lumen
      source_contract: lumen-long-running-stability-query-resilience
      scenarios_dir: apps/lumen/tests/rig/cases/resilience
  - id: lumen-meter-search-stability
    tool: meter
    manifest: meter-search-stability.toml
    category: stability
    command: "target/debug/meter test -- -p lumen --test disk_scale_proof -- --ignored"
    native:
      version: 1
      project: lumen
      source_contract: lumen-long-running-stability-query-resilience
      delegate_command: "target/debug/meter test -- -p lumen --test disk_scale_proof -- --ignored"
```

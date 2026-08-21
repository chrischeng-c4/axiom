---
id: mamba-t1-to-thread-gather-efficiency
summary: External contract for Tier 1 to_thread gather preserves multicore CPU and RSS bounds.
fill_sections: [e2e-test]
---

# EC: Tier 1 to_thread gather preserves multicore CPU and RSS bounds

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: mamba-t1-to-thread-gather-efficiency
    capability_id: mamba-core-semantics
    claim_id: parallel-to-thread-gather-preserves-every-result
    contract_id: MAMBA-T1-FT-GATHER-EFFICIENCY
    category: efficiency
    test_path: apps/mamba/tests/external_contracts/ec_mamba_t1_to_thread_gather_efficiency.rs
    command: "cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_efficiency --exact"
    assertions:
      - "On a host exposing at least four logical CPUs, four equal CPU-bound asyncio.to_thread jobs complete with wall-clock speedup at least 1.50x versus the same jobs run serially, while returning the exact same ordered results."
      - "During the parallel phase, measured process CPU time divided by wall time is at least 1.50, proving concurrent work used more than one core rather than cooperative single-loop scheduling."
      - "Parallel peak RSS is no greater than 1.25 times serial peak RSS plus 16 MiB for the same workload; CPU scaling cannot be purchased with unbounded retained worker/future state."
      - "The gate records logical CPU count, serial and parallel wall time, process CPU time, peak RSS, result digest, and speedup; unsupported hosts are explicit evidence, never a silent pass."
```

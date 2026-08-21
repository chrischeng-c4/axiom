---
id: mamba-t1-to-thread-gather-stability
summary: External contract for Tier 1 to_thread gather remains race deadlock and leak free.
fill_sections: [e2e-test]
---

# EC: Tier 1 to_thread gather remains race deadlock and leak free

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: mamba-t1-to-thread-gather-stability
    capability_id: mamba-core-semantics
    claim_id: parallel-to-thread-gather-preserves-every-result
    contract_id: MAMBA-T1-FT-GATHER-STABILITY
    category: stability
    test_path: apps/mamba/tests/external_contracts/ec_mamba_t1_to_thread_gather_stability.rs
    command: "cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_stability --exact"
    assertions:
      - "Across 100 rounds of eight concurrently gathered CPU-bound asyncio.to_thread calls, every round returns all eight distinct expected values exactly once with zero crash, panic, timeout, or deadlock."
      - "The stability gate varies worker completion order and must fail on any None, missing, duplicate, stale, cross-worker, or wrong result; aggregate pass counts are insufficient evidence."
      - "Using an OS-visible process-thread count, the worker/thread count after a 250 ms quiescence period following the final round returns to the pre-soak baseline plus at most one runtime service thread; private runtime registries are not an EC oracle."
      - "Peak RSS is sampled in two equal soak windows; window two must be no greater than 1.10 times window one plus 8 MiB, so monotonic retained-result leaks fail the required gate."
```

---
id: mamba-t1-to-thread-gather-results
summary: External contract for Tier 1 to_thread gather preserves every result.
fill_sections: [e2e-test]
---

# EC: Tier 1 to_thread gather preserves every result

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: mamba-t1-to-thread-gather-results
    capability_id: mamba-core-semantics
    claim_id: parallel-to-thread-gather-preserves-every-result
    contract_id: MAMBA-T1-FT-GATHER-RESULTS
    category: behavior
    test_path: apps/mamba/tests/external_contracts/ec_mamba_t1_to_thread_gather_results.rs
    command: "cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_results --exact"
    assertions:
      - "Two or more concurrently completing asyncio.to_thread calls with distinct inputs are gathered into one result list containing every expected value exactly once and in asyncio.gather input order."
      - "No gathered slot may be None, missing, duplicated, stale, or borrowed from another worker, regardless of whether that worker finishes before or after the gather await begins."
      - "The contract exercises the public Mamba asyncio surface from a compiled Python program; a Rust-only registry or helper test cannot satisfy this behavior gate by itself."
      - "The CPython control program must produce the same ordered values; Mamba's intentional divergence is multicore execution, not gather result semantics."
```

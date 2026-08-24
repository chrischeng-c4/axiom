---
id: build-globals-dict-key-leak-free
summary: External contract for Tier 1 build_globals_dict leaks no key references.
fill_sections: [e2e-test]
---

# EC: Tier 1 build_globals_dict leaks no key references

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: build-globals-dict-key-leak-free
    capability_id: mamba-core-semantics
    claim_id: build-globals-dict-leaks-no-key-references
    contract_id: MAMBA-T1-BUILD-GLOBALS-DICT-KEY-LEAK-FREE
    category: stability
    test_path: apps/mamba/tests/external_contracts/ec_build_globals_dict_key_leak_free.rs
    command: "cargo test -p mamba --release --test mamba_core_semantics_ec -- build_globals_dict_key_leak_free --exact"
    assertions:
      - "Comparing two mamba runs that call globals() 100 times versus 50,000 times (42 exposed names per call: 20 id_ns values, 20 func_info functions, plus the __name__ and total module globals), peak process RSS for the 50,000-call run is no greater than the 100-call run's peak RSS plus 24 MiB; unreleased per-call key allocations would instead grow roughly linearly with call count."
      - "Equal id_ns and func_info name counts (20 each) ensure a leak isolated to either loop alone still produces per-call growth far in excess of the 24 MiB slack, so the gate cannot pass by exercising only one of the two loops build_globals_dict populates."
      - "Each run's script accumulates and prints the sum of len(globals()) across every call; the gate asserts this sum exactly equals iterations times the expected 42-key count, so a degenerate or empty build_globals_dict cannot pass vacuously by having nothing left to leak."
      - "Peak RSS is sampled via the OS process resource-usage counter (wait4/getrusage-equivalent) on the real compiled mamba binary, not a runtime-internal self-reported counter."
```

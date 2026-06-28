---
id: aw-ec-zero-test-false-green
summary: Reject cargo-test EC commands that pass after running zero tests.
capability_refs:
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-false-green-guard
    claim: ec-false-green-guard
    coverage: full
    rationale: "EC verification must fail when a cargo test filter proves no tests, and EC inventory should keep precise cargo target selectors when a unit target is known."
---

# AW EC Zero-Test False Green

`aw ec verify` and generated Rust EC wrappers must not treat a successful
`cargo test` process as proof when cargo reports that it ran zero tests. EC case
commands generated or retained for Rust unit-test filters should carry explicit
target selectors such as `--lib` when that target is known, instead of relying
on crate-wide filters.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: aw-ec-zero-test-false-green
    capability_id: project-local-td-and-ec-gates
    claim_id: ec-false-green-guard
    contract_id: aw-ec-zero-test-false-green
    category: behavior
    command: "cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture"
    assertions:
      - "aw ec verify marks a cargo test command failed when the command exits 0 after running zero tests"
      - "generated Rust EC wrappers capture stdout and reject the same zero-test false green"
      - "ec gen keeps precise cargo test target selectors instead of relying on crate-wide filters when the source contract provides one"
```

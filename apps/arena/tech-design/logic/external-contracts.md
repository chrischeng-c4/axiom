---
id: arena-external-contracts
summary: External contract gates for arena README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: n-target-comparison-runner
    role: primary
    gap: sequential-target-fanout-and-measurement
    claim: sequential-target-fanout-and-measurement
    coverage: full
    rationale: "The EC gate verifies arena's target fanout and measurement pipeline."
  - id: n-target-comparison-runner
    role: primary
    gap: arena-report-envelope
    claim: arena-report-envelope
    coverage: full
    rationale: "The EC gate verifies arena.report/1 remains the agent-readable report envelope."
  - id: ratio-ratchet-gates
    role: primary
    gap: win-exempt-target-classification
    claim: win-exempt-target-classification
    coverage: full
    rationale: "The EC gate verifies arena's cell classification semantics."
  - id: ratio-ratchet-gates
    role: primary
    gap: host-scoped-baseline-ratchet
    claim: host-scoped-baseline-ratchet
    coverage: full
    rationale: "The EC gate verifies arena's host-scoped baseline ratchet semantics."
  - id: vat-runner-integration
    role: primary
    gap: vat-managed-comparison-runner
    claim: vat-managed-comparison-runner
    coverage: full
    rationale: "The EC gate verifies arena remains usable as a vat runner without owning environment setup."
---

# External Contracts: arena

## Sequential Target Fanout And Measurement EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: arena-sequential-target-fanout-measurement
    capability_id: n-target-comparison-runner
    claim_id: sequential-target-fanout-and-measurement
    contract_id: sequential-target-fanout-and-measurement
    category: benchmark
    command: "cargo test -p arena"
    assertions:
      - "arena pipeline tests pass"
      - "target fanout and measurement remain covered by the local suite"
```

## Arena Report Envelope EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: arena-report-envelope
    capability_id: n-target-comparison-runner
    claim_id: arena-report-envelope
    contract_id: arena-report-envelope
    category: behavior
    command: "cargo test -p arena"
    assertions:
      - "arena report tests pass"
      - "arena.report/1 remains the single JSON report contract"
```

## Win Exempt Target Classification EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: arena-win-exempt-target-classification
    capability_id: ratio-ratchet-gates
    claim_id: win-exempt-target-classification
    contract_id: win-exempt-target-classification
    category: benchmark
    command: "cargo test -p arena"
    assertions:
      - "win, exempt, and target classification tests pass"
      - "floor-dominated cells remain report-only when classified exempt"
```

## Host Scoped Baseline Ratchet EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: arena-host-scoped-baseline-ratchet
    capability_id: ratio-ratchet-gates
    claim_id: host-scoped-baseline-ratchet
    contract_id: host-scoped-baseline-ratchet
    category: benchmark
    command: "cargo test -p arena"
    assertions:
      - "baseline ratchet tests pass"
      - "host-scoped baselines remain part of arena's gate semantics"
```

## Vat Managed Comparison Runner EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: arena-vat-managed-comparison-runner
    capability_id: vat-runner-integration
    claim_id: vat-managed-comparison-runner
    contract_id: vat-managed-comparison-runner
    category: stability
    command: "cargo test -p arena"
    assertions:
      - "arena remains runnable as a vat runner"
      - "arena stays protocol-agnostic and leaves environment setup to vat"
```

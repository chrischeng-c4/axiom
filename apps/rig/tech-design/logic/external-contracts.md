---
id: rig-external-contracts
summary: External contract gates for rig README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: scenario-engine
    role: primary
    gap: record-contract-check-and-json-report
    claim: record-contract-check-and-json-report
    coverage: full
    rationale: "The EC gate verifies rig's record contract and single JSON report."
  - id: scenario-engine
    role: primary
    gap: scenario-step-dsl-execution
    claim: scenario-step-dsl-execution
    coverage: full
    rationale: "The EC gate verifies rig's scenario step DSL execution path."
  - id: load-pins
    role: primary
    gap: open-loop-load-generator
    claim: open-loop-load-generator
    coverage: full
    rationale: "The EC gate verifies rig's open-loop load generator surface."
  - id: load-pins
    role: primary
    gap: floor-and-ratchet-pin-gates
    claim: floor-and-ratchet-pin-gates
    coverage: full
    rationale: "The EC gate verifies rig's pin and baseline gate semantics."
  - id: vat-wrapped-runs
    role: primary
    gap: vat-delegated-scenario-execution
    claim: vat-delegated-scenario-execution
    coverage: full
    rationale: "The EC gate verifies rig's vat delegation boundary."
---

# External Contracts: rig

## Record Contract And JSON Report EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: rig-record-contract-json-report
    capability_id: scenario-engine
    claim_id: record-contract-check-and-json-report
    contract_id: record-contract-check-and-json-report
    category: behavior
    command: "cargo test -p rig"
    assertions:
      - "rig record-contract and report tests pass"
      - "rig.report/1 remains the single agent-readable output contract"
```

## Scenario Step DSL Execution EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: rig-scenario-step-dsl-execution
    capability_id: scenario-engine
    claim_id: scenario-step-dsl-execution
    contract_id: scenario-step-dsl-execution
    category: behavior
    command: "cargo test -p rig"
    assertions:
      - "scenario engine tests pass"
      - "step DSL execution remains covered by rig's unit and e2e tests"
```

## Open Loop Load Generator EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: rig-open-loop-load-generator
    capability_id: load-pins
    claim_id: open-loop-load-generator
    contract_id: open-loop-load-generator
    category: performance
    command: "cargo test -p rig"
    assertions:
      - "load generator tests pass"
      - "open-loop load remains part of rig's public contract"
```

## Floor And Ratchet Pin Gates EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: rig-floor-ratchet-pin-gates
    capability_id: load-pins
    claim_id: floor-and-ratchet-pin-gates
    contract_id: floor-and-ratchet-pin-gates
    category: performance
    command: "cargo test -p rig"
    assertions:
      - "pin gate tests pass"
      - "floor and ratchet baseline semantics remain covered"
```

## Vat Delegated Scenario Execution EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: rig-vat-delegated-scenario-execution
    capability_id: vat-wrapped-runs
    claim_id: vat-delegated-scenario-execution
    contract_id: vat-delegated-scenario-execution
    category: stability
    command: "cargo test -p rig"
    assertions:
      - "vat delegation tests pass"
      - "rig keeps environment setup delegated to vat"
```

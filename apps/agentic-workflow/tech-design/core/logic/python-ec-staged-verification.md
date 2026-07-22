---
id: aw-python-ec-staged-verification
summary: "Run direct Python EC dimensions in core and post-generation operational stages with external evidence."
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: project-local-td-and-ec-gates
    role: primary
    gap: python-ec-staged-verification
    claim: python-ec-staged-verification
    coverage: full
    rationale: "Python EC contracts must separate TD admission from post-generation operational verification without self-signing green."
---

# Python EC Staged Verification

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-python-ec-staged-verification
entry: verify
nodes:
  verify: { kind: start, label: "verify Python EC stage" }
  inventory: { kind: process, label: "validate promise oracle target threshold policy and external evidence declarations" }
  core: { kind: process, label: "run behavior and security only" }
  operational: { kind: process, label: "run stability and efficiency only after generation/build" }
  evidence: { kind: decision, label: "external evidence exists and is non-empty after command?" }
  pass: { kind: terminal, label: "record dimension-scoped green result" }
  fail: { kind: terminal, label: "record red result with dimension and evidence failure" }
edges:
  - { from: verify, to: inventory }
  - { from: inventory, to: core, label: "stage core" }
  - { from: inventory, to: operational, label: "stage operational" }
  - { from: core, to: evidence }
  - { from: operational, to: evidence }
  - { from: evidence, to: pass, label: "yes" }
  - { from: evidence, to: fail, label: "no" }
---
flowchart TD
  verify([verify Python EC stage]) --> inventory[validate external contract metadata]
  inventory -->|core| core[behavior and security only]
  inventory -->|operational| operational[stability and efficiency only]
  core --> evidence{evidence exists?}
  operational --> evidence
  evidence -->|yes| pass([green])
  evidence -->|no| fail([red])
```

Each Python EC case declares a promise, independent oracle label, target,
external command, and one or more safe `evidence/*` outputs. Stability and
efficiency also declare a threshold. `aw ec verify --stage core` runs only
behavior/security and emits explicit skipped entries for operational cases;
`--stage operational` does the reverse. A command is green only when every
declared external evidence file exists and is non-empty afterwards.

Rust defaults to `efficiency_policy = "required"`; all other targets must
declare `required`, `optional`, or `not-applicable` explicitly. Legacy EC
verification remains on its existing all-case behavior.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-python-ec-staged-verification-unit-tests
requirements:
  staged_dimensions:
    id: R1
    text: "Core runs behavior/security and explicitly skips operational cases; operational runs stability/efficiency and rejects missing external evidence."
    kind: contract
    risk: high
    verify: "cargo test -p agentic-workflow ec_python_staged_dimensions -- --nocapture"
elements:
  ec_python_staged_dimensions: { kind: test, type: "rs/#[test]" }
relations:
  - { from: ec_python_staged_dimensions, verifies: staged_dimensions }
---
requirementDiagram
  requirement R1 {
    id: R1
    text: "staged dimensions and external evidence"
    risk: high
    verifymethod: test
  }
  element ec_python_staged_dimensions {
    type: "rs/#[test]"
  }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/services/python_ec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Validate direct-case promise, oracle, target, threshold, external evidence, and target efficiency policy."
  - path: apps/agentic-workflow/src/cli/ec.rs
    action: modify
    section: logic
    impl_mode: codegen
    description: "Run core or operational Python EC subsets and fail successful commands whose declared external evidence is absent."
  - path: apps/agentic-workflow/tests/ec_python_review_lock.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Exercise core/operational filtering and evidence failure through the real CLI."
```

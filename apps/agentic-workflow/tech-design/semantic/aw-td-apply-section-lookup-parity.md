---
id: aw-td-apply-section-lookup-parity
summary: Normalize body-only generic TD payloads into one requested typed section wrapper so mutating apply preserves the same section lookup contract as a valid on-disk TD.
fill_sections: [logic, unit-test, e2e-test]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-apply-section-lookup-parity
    claim: td-apply-section-lookup-parity
    coverage: full
    rationale: "The linear TD lifecycle must preserve typed section identity while applying initialized payloads and reject malformed candidates before mutation."
---

# AW TD apply section lookup parity

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-td-apply-section-lookup-parity
entry: payload
nodes:
  payload: { kind: start, label: "Read initialized section payload JSON" }
  structured: { kind: decision, label: "Structured section payload?" }
  render: { kind: process, label: "Render complete Unit Test wrapper" }
  scan: { kind: process, label: "Fence-aware scan of generic payload body" }
  balanced: { kind: decision, label: "Fences balanced and body non-placeholder?" }
  shape: { kind: decision, label: "Body-only or one complete typed wrapper?" }
  wrap: { kind: process, label: "Add requested canonical H2 and annotation" }
  preserve: { kind: process, label: "Preserve compatible custom heading wrapper bytes" }
  reject: { kind: terminal, label: "Actionable error; spec bytes unchanged" }
  merge: { kind: process, label: "Replace exactly the requested section" }
  validate: { kind: decision, label: "RequireThrough(candidate) passes?" }
  write: { kind: process, label: "Write candidate and advance section queue" }
  next: { kind: terminal, label: "Dispatch next initialized payload" }
edges:
  - { from: payload, to: structured }
  - { from: structured, to: render, label: "yes" }
  - { from: structured, to: scan, label: "no" }
  - { from: scan, to: balanced }
  - { from: balanced, to: reject, label: "no" }
  - { from: balanced, to: shape, label: "yes" }
  - { from: shape, to: wrap, label: "body-only" }
  - { from: shape, to: preserve, label: "one matching wrapper" }
  - { from: shape, to: reject, label: "missing, mismatched, malformed, or multiple" }
  - { from: render, to: merge }
  - { from: wrap, to: merge }
  - { from: preserve, to: merge }
  - { from: merge, to: validate }
  - { from: validate, to: reject, label: "no" }
  - { from: validate, to: write, label: "yes" }
  - { from: write, to: next }
---
flowchart TD
  payload([initialized payload JSON]) --> structured{structured section?}
  structured -->|yes| render[render complete Unit Test wrapper]
  structured -->|no| scan[fence-aware generic body scan]
  scan --> balanced{balanced and authored?}
  balanced -->|no| reject([error; no spec write])
  balanced -->|yes| shape{payload shape?}
  shape -->|body-only| wrap[add requested typed wrapper]
  shape -->|one matching wrapper| preserve[preserve wrapper and custom heading]
  shape -->|invalid or multiple| reject
  render --> merge[replace only requested section]
  wrap --> merge
  preserve --> merge
  merge --> validate{RequireThrough candidate passes?}
  validate -->|no| reject
  validate -->|yes| write[write and advance queue]
  write --> next([next initialized payload])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-td-apply-section-lookup-parity-verification
requirements:
  body_only_normalization:
    id: R1
    text: "A body-only generic Logic payload gains the requested complete H2 and annotation wrapper before merge."
    kind: regression
    risk: high
    verify: cargo test -p agentic-workflow --lib normalize_generic_td_section_payload -- --nocapture
  wrapped_compatibility:
    id: R2
    text: "A complete matching wrapper, including a custom heading, remains byte and semantically compatible."
    kind: compatibility
    risk: high
    verify: cargo test -p agentic-workflow --lib normalize_generic_td_section_payload -- --nocapture
  malformed_non_mutation:
    id: R3
    text: "Empty, wrong-language, unclosed-fence, mismatched, broken, or multiple-wrapper payloads fail before the spec is written."
    kind: negative
    risk: high
    verify: cargo test -p agentic-workflow --test cli_tests td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test -- --nocapture
  sequential_apply:
    id: R4
    text: "A valid existing TD advances initialized applicability Logic and then structured Unit Test payloads without losing either typed wrapper."
    kind: regression
    risk: high
    verify: cargo test -p agentic-workflow --test cli_tests td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test -- --nocapture
elements:
  normalize_generic_td_section_payload:
    kind: function
    type: "rs/fn"
  td_payload_top_level_shape:
    kind: function
    type: "rs/fn"
  normalize_generic_td_section_payload_tests:
    kind: test
    type: "rs/#[test]"
  td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: normalize_generic_td_section_payload, verifies: body_only_normalization }
  - { from: normalize_generic_td_section_payload_tests, verifies: wrapped_compatibility }
  - { from: td_payload_top_level_shape, verifies: malformed_non_mutation }
  - { from: td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test, verifies: sequential_apply }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "body-only payload gains typed wrapper"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "complete custom wrapper stays compatible"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "malformed payload cannot mutate spec"
      risk: high
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "Logic then Unit Test advances"
      risk: high
      verifymethod: test
    }
    element normalize_generic_td_section_payload {
      type: "rs/fn"
    }
    element td_payload_top_level_shape {
      type: "rs/fn"
    }
    element normalize_generic_td_section_payload_tests {
      type: "rs/#[test]"
    }
    element td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test {
      type: "rs/#[test]"
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: aw-td-apply-section-lookup-parity-real-cli
    name: TD body-only section apply parity real CLI
    capability_id: td-cb-lifecycle-automation
    claim_id: td-apply-section-lookup-parity
    command: cargo test -p agentic-workflow --test cli_tests td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test -- --nocapture
    assertions:
      - "the already-valid fixture passes aw td check with zero findings"
      - "missing and malformed payload attempts leave the spec byte-identical"
      - "body-only Logic applies with exactly one typed Logic wrapper"
      - "the next initialized payload is applicability/unit-test.json"
      - "structured Unit Test applies and dispatches contract Logic"
```

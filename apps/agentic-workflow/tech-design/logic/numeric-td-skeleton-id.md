---
id: numeric-td-skeleton-id
summary: Preserve generated TD skeleton ids as YAML strings so numeric GitHub WI ids can enter the applicability section-apply loop without manual frontmatter edits.
fill_sections: [logic, unit-test]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: numeric-td-skeleton-ids
    claim: numeric-td-skeleton-ids
    coverage: full
    rationale: "The TD create producer must emit a validator-compatible skeleton for both numeric issue ids and non-numeric slugs."
---

# Numeric TD skeleton IDs

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: numeric-td-skeleton-id
entry: start
nodes:
  start: { kind: start, label: "initialize_td_spec_skeleton(path, slug)" }
  exists: { kind: decision, label: "spec already exists?" }
  preserve: { kind: terminal, label: "return false; preserve authored bytes" }
  encode: { kind: process, label: "serde_yaml::to_string(slug) as a string scalar" }
  write: { kind: process, label: "write skeleton frontmatter with encoded id" }
  apply: { kind: process, label: "merge first applicability section" }
  validate: { kind: decision, label: "frontmatter id parses with as_str()?" }
  success: { kind: terminal, label: "section apply validates" }
  reject: { kind: terminal, label: "serialization or validation error" }
edges:
  - { from: start, to: exists }
  - { from: exists, to: preserve, label: "yes" }
  - { from: exists, to: encode, label: "no" }
  - { from: encode, to: write }
  - { from: write, to: apply }
  - { from: apply, to: validate }
  - { from: validate, to: success, label: "yes" }
  - { from: validate, to: reject, label: "no" }
---
flowchart TD
  start([initialize skeleton]) --> exists{spec exists?}
  exists -->|yes| preserve([preserve authored bytes])
  exists -->|no| encode[serialize slug as YAML string]
  encode --> write[write skeleton]
  write --> apply[merge first applicability section]
  apply --> validate{id is string?}
  validate -->|yes| success([section apply validates])
  validate -->|no| reject([error])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: numeric-td-skeleton-id-tests
requirements:
  numeric_string_id:
    id: R1
    text: "A generated skeleton for numeric WI id 1487 parses its top-level id as the string 1487."
    kind: functional
    risk: high
    verify: test
  first_section_apply:
    id: R2
    text: "Merging the first applicability logic payload into the numeric-id skeleton passes section-apply validation."
    kind: functional
    risk: high
    verify: test
  preserve_existing_behavior:
    id: R3
    text: "A non-numeric slug remains a valid id and repeated initialization does not overwrite authored content."
    kind: functional
    risk: medium
    verify: test
elements:
  initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply:
    kind: test
    type: "rs/#[test]"
  initialize_td_spec_skeleton_writes_frontmatter_and_is_idempotent:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply, verifies: numeric_string_id }
  - { from: initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply, verifies: first_section_apply }
  - { from: initialize_td_spec_skeleton_writes_frontmatter_and_is_idempotent, verifies: preserve_existing_behavior }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "numeric skeleton id is a YAML string"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "first applicability section validates"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "slug and idempotence remain valid"
      risk: medium
      verifymethod: test
    }
    element initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply {
      type: "rs/#[test]"
    }
    element initialize_td_spec_skeleton_writes_frontmatter_and_is_idempotent {
      type: "rs/#[test]"
    }
```

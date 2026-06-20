---
id: promote-generated-manuals-to-first-class-aw-evidence-artifacts
summary: Promote generated product manuals to first-class AW EC evidence artifacts with typed output paths, runner commands, and optional media metadata.
fill_sections: [logic, unit-test]
capability_refs:
  - id: manual-evidence-artifacts
    role: primary
    gap: generated-manual-ec-evidence-schema
    claim: generated-manual-ec-evidence-schema
    coverage: partial
    rationale: "This TD defines the generated-manual evidence artifact contract for AW EC docs evidence."
  - id: manual-evidence-artifacts
    role: primary
    gap: manual-runner-output-convention
    claim: manual-runner-output-convention
    coverage: full
    rationale: "This TD records generated manual runner commands and renders the output convention in the EC manual surface."
---

# Promote generated manuals to first-class AW evidence artifacts

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: generated-manual-evidence-contract-flow
entry: start
nodes:
  start:
    kind: start
    label: "evidence_artifacts_from_yaml() receives E2eEvidenceYaml.docs"
  artifact_shape:
    kind: process
    label: "Extend EcEvidenceArtifact/manual metadata with format, command, screenshots, highlights, and steps"
  generated_manual:
    kind: decision
    label: "artifact.kind == generated-manual?"
  legacy_doc:
    kind: process
    label: "Keep existing doc evidence behavior for other docs entries"
  validate_manual:
    kind: process
    label: "validate_generated_manual_artifact(): require command, safe project-local path, and markdown/html format"
  valid:
    kind: decision
    label: "manual artifact valid?"
  reject:
    kind: terminal
    label: "Return EC validation finding with artifact path and failed rule"
  render:
    kind: process
    label: "render_ec_doc() prints Manual Evidence section with path, command, format, and optional media"
  report:
    kind: terminal
    label: "aw ec doc/check, aw health, and capability evidence inventory can display the generated manual artifact"
edges:
  - from: start
    to: artifact_shape
  - from: artifact_shape
    to: generated_manual
  - from: generated_manual
    to: legacy_doc
    label: "no"
  - from: generated_manual
    to: validate_manual
    label: "yes"
  - from: validate_manual
    to: valid
  - from: valid
    to: reject
    label: "no"
  - from: valid
    to: render
    label: "yes"
  - from: legacy_doc
    to: render
  - from: render
    to: report
---
flowchart TD
  start([Parse evidence.docs]) --> artifact_shape[Extend evidence artifact metadata]
  artifact_shape --> generated_manual{generated-manual?}
  generated_manual -->|no| legacy_doc[Existing docs evidence behavior]
  generated_manual -->|yes| validate_manual[Validate command, safe path, markdown/html]
  validate_manual --> valid{valid?}
  valid -->|no| reject([EC validation finding])
  valid -->|yes| render[Render Manual Evidence section]
  legacy_doc --> render
  render --> report([manual artifact visible in EC docs/check/health/capability inventory])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: generated-manual-evidence-contract-tests
requirements:
  schema_fields:
    id: CT1
    text: "EcEvidenceArtifact preserves generated-manual metadata fields from e2e evidence docs YAML: path, label, format, command, screenshots, highlights, and steps."
    kind: functional
    risk: high
    verify: test
  validation_success:
    id: CT2
    text: "A markdown generated-manual under docs/ with a non-empty runner command validates cleanly."
    kind: functional
    risk: high
    verify: test
  validation_failures:
    id: CT3
    text: "Missing command, unsupported format, and path escape each produce deterministic EC validation findings."
    kind: safety
    risk: high
    verify: test
  optional_media:
    id: CT4
    text: "Manual screenshots, highlights, and steps are optional and round-trip when present."
    kind: functional
    risk: medium
    verify: test
  documentation_surface:
    id: CT5
    text: "render_ec_doc() includes generated manual path, command, format, and optional media in the generated EC manual."
    kind: integration
    risk: medium
    verify: test
elements:
  ec_generated_manual_artifact_preserves_metadata:
    kind: test
    type: "rs/#[test]"
  ec_generated_manual_artifact_validates_markdown_docs_runner:
    kind: test
    type: "rs/#[test]"
  ec_generated_manual_artifact_reports_invalid_contracts:
    kind: test
    type: "rs/#[test]"
  ec_generated_manual_artifact_round_trips_optional_media:
    kind: test
    type: "rs/#[test]"
  ec_doc_gen_renders_generated_manual_artifact_details:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: ec_generated_manual_artifact_preserves_metadata, verifies: schema_fields }
  - { from: ec_generated_manual_artifact_validates_markdown_docs_runner, verifies: validation_success }
  - { from: ec_generated_manual_artifact_reports_invalid_contracts, verifies: validation_failures }
  - { from: ec_generated_manual_artifact_round_trips_optional_media, verifies: optional_media }
  - { from: ec_doc_gen_renders_generated_manual_artifact_details, verifies: documentation_surface }
---
requirementDiagram
  requirement CT1 {
    id: CT1
    text: "manual metadata schema"
    risk: high
    verifymethod: test
  }
  requirement CT2 {
    id: CT2
    text: "valid docs runner contract"
    risk: high
    verifymethod: test
  }
  requirement CT3 {
    id: CT3
    text: "invalid contract diagnostics"
    risk: high
    verifymethod: test
  }
  requirement CT4 {
    id: CT4
    text: "optional media round trip"
    risk: medium
    verifymethod: test
  }
  requirement CT5 {
    id: CT5
    text: "EC manual rendering"
    risk: medium
    verifymethod: test
  }
  element ec_generated_manual_artifact_preserves_metadata {
    type: "rs/#[test]"
  }
  element ec_generated_manual_artifact_validates_markdown_docs_runner {
    type: "rs/#[test]"
  }
  element ec_generated_manual_artifact_reports_invalid_contracts {
    type: "rs/#[test]"
  }
  element ec_generated_manual_artifact_round_trips_optional_media {
    type: "rs/#[test]"
  }
  element ec_doc_gen_renders_generated_manual_artifact_details {
    type: "rs/#[test]"
  }
  ec_generated_manual_artifact_preserves_metadata - verifies -> CT1
  ec_generated_manual_artifact_validates_markdown_docs_runner - verifies -> CT2
  ec_generated_manual_artifact_reports_invalid_contracts - verifies -> CT3
  ec_generated_manual_artifact_round_trips_optional_media - verifies -> CT4
  ec_doc_gen_renders_generated_manual_artifact_details - verifies -> CT5
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] contract-complete: The contract identifies the concrete EC parsing/rendering path and preserves legacy docs behavior while adding generated-manual validation for command, safe path, supported format, optional media, and report visibility.
- [unit-test] contract-complete: The tests map schema preservation, valid manual contracts, invalid diagnostics, optional media round-trip, and EC manual rendering to named Rust test targets.

---
id: sdd-requirement-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Requirement Plus Schema

## Overview
<!-- type: overview lang: markdown -->

9 types defining Mermaid Plus requirement diagrams. Sibling of
`sdd-content-requirement` and the just-merged `sdd-block-plus-schema`.

Codegen emits the serde import used by generated derives and attributes.
Hand-written outside CODEGEN: module preamble, non-serde `HashMap` import,
and tests block.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReqDirection:
    type: string
    enum: [TB, BT, LR, RL]
    description: Layout direction for requirement diagram.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]

  RequirementTypePlus:
    type: string
    enum:
      - Requirement
      - FunctionalRequirement
      - InterfaceRequirement
      - PerformanceRequirement
      - PhysicalRequirement
      - DesignConstraint
    description: Requirement type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: camelCase
      variants:
        - { name: Requirement,            is_default: true,    doc: "Generic." }
        - { name: FunctionalRequirement,                       doc: "Functional." }
        - { name: InterfaceRequirement,                        doc: "Interface." }
        - { name: PerformanceRequirement,                      doc: "Performance." }
        - { name: PhysicalRequirement,                         doc: "Physical." }
        - { name: DesignConstraint,                            doc: "Design constraint." }

  RiskLevelPlus:
    type: string
    enum: [Low, Medium, High]
    description: Risk level.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]

  VerificationMethodPlus:
    type: string
    enum: [Analysis, Inspection, Test, Demonstration]
    description: Verification method.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]

  ReqRelationshipTypePlus:
    type: string
    enum: [Satisfies, Verifies, Refines, Traces, Contains, Copies, Derives]
    description: Relationship type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]
      serde_rename_all: lowercase

  RequirementDiagramDef:
    type: object
    required: [id, requirements, elements, relationships]
    description: Requirement diagram definition.
    properties:
      id:
        type: string
      title:
        type: string
      direction:
        $ref: "#/definitions/ReqDirection"
        x-rust-type: "Option<ReqDirection>"
        x-serde-default: true
      requirements:
        type: object
        x-rust-type: "HashMap<String, RequirementDefPlus>"
      elements:
        type: object
        x-rust-type: "HashMap<String, ElementDef>"
        x-serde-default: true
      relationships:
        type: array
        items:
          $ref: "#/definitions/ReqRelationshipDef"
        x-serde-default: true
      description:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RequirementDefPlus:
    type: object
    required: [text, req_type, risk, verification]
    description: Requirement definition.
    properties:
      text:
        type: string
      req_type:
        $ref: "#/definitions/RequirementTypePlus"
        x-serde-rename: "type"
        x-serde-default: true
      risk:
        $ref: "#/definitions/RiskLevelPlus"
      verification:
        $ref: "#/definitions/VerificationMethodPlus"
      description:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ElementDef:
    type: object
    required: [text, elem_type]
    description: Design element definition.
    properties:
      text:
        type: string
      elem_type:
        type: string
        x-serde-rename: "type"
      docref:
        type: string
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      description:
        type: string
      test_type:
        type: string
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      given:
        type: string
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      when:
        type: string
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      then:
        type: string
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ReqRelationshipDef:
    type: object
    required: [from, to, rel_type]
    description: Relationship definition.
    properties:
      from:
        type: string
      to:
        type: string
      rel_type:
        $ref: "#/definitions/ReqRelationshipTypePlus"
        x-serde-rename: "type"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/requirement_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - RequirementDiagramDef
      - ReqDirection
      - RequirementDefPlus
      - RequirementTypePlus
      - RiskLevelPlus
      - VerificationMethodPlus
      - ElementDef
      - ReqRelationshipDef
      - ReqRelationshipTypePlus
    description: |
      Codegen replaces all 9 type declarations and emits the serde import
      required by their derives and attributes.
  - path: projects/agentic-workflow/src/generate/diagrams/requirement_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module preamble,
      `use std::collections::HashMap;`, and tests block.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 9 types; sibling pattern.
- [schema] All proven patterns combined.
- [changes] codegen + hand-written split correct.

---
id: sdd-generate-generators-test-generator
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Test Generator Types

## Overview
<!-- type: overview lang: markdown -->

Test scaffold generator types in
`projects/agentic-workflow/src/generate/generators/test_generator.rs`. Four shapes:

- `CoverageIssue` — `req_id`, `req_text`, `message` (all String).
  Derives `[Debug, Clone, Serialize]`.
- `TestGenResult` — `file_path`, `content`, `coverage_issues: Vec<CoverageIssue>`.
  Derives `[Debug, Clone]`.
- `TestGenError` — 2-variant thiserror enum:
  - `NoTestableRequirements` (unit) with error template.
  - `UncoveredRequirements(String)` (tuple) with error template.
- `TestGenerator` — `strict: bool` (public field). No derives.

Codegen replaces the serde import required by generated derives plus all four
type declarations. Module docstring, diagram/std imports, and the
`impl TestGenerator { new, generate, ... }` block are owned by sibling source
templates that replace the legacy HANDWRITE gaps.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CoverageIssue:
    type: object
    required: [req_id, req_text, message]
    description: A coverage issue found during test generation.
    properties:
      req_id:
        type: string
        description: "Requirement ID (e.g. R1)."
      req_text:
        type: string
        description: "Requirement text."
      message:
        type: string
        description: "Issue description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]

  TestGenResult:
    type: object
    required: [file_path, content, coverage_issues]
    description: Result of a successful test generation run.
    properties:
      file_path:
        type: string
        description: "Relative output file path."
      content:
        type: string
        description: "Generated Python source."
      coverage_issues:
        type: array
        items: { $ref: "#/definitions/CoverageIssue" }
        x-rust-type: "Vec<CoverageIssue>"
        description: "Coverage warnings."
    x-rust-struct:
      derive: [Debug, Clone]

  TestGenError:
    type: string
    enum: [NoTestableRequirements, UncoveredRequirements]
    description: Errors returned by TestGenerator.generate.
    x-rust-enum:
      derive: [Debug, "thiserror::Error"]
      variants:
        - name: NoTestableRequirements
          error: "No testable requirements found (all have verifymethod != Test)"
          doc: "All requirements have non-Test verifymethod."
        - name: UncoveredRequirements
          kind: tuple
          error: "Uncovered requirements in strict mode: {0}"
          doc: "Strict mode found uncovered requirements."
          fields:
            - { rust_type: String }

  TestGenerator:
    type: object
    required: [strict]
    description: Test scaffold generator from RequirementDiagramDef.
    properties:
      strict:
        type: boolean
        description: "When true, uncovered requirements become hard errors instead of warnings."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/test_generator.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - CoverageIssue
      - TestGenResult
      - TestGenError
      - TestGenerator
    description: |
      Codegen replaces all four type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 4 types: 2 plain structs, 1 thiserror enum, 1 struct with bool field.
- [schema] All in `required:`; thiserror error templates per variant.
- [changes] Standard split with all four in `replaces`.

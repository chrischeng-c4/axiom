---
id: sdd-services-issue-parser
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Issue Parser Types

## Overview
<!-- type: overview lang: markdown -->

Structured-issue parsing types in
`projects/agentic-workflow/src/services/issue_parser.rs`. Ten shapes:

- `StructuredIssue` — top-level parsed issue.
- `Requirement` — id/text/optional priority.
- `AcceptanceCriterion` — id/text.
- `IssueScope` — in_scope/out_of_scope (Default).
- `Decision` — id/text.
- `IssueReferenceContext` — specs/spec_plan/raw (Default).
- `SpecReference` — path/relevance/key_requirements.
- `SpecPlanEntry` — spec_id/action/main_spec_ref/sections.
- `ValidationError` — error/code/missing (Serialize).
- `IssueQualityResult` — passed/errors (Serialize).

Codegen replaces all ten type declarations and owns the serde import needed
for `ValidationError` and `IssueQualityResult`. Companion source templates own
the module documentation, non-serde imports, section-format checks, structured
issue detection, validation, parsing, slug resolution, artifact generation, and
regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Requirement:
    type: object
    required: [id, text, priority]
    description: A single requirement.
    properties:
      id:
        type: string
        description: "Requirement ID."
      text:
        type: string
        description: "Requirement text."
      priority:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional priority."
    x-rust-struct:
      derive: [Debug, Clone]

  AcceptanceCriterion:
    type: object
    required: [id, text]
    description: A single acceptance criterion.
    properties:
      id:
        type: string
        description: "Criterion ID."
      text:
        type: string
        description: "Criterion text."
    x-rust-struct:
      derive: [Debug, Clone]

  IssueScope:
    type: object
    required: [in_scope, out_of_scope]
    description: Scope extracted from the issue body.
    properties:
      in_scope:
        type: string
        description: "In-scope text."
      out_of_scope:
        type: string
        description: "Out-of-scope text."
    x-rust-struct:
      derive: [Debug, Clone, Default]

  Decision:
    type: object
    required: [id, text]
    description: A key decision.
    properties:
      id:
        type: string
        description: "Decision ID."
      text:
        type: string
        description: "Decision text."
    x-rust-struct:
      derive: [Debug, Clone]

  SpecReference:
    type: object
    required: [path, relevance, key_requirements]
    description: A spec reference from the reference context.
    properties:
      path:
        type: string
        description: "Spec path."
      relevance:
        type: string
        description: "Relevance label."
      key_requirements:
        type: string
        description: "Key requirements text."
    x-rust-struct:
      derive: [Debug, Clone]

  SpecPlanEntry:
    type: object
    required: [spec_id, action, main_spec_ref, sections]
    description: A spec plan entry.
    properties:
      spec_id:
        type: string
        description: "Spec identifier."
      action:
        type: string
        description: "Planned action."
      main_spec_ref:
        type: string
        description: "Main spec reference."
      sections:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Sections to fill."
    x-rust-struct:
      derive: [Debug, Clone]

  IssueReferenceContext:
    type: object
    required: [specs, spec_plan, raw]
    description: Reference context from the issue body.
    properties:
      specs:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecReference>"
        description: "Spec references."
      spec_plan:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecPlanEntry>"
        description: "Spec plan entries."
      raw:
        type: string
        description: "Raw text of the section."
    x-rust-struct:
      derive: [Debug, Clone, Default]

  StructuredIssue:
    type: object
    required: [problem, requirements, acceptance_criteria, scope, key_decisions, reference_context]
    description: A fully parsed structured issue.
    properties:
      problem:
        type: string
        description: "Problem statement."
      requirements:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Requirement>"
        description: "Requirements."
      acceptance_criteria:
        type: array
        items: { type: object }
        x-rust-type: "Vec<AcceptanceCriterion>"
        description: "Acceptance criteria."
      scope:
        type: object
        x-rust-type: "IssueScope"
        description: "Scope."
      key_decisions:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Decision>"
        description: "Key decisions."
      reference_context:
        type: object
        x-rust-type: "Option<IssueReferenceContext>"
        description: "Optional reference context."
    x-rust-struct:
      derive: [Debug, Clone]

  ValidationError:
    type: object
    required: [error, code, missing]
    description: A validation failure for an issue body.
    properties:
      error:
        type: string
        description: "Human-readable error message."
      code:
        type: string
        description: "Stable error code."
      missing:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Missing sections or invalid items."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]

  IssueQualityResult:
    type: object
    required: [passed, errors]
    description: Issue quality result.
    properties:
      passed:
        type: boolean
        description: "Whether the issue passed quality checks."
      errors:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Errors found during quality check."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/issue_parser.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - StructuredIssue
      - Requirement
      - AcceptanceCriterion
      - IssueScope
      - Decision
      - IssueReferenceContext
      - SpecReference
      - SpecPlanEntry
      - ValidationError
      - IssueQualityResult
    description: |
      Codegen replaces all ten type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 10 data carriers across the parsed-issue model.
- [schema] All in `required:`; foreign types via x-rust-type.
- [changes] All ten in `replaces`.

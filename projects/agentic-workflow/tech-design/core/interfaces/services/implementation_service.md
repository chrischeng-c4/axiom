---
id: sdd-services-implementation-service
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Implementation Service Types

## Overview
<!-- type: overview lang: markdown -->

Review/merge input types for the implementation service in
`projects/agentic-workflow/src/services/implementation_service.rs`. Nine shapes:

- `Severity` — High/Medium/Low.
- `ReviewVerdict` — Approved/Reviewed/Rejected.
- `MergeReviewVerdict` — Approved/Reviewed/Rejected.
- `MergeQuality` — Clean/Partial/Failed.
- `ReviewIssue` — severity/title/description/optional file_path/line_number/recommendation.
- `TestResults` — status/total/passed/failed/skipped.
- `CreateReviewInput` — change_id/iteration/test_results/security_status/issues/verdict/optional next_steps.
- `CreateMergeReviewInput` — change_id/iteration/summary/merge_quality/preservation flags/changelog flags/issues/verdict.
- `MergeReviewIssue` — severity/description.

Codegen replaces all nine type declarations. Companion source templates own the
module documentation, imports, change-id validation, git diff helpers,
requirements reader, review display impls, review artifact generation, merge
review generation, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Severity:
    type: string
    enum: [High, Medium, Low]
    description: Issue severity level.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq]

  ReviewVerdict:
    type: string
    enum: [Approved, Reviewed, Rejected]
    description: Review verdict.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq]

  MergeReviewVerdict:
    type: string
    enum: [Approved, Reviewed, Rejected]
    description: Merge review verdict.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq]

  MergeQuality:
    type: string
    enum: [Clean, Partial, Failed]
    description: Merge quality status.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq]

  ReviewIssue:
    type: object
    required: [severity, title, description, file_path, line_number, recommendation]
    description: A single review issue.
    properties:
      severity:
        type: string
        x-rust-type: "Severity"
        description: "Severity level."
      title:
        type: string
        description: "Issue title."
      description:
        type: string
        description: "Issue description."
      file_path:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional file path."
      line_number:
        type: integer
        x-rust-type: "Option<u32>"
        description: "Optional line number."
      recommendation:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional recommendation."
    x-rust-struct:
      derive: [Debug, Clone]

  TestResults:
    type: object
    required: [status, total, passed, failed, skipped]
    description: Test results summary.
    properties:
      status:
        type: string
        description: "Status: PASS, FAIL, PARTIAL, UNKNOWN."
      total:
        type: integer
        x-rust-type: "u32"
        description: "Total tests."
      passed:
        type: integer
        x-rust-type: "u32"
        description: "Passed count."
      failed:
        type: integer
        x-rust-type: "u32"
        description: "Failed count."
      skipped:
        type: integer
        x-rust-type: "u32"
        description: "Skipped count."
    x-rust-struct:
      derive: [Debug, Clone, Default]

  CreateReviewInput:
    type: object
    required: [change_id, iteration, test_results, security_status, issues, verdict, next_steps]
    description: Input for creating a review.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      iteration:
        type: integer
        x-rust-type: "u32"
        description: "Iteration number."
      test_results:
        type: object
        x-rust-type: "TestResults"
        description: "Test results."
      security_status:
        type: string
        description: "CLEAN, WARNINGS, VULNERABILITIES."
      issues:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ReviewIssue>"
        description: "Review issues."
      verdict:
        type: string
        x-rust-type: "ReviewVerdict"
        description: "Review verdict."
      next_steps:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional next steps text."
    x-rust-struct:
      derive: [Debug, Clone]

  CreateMergeReviewInput:
    type: object
    required: [change_id, iteration, summary, merge_quality, requirements_preserved, scenarios_preserved, diagrams_preserved, changelog_present, changelog_accurate, issues, verdict]
    description: Input for creating a merge review.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      iteration:
        type: integer
        x-rust-type: "u32"
        description: "Iteration number."
      summary:
        type: string
        description: "Review summary."
      merge_quality:
        type: string
        x-rust-type: "MergeQuality"
        description: "Merge quality status."
      requirements_preserved:
        type: boolean
        description: "Whether requirements were preserved."
      scenarios_preserved:
        type: boolean
        description: "Whether scenarios were preserved."
      diagrams_preserved:
        type: boolean
        description: "Whether diagrams were preserved."
      changelog_present:
        type: boolean
        description: "Whether changelog is present."
      changelog_accurate:
        type: boolean
        description: "Whether changelog is accurate."
      issues:
        type: array
        items: { type: object }
        x-rust-type: "Vec<MergeReviewIssue>"
        description: "Merge review issues."
      verdict:
        type: string
        x-rust-type: "MergeReviewVerdict"
        description: "Merge review verdict."
    x-rust-struct:
      derive: [Debug, Clone]

  MergeReviewIssue:
    type: object
    required: [severity, description]
    description: A single merge review issue.
    properties:
      severity:
        type: string
        x-rust-type: "Severity"
        description: "Severity level."
      description:
        type: string
        description: "Issue description."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/implementation_service.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - Severity
      - ReviewVerdict
      - MergeReviewVerdict
      - MergeQuality
      - ReviewIssue
      - TestResults
      - CreateReviewInput
      - CreateMergeReviewInput
      - MergeReviewIssue
    description: |
      Codegen replaces all nine type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] 9 types: 4 enums + 5 structs.
- [schema] All in `required:`; foreign types via x-rust-type.
- [changes] Standard split; Display impls preserved.

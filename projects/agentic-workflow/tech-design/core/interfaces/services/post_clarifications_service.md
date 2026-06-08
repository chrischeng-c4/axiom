---
id: sdd-services-post-clarifications
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Post Clarifications Service Types

## Overview
<!-- type: overview lang: markdown -->

Plain input/output types for the SDD post-clarifications service in
`projects/agentic-workflow/src/services/post_clarifications_service.rs`. Four structs
with **no derives at all**, consumed by
`pub fn create_post_clarifications(...)`.

- `PostQuestion` — one question/answer/rationale row (topic, question,
  answer, rationale; all `String`).
- `Contradiction` — one contradiction row (source_artifact,
  original_claim, contradicting_finding, resolution; all `String`).
- `CreatePostClarificationsInput` — change_id (`String`),
  questions (`Vec<PostQuestion>`), contradictions (`Vec<Contradiction>`).
- `PostClarificationsResult` — artifacts_written (`Vec<String>`),
  questions_count (`usize`), contradictions_count (`usize`).

Mirrors the just-merged `sdd-services-init-change` pattern. Vec fields
are listed in `required:` to skip Option auto-wrap; `usize` integer
fields use `x-rust-type: usize` to override the default `u64` mapping.

Codegen replaces the data structs. Companion source templates own the module
documentation, imports, post-clarification artifact formatting, filesystem
writes, result assembly, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  PostQuestion:
    type: object
    required: [topic, question, answer, rationale]
    description: A question-answer pair captured during post-gap clarification.
    properties:
      topic:
        type: string
        description: "Topic / area the question addresses."
      question:
        type: string
        description: "Question text."
      answer:
        type: string
        description: "Answer text."
      rationale:
        type: string
        description: "Reasoning behind the answer."
    x-rust-struct:
      derive: []

  Contradiction:
    type: object
    required: [source_artifact, original_claim, contradicting_finding, resolution]
    description: A contradiction surfaced while cross-referencing artifacts.
    properties:
      source_artifact:
        type: string
        description: "Path / identifier of the source artifact."
      original_claim:
        type: string
        description: "Original claim from the source artifact."
      contradicting_finding:
        type: string
        description: "Finding that contradicts the original claim."
      resolution:
        type: string
        description: "Resolution chosen to reconcile the contradiction."
    x-rust-struct:
      derive: []

  CreatePostClarificationsInput:
    type: object
    required: [change_id, questions, contradictions]
    description: Input for creating post-clarifications artifact.
    properties:
      change_id:
        type: string
        description: "Change identifier slug."
      questions:
        type: array
        items:
          $ref: "#/definitions/PostQuestion"
        description: "Question-answer rows to include."
      contradictions:
        type: array
        items:
          $ref: "#/definitions/Contradiction"
        description: "Contradiction rows to include."
    x-rust-struct:
      derive: []

  PostClarificationsResult:
    type: object
    required: [artifacts_written, questions_count, contradictions_count]
    description: Result of creating the post-clarifications artifact.
    properties:
      artifacts_written:
        type: array
        items: { type: string }
        description: "Relative paths of artifacts written."
      questions_count:
        type: integer
        x-rust-type: usize
        description: "Number of question rows written."
      contradictions_count:
        type: integer
        x-rust-type: usize
        description: "Number of contradiction rows written."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/post_clarifications_service.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - PostQuestion
      - Contradiction
      - CreatePostClarificationsInput
      - PostClarificationsResult
    description: |
      Codegen replaces all 4 struct declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] `usize` fields explicitly use `x-rust-type: usize`, overriding the default `u64` mapping documented in `schema-rs-gaps.md`.

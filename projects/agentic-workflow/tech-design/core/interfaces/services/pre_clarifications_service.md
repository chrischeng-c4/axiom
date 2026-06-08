---
id: sdd-services-pre-clarifications
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Pre Clarifications Service Types

## Overview
<!-- type: overview lang: markdown -->

Plain input types for the SDD pre-clarifications service in
`projects/agentic-workflow/src/services/pre_clarifications_service.rs`. Three serde
structs derived `[Debug, Serialize, Deserialize]` (no Clone in the
source).

- `QuestionAnswer` — one question/answer/rationale row (topic,
  question, answer, rationale; all `String`).
- `CreateClarificationsInput` — change_id (`String`),
  questions (`Vec<QuestionAnswer>`).
- `AppendClarificationsInput` — change_id (`String`),
  issue (`Option<u64>` — explicitly typed via `x-rust-type`),
  questions (`Vec<QuestionAnswer>`).

This spec exercises:

1. **`Option<u64>` via `x-rust-type`** on `AppendClarificationsInput.issue`
   — listed in `required:` to skip Option auto-wrap; `x-rust-type:
   "Option<u64>"` makes the literal type appear verbatim. Same
   integer-Option pattern as `sdd-generate-sequence` (which uses
   `Option<usize>`).
2. **Vec-in-required convention** — both `Vec<QuestionAnswer>` fields
   are in `required:` so they emit bare `Vec<...>` (not
   `Option<Vec<...>>`).
3. **Partial-derive struct** — `[Debug, Serialize, Deserialize]` only,
   no `Clone`. Codegen respects the explicit derive list.

Codegen replaces the data structs. Companion source templates own the module
documentation, non-serde imports, clarification artifact creation, append
behavior, state updates, status formatting, and regression tests. The serde
import is owned by the generated schema block.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  QuestionAnswer:
    type: object
    required: [topic, question, answer, rationale]
    description: A question-answer pair captured during pre-gap clarification.
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
      derive: [Debug, Serialize, Deserialize]

  CreateClarificationsInput:
    type: object
    required: [change_id, questions]
    description: Input for creating pre-clarifications artifact.
    properties:
      change_id:
        type: string
        description: "Change identifier slug."
      questions:
        type: array
        items:
          $ref: "#/definitions/QuestionAnswer"
        description: "Question-answer rows to include in the new artifact."
    x-rust-struct:
      derive: [Debug, Serialize, Deserialize]

  AppendClarificationsInput:
    type: object
    required: [change_id, issue, questions]
    description: Input for appending per-issue pre-clarifications to an existing artifact.
    properties:
      change_id:
        type: string
        description: "Change identifier slug."
      issue:
        type: integer
        x-rust-type: "Option<u64>"
        description: "Optional issue number to anchor the appended Q&A section."
      questions:
        type: array
        items:
          $ref: "#/definitions/QuestionAnswer"
        description: "Question-answer rows to append."
    x-rust-struct:
      derive: [Debug, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/pre_clarifications_service.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - QuestionAnswer
      - CreateClarificationsInput
      - AppendClarificationsInput
    description: |
      Codegen replaces all 3 struct declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [schema] All three struct definitions match the source exactly: field names, types (`String`, `Vec<QuestionAnswer>`, `Option<u64>`), and derives (`[Debug, Serialize, Deserialize]` — no Clone). The `x-rust-type: "Option<u64>"` annotation on `AppendClarificationsInput.issue` combined with its `required:` listing correctly suppresses double-wrapping, consistent with the `sdd-generate-sequence` pattern.
- [overview] Hand-written boundary is fully enumerated (module docstring, all six `use` imports, both function signatures with approximate LOC counts, and test block) — unambiguous for the codegen implementer.
- [changes] Two change entries for the same file cleanly separate the `impl_mode: codegen` (the three structs) from `impl_mode: hand-written` (everything else). The `replaces:` list names all three structs, matching `## Schema` definitions exactly.

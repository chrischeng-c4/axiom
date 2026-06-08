---
id: sdd-tools-validate-proposal
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# ValidationSummary Type

## Overview
<!-- type: overview lang: markdown -->

Validation result summary in `projects/agentic-workflow/src/tools/validate_proposal.rs`.
One public shape:

- `ValidationSummary` — `high_count: usize`, `medium_count: usize`,
  `low_count: usize`, `errors: Vec<String>`,
  `validation_errors: Vec<ValidationError>`, `stale_files: Vec<String>`.
  No derives.

The public struct is schema generated. Source fragments in
`tools/validate_proposal/` own summary behavior, error accumulation, and the
main validation command flow.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ValidationSummary:
    type: object
    required: [high_count, medium_count, low_count, errors, validation_errors, stale_files]
    description: Validation result summary.
    properties:
      high_count:
        type: integer
        x-rust-type: "usize"
        description: "Number of high-severity errors."
      medium_count:
        type: integer
        x-rust-type: "usize"
        description: "Number of medium-severity errors."
      low_count:
        type: integer
        x-rust-type: "usize"
        description: "Number of low-severity errors."
      errors:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Plain-text error messages."
      validation_errors:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ValidationError>"
        description: "Structured validation errors."
      stale_files:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Files with stale content."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/validate_proposal.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ValidationSummary
    description: |
      Codegen replaces the public struct only.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] Single public struct; private accumulator preserved.
- [schema] All in `required:`; usize + Vec<T> via x-rust-type.
- [changes] Standard split.

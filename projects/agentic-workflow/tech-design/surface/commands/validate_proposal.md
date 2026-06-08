---
id: score-validate-proposal
fill_sections: [schema, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "Command/root TDs support CLI workflow chain routing and root-runner dispatch."
---

# ValidationSummary Type

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
  - path: projects/agentic-workflow/src/cli/validate_proposal.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ValidationSummary
    description: |
      Codegen replaces the struct declaration only.
  - path: projects/agentic-workflow/src/cli/validate_proposal.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module imports, the
      `impl ValidationSummary { is_valid, is_valid_strict, has_warnings,
      to_json_output, ... }` block, the `validate_proposal` entry point,
      and all helpers.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Single struct, no derives, with multiple impl methods.
- [schema] All in `required:`; usize + Vec<T> via x-rust-type. Same shape as the sdd-side validate_proposal but in the score.
- [changes] Standard split.

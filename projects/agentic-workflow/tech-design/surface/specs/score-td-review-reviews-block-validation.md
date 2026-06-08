---
id: score-td-review-reviews-block-validation
summary: Keep TD inline review headings outside section-format validation.
fill_sections: [logic, test-plan, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# TD Review Reviews Block Validation

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: score-td-review-reviews-block-validation-flow
entry: scan_document
nodes:
  scan_document: {kind: start, label: "parse spec markdown"}
  read_line: {kind: process, label: "iterate non-frontmatter lines"}
  fence_check: {kind: decision, label: "inside fenced code?"}
  h1_check: {kind: decision, label: "line is H1 # Reviews?"}
  stop_sections: {kind: terminal, label: "stop emitting TD sections"}
  h2_check: {kind: decision, label: "line is H2 section heading?"}
  emit_section: {kind: process, label: "emit annotated or unannotated TD section"}
  next_line: {kind: process, label: "advance"}
  done: {kind: terminal, label: "return SpecDocument"}
edges:
  - {from: scan_document, to: read_line}
  - {from: read_line, to: fence_check}
  - {from: fence_check, to: next_line, label: "yes"}
  - {from: fence_check, to: h1_check, label: "no"}
  - {from: h1_check, to: stop_sections, label: "H1 text equals Reviews"}
  - {from: h1_check, to: h2_check, label: "other line"}
  - {from: h2_check, to: emit_section, label: "yes"}
  - {from: h2_check, to: next_line, label: "no"}
  - {from: emit_section, to: next_line}
  - {from: next_line, to: read_line}
  - {from: read_line, to: done, label: "EOF"}
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Shared-parser exclusion is the right layer because R7a, duplicate-section, and format-priority rules all consume the parsed section list.
- [test-plan] Regression coverage names the observed failure mode and the duplicate-section side effect.
- [changes] Scope is bounded to parser behavior, validation tests, and the td review brief text.

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: score-td-review-reviews-block-validation-tests
---
requirementDiagram
    requirement T1 {
        id: T1
        text: "parser_returns_only_pre_reviews_sections_for_h2_reviews"
        risk: high
        verifymethod: test
    }
    requirement T2 {
        id: T2
        text: "missing_annotation_ignores_review_h2"
        risk: high
        verifymethod: test
    }
    requirement T3 {
        id: T3
        text: "duplicate_section_ignores_review_heading_matching_real_section"
        risk: medium
        verifymethod: test
    }
    requirement T4 {
        id: T4
        text: "td_review_brief_mentions ### Review N"
        risk: medium
        verifymethod: manual
    }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/parser.rs
    action: modify
    section: logic
    description: Stop parsing TD sections once a top-level # Reviews H1 is reached, while still respecting fenced code blocks.
    impl_mode: hand-written
  - path: projects/agentic-workflow/src/validate/rules/r7a_missing_section_annotation.rs
    action: modify
    section: logic
    description: Add regression coverage that unannotated review H2 headings are ignored by R7a.
    impl_mode: hand-written
  - path: projects/agentic-workflow/src/validate/rules/r7c_duplicate_section.rs
    action: modify
    section: logic
    description: Add regression coverage that review H2 headings do not participate in duplicate-section checks.
    impl_mode: hand-written
  - path: projects/agentic-workflow/src/cli/td.rs
    action: modify
    section: logic
    description: Update the td review brief to request ### Review N under # Reviews.
    impl_mode: hand-written
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```

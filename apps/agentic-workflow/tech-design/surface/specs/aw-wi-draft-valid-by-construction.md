---
id: aw-wi-draft-valid-by-construction
summary: "Make aw wi draft authoring valid-by-construction."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# AW WI Draft Valid By Construction

## Overview
<!-- type: overview lang: markdown -->

`aw wi draft init`, `aw wi draft fill`, and `aw wi create` must not write or
publish work-item bodies that immediately fail the required structured
work-item format. The authoring path owns the mechanical shape: R-id
requirement bullets, Scope subsections, publishable default template content,
and hard validation before tracker creation or draft promotion.

Epic authoring additionally owns the planner's requirement-level verification
contract. Its full-section fill template includes a `## Verification
Inventory` table, and terminal WI validation requires every `R<n>` requirement
to map to at least one non-placeholder runnable Gate and observable Oracle.
The optional `Depends On` column declares same-epic requirement ordering and
rejects unknown ids, self-dependencies, and cycles.
This keeps `aw wi plan` reviewable by construction instead of producing
children whose verification must be invented after atomization.

The CLI may normalize known draft shapes that are safe and deterministic:
free-form body text becomes the Problem section inside the structured template,
unnumbered requirement list items receive sequential `R<n>:` ids, and flat
Scope lists are wrapped under `### In Scope` with a default `### Out of Scope`
subsection. Unknown or semantically incomplete structured sections still fail
with actionable validation errors instead of being published.

## Schema
<!-- type: schema lang: yaml -->

```yaml
draft_authoring_contract:
  commands:
    - aw wi draft init
    - aw wi draft fill
    - aw wi create
  invariants:
    - "Default draft templates contain publishable structured sections."
    - "Requirements list items use R-id prefixes before validation."
    - "Scope contains both In Scope and Out of Scope subsections before validation."
    - "Create blocks invalid draft bodies before remote tracker publication."
    - "Validation errors list the exact section/format problems."
    - "Epic full-section fills include a Verification Inventory table."
    - "Every epic Requirement maps to a runnable Gate and observable Oracle before draft promotion."
    - "Epic requirement dependencies are same-epic, known, non-self, and acyclic."
  normalized_shapes:
    free_text_body: "wrap as ## Problem inside the structured template"
    unnumbered_requirement_list: "rewrite each item as - R<n>: ..."
    flat_scope_list: "wrap under ### In Scope and add ### Out of Scope"
  rejected_shapes:
    missing_reference_context_tables: "hard validation error"
    missing_capability_alignment_fields: "hard validation error"
    missing_acceptance_criteria: "hard validation error"
    missing_epic_requirement_verification: "hard validation error naming each uncovered R-id"
    invalid_agent_estimate: "hard validation error"
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-wi-draft-valid-by-construction
scenarios:
  - id: S1
    title: "default draft validates"
    given:
      - "a user runs aw wi draft init without body text"
    then:
      - "the generated draft body contains R1, Scope subsections, Capability Alignment, Acceptance Criteria, and Reference Context tables"
      - "aw wi draft validate passes without requiring manual structural repair"

  - id: S2
    title: "known authoring shapes are normalized"
    given:
      - "a user supplies structured body text with unnumbered requirement bullets"
      - "the Scope section is a flat list"
    then:
      - "Requirements are rewritten to R-id bullets"
      - "Scope is wrapped under In Scope and Out of Scope subsections"
      - "the merged body passes publish validation"

  - id: S3
    title: "create blocks invalid drafts"
    given:
      - "a draft is missing Reference Context tables or other required publish sections"
    when:
      - "the user runs aw wi create <draft>"
    then:
      - "the CLI exits with validation status"
      - "the tracker issue is not created"
      - "the response lists actionable section errors"

  - id: S4
    title: "epic verification is complete before planning"
    given:
      - "an epic contains R1 and R2"
      - "the author fills the canonical all-section payload"
    then:
      - "the payload schema admits exactly one Verification Inventory H2 alongside the normal epic sections"
      - "validation rejects a missing or placeholder Gate or Oracle for either R1 or R2"
      - "validation passes only after both requirement ids have runnable gates and observable oracles"
      - "the optional Depends On column may declare an acyclic dependency on another Requirement id"
      - "aw wi plan receives requirement-specific verification without an out-of-schema workaround"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Add draft-body normalization, publish validation, valid default template
      content, epic Verification Inventory authoring, and regression tests for
      draft init/fill/create behavior.
  - path: apps/agentic-workflow/src/issues/planner.rs
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: |
      Reuse the planner's Verification Inventory parser as the epic
      requirement-coverage validation oracle.
  - path: apps/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md
    action: create
    section: overview
    impl_mode: hand-written
    description: |
      Capture the draft authoring valid-by-construction contract.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

```

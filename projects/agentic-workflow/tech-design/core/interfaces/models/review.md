---
id: sdd-models-review
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Review

## Overview
<!-- type: overview lang: markdown -->

Review model primitives for the `sdd` crate, located in `projects/agentic-workflow/src/models/review.rs`:

- `ReviewVerdict` — 4-variant unit enum (`Approved`, `Reviewed`, `Rejected`, `Unknown`) representing the outcome of a review. Three bool-returning dispatch methods (`is_approved`, `needs_refinement`, `has_major_issues`) use `body:` literal codegen via `matches!(self, Self::<Variant>)`, following the `ChallengeVerdict` pattern established in `challenge.md`. Note: derives do NOT include `Copy` (unlike `ChallengeVerdict`).
- `IssueCategory` — 8-variant unit enum (`Security`, `Performance`, `Style`, `MissingFeature`, `WrongBehavior`, `Consistency`, `TestCoverage`, `EdgeCase`) classifying the type of review issue. No inherent methods; no `x-methods` or `x-constructor`. Its `Display` impl is generated from `x-trait-impls`.
- `ReviewIssue` — struct (6 fields: `title`, `severity`, `category`, `location`, `description`, `recommendation`) representing a single issue found during review. No constructor in source; `x-constructor` is omitted accordingly.
- `IssueSeverity` — re-exported from `models::challenge` via `pub use super::challenge::IssueSeverity`. This re-export is not declared in `review.rs` and is untouched by codegen.

Codegen scope: CODEGEN-BEGIN/CODEGEN-END blocks replace `ReviewVerdict`, `impl ReviewVerdict`, `impl Display for ReviewVerdict`, `IssueCategory`, `impl Display for IssueCategory`, and `ReviewIssue`. The schema block also generates the serde import it needs. All generated items carry `@spec` markers referencing this file's `#schema` anchor.
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReviewVerdict:
    type: string
    enum:
      - Approved
      - Reviewed
      - Rejected
      - Unknown
    description: |
      Verdict from a review.
      Approved: review is approved and ready for implementation.
      Reviewed: review needs refinement to address issues.
      Rejected: review is rejected due to major issues.
      Unknown: verdict could not be determined (parsing error or not filled).
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]
      variants:
        - name: Approved
          doc: "Review is approved and ready for implementation."
        - name: Reviewed
          doc: "Review needs refinement to address issues."
        - name: Rejected
          doc: "Review is rejected due to major issues."
        - name: Unknown
          doc: "Verdict could not be determined (parsing error or not filled)."
    x-methods:
      - name: is_approved
        returns: bool
        impl_mode: codegen
        body: "matches!(self, Self::Approved)"
        doc: "True iff verdict is Approved."
      - name: needs_refinement
        returns: bool
        impl_mode: codegen
        body: "matches!(self, Self::Reviewed)"
        doc: "True iff verdict is Reviewed (needs refinement)."
      - name: has_major_issues
        returns: bool
        impl_mode: codegen
        body: "matches!(self, Self::Rejected)"
        doc: "True iff verdict is Rejected (has major issues)."
    x-trait-impls:
      - trait: Display
        impl_mode: codegen
        dispatch:
          - { variant: Approved, value: "APPROVED" }
          - { variant: Reviewed, value: "REVIEWED" }
          - { variant: Rejected, value: "REJECTED" }
          - { variant: Unknown,  value: "UNKNOWN" }

  IssueCategory:
    type: string
    enum:
      - Security
      - Performance
      - Style
      - MissingFeature
      - WrongBehavior
      - Consistency
      - TestCoverage
      - EdgeCase
    description: |
      Category classifying the type of review issue.
      Display impl is hand-written outside CODEGEN — external trait impl emission is a future generator feature.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Eq]
      variants:
        - name: Security
          doc: "Security-related issue."
        - name: Performance
          doc: "Performance-related issue."
        - name: Style
          doc: "Code style issue."
        - name: MissingFeature
          doc: "Missing feature."
        - name: WrongBehavior
          doc: "Incorrect behavior."
        - name: Consistency
          doc: "Consistency issue."
        - name: TestCoverage
          doc: "Missing or insufficient test coverage."
        - name: EdgeCase
          doc: "Unhandled edge case."
    x-trait-impls:
      # IssueCategory's Display values use spaces between camel-case words
      # ("Missing Feature" not "MissingFeature").
      - trait: Display
        impl_mode: codegen
        dispatch:
          - { variant: Security,       value: "Security" }
          - { variant: Performance,    value: "Performance" }
          - { variant: Style,          value: "Style" }
          - { variant: MissingFeature, value: "Missing Feature" }
          - { variant: WrongBehavior,  value: "Wrong Behavior" }
          - { variant: Consistency,    value: "Consistency" }
          - { variant: TestCoverage,   value: "Test Coverage" }
          - { variant: EdgeCase,       value: "Edge Case" }

  ReviewIssue:
    type: object
    required: [title, severity, category, location, description, recommendation]
    description: Represents a single issue found during review.
    properties:
      title:
        type: string
        description: "Issue title."
      severity:
        $ref: "challenge.md#/definitions/IssueSeverity"
        description: "Severity level of the issue. Re-exported from models::challenge."
      category:
        $ref: "#/definitions/IssueCategory"
        description: "Category of the issue."
      location:
        type: string
        description: "Location in the reviewed artifact (e.g., 'src/models/review.rs:42')."
      description:
        type: string
        description: "Description of the issue."
      recommendation:
        type: string
        description: "Recommended fix or improvement."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/review.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ReviewVerdict
      - "impl ReviewVerdict"
      - IssueCategory
      - ReviewIssue
      - "impl std::fmt::Display for ReviewVerdict"
      - "impl std::fmt::Display for IssueCategory"
    description: |
      Codegen: 3 type decls + 1 inherent impl block + 2 trait impl blocks.
      Emits: ReviewVerdict enum decl, impl ReviewVerdict (is_approved /
      needs_refinement / has_major_issues via body: literal), IssueCategory
      enum decl (no inherent methods), ReviewIssue struct decl (no
      constructor in source), and Display impls for both enums via the
      new `x-trait-impls` extension (commit dcc18350).
      replaces: now also targets the previously-hand-written
      `impl std::fmt::Display for X` blocks so they get swept by the
      attribute-aware deletion before the new CODEGEN block is inserted.
      All generated items carry @spec markers referencing this file's
      #schema anchor.
  - path: projects/agentic-workflow/src/models/review.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Outside CODEGEN-BEGIN/CODEGEN-END (hand-written, untouched by aw td gen-code):
      - pub use super::challenge::IssueSeverity — re-export of IssueSeverity from the
        challenge module; untouched by codegen. IssueSeverity is declared in challenge.md
        and referenced here via $ref.
      This region carries no @spec marker (healthy hand-written region per audit policy).
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Clear and complete: correctly identifies all four codegen-managed items, explicitly calls out the `Copy`-omission divergence from `ChallengeVerdict`, and documents both Display-deferred gaps and the `IssueSeverity` re-export as untouched by codegen.
- [schema] `replaces` contains `"impl ReviewVerdict"` but neither `"impl IssueCategory"` nor any `"impl Display for X"` — satisfies the sanity-check constraint. All three bool methods use `body: "matches!(self, Self::Variant)"` literals with `impl_mode: codegen`, matching the `ChallengeVerdict` pattern. `IssueCategory` has no `x-methods` or `x-constructor`. `ReviewIssue` omits `x-constructor` correctly.
- [changes] Two-entry split (codegen / hand-written) is correctly structured. The hand-written entry lists Display impls and the re-export without any `replaces:` key, ensuring gen-code leaves those regions untouched.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] `ReviewVerdict` `x-trait-impls` confirmed: 1 Display trait impl with 4 dispatch entries (Approved="APPROVED", Reviewed="REVIEWED", Rejected="REJECTED", Unknown="UNKNOWN"). All variants accounted for, values match R6.
- [schema] `IssueCategory` `x-trait-impls` confirmed: 1 Display trait impl with 8 dispatch entries. Spaced display values (e.g. "Missing Feature", "Wrong Behavior", "Test Coverage", "Edge Case") match R2 and preserve semantic parity with the hand-written impls being replaced.
- [changes] `replaces:` in the codegen entry now includes `"impl std::fmt::Display for ReviewVerdict"` and `"impl std::fmt::Display for IssueCategory"` as required by R3. The hand-written entry's `description:` still references these impls, but that entry carries no `replaces:` key — the description is explanatory context for the audit trail, not a conflicting directive. Internally consistent.

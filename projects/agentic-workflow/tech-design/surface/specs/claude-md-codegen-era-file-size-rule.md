---
id: claude-md-codegen-era-file-size-rule
fill_sections: [changes, tests]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: /Users/chris.cheng/cclab/main/CLAUDE.md
    action: modify
    section: changes
    impl_mode: hand-written
    anchor: "## Constraints"
    description: >
      Replace the single "File size limit" bullet with a three-class
      codegen-aware rule. Old text: "- **File size limit**: If file lines >= 1000,
      must split. If >= 500, consider split." New text covers three classes:
      spec markdown, hand-written source, and generated source (no size limit).
    old: |
      - **File size limit**: If file lines >= 1000, must split. If >= 500, consider split.
    new: |
      - **File size limits** (codegen-aware):
        - **Spec markdown** (`.aw/tech-design/**/*.md`): consider split if >= 500 lines, must split if >= 1000. Spec splits should follow semantic grouping (one type-family per file), not arbitrary line cuts.
        - **Hand-written source** (`.rs`/`.ts`/`.py` not inside a `CODEGEN-BEGIN`/`CODEGEN-END` block): consider split if >= 500 lines, must split if >= 1000. Same rule as before — humans navigate this.
        - **Generated source** (between `CODEGEN-BEGIN`/`CODEGEN-END`): no size limit. Size is determined by the spec's type cardinality. If you want smaller emission, split the spec.
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  - name: generated-file-no-longer-flagged
    kind: manual
    description: >
      After merge, verify that `projects/agentic-workflow/src/models/change.rs` (1011 lines,
      generated via a CODEGEN-BEGIN/CODEGEN-END block) is not flagged as a
      size violation. Under the old single-class rule it exceeded the 1000-line
      must-split threshold; under the new three-class rule it is exempt because
      it is generated source.
    steps:
      - step: Open `projects/agentic-workflow/src/models/change.rs` and confirm it contains
               `CODEGEN-BEGIN` and `CODEGEN-END` markers.
      - step: Count the file's lines (expected > 1000).
      - step: Confirm that no review comment or linter rule demands a split
               solely on the basis of line count, because the file falls under
               the "generated source" class with no size limit.
    expected: File is not required to be split; size-related review noise is absent.
  - name: spec-markdown-threshold-unchanged
    kind: manual
    description: >
      Verify that a spec file under `.aw/tech-design/` with >= 1000 lines
      is still flagged as must-split.
    steps:
      - step: Identify any spec markdown file with >= 1000 lines.
      - step: Confirm reviewer or linter still flags it for a split per the
               spec-markdown class rule (>= 1000 lines must split).
    expected: Spec markdown threshold is unchanged at 500 (consider) / 1000 (must).
  - name: hand-written-source-threshold-unchanged
    kind: manual
    description: >
      Verify that a hand-written `.rs` file (no CODEGEN markers) with >= 1000
      lines is still flagged as must-split.
    steps:
      - step: Identify a hand-written `.rs` file with >= 1000 lines and no
               CODEGEN markers.
      - step: Confirm reviewer or linter flags it for a split per the
               hand-written source class rule (>= 1000 lines must split).
    expected: Hand-written source threshold is unchanged at 500 (consider) / 1000 (must).
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Clear and accurate rationale; correctly identifies the failure mode of the single-class rule and motivates all three new classes.
- [changes] The `old` field exactly matches the current CLAUDE.md bullet; the `new` field correctly implements R1 and R2. The absolute path is workable for a doc-only change. R3 rationale is adequately covered inline in the generated-source bullet and in the overview.
- [tests] All three classes are exercised with concrete manual steps and expected outcomes. Coverage is complete.

---
id: codegen-skip-annotation-convention
fill_sections: [changes]
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
  - path: /Users/chris.cheng/cclab/main/projects/agentic-workflow/src/tools/analyze/mod.rs
    action: modify
    section: changes
    impl_mode: hand-written
    anchor: "//! "
    description: |
      Insert above the existing `//!` module-level doc lines (or as the first
      lines if none exist). Three new lines:
        //! @codegen-skip: test-fixture-only
        //! Reason: `pub struct/enum` matches in this file (e.g. line 372:
        //! `pub struct Config`) are inside r#"..."# raw strings used as test
        //! fixtures fed to `rust_lang::analyze`, not real Rust type definitions.

  - path: /Users/chris.cheng/cclab/main/projects/agentic-workflow/src/cli/fillback.rs
    action: modify
    section: changes
    impl_mode: hand-written
    anchor: "//! "
    description: |
      Same annotation as above. Three lines:
        //! @codegen-skip: test-fixture-only
        //! Reason: `pub struct/enum` matches in this file (e.g. line 127:
        //! `pub struct Config`) are inside r#"..."# raw strings used as test
        //! fixtures, not real Rust type definitions.

  - path: /Users/chris.cheng/cclab/main/CLAUDE.md
    action: modify
    section: changes
    impl_mode: hand-written
    anchor: "## Constraints"
    description: |
      Append a new bullet under `## Constraints`, after the codegen-era
      file-size three-class bullet:

        - **Dogfood scan convention** (codegen markers vs skip annotation):
          Files whose `pub struct/enum` matches are inside test fixtures
          (`r#"..."#` raw strings) should carry a top-of-file
          `//! @codegen-skip: test-fixture-only` annotation with a
          one-paragraph reason. Future scans for remaining dogfood
          candidates should filter these out:

          ```bash
          find <paths> -name "*.rs" \
            | xargs grep -L 'CODEGEN-BEGIN\|@codegen-skip' \
            | xargs grep -lE "^pub (struct|enum) "
          ```

          Result is the list of unmarked, undecided files. Apply the
          `@codegen-skip` marker to confirmed fixture-only files; run
          `aw td gen-code` on files with real type definitions.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Accurately identifies both target files and the three-part change scope; no corrections needed.
- [changes] All three entries carry concrete anchors and exact insertion text; the CLAUDE.md bash snippet is self-contained and correct.

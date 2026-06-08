---
id: sdd-validate-router
fill_sections: [overview, schema, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Validate Router

## Overview
<!-- type: overview lang: markdown -->

Path-shape classifier for `aw td validate <target>` in the `sdd` crate.

`PathShape` is a three-variant enum that classifies the string argument passed
to `aw td validate`:

- `Slug(String)` — an issue identifier (no slashes, no `.md` suffix); activates
  the CRRR commit-gate path in the score.
- `Prefix(PathBuf)` — a directory under `.aw/tech-design/`; the validator
  walks every `.md` file recursively in read-only mode.
- `File(PathBuf)` — a single `.md` spec file; the validator checks just that
  file in read-only mode.

Free functions `classify`, `resolve_spec_files`, `walk_markdown`, and
`join_under_root` operate on `PathShape` by reference and remain hand-written.
This spec migrates only the `PathShape` declaration to codegen provenance; no
runtime behaviour changes. The existing `#[cfg(test)] mod tests` block is
preserved verbatim outside any CODEGEN delimiters.

This spec serves as the dogfood reference example for mixed-mode codegen on an
enum with payload (tuple) variants, analogous to `validate/rule.md` for struct
codegen.
## Schema
<!-- type: schema lang: yaml -->

```yaml
$id: sdd-validate-router
description: |
  Path-shape classifier for aw td validate <target>.
  PathShape is a three-variant enum whose variants carry tuple payloads;
  it drives the CRRR commit-gate (Slug) and read-only walking (Prefix, File).
  No serde derives — this type is internal to the sdd crate only.

definitions:
  PathShape:
    type: string
    description: |
      What kind of argument the caller passed to aw td validate.
      Three tuple variants; no unit variants, no serde.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Eq]
      variants:
        - name: Slug
          kind: tuple
          fields:
            - { rust_type: String }
          doc: "Issue identifier — no slashes, no .md."
        - name: Prefix
          kind: tuple
          fields:
            - { rust_type: PathBuf }
          doc: "Directory under .aw/tech-design/."
        - name: File
          kind: tuple
          fields:
            - { rust_type: PathBuf }
          doc: "Single .md spec file."
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validate/router.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - PathShape
    description: |
      Codegen replaces the PathShape enum declaration with three tuple variants
      (Slug(String), Prefix(PathBuf), File(PathBuf)) and four derives
      (Debug, Clone, PartialEq, Eq). The generated block is wrapped in
      CODEGEN-BEGIN/CODEGEN-END delimiters and carries a @spec marker
      referencing this file's #schema anchor.
  - path: projects/agentic-workflow/src/validate/router.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written items outside CODEGEN blocks: free functions classify,
      resolve_spec_files, walk_markdown, and join_under_root (none are inherent
      methods on PathShape — all take PathShape by reference). These live
      outside any CODEGEN-BEGIN/CODEGEN-END delimiter and carry no @spec marker
      (healthy hand-written region per audit policy).
      The #[cfg(test)] mod tests block is also preserved outside CODEGEN
      delimiters; aw td gen-code must not touch it on an action: modify file.
      All existing tests must continue to pass after codegen block insertion.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [schema] Enum declaration matches source exactly: three tuple variants (`Slug(String)`, `Prefix(PathBuf)`, `File(PathBuf)`), four derives (`Debug, Clone, PartialEq, Eq`), no serde — fully consistent with `router.rs` lines 15-27.
- [changes] Both codegen and hand-written entries are present and correctly scoped; `replaces: [PathShape]` is unambiguous for gen-code; test block preservation is explicitly stated.
- [overview] Dogfood reference purpose and mixed-mode provenance intent are clearly stated; free-function hand-written boundary is unambiguous.

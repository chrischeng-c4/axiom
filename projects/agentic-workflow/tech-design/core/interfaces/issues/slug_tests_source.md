---
id: sdd-interfaces-issues-slug-tests-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Issue Slug Tests Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/slug.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BranchKind` | projects/agentic-workflow/src/issues/slug.rs | enum | pub | 28 |  |
| `ResolvedId` | projects/agentic-workflow/src/issues/slug.rs | enum | pub | 48 |  |
| `SlugAliases` | projects/agentic-workflow/src/issues/slug.rs | struct | pub | 77 |  |
| `as_prefix` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 36 | as_prefix(&self) -> &'static str |
| `build_branch_name` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 201 | build_branch_name(kind: BranchKind, id: u64, title: &str) -> String |
| `build_canonical_slug` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 120 | build_canonical_slug(id: u64, _title: &str) -> String |
| `id` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 59 | id(&self) -> u64 |
| `insert` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 112 | insert(&mut self, legacy_slug: String, id: u64) |
| `is_legacy` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 67 | is_legacy(&self) -> bool |
| `load` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 85 | load(project_root: &Path) -> Result<Self> |
| `lookup` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 108 | lookup(&self, legacy_slug: &str) -> Option<u64> |
| `parse_branch_name` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 180 | parse_branch_name(branch: &str) -> Option<(BranchKind, u64)> |
| `parse_slug_input` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 132 | parse_slug_input(input: &str, aliases: &SlugAliases) -> Result<ResolvedId> |
| `save` | projects/agentic-workflow/src/issues/slug.rs | function | pub | 98 | save(&self, project_root: &Path) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap slug-parser-builder-tests -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/slug.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:slug-parser-builder-tests>"
    description: "Source template owns slug parser and branch-name regression tests."
```

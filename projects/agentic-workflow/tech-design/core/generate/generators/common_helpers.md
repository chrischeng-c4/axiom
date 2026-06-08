---
id: sdd-generate-generators-common-helpers
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Generator Common Helper Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/common.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FileStatus` | projects/agentic-workflow/src/generate/generators/common.rs | enum | pub | 16 |  |
| `GeneratedFile` | projects/agentic-workflow/src/generate/generators/common.rs | struct | pub | 25 |  |
| `GeneratorError` | projects/agentic-workflow/src/generate/generators/common.rs | enum | pub | 37 |  |
| `GeneratorSettings` | projects/agentic-workflow/src/generate/generators/common.rs | struct | pub | 53 |  |
| `Manifest` | projects/agentic-workflow/src/generate/generators/common.rs | struct | pub | 80 |  |
| `OverwritePolicy` | projects/agentic-workflow/src/generate/generators/common.rs | enum | pub | 88 |  |
| `add` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 144 | add(&mut self, file: GeneratedFile) |
| `error` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 128 | error(path: PathBuf, error: impl Into<String>) -> Self |
| `error_count` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 162 | error_count(&self) -> usize |
| `new` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 140 | new() -> Self |
| `skipped` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 119 | skipped(path: PathBuf) -> Self |
| `skipped_count` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 155 | skipped_count(&self) -> usize |
| `written` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 103 | written(path: PathBuf, content: &str) -> Self |
| `written_count` | projects/agentic-workflow/src/generate/generators/common.rs | function | pub | 148 | written_count(&self) -> usize |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/common.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns common generator helper impls and traits."
```

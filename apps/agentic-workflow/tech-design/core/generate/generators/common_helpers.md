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

Public API manifest for `apps/agentic-workflow/src/generate/generators/common.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FileStatus` | apps/agentic-workflow/src/generate/generators/common.rs | enum | pub | 16 |  |
| `GeneratedFile` | apps/agentic-workflow/src/generate/generators/common.rs | struct | pub | 25 |  |
| `GeneratorError` | apps/agentic-workflow/src/generate/generators/common.rs | enum | pub | 37 |  |
| `GeneratorSettings` | apps/agentic-workflow/src/generate/generators/common.rs | struct | pub | 53 |  |
| `Manifest` | apps/agentic-workflow/src/generate/generators/common.rs | struct | pub | 80 |  |
| `OverwritePolicy` | apps/agentic-workflow/src/generate/generators/common.rs | enum | pub | 88 |  |
| `add` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 144 | add(&mut self, file: GeneratedFile) |
| `error` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 128 | error(path: PathBuf, error: impl Into<String>) -> Self |
| `error_count` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 162 | error_count(&self) -> usize |
| `new` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 140 | new() -> Self |
| `skipped` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 119 | skipped(path: PathBuf) -> Self |
| `skipped_count` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 155 | skipped_count(&self) -> usize |
| `written` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 103 | written(path: PathBuf, content: &str) -> Self |
| `written_count` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 148 | written_count(&self) -> usize |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/generate/generators/common.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns common generator helper impls and traits."
```

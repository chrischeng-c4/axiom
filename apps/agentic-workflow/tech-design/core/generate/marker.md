---
id: sdd-generate-marker
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Generate Marker Module

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/generate/marker.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodegenBlock` | apps/agentic-workflow/src/generate/marker.rs | struct | pub | 83 |  |
| `Lang` | apps/agentic-workflow/src/generate/marker.rs | enum | pub | 33 |  |
| `MarkerEntry` | apps/agentic-workflow/src/generate/marker.rs | struct | pub | 512 |  |
| `collect_spec_refs` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 522 | collect_spec_refs(file_path: &str, file_content: &str) -> Vec<MarkerEntry> |
| `emit_spec_ref` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 500 | emit_spec_ref(spec_path: &str, section: &str, task: &str, lang: Lang) -> String |
| `group_markers` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 565 | group_markers(entries: Vec<MarkerEntry>) -> HashMap<String, Vec<MarkerEntry>> |
| `insert_codegen_block` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 461 | insert_codegen_block(     file_content: &str,     spec_ref: &str,     initial_content: &str,     anchor_text: Option<&str>,     lang: Lang, ) -> String |
| `line_comment` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 47 | line_comment(&self) -> &'static str |
| `line_comment_end` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 58 | line_comment_end(&self) -> &'static str |
| `parse_codegen_blocks` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 103 | parse_codegen_blocks(file_content: &str) -> Vec<CodegenBlock> |
| `replace_codegen_block` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 378 | replace_codegen_block(file_content: &str, spec_ref: &str, new_content: &str) -> String |
| `rust_raw_string_line_mask` | apps/agentic-workflow/src/generate/marker.rs | function | pub | 199 | rust_raw_string_line_mask(file_content: &str) -> Vec<bool> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=apps/agentic-workflow/src/generate/marker.rs -->
```rust
//! CODEGEN marker parser, replacer, and SPEC-REF emitter.
//!
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/generate/marker.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```

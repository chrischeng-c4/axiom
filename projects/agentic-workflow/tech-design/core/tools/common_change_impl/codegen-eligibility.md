---
id: sdd-tools-common-change-impl-codegen-eligibility
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Code generation tool TDs support CB lifecycle artifact generation and test generation."
---

# sdd tools common change impl codegen eligibility

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/common_change_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ImplSubState` | projects/agentic-workflow/src/tools/common_change_impl.rs | enum | pub | 26 |  |
| `MAX_SPEC_REVISIONS` | projects/agentic-workflow/src/tools/common_change_impl.rs | constant | pub | 15 |  |
| `build_spec_execution_order` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 249 | build_spec_execution_order(specs_dir: &Path) -> Vec<String> |
| `build_spec_execution_order_from_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 409 | build_spec_execution_order_from_paths(paths: &[std::path::PathBuf]) -> Vec<String> |
| `collect_all_spec_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 354 | collect_all_spec_paths(change_dir: &Path) -> Vec<std::path::PathBuf> |
| `find_inline_reviews` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 524 | find_inline_reviews(impl_path: &Path) -> (HashSet<String>, HashSet<String>) |
| `is_codegen_eligible_for_spec` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 583 | is_codegen_eligible_for_spec(specs_dir: &Path, spec_id: &str) -> bool |
| `is_codegen_eligible_in_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 433 | is_codegen_eligible_in_paths(paths: &[std::path::PathBuf], spec_id: &str) -> bool |
| `parse_refs_frontmatter` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 458 | parse_refs_frontmatter(path: &Path) -> Vec<String> |
| `resolve_next_impl` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 64 | resolve_next_impl(     change_dir: &Path,     _change_id: &str, ) -> Result<(ImplSubState, Option<String>, Option<String>)> |
## Source
<!-- type: source lang: rust -->

````rust
/// Check if a change spec is eligible for structured codegen.
pub fn is_codegen_eligible_for_spec(specs_dir: &Path, spec_id: &str) -> bool {
    let path = specs_dir.join(format!("{}.md", spec_id));
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    content.contains("has_json_schema: true") || content.contains("has_api_spec: true")
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_change_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "is_codegen_eligible_for_spec"
    description: "Legacy directory-based spec codegen eligibility check."
```

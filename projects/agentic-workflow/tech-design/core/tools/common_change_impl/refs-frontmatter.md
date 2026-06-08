---
id: sdd-tools-common-change-impl-refs-frontmatter
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change impl refs frontmatter

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
/// Parse `refs:` frontmatter from a spec file (list of dependency spec IDs).
pub fn parse_refs_frontmatter(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    if !content.starts_with("---") {
        return vec![];
    }

    let end = match content[3..].find("---") {
        Some(e) => e,
        None => return vec![],
    };

    let frontmatter = &content[3..3 + end];
    let mut in_refs = false;
    let mut refs = Vec::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("refs:") {
            let rest = trimmed.trim_start_matches("refs:").trim();
            if rest.starts_with('[') && rest.ends_with(']') {
                let inner = &rest[1..rest.len() - 1];
                for item in inner.split(',') {
                    let id = item.trim().trim_matches('"').trim_matches('\'');
                    if !id.is_empty() {
                        refs.push(id.to_string());
                    }
                }
            }
            in_refs = true;
            continue;
        }
        if in_refs {
            if trimmed.starts_with("- ") {
                let id = trimmed
                    .trim_start_matches("- ")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !id.is_empty() {
                    refs.push(id.to_string());
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        }
    }

    refs
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
      - "parse_refs_frontmatter"
    description: "Refs frontmatter parser for dependency spec IDs."
```

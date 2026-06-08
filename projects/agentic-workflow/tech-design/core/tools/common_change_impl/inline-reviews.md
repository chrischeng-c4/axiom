---
id: sdd-tools-common-change-impl-inline-reviews
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change impl inline reviews

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
/// Scan implementation.md for `## Review: {spec_id}` sections.
///
/// Returns `(reviewed, approved)` where:
/// - `reviewed` = all spec_ids that have any inline review section
/// - `approved` = spec_ids whose inline review has `verdict: APPROVED` or `PASS`
pub fn find_inline_reviews(impl_path: &Path) -> (HashSet<String>, HashSet<String>) {
    let mut reviewed = HashSet::new();
    let mut approved = HashSet::new();

    let content = match std::fs::read_to_string(impl_path) {
        Ok(c) => c,
        Err(_) => return (reviewed, approved),
    };

    let mut current_spec: Option<String> = None;
    let mut current_verdict: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("## Review:") {
            // Flush previous review
            if let Some(spec_id) = current_spec.take() {
                reviewed.insert(spec_id.clone());
                if let Some(ref v) = current_verdict {
                    if v == "APPROVED" || v == "PASS" {
                        approved.insert(spec_id);
                    }
                }
                current_verdict = None;
            }
            current_spec = Some(rest.trim().to_string());
            continue;
        }

        if current_spec.is_some() {
            if let Some(v) = trimmed.strip_prefix("verdict:") {
                let verdict = v.trim().trim_matches('"').trim_matches('\'').to_uppercase();
                current_verdict = Some(verdict);
            }
        }
    }

    // Flush last review
    if let Some(spec_id) = current_spec {
        reviewed.insert(spec_id.clone());
        if let Some(ref v) = current_verdict {
            if v == "APPROVED" || v == "PASS" {
                approved.insert(spec_id);
            }
        }
    }

    (reviewed, approved)
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
      - "find_inline_reviews"
    description: "Inline implementation review scanner for reviewed and approved spec IDs."
```

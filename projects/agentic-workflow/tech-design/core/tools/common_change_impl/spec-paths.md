---
id: sdd-tools-common-change-impl-spec-paths
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change impl spec paths

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
/// Collect all spec file paths for a change, supporting both group and legacy layouts.
///
/// - New layout: `change_dir/groups/*/specs/*.md`
/// - Legacy layout: `change_dir/specs/*.md`
///
/// If `groups/` exists and contains specs, those are returned. Otherwise falls
/// back to `specs/` for backward compatibility.
pub fn collect_all_spec_paths(change_dir: &Path) -> Vec<std::path::PathBuf> {
    let groups_dir = change_dir.join("groups");
    if groups_dir.is_dir() {
        let mut paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
            let mut group_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            group_entries.sort_by_key(|e| e.file_name());
            for entry in group_entries {
                if entry.path().is_dir() {
                    let group_specs = entry.path().join("specs");
                    if group_specs.is_dir() {
                        collect_md_paths(&group_specs, &mut paths);
                    }
                }
            }
        }
        if !paths.is_empty() {
            return paths;
        }
    }
    // Legacy fallback
    let specs_dir = change_dir.join("specs");
    let mut paths = Vec::new();
    collect_md_paths(&specs_dir, &mut paths);
    paths
}

/// Recursively collect non-symlink .md files from a directory.
fn collect_md_paths(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        sorted.sort_by_key(|e| e.file_name());
        for entry in sorted {
            let path = entry.path();
            if path.is_file()
                && path.extension().map(|x| x == "md").unwrap_or(false)
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".base.md"))
                    .unwrap_or(false)
            {
                out.push(path);
            } else if path.is_dir() {
                collect_md_paths(&path, out);
            }
        }
    }
}

/// Build topological execution order from a list of spec file paths.
///
/// Same Kahn's algorithm as `build_spec_execution_order` but accepts an
/// explicit path list instead of reading from a directory.
pub fn build_spec_execution_order_from_paths(paths: &[std::path::PathBuf]) -> Vec<String> {
    if paths.is_empty() {
        return vec![];
    }

    let mut spec_refs: Vec<(String, Vec<String>)> = Vec::new();
    for path in paths {
        let spec_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if spec_id.is_empty() {
            continue;
        }
        let refs = parse_refs_frontmatter(path);
        spec_refs.push((spec_id, refs));
    }

    kahn_sort(spec_refs)
}

/// Check if any spec matching `spec_id` in `paths` is codegen-eligible.
pub fn is_codegen_eligible_in_paths(paths: &[std::path::PathBuf], spec_id: &str) -> bool {
    let target = format!("{}.md", spec_id);
    for path in paths {
        if path
            .file_name()
            .map(|n| n == target.as_str())
            .unwrap_or(false)
        {
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if content.contains("has_json_schema: true") || content.contains("has_api_spec: true") {
                return true;
            }
        }
    }
    false
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
      - "collect_all_spec_paths"
      - "collect_md_paths"
      - "build_spec_execution_order_from_paths"
      - "is_codegen_eligible_in_paths"
    description: "Spec path collection and codegen eligibility checks over explicit paths."
```

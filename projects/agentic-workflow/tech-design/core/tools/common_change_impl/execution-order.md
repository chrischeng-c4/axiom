---
id: sdd-tools-common-change-impl-execution-order
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change impl execution order

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
/// Build topological execution order from change specs in specs/ dir.
///
/// Reads each `*.md` file's `refs:` frontmatter (YAML list of spec IDs this
/// spec depends on). Applies Kahn's algorithm with BTreeSet for lexical
/// tie-breaking to produce a deterministic execution order.
pub fn build_spec_execution_order(specs_dir: &Path) -> Vec<String> {
    if !specs_dir.exists() {
        return vec![];
    }

    let entries: Vec<_> = std::fs::read_dir(specs_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file() && e.path().extension().map(|x| x == "md").unwrap_or(false))
        .collect();

    if entries.is_empty() {
        return vec![];
    }

    let mut spec_refs: Vec<(String, Vec<String>)> = Vec::new();
    for entry in &entries {
        let path = entry.path();
        let spec_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if spec_id.is_empty() {
            continue;
        }
        let refs = parse_refs_frontmatter(&path);
        spec_refs.push((spec_id, refs));
    }

    kahn_sort(spec_refs)
}

/// Kahn's topological sort with BTreeSet for deterministic lexical tie-breaking.
///
/// Input: `Vec<(spec_id, deps)>` — dependency pairs.
/// Cycle members are appended in lexical order at the end.
fn kahn_sort(spec_refs: Vec<(String, Vec<String>)>) -> Vec<String> {
    let spec_id_set: HashSet<String> = spec_refs.iter().map(|(id, _)| id.clone()).collect();
    let mut in_degree: HashMap<&str, usize> =
        spec_refs.iter().map(|(id, _)| (id.as_str(), 0)).collect();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for (id, refs) in &spec_refs {
        for dep in refs {
            if spec_id_set.contains(dep) {
                *in_degree.entry(id.as_str()).or_insert(0) += 1;
                dependents
                    .entry(dep.as_str())
                    .or_default()
                    .push(id.as_str());
            }
        }
    }

    let mut ready: BTreeSet<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut order = Vec::new();
    while let Some(&id) = ready.iter().next() {
        ready.remove(id);
        order.push(id.to_string());

        if let Some(deps) = dependents.get(id) {
            for &dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        ready.insert(dep);
                    }
                }
            }
        }
    }

    // Append any remaining (cycle members) in lexical order
    let ordered_set: HashSet<String> = order.iter().cloned().collect();
    let mut remaining: Vec<&str> = spec_id_set
        .iter()
        .filter(|id| !ordered_set.contains(*id))
        .map(|s| s.as_str())
        .collect();
    remaining.sort();
    for id in remaining {
        order.push(id.to_string());
    }

    order
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
      - "build_spec_execution_order"
      - "kahn_sort"
    description: "Topological spec execution ordering with deterministic Kahn sort."
```

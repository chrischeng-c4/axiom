//! Shared helpers for change-implementation per-action tools.
//!
//! - `ImplSubState` enum + `resolve_next_impl()` — sub-state machine
//! - `build_spec_execution_order()` — Kahn's algorithm on `refs:` frontmatter
//! - `find_inline_reviews()` — scan implementation.md for review verdicts
//! - `is_codegen_eligible_for_spec()` — check frontmatter flags

use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

/// Maximum revisions per spec before terminal failure.
pub const MAX_SPEC_REVISIONS: u32 = 2;

// ---------------------------------------------------------------------------
// Sub-state enum
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl.md#schema
// CODEGEN-BEGIN
/// Per-spec implementation sub-state.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl.md#schema
#[derive(Debug, PartialEq)]
pub enum ImplSubState {
    /// No change specs found — cannot implement.
    NoSpecs,
    /// Implement production code for a spec (first spec = begin).
    ImplementSpecCode { spec_id: String, is_first: bool },
    /// Implement with codegen path (has_json_schema/has_api_spec).
    ImplementSpecWithCodegen { spec_id: String },
    /// Gate check: run cargo build before advancing to tests phase.
    BuildCheck { spec_id: String },
    /// Implement test functions for a spec (after build passes).
    ImplementSpecTests { spec_id: String },
    /// Gate check: count #[test] in diff vs spec Unit Test section.
    TestCountCheck { spec_id: String },
    /// All specs implemented, write git diff to implementation.md.
    WriteDiff,
    /// Review implementation for a spec.
    ReviewSpec { spec_id: String },
    /// Revise implementation for a spec (fix review issues).
    ReviseSpec { spec_id: String },
    /// Spec exceeded revision limit.
    TerminalFailure { spec_id: String, revisions: u32 },
    /// All specs implemented and approved -> advance to merge.
    AdvanceToMerge,
}
// CODEGEN-END
// ---------------------------------------------------------------------------
// Sub-state resolver
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl/resolver.md#source
// CODEGEN-BEGIN
/// Resolve the current implementation sub-state from change directory.
///
/// Reads STATE.yaml, builds spec execution order, checks implementation.md
/// existence and inline reviews, then determines the next action.
///
/// Returns `(sub_state, new_current_spec_id, increment_revision_for_spec)`.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/resolver.md#source
pub fn resolve_next_impl(
    change_dir: &Path,
    _change_id: &str,
) -> Result<(ImplSubState, Option<String>, Option<String>)> {
    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();
    let current_spec_id = sm.state().current_task_id.clone();
    let spec_revisions = sm.state().task_revisions.clone();
    let impl_spec_phase = sm.state().impl_spec_phase.clone();
    drop(sm);

    let spec_paths = collect_all_spec_paths(change_dir);
    let spec_ids = build_spec_execution_order_from_paths(&spec_paths);
    let impl_path = change_dir.join("implementation.md");
    let impl_written = impl_path.exists();

    if spec_ids.is_empty() {
        return Ok((ImplSubState::NoSpecs, None, None));
    }

    let (reviewed_specs, approved_specs) = if impl_written {
        find_inline_reviews(&impl_path)
    } else {
        (HashSet::new(), HashSet::new())
    };

    let just_revised = matches!(phase, StatePhase::ChangeImplementationRevised);

    determine_sub_state(
        &spec_ids,
        &spec_paths,
        &current_spec_id,
        &spec_revisions,
        impl_written,
        &reviewed_specs,
        &approved_specs,
        change_dir,
        just_revised,
        &impl_spec_phase,
    )
}

/// Pure logic: determine sub-state from implementation context.
fn determine_sub_state(
    spec_ids: &[String],
    spec_paths: &[std::path::PathBuf],
    current_spec_id: &Option<String>,
    spec_revisions: &HashMap<String, u32>,
    impl_written: bool,
    reviewed_specs: &HashSet<String>,
    approved_specs: &HashSet<String>,
    _change_dir: &Path,
    just_revised: bool,
    impl_spec_phase: &HashMap<String, String>,
) -> Result<(ImplSubState, Option<String>, Option<String>)> {
    if !impl_written {
        // IMPLEMENTATION LOOP: implement each spec in order

        // Check if current spec has an impl_spec_phase entry (phase dispatched but not yet verified)
        if let Some(current) = current_spec_id.as_ref() {
            if let Some(phase) = impl_spec_phase.get(current.as_str()) {
                match phase.as_str() {
                    "code" => {
                        return Ok((
                            ImplSubState::BuildCheck {
                                spec_id: current.clone(),
                            },
                            None,
                            None,
                        ))
                    }
                    "tests" => {
                        return Ok((
                            ImplSubState::TestCountCheck {
                                spec_id: current.clone(),
                            },
                            None,
                            None,
                        ))
                    }
                    _ => {}
                }
            }
        }

        if current_spec_id.is_none() {
            let first = spec_ids[0].clone();
            return Ok((
                ImplSubState::ImplementSpecCode {
                    spec_id: first.clone(),
                    is_first: true,
                },
                Some(first),
                None,
            ));
        }

        let current = current_spec_id.as_ref().unwrap();
        let current_idx = spec_ids.iter().position(|s| s == current);

        match current_idx {
            Some(idx) if idx + 1 < spec_ids.len() => {
                let next = spec_ids[idx + 1].clone();
                let sub_state = if is_codegen_eligible_in_paths(spec_paths, &next) {
                    ImplSubState::ImplementSpecWithCodegen {
                        spec_id: next.clone(),
                    }
                } else {
                    ImplSubState::ImplementSpecCode {
                        spec_id: next.clone(),
                        is_first: false,
                    }
                };
                return Ok((sub_state, Some(next), None));
            }
            _ => {
                return Ok((ImplSubState::WriteDiff, None, None));
            }
        }
    }

    // REVIEW LOOP: implementation.md exists, find first non-approved spec
    for spec_id in spec_ids {
        if approved_specs.contains(spec_id) {
            continue;
        }

        let revisions = spec_revisions.get(spec_id.as_str()).copied().unwrap_or(0);

        if reviewed_specs.contains(spec_id) {
            // Just revised for this spec -> force re-review
            if just_revised && current_spec_id.as_deref() == Some(spec_id.as_str()) {
                return Ok((
                    ImplSubState::ReviewSpec {
                        spec_id: spec_id.clone(),
                    },
                    Some(spec_id.clone()),
                    None,
                ));
            }
            // Has a review but not approved -> revise or terminal failure
            if revisions >= MAX_SPEC_REVISIONS {
                return Ok((
                    ImplSubState::TerminalFailure {
                        spec_id: spec_id.clone(),
                        revisions,
                    },
                    None,
                    None,
                ));
            }
            return Ok((
                ImplSubState::ReviseSpec {
                    spec_id: spec_id.clone(),
                },
                Some(spec_id.clone()),
                Some(spec_id.clone()), // increment revision count
            ));
        }

        // No review yet -> schedule review
        return Ok((
            ImplSubState::ReviewSpec {
                spec_id: spec_id.clone(),
            },
            Some(spec_id.clone()),
            None,
        ));
    }

    Ok((ImplSubState::AdvanceToMerge, None, None))
}
// CODEGEN-END
// ---------------------------------------------------------------------------
// Spec execution order: Kahn's algorithm on refs: frontmatter
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl/execution-order.md#source
// CODEGEN-BEGIN
/// Build topological execution order from change specs in specs/ dir.
///
/// Reads each `*.md` file's `refs:` frontmatter (YAML list of spec IDs this
/// spec depends on). Applies Kahn's algorithm with BTreeSet for lexical
/// tie-breaking to produce a deterministic execution order.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/execution-order.md#source
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
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl/spec-paths.md#source
// CODEGEN-BEGIN
/// Collect all spec file paths for a change, supporting both group and legacy layouts.
///
/// - New layout: `change_dir/groups/*/specs/*.md`
/// - Legacy layout: `change_dir/specs/*.md`
///
/// If `groups/` exists and contains specs, those are returned. Otherwise falls
/// back to `specs/` for backward compatibility.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/spec-paths.md#source
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
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/spec-paths.md#source
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
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/spec-paths.md#source
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
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl/refs-frontmatter.md#source
// CODEGEN-BEGIN
/// Parse `refs:` frontmatter from a spec file (list of dependency spec IDs).
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/refs-frontmatter.md#source
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
// CODEGEN-END
// ---------------------------------------------------------------------------
// Inline review detection
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl/inline-reviews.md#source
// CODEGEN-BEGIN
/// Scan implementation.md for `## Review: {spec_id}` sections.
///
/// Returns `(reviewed, approved)` where:
/// - `reviewed` = all spec_ids that have any inline review section
/// - `approved` = spec_ids whose inline review has `verdict: APPROVED` or `PASS`
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/inline-reviews.md#source
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
// CODEGEN-END
// ---------------------------------------------------------------------------
// Codegen eligibility
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl/codegen-eligibility.md#source
// CODEGEN-BEGIN
/// Check if a change spec is eligible for structured codegen.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_impl/codegen-eligibility.md#source
pub fn is_codegen_eligible_for_spec(specs_dir: &Path, spec_id: &str) -> bool {
    let path = specs_dir.join(format!("{}.md", spec_id));
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    content.contains("has_json_schema: true") || content.contains("has_api_spec: true")
}
// CODEGEN-END
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_impl/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateManager;
    use tempfile::TempDir;

    fn write_spec(specs_dir: &Path, spec_id: &str, refs: &[&str]) {
        std::fs::create_dir_all(specs_dir).unwrap();
        let refs_yaml = if refs.is_empty() {
            String::new()
        } else {
            let items = refs
                .iter()
                .map(|r| format!("  - {}", r))
                .collect::<Vec<_>>()
                .join("\n");
            format!("refs:\n{}\n", items)
        };
        std::fs::write(
            specs_dir.join(format!("{}.md", spec_id)),
            format!(
                "---\nid: {}\ntype: spec\n{}---\n# Spec {}\n",
                spec_id, refs_yaml, spec_id
            ),
        )
        .unwrap();
    }

    fn write_impl_md(change_dir: &Path, reviews: &[(&str, &str)]) {
        let mut content = String::from(
            "---\nid: impl\ntype: change_implementation\n---\n# Implementation\n\n## Diff\n\n```diff\n+code\n```\n\n",
        );
        for (spec_id, verdict) in reviews {
            content.push_str(&format!(
                "## Review: {}\n\nverdict: {}\nsummary: test\n\n",
                spec_id, verdict
            ));
        }
        std::fs::write(change_dir.join("implementation.md"), content).unwrap();
    }

    #[test]
    fn test_kahn_ordering_respects_deps() {
        let tmp = TempDir::new().unwrap();
        let specs_dir = tmp.path();
        write_spec(specs_dir, "spec-c", &["spec-a", "spec-b"]);
        write_spec(specs_dir, "spec-a", &[]);
        write_spec(specs_dir, "spec-b", &["spec-a"]);

        let order = build_spec_execution_order(specs_dir);
        assert_eq!(order[0], "spec-a");
        assert_eq!(order[1], "spec-b");
        assert_eq!(order[2], "spec-c");
    }

    #[test]
    fn test_inline_refs_parsed() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("spec-x.md"),
            "---\nid: spec-x\nrefs: [spec-y, spec-z]\n---\n# Spec X\n",
        )
        .unwrap();
        let refs = parse_refs_frontmatter(&tmp.path().join("spec-x.md"));
        assert_eq!(refs, vec!["spec-y", "spec-z"]);
    }

    #[test]
    fn test_find_inline_reviews_approved() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("implementation.md");
        std::fs::write(
            &path,
            "# Impl\n\n## Review: spec-a\n\nverdict: APPROVED\n\n## Review: spec-b\n\nverdict: REVIEWED\n",
        )
        .unwrap();
        let (reviewed, approved) = find_inline_reviews(&path);
        assert!(reviewed.contains("spec-a"));
        assert!(reviewed.contains("spec-b"));
        assert!(approved.contains("spec-a"));
        assert!(!approved.contains("spec-b"));
    }

    #[test]
    fn test_resolve_no_specs() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.save().unwrap();
        }
        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert_eq!(sub_state, ImplSubState::NoSpecs);
    }

    #[test]
    fn test_resolve_first_spec() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);

        let (sub_state, new_id, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::ImplementSpecCode { ref spec_id, is_first: true } if spec_id == "spec-a")
        );
        assert_eq!(new_id, Some("spec-a".to_string()));
    }

    #[test]
    fn test_resolve_write_diff() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.state_mut().current_task_id = Some("spec-a".into());
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert_eq!(sub_state, ImplSubState::WriteDiff);
    }

    #[test]
    fn test_resolve_review_after_impl_written() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);
        write_impl_md(&change_dir, &[]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::ReviewSpec { ref spec_id } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_resolve_all_approved_triggers_merge() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_reviewed")
                    .unwrap();
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);
        write_impl_md(&change_dir, &[("spec-a", "APPROVED")]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert_eq!(sub_state, ImplSubState::AdvanceToMerge);
    }

    #[test]
    fn test_resolve_terminal_failure() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_reviewed")
                    .unwrap();
            sm.state_mut().task_revisions.insert("spec-a".into(), 2);
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);
        write_impl_md(&change_dir, &[("spec-a", "REJECTED")]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::TerminalFailure { ref spec_id, revisions: 2 } if spec_id == "spec-a")
        );
    }

    #[test]
    fn test_impl_spec_phase_tracking_in_state() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");

        // Set impl_spec_phase["spec-a"] = "code"
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut().phase =
                crate::tools::phase_transition::parse_phase("change_implementation_created")
                    .unwrap();
            sm.state_mut().current_task_id = Some("spec-a".into());
            sm.state_mut()
                .impl_spec_phase
                .insert("spec-a".into(), "code".into());
            sm.save().unwrap();
        }
        write_spec(&change_dir.join("specs"), "spec-a", &[]);

        let (sub_state, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state, ImplSubState::BuildCheck { ref spec_id } if spec_id == "spec-a")
        );

        // Now set to "tests"
        {
            let mut sm = StateManager::load(&change_dir).unwrap();
            sm.state_mut()
                .impl_spec_phase
                .insert("spec-a".into(), "tests".into());
            sm.save().unwrap();
        }
        let (sub_state2, _, _) = resolve_next_impl(&change_dir, "test").unwrap();
        assert!(
            matches!(sub_state2, ImplSubState::TestCountCheck { ref spec_id } if spec_id == "spec-a")
        );
    }
}
// CODEGEN-END

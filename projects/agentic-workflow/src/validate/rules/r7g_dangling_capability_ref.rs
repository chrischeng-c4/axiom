// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7g_dangling_capability_ref-rs.md#source
// CODEGEN-BEGIN
//! R7g — reject `capability_refs` entries (capability/gap/claim ids) that do
//! not resolve against the owning project's capability contract.
//!
//! Wraps `crate::cli::capability::validate_td_capability_refs_for_content`
//! against the `CapabilityDocument` for the TD's owning project: walk up
//! from `spec_path` for `.aw/config.toml`, resolve the owning project name
//! via `crate::cli::standardize::configured_project_name_for_path`, then
//! load that project's configured `cap_path`. A TD with no resolvable
//! owning project, or whose project has no capability contract, passes
//! silently — not every TD tree is capability-governed.
//!
//! Both the `.aw/config.toml` project-path table and the parsed capability
//! document are memoized per project root for the process lifetime, so a
//! whole-tree `aw td check` run reads+parses `.aw/config.toml` and each
//! project's capability contract exactly once instead of once per TD file
//! (a whole-tree scan can touch thousands of TD files, nearly all of which
//! declare `capability_refs`).

use crate::cli::capability::{
    parse_capability_document, resolve_capability_path, validate_td_capability_refs_for_content,
    CapabilityDocument,
};
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7g_dangling_capability_ref-rs.md#source
pub struct DanglingCapabilityRefRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-r7g_dangling_capability_ref-rs.md#source
impl Rule for DanglingCapabilityRefRule {
    fn id(&self) -> RuleId {
        RuleId::DanglingCapabilityRef
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        // Cheap short-circuit: the overwhelming majority of specs in a
        // whole-tree scan carry no `capability_refs:` frontmatter at all, so
        // skip project/contract resolution entirely for those.
        if !content.contains("capability_refs:") {
            return;
        }
        let Some(project_root) = find_owning_project_root(spec_path) else {
            return;
        };
        let Some(rel) = spec_path_relative(&project_root, spec_path) else {
            return;
        };
        let Some(project_name) = configured_project_name_for_path_cached(&project_root, &rel)
        else {
            return;
        };
        let Some(document) = capability_document_for(&project_root, &project_name) else {
            return;
        };
        let Ok((_, _, findings)) = validate_td_capability_refs_for_content(content, &document)
        else {
            return;
        };
        for finding in findings {
            report.push(Finding::error(
                RuleId::DanglingCapabilityRef,
                spec_path,
                format!(
                    "{} — register the work-root row in {} or fix the ref",
                    finding,
                    document.cap_path.display()
                ),
            ));
        }
    }
}

/// Walk up from `spec_path` looking for the owning `.aw/config.toml`. Same
/// shape as `rules::section_format`'s private `find_score_project_root` —
/// duplicated locally since that helper is module-private.
fn find_owning_project_root(spec_path: &Path) -> Option<PathBuf> {
    let abs = if spec_path.is_absolute() {
        spec_path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(spec_path)
    };
    let mut dir = abs.parent()?;
    loop {
        if dir.join(".aw/config.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Repo-relative, forward-slash path of `spec_path` under `project_root` —
/// the shape `configured_project_name_for_path` matches against.
fn spec_path_relative(project_root: &Path, spec_path: &Path) -> Option<String> {
    let abs = if spec_path.is_absolute() {
        spec_path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(spec_path)
    };
    let rel = abs.strip_prefix(project_root).ok()?;
    Some(rel.to_string_lossy().replace('\\', "/"))
}

/// Process-lifetime memoized `(project name, configured project path)`
/// table, keyed by project root. Reading and parsing `.aw/config.toml` is
/// the expensive step in project resolution (dozens of configured projects,
/// hundreds of lines); every TD file under the same project root shares one
/// parse instead of paying for it per file.
fn configured_project_paths_cache() -> &'static Mutex<HashMap<PathBuf, Vec<(String, String)>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Vec<(String, String)>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Read `<project_root>/.aw/config.toml` and extract `(name, path)` for
/// every `[[projects]]` entry that declares at least one non-empty
/// `[[projects.workspaces]] paths`, mirroring the candidate set
/// `crate::cli::standardize::configured_project_name_for_path` builds from
/// `ConfiguredScope`.
fn load_configured_project_paths(project_root: &Path) -> Vec<(String, String)> {
    let config_path = project_root.join(".aw/config.toml");
    let Ok(content) = std::fs::read_to_string(&config_path) else {
        return Vec::new();
    };
    let Ok(value) = toml::from_str::<toml::Value>(&content) else {
        return Vec::new();
    };
    let Some(projects) = value.get("projects").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    projects
        .iter()
        .filter_map(|project| {
            let name = project.get("name")?.as_str()?.to_string();
            let path = project.get("path")?.as_str()?.to_string();
            let has_workspace_paths = project
                .get("workspaces")
                .and_then(|v| v.as_array())
                .is_some_and(|workspaces| {
                    workspaces.iter().any(|workspace| {
                        workspace
                            .get("paths")
                            .and_then(|v| v.as_array())
                            .is_some_and(|paths| !paths.is_empty())
                    })
                });
            has_workspace_paths.then_some((name, path))
        })
        .collect()
}

/// Same string-prefix-with-boundary check as the private
/// `crate::cli::standardize::path_prefix_of` — duplicated locally since
/// that helper isn't visible outside its module.
fn path_prefix_of(prefix: &str, path: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// Cached equivalent of
/// `crate::cli::standardize::configured_project_name_for_path`: longest
/// configured project-path prefix match against `target`.
fn configured_project_name_for_path_cached(project_root: &Path, target: &str) -> Option<String> {
    let entries = {
        let mut cache = configured_project_paths_cache().lock().unwrap();
        cache
            .entry(project_root.to_path_buf())
            .or_insert_with(|| load_configured_project_paths(project_root))
            .clone()
    };
    entries
        .into_iter()
        .filter(|(_, path)| path_prefix_of(path, target))
        .max_by_key(|(_, path)| path.len())
        .map(|(name, _)| name)
}

/// Process-lifetime memoized capability-document cache, keyed by
/// `(project_root, project name)`.
fn capability_document_cache(
) -> &'static Mutex<HashMap<(PathBuf, String), Option<CapabilityDocument>>> {
    static CACHE: OnceLock<Mutex<HashMap<(PathBuf, String), Option<CapabilityDocument>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn capability_document_for(project_root: &Path, project_name: &str) -> Option<CapabilityDocument> {
    let key = (project_root.to_path_buf(), project_name.to_string());
    if let Some(hit) = capability_document_cache().lock().unwrap().get(&key) {
        return hit.clone();
    }
    let doc = load_capability_document(project_root, project_name);
    capability_document_cache()
        .lock()
        .unwrap()
        .insert(key, doc.clone());
    doc
}

fn load_capability_document(project_root: &Path, project_name: &str) -> Option<CapabilityDocument> {
    let cap_path = resolve_capability_path(project_root, project_name, None).ok()?;
    let body = std::fs::read_to_string(&cap_path).ok()?;
    parse_capability_document(&body, &cap_path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn run(spec_path: &Path, content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        DanglingCapabilityRefRule {}.check(spec_path, content, &mut r);
        r
    }

    /// Minimal field-style capability contract (same shape as
    /// `CAPABILITIES.md`): one capability with one registered Work Root row.
    /// `slugify("Registered work root")` == `registered-work-root`, and the
    /// row's `smoke` maturity auto-derives a matching claim id.
    const DEMO_CAPABILITIES: &str = r#"# demo

## Demo Capability

ID: demo-capability
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Demo capability contract for rule tests.
Gate Inventory:
- demo

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Registered work root | change | - | implemented | verified | smoke | demo |
"#;

    /// Set up `<tempdir>/.aw/config.toml` + `<tempdir>/proj/CAPABILITIES.md`
    /// (= `DEMO_CAPABILITIES`) for project `demo`, and return
    /// `(tempdir, spec_path)` where `spec_path` sits under the project's TD
    /// root (the file itself need not exist on disk — only `content`, passed
    /// directly to `check`, is read).
    fn demo_project() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::TempDir::new().unwrap();
        let aw_dir = tmp.path().join(".aw");
        fs::create_dir_all(&aw_dir).unwrap();
        fs::write(
            aw_dir.join("config.toml"),
            r#"[[projects]]
name = "demo"
path = "proj"
td_path = "proj/tech-design"
cap_path = "proj/CAPABILITIES.md"

[[projects.workspaces]]
name = "demo"
paths = ["proj/**"]
"#,
        )
        .unwrap();
        let proj_dir = tmp.path().join("proj");
        fs::create_dir_all(&proj_dir).unwrap();
        fs::write(proj_dir.join("CAPABILITIES.md"), DEMO_CAPABILITIES).unwrap();
        let spec_path = proj_dir.join("tech-design/some_change.md");
        (tmp, spec_path)
    }

    fn td_with_refs(gap: &str, claim: &str) -> String {
        format!(
            r#"---
id: demo-td
capability_refs:
  - id: demo-capability
    role: primary
    gap: {gap}
    claim: {claim}
    coverage: full
    rationale: "demo"
---

# Demo TD

## Overview
<!-- type: overview lang: markdown -->

Demo.
"#
        )
    }

    #[test]
    fn known_refs_pass_cleanly() {
        let (_tmp, spec_path) = demo_project();
        let content = td_with_refs("registered-work-root", "registered-work-root");
        let report = run(&spec_path, &content);
        assert!(report.is_empty(), "expected no findings, got {:?}", report);
    }

    #[test]
    fn dangling_gap_id_flagged() {
        let (_tmp, spec_path) = demo_project();
        let content = td_with_refs("registered-work-root", "registered-work-root")
            .replace("gap: registered-work-root", "gap: nonexistent-work-root");
        let report = run(&spec_path, &content);
        assert!(!report.is_empty());
        let finding = &report.findings[0];
        assert_eq!(finding.rule, RuleId::DanglingCapabilityRef);
        assert!(
            finding.message.contains("nonexistent-work-root"),
            "message must name the unknown id: {}",
            finding.message
        );
        assert!(
            finding.message.contains("demo-capability"),
            "message must name the capability checked: {}",
            finding.message
        );
        assert!(
            finding.message.contains("CAPABILITIES.md"),
            "message must point at the remediation cap_path: {}",
            finding.message
        );
    }

    #[test]
    fn dangling_claim_id_flagged() {
        let (_tmp, spec_path) = demo_project();
        let content = td_with_refs("registered-work-root", "registered-work-root").replace(
            "claim: registered-work-root",
            "claim: nonexistent-work-root",
        );
        let report = run(&spec_path, &content);
        assert!(!report.is_empty());
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("nonexistent-work-root")));
    }

    #[test]
    fn td_outside_any_project_passes_silently() {
        let tmp = tempfile::TempDir::new().unwrap();
        // No `.aw/config.toml` anywhere above this path — not every TD tree
        // is capability-governed.
        let spec_path = tmp.path().join("loose/some_change.md");
        let content = td_with_refs("anything", "anything");
        let report = run(&spec_path, &content);
        assert!(report.is_empty());
    }

    #[test]
    fn content_without_capability_refs_is_never_resolved() {
        let (_tmp, spec_path) = demo_project();
        let content = "# Demo TD\n\nNo frontmatter at all.\n";
        let report = run(&spec_path, &content);
        assert!(report.is_empty());
    }

    /// Minimal replica of the `td_no_merge_test.md` incident shape: a
    /// primary `capability_refs` entry whose `gap` and `claim` are two
    /// different slugs (an authoring-time divergence) and neither is
    /// registered in the capability contract's Work Root table. Before
    /// #852, `aw td check` reported 0 findings on this shape; the rule now
    /// catches it.
    #[test]
    fn replica_td_no_merge_shape_two_different_slugs_caught() {
        let (_tmp, spec_path) = demo_project();
        let content = td_with_refs("remove-td-merge-command", "remove-tdmerge-command");
        let report = run(&spec_path, &content);
        assert!(!report.is_empty());
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("remove-td-merge-command")));
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("remove-tdmerge-command")));
    }

    /// Minimal replica of the `chain_liveness_test.md` incident shape: a
    /// `capability_refs` entry naming a gap/claim id that has no
    /// corresponding Work Root row yet (registered later, after the fix).
    #[test]
    fn replica_chain_liveness_shape_unregistered_id_caught() {
        let (_tmp, spec_path) = demo_project();
        let content = td_with_refs("chain-liveness-proof", "chain-liveness-proof");
        let report = run(&spec_path, &content);
        assert!(!report.is_empty());
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("chain-liveness-proof")));
    }

    #[test]
    fn unknown_capability_id_flagged() {
        let (_tmp, spec_path) = demo_project();
        let content = td_with_refs("registered-work-root", "registered-work-root")
            .replace("id: demo-capability", "id: no-such-capability");
        let report = run(&spec_path, &content);
        assert!(!report.is_empty());
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("no-such-capability")));
    }
}

// CODEGEN-END

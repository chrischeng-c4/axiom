//! Guard sanctioned-path resolver (#1428, parent #1269).
//!
//! Pure, offline answer to "is path P sanctioned for direct edit in project
//! X right now?" — a WI's TD may declare a target `impl_mode: hand-written`
//! in its `## Changes` block, which means `aw td gen` intentionally SKIPs
//! that file and an agent is expected to hand-edit it directly while the WI
//! sits in a handwrite-eligible phase. This module only *answers* that
//! question; it does not change `aw guard pretool` behavior (that wiring is
//! #1429).
//!
//! ## Data sources (no new parsing formats)
//!
//! - **Active WI phase**: read via the project's already-configured issue
//!   backend, chosen the same way `aw wi` picks one
//!   (`crate::issues::resolve_default_backend`, itself just an offline
//!   `aw.toml` read) — but this module never lets that backend touch the
//!   network. For a `local`-platform project that is the durable local
//!   issue store (`crate::issues::local_backend`); for the common
//!   `github`/`gitlab` case it is the **read-through cache**
//!   (`crate::issues::remote_read_cache_backend`, under
//!   `/tmp/aw/issues/<host>-<repo>/<kind>`) that every `aw wi
//!   list`/`show`/`create`/`update` already writes through. Staleness
//!   window: as fresh as the last `aw wi`/`aw td` command that touched this
//!   project's issues in the current environment — in the worst case
//!   (a long-idle checkout with no prior `aw wi` traffic) the cache can be
//!   empty and every path resolves unsanctioned (fail-closed, not a false
//!   allow); in the common case (mid-lifecycle-loop) it reflects the WI's
//!   just-written phase because the lifecycle command that advanced the
//!   phase is what populated/refreshed the cache.
//! - **TD spec path**: `Issue.implements` (first `.md` entry) — the same
//!   resolution `cb_fill::derive_spec_path_from_implements` uses.
//! - **Hand-written declared paths**: the TD's `## Changes` YAML block,
//!   `impl_mode: hand-written` entries — the same block shape
//!   `cb_fill::extract_change_paths_from_spec` and
//!   `td::validate_section_implementation_edges` already parse (this module
//!   re-parses it narrowly to also carry `impl_mode` through, which the
//!   path-only extractor drops).
//!
//! [`sanctioned_edit_paths_from_issues`] is the pure core (no I/O beyond
//! reading TD files already named by the caller's issue list) and is what
//! the unit tests below exercise directly, per #1428 AC3: deterministic,
//! no git/tracker network access. [`sanctioned_edit_paths`] and
//! [`is_sanctioned`] are the offline-backend-resolving convenience
//! wrappers layered on top.
//!
//! Not SPEC-MANAGED — a brand-new hand-authored module, same as
//! `src/cli/drift.rs`.

use crate::issues::types::td_phase;
use crate::issues::{
    local_backend, remote_read_cache_backend, resolve_default_backend, Issue, IssueBackend,
    IssueFilter, IssueState,
};
use crate::services::project_registry;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Phases in which a WI's TD-declared hand-written paths are open for
/// direct edit: post-gen, pre-terminal-code-check. `cb_genned` covers the
/// canonical and legacy (`td_gen_coded`, normalized on read — see
/// `td_phase::normalize`) post-gen phase; `cb_filled` covers post-fill;
/// `cb_fill_in_progress` is the fill loop's own transient in-flight phase
/// (see `crate::cli::run::wi_change_lifecycle_step` and
/// `crate::cli::capability::lifecycle_action_for_work_item`, which already
/// treat it as `cb_genned`-equivalent for routing — `td_phase::normalize`
/// passes it through unchanged, so it is listed explicitly here rather than
/// folded into `td_phase::is_terminal_code_checkable`, which does not
/// include it).
const ELIGIBLE_PHASES: &[&str] = &[
    td_phase::CB_GENNED,
    "cb_fill_in_progress",
    td_phase::CB_FILLED,
];

/// Why a path is sanctioned for direct edit right now. `#1429` prints this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanctionReason {
    /// The active WI's tracker id (bare, e.g. `"1428"`) or local slug when
    /// no tracker id is assigned yet.
    pub wi_id: String,
    /// The WI's TD spec path, project-root-relative (as recorded in
    /// `Issue.implements`).
    pub td_path: String,
    /// The WI's normalized phase at resolution time (`td_phase::normalize`
    /// output, or the `cb_fill_in_progress` transient).
    pub phase: String,
}

/// Pure core: given a project root and a caller-supplied issue list, return
/// every TD-declared `impl_mode: hand-written` path (project-root-relative)
/// that is currently sanctioned for direct edit, keyed by that relative
/// path.
///
/// Single pass over `issues`; for each eligible one, one TD file read. No
/// repo-wide scan. Fail-closed: any issue that is closed, phase-ineligible,
/// has no resolvable TD ref, or whose TD is missing/malformed contributes
/// nothing (skipped, never a panic or an `Err`).
pub fn sanctioned_edit_paths_from_issues(
    project_root: &Path,
    issues: &[Issue],
) -> HashMap<PathBuf, SanctionReason> {
    let mut out: HashMap<PathBuf, SanctionReason> = HashMap::new();

    for issue in issues {
        if issue.state != IssueState::Open {
            continue;
        }
        let Some(raw_phase) = issue.phase.as_deref() else {
            continue;
        };
        let normalized = td_phase::normalize(raw_phase);
        if !ELIGIBLE_PHASES.contains(&normalized) {
            continue;
        }
        let Some(td_rel) = spec_path_from_implements(issue) else {
            continue;
        };
        let Ok(spec_content) = std::fs::read_to_string(project_root.join(&td_rel)) else {
            continue;
        };
        let wi_id = wi_id_for(issue);

        for change_path in hand_written_change_paths(&spec_content) {
            out.entry(PathBuf::from(change_path))
                .or_insert_with(|| SanctionReason {
                    wi_id: wi_id.clone(),
                    td_path: td_rel.clone(),
                    phase: normalized.to_string(),
                });
        }
    }

    out
}

/// Full resolver: resolve `project`'s root + offline issue source, list its
/// open WIs, and delegate to [`sanctioned_edit_paths_from_issues`]. Never
/// makes a network call — see the module doc for the offline data source
/// and its staleness window. Fails closed (returns an empty map) when the
/// project or its backend config cannot be resolved; propagates only
/// unexpected I/O errors from the offline backend's own `list()`.
pub async fn sanctioned_edit_paths(
    repo_root: &Path,
    project: &str,
) -> Result<HashMap<PathBuf, SanctionReason>> {
    let Ok(row) = project_registry::resolve_project_config_row(repo_root, project) else {
        return Ok(HashMap::new());
    };
    let project_root = repo_root.join(&row.path);

    let Some(backend) = offline_issue_backend(&project_root) else {
        return Ok(HashMap::new());
    };

    let filter = IssueFilter {
        state: Some(IssueState::Open),
        ..Default::default()
    };
    let issues = match backend.list(&filter).await {
        Ok(issues) => issues,
        Err(_) => return Ok(HashMap::new()),
    };

    Ok(sanctioned_edit_paths_from_issues(&project_root, &issues))
}

/// Convenience: is `path` sanctioned for direct edit in `project` right
/// now? `path` may be absolute (under `repo_root`/the project root) or
/// already project-root-relative.
pub async fn is_sanctioned(
    repo_root: &Path,
    project: &str,
    path: &Path,
) -> Result<Option<SanctionReason>> {
    let Ok(row) = project_registry::resolve_project_config_row(repo_root, project) else {
        return Ok(None);
    };
    let project_root = repo_root.join(&row.path);
    let map = sanctioned_edit_paths(repo_root, project).await?;
    let key = relative_lookup_key(&project_root, path);
    Ok(map.get(&key).cloned())
}

/// Pick the offline-only issue source for `project_root`'s configured
/// backend: the durable local store for a `local`-platform project, else
/// the read-through cache for `github`/`gitlab` (never the live backend
/// itself — see the module doc). `None` when the backend config cannot be
/// resolved (missing/invalid `aw.toml`) — callers fail closed.
fn offline_issue_backend(project_root: &Path) -> Option<Box<dyn IssueBackend>> {
    let (kind, repo, host) = resolve_default_backend(project_root).ok()?;
    if kind == "local" {
        Some(Box::new(local_backend(project_root)))
    } else {
        Some(Box::new(remote_read_cache_backend(
            &kind,
            repo.as_deref(),
            host.as_deref(),
        )))
    }
}

/// Resolve a project-root-relative TD spec path from `Issue.implements`
/// (best effort) — same resolution `cb_fill::derive_spec_path_from_implements`
/// uses.
fn spec_path_from_implements(issue: &Issue) -> Option<String> {
    issue
        .implements
        .iter()
        .find(|s| s.ends_with(".md"))
        .cloned()
}

/// Bare WI id (no `#`) for embedding in a future `aw td ...` command
/// string, falling back to the local slug when no tracker id is assigned.
fn wi_id_for(issue: &Issue) -> String {
    issue
        .github_id
        .or(issue.gitlab_id)
        .map(|id| id.to_string())
        .unwrap_or_else(|| issue.slug.clone())
}

/// Parse a TD's `## Changes` YAML fence and return the project-root-relative
/// paths of every `impl_mode: hand-written` (or `hand_written`) entry.
/// Same block shape as `cb_fill::extract_change_paths_from_spec` /
/// `td::validate_section_implementation_edges` (`## Changes` heading, first
/// ```` ```yaml ```` fence, a `changes:`/`files:` sequence of `path:`/`file:`
/// + `impl_mode:` maps); reimplemented narrowly here because the existing
/// extractor drops `impl_mode`.
fn hand_written_change_paths(spec_content: &str) -> Vec<String> {
    let mut in_changes = false;
    let mut in_yaml = false;
    let mut yaml_content = String::new();
    let mut paths = Vec::new();

    for line in spec_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") && trimmed.to_lowercase().contains("changes") {
            in_changes = true;
            continue;
        }
        if in_changes && trimmed.starts_with("## ") {
            break;
        }
        if in_changes && trimmed == "```yaml" {
            in_yaml = true;
            yaml_content.clear();
            continue;
        }
        if in_yaml && trimmed == "```" {
            append_hand_written_paths_from_yaml(&yaml_content, &mut paths);
            in_yaml = false;
            continue;
        }
        if in_yaml {
            yaml_content.push_str(line);
            yaml_content.push('\n');
        }
    }

    paths.sort();
    paths.dedup();
    paths
}

fn append_hand_written_paths_from_yaml(yaml_content: &str, paths: &mut Vec<String>) {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(yaml_content) else {
        return;
    };
    let entries = value.get("changes").or_else(|| value.get("files"));
    let Some(serde_yaml::Value::Sequence(entries)) = entries else {
        return;
    };
    for entry in entries {
        let impl_mode = entry
            .get("impl_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("codegen");
        if !matches!(impl_mode, "hand-written" | "hand_written") {
            continue;
        }
        let path = entry
            .get("path")
            .or_else(|| entry.get("file"))
            .and_then(|v| v.as_str());
        if let Some(path) = path {
            let path = normalize_rel_path(path);
            if !path.is_empty() {
                paths.push(path);
            }
        }
    }
}

fn normalize_rel_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

/// Build the map lookup key for `path` relative to `project_root`: strip
/// the project-root prefix when `path` is absolute, otherwise use it as-is
/// (already assumed project-root-relative, matching
/// [`sanctioned_edit_paths_from_issues`]'s key convention).
fn relative_lookup_key(project_root: &Path, path: &Path) -> PathBuf {
    let rel = if path.is_absolute() {
        path.strip_prefix(project_root).unwrap_or(path)
    } else {
        path
    };
    PathBuf::from(normalize_rel_path(&rel.to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::IssueType;

    fn open_issue(github_id: u64, slug: &str, phase: &str, implements: Vec<&str>) -> Issue {
        Issue {
            issue_type: IssueType::Enhancement,
            title: format!("wi {github_id}"),
            state: IssueState::Open,
            id: None,
            github_id: Some(github_id),
            gitlab_id: None,
            url: None,
            author: None,
            labels: Vec::new(),
            created_at: None,
            updated_at: None,
            slug: slug.to_string(),
            body: String::new(),
            related: Vec::new(),
            implements: implements.into_iter().map(str::to_string).collect(),
            phase: Some(phase.to_string()),
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    fn write_td(dir: &std::path::Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    const HAND_WRITTEN_TD: &str = "\
# TD

## Changes
```yaml
changes:
  - path: src/bundler/dts.rs
    section: source
    impl_mode: hand-written
  - path: src/bundler/gen.rs
    section: source
    impl_mode: codegen
```
";

    #[test]
    fn sanctioned_path_at_eligible_phase_names_wi_td_phase() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let issues = vec![open_issue(
            937,
            "937",
            td_phase::CB_GENNED,
            vec!["tech-design/dts.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);

        let reason = map
            .get(&PathBuf::from("src/bundler/dts.rs"))
            .expect("hand-written path must be sanctioned at cb_genned");
        assert_eq!(reason.wi_id, "937");
        assert_eq!(reason.td_path, "tech-design/dts.md");
        assert_eq!(reason.phase, td_phase::CB_GENNED);
    }

    #[test]
    fn cb_fill_in_progress_is_also_eligible() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let issues = vec![open_issue(
            937,
            "937",
            "cb_fill_in_progress",
            vec!["tech-design/dts.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        assert!(map.contains_key(&PathBuf::from("src/bundler/dts.rs")));
    }

    #[test]
    fn same_path_at_td_created_is_not_sanctioned() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let issues = vec![open_issue(
            937,
            "937",
            td_phase::TD_CREATED,
            vec!["tech-design/dts.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        assert!(map.is_empty());
    }

    #[test]
    fn post_terminal_td_merged_is_not_sanctioned() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let issues = vec![open_issue(
            937,
            "937",
            td_phase::TD_MERGED,
            vec!["tech-design/dts.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        assert!(map.is_empty());
    }

    #[test]
    fn closed_issue_at_eligible_phase_is_not_sanctioned() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let mut issue = open_issue(937, "937", td_phase::CB_GENNED, vec!["tech-design/dts.md"]);
        issue.state = IssueState::Closed;

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &[issue]);
        assert!(map.is_empty());
    }

    #[test]
    fn undeclared_sibling_path_is_not_sanctioned() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let issues = vec![open_issue(
            937,
            "937",
            td_phase::CB_GENNED,
            vec!["tech-design/dts.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        assert!(!map.contains_key(&PathBuf::from("src/bundler/sibling.rs")));
        // codegen-mode sibling entry in the same changes block is also excluded.
        assert!(!map.contains_key(&PathBuf::from("src/bundler/gen.rs")));
    }

    #[test]
    fn missing_td_is_not_sanctioned_and_does_not_panic() {
        let tmp = tempfile::tempdir().unwrap();
        // No TD written at all.
        let issues = vec![open_issue(
            937,
            "937",
            td_phase::CB_GENNED,
            vec!["tech-design/missing.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        assert!(map.is_empty());
    }

    #[test]
    fn malformed_td_yaml_is_not_sanctioned_and_does_not_panic() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(
            tmp.path(),
            "tech-design/dts.md",
            "# TD\n\n## Changes\n```yaml\nchanges: [not: [valid\n```\n",
        );
        let issues = vec![open_issue(
            937,
            "937",
            td_phase::CB_GENNED,
            vec!["tech-design/dts.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        assert!(map.is_empty());
    }

    #[test]
    fn no_implements_ref_is_not_sanctioned() {
        let tmp = tempfile::tempdir().unwrap();
        let issues = vec![open_issue(937, "937", td_phase::CB_GENNED, vec![])];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        assert!(map.is_empty());
    }

    #[test]
    fn no_phase_is_not_sanctioned() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let mut issue = open_issue(937, "937", td_phase::CB_GENNED, vec!["tech-design/dts.md"]);
        issue.phase = None;

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &[issue]);
        assert!(map.is_empty());
    }

    #[test]
    fn relative_lookup_key_strips_project_root_prefix() {
        let project_root = Path::new("/repo/apps/jet");
        let abs = Path::new("/repo/apps/jet/src/bundler/dts.rs");
        assert_eq!(
            relative_lookup_key(project_root, abs),
            PathBuf::from("src/bundler/dts.rs")
        );
        assert_eq!(
            relative_lookup_key(project_root, Path::new("src/bundler/dts.rs")),
            PathBuf::from("src/bundler/dts.rs")
        );
    }

    #[test]
    fn legacy_td_gen_coded_phase_normalizes_to_eligible() {
        let tmp = tempfile::tempdir().unwrap();
        write_td(tmp.path(), "tech-design/dts.md", HAND_WRITTEN_TD);
        let issues = vec![open_issue(
            937,
            "937",
            td_phase::LEGACY_TD_GEN_CODED,
            vec!["tech-design/dts.md"],
        )];

        let map = sanctioned_edit_paths_from_issues(tmp.path(), &issues);
        let reason = map
            .get(&PathBuf::from("src/bundler/dts.rs"))
            .expect("legacy td_gen_coded normalizes to cb_genned, which is eligible");
        assert_eq!(reason.phase, td_phase::CB_GENNED);
    }
}

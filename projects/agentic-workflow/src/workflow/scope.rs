// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
// CODEGEN-BEGIN
//! Scope extraction and cascade for explore phases.
//!
//! Reads issue labels and clarifications to determine affected crates/modules,
//! then formats scoped instructions for each explore phase prompt.

use crate::models::SddConfig;
use crate::parser::frontmatter;
use crate::shared::workspace;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Scope information extracted from issue labels and clarifications.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#schema
#[derive(Debug, Clone, Default)]
pub struct ScopeInfo {
    /// Affected crate names.
    pub affected_crates: Vec<String>,
    /// Spec groups derived from crate names.
    pub spec_groups: Vec<String>,
    /// Specific paths mentioned in clarifications.
    pub affected_paths: Vec<String>,
    /// Keywords extracted from scope answers.
    pub keywords: Vec<String>,
    /// True when user answered unknown or no scope info available.
    pub is_unknown: bool,
    /// Source of scope info.
    pub source: String,
}
/// Extract scope from issue labels and clarifications.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
pub fn extract_scope(change_dir: &Path) -> ScopeInfo {
    let mut info = ScopeInfo::default();
    let mut from_issues = false;
    let mut from_clarifications = false;

    // 1. Read issue_*.md files for crate:* labels (from issues/ subdirectory)
    let issues_dir = change_dir.join("issues");
    if let Ok(entries) = std::fs::read_dir(&issues_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("issue_") && name_str.ends_with(".md") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Some(crates) = extract_crate_labels_from_content(&content) {
                        for c in crates {
                            if !info.affected_crates.contains(&c) {
                                info.affected_crates.push(c);
                                from_issues = true;
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Read pre_clarifications.md for scope answers (fallback: context_clarifications.md)
    let mut clarifications_path = change_dir.join("pre_clarifications.md");
    if !clarifications_path.exists() {
        let legacy = change_dir.join("context_clarifications.md");
        if legacy.exists() {
            clarifications_path = legacy;
        }
    }
    if clarifications_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&clarifications_path) {
            let (scope_crates, scope_paths, scope_keywords, is_unknown) =
                extract_scope_from_clarifications(&content);

            if is_unknown {
                info.is_unknown = true;
            }

            for c in scope_crates {
                if !info.affected_crates.contains(&c) {
                    info.affected_crates.push(c);
                    from_clarifications = true;
                }
            }
            for p in scope_paths {
                if !info.affected_paths.contains(&p) {
                    info.affected_paths.push(p);
                    from_clarifications = true;
                }
            }
            for k in scope_keywords {
                if !info.keywords.contains(&k) {
                    info.keywords.push(k);
                }
            }
        }
    }

    // 3. Extract spec groups from requirements text in groups/*/requirements.md
    let groups_dir = change_dir.join("groups");
    if let Ok(entries) = std::fs::read_dir(&groups_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let req_path = entry.path().join("requirements.md");
                if let Ok(content) = std::fs::read_to_string(&req_path) {
                    for name in extract_scope_from_requirements(&content) {
                        if !info.affected_crates.contains(&name) {
                            info.affected_crates.push(name);
                            from_clarifications = true;
                        }
                    }
                }
            }
        }
    }

    // Derive spec_groups from affected_crates
    info.spec_groups = info.affected_crates.clone();

    info.source = match (from_issues, from_clarifications) {
        (true, true) => "both".to_string(),
        (true, false) => "issues".to_string(),
        (false, true) => "clarifications".to_string(),
        (false, false) => "none".to_string(),
    };

    info
}

/// Resolve the directory for a spec group using config-driven or fallback probes.
///
/// If `scopes` contains `group`, returns `specs_base / scopes[group] / group` when
/// that path exists on disk; returns `None` if the configured path is absent (no
/// fallback for explicitly-configured groups — see Scenario: Config-scoped entry
/// does not exist on disk).
///
/// When `group` is not in `scopes`, falls back to the classic probe order:
/// `crates/{group}` → `projects/{group}` → `{group}` (backward compatibility).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
pub fn resolve_spec_dir(
    group: &str,
    specs_base: &Path,
    scopes: &HashMap<String, String>,
) -> Option<PathBuf> {
    if let Some(subdir) = scopes.get(group) {
        // Config-driven resolution: honor the configured subdir; no fallback.
        let candidate = specs_base.join(subdir).join(group);
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    } else {
        // Fallback probe: crates/ → projects/ → root
        let crates_path = specs_base.join("crates").join(group);
        if crates_path.exists() {
            return Some(crates_path);
        }
        let projects_path = specs_base.join("projects").join(group);
        if projects_path.exists() {
            return Some(projects_path);
        }
        let root_path = specs_base.join(group);
        if root_path.exists() {
            return Some(root_path);
        }
        None
    }
}

/// Resolve a spec group against registered project `td_path` values first,
/// then fall back to legacy specs scopes and classic probes.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
pub fn resolve_spec_dir_for_root(
    group: &str,
    project_root: &Path,
    specs_base: &Path,
    scopes: &HashMap<String, String>,
) -> Option<PathBuf> {
    if group == "agentic-workflow" {
        let candidate = project_root.join("projects/agentic-workflow/tech-design");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    if let Ok(resolved) =
        crate::services::project_registry::resolve_td_root_from_config(project_root, group)
    {
        let candidate = PathBuf::from(resolved.root);
        if candidate.exists() {
            return Some(candidate);
        }
        return None;
    }

    resolve_spec_dir(group, specs_base, scopes)
}

/// Pre-filter main specs by scope groups and return a markdown listing.
///
/// Reads `.aw/tech-design/{group}/` for each group in `spec_groups` and returns
/// a concatenated listing. Falls back to a short message if no specs found.
///
/// `config` is optional. When provided, `config.specs.scopes` drives directory
/// resolution via [`resolve_spec_dir`]. Existing callers may pass `None` to
/// retain the original `crates/ → projects/ → root` fallback behaviour.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
pub fn pre_filter_specs(
    spec_groups: &[String],
    project_root: &Path,
    config: Option<&SddConfig>,
) -> String {
    if spec_groups.is_empty() {
        return String::new();
    }

    let specs_dir = workspace::tech_design_path(project_root);
    if !any_tech_design_root_exists(project_root, &specs_dir) {
        return String::new();
    }

    let empty_scopes: HashMap<String, String> = HashMap::new();
    let scopes = config.map(|c| &c.specs.scopes).unwrap_or(&empty_scopes);

    let mut output = String::new();
    let mut found_any = false;

    for group in spec_groups {
        let group_dir = match resolve_spec_dir_for_root(group, project_root, &specs_dir, scopes) {
            Some(d) => d,
            None => continue,
        };
        // Walk the resolved directory recursively to collect every .md file at any depth.
        let mut spec_paths: Vec<String> = Vec::new();
        let rel_base = if group_dir.starts_with(&specs_dir) {
            specs_dir.as_path()
        } else {
            group_dir.as_path()
        };
        collect_md_files_recursive(&group_dir, rel_base, &mut spec_paths);
        spec_paths.sort();
        if !spec_paths.is_empty() {
            found_any = true;
            output.push_str(&format!("### {}\n", group));
            for rel_path in &spec_paths {
                output.push_str(&format!("- `read_path:specs/{}`\n", rel_path));
            }
            output.push('\n');
        }
    }

    if !found_any {
        return String::new();
    }

    output
}

/// Build an ASCII directory tree for spec directories associated with the given spec groups.
///
/// Renders the same format as the `tree` CLI tool (├── / └── / │   prefixes).
/// Returns an empty string if no spec directories are found.
///
/// Used by `build_create_prompt` to populate the `{{spec_dir_tree}}` template variable
/// in the reference context CREATE prompt.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
pub fn build_spec_dir_tree(
    spec_groups: &[String],
    project_root: &Path,
    config: Option<&SddConfig>,
) -> String {
    if spec_groups.is_empty() {
        return String::new();
    }

    let specs_dir = workspace::tech_design_path(project_root);
    if !any_tech_design_root_exists(project_root, &specs_dir) {
        return String::new();
    }

    let empty_scopes: HashMap<String, String> = HashMap::new();
    let scopes = config.map(|c| &c.specs.scopes).unwrap_or(&empty_scopes);

    let mut output = String::new();
    let mut found_any = false;

    for group in spec_groups {
        let group_dir = match resolve_spec_dir_for_root(group, project_root, &specs_dir, scopes) {
            Some(d) => d,
            None => continue,
        };
        if !group_dir.exists() {
            continue;
        }

        found_any = true;
        output.push_str(&format!("{}\n", group));
        render_tree_recursive(&group_dir, &mut output, "");
        output.push('\n');
    }

    if !found_any {
        return String::new();
    }

    output
}

fn any_tech_design_root_exists(project_root: &Path, specs_dir: &Path) -> bool {
    specs_dir.exists()
        || project_root
            .join("projects/agentic-workflow/tech-design")
            .exists()
        || workspace::project_tech_design_paths(project_root)
            .iter()
            .any(|(_, path)| path.exists())
}

/// Recursively render directory entries as an ASCII tree (tree CLI format).
fn render_tree_recursive(dir: &Path, output: &mut String, prefix: &str) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());

    let count = sorted.len();
    for (i, entry) in sorted.into_iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        output.push_str(&format!("{}{}{}\n", prefix, connector, name_str));

        if entry.path().is_dir() {
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            render_tree_recursive(&entry.path(), output, &child_prefix);
        }
    }
}

/// Recursively collect paths of all `.md` files under `dir`.
///
/// Returns paths relative to `specs_base` (the `.aw/tech-design/` directory) using
/// forward slashes for cross-platform consistency. Used by [`pre_filter_specs`]
/// to enumerate spec files at any nesting depth (depth-first traversal).
fn collect_md_files_recursive(dir: &Path, specs_base: &Path, results: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files_recursive(&path, specs_base, results);
        } else if path.extension().map_or(false, |ext| ext == "md") {
            if let Ok(rel) = path.strip_prefix(specs_base) {
                results.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// Suggest question topics based on description (moved from helpers.rs).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
pub fn suggest_topics(description: &str) -> Vec<&'static str> {
    let desc_lower = description.to_lowercase();
    let mut topics = Vec::new();

    if desc_lower.contains("auth") || desc_lower.contains("login") || desc_lower.contains("oauth") {
        topics.push("Authentication method (OAuth, JWT, session)");
        topics.push("Provider selection (Google, GitHub, etc.)");
    }
    if desc_lower.contains("api") || desc_lower.contains("endpoint") {
        topics.push("API versioning strategy");
        topics.push("Response format (JSON, protobuf, etc.)");
    }
    if desc_lower.contains("database")
        || desc_lower.contains("storage")
        || desc_lower.contains("persist")
    {
        topics.push("Database choice (SQL, NoSQL, etc.)");
        topics.push("Data migration strategy");
    }
    if desc_lower.contains("ui")
        || desc_lower.contains("frontend")
        || desc_lower.contains("component")
    {
        topics.push("UI framework preference");
        topics.push("Styling approach (CSS, Tailwind, etc.)");
    }
    if desc_lower.contains("performance")
        || desc_lower.contains("optimize")
        || desc_lower.contains("cache")
    {
        topics.push("Caching strategy");
        topics.push("Performance targets");
    }
    if topics.is_empty() {
        topics.push("Implementation approach");
        topics.push("Scope boundaries");
        topics.push("Integration points");
    }

    // Always suggest scope topic
    topics.push("Affected crates/modules");
    topics
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Extract crate/module references from requirements text.
///
/// Matches patterns like `crates/{name}/`, `cclab-{name}`, `crate:{name}`, and
/// the no-prefix arsenal crates (`agentic-workflow`). Returns deduplicated crate names.
fn extract_scope_from_requirements(requirements: &str) -> Vec<String> {
    let mut names = Vec::new();

    // Arsenal crates that don't carry the `cclab-` prefix.
    const BARE_CRATES: &[&str] = &["agentic-workflow"];

    for word in requirements.split_whitespace() {
        let w =
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != ':' && c != '/');

        // Match `crates/cclab-{name}/...`
        if let Some(rest) = w.strip_prefix("crates/cclab-") {
            if let Some(name) = rest.split('/').next() {
                let name = name.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                if !name.is_empty() {
                    let crate_name = format!("cclab-{}", name);
                    if !names.contains(&crate_name) {
                        names.push(crate_name);
                    }
                }
            }
        }
        // Match `crates/{bare-crate}/...` or `projects/{bare-crate}/...`.
        else if let Some(rest) = w.strip_prefix("crates/") {
            if let Some(name) = rest.split('/').next() {
                let name = name.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                if BARE_CRATES.contains(&name) && !names.contains(&name.to_string()) {
                    names.push(name.to_string());
                }
            }
        } else if let Some(rest) = w.strip_prefix("projects/") {
            if let Some(name) = rest.split('/').next() {
                let name = name.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                if BARE_CRATES.contains(&name) && !names.contains(&name.to_string()) {
                    names.push(name.to_string());
                }
            }
        }
        // Match `crate:{name}`
        else if let Some(name) = w.strip_prefix("crate:") {
            let name = name.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if !name.is_empty() {
                let crate_name = match name {
                    "genesis" | "aurora" => "agentic-workflow".to_string(),
                    n if BARE_CRATES.contains(&n) => n.to_string(),
                    _ => format!("cclab-{}", name),
                };
                if !names.contains(&crate_name) {
                    names.push(crate_name);
                }
            }
        }
        // Match standalone `cclab-{name}` (6+ chars after prefix)
        else if w.starts_with("cclab-") && w.len() > 6 {
            let crate_name = w.split('/').next().unwrap_or(w).to_string();
            let crate_name = crate_name
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();
            if !crate_name.is_empty() && !names.contains(&crate_name) {
                names.push(crate_name);
            }
        }
        // Match standalone bare arsenal crate names (e.g. `agentic-workflow`)
        else if BARE_CRATES.contains(&w) && !names.contains(&w.to_string()) {
            names.push(w.to_string());
        }
    }

    names
}

/// Extract `crate:*` labels from issue file content → crate names.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/scope.md#source
pub(crate) fn extract_crate_labels_from_content(content: &str) -> Option<Vec<String>> {
    const BARE_CRATES: &[&str] = &["agentic-workflow"];
    let fm = frontmatter::parse_frontmatter_value(content).ok()?;
    let labels = fm.get("labels")?.as_sequence()?;
    let crates: Vec<String> = labels
        .iter()
        .filter_map(|v| v.as_str())
        .filter_map(|s| s.strip_prefix("crate:"))
        .map(|name| match name {
            "genesis" | "aurora" => "agentic-workflow".to_string(),
            n if BARE_CRATES.contains(&n) => n.to_string(),
            other => format!("cclab-{}", other),
        })
        .collect();
    if crates.is_empty() {
        None
    } else {
        Some(crates)
    }
}

/// Parse pre_clarifications.md for scope-related Q&A answers.
///
/// Returns (crates, paths, keywords, is_unknown).
fn extract_scope_from_clarifications(
    content: &str,
) -> (Vec<String>, Vec<String>, Vec<String>, bool) {
    const BARE_CRATES: &[&str] = &["agentic-workflow"];
    let mut crates = Vec::new();
    let mut paths = Vec::new();
    let mut keywords = Vec::new();
    let mut is_unknown = false;

    let content_lower = content.to_lowercase();

    // Check for scope-related Q&A blocks
    let in_scope_section = content_lower.contains("scope")
        || content_lower.contains("module")
        || content_lower.contains("crate")
        || content_lower.contains("affected");

    let is_crate_ref =
        |w: &str| -> bool { (w.starts_with("cclab-") && w.len() > 6) || BARE_CRATES.contains(&w) };

    if !in_scope_section {
        // Fallback: scan whole content for cclab-* patterns + bare arsenal crates
        for word in content.split_whitespace() {
            let w = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if is_crate_ref(w) {
                let crate_name = w.to_string();
                if !crates.contains(&crate_name) {
                    crates.push(crate_name);
                }
            }
        }
        return (crates, paths, keywords, is_unknown);
    }

    // Check for "unknown" / "unsure" answers
    if content_lower.contains("unknown") || content_lower.contains("unsure") {
        is_unknown = true;
    }

    // Extract cclab-* crate references + bare arsenal crates
    for word in content.split_whitespace() {
        let w = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if is_crate_ref(w) {
            let crate_name = w.to_string();
            if !crates.contains(&crate_name) {
                crates.push(crate_name);
            }
        }
    }

    // Extract path references (crates/*/src/ patterns)
    for word in content.split_whitespace() {
        let w = word.trim_matches(|c: char| {
            !c.is_alphanumeric() && c != '/' && c != '-' && c != '_' && c != '.'
        });
        if w.starts_with("crates/") || w.starts_with("src/") {
            let path = w.to_string();
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }

    // Extract keywords from scope context
    for kw in [
        "mcp", "cli", "state", "parser", "service", "model", "fillback",
    ] {
        if content_lower.contains(kw) {
            keywords.push(kw.to_string());
        }
    }

    (crates, paths, keywords, is_unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_scope_from_issue_labels() {
        let temp = TempDir::new().unwrap();
        let change_dir = temp.path();
        std::fs::create_dir_all(change_dir.join("issues")).unwrap();
        std::fs::write(
            change_dir.join("issues/issue_42.md"),
            "---\nnumber: 42\ntitle: \"Test\"\nstate: OPEN\nlabels: [bug, crate:genesis]\n---\n\n# Body\n",
        )
        .unwrap();

        let scope = extract_scope(change_dir);
        assert_eq!(scope.affected_crates, vec!["agentic-workflow"]);
        assert_eq!(scope.source, "issues");
        assert!(!scope.is_unknown);
    }

    #[test]
    fn test_extract_scope_from_clarifications() {
        let temp = TempDir::new().unwrap();
        let change_dir = temp.path();
        std::fs::write(
            change_dir.join("pre_clarifications.md"),
            "## Scope\n\nQ: Which modules are affected?\nA: The agentic-workflow crate, specifically the MCP tools.\n",
        )
        .unwrap();

        let scope = extract_scope(change_dir);
        assert!(scope
            .affected_crates
            .contains(&"agentic-workflow".to_string()));
        assert_eq!(scope.source, "clarifications");
    }

    #[test]
    fn test_extract_scope_unknown() {
        let temp = TempDir::new().unwrap();
        let change_dir = temp.path();
        std::fs::write(
            change_dir.join("pre_clarifications.md"),
            "## Scope\n\nQ: Which modules?\nA: Unknown at this point.\n",
        )
        .unwrap();

        let scope = extract_scope(change_dir);
        assert!(scope.is_unknown);
    }

    #[test]
    fn test_extract_scope_combined() {
        let temp = TempDir::new().unwrap();
        let change_dir = temp.path();
        std::fs::create_dir_all(change_dir.join("issues")).unwrap();
        std::fs::write(
            change_dir.join("issues/issue_10.md"),
            "---\nnumber: 10\ntitle: \"T\"\nstate: OPEN\nlabels: [crate:genesis]\n---\n\nBody\n",
        )
        .unwrap();
        std::fs::write(
            change_dir.join("pre_clarifications.md"),
            "## Scope\n\nAffected: cclab-lens and projects/agentic-workflow/src/mcp/\n",
        )
        .unwrap();

        let scope = extract_scope(change_dir);
        assert!(scope
            .affected_crates
            .contains(&"agentic-workflow".to_string()));
        assert!(scope.affected_crates.contains(&"cclab-lens".to_string()));
        assert_eq!(scope.source, "both");
    }

    #[test]
    fn test_extract_scope_no_artifacts() {
        let temp = TempDir::new().unwrap();
        let change_dir = temp.path();

        let scope = extract_scope(change_dir);
        assert!(scope.affected_crates.is_empty());
        assert_eq!(scope.source, "none");
        assert!(!scope.is_unknown);
    }

    #[test]
    fn test_suggest_topics_always_has_scope() {
        let topics = suggest_topics("Generic task description");
        assert!(topics.iter().any(|t| t.contains("Affected crates")));
    }

    #[test]
    fn test_suggest_topics_auth() {
        let topics = suggest_topics("Add OAuth authentication with Google");
        assert!(topics.iter().any(|t| t.contains("Authentication")));
        assert!(topics.iter().any(|t| t.contains("Affected crates")));
    }

    #[test]
    fn test_pre_filter_specs_with_matching_group() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join(".aw/tech-design/agentic-workflow");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("workflow.md"), "# Workflow\n").unwrap();
        std::fs::write(specs_dir.join("state.md"), "# State\n").unwrap();

        let result = pre_filter_specs(&["agentic-workflow".to_string()], temp.path(), None);
        assert!(result.contains("### agentic-workflow"));
        assert!(result.contains("read_path:specs/agentic-workflow/state.md"));
        assert!(result.contains("read_path:specs/agentic-workflow/workflow.md"));
    }

    #[test]
    fn test_extract_scope_from_requirements_patterns() {
        // crates/cclab-{name}/ pattern
        let result =
            extract_scope_from_requirements("Modify projects/agentic-workflow/src/tools/agent.rs");
        assert_eq!(result, vec!["agentic-workflow"]);

        // crate:{name} pattern
        let result = extract_scope_from_requirements("Affects crate:lens and crate:genesis");
        assert!(result.contains(&"cclab-lens".to_string()));
        assert!(result.contains(&"agentic-workflow".to_string())); // genesis → agentic-workflow

        // standalone cclab-{name}
        let result = extract_scope_from_requirements("Changes in cclab-pg and agentic-workflow");
        assert!(result.contains(&"cclab-pg".to_string()));
        assert!(result.contains(&"agentic-workflow".to_string()));

        // no matches
        let result = extract_scope_from_requirements("No crate references here");
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_scope_from_group_requirements() {
        let temp = TempDir::new().unwrap();
        let change_dir = temp.path();
        let group_dir = change_dir.join("groups/my-group");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(
            group_dir.join("requirements.md"),
            "Fix bug in crates/cclab-lens/src/parser.rs and update cclab-pg connection pool.",
        )
        .unwrap();

        let scope = extract_scope(change_dir);
        assert!(scope.affected_crates.contains(&"cclab-lens".to_string()));
        assert!(scope.affected_crates.contains(&"cclab-pg".to_string()));
    }

    #[test]
    fn test_pre_filter_specs_empty_groups() {
        let temp = TempDir::new().unwrap();
        let result = pre_filter_specs(&[], temp.path(), None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_pre_filter_specs_no_specs_dir() {
        let temp = TempDir::new().unwrap();
        let result = pre_filter_specs(&["agentic-workflow".to_string()], temp.path(), None);
        assert!(result.is_empty());
    }

    // ---------------------------------------------------------------------------
    // TC_resolve_hit — REQ-2: resolve_spec_dir returns config path when group in scopes
    // ---------------------------------------------------------------------------

    #[test]
    fn test_resolve_spec_dir_config_hit() {
        let temp = TempDir::new().unwrap();
        let specs_base = temp.path();
        // Create the configured path: specs_base/projects/agentic-workflow
        std::fs::create_dir_all(specs_base.join("projects/agentic-workflow")).unwrap();

        let mut scopes = HashMap::new();
        scopes.insert("agentic-workflow".to_string(), "projects".to_string());

        let result = resolve_spec_dir("agentic-workflow", specs_base, &scopes);
        assert_eq!(result, Some(specs_base.join("projects/agentic-workflow")));
    }

    // Config-scoped entry that does not exist on disk → None (no fallback)
    #[test]
    fn test_resolve_spec_dir_config_miss_no_fallback() {
        let temp = TempDir::new().unwrap();
        let specs_base = temp.path();
        // Create crates/cclab-foo (classic fallback path), but group is configured to "custom"
        std::fs::create_dir_all(specs_base.join("crates/cclab-foo")).unwrap();

        let mut scopes = HashMap::new();
        scopes.insert("cclab-foo".to_string(), "custom".to_string());

        // Explicitly configured to "custom", but custom/cclab-foo doesn't exist → None
        let result = resolve_spec_dir("cclab-foo", specs_base, &scopes);
        assert_eq!(result, None);
    }

    // TC_resolve_miss_fallback — REQ-7: fallback probe when group not in scopes
    #[test]
    fn test_resolve_spec_dir_fallback_crates() {
        let temp = TempDir::new().unwrap();
        let specs_base = temp.path();
        std::fs::create_dir_all(specs_base.join("crates/my-crate")).unwrap();

        let scopes = HashMap::new();
        let result = resolve_spec_dir("my-crate", specs_base, &scopes);
        assert_eq!(result, Some(specs_base.join("crates/my-crate")));
    }

    #[test]
    fn test_resolve_spec_dir_fallback_projects() {
        let temp = TempDir::new().unwrap();
        let specs_base = temp.path();
        std::fs::create_dir_all(specs_base.join("projects/my-project")).unwrap();

        let scopes = HashMap::new();
        let result = resolve_spec_dir("my-project", specs_base, &scopes);
        assert_eq!(result, Some(specs_base.join("projects/my-project")));
    }

    #[test]
    fn test_resolve_spec_dir_fallback_root() {
        let temp = TempDir::new().unwrap();
        let specs_base = temp.path();
        std::fs::create_dir_all(specs_base.join("some-group")).unwrap();

        let scopes = HashMap::new();
        let result = resolve_spec_dir("some-group", specs_base, &scopes);
        assert_eq!(result, Some(specs_base.join("some-group")));
    }

    #[test]
    fn test_resolve_spec_dir_for_root_prefers_project_td_path() {
        let temp = TempDir::new().unwrap();
        let specs_base = temp.path().join(".aw/tech-design");
        let project_td = temp.path().join("projects/agentic-workflow/tech-design");
        std::fs::create_dir_all(&project_td).unwrap();
        std::fs::create_dir_all(temp.path().join(".aw")).unwrap();
        std::fs::write(
            temp.path().join(".aw/config.toml"),
            r#"
[agentic_workflow.tech_design_platform]
path = ".aw/tech-design"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
"#,
        )
        .unwrap();

        let scopes = HashMap::new();
        let result =
            resolve_spec_dir_for_root("agentic-workflow", temp.path(), &specs_base, &scopes);

        assert_eq!(result, Some(project_td));
    }

    // TC_resolve_miss_none — REQ-2: returns None when group absent everywhere
    #[test]
    fn test_resolve_spec_dir_not_found() {
        let temp = TempDir::new().unwrap();
        let specs_base = temp.path();

        let scopes = HashMap::new();
        let result = resolve_spec_dir("unknown-group", specs_base, &scopes);
        assert_eq!(result, None);
    }

    // TC_pre_filter_config — REQ-3: pre_filter_specs with config finds specs in configured subdir
    #[test]
    fn test_pre_filter_specs_with_config_scoped_subdir() {
        let temp = TempDir::new().unwrap();
        // Create .aw/tech-design/crates/cclab-lens/ with spec files
        let specs_dir = temp.path().join(".aw/tech-design/crates/cclab-lens");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("parser.md"), "# Parser\n").unwrap();
        std::fs::write(specs_dir.join("semantic.md"), "# Semantic\n").unwrap();

        let mut config = SddConfig::default();
        config
            .specs
            .scopes
            .insert("cclab-lens".to_string(), "crates".to_string());

        let result = pre_filter_specs(&["cclab-lens".to_string()], temp.path(), Some(&config));
        assert!(result.contains("### cclab-lens"));
        assert!(result.contains("parser"));
        assert!(result.contains("semantic"));
    }

    #[test]
    fn test_pre_filter_specs_with_project_td_path() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp
            .path()
            .join("projects/agentic-workflow/tech-design/logic");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("state.md"), "# State\n").unwrap();
        std::fs::create_dir_all(temp.path().join(".aw")).unwrap();
        std::fs::write(
            temp.path().join(".aw/config.toml"),
            r#"
[agentic_workflow.tech_design_platform]
path = ".aw/tech-design"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
"#,
        )
        .unwrap();

        let result = pre_filter_specs(&["agentic-workflow".to_string()], temp.path(), None);

        assert!(result.contains("### agentic-workflow"));
        assert!(result.contains("logic/state.md"));
    }

    // TC_backward_compat — REQ-7: empty scopes triggers crates/ → projects/ → root fallback unchanged
    #[test]
    fn test_backward_compat_empty_scopes_fallback() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join(".aw/tech-design/crates/cclab-pg");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("query.md"), "# Query\n").unwrap();

        // Pass None config (no scopes) — should fall back to crates/ probe
        let result = pre_filter_specs(&["cclab-pg".to_string()], temp.path(), None);
        assert!(result.contains("### cclab-pg"));
        assert!(result.contains("query"));
    }

    // pre_filter_specs with config where group is absent from scopes falls back too
    #[test]
    fn test_pre_filter_specs_config_present_but_group_not_in_scopes() {
        let temp = TempDir::new().unwrap();
        // spec lives under crates/ (classic fallback)
        let specs_dir = temp
            .path()
            .join("projects/agentic-workflow/tech-design/core");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("logic.md"), "# Logic\n").unwrap();

        // Config exists but does NOT have agentic-workflow in scopes — fallback still works
        let config = SddConfig::default();
        let result = pre_filter_specs(
            &["agentic-workflow".to_string()],
            temp.path(),
            Some(&config),
        );
        assert!(result.contains("### agentic-workflow"));
        assert!(result.contains("logic"));
    }

    // ---------------------------------------------------------------------------
    // build_spec_dir_tree tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_build_spec_dir_tree_empty_groups() {
        let temp = TempDir::new().unwrap();
        let result = build_spec_dir_tree(&[], temp.path(), None);
        assert!(result.is_empty(), "empty spec_groups → empty string");
    }

    #[test]
    fn test_build_spec_dir_tree_no_specs_dir() {
        let temp = TempDir::new().unwrap();
        // No .aw/tech-design/ directory created
        let result = build_spec_dir_tree(&["agentic-workflow".to_string()], temp.path(), None);
        assert!(result.is_empty(), "missing .aw/tech-design/ → empty string");
    }

    #[test]
    fn test_build_spec_dir_tree_unknown_group_returns_empty() {
        let temp = TempDir::new().unwrap();
        // Create .aw/tech-design/ but no group directory inside
        std::fs::create_dir_all(temp.path().join(".aw/tech-design")).unwrap();
        let result = build_spec_dir_tree(&["unknown-group".to_string()], temp.path(), None);
        assert!(result.is_empty(), "group not found → empty string");
    }

    #[test]
    fn test_build_spec_dir_tree_single_group_flat_files() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp
            .path()
            .join("projects/agentic-workflow/tech-design/core");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("logic.md"), "# Logic\n").unwrap();
        std::fs::write(specs_dir.join("state.md"), "# State\n").unwrap();

        let result = build_spec_dir_tree(&["agentic-workflow".to_string()], temp.path(), None);
        assert!(!result.is_empty(), "should return tree output");
        // Directory name should appear as tree root line
        assert!(
            result.contains("agentic-workflow"),
            "tree should contain group dir name"
        );
        // Both files should appear in tree
        assert!(
            result.contains("logic.md"),
            "logic.md should appear in tree"
        );
        assert!(
            result.contains("state.md"),
            "state.md should appear in tree"
        );
        // Tree connectors should be present
        assert!(
            result.contains("├──") || result.contains("└──"),
            "tree connectors must be present"
        );
    }

    #[test]
    fn test_build_spec_dir_tree_last_entry_uses_corner_connector() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join(".aw/tech-design/crates/my-crate");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("api.md"), "# API\n").unwrap();

        let result = build_spec_dir_tree(&["my-crate".to_string()], temp.path(), None);
        // Single file → last (and only) entry uses └──
        assert!(result.contains("└──"), "last entry must use └── connector");
        assert!(!result.contains("├──"), "only one file → no ├── connector");
    }

    #[test]
    fn test_build_spec_dir_tree_multiple_files_uses_correct_connectors() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join(".aw/tech-design/crates/cclab-pg");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("a.md"), "# A\n").unwrap();
        std::fs::write(specs_dir.join("b.md"), "# B\n").unwrap();
        std::fs::write(specs_dir.join("c.md"), "# C\n").unwrap();

        let result = build_spec_dir_tree(&["cclab-pg".to_string()], temp.path(), None);
        // Multiple entries: non-last entries use ├──, last uses └──
        assert!(
            result.contains("├──"),
            "non-last entries must use ├── connector"
        );
        assert!(result.contains("└──"), "last entry must use └── connector");
    }

    #[test]
    fn test_build_spec_dir_tree_nested_subdirs() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp
            .path()
            .join("projects/agentic-workflow/tech-design/core");
        std::fs::create_dir_all(specs_dir.join("logic")).unwrap();
        std::fs::write(specs_dir.join("logic").join("state-machine.md"), "# SM\n").unwrap();
        std::fs::write(specs_dir.join("logic").join("rules.md"), "# Rules\n").unwrap();

        let result = build_spec_dir_tree(&["agentic-workflow".to_string()], temp.path(), None);
        assert!(result.contains("logic"), "subdir name should appear");
        // At least one nested file should appear
        assert!(
            result.contains("state-machine.md") || result.contains("rules.md"),
            "nested files should appear in tree"
        );
    }

    #[test]
    fn test_build_spec_dir_tree_nested_subdir_uses_pipe_prefix() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp
            .path()
            .join("projects/agentic-workflow/tech-design/core");
        // Two top-level entries: subdir + file (so subdir uses ├── and file uses └──)
        std::fs::create_dir_all(specs_dir.join("logic")).unwrap();
        std::fs::write(specs_dir.join("logic").join("spec.md"), "# Spec\n").unwrap();
        std::fs::write(specs_dir.join("top.md"), "# Top\n").unwrap();

        let result = build_spec_dir_tree(&["agentic-workflow".to_string()], temp.path(), None);
        // Non-last top-level entries cause a │ pipe prefix for children
        assert!(
            result.contains("│"),
            "nested entries under non-last dir must use │ prefix"
        );
    }

    #[test]
    fn test_build_spec_dir_tree_multiple_groups() {
        let temp = TempDir::new().unwrap();
        let specs_a = temp.path().join(".aw/tech-design/crates/group-a");
        let specs_b = temp.path().join(".aw/tech-design/crates/group-b");
        std::fs::create_dir_all(&specs_a).unwrap();
        std::fs::create_dir_all(&specs_b).unwrap();
        std::fs::write(specs_a.join("spec-a.md"), "# A\n").unwrap();
        std::fs::write(specs_b.join("spec-b.md"), "# B\n").unwrap();

        let groups = vec!["group-a".to_string(), "group-b".to_string()];
        let result = build_spec_dir_tree(&groups, temp.path(), None);
        assert!(result.contains("group-a"), "group-a dir should appear");
        assert!(result.contains("group-b"), "group-b dir should appear");
        assert!(result.contains("spec-a.md"), "spec-a.md should appear");
        assert!(result.contains("spec-b.md"), "spec-b.md should appear");
    }

    #[test]
    fn test_build_spec_dir_tree_with_config_scope() {
        let temp = TempDir::new().unwrap();
        // Create spec dir at the config-driven path: .aw/tech-design/crates/cclab-lens
        let specs_dir = temp.path().join(".aw/tech-design/crates/cclab-lens");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("parser.md"), "# Parser\n").unwrap();

        let mut config = SddConfig::default();
        config
            .specs
            .scopes
            .insert("cclab-lens".to_string(), "crates".to_string());

        let result = build_spec_dir_tree(&["cclab-lens".to_string()], temp.path(), Some(&config));
        assert!(
            result.contains("cclab-lens"),
            "group dir name should appear in tree"
        );
        assert!(
            result.contains("parser.md"),
            "spec file should appear in tree"
        );
    }

    #[test]
    fn test_build_spec_dir_tree_config_miss_returns_empty() {
        let temp = TempDir::new().unwrap();
        // Config-driven path doesn't exist on disk → None → group skipped → empty result
        std::fs::create_dir_all(temp.path().join(".aw/tech-design")).unwrap();

        let mut config = SddConfig::default();
        config
            .specs
            .scopes
            .insert("cclab-foo".to_string(), "custom".to_string());
        // "custom/cclab-foo" doesn't exist → no fallback → empty

        let result = build_spec_dir_tree(&["cclab-foo".to_string()], temp.path(), Some(&config));
        assert!(
            result.is_empty(),
            "config-miss with no fallback → empty string"
        );
    }
}

// CODEGEN-END

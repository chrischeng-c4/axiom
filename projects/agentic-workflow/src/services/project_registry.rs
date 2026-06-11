// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
// CODEGEN-BEGIN
//! Project registry: read/write `[[projects]]` block in `.aw/config.toml`.
//!
//! The auto-generated block is delimited by:
//! - `SYNC_BEGIN_MARKER` — `# BEGIN AW SYNC — auto-generated, do not edit by hand`
//! - `SYNC_END_MARKER`   — `# END AW SYNC`
//!
//! `toml_edit` is used for lossless round-trips so that non-generated sections
//! (comments, formatting, sdd.* tables) are preserved byte-identical on each sync.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::models::project::Project;
use crate::services::project_discovery::discover_projects;
use crate::shared::workspace::{config_path, workspace_path, SYNC_BEGIN_MARKER, SYNC_END_MARKER};

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R1
// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R4
/// Write `[[projects]]` entries into the marker-delimited block in `.aw/config.toml`.
///
/// - If the BEGIN/END AW SYNC marker pair is already present, the content
///   between the markers is replaced with a freshly serialized `[[projects]]` block.
/// - If the markers are absent, the block (with markers) is appended at EOF.
/// - After a successful write, `.aw/projects.toml` is deleted if it exists (R10 migration).
/// - Non-generated content in `config.toml` is preserved byte-identical via `toml_edit`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub fn write_projects_config(root: &Path, projects: &[Project]) -> Result<()> {
    let config_file = config_path(root);
    std::fs::create_dir_all(config_file.parent().unwrap())?;

    // Read existing config.toml or start with empty string
    let existing = if config_file.exists() {
        std::fs::read_to_string(&config_file)
            .with_context(|| format!("reading {}", config_file.display()))?
    } else {
        String::new()
    };

    // Serialize the [[projects]] block
    let block = serialize_projects_block(projects)?;

    // Splice or append the marker-delimited block
    let new_content = splice_or_append(&existing, &block);

    std::fs::write(&config_file, &new_content)
        .with_context(|| format!("writing {}", config_file.display()))?;

    // Migration: delete stale projects.toml if present (R10)
    let stale = workspace_path(root).join("projects.toml");
    if stale.exists() {
        std::fs::remove_file(&stale)
            .with_context(|| format!("deleting stale {}", stale.display()))?;
    }

    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R9
/// Load the project list from `.aw/config.toml`.
///
/// Reads `[[projects]]` entries directly from `config.toml` — the marker block
/// is written there by `write_projects_config`. No projects.toml overlay.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub fn load_projects(root: &Path) -> Result<Vec<Project>> {
    let config_file = config_path(root);
    if !config_file.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&config_file)
        .with_context(|| format!("reading {}", config_file.display()))?;

    #[derive(serde::Deserialize, Default)]
    struct ConfigWithProjects {
        #[serde(default)]
        projects: Vec<Project>,
    }

    let parsed: ConfigWithProjects = toml::from_str(&content)
        .with_context(|| format!("parsing projects from {}", config_file.display()))?;

    Ok(parsed.projects)
}

// @spec projects/agentic-workflow/tech-design/surface/specs/sync-command.md#R11
/// Compute a diff between the current marker-delimited block in `config.toml`
/// and a freshly discovered set of projects.
///
/// Returns `Some(unified_diff)` if different, `None` if identical.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub fn check_drift(root: &Path) -> Result<Option<String>> {
    // Generate fresh block content (without writing)
    let discovered = discover_projects(root)?;
    let fresh_block = serialize_projects_block(&discovered)?;

    let config_file = config_path(root);
    if !config_file.exists() {
        if fresh_block.trim().is_empty() {
            return Ok(None);
        }
        return Ok(Some(build_diff("", &fresh_block, "config.toml")));
    }

    let existing_content = std::fs::read_to_string(&config_file)
        .with_context(|| format!("reading {}", config_file.display()))?;

    // Extract the current block from config.toml (between markers)
    let current_block = extract_sync_block(&existing_content).unwrap_or_default();

    if current_block.trim() == fresh_block.trim() {
        Ok(None)
    } else {
        Ok(Some(build_diff(
            &current_block,
            &fresh_block,
            "config.toml",
        )))
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Serialize a list of projects into the `[[projects]]` TOML block string
/// (without markers).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
fn serialize_projects_block(projects: &[Project]) -> Result<String> {
    #[derive(serde::Serialize)]
    struct ProjectsOnly<'a> {
        projects: &'a [Project],
    }
    let doc = ProjectsOnly { projects };
    toml::to_string_pretty(&doc).context("serializing [[projects]] block")
}

/// If BEGIN/END AW SYNC markers are found in `existing`, replace the content
/// between them (inclusive of the marker lines) with the new block surrounded by
/// markers. Otherwise append the block at EOF with a blank-line separator.
///
/// The result preserves all content outside the delimited region byte-identical.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
fn splice_or_append(existing: &str, block: &str) -> String {
    let begin = SYNC_BEGIN_MARKER;
    let end = SYNC_END_MARKER;

    // Find marker positions by scanning lines
    let lines: Vec<&str> = existing.lines().collect();
    let begin_idx = lines.iter().position(|l| l.trim() == begin);
    let end_idx = lines.iter().position(|l| l.trim() == end);

    if let (Some(bi), Some(ei)) = (begin_idx, end_idx) {
        // Replace lines from bi..=ei (inclusive) with fresh marker block
        let mut result = String::new();
        for line in &lines[..bi] {
            result.push_str(line);
            result.push('\n');
        }
        result.push_str(begin);
        result.push('\n');
        result.push('\n');
        result.push_str(block.trim_end());
        result.push('\n');
        result.push('\n');
        result.push_str(end);
        result.push('\n');
        for line in &lines[(ei + 1)..] {
            result.push_str(line);
            result.push('\n');
        }
        result
    } else {
        // Append at EOF with blank-line separator
        let mut result = existing.to_string();
        // Ensure we don't double-newline if existing ends with newline
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(begin);
        result.push('\n');
        result.push('\n');
        result.push_str(block.trim_end());
        result.push('\n');
        result.push('\n');
        result.push_str(end);
        result.push('\n');
        result
    }
}

/// Extract only the content between BEGIN and END AW SYNC markers (exclusive
/// of the marker lines themselves).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
fn extract_sync_block(content: &str) -> Option<String> {
    let begin = SYNC_BEGIN_MARKER;
    let end = SYNC_END_MARKER;
    let lines: Vec<&str> = content.lines().collect();
    let begin_idx = lines.iter().position(|l| l.trim() == begin)?;
    let end_idx = lines.iter().position(|l| l.trim() == end)?;
    if end_idx <= begin_idx {
        return None;
    }
    Some(lines[(begin_idx + 1)..end_idx].join("\n"))
}

/// Build a simple unified-style diff between two strings.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
fn build_diff(old: &str, new: &str, label: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut out = format!("--- {}\n+++ {} (fresh discovery)\n", label, label);

    let mut i = 0;
    let mut j = 0;
    while i < old_lines.len() || j < new_lines.len() {
        let old_line = old_lines.get(i).copied();
        let new_line = new_lines.get(j).copied();

        match (old_line, new_line) {
            (Some(o), Some(n)) if o == n => {
                out.push(' ');
                out.push_str(o);
                out.push('\n');
                i += 1;
                j += 1;
            }
            (Some(o), _) => {
                out.push('-');
                out.push_str(o);
                out.push('\n');
                i += 1;
            }
            (None, Some(n)) => {
                out.push('+');
                out.push_str(n);
                out.push('\n');
                j += 1;
            }
            (None, None) => break,
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
mod tests {
    use super::*;
    use crate::models::project::{Project, Workspace};
    use crate::models::tech_stack::Language;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Build a minimal "repo root" with a `.aw/` dir and return the TempDir.
    fn make_score_root() -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        tmp
    }

    /// Write `content` to `.aw/config.toml` inside `root`.
    fn write_config_file(root: &std::path::Path, content: &str) {
        let path = root.join(".aw").join("config.toml");
        fs::write(&path, content).unwrap();
    }

    /// Create a minimal Project for use in tests.
    fn make_project(name: &str, target: Language, test_cmd: Option<&str>) -> Project {
        Project {
            name: name.to_string(),
            path: PathBuf::from(format!("crates/{}", name)),
            tech_design_dir: None,
            ec: Default::default(),
            workspaces: vec![Workspace {
                name: Some(name.to_string()),
                paths: vec![format!("crates/{}/**", name)],
                target,
                test_cmd: test_cmd.map(|s| s.to_string()),
                codegen: None,
            }],
        }
    }

    // REQ: REQ-001 (R1: writes to config.toml)
    // T17: marker_upsert_first_run
    #[test]
    fn marker_upsert_first_run() {
        let tmp = make_score_root();

        // config.toml exists but has no markers
        write_config_file(
            tmp.path(),
            "[agentic_workflow.test.scope]\nroots = [\"crates\"]\n",
        );

        let projects = vec![make_project(
            "proj-a",
            Language::Rust,
            Some("cargo test -p proj-a"),
        )];
        write_projects_config(tmp.path(), &projects).unwrap();

        let path = tmp.path().join(".aw").join("config.toml");
        let content = fs::read_to_string(&path).unwrap();

        // R2: markers present
        assert!(
            content.contains(SYNC_BEGIN_MARKER),
            "config.toml must contain BEGIN AW SYNC marker; got:\n{content}"
        );
        assert!(
            content.contains(SYNC_END_MARKER),
            "config.toml must contain END AW SYNC marker; got:\n{content}"
        );

        // R1: [[projects]] entries present
        assert!(
            content.contains("proj-a"),
            "config.toml must contain discovered project; got:\n{content}"
        );

        // R3: user content untouched
        assert!(
            content.contains("[agentic_workflow.test.scope]"),
            "user-authored section must survive sync; got:\n{content}"
        );
        assert!(
            content.contains("roots = [\"crates\"]"),
            "user-authored content must survive sync byte-identical; got:\n{content}"
        );
    }

    // REQ: REQ-003 (R3: non-projects content preserved)
    // REQ: REQ-005 (R5: idempotency)
    // REQ: REQ-006 (R6: full enumeration)
    // T18: marker_upsert_round_trip
    #[test]
    fn marker_upsert_round_trip() {
        let tmp = make_score_root();

        // Pre-existing user content
        write_config_file(
            tmp.path(),
            "# user comment\n[agentic_workflow.test.scope]\nroots = [\"crates\"]\n\n[defaults.workspace]\ncodegen.target = \"rust\"\n",
        );

        let projects = vec![
            make_project("alpha", Language::Rust, Some("cargo test -p alpha")),
            make_project(
                "beta",
                Language::Python,
                Some("cd projects/beta && uv run pytest"),
            ),
            make_project("gamma", Language::TypeScript, None),
        ];

        // First sync
        write_projects_config(tmp.path(), &projects).unwrap();
        let after_first = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();

        // Second sync with identical input (idempotency check, R5)
        write_projects_config(tmp.path(), &projects).unwrap();
        let after_second = fs::read_to_string(tmp.path().join(".aw").join("config.toml")).unwrap();

        assert_eq!(
            after_first, after_second,
            "double sync with identical input must produce zero diff (R5 idempotency)"
        );

        // R3: non-projects sections byte-identical
        assert!(
            after_second.contains("# user comment"),
            "user comment must be preserved; got:\n{after_second}"
        );
        assert!(
            after_second.contains("[agentic_workflow.test.scope]"),
            "user section must be preserved; got:\n{after_second}"
        );
        assert!(
            after_second.contains("[defaults.workspace]"),
            "defaults section must be preserved; got:\n{after_second}"
        );

        // R6: full enumeration — assert count via round-trip load
        let loaded = load_projects(tmp.path()).unwrap();
        assert_eq!(
            loaded.len(),
            projects.len(),
            "R6: all {} projects must be written and readable; got {}",
            projects.len(),
            loaded.len()
        );
    }

    // REQ: REQ-010 (R10: migration deletes projects.toml)
    // T21: migration_deletes_projects_toml
    #[test]
    fn migration_deletes_projects_toml() {
        let tmp = make_score_root();

        // Write a stale projects.toml
        let stale = tmp.path().join(".aw").join("projects.toml");
        fs::write(&stale, "# old file\n[[projects]]\nname = \"old\"\n").unwrap();
        assert!(stale.exists(), "stale projects.toml must exist before sync");

        let projects = vec![make_project(
            "new-proj",
            Language::Rust,
            Some("cargo test -p new-proj"),
        )];
        write_projects_config(tmp.path(), &projects).unwrap();

        assert!(
            !stale.exists(),
            "stale .aw/projects.toml must be deleted after successful sync"
        );
    }

    // REQ: REQ-009 (R9: consumers read from config.toml only)
    #[test]
    fn load_reads_from_config_toml() {
        let tmp = make_score_root();

        // Write projects via write_projects_config
        let projects = vec![make_project(
            "my-crate",
            Language::Rust,
            Some("cargo test -p my-crate"),
        )];
        write_projects_config(tmp.path(), &projects).unwrap();

        // load_projects must return data from config.toml
        let loaded = load_projects(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "my-crate");
    }

    // REQ: REQ-005 (R5: check_drift detects changes)
    #[test]
    fn check_drift_round_trip() {
        let tmp = make_score_root();

        // Create a minimal Cargo project so discovery finds one project.
        let proj_dir = tmp.path().join("crates").join("round-trip");
        fs::create_dir_all(&proj_dir).unwrap();
        fs::write(
            proj_dir.join("Cargo.toml"),
            "[package]\nname = \"round-trip\"\n",
        )
        .unwrap();

        // Discover and write to config.toml.
        let discovered = discover_projects(tmp.path()).unwrap();
        write_projects_config(tmp.path(), &discovered).unwrap();

        // check_drift should detect no difference.
        let drift = check_drift(tmp.path()).unwrap();
        assert!(drift.is_none(), "expected no drift after round-trip write");
    }

    // REQ: REQ-011 (R11: --check / check_drift targets config.toml)
    #[test]
    fn check_drift_no_write() {
        let tmp = make_score_root();

        // Write a config.toml with a stale entry that won't match fresh discovery
        write_config_file(
            tmp.path(),
            &format!("{}\n\n[[projects]]\nname = \"stale-proj\"\npath = \"crates/stale-proj\"\n\n[[projects.workspaces]]\nname = \"stale-proj\"\npaths = [\"crates/stale-proj/**\"]\ntarget = \"rust\"\n\n{}\n", SYNC_BEGIN_MARKER, SYNC_END_MARKER),
        );

        let drift = check_drift(tmp.path()).unwrap();
        assert!(
            drift.is_some(),
            "expected drift when config.toml differs from fresh discovery"
        );

        // config.toml must NOT be modified by check_drift
        let path = tmp.path().join(".aw").join("config.toml");
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("stale-proj"),
            "check_drift must not modify config.toml"
        );
    }

    // REQ: REQ-011 (R11: check_drift returns Some when content differs)
    #[test]
    fn check_drift_exits_nonzero_on_diff() {
        let tmp = make_score_root();

        // Write a config.toml that won't match fresh discovery (no real dirs)
        write_config_file(
            tmp.path(),
            &format!("{}\n\n[[projects]]\nname = \"ghost\"\npath = \"crates/ghost\"\n\n[[projects.workspaces]]\nname = \"ghost\"\npaths = [\"crates/ghost/**\"]\ntarget = \"rust\"\n\n{}\n", SYNC_BEGIN_MARKER, SYNC_END_MARKER),
        );

        let drift = check_drift(tmp.path()).unwrap();
        assert!(
            drift.is_some(),
            "check_drift should return Some when content differs (drift detected)"
        );
    }

    // REQ: REQ-001 (R1: write target is config.toml, diff references config.toml)
    #[test]
    fn check_drift_references_config_toml() {
        let tmp = make_score_root();

        // Stale config.toml with a ghost project
        write_config_file(
            tmp.path(),
            &format!("{}\n\n[[projects]]\nname = \"ghost2\"\npath = \"crates/ghost2\"\n\n[[projects.workspaces]]\nname = \"ghost2\"\npaths = [\"crates/ghost2/**\"]\ntarget = \"rust\"\n\n{}\n", SYNC_BEGIN_MARKER, SYNC_END_MARKER),
        );

        let drift = check_drift(tmp.path()).unwrap().expect("expected drift");
        assert!(
            drift.contains("config.toml"),
            "--check / check_drift output must reference config.toml, not projects.toml; got:\n{drift}"
        );
    }
}

/// Project descriptor consumed by `ProjectRegistry::resolve_td_root`.
/// Materialised from `[[projects]]` table rows in `.aw/config.toml`.
/// @spec projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub struct TdRootInput {
    /// Project name (matches `[[projects]].name`).
    pub name: String,
    /// Optional per-project repo-relative spec root override
    /// (matches `[[projects]].td_path`).
    /// When non-null this value wins over the project-local convention.
    #[serde(default)]
    pub td_path: Option<String>,
    /// Project root (matches `[[projects]].path`). Used to derive the
    /// conventional TD root when `td_path` is null.
    pub source_path: String,
}

/// Output of `resolve_td_root`.
/// @spec projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub struct TdRootResult {
    /// Absolute filesystem path to the project's TD spec root.
    pub root: String,
    /// Indicates which precedence branch resolved the path. Surfaced for
    /// diagnostics; consumers MUST NOT branch on this value.
    pub precedence: String,
}

/// Failure modes of `resolve_td_root`.
/// @spec projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub struct TdResolveError {
    /// - unknown_project: project name not in `[[projects]]` table.
    /// - td_path_escapes_repo_root: `td_path` resolves outside the repo
    ///   root after canonicalisation (e.g. `../../something`).
    /// Project-local fallback does not require global TD platform config.
    pub kind: String,
    pub message: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
fn lex_normalize(path: &std::path::Path) -> std::path::PathBuf {
    let mut out = std::path::PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
impl TdResolveError {
    fn unknown_project(name: &str) -> Self {
        Self {
            kind: "unknown_project".into(),
            message: format!("project '{name}' is not registered in [[projects]]"),
        }
    }

    fn td_path_escapes_repo_root(td_path: &str) -> Self {
        Self {
            kind: "td_path_escapes_repo_root".into(),
            message: format!("td_path '{td_path}' resolves outside the repository root"),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub fn default_project_td_path(source_path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(source_path).join("tech-design")
}

/// Resolve the TD spec root for `input` against `repo_root`.
///
/// Precedence (matches spec flowchart):
/// 1. If `input.td_path` is `Some(p)` → candidate = `repo_root.join(p)`, precedence = `"td_path"`.
/// 2. Else → candidate = `repo_root.join(input.source_path).join("tech-design")`,
///    precedence = `"project_path"`.
///
/// After composing `candidate`, canonicalise and verify it stays inside
/// `repo_root` (defends against `../` escapes). Canonicalisation falls back
/// to lexical containment when the candidate does not yet exist on disk —
/// the resolver runs during `aw td init` before the spec dir is created.
///
/// @spec projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md#logic
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub fn resolve_td_root(
    input: &TdRootInput,
    _global_base: Option<&str>,
    repo_root: &std::path::Path,
) -> std::result::Result<TdRootResult, TdResolveError> {
    let (candidate, precedence) = if let Some(td_path) = input.td_path.as_deref() {
        (repo_root.join(td_path), "td_path")
    } else {
        (
            repo_root.join(default_project_td_path(&input.source_path)),
            "project_path",
        )
    };

    // Lexical containment check — the spec only requires defending against
    // `../` escape sequences. Filesystem canonicalisation is unreliable here
    // because the candidate may not exist yet (resolver runs during
    // `aw td init`, before the spec directory is created) and because
    // macOS resolves `/var/...` → `/private/var/...` asymmetrically.
    let normalized_root = lex_normalize(repo_root);
    let normalized_candidate = lex_normalize(&candidate);

    if !normalized_candidate.starts_with(&normalized_root) {
        let display = input
            .td_path
            .as_deref()
            .map(str::to_string)
            .unwrap_or_else(|| {
                default_project_td_path(&input.source_path)
                    .to_string_lossy()
                    .into_owned()
            })
            .to_string();
        return Err(TdResolveError::td_path_escapes_repo_root(&display));
    }

    Ok(TdRootResult {
        root: candidate.to_string_lossy().into_owned(),
        precedence: precedence.into(),
    })
}

/// Convenience wrapper: load `.aw/config.toml`, look up the project by
/// name, and dispatch to `resolve_td_root`. Reads only the narrow
/// projection needed (`[[projects]].{name,aliases,path,td_path}`) to stay decoupled from the broader
/// `Project` schema.
///
/// @spec projects/agentic-workflow/tech-design/core/specs/td-root-resolver.md#logic
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
pub fn resolve_td_root_from_config(
    repo_root: &std::path::Path,
    project_name: &str,
) -> std::result::Result<TdRootResult, TdResolveError> {
    #[derive(serde::Deserialize, Default)]
    struct Config {
        #[serde(default)]
        projects: Vec<ProjectRow>,
    }
    #[derive(serde::Deserialize)]
    struct ProjectRow {
        name: String,
        #[serde(default)]
        aliases: Vec<String>,
        path: String,
        #[serde(default)]
        td_path: Option<String>,
    }

    let config_file = repo_root.join(".aw").join("config.toml");
    let content = std::fs::read_to_string(&config_file)
        .map_err(|_| TdResolveError::unknown_project(project_name))?;
    let parsed: Config = toml::from_str(&content).map_err(|e| TdResolveError {
        kind: "config_parse_error".into(),
        message: format!("parsing {}: {}", config_file.display(), e),
    })?;

    let row = parsed
        .projects
        .into_iter()
        .find(|r| r.name == project_name || r.aliases.iter().any(|alias| alias == project_name))
        .ok_or_else(|| TdResolveError::unknown_project(project_name))?;

    let input = TdRootInput {
        name: row.name,
        td_path: row.td_path,
        source_path: row.path,
    };

    resolve_td_root(&input, None, repo_root)
}

#[cfg(test)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/project_registry.md#source
mod resolver_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn repo() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn td_path_wins_when_set() {
        let tmp = repo();
        let input = TdRootInput {
            name: "sdd".into(),
            td_path: Some("projects/cgdb/tech_design".into()),
            source_path: "projects/cgdb".into(),
        };
        let out = resolve_td_root(&input, Some(".aw/tech-design"), tmp.path()).unwrap();
        assert_eq!(out.precedence, "td_path");
        assert_eq!(
            PathBuf::from(&out.root),
            tmp.path().join("projects/cgdb/tech_design")
        );
    }

    #[test]
    fn falls_back_to_project_tech_design() {
        let tmp = repo();
        let input = TdRootInput {
            name: "sdd".into(),
            td_path: None,
            source_path: "projects/agentic-workflow".into(),
        };
        let out = resolve_td_root(&input, Some(".aw/tech-design"), tmp.path()).unwrap();
        assert_eq!(out.precedence, "project_path");
        assert_eq!(
            PathBuf::from(&out.root),
            tmp.path().join("projects/agentic-workflow/tech-design")
        );
    }

    #[test]
    fn does_not_require_global_base_for_project_fallback() {
        let tmp = repo();
        let input = TdRootInput {
            name: "x".into(),
            td_path: None,
            source_path: "x".into(),
        };
        let out = resolve_td_root(&input, None, tmp.path()).unwrap();
        assert_eq!(PathBuf::from(out.root), tmp.path().join("x/tech-design"));
    }

    #[test]
    fn errors_when_td_path_escapes_repo_root() {
        let tmp = repo();
        let input = TdRootInput {
            name: "x".into(),
            td_path: Some("../escape".into()),
            source_path: "x".into(),
        };
        let err = resolve_td_root(&input, Some(".aw/tech-design"), tmp.path()).unwrap_err();
        assert_eq!(err.kind, "td_path_escapes_repo_root");
    }
}

// CODEGEN-END

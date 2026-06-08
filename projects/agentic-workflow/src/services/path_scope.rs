// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_preamble_source.md#source
// CODEGEN-BEGIN
//! Edit-scope resolution for the internal pretooluse write-scope hook.
//!
//! Reads `.aw/config.toml`, locates the `[[projects]]` entry by name,
//! and answers "is this relative path inside the project's allowed
//! edit scope?" (path prefix, td_path prefix, or any
//! `[[projects.workspaces]].paths` glob).
//!
//! Schema is intentionally narrower than `models::project::Project` —
//! we only deserialize the fields the hook needs (`name`, `path`,
//! `td_path`, `workspaces.paths`). Anything else (`label`, `target`,
//! `test_cmd`, `codegen`, …) is ignored. This decouples the hook from
//! `Project` schema churn and matches the original Python stopgap
//! exactly.
//!
//! @spec projects/agentic-workflow/tech-design/core/specs/score-hook-pretooluse-write-scope.md#logic
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
// CODEGEN-BEGIN

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;

/// Narrow projection of `.aw/config.toml` — only the fields the
/// write-scope hook needs.
#[derive(Debug, Clone, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
pub struct ScoreScopeConfig {
    #[serde(default)]
    pub projects: Vec<ScopeProject>,
}

/// Narrow `[[projects]]` projection.
#[derive(Debug, Clone, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
pub struct ScopeProject {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub td_path: Option<PathBuf>,
    #[serde(default)]
    pub workspaces: Vec<ScopeWorkspace>,
}

/// Narrow `[[projects.workspaces]]` projection — only `paths`.
#[derive(Debug, Clone, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
pub struct ScopeWorkspace {
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Built edit-scope: literal directory prefixes + glob set for free-form
/// `workspaces[].paths` patterns.
#[derive(Debug, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
pub struct AllowedScope {
    /// Directory prefixes. A target path is allowed if it equals any
    /// prefix or starts with `<prefix>/`. Stored without trailing `/`.
    prefixes: Vec<String>,
    /// Compiled glob matchers from `workspaces[].paths`.
    globs: GlobSet,
    /// Original glob patterns for diagnostic output.
    glob_strings: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
impl AllowedScope {
    /// Build the matcher for a single `[[projects]]` entry.
    pub fn for_project(project: &ScopeProject) -> Result<Self> {
        let mut prefixes: Vec<String> = Vec::new();
        for p in [Some(project.path.as_path()), project.td_path.as_deref()]
            .into_iter()
            .flatten()
        {
            let s = p.to_string_lossy().trim_end_matches('/').to_string();
            if !s.is_empty() {
                prefixes.push(s);
            }
        }

        let mut builder = GlobSetBuilder::new();
        let mut glob_strings: Vec<String> = Vec::new();
        for ws in &project.workspaces {
            for pat in &ws.paths {
                let glob = Glob::new(pat).with_context(|| {
                    format!("invalid glob in [[projects.workspaces]].paths: {pat}")
                })?;
                builder.add(glob);
                glob_strings.push(pat.clone());
            }
        }
        let globs = builder.build().context("compiling workspace glob set")?;

        Ok(Self {
            prefixes,
            globs,
            glob_strings,
        })
    }

    /// `true` if `rel` (a path relative to the repo root, posix-form)
    /// is within the allowed scope.
    pub fn contains(&self, rel: &str) -> bool {
        for prefix in &self.prefixes {
            if rel == prefix || rel.starts_with(&format!("{prefix}/")) {
                return true;
            }
        }
        self.globs.is_match(rel)
    }

    /// Human-readable description of the scope, used in block reasons.
    /// Matches the Python stopgap's format: `<prefix>/**, <prefix>/**, <glob>, ...`.
    pub fn describe(&self) -> String {
        if self.prefixes.is_empty() && self.glob_strings.is_empty() {
            return "(no paths configured)".to_string();
        }
        let mut parts: Vec<String> = self.prefixes.iter().map(|p| format!("{p}/**")).collect();
        parts.extend(self.glob_strings.iter().cloned());
        parts.join(", ")
    }
}

/// Load `.aw/config.toml` under `root` and return its scope projection.
/// Returns `Ok(None)` when the file is missing (hook should fail-open).
/// Returns `Err` only on read/parse failure (hook should fail-open with
/// a stderr warning).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
pub fn load_scope(root: &Path) -> Result<Option<ScoreScopeConfig>> {
    let path = root.join(".aw").join("config.toml");
    if !path.is_file() {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let cfg: ScoreScopeConfig =
        toml::from_str(&text).with_context(|| format!("parsing TOML at {}", path.display()))?;
    Ok(Some(cfg))
}

/// Find a `[[projects]]` entry by name.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/path_scope_runtime_source.md#source
pub fn project_by_name<'a>(cfg: &'a ScoreScopeConfig, name: &str) -> Option<&'a ScopeProject> {
    cfg.projects.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_config(td: &TempDir, body: &str) -> PathBuf {
        let dir = td.path().join(".aw");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        fs::write(&path, body).unwrap();
        path
    }

    fn jet_project() -> ScopeProject {
        ScopeProject {
            name: "jet".to_string(),
            path: PathBuf::from("projects/jet"),
            td_path: Some(PathBuf::from(".aw/tech-design/projects/jet")),
            workspaces: vec![ScopeWorkspace {
                paths: vec!["projects/jet/**".to_string()],
            }],
        }
    }

    #[test]
    fn tp1_path_prefix_in_scope() {
        let scope = AllowedScope::for_project(&jet_project()).unwrap();
        assert!(scope.contains("projects/jet/src/lib.rs"));
        assert!(scope.contains("projects/jet"));
    }

    #[test]
    fn tp2_td_path_prefix_in_scope() {
        let scope = AllowedScope::for_project(&jet_project()).unwrap();
        assert!(scope.contains(".aw/tech-design/projects/jet/spec.md"));
        assert!(scope.contains(".aw/tech-design/projects/jet"));
    }

    #[test]
    fn tp3_workspace_glob_in_scope() {
        let mut p = jet_project();
        p.path = PathBuf::from("unrelated");
        p.td_path = None;
        let scope = AllowedScope::for_project(&p).unwrap();
        assert!(scope.contains("projects/jet/sub/file.rs"));
    }

    #[test]
    fn tp4_path_outside_blocks() {
        let scope = AllowedScope::for_project(&jet_project()).unwrap();
        assert!(!scope.contains("crates/mamba/src/lib.rs"));
        assert!(!scope.contains(".aw/tech-design/crates/mamba/spec.md"));
        assert!(!scope.contains("README.md"));
    }

    #[test]
    fn tp7_no_matching_project_entry() {
        let cfg = ScoreScopeConfig {
            projects: vec![jet_project()],
        };
        assert!(project_by_name(&cfg, "mamba").is_none());
        assert!(project_by_name(&cfg, "jet").is_some());
    }

    #[test]
    fn load_missing_returns_none() {
        let td = TempDir::new().unwrap();
        let cfg = load_scope(td.path()).unwrap();
        assert!(cfg.is_none());
    }

    #[test]
    fn load_parses_real_schema() {
        let td = TempDir::new().unwrap();
        write_config(
            &td,
            r#"
[[projects]]
name = "jet"
path = "projects/jet"
td_path = ".aw/tech-design/projects/jet"
label = "project:jet"

[[projects.workspaces]]
name = "jet"
paths = ["projects/jet/**"]
target = "rust"
test_cmd = "cargo test -p jet"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design/surface"
label = "project:agentic-workflow"

[[projects.workspaces]]
name = "agentic-workflow"
paths = ["projects/agentic-workflow/**"]
target = "rust"
"#,
        );
        let cfg = load_scope(td.path()).unwrap().unwrap();
        assert_eq!(cfg.projects.len(), 2);
        let jet = project_by_name(&cfg, "jet").unwrap();
        assert_eq!(jet.path, PathBuf::from("projects/jet"));
        assert_eq!(
            jet.td_path.as_deref(),
            Some(Path::new(".aw/tech-design/projects/jet"))
        );
        assert_eq!(jet.workspaces[0].paths, vec!["projects/jet/**".to_string()]);
    }

    #[test]
    fn load_malformed_toml_errs() {
        let td = TempDir::new().unwrap();
        write_config(&td, "not [valid toml = =");
        let err = load_scope(td.path()).unwrap_err();
        // Parse failure surfaces as Err — caller (hook) translates to
        // fail-open + stderr warning per R4.
        assert!(format!("{err:#}").contains("parsing TOML"));
    }

    #[test]
    fn describe_lists_all_paths() {
        let scope = AllowedScope::for_project(&jet_project()).unwrap();
        let d = scope.describe();
        assert!(d.contains("projects/jet/**"));
        assert!(d.contains(".aw/tech-design/projects/jet/**"));
    }
}
// CODEGEN-END

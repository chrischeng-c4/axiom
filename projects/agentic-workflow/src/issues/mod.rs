// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_preamble_source.md#source
// CODEGEN-BEGIN
//! Issue artifact store — uniform interface over local files, GitHub, GitLab, Jira.
//!
//! # Architecture
//!
//! - [`Issue`] / [`IssueType`] / [`IssueState`] / [`IssueFilter`] — wire
//!   format (also the local issue `{open,closed}/*.md` frontmatter schema)
//! - [`IssueBackend`] — storage trait implemented by each backend
//! - [`backends::LocalBackend`] — reads/writes issue files under a chosen root
//! - [`backends::GitHubBackend`] — shells out to `gh` CLI (read-only MVP)
//! - [`remote_read_cache_backend`] — ephemeral `/tmp` cache for remote reads
//! - [`make_backend`] — factory that picks a backend from resolved kind + repo + host
//! - [`resolve_default_backend`] — read `.aw/config.toml` and return the
//!   `(kind, repo, host)` triple to feed into `make_backend`.
//!
//! # Agent usage
//!
//! All verbs are exposed via the `aw wi` CLI subcommand with a
//! `--json` flag for machine-parseable output. Agents should invoke the
//! CLI rather than linking this module directly.
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_runtime_source.md#source
// CODEGEN-BEGIN
pub mod backend;
pub mod backends;
pub mod labels;
pub mod next_id;
pub mod push_through;
pub mod slug;
pub mod types;

pub use backend::{sync, IssueBackend, SyncReport};
pub use backends::{GitHubBackend, GitLabBackend, LocalBackend};
pub use push_through::push_through;
pub use types::{
    Issue, IssueErrorCode, IssueFilter, IssuePatch, IssuePhase, IssueSection, IssueState,
    IssueType, ShipStatus,
};

use crate::models::SddConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Pick an `IssueBackend` implementation for the given kind.
///
/// Supported kinds:
/// - `"local"` → `LocalBackend` rooted in `/tmp/aw`
/// - `"github"` → `GitHubBackend` talking to the configured repo
/// - `"gitlab"` → `GitLabBackend` talking to the configured repo
/// - `"jira"` → not implemented (returns `Err`)
///
/// `host` is currently threaded only via setters on github/gitlab backends; it
/// is preserved for self-hosted GitLab (`gitlab.example.com`) and GitHub
/// Enterprise URLs. `local` ignores both `repo` and `host`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_runtime_source.md#source
pub fn make_backend(
    kind: &str,
    project_root: &Path,
    repo: Option<String>,
    host: Option<String>,
) -> Result<Box<dyn IssueBackend>> {
    match kind {
        "local" => Ok(Box::new(LocalBackend::from_project_root(project_root))),
        "github" => Ok(Box::new(GitHubBackend::with_host(repo, host))),
        "gitlab" => Ok(Box::new(GitLabBackend::with_host(repo, host))),
        "jira" => anyhow::bail!("Jira issue backend not implemented yet"),
        other => anyhow::bail!("Unknown issue backend: '{}'", other),
    }
}

/// Resolution priority for the default `aw wi` backend (Phase A).
///
/// 1. `[agentic_workflow.issue_platform].type` (+ optional `repo`, `host`)
/// 2. `[agentic_workflow.repo_platform].type` (+ `repo`, `host`) — emits a stderr hint
/// 3. else: error — the user must configure at least one platform
///
/// Only `github` / `gitlab` are accepted; `local` / `jira` / others are
/// rejected so the default targets a remote source of truth (Phase A goal).
/// Internal lifecycle verbs still use `LocalBackend` directly for ephemeral
/// lifecycle issue files; public `aw wi` verbs resolve backend selection from
/// config.
///
/// Returns `(kind, repo, host)` where `repo` and `host` may be `None` when
/// the backend can auto-detect them (e.g. `gh` CLI in a checked-out repo).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_runtime_source.md#source
pub fn resolve_default_backend(
    project_root: &Path,
) -> Result<(String, Option<String>, Option<String>)> {
    use serde::Deserialize;
    use std::fs;

    #[derive(Deserialize)]
    struct ConfigFile {
        agentic_workflow: Option<SddSection>,
    }

    #[derive(Deserialize)]
    struct SddSection {
        #[serde(default)]
        issue_platform: Option<PlatformLite>,
        #[serde(default)]
        repo_platform: Option<RepoPlatformLite>,
    }

    #[derive(Deserialize)]
    struct PlatformLite {
        #[serde(rename = "type")]
        type_: String,
        #[serde(default)]
        repo: Option<String>,
        #[serde(default)]
        host: Option<String>,
    }

    #[derive(Deserialize)]
    struct RepoPlatformLite {
        #[serde(rename = "type")]
        type_: String,
        repo: String,
        #[serde(default)]
        host: Option<String>,
    }

    let config_path = project_root.join(".aw/config.toml");
    if !config_path.exists() {
        anyhow::bail!(
            "no .aw/config.toml found at {} — run `aw init` and configure [agentic_workflow.issue_platform] or [agentic_workflow.repo_platform]",
            project_root.display()
        );
    }

    let content = fs::read_to_string(&config_path)?;
    let parsed: ConfigFile = toml::from_str(&content)?;
    let workflow = parsed.agentic_workflow;

    fn validate_kind(kind: &str) -> Result<()> {
        match kind {
            "github" | "gitlab" => Ok(()),
            other => anyhow::bail!(
                "platform type must be 'github' or 'gitlab', got '{}' \
                 — configure [agentic_workflow.issue_platform] or [agentic_workflow.repo_platform]",
                other
            ),
        }
    }

    if let Some(ref s) = workflow {
        if let Some(ref ip) = s.issue_platform {
            validate_kind(&ip.type_)?;
            // `repo` may be inherited from repo_platform when issue_platform omits it.
            let repo = ip
                .repo
                .clone()
                .or_else(|| s.repo_platform.as_ref().map(|rp| rp.repo.clone()));
            let host = ip
                .host
                .clone()
                .or_else(|| s.repo_platform.as_ref().and_then(|rp| rp.host.clone()));
            return Ok((ip.type_.clone(), repo, host));
        }
    }

    if let Some(ref s) = workflow {
        if let Some(ref rp) = s.repo_platform {
            validate_kind(&rp.type_)?;
            eprintln!(
                "[aw wi] no [agentic_workflow.issue_platform] in .aw/config.toml; \
                 using [agentic_workflow.repo_platform] ({})",
                rp.type_
            );
            return Ok((rp.type_.clone(), Some(rp.repo.clone()), rp.host.clone()));
        }
    }

    anyhow::bail!(
        "must configure [agentic_workflow.repo_platform] (or [agentic_workflow.issue_platform]) in .aw/config.toml \
         — example:\n\n[agentic_workflow.repo_platform]\ntype = \"github\"\nrepo = \"owner/name\"\n"
    )
}

/// Convenience: always returns a local backend for the given project root.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_runtime_source.md#source
pub fn local_backend(project_root: &Path) -> LocalBackend {
    LocalBackend::from_project_root(project_root)
}

/// Directory for best-effort read-through cache of remote issues.
///
/// This cache is intentionally outside the repository.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_runtime_source.md#source
pub fn remote_read_cache_dir(kind: &str, repo: Option<&str>, host: Option<&str>) -> PathBuf {
    let scope = match (host, repo) {
        (Some(host), Some(repo)) => format!("{host}-{repo}"),
        (None, Some(repo)) => repo.to_string(),
        (Some(host), None) => host.to_string(),
        (None, None) => "auto".to_string(),
    };
    crate::shared::workspace::aw_tmp_path()
        .join("issues")
        .join(sanitize_cache_component(&scope))
        .join(sanitize_cache_component(kind))
}

/// Ephemeral LocalBackend for remote read-through cache.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_runtime_source.md#source
pub fn remote_read_cache_backend(
    kind: &str,
    repo: Option<&str>,
    host: Option<&str>,
) -> LocalBackend {
    LocalBackend::at(remote_read_cache_dir(kind, repo, host))
}

fn sanitize_cache_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut last_dash = true;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "default".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Convenience: returns a GitHub backend, repo-detection via the `gh` CLI
/// (no explicit repo provided).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/issues_module_runtime_source.md#source
pub fn github_backend() -> GitHubBackend {
    GitHubBackend::new(None)
}

#[allow(dead_code)]
fn _config_typecheck(_c: &SddConfig) {}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_config(dir: &Path, body: &str) {
        let cfg_dir = dir.join(".aw");
        fs::create_dir_all(&cfg_dir).unwrap();
        fs::write(cfg_dir.join("config.toml"), body).unwrap();
    }

    #[test]
    fn remote_read_cache_dir_is_under_tmp_and_sanitized() {
        let path = remote_read_cache_dir("github", Some("ChrisCheng-C4/cclab"), None);
        assert!(path.starts_with(crate::shared::workspace::aw_tmp_path().join("issues")));
        assert!(path.ends_with("chrischeng-c4-cclab/github"));
    }

    #[test]
    fn remote_read_cache_dir_includes_host_when_present() {
        let path = remote_read_cache_dir(
            "gitlab",
            Some("Group/Sub Project"),
            Some("gitlab.example.com"),
        );
        assert!(path.ends_with("gitlab-example-com-group-sub-project/gitlab"));
    }

    #[test]
    fn issue_platform_set_returns_its_values() {
        let tmp = TempDir::new().unwrap();
        write_config(
            tmp.path(),
            r#"
[agentic_workflow.issue_platform]
type = "github"
repo = "myorg/myrepo"
host = "github.example.com"
"#,
        );
        let (kind, repo, host) = resolve_default_backend(tmp.path()).unwrap();
        assert_eq!(kind, "github");
        assert_eq!(repo.as_deref(), Some("myorg/myrepo"));
        assert_eq!(host.as_deref(), Some("github.example.com"));
    }

    #[test]
    fn fallback_to_repo_platform_when_issue_platform_absent() {
        let tmp = TempDir::new().unwrap();
        write_config(
            tmp.path(),
            r#"
[agentic_workflow.repo_platform]
type = "gitlab"
repo = "group/project"
"#,
        );
        let (kind, repo, host) = resolve_default_backend(tmp.path()).unwrap();
        assert_eq!(kind, "gitlab");
        assert_eq!(repo.as_deref(), Some("group/project"));
        assert_eq!(host, None);
    }

    #[test]
    fn missing_config_file_errors() {
        let tmp = TempDir::new().unwrap();
        let err = resolve_default_backend(tmp.path()).unwrap_err().to_string();
        assert!(err.contains("no .aw/config.toml"), "got: {err}");
    }

    #[test]
    fn missing_platform_sections_errors() {
        let tmp = TempDir::new().unwrap();
        write_config(tmp.path(), r#"version = "0.0.0""#);
        let err = resolve_default_backend(tmp.path()).unwrap_err().to_string();
        assert!(
            err.contains("must configure [agentic_workflow.repo_platform]"),
            "got: {err}"
        );
    }

    #[test]
    fn invalid_type_errors() {
        let tmp = TempDir::new().unwrap();
        write_config(
            tmp.path(),
            r#"
[agentic_workflow.issue_platform]
type = "local"
"#,
        );
        let err = resolve_default_backend(tmp.path()).unwrap_err().to_string();
        assert!(err.contains("must be 'github' or 'gitlab'"), "got: {err}");
    }

    #[test]
    fn issue_platform_inherits_repo_from_repo_platform() {
        let tmp = TempDir::new().unwrap();
        write_config(
            tmp.path(),
            r#"
[agentic_workflow.issue_platform]
type = "github"

[agentic_workflow.repo_platform]
type = "github"
repo = "shared/repo"
"#,
        );
        let (kind, repo, _) = resolve_default_backend(tmp.path()).unwrap();
        assert_eq!(kind, "github");
        assert_eq!(repo.as_deref(), Some("shared/repo"));
    }

    #[test]
    fn issue_platform_host_overrides_repo_platform_host() {
        let tmp = TempDir::new().unwrap();
        write_config(
            tmp.path(),
            r#"
[agentic_workflow.issue_platform]
type = "gitlab"
host = "gitlab.issue.example"

[agentic_workflow.repo_platform]
type = "gitlab"
repo = "g/p"
host = "gitlab.repo.example"
"#,
        );
        let (_, _, host) = resolve_default_backend(tmp.path()).unwrap();
        assert_eq!(host.as_deref(), Some("gitlab.issue.example"));
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Args)]
/// Args for `aw td lock --project <project>`.
/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub struct TdLockArgs {
    /// Check the lock without rewriting it. Exits non-zero when missing or stale.
    #[arg(long)]
    pub check: bool,
    /// Show current lock status without rewriting it.
    #[arg(long)]
    pub show: bool,
    /// Emit JSON status.
    #[arg(long)]
    pub json: bool,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TdLockStatus {
    pub project: String,
    pub td_path: String,
    pub lock_path: String,
    pub status: TdLockState,
    pub clean: bool,
    pub current_digest: String,
    pub locked_digest: Option<String>,
    pub file_count: usize,
    pub changed: Vec<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub message: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl TdLockStatus {
    pub fn ready_fixture(project: &str) -> Self {
        Self {
            project: project.to_string(),
            td_path: format!("projects/{project}/tech-design"),
            lock_path: format!("projects/{project}/tech-design/td.lock"),
            status: TdLockState::Locked,
            clean: true,
            current_digest: "sha256:fixture".to_string(),
            locked_digest: Some("sha256:fixture".to_string()),
            file_count: 1,
            changed: Vec::new(),
            added: Vec::new(),
            removed: Vec::new(),
            message: "td lock clean".to_string(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TdLockState {
    Locked,
    Missing,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TdLockFile {
    version: u8,
    project: String,
    td_path: String,
    generated_at: String,
    digest: String,
    files: Vec<TdLockEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct TdLockEntry {
    path: String,
    digest: String,
}

#[derive(Debug)]
struct TdLockTarget {
    project: String,
    td_path: PathBuf,
    td_path_display: String,
    lock_path: PathBuf,
    lock_path_display: String,
}

#[derive(Debug, Deserialize)]
struct TdLockConfig {
    #[serde(default)]
    projects: Vec<TdLockProject>,
}

#[derive(Debug, Deserialize)]
struct TdLockProject {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl TdLockProject {
    fn matches(&self, requested: &str) -> bool {
        self.name == requested || self.aliases.iter().any(|alias| alias == requested)
    }
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn run(project: Option<&str>, args: TdLockArgs) -> Result<()> {
    let project = project.ok_or_else(|| anyhow::anyhow!("td lock requires --project <project>"))?;
    if args.check || args.show {
        let status = check_project_td_lock(project)?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&status)?);
        } else {
            print_status(&status);
        }
        if args.check && !status.clean {
            anyhow::bail!("{}", status.message);
        }
        return Ok(());
    }

    let (status, wrote) = write_project_td_lock(project)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!(
            "td lock {}: {} {} ({} file(s), {})",
            status.project,
            if wrote { "wrote" } else { "already clean" },
            status.lock_path,
            status.file_count,
            status.current_digest
        );
    }
    Ok(())
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub fn check_project_td_lock(project: &str) -> Result<TdLockStatus> {
    let project_root = crate::find_project_root()?;
    check_project_td_lock_at_root(&project_root, project)
}

fn write_project_td_lock(project: &str) -> Result<(TdLockStatus, bool)> {
    let project_root = crate::find_project_root()?;
    let target = resolve_td_lock_target(&project_root, project)?;
    if target.lock_path.is_file() {
        let status = check_project_td_lock_at_root(&project_root, project)?;
        if status.clean {
            return Ok((status, false));
        }
    }
    let snapshot = snapshot_td_root(&target.td_path)?;
    let lock = TdLockFile {
        version: 1,
        project: target.project.clone(),
        td_path: target.td_path_display.clone(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        digest: snapshot.digest,
        files: snapshot.files,
    };
    let encoded = toml::to_string_pretty(&lock).context("serialize td lock")?;
    fs::write(&target.lock_path, encoded)
        .with_context(|| format!("write {}", target.lock_path.display()))?;
    let digest = lock.digest.clone();
    let file_count = lock.files.len();
    Ok((
        status_from_parts(
            &target,
            TdLockState::Locked,
            true,
            digest.clone(),
            Some(digest),
            file_count,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            "td lock clean".to_string(),
        ),
        true,
    ))
}

fn check_project_td_lock_at_root(project_root: &Path, project: &str) -> Result<TdLockStatus> {
    let target = resolve_td_lock_target(project_root, project)?;
    let current = snapshot_td_root(&target.td_path)?;
    if !target.lock_path.is_file() {
        let current_digest = current.digest.clone();
        let file_count = current.files.len();
        return Ok(TdLockStatus {
            project: target.project.clone(),
            td_path: target.td_path_display.clone(),
            lock_path: target.lock_path_display.clone(),
            status: TdLockState::Missing,
            clean: false,
            current_digest,
            locked_digest: None,
            file_count,
            changed: Vec::new(),
            added: Vec::new(),
            removed: Vec::new(),
            message: format!(
                "td lock missing; run `aw td lock --project {}`",
                target.project
            ),
        });
    }

    let lock_content = fs::read_to_string(&target.lock_path)
        .with_context(|| format!("read {}", target.lock_path.display()))?;
    let lock: TdLockFile = toml::from_str(&lock_content)
        .with_context(|| format!("parse {}", target.lock_path.display()))?;
    let (changed, added, removed) = diff_entries(&lock.files, &current.files);
    let metadata_changed = lock.version != 1
        || lock.project != target.project
        || lock.td_path != target.td_path_display;
    if lock.digest == current.digest && !metadata_changed {
        let current_digest = current.digest.clone();
        let locked_digest = lock.digest.clone();
        let file_count = current.files.len();
        return Ok(status_from_parts(
            &target,
            TdLockState::Locked,
            true,
            current_digest,
            Some(locked_digest),
            file_count,
            changed,
            added,
            removed,
            "td lock clean".to_string(),
        ));
    }

    let message = stale_message(
        &target.project,
        metadata_changed,
        &changed,
        &added,
        &removed,
    );
    let current_digest = current.digest.clone();
    let locked_digest = lock.digest.clone();
    let file_count = current.files.len();
    Ok(status_from_parts(
        &target,
        TdLockState::Stale,
        false,
        current_digest,
        Some(locked_digest),
        file_count,
        changed,
        added,
        removed,
        message,
    ))
}

#[allow(clippy::too_many_arguments)]
fn status_from_parts(
    target: &TdLockTarget,
    status: TdLockState,
    clean: bool,
    current_digest: String,
    locked_digest: Option<String>,
    file_count: usize,
    changed: Vec<String>,
    added: Vec<String>,
    removed: Vec<String>,
    message: String,
) -> TdLockStatus {
    TdLockStatus {
        project: target.project.clone(),
        td_path: target.td_path_display.clone(),
        lock_path: target.lock_path_display.clone(),
        status,
        clean,
        current_digest,
        locked_digest,
        file_count,
        changed,
        added,
        removed,
        message,
    }
}

fn stale_message(
    project: &str,
    metadata_changed: bool,
    changed: &[String],
    added: &[String],
    removed: &[String],
) -> String {
    let mut parts = Vec::new();
    if metadata_changed {
        parts.push("metadata changed".to_string());
    }
    if !changed.is_empty() {
        parts.push(format!("{} changed", changed.len()));
    }
    if !added.is_empty() {
        parts.push(format!("{} added", added.len()));
    }
    if !removed.is_empty() {
        parts.push(format!("{} removed", removed.len()));
    }
    if parts.is_empty() {
        parts.push("digest changed".to_string());
    }
    format!(
        "td lock stale ({}); review TD changes, then run `aw td lock --project {project}`",
        parts.join(", ")
    )
}

fn print_status(status: &TdLockStatus) {
    println!("td lock {}: {:?}", status.project, status.status);
    println!("td_path: {}", status.td_path);
    println!("lock_path: {}", status.lock_path);
    println!("current_digest: {}", status.current_digest);
    if let Some(locked_digest) = &status.locked_digest {
        println!("locked_digest: {locked_digest}");
    }
    println!("files: {}", status.file_count);
    if !status.changed.is_empty() {
        println!("changed:");
        for path in &status.changed {
            println!("  {path}");
        }
    }
    if !status.added.is_empty() {
        println!("added:");
        for path in &status.added {
            println!("  {path}");
        }
    }
    if !status.removed.is_empty() {
        println!("removed:");
        for path in &status.removed {
            println!("  {path}");
        }
    }
    println!("{}", status.message);
}

fn resolve_td_lock_target(project_root: &Path, requested: &str) -> Result<TdLockTarget> {
    let config_path = project_root.join(".aw/config.toml");
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: TdLockConfig =
        toml::from_str(&content).with_context(|| format!("parse {}", config_path.display()))?;
    let project = config
        .projects
        .into_iter()
        .find(|project| project.matches(requested))
        .ok_or_else(|| anyhow::anyhow!("project `{requested}` not found in .aw/config.toml"))?;
    let td_path =
        crate::services::project_registry::resolve_td_root_from_config(project_root, &project.name)
            .map(|resolved| PathBuf::from(resolved.root))
            .map_err(|err| anyhow::anyhow!("{}", err.message))?;
    let td_path_display = repo_relative_display(project_root, &td_path);
    if !td_path.is_dir() {
        anyhow::bail!(
            "project `{}` td_path does not exist: {}",
            project.name,
            td_path.display()
        );
    }
    let lock_path = td_path.join("td.lock");
    let lock_path_display = format!("{}/td.lock", td_path_display.trim_end_matches('/'));
    Ok(TdLockTarget {
        project: project.name,
        td_path,
        td_path_display,
        lock_path,
        lock_path_display,
    })
}

fn repo_relative_display(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[derive(Debug)]
struct TdSnapshot {
    digest: String,
    files: Vec<TdLockEntry>,
}

fn snapshot_td_root(td_root: &Path) -> Result<TdSnapshot> {
    let mut files = Vec::new();
    collect_td_files(td_root, td_root, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let digest = root_digest(&files);
    Ok(TdSnapshot { digest, files })
}

fn collect_td_files(root: &Path, current: &Path, files: &mut Vec<TdLockEntry>) -> Result<()> {
    let mut entries = fs::read_dir(current)
        .with_context(|| format!("read td directory {}", current.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("read td directory {}", current.display()))?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("stat {}", path.display()))?;
        if file_type.is_dir() {
            collect_td_files(root, &path, files)?;
            continue;
        }
        if !file_type.is_file()
            || path.file_name().and_then(|name| name.to_str()) == Some("td.lock")
        {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        files.push(TdLockEntry {
            path: rel,
            digest: digest_bytes(&bytes),
        });
    }
    Ok(())
}

fn digest_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn root_digest(files: &[TdLockEntry]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.path.as_bytes());
        hasher.update(b"\0");
        hasher.update(file.digest.as_bytes());
        hasher.update(b"\n");
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn diff_entries(
    locked: &[TdLockEntry],
    current: &[TdLockEntry],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let locked_by_path: BTreeMap<_, _> = locked
        .iter()
        .map(|entry| (entry.path.as_str(), entry.digest.as_str()))
        .collect();
    let current_by_path: BTreeMap<_, _> = current
        .iter()
        .map(|entry| (entry.path.as_str(), entry.digest.as_str()))
        .collect();
    let changed = current_by_path
        .iter()
        .filter_map(|(path, digest)| {
            locked_by_path
                .get(path)
                .filter(|locked_digest| *locked_digest != digest)
                .map(|_| (*path).to_string())
        })
        .collect();
    let added = current_by_path
        .keys()
        .filter(|path| !locked_by_path.contains_key(*path))
        .map(|path| (*path).to_string())
        .collect();
    let removed = locked_by_path
        .keys()
        .filter(|path| !current_by_path.contains_key(*path))
        .map(|path| (*path).to_string())
        .collect();
    (changed, added, removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn snapshot_digest_detects_td_file_changes() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.md"), "one\n");
        let before = snapshot_td_root(tmp.path()).unwrap();

        write(&tmp.path().join("a.md"), "two\n");
        let after = snapshot_td_root(tmp.path()).unwrap();

        assert_ne!(before.digest, after.digest);
        assert_eq!(after.files.len(), 1);
        assert_eq!(after.files[0].path, "a.md");
    }

    #[test]
    fn snapshot_ignores_td_lock_file() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.md"), "one\n");
        let before = snapshot_td_root(tmp.path()).unwrap();

        write(&tmp.path().join("td.lock"), "ignored = true\n");
        let after = snapshot_td_root(tmp.path()).unwrap();

        assert_eq!(before.digest, after.digest);
        assert_eq!(after.files.len(), 1);
    }

    #[test]
    fn lock_target_defaults_to_project_tech_design() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "demo"
aliases = ["d"]
path = "projects/demo"
"#,
        );
        write(
            &tmp.path().join("projects/demo/tech-design/design.md"),
            "design\n",
        );

        let status = check_project_td_lock_at_root(tmp.path(), "d").unwrap();

        assert_eq!(status.project, "demo");
        assert_eq!(status.status, TdLockState::Missing);
        assert_eq!(status.td_path, "projects/demo/tech-design");
        assert_eq!(status.lock_path, "projects/demo/tech-design/td.lock");
    }

    #[test]
    fn diff_entries_reports_changed_added_and_removed() {
        let locked = vec![
            TdLockEntry {
                path: "a.md".to_string(),
                digest: "sha256:1".to_string(),
            },
            TdLockEntry {
                path: "b.md".to_string(),
                digest: "sha256:2".to_string(),
            },
        ];
        let current = vec![
            TdLockEntry {
                path: "a.md".to_string(),
                digest: "sha256:changed".to_string(),
            },
            TdLockEntry {
                path: "c.md".to_string(),
                digest: "sha256:3".to_string(),
            },
        ];

        let (changed, added, removed) = diff_entries(&locked, &current);

        assert_eq!(changed, vec!["a.md"]);
        assert_eq!(added, vec!["c.md"]);
        assert_eq!(removed, vec!["b.md"]);
    }

    #[test]
    fn check_reports_stale_when_locked_td_file_changes() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
td_path = ".aw/tech-design/projects/demo"
"#,
        );
        let td_root = tmp.path().join(".aw/tech-design/projects/demo");
        write(&td_root.join("design.md"), "before\n");
        let snapshot = snapshot_td_root(&td_root).unwrap();
        let lock = TdLockFile {
            version: 1,
            project: "demo".to_string(),
            td_path: ".aw/tech-design/projects/demo".to_string(),
            generated_at: "2026-06-05T00:00:00Z".to_string(),
            digest: snapshot.digest,
            files: snapshot.files,
        };
        write(
            &td_root.join("td.lock"),
            &toml::to_string_pretty(&lock).unwrap(),
        );

        write(&td_root.join("design.md"), "after\n");
        let status = check_project_td_lock_at_root(tmp.path(), "demo").unwrap();

        assert_eq!(status.status, TdLockState::Stale);
        assert!(!status.clean);
        assert_eq!(status.changed, vec!["design.md"]);
    }
}
// CODEGEN-END

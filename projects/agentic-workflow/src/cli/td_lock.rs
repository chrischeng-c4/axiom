// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const TD_LOCK_VERSION: u8 = 2;
const TD_IR_KIND: &str = "td";

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
    pub ir_kind: String,
    pub td_path: String,
    pub lock_path: String,
    pub status: TdLockState,
    pub clean: bool,
    pub source_digest: String,
    pub locked_source_digest: Option<String>,
    pub ir_digest: String,
    pub locked_ir_digest: Option<String>,
    pub current_digest: String,
    pub locked_digest: Option<String>,
    pub file_count: usize,
    pub td_ir_count: usize,
    pub td_ir_error_count: usize,
    pub changed: Vec<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub ir_changed: Vec<String>,
    pub message: String,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl TdLockStatus {
    pub fn ready_fixture(project: &str) -> Self {
        Self {
            project: project.to_string(),
            ir_kind: TD_IR_KIND.to_string(),
            td_path: format!("projects/{project}/tech-design"),
            lock_path: format!("projects/{project}/tech-design/td.lock"),
            status: TdLockState::Locked,
            clean: true,
            source_digest: "sha256:fixture".to_string(),
            locked_source_digest: Some("sha256:fixture".to_string()),
            ir_digest: "sha256:fixture-ir".to_string(),
            locked_ir_digest: Some("sha256:fixture-ir".to_string()),
            current_digest: "sha256:fixture".to_string(),
            locked_digest: Some("sha256:fixture".to_string()),
            file_count: 1,
            td_ir_count: 1,
            td_ir_error_count: 0,
            changed: Vec::new(),
            added: Vec::new(),
            removed: Vec::new(),
            ir_changed: Vec::new(),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ir_kind: Option<String>,
    td_path: String,
    generated_at: String,
    /// Backward-compatible source-tree digest. New readers should prefer
    /// `source_digest`; v1 locks only carried this field.
    digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ir_digest: Option<String>,
    files: Vec<TdLockEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct TdLockEntry {
    path: String,
    digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ir_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parse_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    section_count: Option<usize>,
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

pub(crate) fn check_project_td_lock_for_spec_at_root(
    project_root: &Path,
    spec_path: &Path,
) -> Result<TdLockStatus> {
    let spec_path = if spec_path.is_absolute() {
        spec_path.to_path_buf()
    } else {
        project_root.join(spec_path)
    };
    let config_path = project_root.join(".aw/config.toml");
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: TdLockConfig =
        toml::from_str(&content).with_context(|| format!("parse {}", config_path.display()))?;
    for project in config.projects {
        let td_root = crate::services::project_registry::resolve_td_root_from_config(
            project_root,
            &project.name,
        )
        .map(|resolved| PathBuf::from(resolved.root))
        .map_err(|err| anyhow::anyhow!("{}", err.message))?;
        if spec_path.starts_with(&td_root) {
            return check_project_td_lock_at_root(project_root, &project.name);
        }
    }
    anyhow::bail!(
        "TD spec {} is not under any configured project td_path",
        repo_relative_display(project_root, &spec_path)
    )
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
        version: TD_LOCK_VERSION,
        project: target.project.clone(),
        ir_kind: Some(TD_IR_KIND.to_string()),
        td_path: target.td_path_display.clone(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        digest: snapshot.source_digest.clone(),
        source_digest: Some(snapshot.source_digest.clone()),
        ir_digest: Some(snapshot.ir_digest.clone()),
        files: snapshot.files,
    };
    let encoded = toml::to_string_pretty(&lock).context("serialize td lock")?;
    fs::write(&target.lock_path, encoded)
        .with_context(|| format!("write {}", target.lock_path.display()))?;
    let source_digest = lock.digest.clone();
    let ir_digest = lock
        .ir_digest
        .clone()
        .unwrap_or_else(|| "sha256:missing-td-ir".to_string());
    let file_count = lock.files.len();
    let td_ir_count = lock
        .files
        .iter()
        .filter(|entry| entry.ir_digest.is_some())
        .count();
    let td_ir_error_count = lock
        .files
        .iter()
        .filter(|entry| entry.parse_error.is_some())
        .count();
    Ok((
        status_from_parts(
            &target,
            TdLockState::Locked,
            true,
            source_digest.clone(),
            Some(source_digest),
            ir_digest.clone(),
            Some(ir_digest),
            file_count,
            td_ir_count,
            td_ir_error_count,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            "td lock clean".to_string(),
        ),
        true,
    ))
}

pub(crate) fn check_project_td_lock_at_root(
    project_root: &Path,
    project: &str,
) -> Result<TdLockStatus> {
    let target = resolve_td_lock_target(project_root, project)?;
    let current = snapshot_td_root(&target.td_path)?;
    if !target.lock_path.is_file() {
        let current_digest = current.source_digest.clone();
        let file_count = current.files.len();
        return Ok(TdLockStatus {
            project: target.project.clone(),
            ir_kind: TD_IR_KIND.to_string(),
            td_path: target.td_path_display.clone(),
            lock_path: target.lock_path_display.clone(),
            status: TdLockState::Missing,
            clean: false,
            source_digest: current_digest.clone(),
            locked_source_digest: None,
            ir_digest: current.ir_digest,
            locked_ir_digest: None,
            current_digest,
            locked_digest: None,
            file_count,
            td_ir_count: current
                .files
                .iter()
                .filter(|entry| entry.ir_digest.is_some())
                .count(),
            td_ir_error_count: current
                .files
                .iter()
                .filter(|entry| entry.parse_error.is_some())
                .count(),
            changed: Vec::new(),
            added: Vec::new(),
            removed: Vec::new(),
            ir_changed: Vec::new(),
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
    let ir_changed = diff_ir_entries(&lock.files, &current.files);
    let locked_source_digest = lock
        .source_digest
        .clone()
        .unwrap_or_else(|| lock.digest.clone());
    let locked_ir_digest = lock.ir_digest.clone();
    let metadata_changed = lock.version != TD_LOCK_VERSION
        || lock.project != target.project
        || lock.td_path != target.td_path_display
        || lock.ir_kind.as_deref() != Some(TD_IR_KIND);
    if locked_source_digest == current.source_digest
        && locked_ir_digest.as_deref() == Some(current.ir_digest.as_str())
        && !metadata_changed
    {
        let current_digest = current.source_digest.clone();
        let locked_digest = locked_source_digest.clone();
        let file_count = current.files.len();
        return Ok(status_from_parts(
            &target,
            TdLockState::Locked,
            true,
            current_digest,
            Some(locked_digest),
            current.ir_digest,
            locked_ir_digest,
            file_count,
            current
                .files
                .iter()
                .filter(|entry| entry.ir_digest.is_some())
                .count(),
            current
                .files
                .iter()
                .filter(|entry| entry.parse_error.is_some())
                .count(),
            changed,
            added,
            removed,
            ir_changed,
            "td lock clean".to_string(),
        ));
    }

    let message = stale_message(
        &target.project,
        metadata_changed,
        locked_ir_digest.as_deref() != Some(current.ir_digest.as_str()),
        &changed,
        &added,
        &removed,
        &ir_changed,
    );
    let current_digest = current.source_digest.clone();
    let locked_digest = locked_source_digest.clone();
    let file_count = current.files.len();
    Ok(status_from_parts(
        &target,
        TdLockState::Stale,
        false,
        current_digest,
        Some(locked_digest),
        current.ir_digest,
        locked_ir_digest,
        file_count,
        current
            .files
            .iter()
            .filter(|entry| entry.ir_digest.is_some())
            .count(),
        current
            .files
            .iter()
            .filter(|entry| entry.parse_error.is_some())
            .count(),
        changed,
        added,
        removed,
        ir_changed,
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
    ir_digest: String,
    locked_ir_digest: Option<String>,
    file_count: usize,
    td_ir_count: usize,
    td_ir_error_count: usize,
    changed: Vec<String>,
    added: Vec<String>,
    removed: Vec<String>,
    ir_changed: Vec<String>,
    message: String,
) -> TdLockStatus {
    TdLockStatus {
        project: target.project.clone(),
        ir_kind: TD_IR_KIND.to_string(),
        td_path: target.td_path_display.clone(),
        lock_path: target.lock_path_display.clone(),
        status,
        clean,
        source_digest: current_digest.clone(),
        locked_source_digest: locked_digest.clone(),
        ir_digest,
        locked_ir_digest,
        current_digest,
        locked_digest,
        file_count,
        td_ir_count,
        td_ir_error_count,
        changed,
        added,
        removed,
        ir_changed,
        message,
    }
}

fn stale_message(
    project: &str,
    metadata_changed: bool,
    ir_digest_changed: bool,
    changed: &[String],
    added: &[String],
    removed: &[String],
    ir_changed: &[String],
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
    if !ir_changed.is_empty() {
        parts.push(format!("{} TD IR changed", ir_changed.len()));
    } else if ir_digest_changed {
        parts.push("TD IR digest changed".to_string());
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
    println!("ir_kind: {}", status.ir_kind);
    println!("td_path: {}", status.td_path);
    println!("lock_path: {}", status.lock_path);
    println!("source_digest: {}", status.source_digest);
    if let Some(locked_source_digest) = &status.locked_source_digest {
        println!("locked_source_digest: {locked_source_digest}");
    }
    println!("ir_digest: {}", status.ir_digest);
    if let Some(locked_ir_digest) = &status.locked_ir_digest {
        println!("locked_ir_digest: {locked_ir_digest}");
    }
    println!("files: {}", status.file_count);
    println!(
        "td_ir: {} parsed, {} parse error(s)",
        status.td_ir_count, status.td_ir_error_count
    );
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
    if !status.ir_changed.is_empty() {
        println!("ir_changed:");
        for path in &status.ir_changed {
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
    source_digest: String,
    ir_digest: String,
    files: Vec<TdLockEntry>,
}

fn snapshot_td_root(td_root: &Path) -> Result<TdSnapshot> {
    let mut files = Vec::new();
    collect_td_files(td_root, td_root, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let source_digest = root_digest(&files);
    let ir_digest = root_ir_digest(&files);
    Ok(TdSnapshot {
        source_digest,
        ir_digest,
        files,
    })
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
        let digest = digest_bytes(&bytes);
        let (ir_digest, parse_error, section_count) = td_ir_entry(&bytes);
        files.push(TdLockEntry {
            path: rel,
            digest,
            ir_digest,
            parse_error,
            section_count,
        });
    }
    Ok(())
}

fn td_ir_entry(bytes: &[u8]) -> (Option<String>, Option<String>, Option<usize>) {
    let raw = match std::str::from_utf8(bytes) {
        Ok(raw) => raw,
        Err(err) => {
            return (None, Some(format!("non-utf8 TD source: {err}")), None);
        }
    };
    match crate::td_ast::parse::parse_td_str(raw) {
        Ok(ast) => {
            let section_count = ast.sections.len();
            match serde_json::to_vec(&ast) {
                Ok(bytes) => (Some(digest_bytes(&bytes)), None, Some(section_count)),
                Err(err) => (None, Some(format!("serialize TD IR failed: {err}")), None),
            }
        }
        Err(err) => (
            None,
            Some(format!(
                "{}:{}-{}: {}",
                err.section_type.as_str(),
                err.line_start,
                err.line_end,
                err.message
            )),
            None,
        ),
    }
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

fn root_ir_digest(files: &[TdLockEntry]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.path.as_bytes());
        hasher.update(b"\0");
        if let Some(ir_digest) = &file.ir_digest {
            hasher.update(b"ir:");
            hasher.update(ir_digest.as_bytes());
        } else if let Some(parse_error) = &file.parse_error {
            hasher.update(b"parse-error:");
            hasher.update(parse_error.as_bytes());
        } else {
            hasher.update(b"ir:none");
        }
        hasher.update(b"\0");
        if let Some(section_count) = file.section_count {
            hasher.update(section_count.to_string().as_bytes());
        }
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

fn diff_ir_entries(locked: &[TdLockEntry], current: &[TdLockEntry]) -> Vec<String> {
    let locked_by_path: BTreeMap<_, _> = locked
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    current
        .iter()
        .filter_map(|entry| {
            locked_by_path.get(entry.path.as_str()).and_then(|locked| {
                if locked.ir_digest != entry.ir_digest
                    || locked.parse_error != entry.parse_error
                    || locked.section_count != entry.section_count
                {
                    Some(entry.path.clone())
                } else {
                    None
                }
            })
        })
        .collect()
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

    fn lock_entry(path: &str, digest: &str) -> TdLockEntry {
        TdLockEntry {
            path: path.to_string(),
            digest: digest.to_string(),
            ir_digest: Some(format!("{digest}:ir")),
            parse_error: None,
            section_count: Some(1),
        }
    }

    #[test]
    fn snapshot_digest_detects_td_file_changes() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.md"), "one\n");
        let before = snapshot_td_root(tmp.path()).unwrap();

        write(&tmp.path().join("a.md"), "two\n");
        let after = snapshot_td_root(tmp.path()).unwrap();

        assert_ne!(before.source_digest, after.source_digest);
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

        assert_eq!(before.source_digest, after.source_digest);
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
    fn lock_status_can_be_resolved_from_spec_path() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join(".aw/config.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
"#,
        );
        let spec = tmp.path().join("projects/demo/tech-design/design.md");
        write(&spec, "design\n");

        let status = check_project_td_lock_for_spec_at_root(tmp.path(), &spec).unwrap();

        assert_eq!(status.project, "demo");
        assert_eq!(status.status, TdLockState::Missing);
        assert_eq!(status.lock_path, "projects/demo/tech-design/td.lock");
    }

    #[test]
    fn diff_entries_reports_changed_added_and_removed() {
        let locked = vec![
            lock_entry("a.md", "sha256:1"),
            lock_entry("b.md", "sha256:2"),
        ];
        let current = vec![
            lock_entry("a.md", "sha256:changed"),
            lock_entry("c.md", "sha256:3"),
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
            version: TD_LOCK_VERSION,
            project: "demo".to_string(),
            ir_kind: Some(TD_IR_KIND.to_string()),
            td_path: ".aw/tech-design/projects/demo".to_string(),
            generated_at: "2026-06-05T00:00:00Z".to_string(),
            digest: snapshot.source_digest.clone(),
            source_digest: Some(snapshot.source_digest),
            ir_digest: Some(snapshot.ir_digest),
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

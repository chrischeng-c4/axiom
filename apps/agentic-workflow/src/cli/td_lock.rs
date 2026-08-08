// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const TD_LOCK_VERSION: u8 = 2;
const TD_IR_KIND: &str = "td";

#[derive(Debug, Args)]
/// Args for `aw td lock --project <project>`.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub struct TdLockArgs {
    /// Check the lock without rewriting it. Exits non-zero when missing or stale.
    #[arg(long)]
    pub check: bool,
    /// Show current lock status without rewriting it.
    #[arg(long)]
    pub show: bool,
    /// Commit only the generated lock path. Without this flag, the command
    /// writes the lock and leaves staging and committing to the caller.
    #[arg(long, conflicts_with_all = ["check", "show"])]
    pub commit: bool,
    /// Emit JSON status.
    #[arg(long)]
    pub json: bool,
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
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

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
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

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
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
    artifact_model: crate::models::project::ProjectArtifactModel,
    td_path: PathBuf,
    td_path_display: String,
    lock_path: PathBuf,
    lock_path_display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TdLockWriteAction {
    Wrote,
    WroteAndCommitted,
    RecoveredAndCommitted,
    AlreadyClean,
}

#[derive(Debug)]
struct TdLockWriteResult {
    status: TdLockStatus,
    action: TdLockWriteAction,
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

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl TdLockProject {
    fn matches(&self, requested: &str) -> bool {
        self.name == requested || self.aliases.iter().any(|alias| alias == requested)
    }
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
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

    let result = write_project_td_lock(project, args.commit)?;
    let status = result.status;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        let action = match result.action {
            TdLockWriteAction::Wrote => "wrote (not committed)",
            TdLockWriteAction::WroteAndCommitted => "wrote and committed",
            TdLockWriteAction::RecoveredAndCommitted => "recovered and committed",
            TdLockWriteAction::AlreadyClean => "already clean",
        };
        println!(
            "td lock {}: {} {} ({} file(s), {})",
            status.project, action, status.lock_path, status.file_count, status.current_digest
        );
    }
    Ok(())
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
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
    let config_path = project_root.join("aw.toml");
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

fn write_project_td_lock(project: &str, commit: bool) -> Result<TdLockWriteResult> {
    let project_root = crate::find_project_root()?;
    write_project_td_lock_at_root(&project_root, project, commit)
}

fn write_project_td_lock_at_root(
    project_root: &Path,
    project: &str,
    commit: bool,
) -> Result<TdLockWriteResult> {
    let target = resolve_td_lock_target(project_root, project)?;
    let lock_path = preflight_repo_relative_td_lock_path(project_root, &target.lock_path)?;
    let (status, wrote) = write_project_td_lock_file_at_root(project_root, &target)?;
    let committed = if commit {
        commit_td_lock_update(project_root, &target, &lock_path)?
    } else {
        false
    };
    if commit && wrote && !committed {
        anyhow::bail!(
            "td lock wrote {} but could not commit the generated lock",
            target.lock_path_display
        );
    }
    let action = match (wrote, committed) {
        (true, true) => TdLockWriteAction::WroteAndCommitted,
        (false, true) => TdLockWriteAction::RecoveredAndCommitted,
        (false, false) => TdLockWriteAction::AlreadyClean,
        (true, false) => TdLockWriteAction::Wrote,
    };
    Ok(TdLockWriteResult { status, action })
}

fn write_project_td_lock_file_at_root(
    project_root: &Path,
    target: &TdLockTarget,
) -> Result<(TdLockStatus, bool)> {
    if target.lock_path.is_file() {
        let status = check_project_td_lock_at_root(project_root, &target.project)?;
        if status.clean {
            return Ok((status, false));
        }
    }
    let snapshot = snapshot_td_lock_target(&target)?;
    let td_ir_count = snapshot.td_ir_count;
    let td_ir_error_count = snapshot.td_ir_error_count;
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

/// Write a TD lock snapshot without creating the lifecycle commit used by the
/// public CLI.  Hermetic graph-verification tests use this to prepare a
/// committed-equivalent lock without depending on a git fixture.
pub(crate) fn write_project_td_lock_snapshot_at_root(
    project_root: &Path,
    project: &str,
) -> Result<TdLockStatus> {
    let target = resolve_td_lock_target(project_root, project)?;
    let (status, _) = write_project_td_lock_file_at_root(project_root, &target)?;
    Ok(status)
}

/// Commit only the generated lock while deliberately allowing unrelated index
/// state. The shared lifecycle commit helper rejects any pre-existing staged
/// change, so this lock handoff needs the narrower `git commit --only` policy.
fn commit_td_lock_update(
    project_root: &Path,
    target: &TdLockTarget,
    lock_path: &Path,
) -> Result<bool> {
    if !crate::git::is_git_repo(project_root) {
        return Ok(false);
    }
    let current_lock_path = preflight_repo_relative_td_lock_path(project_root, &target.lock_path)?;
    if current_lock_path != lock_path {
        anyhow::bail!(
            "generated TD lock path changed between write and commit: {}",
            target.lock_path_display
        );
    }

    crate::git::stage_paths(project_root, &[lock_path], true)?;

    if !crate::git::has_staged_changes_for_paths(project_root, &[lock_path], true)? {
        return Ok(false);
    }

    let message = format!(
        "td-lock({}) — update TD IR snapshot\n\nTD-Lock-Project: {}\nTD-Lock-Path: {}",
        target.project, target.project, target.lock_path_display
    );
    crate::git::commit_only_paths(project_root, &[lock_path], &message, true)?;

    let git = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;

    let lock_status = Command::new(&git)
        .arg("--literal-pathspecs")
        .arg("-C")
        .arg(project_root)
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--",
        ])
        .arg(&lock_path)
        .output()
        .context("git status generated TD lock")?;
    if !lock_status.status.success() {
        anyhow::bail!(
            "git status generated TD lock failed: {}",
            String::from_utf8_lossy(&lock_status.stderr).trim()
        );
    }
    if !lock_status.stdout.is_empty() {
        anyhow::bail!(
            "generated TD lock remained dirty after commit: {}",
            target.lock_path_display
        );
    }
    Ok(true)
}

fn preflight_repo_relative_td_lock_path(project_root: &Path, lock_path: &Path) -> Result<PathBuf> {
    let canonical_root = fs::canonicalize(project_root)
        .with_context(|| format!("resolve repository root {}", project_root.display()))?;
    let relative = lock_path.strip_prefix(project_root).map_err(|_| {
        anyhow::anyhow!(
            "generated TD lock is outside repository root: {}",
            lock_path.display()
        )
    })?;
    let file_name = relative.file_name().ok_or_else(|| {
        anyhow::anyhow!(
            "generated TD lock path has no file name: {}",
            lock_path.display()
        )
    })?;
    let relative_parent = relative.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "generated TD lock path has no parent: {}",
            lock_path.display()
        )
    })?;
    let mut lexical_parent = project_root.to_path_buf();
    for component in relative_parent.components() {
        let Component::Normal(component) = component else {
            anyhow::bail!(
                "generated TD lock path contains a non-normal repository component: {}",
                lock_path.display()
            );
        };
        lexical_parent.push(component);
        let metadata = fs::symlink_metadata(&lexical_parent)
            .with_context(|| format!("stat TD lock parent {}", lexical_parent.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            anyhow::bail!(
                "generated TD lock parent must be a real repository directory: {}",
                lexical_parent.display()
            );
        }
    }
    let canonical_parent = fs::canonicalize(&lexical_parent)
        .with_context(|| format!("resolve TD lock parent {}", lexical_parent.display()))?;
    if !canonical_parent.starts_with(&canonical_root) {
        anyhow::bail!(
            "generated TD lock parent is outside repository root: {}",
            lexical_parent.display()
        );
    }
    let expected_lock = canonical_parent.join(file_name);
    match fs::symlink_metadata(lock_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                anyhow::bail!(
                    "generated TD lock must be a non-symlink regular file inside the repository: {}",
                    lock_path.display()
                );
            }
            let canonical_lock = fs::canonicalize(lock_path)
                .with_context(|| format!("resolve generated TD lock {}", lock_path.display()))?;
            if canonical_lock != expected_lock || !canonical_lock.starts_with(&canonical_root) {
                anyhow::bail!(
                    "generated TD lock does not resolve to its exact repository path: {}",
                    lock_path.display()
                );
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("stat generated TD lock {}", lock_path.display()));
        }
    }
    Ok(relative.to_path_buf())
}

pub(crate) fn check_project_td_lock_at_root(
    project_root: &Path,
    project: &str,
) -> Result<TdLockStatus> {
    let target = resolve_td_lock_target(project_root, project)?;
    let current = snapshot_td_lock_target(&target)?;
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
            td_ir_count: current.td_ir_count,
            td_ir_error_count: current.td_ir_error_count,
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
            current.td_ir_count,
            current.td_ir_error_count,
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
        current.td_ir_count,
        current.td_ir_error_count,
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
    let config_path = project_root.join("aw.toml");
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: TdLockConfig =
        toml::from_str(&content).with_context(|| format!("parse {}", config_path.display()))?;
    let project = config
        .projects
        .into_iter()
        .find(|project| project.matches(requested))
        .ok_or_else(|| anyhow::anyhow!("project `{requested}` not found in aw.toml"))?;
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
    let artifact_model =
        crate::services::project_registry::resolve_project_config_row(project_root, &project.name)?
            .effective_artifact_model();
    Ok(TdLockTarget {
        project: project.name,
        artifact_model,
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
    td_ir_count: usize,
    td_ir_error_count: usize,
}

fn snapshot_td_root(td_root: &Path) -> Result<TdSnapshot> {
    let mut files = Vec::new();
    collect_td_files_with_policy(td_root, td_root, &mut files, false)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let source_digest = root_digest(&files);
    let ir_digest = root_ir_digest(&files);
    let td_ir_count = files
        .iter()
        .filter(|entry| entry.ir_digest.is_some())
        .count();
    let td_ir_error_count = files
        .iter()
        .filter(|entry| entry.parse_error.is_some())
        .count();
    Ok(TdSnapshot {
        source_digest,
        ir_digest,
        files,
        td_ir_count,
        td_ir_error_count,
    })
}

fn snapshot_td_lock_target(target: &TdLockTarget) -> Result<TdSnapshot> {
    let python_v1 = target.artifact_model == crate::models::project::ProjectArtifactModel::PythonV1;
    if python_v1 {
        let ir = crate::services::python_td::compile_python_td_project(&target.td_path)?;
        let mut compiler_inputs = ir
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<BTreeSet<_>>();
        for module in &ir.modules {
            if let Some(crate::services::python_td::PythonTdCodegen::OpenApi(openapi)) =
                module.codegen.as_ref()
            {
                compiler_inputs.insert(openapi.document_path.clone());
            }
        }
        // Python TD execution is frozen to its project manifest and uv lock.
        // Keep both files in the durable TD snapshot so dependency drift cannot
        // preserve a green semantic IR digest while changing the executable
        // reference product underneath it.
        for dependency_file in ["pyproject.toml", "uv.lock"] {
            let path = target.td_path.join(dependency_file);
            if !path.is_file() {
                anyhow::bail!(
                    "Python TD lock requires {}; run `uv lock --project {}`",
                    dependency_file,
                    target.td_path.display()
                );
            }
            compiler_inputs.insert(dependency_file.to_string());
        }
        let files = compiler_inputs
            .into_iter()
            .map(|relative| {
                let path = target.td_path.join(&relative);
                let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
                Ok(TdLockEntry {
                    path: relative,
                    digest: digest_bytes(&bytes),
                    ir_digest: None,
                    parse_error: None,
                    section_count: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let td_ir_count = ir.modules.len();
        return Ok(TdSnapshot {
            source_digest: root_digest(&files),
            ir_digest: ir.semantic_digest,
            files,
            td_ir_count,
            td_ir_error_count: 0,
        });
    }
    snapshot_td_root(&target.td_path)
}

#[cfg(test)]
fn collect_td_files(root: &Path, current: &Path, files: &mut Vec<TdLockEntry>) -> Result<()> {
    collect_td_files_with_policy(root, current, files, false)
}

fn collect_td_files_with_policy(
    root: &Path,
    current: &Path,
    files: &mut Vec<TdLockEntry>,
    ignore_python_runtime_artifacts: bool,
) -> Result<()> {
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
            if ignore_python_runtime_artifacts
                && matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some(
                        "__pycache__"
                            | ".venv"
                            | "venv"
                            | ".pytest_cache"
                            | ".mypy_cache"
                            | ".ruff_cache"
                            | ".tox"
                    )
                )
            {
                continue;
            }
            collect_td_files_with_policy(root, &path, files, ignore_python_runtime_artifacts)?;
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

    fn git_available() -> bool {
        crate::git::find_git_bin().is_some()
    }

    fn git_output(root: &Path, args: &[&str]) -> std::process::Output {
        let git = crate::git::find_git_bin().expect("git binary");
        let output = Command::new(git)
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("git {args:?}: {error}"));
        assert!(
            output.status.success(),
            "git {args:?} failed:\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn git_stdout(root: &Path, args: &[&str]) -> String {
        String::from_utf8(git_output(root, args).stdout).expect("utf-8 git stdout")
    }

    fn init_git_repo(root: &Path) {
        git_output(root, &["init", "-q"]);
        git_output(root, &["config", "user.email", "aw-test@example.com"]);
        git_output(root, &["config", "user.name", "AW Test"]);
    }

    fn write_python_td_environment(td_root: &Path) {
        write(
            &td_root.join("pyproject.toml"),
            "[project]\nname = \"demo-tech-design\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\n",
        );
        write(
            &td_root.join("uv.lock"),
            "version = 1\nrevision = 3\nrequires-python = \">=3.11\"\n",
        );
    }

    fn write_td_lock_repo(root: &Path) {
        write(
            &root.join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
"#,
        );
        write(
            &root.join("projects/demo/tech-design/design.md"),
            "# Demo design\n",
        );
        write(
            &root.join("projects/demo/tech-design/src/demo/design.py"),
            "__aw_artifact_id__ = \"artifact:demo/design\"\n\ndef design() -> None:\n    pass\n",
        );
        write_python_td_environment(&root.join("projects/demo/tech-design"));
        write(&root.join("notes/staged.txt"), "staged baseline\n");
        write(&root.join("notes/unstaged.txt"), "unstaged baseline\n");
        git_output(root, &["add", "."]);
        git_output(root, &["commit", "-m", "bootstrap TD lock fixture"]);
    }

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
    fn td_lock_default_writes_without_staging_or_committing() {
        if !git_available() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        write_td_lock_repo(root);
        let head_before = git_stdout(root, &["rev-parse", "HEAD"]);

        let result = write_project_td_lock_at_root(root, "demo", false).unwrap();

        assert_eq!(result.action, TdLockWriteAction::Wrote);
        assert_eq!(git_stdout(root, &["rev-parse", "HEAD"]), head_before);
        assert_eq!(
            git_stdout(
                root,
                &[
                    "status",
                    "--porcelain=v1",
                    "--untracked-files=all",
                    "--",
                    "projects/demo/tech-design/td.lock",
                ],
            )
            .trim(),
            "?? projects/demo/tech-design/td.lock"
        );
    }

    #[test]
    fn td_lock_commit_preserves_unrelated_staged_unstaged_and_untracked_state() {
        if !git_available() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        write_td_lock_repo(root);

        write(&root.join("notes/staged.txt"), "staged change\n");
        git_output(root, &["add", "notes/staged.txt"]);
        write(&root.join("notes/unstaged.txt"), "unstaged change\n");
        write(&root.join("notes/untracked.txt"), "untracked change\n");
        let unrelated_before = git_output(
            root,
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )
        .stdout;
        let head_before = git_stdout(root, &["rev-parse", "HEAD"]);

        let result = write_project_td_lock_at_root(root, "demo", true).unwrap();

        assert_eq!(result.action, TdLockWriteAction::WroteAndCommitted);
        assert!(result.status.clean);
        assert_ne!(git_stdout(root, &["rev-parse", "HEAD"]), head_before);
        assert_eq!(
            git_stdout(root, &["show", "--format=", "--name-only", "HEAD"])
                .lines()
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>(),
            vec!["projects/demo/tech-design/td.lock"]
        );
        assert_eq!(
            git_stdout(root, &["log", "-1", "--format=%s"]).trim(),
            "td-lock(demo) — update TD IR snapshot"
        );
        let message = git_stdout(root, &["log", "-1", "--format=%B"]);
        assert!(message.contains("TD-Lock-Project: demo"));
        assert!(message.contains("TD-Lock-Path: projects/demo/tech-design/td.lock"));
        assert_eq!(
            git_output(
                root,
                &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            )
            .stdout,
            unrelated_before,
            "the lock-only commit must preserve every unrelated index/worktree byte"
        );

        let committed_head = git_stdout(root, &["rev-parse", "HEAD"]);
        let repeat = write_project_td_lock_at_root(root, "demo", true).unwrap();
        assert_eq!(repeat.action, TdLockWriteAction::AlreadyClean);
        assert_eq!(git_stdout(root, &["rev-parse", "HEAD"]), committed_head);
        assert_eq!(
            git_output(
                root,
                &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            )
            .stdout,
            unrelated_before,
            "an already-clean lock must remain a no-op even with unrelated staged state"
        );
    }

    #[test]
    fn td_lock_commit_recovers_semantically_clean_uncommitted_lock_once() {
        if !git_available() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        init_git_repo(root);
        write_td_lock_repo(root);
        let target = resolve_td_lock_target(root, "demo").unwrap();
        let (status, wrote) = write_project_td_lock_file_at_root(root, &target).unwrap();
        assert!(wrote);
        assert!(status.clean);
        assert!(
            git_stdout(
                root,
                &[
                    "status",
                    "--porcelain=v1",
                    "--untracked-files=all",
                    "--",
                    "projects/demo/tech-design/td.lock",
                ],
            )
            .starts_with("?? "),
            "fixture must reproduce the old semantically-clean uncommitted lock"
        );
        let head_before = git_stdout(root, &["rev-parse", "HEAD"]);

        let recovered = write_project_td_lock_at_root(root, "demo", true).unwrap();

        assert_eq!(recovered.action, TdLockWriteAction::RecoveredAndCommitted);
        assert_ne!(git_stdout(root, &["rev-parse", "HEAD"]), head_before);
        assert_eq!(
            git_stdout(root, &["show", "--format=", "--name-only", "HEAD"])
                .lines()
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>(),
            vec!["projects/demo/tech-design/td.lock"]
        );
        let recovered_head = git_stdout(root, &["rev-parse", "HEAD"]);
        let repeat = write_project_td_lock_at_root(root, "demo", true).unwrap();
        assert_eq!(repeat.action, TdLockWriteAction::AlreadyClean);
        assert_eq!(git_stdout(root, &["rev-parse", "HEAD"]), recovered_head);
    }

    #[cfg(unix)]
    #[test]
    fn td_lock_commit_preflight_rejects_external_td_path_symlink_without_mutation() {
        use std::os::unix::fs::symlink;

        if !git_available() {
            return;
        }
        let repo = TempDir::new().unwrap();
        let external = TempDir::new().unwrap();
        let root = repo.path();
        init_git_repo(root);
        write(
            &root.join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
td_path = "projects/demo/tech-design"
"#,
        );
        fs::create_dir_all(root.join("projects/demo")).unwrap();
        let external_design = external.path().join("design.md");
        write(&external_design, "external design sentinel\n");
        symlink(external.path(), root.join("projects/demo/tech-design")).unwrap();
        git_output(root, &["add", "."]);
        git_output(root, &["commit", "-m", "bootstrap external TD path"]);
        let external_before = fs::read(&external_design).unwrap();
        let head_before = git_stdout(root, &["rev-parse", "HEAD"]);
        let status_before = git_output(
            root,
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )
        .stdout;

        let error = write_project_td_lock_at_root(root, "demo", true).unwrap_err();

        assert!(
            error.to_string().contains("outside repository root")
                || error.to_string().contains("real repository directory"),
            "unexpected preflight error: {error:#}"
        );
        assert_eq!(fs::read(&external_design).unwrap(), external_before);
        assert!(!external.path().join("td.lock").exists());
        assert_eq!(git_stdout(root, &["rev-parse", "HEAD"]), head_before);
        assert_eq!(
            git_output(
                root,
                &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            )
            .stdout,
            status_before
        );
    }

    #[cfg(unix)]
    #[test]
    fn td_lock_commit_preflight_rejects_external_lock_symlink_without_mutation() {
        use std::os::unix::fs::symlink;

        if !git_available() {
            return;
        }
        let repo = TempDir::new().unwrap();
        let external = TempDir::new().unwrap();
        let root = repo.path();
        init_git_repo(root);
        write(
            &root.join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
"#,
        );
        let td_root = root.join("projects/demo/tech-design");
        write(&td_root.join("design.md"), "current design\n");
        let stale_lock = TdLockFile {
            version: TD_LOCK_VERSION,
            project: "demo".to_string(),
            ir_kind: Some(TD_IR_KIND.to_string()),
            td_path: "projects/demo/tech-design".to_string(),
            generated_at: "2026-07-14T00:00:00Z".to_string(),
            digest: "sha256:stale".to_string(),
            source_digest: Some("sha256:stale".to_string()),
            ir_digest: Some("sha256:stale-ir".to_string()),
            files: Vec::new(),
        };
        let external_lock = external.path().join("td.lock");
        fs::write(&external_lock, toml::to_string_pretty(&stale_lock).unwrap()).unwrap();
        symlink(&external_lock, td_root.join("td.lock")).unwrap();
        git_output(root, &["add", "."]);
        git_output(root, &["commit", "-m", "bootstrap external TD lock leaf"]);
        let external_before = fs::read(&external_lock).unwrap();
        let head_before = git_stdout(root, &["rev-parse", "HEAD"]);
        let status_before = git_output(
            root,
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )
        .stdout;

        let error = write_project_td_lock_at_root(root, "demo", true).unwrap_err();

        assert!(
            error.to_string().contains("non-symlink regular file"),
            "unexpected preflight error: {error:#}"
        );
        assert_eq!(fs::read(&external_lock).unwrap(), external_before);
        assert_eq!(git_stdout(root, &["rev-parse", "HEAD"]), head_before);
        assert_eq!(
            git_output(
                root,
                &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            )
            .stdout,
            status_before
        );
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
    fn python_td_canonical_routing_lock_includes_executable_dependency_inputs() {
        let root = TempDir::new().unwrap();
        write(
            &root.path().join("aw.toml"),
            "[[projects]]\nname = \"demo\"\npath = \"projects/demo\"\nartifact_model = \"python-v1\"\n",
        );
        let td_root = root
            .path()
            .join("projects/demo/tech-design/src/demo/domain");
        std::fs::create_dir_all(&td_root).unwrap();
        write(
            &td_root.join("invoice.py"),
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\n@openapi_client(source=\"openapi.json\", python=\"python-3.11\", typescript=\"typescript-5.0\", rust=\"rust-2021\")\nclass InvoiceClient:\n    pass\n\ndef issue_invoice() -> None:\n    pass\n",
        );
        write(
            &root
                .path()
                .join("projects/demo/tech-design/openapi.json"),
            "{\"openapi\":\"3.1.0\",\"info\":{\"title\":\"Invoice\",\"version\":\"1\"},\"paths\":{}}\n",
        );
        write(
            &root
                .path()
                .join("projects/demo/tech-design/pyproject.toml"),
            "[project]\nname = \"demo-tech-design\"\nversion = \"0.1.0\"\nrequires-python = \">=3.11\"\n",
        );
        write(
            &root.path().join("projects/demo/tech-design/uv.lock"),
            "version = 1\nrevision = 3\nrequires-python = \">=3.11\"\n",
        );
        write(
            &td_root.join("__pycache__/invoice.cpython-313.pyc"),
            "runtime cache\n",
        );
        write(
            &root
                .path()
                .join("projects/demo/tech-design/legacy-design.md"),
            "retired Markdown input\n",
        );

        let target = resolve_td_lock_target(root.path(), "demo").unwrap();
        let before = snapshot_td_lock_target(&target).unwrap();
        let compiled =
            crate::services::python_td::compile_python_td_project(&target.td_path).unwrap();

        assert_eq!(before.ir_digest, compiled.semantic_digest);
        assert!(before.files.iter().all(|entry| entry.parse_error.is_none()));
        assert!(before.files.iter().all(|entry| entry.ir_digest.is_none()));
        assert_eq!(before.files.len(), 4);
        assert_eq!(before.td_ir_count, 1);
        assert_eq!(before.td_ir_error_count, 0);
        assert_eq!(before.files[0].path, "openapi.json");
        assert_eq!(before.files[1].path, "pyproject.toml");
        assert_eq!(before.files[2].path, "src/demo/domain/invoice.py");
        assert_eq!(before.files[3].path, "uv.lock");

        write(
            &root
                .path()
                .join("projects/demo/tech-design/legacy-design.md"),
            "changed retired Markdown input\n",
        );
        let after_markdown = snapshot_td_lock_target(&target).unwrap();
        assert_eq!(before.source_digest, after_markdown.source_digest);
        assert_eq!(before.ir_digest, after_markdown.ir_digest);

        write(
            &root
                .path()
                .join("projects/demo/tech-design/openapi.json"),
            "{\"openapi\":\"3.1.0\",\"info\":{\"title\":\"Invoice v2\",\"version\":\"1\"},\"paths\":{}}\n",
        );
        let after_openapi = snapshot_td_lock_target(&target).unwrap();
        assert_ne!(before.source_digest, after_openapi.source_digest);
        assert_ne!(before.ir_digest, after_openapi.ir_digest);

        write(
            &root.path().join("projects/demo/tech-design/uv.lock"),
            "version = 1\nrevision = 4\nrequires-python = \">=3.11\"\n",
        );
        let after_lock = snapshot_td_lock_target(&target).unwrap();
        assert_ne!(after_openapi.source_digest, after_lock.source_digest);
        assert_eq!(after_openapi.ir_digest, after_lock.ir_digest);

        write(
            &td_root.join("invoice.py"),
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\n@openapi_client(source=\"openapi.json\", python=\"python-3.11\", typescript=\"typescript-5.0\", rust=\"rust-2021\")\nclass InvoiceClient:\n    pass\n\ndef issue_invoice(reference: str) -> None:\n    pass\n",
        );
        let after_python = snapshot_td_lock_target(&target).unwrap();
        assert_ne!(after_lock.source_digest, after_python.source_digest);
        assert_ne!(after_lock.ir_digest, after_python.ir_digest);
    }

    #[test]
    fn lock_target_defaults_to_project_tech_design() {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("aw.toml"),
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
        write(
            &tmp.path()
                .join("projects/demo/tech-design/src/demo/design.py"),
            "__aw_artifact_id__ = \"artifact:demo/design\"\n\ndef design() -> None:\n    pass\n",
        );
        write_python_td_environment(&tmp.path().join("projects/demo/tech-design"));

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
            &tmp.path().join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
"#,
        );
        let spec = tmp.path().join("projects/demo/tech-design/design.md");
        write(&spec, "design\n");
        write(
            &tmp.path()
                .join("projects/demo/tech-design/src/demo/design.py"),
            "__aw_artifact_id__ = \"artifact:demo/design\"\n\ndef design() -> None:\n    pass\n",
        );
        write_python_td_environment(&tmp.path().join("projects/demo/tech-design"));

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
            &tmp.path().join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
td_path = "projects/demo/tech-design"
"#,
        );
        let td_root = tmp.path().join("projects/demo/tech-design");
        write(&td_root.join("design.md"), "before\n");
        let python_td = td_root.join("src/demo/design.py");
        write(
            &python_td,
            "__aw_artifact_id__ = \"artifact:demo/design\"\n\ndef design() -> None:\n    pass\n",
        );
        write_python_td_environment(&td_root);
        let snapshot = snapshot_td_root(&td_root).unwrap();
        let lock = TdLockFile {
            version: TD_LOCK_VERSION,
            project: "demo".to_string(),
            ir_kind: Some(TD_IR_KIND.to_string()),
            td_path: "projects/demo/tech-design".to_string(),
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

        write(
            &python_td,
            "__aw_artifact_id__ = \"artifact:demo/design\"\n\ndef design() -> None:\n    return None\n",
        );
        let status = check_project_td_lock_at_root(tmp.path(), "demo").unwrap();

        assert_eq!(status.status, TdLockState::Stale);
        assert!(!status.clean);
        assert_eq!(status.changed, vec!["src/demo/design.py"]);
    }
}
// CODEGEN-END

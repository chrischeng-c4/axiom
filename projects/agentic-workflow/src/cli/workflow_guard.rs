// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
// CODEGEN-BEGIN
//! WI-rooted workflow projection and transition locks.
//!
//! The lock source of truth is the work-item itself: tracker labels provide a
//! coarse lock signal and the issue body carries a structured hidden state
//! block. The repo never writes `.aw/workflow/locks/*.toml`.

use crate::issues::{Issue, IssueBackend, IssueFilter, IssuePatch, LocalBackend};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const LOCK_LABEL: &str = "score:locked";
pub const TD_LOCK_LABEL: &str = "score:lock:td";
pub const CB_LOCK_LABEL: &str = "score:lock:cb";

const STATE_START: &str = "<!-- score:workflow-state";
const STATE_END: &str = "-->";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub struct TransitionLock {
    pub version: u8,
    pub issue_id: String,
    pub owner: String,
    pub expected_command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_payload: Option<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_section: Option<String>,
    #[serde(default)]
    pub remaining_sections: Vec<String>,
    #[serde(default)]
    pub dirty_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker_summary: Option<String>,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
impl TransitionLock {
    pub fn new(
        issue_id: impl Into<String>,
        owner: impl Into<String>,
        expected_command: impl Into<String>,
    ) -> Self {
        Self {
            version: 1,
            issue_id: issue_id.into(),
            owner: owner.into(),
            expected_command: expected_command.into(),
            expected_payload: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            phase_from: None,
            active_phase: None,
            active_branch: None,
            current_section: None,
            remaining_sections: Vec::new(),
            dirty_paths: Vec::new(),
            blocker_summary: None,
        }
    }

    pub fn with_expected_payload(mut self, path: impl Into<String>) -> Self {
        self.expected_payload = Some(normalize_rel(path.into()));
        self
    }

    pub fn with_phase_from(mut self, phase: impl Into<String>) -> Self {
        self.phase_from = Some(phase.into());
        self
    }

    pub fn with_active_phase(mut self, phase: impl Into<String>) -> Self {
        self.active_phase = Some(phase.into());
        self
    }

    pub fn with_active_branch(mut self, branch: impl Into<String>) -> Self {
        self.active_branch = Some(branch.into());
        self
    }

    pub fn with_current_section(mut self, section: impl Into<String>) -> Self {
        self.current_section = Some(section.into());
        self
    }

    pub fn with_remaining_sections(
        mut self,
        sections: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.remaining_sections = sections.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_dirty_paths(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.dirty_paths = paths.into_iter().map(|p| normalize_rel(p.into())).collect();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub struct WorkflowProjection {
    pub version: u8,
    pub issue_id: String,
    #[serde(default)]
    pub locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_payload: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_section: Option<String>,
    #[serde(default)]
    pub remaining_sections: Vec<String>,
    #[serde(default)]
    pub dirty_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
impl WorkflowProjection {
    fn from_lock(lock: &TransitionLock) -> Self {
        Self {
            version: lock.version,
            issue_id: lock.issue_id.clone(),
            locked: true,
            owner: Some(lock.owner.clone()),
            active_phase: lock
                .active_phase
                .clone()
                .or_else(|| lock.phase_from.clone()),
            active_branch: lock.active_branch.clone(),
            expected_payload: lock.expected_payload.clone(),
            expected_command: Some(lock.expected_command.clone()),
            current_section: lock.current_section.clone(),
            remaining_sections: lock.remaining_sections.clone(),
            dirty_paths: lock.dirty_paths.clone(),
            blocker_summary: lock.blocker_summary.clone(),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub struct IssueLockView {
    pub issue_id: String,
    pub owner: String,
    pub expected_command: String,
    pub expected_payload: Option<String>,
    pub active_phase: Option<String>,
    pub current_section: Option<String>,
    pub dirty_paths: Vec<String>,
    pub blocker_summary: Option<String>,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
impl IssueLockView {
    fn from_issue(issue: &Issue) -> Option<Self> {
        if !issue.labels.iter().any(|l| l == LOCK_LABEL) {
            return None;
        }
        let projection = parse_projection(&issue.body);
        if projection.as_ref().is_some_and(|p| !p.locked) {
            return None;
        }
        let owner = projection
            .as_ref()
            .and_then(|p| p.owner.clone())
            .or_else(|| lock_owner_from_labels(&issue.labels).map(str::to_string))?;
        let issue_id = projection
            .as_ref()
            .map(|p| p.issue_id.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| issue.slug.clone());
        Some(Self {
            issue_id,
            owner,
            expected_command: projection
                .as_ref()
                .and_then(|p| p.expected_command.clone())
                .unwrap_or_default(),
            expected_payload: projection.as_ref().and_then(|p| p.expected_payload.clone()),
            active_phase: projection.as_ref().and_then(|p| p.active_phase.clone()),
            current_section: projection.as_ref().and_then(|p| p.current_section.clone()),
            dirty_paths: projection
                .as_ref()
                .map(|p| p.dirty_paths.clone())
                .unwrap_or_default(),
            blocker_summary: projection.as_ref().and_then(|p| p.blocker_summary.clone()),
        })
    }
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub fn parse_projection(body: &str) -> Option<WorkflowProjection> {
    let start = body.find(STATE_START)?;
    let after_start = start + STATE_START.len();
    let rest = &body[after_start..];
    let end = rest.find(STATE_END)?;
    let yaml = rest[..end].trim();
    serde_yaml::from_str(yaml).ok()
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub fn upsert_projection(body: &str, projection: &WorkflowProjection) -> Result<String> {
    let yaml = serde_yaml::to_string(projection).context("serializing workflow projection")?;
    let block = format!("{STATE_START}\n{}{}{}\n", yaml.trim_end(), "\n", STATE_END);
    if let Some(start) = body.find(STATE_START) {
        let rest = &body[start + STATE_START.len()..];
        if let Some(end_rel) = rest.find(STATE_END) {
            let end = start + STATE_START.len() + end_rel + STATE_END.len();
            let mut out = String::new();
            out.push_str(body[..start].trim_end());
            out.push_str("\n\n");
            out.push_str(&block);
            out.push_str(body[end..].trim_start_matches('\n'));
            return Ok(out);
        }
    }
    let mut out = body.trim_end().to_string();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str(&block);
    Ok(out)
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub fn unlock_projection_for_closed_issue(body: &str, issue_id: &str) -> Result<Option<String>> {
    let Some(mut projection) = parse_projection(body) else {
        return Ok(None);
    };
    projection.locked = false;
    projection.owner = None;
    projection.active_phase = None;
    projection.expected_payload = None;
    projection.expected_command = None;
    projection.current_section = None;
    projection.remaining_sections.clear();
    projection.dirty_paths.clear();
    projection.blocker_summary = None;
    projection.updated_at = Some(chrono::Utc::now().to_rfc3339());
    if projection.version == 0 {
        projection.version = 1;
    }
    if projection.issue_id.is_empty() {
        projection.issue_id = issue_id.to_string();
    }
    Ok(Some(upsert_projection(body, &projection)?))
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub async fn create_issue_lock(project_root: &Path, lock: &TransitionLock) -> Result<()> {
    let backend = LocalBackend::from_project_root(project_root);
    let issue = backend
        .get(&lock.issue_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("work-item '{}' not found", lock.issue_id))?;
    let projection = WorkflowProjection::from_lock(lock);
    let body = upsert_projection(&issue.body, &projection)?;
    let mut remove_labels = owner_labels();
    remove_labels.retain(|label| label != &owner_label(&lock.owner));
    let patch = IssuePatch {
        add_labels: vec![LOCK_LABEL.to_string(), owner_label(&lock.owner)],
        remove_labels,
        body: Some(body),
        ..Default::default()
    };
    backend.update(&lock.issue_id, &patch).await?;
    maybe_push_issue(project_root, &issue, &lock.issue_id).await?;
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub async fn complete_issue_lock(project_root: &Path, issue_id: &str, owner: &str) -> Result<()> {
    let backend = LocalBackend::from_project_root(project_root);
    let issue = match backend.get(issue_id).await? {
        Some(issue) => issue,
        None => return Ok(()),
    };
    if !issue.labels.iter().any(|l| l == LOCK_LABEL) && parse_projection(&issue.body).is_none() {
        return Ok(());
    }
    let mut projection = parse_projection(&issue.body).unwrap_or_else(|| WorkflowProjection {
        version: 1,
        issue_id: issue_id.to_string(),
        ..Default::default()
    });
    if projection.locked {
        if projection.owner.as_deref().is_some_and(|o| o != owner) {
            anyhow::bail!(
                "workflow lock for {} is owned by {}, not {}",
                issue_id,
                projection.owner.unwrap_or_default(),
                owner
            );
        }
        projection.locked = false;
        projection.owner = None;
        projection.expected_payload = None;
        projection.expected_command = None;
        projection.current_section = None;
        projection.remaining_sections.clear();
        projection.dirty_paths.clear();
        projection.blocker_summary = None;
        projection.updated_at = Some(chrono::Utc::now().to_rfc3339());
        if projection.version == 0 {
            projection.version = 1;
        }
        if projection.issue_id.is_empty() {
            projection.issue_id = issue_id.to_string();
        }
    }
    let body = upsert_projection(&issue.body, &projection)?;
    let patch = IssuePatch {
        remove_labels: vec![LOCK_LABEL.to_string(), owner_label(owner)],
        body: Some(body),
        ..Default::default()
    };
    backend.update(issue_id, &patch).await?;
    maybe_push_issue(project_root, &issue, issue_id).await?;
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub async fn record_issue_blocker(
    project_root: &Path,
    issue_id: &str,
    owner: &str,
    message: &str,
) -> Result<()> {
    let backend = LocalBackend::from_project_root(project_root);
    let issue = backend
        .get(issue_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("work-item '{}' not found", issue_id))?;
    let mut projection = parse_projection(&issue.body).unwrap_or_default();
    if projection.version == 0 {
        projection.version = 1;
    }
    if projection.issue_id.is_empty() {
        projection.issue_id = issue_id.to_string();
    }
    projection.locked = true;
    projection.owner = Some(owner.to_string());
    projection.blocker_summary = Some(message.to_string());
    projection.updated_at = Some(chrono::Utc::now().to_rfc3339());
    let body = upsert_projection(&issue.body, &projection)?;
    let patch = IssuePatch {
        add_labels: vec![LOCK_LABEL.to_string(), owner_label(owner)],
        body: Some(body),
        ..Default::default()
    };
    backend.update(issue_id, &patch).await?;
    maybe_push_issue(project_root, &issue, issue_id).await?;
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub async fn guard_issue_mutation(
    project_root: &Path,
    allowed: Option<(&str, &str)>,
) -> Result<()> {
    for lock in issue_locks(project_root).await? {
        let allowed_match = allowed
            .map(|(owner, issue_id)| lock.owner == owner && lock.issue_id == issue_id)
            .unwrap_or(false);
        if !allowed_match {
            let expected = if lock.expected_command.is_empty() {
                "the matching gate".to_string()
            } else {
                format!("`{}`", lock.expected_command)
            };
            anyhow::bail!(
                "pending workflow lock for {} owned by {}; run {} before starting another mutating workflow command",
                lock.issue_id,
                lock.owner,
                expected
            );
        }
    }
    Ok(())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub async fn issue_locks(project_root: &Path) -> Result<Vec<IssueLockView>> {
    let backend = LocalBackend::from_project_root(project_root);
    let filter = IssueFilter {
        label: Some(LOCK_LABEL.to_string()),
        ..Default::default()
    };
    let issues = backend.list(&filter).await?;
    Ok(issues
        .iter()
        .filter_map(IssueLockView::from_issue)
        .collect())
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub async fn hook_pretooluse_workflow_guard(
    project_root: &Path,
    payload: &Value,
) -> Result<HookDecision> {
    let locks = issue_locks(project_root).await?;
    if locks.is_empty() {
        return Ok(HookDecision::Allow);
    }
    let Some(lock) = locks.first() else {
        return Ok(HookDecision::Allow);
    };
    let tool = payload
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if matches!(tool, "Write" | "Edit" | "MultiEdit") {
        if let Some(rel) = payload_file_path(project_root, payload) {
            if path_allowed_by_lock(&rel, lock) {
                return Ok(HookDecision::Allow);
            }
            return Ok(HookDecision::Block(format!(
                "workflow lock for {} expects write to {} before `{}`; got {}",
                lock.issue_id,
                lock.expected_payload.as_deref().unwrap_or("<no payload>"),
                lock.expected_command,
                rel
            )));
        }
    }
    if tool == "Bash" {
        let cmd = payload
            .get("tool_input")
            .and_then(|v| v.get("command"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if is_score_workflow_mutation(cmd) && !command_matches(cmd, &lock.expected_command) {
            return Ok(HookDecision::Block(format!(
                "workflow lock for {} owned by {} expects `{}`; got `{}`",
                lock.issue_id, lock.owner, lock.expected_command, cmd
            )));
        }
    }
    Ok(HookDecision::Allow)
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub async fn hook_posttooluse_workflow_apply(
    project_root: &Path,
    payload: &Value,
) -> Result<HookDecision> {
    let locks = issue_locks(project_root).await?;
    let Some(lock) = locks.first() else {
        return Ok(HookDecision::Allow);
    };
    let tool = payload
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !matches!(tool, "Write" | "Edit" | "MultiEdit") {
        return Ok(HookDecision::Allow);
    }
    let Some(rel) = payload_file_path(project_root, payload) else {
        return Ok(HookDecision::Allow);
    };
    if lock.expected_payload.as_deref() != Some(rel.as_str()) {
        return Ok(HookDecision::Allow);
    }
    if lock.expected_command.trim().is_empty() {
        return Ok(HookDecision::Allow);
    }

    let output = Command::new("sh")
        .arg("-lc")
        .arg(&lock.expected_command)
        .current_dir(project_root)
        .output()
        .with_context(|| format!("running workflow command `{}`", lock.expected_command))?;
    if output.status.success() {
        let mut text = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            if !text.ends_with('\n') && !text.is_empty() {
                text.push('\n');
            }
            text.push_str(stderr.trim());
        }
        return Ok(HookDecision::AllowWithOutput(text));
    }

    let msg = format!(
        "workflow auto-apply failed for {}: {}{}{}",
        lock.issue_id,
        String::from_utf8_lossy(&output.stderr).trim(),
        if output.stdout.is_empty() { "" } else { "\n" },
        String::from_utf8_lossy(&output.stdout).trim()
    );
    let _ = record_issue_blocker(project_root, &lock.issue_id, &lock.owner, &msg).await;
    Ok(HookDecision::Block(msg))
}

#[derive(Debug, Clone, PartialEq, Eq)]
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/workflow_guard.md#source
pub enum HookDecision {
    Allow,
    AllowWithOutput(String),
    Block(String),
}

fn lock_owner_from_labels(labels: &[String]) -> Option<&'static str> {
    if labels.iter().any(|l| l == TD_LOCK_LABEL) {
        Some("td")
    } else if labels.iter().any(|l| l == CB_LOCK_LABEL) {
        Some("cb")
    } else {
        None
    }
}

fn owner_label(owner: &str) -> String {
    match owner {
        "td" => TD_LOCK_LABEL.to_string(),
        "cb" => CB_LOCK_LABEL.to_string(),
        other => format!("score:lock:{other}"),
    }
}

fn owner_labels() -> Vec<String> {
    vec![TD_LOCK_LABEL.to_string(), CB_LOCK_LABEL.to_string()]
}

async fn maybe_push_issue(project_root: &Path, issue: &Issue, issue_id: &str) -> Result<()> {
    let backend = crate::issues::LocalBackend::from_project_root(project_root);
    let path = backend.issue_path(issue);
    crate::cli::remote_push::maybe_push_remote(project_root, &path, issue_id).await
}

fn payload_file_path(project_root: &Path, payload: &Value) -> Option<String> {
    let raw = payload
        .get("tool_input")
        .and_then(|v| v.get("file_path"))
        .and_then(Value::as_str)?;
    Some(path_to_rel(project_root, raw))
}

fn path_to_rel(project_root: &Path, raw: &str) -> String {
    let path = PathBuf::from(raw);
    let abs = if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    };
    let root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let resolved = abs.canonicalize().unwrap_or(abs);
    match resolved.strip_prefix(&root) {
        Ok(rel) => normalize_rel(rel.to_string_lossy().to_string()),
        Err(_) => normalize_rel(raw.to_string()),
    }
}

fn path_allowed_by_lock(rel: &str, lock: &IssueLockView) -> bool {
    lock.expected_payload.as_deref() == Some(rel)
}

fn is_score_workflow_mutation(cmd: &str) -> bool {
    let norm = normalize_command(cmd);
    let tokens: Vec<&str> = norm.split_whitespace().collect();
    for window in tokens.windows(3) {
        if window[0] == "score" && window[1] == "td" {
            return !matches!(window[2], "check" | "ast");
        }
        if window[0] == "score" && window[1] == "cb" {
            return !matches!(window[2], "check");
        }
    }
    false
}

fn command_matches(actual: &str, expected: &str) -> bool {
    let actual = normalize_command(actual);
    let expected = normalize_command(expected);
    !expected.is_empty() && (actual == expected || actual.contains(&expected))
}

fn normalize_command(cmd: &str) -> String {
    cmd.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_rel(path: String) -> String {
    path.trim()
        .trim_start_matches("./")
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::{IssueState, IssueType};

    fn issue(slug: &str, body: &str, labels: Vec<&str>) -> Issue {
        Issue {
            issue_type: IssueType::Enhancement,
            title: "Test".to_string(),
            state: IssueState::Open,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: labels.into_iter().map(str::to_string).collect(),
            created_at: None,
            updated_at: None,
            slug: slug.to_string(),
            body: body.to_string(),
            related: vec![],
            implements: vec![],
            phase: None,
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
            validation_errors: vec![],
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    #[test]
    fn projection_round_trip_preserves_body() {
        let body = "# Title\n\nBody text\n";
        let projection = WorkflowProjection {
            version: 1,
            issue_id: "123".to_string(),
            locked: true,
            owner: Some("td".to_string()),
            expected_command: Some("aw td create 123 --apply".to_string()),
            ..Default::default()
        };
        let updated = upsert_projection(body, &projection).unwrap();
        assert!(updated.starts_with("# Title\n\nBody text"));
        let parsed = parse_projection(&updated).unwrap();
        assert_eq!(parsed.issue_id, "123");
        assert!(parsed.locked);
    }

    #[test]
    fn issue_lock_view_uses_labels_and_projection() {
        let projection = WorkflowProjection {
            version: 1,
            issue_id: "123".to_string(),
            locked: true,
            owner: Some("td".to_string()),
            expected_payload: Some(".aw/payloads/123/applicability/schema.md".to_string()),
            expected_command: Some("aw td create 123 --apply".to_string()),
            ..Default::default()
        };
        let body = upsert_projection("Body", &projection).unwrap();
        let issue = issue("123", &body, vec![LOCK_LABEL, TD_LOCK_LABEL]);
        let view = IssueLockView::from_issue(&issue).unwrap();
        assert_eq!(view.owner, "td");
        assert_eq!(
            view.expected_payload.as_deref(),
            Some(".aw/payloads/123/applicability/schema.md")
        );
    }

    #[test]
    fn issue_lock_view_ignores_stale_labels_when_projection_unlocked() {
        let projection = WorkflowProjection {
            version: 1,
            issue_id: "123".to_string(),
            locked: false,
            owner: None,
            ..Default::default()
        };
        let body = upsert_projection("Body", &projection).unwrap();
        let issue = issue("123", &body, vec![LOCK_LABEL, TD_LOCK_LABEL]);

        assert!(IssueLockView::from_issue(&issue).is_none());
    }

    #[test]
    fn unlock_projection_for_closed_issue_clears_active_command() {
        let projection = WorkflowProjection {
            version: 1,
            issue_id: "123".to_string(),
            locked: true,
            owner: Some("td".to_string()),
            active_phase: Some("td_inited".to_string()),
            expected_command: Some("aw td validate 123".to_string()),
            expected_payload: Some(".aw/payloads/123/applicability/schema.md".to_string()),
            remaining_sections: vec!["schema".to_string()],
            dirty_paths: vec![".aw/payloads/123/applicability/schema.md".to_string()],
            blocker_summary: Some("waiting".to_string()),
            ..Default::default()
        };
        let body = upsert_projection("Body", &projection).unwrap();
        let updated = unlock_projection_for_closed_issue(&body, "123")
            .unwrap()
            .expect("projection should be updated");
        let parsed = parse_projection(&updated).unwrap();

        assert!(!parsed.locked);
        assert_eq!(parsed.owner, None);
        assert_eq!(parsed.active_phase, None);
        assert_eq!(parsed.expected_command, None);
        assert!(parsed.remaining_sections.is_empty());
        assert!(parsed.dirty_paths.is_empty());
    }

    #[test]
    fn hook_guard_blocks_wrong_payload_path() {
        let lock = IssueLockView {
            issue_id: "123".to_string(),
            owner: "td".to_string(),
            expected_command: "aw td create 123 --apply".to_string(),
            expected_payload: Some(".aw/payloads/123/applicability/schema.md".to_string()),
            active_phase: None,
            current_section: None,
            dirty_paths: vec![
                "projects/agentic-workflow/tech-design/surface/specs/123.md".to_string()
            ],
            blocker_summary: None,
        };
        assert!(path_allowed_by_lock(
            ".aw/payloads/123/applicability/schema.md",
            &lock
        ));
        assert!(!path_allowed_by_lock(
            "projects/agentic-workflow/tech-design/surface/specs/123.md",
            &lock
        ));
        assert!(!path_allowed_by_lock(
            ".aw/payloads/456/applicability/schema.md",
            &lock
        ));
    }

    #[test]
    fn command_match_normalizes_whitespace() {
        assert!(command_matches(
            "  aw   td create 123 --apply ",
            "aw td create 123 --apply"
        ));
        assert!(!command_matches(
            "aw td review 123 --apply",
            "aw td create 123 --apply"
        ));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END

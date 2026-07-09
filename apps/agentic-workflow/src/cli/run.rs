// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
// CODEGEN-BEGIN
//! `aw wi run` / `aw capability run` -- root-driven workflow runner envelope.

use crate::cli::capability::{
    self, CapabilityAction, CapabilityActionKind, CapabilityStatus, HitlChoice, HitlQuestion,
};
#[cfg(test)]
use crate::cli::issues as wi_cli;
use crate::issues::{make_backend, resolve_default_backend, Issue, IssueState, IssueType};
use crate::models::artifact_quality::{
    infer_artifact_kind_from_hint, ArtifactKind, ArtifactQualityProfile,
};
use crate::models::preflight::{
    default_preflight_gates, PreFlightEvidenceKind, PreFlightGateSeverity,
};
use anyhow::{Context, Result};
#[cfg(test)]
use serde::Deserialize;
use serde::{ser::SerializeStruct, Serialize};
#[cfg(test)]
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::future::Future;
#[cfg(test)]
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const GOAL_INLINE_LIMIT_BYTES: usize = 4000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResolvedRunRoot {
    Capability {
        project: String,
        capability_id: String,
        command: String,
    },
    Wi {
        wi: String,
        command: String,
    },
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl ResolvedRunRoot {
    /// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
    fn command(&self) -> &str {
        match self {
            ResolvedRunRoot::Capability { command, .. } | ResolvedRunRoot::Wi { command, .. } => {
                command
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkflowNode {
    kind: String,
    id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkflowNext {
    kind: String,
    command: String,
    reason: String,
    payload_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkflowInvoke {
    command: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkflowCompletion {
    root_complete: bool,
    workflow_complete: bool,
    criteria: Vec<String>,
    missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkflowPersistence {
    status: String,
    commit_complete: bool,
    wi_evidence_complete: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    dirty_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    scopes: Vec<String>,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowEnvelope {
    action: String,
    root: WorkflowNode,
    current: WorkflowNode,
    completed: Option<WorkflowNode>,
    completion: WorkflowCompletion,
    next: WorkflowNext,
    invoke: WorkflowInvoke,
    agent_prompt: String,
    requires_hitl: bool,
    artifact_quality_profile: Option<ArtifactQualityProfile>,
    hitl_question: Option<HitlQuestion>,
    persistence: Option<WorkflowPersistence>,
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl Serialize for WorkflowEnvelope {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("WorkflowEnvelope", 15)?;
        state.serialize_field("schema_version", "aw.cli.v1")?;
        state.serialize_field("status", workflow_status(self))?;
        state.serialize_field("action", &self.action)?;
        state.serialize_field("root", &self.root)?;
        state.serialize_field("current", &self.current)?;
        if let Some(completed) = &self.completed {
            state.serialize_field("completed", completed)?;
        }
        state.serialize_field(
            "completion",
            &SerializableWorkflowCompletion {
                root_complete: self.completion.root_complete,
                workflow_complete: self.completion.workflow_complete,
                requires_hitl: self.requires_hitl,
                criteria: &self.completion.criteria,
                missing: &self.completion.missing,
            },
        )?;
        state.serialize_field("next", &serializable_next(self))?;
        if let Some(payload_path) = self.next.payload_path.as_deref() {
            state.serialize_field("payload_path", payload_path)?;
        }
        state.serialize_field("agent_prompt", &self.agent_prompt)?;
        if let Some(profile) = &self.artifact_quality_profile {
            state.serialize_field("artifact_quality_profile", profile)?;
        }
        if let Some(question) = &self.hitl_question {
            state.serialize_field("hitl_question", question)?;
        }
        if let Some(persistence) = &self.persistence {
            state.serialize_field("persistence", persistence)?;
        }
        state.end()
    }
}

#[derive(Serialize)]
struct SerializableWorkflowCompletion<'a> {
    root_complete: bool,
    workflow_complete: bool,
    requires_hitl: bool,
    criteria: &'a [String],
    missing: &'a [String],
}

#[derive(Serialize)]
struct SerializableWorkflowNext<'a> {
    kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<&'a str>,
    reason: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_path: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct CanonicalWorkflowCompletion {
    root_complete: bool,
    workflow_complete: bool,
    requires_hitl: bool,
    criteria: Vec<String>,
    missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct CanonicalWorkflowNext {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_path: Option<String>,
}

fn workflow_status(envelope: &WorkflowEnvelope) -> &'static str {
    if envelope.completion.workflow_complete {
        "done"
    } else if envelope.requires_hitl || envelope.action == "blocked" {
        "blocked"
    } else {
        "continue"
    }
}

fn serializable_next(envelope: &WorkflowEnvelope) -> SerializableWorkflowNext<'_> {
    let command =
        (!envelope.next.command.trim().is_empty()).then_some(envelope.next.command.as_str());
    SerializableWorkflowNext {
        kind: canonical_next_kind(envelope, command.is_some()),
        command,
        reason: &envelope.next.reason,
        payload_path: envelope.next.payload_path.as_deref(),
    }
}

fn canonical_completion(envelope: &WorkflowEnvelope) -> CanonicalWorkflowCompletion {
    CanonicalWorkflowCompletion {
        root_complete: envelope.completion.root_complete,
        workflow_complete: envelope.completion.workflow_complete,
        requires_hitl: envelope.requires_hitl,
        criteria: envelope.completion.criteria.clone(),
        missing: envelope.completion.missing.clone(),
    }
}

fn canonical_next_owned(envelope: &WorkflowEnvelope) -> CanonicalWorkflowNext {
    let has_command = !envelope.next.command.trim().is_empty();
    CanonicalWorkflowNext {
        kind: canonical_next_kind(envelope, has_command).to_string(),
        command: has_command.then(|| envelope.next.command.clone()),
        reason: envelope.next.reason.clone(),
        payload_path: envelope.next.payload_path.clone(),
    }
}

fn canonical_next_kind(envelope: &WorkflowEnvelope, has_command: bool) -> &'static str {
    if envelope.completion.workflow_complete {
        "done"
    } else if envelope.requires_hitl {
        "hitl"
    } else if envelope.action == "blocked" {
        "blocked"
    } else if has_command {
        "run_command"
    } else {
        "error"
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkflowGoalEnvelope {
    schema_version: String,
    status: String,
    action: String,
    root: WorkflowNode,
    root_command: String,
    first_next: CanonicalWorkflowNext,
    completion: CanonicalWorkflowCompletion,
    payload_path: String,
    prompt_size_bytes: usize,
    inline_limit_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    goal_prompt: Option<String>,
}

/// Print options shared by every thin runner shell (`aw wi run`,
/// `aw capability run <id>`). Mirrors the human/pretty/goal subset that
/// actually affects output shape.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RunPrintOptions {
    pub(crate) human: bool,
    pub(crate) pretty: bool,
    pub(crate) goal: bool,
}

/// One deterministic tick over a resolved workflow root -- the single
/// implementation shared by `aw wi run <id>` and
/// `aw capability run <capability-id>`. No caller duplicates this
/// dispatch; they only resolve a root and forward here.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
async fn run_resolved_root(root: ResolvedRunRoot, print: RunPrintOptions) -> Result<()> {
    let root_command = root.command().to_string();
    let progress = RunProgressSink::new(&root, !print.human && !print.pretty && !print.goal);
    progress.emit(
        5,
        "start",
        "resolved workflow root",
        Some(root_command.as_str()),
    );
    let mut envelope = match &root {
        ResolvedRunRoot::Capability {
            project,
            capability_id,
            ..
        } => capability_envelope(project, capability_id, &progress).await,
        ResolvedRunRoot::Wi { wi, .. } => wi_envelope(wi, &progress).await,
    };
    ensure_hitl_question(&mut envelope, &root_command);
    apply_artifact_quality_gate(&mut envelope);
    let summary_command =
        (!envelope.next.command.trim().is_empty()).then_some(envelope.next.command.as_str());
    progress.emit(
        95,
        "summary",
        "building workflow runner envelope",
        summary_command,
    );

    if print.goal {
        let goal = workflow_goal_envelope(&envelope, &root_command)?;
        if print.human {
            print_goal_text(&goal);
        } else if print.pretty {
            println!("{}", serde_json::to_string_pretty(&goal)?);
        } else {
            println!("{}", serde_json::to_string(&goal)?);
        }
        return Ok(());
    }

    if print.human {
        print_text(&envelope);
    } else if print.pretty {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("{}", serde_json::to_string(&envelope)?);
    }
    Ok(())
}

/// Command string `aw wi run <id>` would print for the given work-item id.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub(crate) fn wi_run_command(id: &str) -> String {
    format!("aw wi run {id}")
}

/// Thin shell: `aw wi run <id>` -- drive one work item's next lifecycle tick
/// via the shared root loop.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub(crate) async fn run_wi_root(id: &str, print: RunPrintOptions) -> Result<()> {
    let root = ResolvedRunRoot::Wi {
        wi: id.to_string(),
        command: wi_run_command(id),
    };
    run_resolved_root(root, print).await
}

/// Command string `aw capability run <capability-id> --project <project>`
/// would print.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub(crate) fn capability_run_command(project: &str, capability_id: &str) -> String {
    format!("aw capability run {capability_id} --project {project}")
}

/// Thin shell: `aw capability run <capability-id>` -- drive that
/// capability's next work-root tick via the shared root loop.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub(crate) async fn run_capability_root(
    project: &str,
    capability_id: &str,
    print: RunPrintOptions,
) -> Result<()> {
    let root = ResolvedRunRoot::Capability {
        project: project.to_string(),
        capability_id: capability_id.to_string(),
        command: capability_run_command(project, capability_id),
    };
    run_resolved_root(root, print).await
}

/// Command string for the project-scoped capability completion loop that
/// subsumes a project root.
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
pub(crate) fn project_capability_rollup_command(project: &str) -> String {
    format!("aw capability run --project {project} --non-interactive --max-ticks 1")
}

struct RunProgressSink {
    root_kind: String,
    root_id: String,
    started: Instant,
    enabled: bool,
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl RunProgressSink {
    fn new(root: &ResolvedRunRoot, enabled: bool) -> Self {
        let (root_kind, root_id) = match root {
            ResolvedRunRoot::Capability { capability_id, .. } => {
                ("capability", capability_id.as_str())
            }
            ResolvedRunRoot::Wi { wi, .. } => ("wi", wi.as_str()),
        };
        Self {
            root_kind: root_kind.to_string(),
            root_id: root_id.to_string(),
            started: Instant::now(),
            enabled,
        }
    }

    fn emit(&self, percent: u8, phase: &str, message: &str, command: Option<&str>) {
        if !self.enabled {
            return;
        }
        emit_run_progress_event(
            &self.root_kind,
            &self.root_id,
            self.started,
            percent,
            phase,
            message,
            command,
        );
    }

    fn heartbeat(
        &self,
        percent: u8,
        phase: &str,
        message: &str,
        command: Option<String>,
    ) -> Option<RunProgressHeartbeat> {
        if !self.enabled {
            return None;
        }
        let root_kind = self.root_kind.clone();
        let root_id = self.root_id.clone();
        let started = self.started;
        let phase = phase.to_string();
        let message = message.to_string();
        let (tx, rx) = mpsc::channel::<()>();
        let handle = thread::spawn(move || loop {
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => emit_run_progress_event(
                    &root_kind,
                    &root_id,
                    started,
                    percent,
                    &phase,
                    &message,
                    command.as_deref(),
                ),
            }
        });
        Some(RunProgressHeartbeat {
            stop: Some(tx),
            handle: Some(handle),
        })
    }
}

struct RunProgressHeartbeat {
    stop: Option<mpsc::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
impl Drop for RunProgressHeartbeat {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn emit_run_progress_event(
    root_kind: &str,
    root_id: &str,
    started: Instant,
    percent: u8,
    phase: &str,
    message: &str,
    command: Option<&str>,
) {
    let event = serde_json::json!({
        "schema_version": "aw.cli.v1",
        "event": "progress",
        "root": {
            "kind": root_kind,
            "id": root_id,
        },
        "percent": percent.min(100),
        "phase": phase,
        "message": message,
        "elapsed_ms": started.elapsed().as_millis(),
        "command": command,
    });
    println!("{event}");
    let _ = std::io::stdout().flush();
}

async fn await_with_progress<F, T>(
    progress: &RunProgressSink,
    percent: u8,
    phase: &str,
    message: &str,
    command: Option<String>,
    future: F,
) -> T
where
    F: Future<Output = T>,
{
    if !progress.enabled {
        return future.await;
    }
    let _heartbeat = progress.heartbeat(percent, phase, message, command);
    future.await
}

fn workflow_goal_envelope(
    envelope: &WorkflowEnvelope,
    root_command: &str,
) -> Result<WorkflowGoalEnvelope> {
    let prompt = workflow_goal_prompt(envelope, root_command);
    let payload_path = workflow_goal_payload_path(envelope);
    write_goal_payload(&payload_path, &prompt)?;
    let prompt_size_bytes = prompt.len();
    let goal_prompt = (prompt_size_bytes <= GOAL_INLINE_LIMIT_BYTES).then_some(prompt);
    Ok(WorkflowGoalEnvelope {
        schema_version: "aw.cli.v1".to_string(),
        status: workflow_status(envelope).to_string(),
        action: "goal_prompt".to_string(),
        root: envelope.root.clone(),
        root_command: root_command.to_string(),
        first_next: canonical_next_owned(envelope),
        completion: canonical_completion(envelope),
        payload_path: payload_path.to_string_lossy().replace('\\', "/"),
        prompt_size_bytes,
        inline_limit_bytes: GOAL_INLINE_LIMIT_BYTES,
        goal_prompt,
    })
}

fn workflow_goal_prompt(envelope: &WorkflowEnvelope, root_command: &str) -> String {
    let next_command = if envelope.next.command.trim().is_empty() {
        "(none)".to_string()
    } else {
        envelope.next.command.clone()
    };
    let payload_line = envelope
        .next
        .payload_path
        .as_deref()
        .map(|path| format!("- If you need detailed payload context, read `{path}`.\n"))
        .unwrap_or_default();
    let hitl_line = if envelope.requires_hitl {
        "- The initial envelope already has `requires_hitl=true`; surface the HITL question/blocker before doing unattended work.\n"
    } else {
        ""
    };
    let missing = if envelope.completion.missing.is_empty() {
        "- none\n".to_string()
    } else {
        envelope
            .completion
            .missing
            .iter()
            .map(|item| format!("- {item}\n"))
            .collect::<String>()
    };

    format!(
        "Drive the Agentic Workflow root `{root_kind}:{root_id}` until the root workflow reaches a terminal state.\n\n\
Completion condition:\n\
- Run `{root_command}` after each child command and inspect its JSON stdout.\n\
- The goal is complete only when `completion.workflow_complete=true`, or when `requires_hitl=true` has been surfaced with the blocker/question and exact resume command.\n\
- Do not treat `action=done` alone as root completion; it can mean only the current child root is complete.\n\n\
Execution protocol:\n\
- Start with this first command: `{next_command}`.\n\
- After any child command succeeds, run `{root_command}` again and follow the new `next.command`.\n\
- Follow `next.command` exactly; do not add `--json`, and do not add default/no-op flags.\n\
- If stdout includes `next.payload_path`, read or update that path as instructed by the command.\n\
{payload_line}\
{hitl_line}\
- Keep diagnostics/progress out of the final claim; prove completion with the latest `{root_command}` JSON.\n\n\
Initial state:\n\
- action: `{action}`\n\
- current: `{current_kind}:{current_id}`\n\
- next.kind: `{next_kind}`\n\
- next.reason: {next_reason}\n\
- requires_hitl: {requires_hitl}\n\
- workflow_complete: {workflow_complete}\n\n\
Initial missing criteria:\n{missing}",
        root_kind = envelope.root.kind,
        root_id = envelope.root.id,
        current_kind = envelope.current.kind,
        current_id = envelope.current.id,
        action = envelope.action,
        next_kind = envelope.next.kind,
        next_reason = envelope.next.reason,
        requires_hitl = envelope.requires_hitl,
        workflow_complete = envelope.completion.workflow_complete,
    )
}

fn workflow_goal_payload_path(envelope: &WorkflowEnvelope) -> PathBuf {
    crate::shared::workspace::aw_tmp_path()
        .join("goals")
        .join(format!(
            "aw-run-{}-{}.md",
            slug_for_goal_path(&envelope.root.kind),
            slug_for_goal_path(&envelope.root.id)
        ))
}

fn write_goal_payload(path: &Path, prompt: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create goal prompt directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(path, prompt).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn slug_for_goal_path(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "root".to_string()
    } else {
        trimmed.to_string()
    }
}

fn agent_command(command: impl AsRef<str>) -> String {
    let mut out = Vec::new();
    let mut parts = command.as_ref().split_whitespace().peekable();
    while let Some(part) = parts.next() {
        if part == "--json" {
            continue;
        }
        if part == "--max-ticks" && parts.peek().is_some_and(|next| *next == "1") {
            parts.next();
            continue;
        }
        out.push(part);
    }
    out.join(" ")
}

fn agent_hitl_question(question: Option<HitlQuestion>) -> Option<HitlQuestion> {
    question.map(|mut question| {
        question.resume_command = agent_command(&question.resume_command);
        question
    })
}

fn apply_artifact_quality_gate(envelope: &mut WorkflowEnvelope) {
    if envelope.action == "done" || envelope.next.command.trim().is_empty() {
        return;
    }

    let profile = envelope
        .artifact_quality_profile
        .clone()
        .unwrap_or_else(|| ArtifactQualityProfile::default_for_kind(infer_artifact_kind(envelope)));
    if !envelope
        .completion
        .criteria
        .iter()
        .any(|criterion| criterion == "artifact quality hard preflight gates are satisfied")
    {
        envelope
            .completion
            .criteria
            .push("artifact quality hard preflight gates are satisfied".to_string());
    }
    envelope.agent_prompt = append_artifact_quality_prompt(&envelope.agent_prompt, &profile);
    envelope.artifact_quality_profile = Some(profile);
}

fn infer_artifact_kind(envelope: &WorkflowEnvelope) -> ArtifactKind {
    let text = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        envelope.next.kind,
        envelope.next.command,
        envelope.next.reason,
        envelope.root.id,
        envelope.current.id,
        envelope.agent_prompt
    );
    infer_artifact_kind_from_hint(&text)
}

fn append_artifact_quality_prompt(base: &str, profile: &ArtifactQualityProfile) -> String {
    if base.contains("Artifact Quality Gate") {
        return base.to_string();
    }

    let mut out = base.trim_end().to_string();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str("Artifact Quality Gate\n");
    out.push_str(&profile.to_review_prompt_context());
    out.push_str("required_preflight_gates:\n");
    for gate in default_preflight_gates(profile.artifact_kind) {
        out.push_str(&format!(
            "- {} [{} {}]: {}\n",
            gate.id,
            preflight_severity_label(gate.severity),
            preflight_evidence_label(gate.evidence_kind),
            gate.description
        ));
    }
    out.push_str("Hard gates require machine-verifiable evidence before completion. ");
    out.push_str("For frontend/UI artifacts, include desktop and mobile viewport evidence, interaction smoke proof, accessibility/readability smoke proof, and no placeholder or skeleton-only primary state.\n");
    out
}

fn preflight_severity_label(severity: PreFlightGateSeverity) -> &'static str {
    match severity {
        PreFlightGateSeverity::Hard => "hard",
        PreFlightGateSeverity::Advisory => "advisory",
    }
}

fn preflight_evidence_label(kind: PreFlightEvidenceKind) -> &'static str {
    match kind {
        PreFlightEvidenceKind::Test => "test",
        PreFlightEvidenceKind::Screenshot => "screenshot",
        PreFlightEvidenceKind::Transcript => "transcript",
        PreFlightEvidenceKind::LinkCheck => "link-check",
        PreFlightEvidenceKind::SourceAnnotation => "source-annotation",
        PreFlightEvidenceKind::ReviewNote => "review-note",
    }
}

async fn capability_envelope(
    project: &str,
    capability_id: &str,
    progress: &RunProgressSink,
) -> WorkflowEnvelope {
    let root = WorkflowNode {
        kind: "capability".to_string(),
        id: capability_id.to_string(),
    };
    progress.emit(
        25,
        "capability",
        "evaluating scoped capability readiness",
        Some(format!("aw capability check --project {project} --verify").as_str()),
    );
    let capability_command = format!("aw capability check --project {project} --verify");
    let report_result = await_with_progress(
        progress,
        25,
        "capability",
        "still evaluating scoped capability readiness",
        Some(capability_command),
        capability::build_capability_report_for_capability(
            project,
            None,
            true,
            true,
            capability_id,
        ),
    )
    .await;
    match report_result {
        Ok(report) => {
            let Some(item) = report
                .capabilities
                .iter()
                .find(|item| item.id == capability_id)
            else {
                return blocked_envelope(
                    root.clone(),
                    root,
                    format!("aw capability report --project {project}"),
                    format!("capability `{capability_id}` was not found in project `{project}`"),
                    true,
                );
            };
            if item.production_ready {
                let command = project_capability_rollup_command(project);
                return WorkflowEnvelope {
                    action: "done".to_string(),
                    current: root.clone(),
                    completed: Some(root.clone()),
                    completion: capability_completion(true, Vec::new()),
                    next: WorkflowNext {
                        kind: "inspect_parent".to_string(),
                        command: command.clone(),
                        reason: "capability is production ready; inspect project root for rollup"
                            .to_string(),
                        payload_path: None,
                    },
                    invoke: WorkflowInvoke { command },
                    agent_prompt:
                        "Capability production scope is complete. Re-run the project root and continue rollup."
                            .to_string(),
                    requires_hitl: false,
                    artifact_quality_profile: None,
                    hitl_question: None,
                    persistence: None,
                    root,
                };
            }
            if report.next_action.kind == CapabilityActionKind::None {
                return capability_production_blocked_envelope(
                    project,
                    capability_id,
                    root.clone(),
                    item.production_blockers.clone(),
                    report.production_blockers.clone(),
                );
            }
            capability_action_envelope(
                root.clone(),
                WorkflowNode {
                    kind: "capability".to_string(),
                    id: capability_id.to_string(),
                },
                project,
                &report.next_action,
                capability_completion(false, capability_missing(item, &report.next_action)),
            )
        }
        Err(err) => capability_root_unrunnable_envelope(root.clone(), root, err),
    }
}

fn capability_root_unrunnable_envelope(
    root: WorkflowNode,
    current: WorkflowNode,
    err: anyhow::Error,
) -> WorkflowEnvelope {
    blocked_envelope(
        root,
        current,
        String::new(),
        format!("capability root is not runnable: {err}"),
        false,
    )
}

async fn wi_envelope(wi: &str, progress: &RunProgressSink) -> WorkflowEnvelope {
    let root = WorkflowNode {
        kind: "change".to_string(),
        id: wi.to_string(),
    };
    let project_root = match crate::find_project_root() {
        Ok(root) => root,
        Err(err) => {
            return blocked_envelope(
                root.clone(),
                root,
                format!("aw wi show {wi}"),
                format!("project root unavailable: {err}"),
                true,
            )
        }
    };
    progress.emit(
        25,
        "work_item",
        "loading work item state",
        Some(format!("aw wi show {wi}").as_str()),
    );
    let issue_result = await_with_progress(
        progress,
        25,
        "work_item",
        "still loading work item state",
        Some(format!("aw wi show {wi}")),
        resolve_issue(wi, &project_root),
    )
    .await;
    let issue = match issue_result {
        Ok(Some(issue)) => issue,
        Ok(None) => {
            return blocked_envelope(
                root.clone(),
                root,
                format!("aw wi show {wi}"),
                format!("work item `{wi}` was not found"),
                true,
            )
        }
        Err(err) => {
            return blocked_envelope(
                root.clone(),
                root,
                format!("aw wi show {wi}"),
                format!("work item inventory unavailable: {err}"),
                true,
            )
        }
    };

    if issue.state == IssueState::Closed {
        return closed_wi_envelope(&issue);
    }

    // #188 E1: when the WI carries an `<!-- aw:loop-state -->` block, the loop
    // engine drives the next act from the verifier result (decide_next_action),
    // not the CRRR phase machine. The block is authored on the LOCAL lifecycle
    // copy (by `aw ec record`), which may not be synced to the resolved
    // (configured-backend) issue — so prefer the local copy's body, falling back
    // to the resolved body. Absent any block, fall through to the phase-driven
    // path below (fully backward-compatible).
    let loop_body = {
        use crate::issues::IssueBackend;
        match crate::issues::local_backend(&project_root).get(wi).await {
            Ok(Some(local)) if crate::cli::loop_state::parse_loop_state(&local.body).is_some() => {
                local.body
            }
            _ => issue.body.clone(),
        }
    };
    if let Some(loop_state) = crate::cli::loop_state::parse_loop_state(&loop_body) {
        return loop_state_envelope(root, &issue, &loop_state);
    }

    if issue.issue_type == IssueType::Epic {
        let project = project_from_labels(&issue).unwrap_or_else(|| "PROJECT".to_string());
        let command = format!("aw wi atomize --project {project}");
        WorkflowEnvelope {
            action: "dispatch".to_string(),
            root: WorkflowNode {
                kind: "epic".to_string(),
                id: issue_ref(&issue),
            },
            current: WorkflowNode {
                kind: "epic".to_string(),
                id: issue_ref(&issue),
            },
            completed: None,
            completion: wi_completion(
                false,
                false,
                vec!["epic is open and must be atomized into bounded change WIs".to_string()],
            ),
            next: WorkflowNext {
                kind: "atomize".to_string(),
                command: command.clone(),
                reason: "epic roots must be split into bounded change work-items".to_string(),
                payload_path: None,
            },
            invoke: WorkflowInvoke { command },
            agent_prompt: "Epic root is not atomic. Atomize it, then run each child change."
                .to_string(),
            requires_hitl: false,
            artifact_quality_profile: None,
            hitl_question: None,
            persistence: None,
        }
    } else {
        let command = format!("aw td create {}", issue_ref(&issue).trim_start_matches('#'));
        WorkflowEnvelope {
            action: "dispatch".to_string(),
            root,
            current: WorkflowNode {
                kind: "change".to_string(),
                id: issue_ref(&issue),
            },
            completed: None,
            completion: wi_completion(
                false,
                false,
                vec!["change work item is still open; run the TD/CB lifecycle".to_string()],
            ),
            next: WorkflowNext {
                kind: "execute_change".to_string(),
                command: command.clone(),
                reason: "atomic change roots enter the WI -> TD -> CB -> TD merge lifecycle"
                    .to_string(),
                payload_path: None,
            },
            invoke: WorkflowInvoke { command },
            agent_prompt:
                "Run the change lifecycle. When it completes, re-run the parent epic root."
                    .to_string(),
            requires_hitl: false,
            artifact_quality_profile: None,
            hitl_question: None,
            persistence: None,
        }
    }
}

/// #188 E1: build the run envelope from a WI's loop-state block. The loop's
/// `next_action` (set by `decide_next_action` from the latest verifier result)
/// is the authority: a command -> dispatch it (iterate or finalize); none ->
/// the loop is blocked / awaiting the human (HITL).
fn loop_state_envelope(
    root: WorkflowNode,
    issue: &Issue,
    loop_state: &crate::cli::loop_state::LoopState,
) -> WorkflowEnvelope {
    let current = WorkflowNode {
        kind: "change".to_string(),
        id: issue_ref(issue),
    };
    let slug = issue_cli_ref(issue);
    match loop_state.next_action.as_deref() {
        // #844/#845: a persisted `next_action` is a raw text template that
        // predates chain validation — it can be a bare, chain-invalid
        // `aw td code-check` (#844) or a stale legacy string like
        // `aw td merge` that no longer exists in the LINEAR lifecycle
        // (#845). Normalize/repair it before dispatching, or fall through to
        // a blocked/HITL envelope naming the bad command instead of running
        // it verbatim.
        Some(raw_command) => {
            match crate::cli::chain::normalize_legacy_next_action(raw_command, &slug) {
                Some(command) => {
                    let converged = matches!(
                        loop_state.status,
                        crate::cli::loop_state::LoopStatus::Converged
                    );
                    let reason = if converged {
                        "loop converged: ec is green — run the finalize act".to_string()
                    } else {
                        "loop iterating: ec is not green — run the next act".to_string()
                    };
                    WorkflowEnvelope {
                        action: "dispatch".to_string(),
                        root,
                        current,
                        completed: None,
                        completion: wi_completion(
                            false,
                            false,
                            vec![
                                "loop not yet terminal; ec verifier drives the next act"
                                    .to_string(),
                            ],
                        ),
                        next: WorkflowNext {
                            kind: "loop_act".to_string(),
                            command: command.clone(),
                            reason,
                            payload_path: None,
                        },
                        invoke: WorkflowInvoke { command },
                        agent_prompt: format!(
                            "Loop engine: run next.command, then re-run `aw wi run {slug}` to re-observe the loop state."
                        ),
                        requires_hitl: false,
                        artifact_quality_profile: None,
                        hitl_question: None,
                        persistence: None,
                    }
                }
                None => blocked_envelope(
                    root,
                    current,
                    format!("aw wi show {slug}"),
                    format!(
                        "loop next_action `{raw_command}` is not a runnable aw command; \
                         inspect and repair the WI's loop-state block"
                    ),
                    true,
                ),
            }
        }
        None => blocked_envelope(
            root,
            current,
            format!("aw wi show {slug}"),
            "loop has no next act: ec verifier is blocked or undefined — human input required"
                .to_string(),
            true,
        ),
    }
}

fn closed_wi_envelope(issue: &Issue) -> WorkflowEnvelope {
    let kind = if issue.issue_type == IssueType::Epic {
        "epic"
    } else {
        "change"
    };
    let node = WorkflowNode {
        kind: kind.to_string(),
        id: issue_ref(issue),
    };
    let command = parent_inspection_command(issue);
    WorkflowEnvelope {
        action: "done".to_string(),
        root: node.clone(),
        current: node.clone(),
        completed: Some(node.clone()),
        completion: wi_completion(true, false, Vec::new()),
        next: WorkflowNext {
            kind: "inspect_parent".to_string(),
            command: command.clone(),
            reason: format!("{kind} work item is closed; inspect the parent root for rollup"),
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt: if kind == "epic" {
            "Epic subtree is closed. Re-run the parent capability or project root.".to_string()
        } else {
            "Child change is closed. Re-run the parent epic root and continue rollup.".to_string()
        },
        requires_hitl: false,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: None,
    }
}

fn parent_inspection_command(issue: &Issue) -> String {
    if issue.issue_type == IssueType::Epic {
        if let Some(project) = project_from_labels(issue) {
            return project_capability_rollup_command(&project);
        }
    }
    issue
        .related
        .iter()
        .chain(issue.implements.iter())
        .find_map(|reference| extract_issue_number(reference))
        .map(|id| wi_run_command(&id))
        .unwrap_or_else(|| format!("aw wi show {}", issue_cli_ref(issue)))
}

fn extract_issue_number(reference: &str) -> Option<String> {
    let mut chars = reference.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '#' {
            continue;
        }
        let mut digits = String::new();
        while let Some(next) = chars.peek() {
            if next.is_ascii_digit() {
                digits.push(*next);
                chars.next();
            } else {
                break;
            }
        }
        if !digits.is_empty() {
            return Some(digits);
        }
    }
    None
}

async fn resolve_issue(wi: &str, project_root: &std::path::Path) -> Result<Option<Issue>> {
    let (kind, repo, host) = resolve_default_backend(project_root)?;
    let backend = make_backend(&kind, project_root, repo, host)?;
    backend.get(wi).await
}

fn capability_action_envelope(
    root: WorkflowNode,
    current: WorkflowNode,
    project: &str,
    action: &CapabilityAction,
    completion: WorkflowCompletion,
) -> WorkflowEnvelope {
    capability_action_envelope_with_planning_base(
        root,
        current,
        project,
        action,
        completion,
        Path::new("/tmp/aw/test/root"),
    )
}

fn capability_action_envelope_with_planning_base(
    root: WorkflowNode,
    current: WorkflowNode,
    project: &str,
    action: &CapabilityAction,
    mut completion: WorkflowCompletion,
    planning_base: &Path,
) -> WorkflowEnvelope {
    if matches!(
        action.kind,
        CapabilityActionKind::CreateWi | CapabilityActionKind::LinkClaimVerification
    ) {
        if let Some(path) =
            latest_pending_planning_artifact(planning_base, project, "epics", "epicize")
        {
            let reason = format!(
                "pending epic planning artifact requires review before creating or linking WIs: {}",
                path.display()
            );
            if !completion.missing.iter().any(|missing| missing == &reason) {
                completion.missing.push(reason.clone());
            }
            let command = project_capability_rollup_command(project);
            return WorkflowEnvelope {
                action: "blocked".to_string(),
                root,
                current,
                completed: None,
                completion,
                next: WorkflowNext {
                    kind: "review_planning_artifact".to_string(),
                    command: command.clone(),
                    reason: reason.clone(),
                    payload_path: Some(path.display().to_string()),
                },
                invoke: WorkflowInvoke { command },
                agent_prompt: "Review next.payload_path, capture the HITL decision, then re-run next.command to continue rollup.".to_string(),
                requires_hitl: true,
                artifact_quality_profile: None,
                hitl_question: Some(planning_artifact_hitl_question(project, &path, &reason)),
                persistence: None,
            };
        }
    }

    let (kind, command) = match action.kind {
        CapabilityActionKind::FormatMigrationRequired => (
            "capability",
            format!("aw capability run --project {project} --non-interactive"),
        ),
        CapabilityActionKind::CreateWi | CapabilityActionKind::LinkClaimVerification => {
            ("epicize", format!("aw wi epicize --project {project}"))
        }
        CapabilityActionKind::ReconcileWiRefs => {
            ("reconcile_wi_refs", agent_command(&action.command))
        }
        CapabilityActionKind::AtomizeWi => {
            ("atomize", format!("aw wi atomize --project {project}"))
        }
        CapabilityActionKind::RunTd | CapabilityActionKind::RunCb => {
            ("execute_change", agent_command(&action.command))
        }
        CapabilityActionKind::RunVerify => ("verify", agent_command(&action.command)),
        CapabilityActionKind::DefineCapabilityMap
        | CapabilityActionKind::HumanConfirmRequired
        | CapabilityActionKind::UpdateCapabilityStatus
        | CapabilityActionKind::EnvBlocked
        | CapabilityActionKind::StaleProjectConfig
        | CapabilityActionKind::DefineVerificationContract
        | CapabilityActionKind::AssignCapabilityType => ("hitl", agent_command(&action.command)),
        CapabilityActionKind::None => ("inspect_parent", String::new()),
    };
    let blocked = action.requires_hitl
        || matches!(
            action.kind,
            CapabilityActionKind::EnvBlocked | CapabilityActionKind::StaleProjectConfig
        );
    WorkflowEnvelope {
        action: if blocked { "blocked" } else { "dispatch" }.to_string(),
        root,
        current,
        completed: None,
        completion,
        next: WorkflowNext {
            kind: kind.to_string(),
            command: command.clone(),
            reason: action.reason.clone(),
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt: if blocked {
            "This root needs human confirmation or environment repair before continuing."
                .to_string()
        } else {
            "Run next.command. When that layer completes, re-run this root to continue rollup."
                .to_string()
        },
        requires_hitl: blocked,
        artifact_quality_profile: None,
        hitl_question: agent_hitl_question(action.hitl_question.clone()),
        persistence: None,
    }
}

fn latest_pending_planning_artifact(
    base: &Path,
    project: &str,
    bucket: &str,
    expected_kind: &str,
) -> Option<PathBuf> {
    let dir = base.join("aw").join(project).join(bucket);
    let mut candidates = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == std::ffi::OsStr::new("md"))
        })
        .filter(|path| planning_artifact_requires_review(path, expected_kind))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    candidates.pop()
}

fn planning_artifact_requires_review(path: &Path, expected_kind: &str) -> bool {
    let Ok(body) = std::fs::read_to_string(path) else {
        return false;
    };
    let Some(frontmatter) = planning_artifact_frontmatter(&body) else {
        return false;
    };
    let pending = frontmatter_string(&frontmatter, "kind")
        .is_some_and(|kind| kind == expected_kind)
        && frontmatter_bool(&frontmatter, "agent_review_required").unwrap_or(false)
        && frontmatter_string(&frontmatter, "review_status")
            .is_some_and(|status| status.eq_ignore_ascii_case("pending"));
    pending && !planning_artifact_is_noop(&frontmatter, &body, expected_kind)
}

fn planning_artifact_is_noop(
    frontmatter: &serde_yaml::Value,
    body: &str,
    expected_kind: &str,
) -> bool {
    if expected_kind != "epicize" || frontmatter_i64(frontmatter, "issue_count") != Some(0) {
        return false;
    }
    markdown_section_is_none(body, "Epic Candidates") && capability_epic_candidates_terminal(body)
}

fn planning_artifact_frontmatter(body: &str) -> Option<serde_yaml::Value> {
    let rest = body.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    serde_yaml::from_str(&rest[..end]).ok()
}

fn frontmatter_string<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a str> {
    value
        .as_mapping()?
        .get(&serde_yaml::Value::String(key.to_string()))?
        .as_str()
}

fn frontmatter_bool(value: &serde_yaml::Value, key: &str) -> Option<bool> {
    value
        .as_mapping()?
        .get(&serde_yaml::Value::String(key.to_string()))?
        .as_bool()
}

fn frontmatter_i64(value: &serde_yaml::Value, key: &str) -> Option<i64> {
    value
        .as_mapping()?
        .get(&serde_yaml::Value::String(key.to_string()))?
        .as_i64()
}

fn markdown_section_is_none(body: &str, heading: &str) -> bool {
    markdown_section(body, heading)
        .map(|section| {
            section
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .is_some_and(|line| line == "- none")
        })
        .unwrap_or(false)
}

fn capability_epic_candidates_terminal(body: &str) -> bool {
    let Some(section) = markdown_section(body, "Capability Epic Candidates") else {
        return true;
    };
    let mut saw_row = false;
    for line in section.lines().map(str::trim) {
        if !line.starts_with('|') || line.contains("|---") || line.contains("| Work Root |") {
            continue;
        }
        let cells = line
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim())
            .collect::<Vec<_>>();
        let Some(status) = cells.get(4) else {
            return false;
        };
        saw_row = true;
        if !matches!(
            status.trim_matches('`').to_ascii_lowercase().as_str(),
            "verified" | "closed" | "deferred" | "retired" | "out_of_scope" | "out of scope"
        ) {
            return false;
        }
    }
    saw_row
}

fn markdown_section<'a>(body: &'a str, heading: &str) -> Option<&'a str> {
    let marker = format!("## {heading}");
    let mut start = None;
    let mut end = body.len();
    let mut offset = 0usize;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
        if trimmed == marker {
            start = Some(offset + line.len());
        } else if start.is_some() && trimmed.starts_with("## ") {
            end = offset;
            break;
        }
        offset += line.len();
    }
    start.map(|start| &body[start..end])
}

// Kept test-only (issue #860 cleanup): the only production caller
// (project_backlog_envelope, part of the now-removed `aw run --project`
// root) was deleted; the direct `#[test]` coverage below is retained.
#[cfg(test)]
fn project_ready_wi_envelope(project: &str, root: WorkflowNode, issue: &Issue) -> WorkflowEnvelope {
    let wi = issue_ref(issue);
    let command = wi_run_command(wi.trim_start_matches('#'));
    WorkflowEnvelope {
        action: "dispatch".to_string(),
        root: root.clone(),
        current: WorkflowNode {
            kind: "change".to_string(),
            id: wi.clone(),
        },
        completed: None,
        completion: project_completion(
            false,
            vec![format!(
                "prioritize readiness selected {wi} as the next executable work item for project `{project}`"
            )],
        ),
        next: WorkflowNext {
            kind: "execute_change".to_string(),
            command: command.clone(),
            reason: "ready_now lane has an executable work item".to_string(),
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt:
            "Run next.command. When that work item completes, re-run this project root."
                .to_string(),
        requires_hitl: false,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: None,
    }
}

// Kept test-only (issue #860 cleanup): see project_ready_wi_envelope.
#[cfg(test)]
fn project_atomize_backlog_envelope(
    project: &str,
    root: WorkflowNode,
    lanes: &wi_cli::PrioritizeLanes,
) -> WorkflowEnvelope {
    let command = format!("aw wi atomize --project {project}");
    WorkflowEnvelope {
        action: "dispatch".to_string(),
        root: root.clone(),
        current: root,
        completed: None,
        completion: project_completion(
            false,
            vec![format!(
                "prioritize readiness found {} work item(s) that need atomization before execution",
                lanes.needs_atomize.len()
            )],
        ),
        next: WorkflowNext {
            kind: "atomize".to_string(),
            command: command.clone(),
            reason: "no ready_now work item is available until oversized or epic work is atomized"
                .to_string(),
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt:
            "Run next.command, review the atomization artifact, then re-run this project root."
                .to_string(),
        requires_hitl: false,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: None,
    }
}

// Kept test-only (issue #860 cleanup): see project_ready_wi_envelope.
#[cfg(test)]
fn project_prioritize_blocked_envelope(
    project: &str,
    root: WorkflowNode,
    lanes: &wi_cli::PrioritizeLanes,
) -> WorkflowEnvelope {
    let command = format!("aw wi prioritize --project {project}");
    let reason = format!(
        "prioritize readiness has no ready_now work: blocked_by_dependency={}, needs_triage={}, deferred={}",
        lanes.blocked_by_dependency.len(),
        lanes.needs_triage.len(),
        lanes.deferred.len()
    );
    WorkflowEnvelope {
        action: "blocked".to_string(),
        root: root.clone(),
        current: root,
        completed: None,
        completion: project_completion(false, vec![reason.clone()]),
        next: WorkflowNext {
            kind: "prioritize".to_string(),
            command: command.clone(),
            reason,
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt:
            "Run next.command to inspect readiness lanes, then resolve dependency, triage, or deferred blockers."
                .to_string(),
        requires_hitl: true,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: None,
    }
}

fn planning_artifact_hitl_question(project: &str, path: &Path, reason: &str) -> HitlQuestion {
    HitlQuestion {
        id: format!("planning:{}:epicize_review", project),
        question: format!(
            "Review pending epic planning artifact `{}` before Agentic Workflow creates or links WIs?",
            path.display()
        ),
        target: path.display().to_string(),
        resume_command: project_capability_rollup_command(project),
        tool_hint: "ask_user_question".to_string(),
        choices: vec![
            HitlChoice {
                id: "approve_epic_plan".to_string(),
                label: "Approve epic plan".to_string(),
                description:
                    "Accept this local artifact as reviewed input before publishing tracker changes."
                        .to_string(),
            },
            HitlChoice {
                id: "revise_epic_plan".to_string(),
                label: "Revise epic plan".to_string(),
                description:
                    "Adjust the candidate grouping, scope, or deferred work before continuing."
                        .to_string(),
            },
            HitlChoice {
                id: "regenerate_epic_plan".to_string(),
                label: "Regenerate plan".to_string(),
                description: "Discard this pending review artifact and run epicize again."
                    .to_string(),
            },
        ],
        default_choice: Some("approve_epic_plan".to_string()),
        freeform_prompt: Some(reason.to_string()),
    }
}

// Kept test-only (issue #860 cleanup): the only production caller was inside
// the now-deleted project_envelope tree; direct `#[test]` coverage and the
// project_done_or_dirty_envelope_with_health/project_production_blocked_envelope
// test-twin helpers are retained.
#[cfg(test)]
fn project_completion(root_complete: bool, missing: Vec<String>) -> WorkflowCompletion {
    WorkflowCompletion {
        root_complete,
        workflow_complete: root_complete,
        criteria: vec![
            "README capability map is Markdown-table runnable".to_string(),
            "current production scope is resolved from Production=ready capabilities".to_string(),
            "scoped capability dependency closure is production ready".to_string(),
            "scoped capability claims and gates are verified".to_string(),
            "declared gates and TD/WI refs have no blockers".to_string(),
            "prioritize readiness has no executable or blocked open work-items".to_string(),
            "configured production test gates passed".to_string(),
            "project health production_ready is true".to_string(),
        ],
        missing,
    }
}

#[cfg(test)]
fn project_done_or_dirty_envelope_with_health<F>(
    project: &str,
    root: WorkflowNode,
    project_root: &Path,
    mut build_health: F,
) -> WorkflowEnvelope
where
    F: FnMut() -> Result<crate::cli::project::ProjectHealthReport>,
{
    match project_repo_side_dirty_paths_at(project_root, project) {
        Ok((paths, scopes)) if paths.is_empty() => match build_health() {
            Ok(health) if health.production_ready => project_done_envelope(root, scopes),
            Ok(health) => project_production_blocked_envelope(project, root, health, scopes),
            Err(err) => blocked_envelope(
                root.clone(),
                root,
                project_capability_rollup_command(project),
                format!("project production readiness guard failed: {err}"),
                true,
            ),
        },
        Ok((paths, scopes)) => {
            match commit_project_persistence_if_approved(project_root, project, &paths, &scopes) {
                Ok(true) => match project_repo_side_dirty_paths_at(project_root, project) {
                    Ok((remaining_paths, scopes)) if remaining_paths.is_empty() => {
                        match build_health() {
                            Ok(health) if health.production_ready => {
                                project_done_envelope(root, scopes)
                            }
                            Ok(health) => {
                                project_production_blocked_envelope(project, root, health, scopes)
                            }
                            Err(err) => blocked_envelope(
                                root.clone(),
                                root,
                                project_capability_rollup_command(project),
                                format!("project production readiness guard failed: {err}"),
                                true,
                            ),
                        }
                    }
                    Ok((remaining_paths, scopes)) => {
                        persistence_blocked_envelope(project, root, remaining_paths, scopes)
                    }
                    Err(err) => blocked_envelope(
                        root.clone(),
                        root,
                        project_capability_rollup_command(project),
                        format!("repo persistence guard failed after commit: {err}"),
                        true,
                    ),
                },
                Ok(false) => persistence_blocked_envelope(project, root, paths, scopes),
                Err(err) => blocked_envelope(
                    root.clone(),
                    root,
                    project_capability_rollup_command(project),
                    format!("repo persistence commit failed: {err}"),
                    true,
                ),
            }
        }
        Err(err) => blocked_envelope(
            root.clone(),
            root,
            project_capability_rollup_command(project),
            format!("repo persistence guard failed: {err}"),
            true,
        ),
    }
}

#[cfg(test)]
fn project_production_blocked_envelope(
    project: &str,
    root: WorkflowNode,
    health: crate::cli::project::ProjectHealthReport,
    scopes: Vec<String>,
) -> WorkflowEnvelope {
    let command = format!("aw health --project {project}");
    let reason = if health.production_blockers.is_empty() {
        "project production readiness is blocked".to_string()
    } else {
        format!(
            "project production readiness is blocked: {}",
            health
                .production_blockers
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        )
    };
    WorkflowEnvelope {
        action: "blocked".to_string(),
        root: root.clone(),
        current: root,
        completed: None,
        completion: project_completion(false, health.production_blockers.clone()),
        next: WorkflowNext {
            kind: "production_verification".to_string(),
            command: command.clone(),
            reason: reason.clone(),
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt:
            "Run next.command to inspect scoped production readiness, then resolve the listed TD/CB/test/semantic blockers through existing AW lifecycle commands."
                .to_string(),
        requires_hitl: true,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: Some(WorkflowPersistence {
            status: "production_blocked".to_string(),
            commit_complete: true,
            wi_evidence_complete: true,
            dirty_paths: Vec::new(),
            scopes,
            reason,
        }),
    }
}

fn capability_production_blocked_envelope(
    project: &str,
    capability_id: &str,
    root: WorkflowNode,
    mut capability_blockers: Vec<String>,
    global_blockers: Vec<String>,
) -> WorkflowEnvelope {
    capability_blockers.extend(global_blockers);
    capability_blockers.sort();
    capability_blockers.dedup();
    let command = format!("aw capability check --project {project} --verify");
    let reason = if capability_blockers.is_empty() {
        format!("capability `{capability_id}` is not production ready")
    } else {
        format!(
            "capability `{capability_id}` is not production ready: {}",
            capability_blockers
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        )
    };
    WorkflowEnvelope {
        action: "blocked".to_string(),
        root: root.clone(),
        current: root,
        completed: None,
        completion: capability_completion(false, capability_blockers),
        next: WorkflowNext {
            kind: "production_verification".to_string(),
            command: command.clone(),
            reason: reason.clone(),
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt:
            "Run next.command to inspect scoped capability production readiness, then resolve listed blockers through existing AW lifecycle commands."
                .to_string(),
        requires_hitl: true,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: None,
    }
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn project_done_envelope(root: WorkflowNode, scopes: Vec<String>) -> WorkflowEnvelope {
    WorkflowEnvelope {
        action: "done".to_string(),
        current: root.clone(),
        completed: Some(root.clone()),
        completion: project_completion(true, Vec::new()),
        next: WorkflowNext {
            kind: "inspect_parent".to_string(),
            command: String::new(),
            reason: "scoped production readiness is complete".to_string(),
            payload_path: None,
        },
        invoke: WorkflowInvoke {
            command: String::new(),
        },
        agent_prompt: "Project root is complete; no parent scope remains.".to_string(),
        requires_hitl: false,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: Some(WorkflowPersistence {
            status: "complete".to_string(),
            commit_complete: true,
            wi_evidence_complete: true,
            dirty_paths: Vec::new(),
            scopes,
            reason: "AW-owned repo scopes are clean; no milestone commit is pending".to_string(),
        }),
        root,
    }
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn persistence_blocked_envelope(
    project: &str,
    root: WorkflowNode,
    dirty_paths: Vec<String>,
    scopes: Vec<String>,
) -> WorkflowEnvelope {
    let command = project_capability_rollup_command(project);
    let reason = format!(
        "repo-side lifecycle changes are uncommitted under AW-owned scopes: {}",
        dirty_paths.join(", ")
    );
    WorkflowEnvelope {
        action: "blocked".to_string(),
        root: root.clone(),
        current: root,
        completed: None,
        completion: blocked_completion(reason.clone()),
        next: WorkflowNext {
            kind: "milestone_persistence".to_string(),
            command: command.clone(),
            reason: reason.clone(),
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt: "Repo-side lifecycle changes remain dirty inside AW-owned scopes. The mutating AW CLI step that produced them must own the commit; do not invent a separate commit CLI.".to_string(),
        requires_hitl: true,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: Some(WorkflowPersistence {
            status: "repo_dirty".to_string(),
            commit_complete: false,
            wi_evidence_complete: false,
            dirty_paths,
            scopes,
            reason,
        }),
    }
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn project_repo_side_dirty_paths_at(
    project_root: &Path,
    project: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    let scopes = project_repo_side_scopes(project_root, project)?;
    let dirty_paths = crate::git::dirty_paths(project_root, &scopes, true)?;
    Ok((dirty_paths, scope_strings(project_root, &scopes)))
}

// Kept test-only (issue #860 cleanup): retains its own direct `#[test]`
// coverage in addition to the project_repo_side_dirty_paths_at caller above.
#[cfg(test)]
fn project_repo_side_scopes(project_root: &Path, project: &str) -> Result<Vec<PathBuf>> {
    let config = load_run_project_config(project_root)?;
    let Some(row) = config.projects.iter().find(|row| row.matches(project)) else {
        anyhow::bail!("project `{project}` is not configured in aw.toml");
    };

    let mut scopes = Vec::new();
    if let Some(cap_path) = row.cap_path.as_deref() {
        scopes.push(project_root.join(cap_path));
    } else if let Some(path) = row.path.as_deref() {
        scopes.push(project_root.join(path).join("README.md"));
    }
    if let Some(td_path) = row.td_path.as_deref() {
        scopes.push(project_root.join(td_path));
    } else if let Some(path) = row.path.as_deref() {
        scopes.push(
            project_root.join(crate::services::project_registry::default_project_td_path(
                path,
            )),
        );
    }
    if let Some(path) = row.path.as_deref() {
        scopes.push(project_root.join(path));
    }
    scopes.sort();
    scopes.dedup();
    Ok(scopes)
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ProjectPersistenceRequest {
    project: String,
    project_root: String,
    dirty_paths: Vec<String>,
    scopes: Vec<String>,
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn commit_project_persistence_if_approved(
    project_root: &Path,
    project: &str,
    dirty_paths: &[String],
    scopes: &[String],
) -> Result<bool> {
    let request = ProjectPersistenceRequest {
        project: project.to_string(),
        project_root: project_root.display().to_string(),
        dirty_paths: dirty_paths.to_vec(),
        scopes: scopes.to_vec(),
    };
    let path = project_persistence_request_path(project_root, project);
    if let Ok(raw) = fs::read_to_string(&path) {
        if serde_json::from_str::<ProjectPersistenceRequest>(&raw).ok() == Some(request.clone()) {
            let paths = dirty_paths
                .iter()
                .map(|path| project_root.join(path))
                .collect::<Vec<_>>();
            let message = format!(
                "aw capability run({project}) lifecycle persistence\n\nProject: {project}\nLifecycle-Stage: Project-Persistence\nDirty-Paths: {}\n",
                dirty_paths.join(", ")
            );
            crate::git::commit_scoped_paths(project_root, &paths, &message)?;
            let _ = fs::remove_file(&path);
            return Ok(true);
        }
    }
    write_project_persistence_request(&path, &request)?;
    Ok(false)
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn project_persistence_request_path(project_root: &Path, project: &str) -> PathBuf {
    crate::shared::workspace::aw_tmp_path()
        .join(project)
        .join("run")
        .join(format!(
            "project-persistence-{}.json",
            stable_project_root_hash(project_root)
        ))
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn stable_project_root_hash(project_root: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    project_root.display().to_string().hash(&mut hasher);
    hasher.finish()
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn write_project_persistence_request(
    path: &Path,
    request: &ProjectPersistenceRequest,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create project persistence directory {}",
                parent.display()
            )
        })?;
    }
    let raw =
        serde_json::to_vec_pretty(request).context("serialize project persistence request")?;
    fs::write(path, raw).with_context(|| {
        format!(
            "failed to write project persistence request {}",
            path.display()
        )
    })?;
    Ok(())
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn load_run_project_config(project_root: &Path) -> Result<RunProjectConfig> {
    let config_path = project_root.join("aw.toml");
    let content = std::fs::read_to_string(&config_path)?;
    toml::from_str(&content).with_context(|| format!("parse {}", config_path.display()))
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
fn scope_strings(project_root: &Path, scopes: &[PathBuf]) -> Vec<String> {
    scopes
        .iter()
        .map(|scope| scope.strip_prefix(project_root).unwrap_or(scope))
        .map(|scope| scope.to_string_lossy().to_string())
        .collect()
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
#[derive(Debug, Default, Deserialize)]
struct RunProjectConfig {
    #[serde(default)]
    projects: Vec<RunProjectRow>,
}

// Kept test-only (issue #860 cleanup): see project_completion.
#[cfg(test)]
#[derive(Debug, Default, Deserialize)]
struct RunProjectRow {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    td_path: Option<String>,
    #[serde(default)]
    cap_path: Option<String>,
}

/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
#[cfg(test)]
impl RunProjectRow {
    fn matches(&self, project: &str) -> bool {
        self.name == project || self.aliases.iter().any(|alias| alias == project)
    }
}

fn capability_completion(root_complete: bool, missing: Vec<String>) -> WorkflowCompletion {
    WorkflowCompletion {
        root_complete,
        workflow_complete: false,
        criteria: vec![
            "capability production scope is resolved".to_string(),
            "capability and dependency closure are production ready".to_string(),
            "required claims and gates are verified".to_string(),
            "TD/WI refs resolve without blockers".to_string(),
        ],
        missing,
    }
}

fn wi_completion(
    root_complete: bool,
    workflow_complete: bool,
    missing: Vec<String>,
) -> WorkflowCompletion {
    WorkflowCompletion {
        root_complete,
        workflow_complete,
        criteria: vec![
            "work item is closed".to_string(),
            "parent root can be inspected for rollup".to_string(),
        ],
        missing,
    }
}

fn blocked_completion(reason: String) -> WorkflowCompletion {
    WorkflowCompletion {
        root_complete: false,
        workflow_complete: false,
        criteria: vec!["blocker is resolved".to_string()],
        missing: vec![reason],
    }
}

fn capability_missing(
    item: &capability::CapabilityReportItem,
    action: &CapabilityAction,
) -> Vec<String> {
    let mut missing = Vec::new();
    if item.status != CapabilityStatus::Verified {
        missing.push(format!(
            "capability `{}` status is {:?}",
            item.id, item.status
        ));
    }
    for gap in &item.gaps {
        if !matches!(
            gap.status,
            capability::CapabilityGapStatus::Closed | capability::CapabilityGapStatus::Deferred
        ) {
            missing.push(format!("gap `{}` is {:?}", gap.id, gap.status));
        }
    }
    for claim in &item.claims {
        if claim.required_for_verified && !claim.verified {
            missing.push(format!("claim `{}` is not verified", claim.id));
        }
    }
    if !action.reason.trim().is_empty() {
        missing.push(action.reason.clone());
    }
    missing.sort();
    missing.dedup();
    missing
}

fn blocked_envelope(
    root: WorkflowNode,
    current: WorkflowNode,
    command: String,
    reason: String,
    requires_hitl: bool,
) -> WorkflowEnvelope {
    WorkflowEnvelope {
        action: "blocked".to_string(),
        root,
        current,
        completed: None,
        completion: blocked_completion(reason.clone()),
        next: WorkflowNext {
            kind: "hitl".to_string(),
            command: command.clone(),
            reason,
            payload_path: None,
        },
        invoke: WorkflowInvoke { command },
        agent_prompt: "Resolve the blocker, then re-run the same workflow root.".to_string(),
        requires_hitl,
        artifact_quality_profile: None,
        hitl_question: None,
        persistence: None,
    }
}

fn ensure_hitl_question(envelope: &mut WorkflowEnvelope, root_command: &str) {
    if !envelope.requires_hitl || envelope.hitl_question.is_some() {
        return;
    }
    envelope.hitl_question = Some(generic_hitl_question(envelope, root_command));
}

fn generic_hitl_question(envelope: &WorkflowEnvelope, root_command: &str) -> HitlQuestion {
    let next_command = if envelope.next.command.trim().is_empty() {
        root_command.to_string()
    } else {
        envelope.next.command.clone()
    };
    let reason = envelope.next.reason.trim();
    HitlQuestion {
        id: format!(
            "aw-run:{}:{}:hitl",
            slug_for_goal_path(&envelope.root.kind),
            slug_for_goal_path(&envelope.root.id)
        ),
        question: if reason.is_empty() {
            format!(
                "Agentic Workflow root `{}:{}` requires human input before continuing. Should the agent run `{}` or pause?",
                envelope.root.kind, envelope.root.id, next_command
            )
        } else {
            format!(
                "Agentic Workflow root `{}:{}` is blocked: {reason}. Should the agent run `{}` or pause?",
                envelope.root.kind, envelope.root.id, next_command
            )
        },
        target: format!("{}:{}", envelope.current.kind, envelope.current.id),
        resume_command: root_command.to_string(),
        tool_hint: "ask_user_question".to_string(),
        choices: vec![
            HitlChoice {
                id: "run_next_command".to_string(),
                label: "Run next command".to_string(),
                description:
                    "Allow the agent to run the suggested command, then re-run the workflow root."
                        .to_string(),
            },
            HitlChoice {
                id: "repair_environment".to_string(),
                label: "Repair environment".to_string(),
                description:
                    "Fix missing auth, permissions, or local state before resuming this root."
                        .to_string(),
            },
            HitlChoice {
                id: "pause".to_string(),
                label: "Pause".to_string(),
                description: "Stop unattended work and wait for explicit user direction."
                    .to_string(),
            },
        ],
        default_choice: Some("run_next_command".to_string()),
        freeform_prompt: (!envelope.completion.missing.is_empty()).then(|| {
            envelope
                .completion
                .missing
                .iter()
                .take(10)
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
                .join("\n")
        }),
    }
}

fn issue_ref(issue: &Issue) -> String {
    issue
        .github_id
        .or(issue.gitlab_id)
        .map(|id| format!("#{id}"))
        .unwrap_or_else(|| issue.slug.clone())
}

fn issue_cli_ref(issue: &Issue) -> String {
    issue
        .github_id
        .or(issue.gitlab_id)
        .map(|id| id.to_string())
        .unwrap_or_else(|| issue.slug.clone())
}

fn project_from_labels(issue: &Issue) -> Option<String> {
    issue.labels.iter().find_map(|label| {
        label
            .strip_prefix("app:")
            .or_else(|| label.strip_prefix("lib:"))
            .map(|project| project.to_string())
    })
}

fn print_text(envelope: &WorkflowEnvelope) {
    println!(
        "{} {}:{}",
        envelope.action, envelope.root.kind, envelope.root.id
    );
    println!(
        "completion: root_complete={} workflow_complete={}",
        envelope.completion.root_complete, envelope.completion.workflow_complete
    );
    for missing in &envelope.completion.missing {
        println!("- missing: {missing}");
    }
    if !envelope.next.command.is_empty() {
        println!("next: {}", envelope.next.command);
    }
    if let Some(question) = &envelope.hitl_question {
        println!("hitl_question: {}", question.question);
    }
    println!("reason: {}", envelope.next.reason);
    println!("{}", envelope.agent_prompt);
}

fn print_goal_text(goal: &WorkflowGoalEnvelope) {
    println!("goal_prompt {}", goal.payload_path);
    println!("root: {}", goal.root_command);
    if let Some(command) = &goal.first_next.command {
        println!("first: {command}");
    }
    println!("prompt_size_bytes: {}", goal.prompt_size_bytes);
    if let Some(prompt) = &goal.goal_prompt {
        println!();
        println!("{prompt}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    fn init_git_repo(root: &std::path::Path) {
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
            vec!["add", "."],
            vec!["commit", "--allow-empty", "-m", "init", "-q"],
        ] {
            let out = std::process::Command::new("git")
                .args(&args)
                .current_dir(root)
                .output()
                .expect("git command");
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }

    fn git_stdout(root: &std::path::Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git command");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn project_health_fixture(
        project: &str,
        production_ready: bool,
        production_blockers: Vec<&str>,
    ) -> crate::cli::project::ProjectHealthReport {
        crate::cli::project::ProjectHealthReport {
            project: project.to_string(),
            status: if production_ready {
                crate::cli::project::ProjectHealthStatus::Healthy
            } else {
                crate::cli::project::ProjectHealthStatus::Blocked
            },
            capability_ready: true,
            managed_ready: true,
            semantic_ready: true,
            traceability_ready: true,
            takeover_ready: true,
            generator_request_ready: true,
            production_ready,
            production_status: if production_ready {
                crate::cli::production::ProductionStatus::Ready
            } else {
                crate::cli::production::ProductionStatus::Blocked
            },
            production_scope: vec!["core".to_string()],
            production_blockers: production_blockers
                .into_iter()
                .map(|blocker| blocker.to_string())
                .collect(),
            global_blockers: Vec::new(),
            scoped_capabilities: Vec::new(),
            capability: crate::cli::project::CapabilityHealthReport {
                evaluated: true,
                production_evaluated: true,
                note: None,
                cap_path: "projects/demo/README.md".to_string(),
                format: "markdown_tables".to_string(),
                format_version: 2,
                capability_count: 1,
                release_scope_count: 1,
                root_runner_ready: true,
                production_ready_count: usize::from(production_ready),
                production_scope_count: 1,
                production_percent: if production_ready { 100.0 } else { 0.0 },
                blocker_count: 0,
                blockers: Vec::new(),
            },
            test_gates: crate::cli::project::ProjectTestGateReport::passed_fixture("true"),
            ec: crate::cli::project::ProjectEcGateReport::not_evaluated(project),
            claim_closure: crate::cli::project::ProjectClaimClosureReport::not_evaluated(project),
            preflight_gate_reports: Vec::new(),
            optional_quality_warnings: Vec::new(),
            managed_percent: 100.0,
            semantic_percent: 100.0,
            codegen_percent: 100.0,
            codegen_eligible_files: 1,
            codegen_files: 1,
            cb_ownership: crate::cli::project::CbOwnershipSummary {
                eligible_files: 1,
                codegen_files: 1,
                handwrite_files: 0,
                unmarked_files: 0,
                codegen_percent: 100.0,
                handwrite_percent: 0.0,
                unmarked_percent: 0.0,
            },
            codegen_origin: crate::cli::cb::CbCodegenOriginSummary {
                target_files: 1,
                td_ast_files: 1,
                artifact_replay_files: 0,
                source_template_files: 0,
            },
            traceability_evaluated: true,
            traceability_note: None,
            traceability_percent: 100.0,
            traceability_blocker_count: 0,
            traceability_internal_td_count: 0,
            traceability_orphan_td_count: 0,
            command_traceability_percent: 100.0,
            command_traceability_blocker_count: 0,
            command_traceability_hidden_command_count: 0,
            command_traceability_orphan_command_count: 0,
            traceability: crate::cli::standardize::TraceabilityCoverage::ready_fixture(project),
            next_gap: None,
            blocked_gap_count: 0,
            human_decision_required_count: 0,
            handwrite_files: 0,
            unmarked_files: 0,
            cb_verify_evaluated: true,
            cb_verify_note: None,
            cb_verify_clean: true,
            public_api_covered: 0,
            public_api_total: 0,
            semantic_review_required: 0,
            cold_rebuild_evaluated: true,
            cold_rebuild_note: None,
            cold_rebuild_clean: true,
            cold_rebuild_workspace_count: 0,
            cold_rebuild_failures: Vec::new(),
            cold_rebuilds: Vec::new(),
            stack_migration_percent: 100.0,
            stack_migration_incomplete_workspaces: 0,
            stack_migration: crate::cli::standardize::StackMigrationCoverage {
                project: project.to_string(),
                workspaces: Vec::new(),
                migration_normalized_percent: 100.0,
                incomplete_workspace_count: 0,
                dependency_policy_blockers: Vec::new(),
                deployment_policy_blockers: Vec::new(),
                blockers: Vec::new(),
            },
            workflow_lock_count: 0,
            td_lock: crate::cli::td_lock::TdLockStatus::ready_fixture(project),
            regenerability_authority: crate::cli::project::RegenerabilityAuthorityReport {
                authority:
                    crate::cli::regenerability_policy::RegenerabilityAuthority::ExternalAdvisory,
                required_for_production: false,
                gap_count: 0,
                reason: "test fixture".to_string(),
                blockers: Vec::new(),
                advisory_gaps: Vec::new(),
            },
            optional_regenerability_gaps: Vec::new(),
            blockers: Vec::new(),
            managed_next_uncovered_file: None,
            drift_marker: crate::cli::standardize::DriftMarkerCoverage {
                project: project.to_string(),
                clean_files: 1,
                drift_count: 0,
                marker_gap_count: 0,
                uncovered_count: 0,
                findings: Vec::new(),
            },
            takeover_audit: crate::cli::standardize::TakeoverAuditCoverage {
                project: project.to_string(),
                recorded: false,
                audit_path: String::new(),
                surfaces_to_preserve: Vec::new(),
                quality_debt_count: 0,
                safe_lever_count: 0,
            },
        }
    }

    fn open_issue(issue_type: IssueType, number: u64) -> Issue {
        Issue {
            issue_type,
            title: format!("Work item {number}"),
            state: crate::issues::IssueState::Open,
            id: None,
            github_id: Some(number),
            gitlab_id: None,
            url: None,
            author: None,
            created_at: None,
            updated_at: None,
            slug: number.to_string(),
            body: String::new(),
            labels: vec!["app:jet".to_string()],
            related: Vec::new(),
            implements: Vec::new(),
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
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    fn assert_no_removed_wi_verbs(envelope: &WorkflowEnvelope) {
        let serialized = serde_json::to_string(envelope).unwrap();
        assert!(!serialized.contains("aw wi estimate"));
        assert!(!serialized.contains("aw wi sprintize"));
    }

    #[test]
    fn loop_state_envelope_dispatches_or_blocks_on_next_action() {
        use crate::cli::loop_state::{LoopState, LoopStatus};
        let issue = open_issue(IssueType::Enhancement, 188);
        let root = WorkflowNode {
            kind: "change".to_string(),
            id: issue_ref(&issue),
        };

        // Iterating with a command -> the loop engine dispatches the act.
        let s = LoopState {
            status: LoopStatus::Iterating,
            next_action: Some("aw td gen".to_string()),
            ..Default::default()
        };
        let e = loop_state_envelope(root.clone(), &issue, &s);
        assert_eq!(e.action, "dispatch");
        assert_eq!(e.next.command, "aw td gen");
        assert_eq!(e.invoke.command, "aw td gen");
        assert!(!e.completion.workflow_complete);
        assert!(!e.requires_hitl);

        // No next act -> blocked / HITL (the single human gate).
        let s = LoopState {
            status: LoopStatus::Blocked,
            next_action: None,
            ..Default::default()
        };
        let e = loop_state_envelope(root, &issue, &s);
        assert_eq!(e.action, "blocked");
        assert!(e.requires_hitl);
    }

    // #844/#845: a persisted `next_action` predating chain validation must be
    // normalized/repaired before dispatch, not run verbatim.
    #[test]
    fn loop_state_envelope_repairs_legacy_next_action() {
        use crate::cli::loop_state::{LoopState, LoopStatus};
        let issue = open_issue(IssueType::Enhancement, 188);
        let root = WorkflowNode {
            kind: "change".to_string(),
            id: issue_ref(&issue),
        };

        // #844: a persisted bare `aw td code-check` must pick up the WI's
        // slug rather than dispatch a whole-tree audit.
        let s = LoopState {
            status: LoopStatus::Converged,
            next_action: Some("aw td code-check".to_string()),
            ..Default::default()
        };
        let e = loop_state_envelope(root.clone(), &issue, &s);
        assert_eq!(e.action, "dispatch");
        assert_eq!(e.next.command, "aw td code-check 188");
        assert_eq!(e.invoke.command, "aw td code-check 188");

        // #845: `aw td merge` was removed from the LINEAR lifecycle; a stale
        // persisted string must repair to the current terminal step.
        let s = LoopState {
            status: LoopStatus::Converged,
            next_action: Some("aw td merge".to_string()),
            ..Default::default()
        };
        let e = loop_state_envelope(root.clone(), &issue, &s);
        assert_eq!(e.action, "dispatch");
        assert_eq!(e.next.command, "aw td code-check 188");

        // An unparseable/unrepairable command must not dispatch verbatim —
        // fall through to blocked/HITL naming the bad command.
        let s = LoopState {
            status: LoopStatus::Iterating,
            next_action: Some("aw bogus-verb".to_string()),
            ..Default::default()
        };
        let e = loop_state_envelope(root, &issue, &s);
        assert_eq!(e.action, "blocked");
        assert!(e.requires_hitl);
        assert!(e.next.command.contains("aw wi show 188"));
    }

    #[test]
    fn agent_command_drops_legacy_json_and_default_tick_flags() {
        assert_eq!(
            agent_command("aw run --project jet --max-ticks 1 --json"),
            "aw run --project jet"
        );
        assert_eq!(
            agent_command("aw capability check --project jet --json --verify"),
            "aw capability check --project jet --verify"
        );
    }

    #[test]
    fn workflow_goal_prompt_encodes_run_loop_contract() {
        let envelope = test_envelope("aw td create 3903", "next TD section");
        let prompt = workflow_goal_prompt(&envelope, "aw run --project demo");

        assert!(prompt.contains("completion.workflow_complete=true"));
        assert!(prompt.contains("requires_hitl=true"));
        assert!(prompt.contains("Do not treat `action=done` alone as root completion"));
        assert!(prompt.contains("Start with this first command: `aw td create 3903`"));
        assert!(!prompt.contains("aw td create 3903 --json"));
        assert!(!prompt.contains("aw run --project demo --json"));
    }

    #[test]
    fn workflow_goal_envelope_writes_tmp_payload_and_inlines_small_prompt() {
        let envelope = test_envelope("aw td create 3903", "next TD section");
        let goal = workflow_goal_envelope(&envelope, "aw run --project demo").unwrap();

        assert_eq!(goal.action, "goal_prompt");
        let json = serde_json::to_value(&goal).unwrap();
        assert_eq!(json["schema_version"], "aw.cli.v1");
        assert_eq!(json["status"], "continue");
        assert_eq!(json["completion"]["requires_hitl"], false);
        assert_eq!(json["first_next"]["kind"], "run_command");
        assert_eq!(json["first_next"]["command"], "aw td create 3903");
        assert!(json.get("first_invoke").is_none());
        assert!(json.get("requires_hitl").is_none());
        assert_eq!(goal.payload_path, "/tmp/aw/goals/aw-run-project-demo.md");
        assert!(goal.goal_prompt.is_some());
        let payload = std::fs::read_to_string(&goal.payload_path).unwrap();
        assert!(payload.contains("Drive the Agentic Workflow root `project:demo`"));
        assert!(payload.contains("aw run --project demo"));
    }

    #[test]
    fn workflow_goal_envelope_omits_inline_large_prompt() {
        let mut envelope = test_envelope("aw td create 3903", "next TD section");
        envelope.completion.missing = vec!["large missing criterion ".repeat(300)];

        let goal = workflow_goal_envelope(&envelope, "aw run --project demo").unwrap();

        assert!(goal.prompt_size_bytes > goal.inline_limit_bytes);
        assert!(goal.goal_prompt.is_none());
        assert_eq!(goal.payload_path, "/tmp/aw/goals/aw-run-project-demo.md");
    }

    #[test]
    fn capability_root_unrunnable_blocks_without_invalid_next_command() {
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "cap".to_string(),
        };

        let envelope = capability_root_unrunnable_envelope(
            root.clone(),
            root,
            anyhow::anyhow!("failed to parse capability map"),
        );

        assert_eq!(envelope.action, "blocked");
        assert_eq!(envelope.next.command, "");
        assert!(!envelope.requires_hitl);
        assert_eq!(
            envelope.next.reason,
            "capability root is not runnable: failed to parse capability map"
        );
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["next"]["kind"], "blocked");
        assert!(json["next"].get("command").is_none());
        assert!(json.get("hitl_question").is_none());
    }

    #[test]
    fn project_repo_side_scopes_include_default_td_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".aw")).unwrap();
        std::fs::write(
            root.join("aw.toml"),
            r#"
[[projects]]
name = "jet"
aliases = ["j"]
path = "apps/jet"
cap_path = "apps/jet/README.md"
"#,
        )
        .unwrap();

        let scopes = project_repo_side_scopes(root, "j").unwrap();

        assert!(scopes.contains(&root.join("apps/jet/README.md")));
        assert!(scopes.contains(&root.join("apps/jet/tech-design")));
        assert!(scopes.contains(&root.join("apps/jet")));
    }

    #[test]
    fn hitl_envelope_gets_agent_question_before_serialization() {
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "agentic-workflow".to_string(),
        };
        let mut envelope = blocked_envelope(
            root.clone(),
            root,
            "aw wi prioritize --project agentic-workflow".to_string(),
            "work-item readiness inventory is unavailable".to_string(),
            true,
        );

        ensure_hitl_question(&mut envelope, "aw run --project agentic-workflow");

        let question = envelope.hitl_question.as_ref().expect("hitl question");
        assert_eq!(question.tool_hint, "ask_user_question");
        assert_eq!(question.resume_command, "aw run --project agentic-workflow");
        assert_eq!(question.default_choice.as_deref(), Some("run_next_command"));

        let json = serde_json::to_value(envelope).unwrap();
        assert_eq!(json["completion"]["requires_hitl"], true);
        assert_eq!(
            json["hitl_question"]["resume_command"],
            "aw run --project agentic-workflow"
        );
        assert!(json["hitl_question"]["question"]
            .as_str()
            .unwrap()
            .contains("aw wi prioritize --project agentic-workflow"));
    }

    #[test]
    fn epic_issue_dispatches_atomize() {
        let issue = Issue {
            issue_type: IssueType::Epic,
            title: "Py312 Compatible".to_string(),
            state: crate::issues::IssueState::Open,
            id: None,
            github_id: Some(4100),
            gitlab_id: None,
            url: None,
            author: None,
            created_at: None,
            updated_at: None,
            slug: "4100".to_string(),
            body: String::new(),
            labels: vec!["app:mamba".to_string()],
            related: Vec::new(),
            implements: Vec::new(),
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
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };
        assert_eq!(project_from_labels(&issue).as_deref(), Some("mamba"));
        assert_eq!(issue_ref(&issue), "#4100");
    }

    #[test]
    fn format_migration_dispatches_capability_without_hitl() {
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let action = CapabilityAction {
            kind: CapabilityActionKind::FormatMigrationRequired,
            capability_id: None,
            gap_id: None,
            claim_id: None,
            target: "README.md".to_string(),
            command: "aw capability run --project jet --non-interactive --max-ticks 1".to_string(),
            reason: "README capability map needs Markdown-table migration".to_string(),
            requires_hitl: false,
            hitl_question: None,
        };

        let envelope = capability_action_envelope(
            root.clone(),
            root,
            "jet",
            &action,
            project_completion(false, vec![action.reason.clone()]),
        );

        assert_eq!(envelope.action, "dispatch");
        assert_eq!(envelope.next.kind, "capability");
        assert_eq!(
            envelope.next.command,
            "aw capability run --project jet --non-interactive"
        );
        assert!(!envelope.requires_hitl);
        assert!(!envelope.completion.workflow_complete);
    }

    #[test]
    fn project_ready_lane_dispatches_next_wi_run() {
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let issue = open_issue(IssueType::Enhancement, 4301);

        let envelope = project_ready_wi_envelope("jet", root, &issue);

        assert_eq!(envelope.action, "dispatch");
        assert_eq!(envelope.next.kind, "execute_change");
        assert_eq!(envelope.next.command, "aw wi run 4301");
        assert!(!envelope.requires_hitl);
        assert!(!envelope.completion.workflow_complete);
        assert_no_removed_wi_verbs(&envelope);
    }

    #[test]
    fn project_atomize_lane_dispatches_atomize_without_sprint_batching() {
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let lanes = wi_cli::PrioritizeLanes {
            needs_atomize: vec![open_issue(IssueType::Epic, 4302)],
            ..Default::default()
        };

        let envelope = project_atomize_backlog_envelope("jet", root, &lanes);

        assert_eq!(envelope.action, "dispatch");
        assert_eq!(envelope.next.kind, "atomize");
        assert_eq!(envelope.next.command, "aw wi atomize --project jet");
        assert!(!envelope.requires_hitl);
        assert!(!envelope.completion.workflow_complete);
        assert_no_removed_wi_verbs(&envelope);
    }

    #[test]
    fn project_blocked_lanes_invoke_prioritize_readiness() {
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let lanes = wi_cli::PrioritizeLanes {
            blocked_by_dependency: vec![open_issue(IssueType::Bug, 4303)],
            needs_triage: vec![open_issue(IssueType::Enhancement, 4304)],
            ..Default::default()
        };

        let envelope = project_prioritize_blocked_envelope("jet", root, &lanes);

        assert_eq!(envelope.action, "blocked");
        assert_eq!(envelope.next.kind, "prioritize");
        assert_eq!(envelope.next.command, "aw wi prioritize --project jet");
        assert!(!envelope.next.command.contains("--minutes"));
        assert!(envelope.requires_hitl);
        assert!(!envelope.completion.workflow_complete);
        assert_no_removed_wi_verbs(&envelope);
    }

    #[test]
    fn project_completion_blocks_on_repo_side_dirty_paths() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::fs::create_dir_all(root_dir.join(".aw")).unwrap();
        std::fs::create_dir_all(root_dir.join("apps/jet/src")).unwrap();
        std::fs::create_dir_all(root_dir.join("apps/jet/tech-design")).unwrap();
        std::fs::write(
            root_dir.join("aw.toml"),
            r#"
[[projects]]
name = "jet"
path = "apps/jet"
td_path = "apps/jet/tech-design"
cap_path = "apps/jet/README.md"

[[projects.workspaces]]
name = "jet"
paths = ["apps/jet/**"]
target = "rust"
test_cmd = "true"
"#,
        )
        .unwrap();
        std::fs::write(root_dir.join("apps/jet/README.md"), "# jet\n").unwrap();
        std::fs::write(root_dir.join("apps/jet/src/lib.rs"), "pub fn demo() {}\n").unwrap();
        init_git_repo(root_dir);

        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let clean =
            project_done_or_dirty_envelope_with_health("jet", root.clone(), root_dir, || {
                Ok(project_health_fixture(
                    "jet",
                    false,
                    vec!["test gates failed: 1/1 command(s)"],
                ))
            });
        assert_eq!(clean.action, "blocked");
        assert_eq!(clean.next.kind, "production_verification");
        assert!(!clean.completion.workflow_complete);
        let clean_persistence = clean.persistence.as_ref().unwrap();
        assert_eq!(clean_persistence.status, "production_blocked");
        assert!(clean_persistence.commit_complete);
        assert!(clean_persistence.wi_evidence_complete);

        std::fs::write(
            root_dir.join("apps/jet/src/lib.rs"),
            "pub fn demo() { println!(\"dirty\"); }\n",
        )
        .unwrap();

        let mut dirty = project_done_or_dirty_envelope_with_health(
            "jet",
            root,
            root_dir,
            || -> Result<crate::cli::project::ProjectHealthReport> {
                panic!("health report should not be evaluated while repo-side scopes are dirty")
            },
        );
        assert_eq!(dirty.action, "blocked");
        assert_eq!(dirty.next.kind, "milestone_persistence");
        assert!(dirty.requires_hitl);
        assert!(!dirty.completion.workflow_complete);
        assert!(dirty.next.reason.contains("apps/jet/src/lib.rs"));
        let dirty_persistence = dirty.persistence.as_ref().unwrap();
        assert_eq!(dirty_persistence.status, "repo_dirty");
        assert!(!dirty_persistence.commit_complete);
        assert!(!dirty_persistence.wi_evidence_complete);
        assert_eq!(
            dirty_persistence.dirty_paths,
            vec!["apps/jet/src/lib.rs".to_string()]
        );
        ensure_hitl_question(&mut dirty, "aw run --project jet");
        let dirty_json = serde_json::to_value(&dirty).unwrap();
        assert_eq!(dirty_json["completion"]["requires_hitl"], true);
        assert_eq!(
            dirty_json["persistence"]["wi_evidence_complete"],
            serde_json::Value::Bool(false)
        );
        assert!(dirty_json.get("hitl_question").is_some());

        let after_approval = project_done_or_dirty_envelope_with_health(
            "jet",
            WorkflowNode {
                kind: "project".to_string(),
                id: "jet".to_string(),
            },
            root_dir,
            || {
                Ok(project_health_fixture(
                    "jet",
                    false,
                    vec!["test gates failed: 1/1 command(s)"],
                ))
            },
        );
        assert_eq!(after_approval.action, "blocked");
        assert_eq!(after_approval.next.kind, "production_verification");
        let persisted = after_approval.persistence.as_ref().unwrap();
        assert_eq!(persisted.status, "production_blocked");
        assert!(persisted.commit_complete);
        assert!(persisted.wi_evidence_complete);
        let log = git_stdout(root_dir, &["log", "-1", "--pretty=%B"]);
        assert!(log.contains("Lifecycle-Stage: Project-Persistence"));
        assert!(log.contains("Dirty-Paths: apps/jet/src/lib.rs"));
    }

    #[test]
    fn project_completion_finishes_on_clean_ready_health() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::fs::create_dir_all(root_dir.join(".aw")).unwrap();
        std::fs::create_dir_all(root_dir.join("apps/jet/src")).unwrap();
        std::fs::create_dir_all(root_dir.join("apps/jet/tech-design")).unwrap();
        std::fs::write(
            root_dir.join("aw.toml"),
            r#"
[[projects]]
name = "jet"
path = "apps/jet"
td_path = "apps/jet/tech-design"
cap_path = "apps/jet/README.md"

[[projects.workspaces]]
name = "jet"
paths = ["apps/jet/**"]
target = "rust"
test_cmd = "true"
"#,
        )
        .unwrap();
        std::fs::write(root_dir.join("apps/jet/README.md"), "# jet\n").unwrap();
        std::fs::write(root_dir.join("apps/jet/src/lib.rs"), "pub fn demo() {}\n").unwrap();
        init_git_repo(root_dir);

        let envelope = project_done_or_dirty_envelope_with_health(
            "jet",
            WorkflowNode {
                kind: "project".to_string(),
                id: "jet".to_string(),
            },
            root_dir,
            || Ok(project_health_fixture("jet", true, vec![])),
        );

        assert_eq!(envelope.action, "done");
        assert_eq!(envelope.next.kind, "inspect_parent");
        assert!(envelope.completion.workflow_complete);
        assert!(envelope.completion.missing.is_empty());
    }

    #[test]
    fn project_completion_blocks_on_failing_test_gates() {
        if !git_available() {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::fs::create_dir_all(root_dir.join(".aw")).unwrap();
        std::fs::create_dir_all(root_dir.join("apps/jet/src")).unwrap();
        std::fs::create_dir_all(root_dir.join("apps/jet/tech-design")).unwrap();
        std::fs::write(
            root_dir.join("aw.toml"),
            r#"
[[projects]]
name = "jet"
path = "apps/jet"
td_path = "apps/jet/tech-design"
cap_path = "apps/jet/README.md"

[[projects.workspaces]]
name = "jet"
paths = ["apps/jet/**"]
target = "rust"
test_cmd = "false"
"#,
        )
        .unwrap();
        std::fs::write(root_dir.join("apps/jet/README.md"), "# jet\n").unwrap();
        std::fs::write(root_dir.join("apps/jet/src/lib.rs"), "pub fn demo() {}\n").unwrap();
        init_git_repo(root_dir);

        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let envelope = project_done_or_dirty_envelope_with_health("jet", root, root_dir, || {
            Ok(project_health_fixture(
                "jet",
                false,
                vec!["test gates failed: 1/1 command(s)"],
            ))
        });
        assert_eq!(envelope.action, "blocked");
        assert_eq!(envelope.next.kind, "production_verification");
        assert_eq!(envelope.next.command, "aw health --project jet");
        assert!(envelope.requires_hitl);
        assert!(!envelope.completion.workflow_complete);
        assert!(envelope
            .completion
            .missing
            .iter()
            .any(|missing| missing.contains("test gates failed")));
        let persistence = envelope.persistence.as_ref().unwrap();
        assert_eq!(persistence.status, "production_blocked");
        assert!(persistence.commit_complete);
        assert!(persistence.wi_evidence_complete);
    }

    #[test]
    fn latest_pending_planning_artifact_finds_newest_pending_review() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("aw").join("jet").join("epics");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("20260101000000-jet-plan.md"),
            "---\nkind: epicize\nagent_review_required: true\nreview_status: pending\n---\n# old\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("20260102000000-jet-plan.md"),
            "---\nkind: epicize\nagent_review_required: true\nreview_status: pending\n---\n# new\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("20260103000000-jet-plan.md"),
            "---\nkind: epicize\nagent_review_required: true\nreview_status: approved\n---\n# approved\n",
        )
        .unwrap();

        let latest =
            latest_pending_planning_artifact(tmp.path(), "jet", "epics", "epicize").unwrap();

        assert_eq!(
            latest.file_name().and_then(|name| name.to_str()),
            Some("20260102000000-jet-plan.md")
        );
    }

    #[test]
    fn latest_pending_planning_artifact_ignores_noop_epicize_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("aw").join("jet").join("epics");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("20260102000000-jet-plan.md"),
            r#"---
kind: epicize
issue_count: 0
agent_review_required: true
review_status: pending
---

# jet next phase

## Capability Epic Candidates

| Work Root | Kind | Source Capability | WI | Status | Promise / Scope |
|---|---|---|---:|---|---|
| Package Manager | epic | package-manager | - | verified | Resolve packages |
| Lockfile parity | subepic | package-manager | - | closed | lockfile-parity |
| Optional refactor | subepic | package-manager | - | deferred | optional-refactor |

## Existing Epics

- none

## Requirement Groups

### Urgent fixes

- none

## Epic Candidates

- none

## Required Agent Review Brief

This epic draft requires agent review before publishing tracker changes.
"#,
        )
        .unwrap();

        assert!(latest_pending_planning_artifact(tmp.path(), "jet", "epics", "epicize").is_none());
    }

    #[test]
    fn latest_pending_planning_artifact_keeps_actionable_capability_epicize_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("aw").join("jet").join("epics");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("20260102000000-jet-plan.md");
        std::fs::write(
            &path,
            r#"---
kind: epicize
issue_count: 0
agent_review_required: true
review_status: pending
---

# jet next phase

## Capability Epic Candidates

| Work Root | Kind | Source Capability | WI | Status | Promise / Scope |
|---|---|---|---:|---|---|
| Package Manager | epic | package-manager | - | candidate | Resolve packages |

## Epic Candidates

- none
"#,
        )
        .unwrap();

        let latest =
            latest_pending_planning_artifact(tmp.path(), "jet", "epics", "epicize").unwrap();
        assert_eq!(latest, path);
    }

    #[test]
    fn create_wi_blocks_on_pending_epicize_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("aw").join("jet").join("epics");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("20260102000000-jet-plan.md");
        std::fs::write(
            &path,
            "---\nkind: epicize\nagent_review_required: true\nreview_status: pending\n---\n# plan\n",
        )
        .unwrap();
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let action = CapabilityAction {
            kind: CapabilityActionKind::CreateWi,
            capability_id: Some("package-manager".to_string()),
            gap_id: Some("lockfile-parity".to_string()),
            claim_id: None,
            target: "Package Manager".to_string(),
            command: "aw wi plan --project jet".to_string(),
            reason: "open capability gap has no active WI".to_string(),
            requires_hitl: false,
            hitl_question: None,
        };

        let envelope = capability_action_envelope_with_planning_base(
            root.clone(),
            root,
            "jet",
            &action,
            project_completion(false, vec![action.reason.clone()]),
            tmp.path(),
        );

        assert_eq!(envelope.action, "blocked");
        assert_eq!(envelope.next.kind, "review_planning_artifact");
        assert_eq!(
            envelope.next.payload_path.as_deref(),
            Some(path.to_str().unwrap())
        );
        assert!(envelope.requires_hitl);
        assert_eq!(
            envelope
                .hitl_question
                .as_ref()
                .map(|question| question.tool_hint.as_str()),
            Some("ask_user_question")
        );
    }

    #[test]
    fn create_wi_ignores_noop_pending_epicize_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("aw").join("jet").join("epics");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("20260102000000-jet-plan.md"),
            r#"---
kind: epicize
issue_count: 0
agent_review_required: true
review_status: pending
---

# jet next phase

## Capability Epic Candidates

| Work Root | Kind | Source Capability | WI | Status | Promise / Scope |
|---|---|---|---:|---|---|
| Package Manager | epic | package-manager | - | verified | Resolve packages |

## Epic Candidates

- none
"#,
        )
        .unwrap();
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let action = CapabilityAction {
            kind: CapabilityActionKind::CreateWi,
            capability_id: Some("package-manager".to_string()),
            gap_id: Some("lockfile-parity".to_string()),
            claim_id: None,
            target: "Package Manager".to_string(),
            command: "aw wi plan --project jet".to_string(),
            reason: "open capability gap has no active WI".to_string(),
            requires_hitl: false,
            hitl_question: None,
        };

        let envelope = capability_action_envelope_with_planning_base(
            root.clone(),
            root,
            "jet",
            &action,
            project_completion(false, vec![action.reason.clone()]),
            tmp.path(),
        );

        assert_eq!(envelope.action, "dispatch");
        assert_eq!(envelope.next.kind, "epicize");
        assert!(!envelope.requires_hitl);
    }

    #[test]
    fn reconcile_wi_refs_dispatches_capability_plan_not_epicize() {
        let tmp = tempfile::tempdir().unwrap();
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let action = CapabilityAction {
            kind: CapabilityActionKind::ReconcileWiRefs,
            capability_id: Some("package-manager".to_string()),
            gap_id: Some("lockfile-parity".to_string()),
            claim_id: None,
            target: "Package Manager".to_string(),
            command: "aw wi plan --project jet".to_string(),
            reason: "active WI reference is not present in project issue inventory".to_string(),
            requires_hitl: false,
            hitl_question: None,
        };

        let envelope = capability_action_envelope_with_planning_base(
            root.clone(),
            root,
            "jet",
            &action,
            project_completion(false, vec![action.reason.clone()]),
            tmp.path(),
        );

        assert_eq!(envelope.action, "dispatch");
        assert_eq!(envelope.next.kind, "reconcile_wi_refs");
        assert_eq!(envelope.next.command, "aw wi plan --project jet");
        assert!(!envelope.requires_hitl);
    }

    #[test]
    fn hitl_capability_action_propagates_question() {
        let root = WorkflowNode {
            kind: "project".to_string(),
            id: "jet".to_string(),
        };
        let question = HitlQuestion {
            id: "capability:package-manager:confirm_candidate".to_string(),
            question: "Should this capability be confirmed?".to_string(),
            target: "Package Manager".to_string(),
            resume_command: "aw capability run --project jet --non-interactive --max-ticks 1"
                .to_string(),
            tool_hint: "ask_user_question".to_string(),
            choices: Vec::new(),
            default_choice: None,
            freeform_prompt: None,
        };
        let action = CapabilityAction {
            kind: CapabilityActionKind::HumanConfirmRequired,
            capability_id: Some("package-manager".to_string()),
            gap_id: None,
            claim_id: None,
            target: "Package Manager".to_string(),
            command: "aw capability report --project jet".to_string(),
            reason: "capability is still candidate and requires human confirmation".to_string(),
            requires_hitl: true,
            hitl_question: Some(question.clone()),
        };

        let envelope = capability_action_envelope(
            root.clone(),
            root,
            "jet",
            &action,
            project_completion(false, vec![action.reason.clone()]),
        );

        assert_eq!(envelope.action, "blocked");
        assert!(envelope.requires_hitl);
        assert_eq!(envelope.next.command, "aw capability report --project jet");
        let mut expected_question = question;
        expected_question.resume_command =
            "aw capability run --project jet --non-interactive".to_string();
        assert_eq!(envelope.hitl_question, Some(expected_question));
    }

    #[test]
    fn closed_change_outputs_parent_inspection() {
        let issue = Issue {
            issue_type: IssueType::Enhancement,
            title: "Bounded change".to_string(),
            state: crate::issues::IssueState::Closed,
            id: None,
            github_id: Some(4102),
            gitlab_id: None,
            url: None,
            author: None,
            created_at: None,
            updated_at: None,
            slug: "4102".to_string(),
            body: String::new(),
            labels: vec!["app:mamba".to_string()],
            related: vec!["parent #4101".to_string()],
            implements: Vec::new(),
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
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };
        let envelope = closed_wi_envelope(&issue);
        assert_eq!(envelope.action, "done");
        assert!(envelope.completion.root_complete);
        assert!(!envelope.completion.workflow_complete);
        assert_eq!(envelope.next.kind, "inspect_parent");
        assert_eq!(envelope.next.command, "aw wi run 4101");
    }

    #[test]
    fn closed_change_without_parent_outputs_executable_show_command() {
        let issue = Issue {
            issue_type: IssueType::Enhancement,
            title: "Bounded change".to_string(),
            state: crate::issues::IssueState::Closed,
            id: None,
            github_id: Some(4041),
            gitlab_id: None,
            url: None,
            author: None,
            created_at: None,
            updated_at: None,
            slug: "4041".to_string(),
            body: String::new(),
            labels: vec!["app:jet".to_string()],
            related: Vec::new(),
            implements: Vec::new(),
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
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };
        let envelope = closed_wi_envelope(&issue);
        assert_eq!(envelope.next.command, "aw wi show 4041");
    }

    #[test]
    fn artifact_quality_gate_injects_frontend_profile_and_prompt() {
        let mut envelope = test_envelope(
            "aw td gen --project jet frontend/src/App.tsx",
            "generate frontend page component under frontend/src/App.tsx",
        );

        apply_artifact_quality_gate(&mut envelope);

        let profile = envelope.artifact_quality_profile.as_ref().unwrap();
        assert_eq!(profile.artifact_kind, ArtifactKind::FrontendPage);
        assert!(envelope.agent_prompt.contains("Artifact Quality Gate"));
        assert!(envelope
            .agent_prompt
            .contains("frontend-page-viewport-screenshots"));
        assert!(envelope
            .agent_prompt
            .contains("desktop and mobile viewport evidence"));
        assert!(envelope
            .completion
            .criteria
            .contains(&"artifact quality hard preflight gates are satisfied".to_string()));
    }

    #[test]
    fn artifact_quality_gate_defaults_non_ui_change_to_code_artifact() {
        let mut envelope = test_envelope("aw td create 3903", "atomic change enters TD lifecycle");

        apply_artifact_quality_gate(&mut envelope);

        assert_eq!(
            envelope
                .artifact_quality_profile
                .as_ref()
                .unwrap()
                .artifact_kind,
            ArtifactKind::CodeArtifact
        );
        assert!(envelope.agent_prompt.contains("code-artifact-test"));
        assert!(!envelope
            .agent_prompt
            .contains("frontend-page-viewport-screenshots"));
    }

    #[test]
    fn artifact_quality_gate_skips_done_envelope() {
        let mut envelope = test_envelope("", "complete");
        envelope.action = "done".to_string();

        apply_artifact_quality_gate(&mut envelope);

        assert!(envelope.artifact_quality_profile.is_none());
        assert!(!envelope.agent_prompt.contains("Artifact Quality Gate"));
    }

    #[test]
    fn workflow_envelope_serializes_optional_artifact_quality_profile() {
        let envelope = WorkflowEnvelope {
            action: "dispatch".to_string(),
            root: WorkflowNode {
                kind: "wi".to_string(),
                id: "3903".to_string(),
            },
            current: WorkflowNode {
                kind: "td".to_string(),
                id: "3903".to_string(),
            },
            completed: None,
            completion: WorkflowCompletion {
                root_complete: false,
                workflow_complete: false,
                criteria: Vec::new(),
                missing: Vec::new(),
            },
            next: WorkflowNext {
                kind: "td".to_string(),
                command: "aw td create 3903".to_string(),
                reason: "test".to_string(),
                payload_path: None,
            },
            invoke: WorkflowInvoke {
                command: "aw td create 3903".to_string(),
            },
            agent_prompt: "test".to_string(),
            requires_hitl: false,
            artifact_quality_profile: Some(ArtifactQualityProfile::default_for_kind(
                crate::models::ArtifactKind::CliSurface,
            )),
            hitl_question: None,
            persistence: None,
        };
        let json = serde_json::to_value(envelope).unwrap();
        assert_eq!(json["schema_version"], "aw.cli.v1");
        assert_eq!(json["status"], "continue");
        assert!(json.get("invoke").is_none());
        assert_eq!(json["completion"]["requires_hitl"], false);
        assert_eq!(json["next"]["kind"], "run_command");
        assert_eq!(json["next"]["command"], "aw td create 3903");
        assert!(json.get("artifact_quality_profile").is_some());
        assert_eq!(
            json["artifact_quality_profile"]["artifact_kind"],
            "cli_surface"
        );
    }

    fn test_envelope(command: &str, reason: &str) -> WorkflowEnvelope {
        WorkflowEnvelope {
            action: "dispatch".to_string(),
            root: WorkflowNode {
                kind: "project".to_string(),
                id: "demo".to_string(),
            },
            current: WorkflowNode {
                kind: "change".to_string(),
                id: "#3903".to_string(),
            },
            completed: None,
            completion: WorkflowCompletion {
                root_complete: false,
                workflow_complete: false,
                criteria: Vec::new(),
                missing: Vec::new(),
            },
            next: WorkflowNext {
                kind: "execute_change".to_string(),
                command: command.to_string(),
                reason: reason.to_string(),
                payload_path: None,
            },
            invoke: WorkflowInvoke {
                command: command.to_string(),
            },
            agent_prompt: "Run next.command.".to_string(),
            requires_hitl: false,
            artifact_quality_profile: None,
            hitl_question: None,
            persistence: None,
        }
    }
}
// CODEGEN-END

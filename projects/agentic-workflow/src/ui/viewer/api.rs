//! SddDataSource REST API handlers.
//!
//! These handlers implement the backend for the `LocalDataSource` TypeScript
//! class defined in `@score/core`. Each handler reads from the local `.aw/`
//! filesystem and returns JSON matching the shared TS types.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::issues::backend::IssueBackend;
use crate::issues::backends::local::LocalBackend;
use crate::issues::types::IssueFilter;
use crate::models::state::State as ChangeState;
use crate::shared::workspace;

use super::AppState;

// ============================================================================
// Response types — match @score/core/src/types.ts
// ============================================================================

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
// CODEGEN-BEGIN
use serde::Serialize;

/// Full change detail.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct ChangeResponse {
    /// Change identifier.
    pub id: String,
    /// Optional description.
    pub description: Option<String>,
    /// Lifecycle phase.
    pub phase: String,
    /// Linked issue IDs.
    pub issue_ids: Vec<String>,
    /// Linked spec IDs.
    pub spec_ids: Vec<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Change summary card.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct ChangeSummaryResponse {
    /// Change identifier.
    pub id: String,
    /// Optional description.
    pub description: Option<String>,
    /// Lifecycle phase.
    pub phase: String,
    /// Linked issue IDs.
    pub issue_ids: Vec<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Full issue detail.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct IssueResponse {
    /// Issue identifier.
    pub id: String,
    /// Numeric issue number.
    pub issue_number: u64,
    /// Issue title.
    pub title: String,
    /// Optional issue body.
    pub description: Option<String>,
    /// Status string.
    pub status: String,
    /// Optional priority.
    pub priority: Option<String>,
    /// Labels.
    pub labels: Vec<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
    /// Optional closed timestamp.
    pub closed_at: Option<String>,
}

/// Issue summary card.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct IssueSummaryResponse {
    /// Issue identifier.
    pub id: String,
    /// Numeric issue number.
    pub issue_number: u64,
    /// Issue title.
    pub title: String,
    /// Status string.
    pub status: String,
    /// Optional priority.
    pub priority: Option<String>,
    /// Labels.
    pub labels: Vec<String>,
    /// Creation timestamp.
    pub created_at: String,
}

/// Opaque lineage graph.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct LineageGraphResponse {
    /// Graph nodes.
    pub nodes: Vec<serde_json::Value>,
    /// Graph edges.
    pub edges: Vec<serde_json::Value>,
}

/// Project metadata.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct ProjectInfoResponse {
    /// Project name.
    pub name: String,
    /// Project root path (string).
    pub root: String,
    /// Whether `.aw/` exists at root.
    pub has_score: bool,
}

/// Full TD detail.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct TechDesignResponse {
    /// TD identifier.
    pub id: String,
    /// Crate name.
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// Spec path.
    pub path: String,
    /// TD title.
    pub title: String,
    /// Spec markdown body.
    pub content: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// TD summary card.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/ui/viewer/api.md#schema
#[derive(Serialize)]
pub struct TechDesignSummaryResponse {
    /// TD identifier.
    pub id: String,
    /// Crate name (serialised as `crate`).
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// Spec path.
    pub path: String,
    /// TD title.
    pub title: String,
    /// Last update timestamp.
    pub updated_at: String,
}
// CODEGEN-END
// ============================================================================
// Helper — project root from AppState
// ============================================================================

fn project_root(state: &AppState) -> &std::path::Path {
    &state.project_root
}

/// Map issue state to TS IssueStatus string.
fn issue_status_str(state: crate::issues::types::IssueState) -> &'static str {
    match state {
        crate::issues::types::IssueState::Open | crate::issues::types::IssueState::Draft => "open",
        crate::issues::types::IssueState::Closed => "closed",
    }
}

/// Extract priority from labels (e.g. "priority:p0" -> "critical").
fn priority_from_labels(labels: &[String]) -> Option<String> {
    for label in labels {
        if let Some(level) = label.strip_prefix("priority:") {
            let mapped = match level {
                "p0" => "critical",
                "p1" => "high",
                "p2" => "medium",
                "p3" => "low",
                _ => level,
            };
            return Some(mapped.to_string());
        }
    }
    None
}

/// Derive a human-readable title from a spec filename.
fn title_from_filename(filename: &str) -> String {
    filename
        .trim_end_matches(".md")
        .replace('-', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => {
                    let upper: String = f.to_uppercase().collect();
                    upper + c.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Derive the crate name from a tech design path relative to tech_design dir.
/// Example: "crates/cclab-agent/architecture.md" -> "cclab-agent"
fn crate_from_relative_path(rel: &str) -> String {
    let parts: Vec<&str> = rel.split('/').collect();
    if parts.len() >= 2 && parts[0] == "crates" {
        parts[1].to_string()
    } else if parts.len() >= 2 && parts[0] == "projects" {
        parts[1].to_string()
    } else {
        // Top-level specs have no crate
        String::new()
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/issues — list all issues from the temp-backed issue store.
pub async fn api_list_issues(State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let state = state.lock().await;
    let root = project_root(&state);
    let backend = LocalBackend::from_project_root(root);

    match backend.list(&IssueFilter::default()).await {
        Ok(issues) => {
            let summaries: Vec<IssueSummaryResponse> = issues
                .iter()
                .enumerate()
                .map(|(idx, i)| IssueSummaryResponse {
                    id: i.slug.clone(),
                    issue_number: i.github_id.or(i.gitlab_id).unwrap_or(idx as u64),
                    title: i.title.clone(),
                    status: issue_status_str(i.state).to_string(),
                    priority: priority_from_labels(&i.labels),
                    labels: i.labels.clone(),
                    created_at: i.created_at.clone().unwrap_or_default(),
                })
                .collect();
            Json(summaries).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/issues/:id — get a single issue by slug
pub async fn api_get_issue(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> Response {
    let state = state.lock().await;
    let root = project_root(&state);
    let backend = LocalBackend::from_project_root(root);

    match backend.get(&id).await {
        Ok(Some(i)) => {
            let resp = IssueResponse {
                id: i.slug.clone(),
                issue_number: i.github_id.or(i.gitlab_id).unwrap_or(0),
                title: i.title.clone(),
                description: if i.body.is_empty() {
                    None
                } else {
                    Some(i.body.clone())
                },
                status: issue_status_str(i.state).to_string(),
                priority: priority_from_labels(&i.labels),
                labels: i.labels.clone(),
                created_at: i.created_at.clone().unwrap_or_default(),
                updated_at: i.updated_at.clone().unwrap_or_default(),
                closed_at: if i.state == crate::issues::types::IssueState::Closed {
                    i.updated_at.clone()
                } else {
                    None
                },
            };
            Json(resp).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("issue '{}' not found", id) })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/tech-designs — list all .aw/tech-design/**/*.md files
pub async fn api_list_tech_designs(State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let state = state.lock().await;
    let root = project_root(&state);

    match collect_tech_design_files(root) {
        Ok(files) => {
            let summaries: Vec<TechDesignSummaryResponse> = files
                .into_iter()
                .map(|(rel_path, modified)| {
                    let title =
                        title_from_filename(rel_path.rsplit('/').next().unwrap_or(&rel_path));
                    let crate_name = crate_from_relative_path(&rel_path);
                    let id = rel_path.trim_end_matches(".md").replace('/', "::");
                    TechDesignSummaryResponse {
                        id,
                        crate_name,
                        path: rel_path,
                        title,
                        updated_at: modified,
                    }
                })
                .collect();
            Json(summaries).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/tech-designs/:id — read a single spec file
pub async fn api_get_tech_design(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> Response {
    let state = state.lock().await;
    let root = project_root(&state);

    // id is encoded as "crates::cclab-agent::architecture" -> "crates/cclab-agent/architecture.md"
    let rel_path = id.replace("::", "/") + ".md";
    let Some(file_path) = resolve_tech_design_file(root, &rel_path) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("tech design '{}' not found", id) })),
        )
            .into_response();
    };

    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("tech design '{}' not found", id) })),
        )
            .into_response();
    }

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            let title = title_from_filename(rel_path.rsplit('/').next().unwrap_or(&rel_path));
            let crate_name = crate_from_relative_path(&rel_path);
            let modified = file_modified_iso(&file_path);
            let resp = TechDesignResponse {
                id,
                crate_name,
                path: rel_path,
                title,
                content,
                updated_at: modified,
            };
            Json(resp).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/changes — list all .aw/changes/*/STATE.yaml
pub async fn api_list_changes(State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let state = state.lock().await;
    let root = project_root(&state);
    let changes_dir = workspace::changes_path(root);

    if !changes_dir.exists() {
        return Json(Vec::<ChangeSummaryResponse>::new()).into_response();
    }

    let mut summaries = Vec::new();
    let entries = match std::fs::read_dir(&changes_dir) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let state_file = path.join("STATE.yaml");
        if !state_file.exists() {
            continue;
        }
        if let Ok(cs) = load_change_state(&state_file) {
            let issue_ids = cs
                .dag
                .as_ref()
                .map(|d| d.issues.iter().map(|i| i.number.to_string()).collect())
                .unwrap_or_default();
            summaries.push(ChangeSummaryResponse {
                id: cs.change_id.clone(),
                description: cs.last_action.clone(),
                phase: serde_json::to_value(&cs.phase)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| map_phase_to_frontend(s).to_string()))
                    .unwrap_or_else(|| "init".to_string()),
                issue_ids,
                created_at: cs.created_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                updated_at: cs.updated_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            });
        }
    }

    // Sort by change_id for deterministic output
    summaries.sort_by(|a, b| a.id.cmp(&b.id));
    Json(summaries).into_response()
}

/// GET /api/changes/:id — read a single change (STATE + artifacts)
pub async fn api_get_change(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> Response {
    let state = state.lock().await;
    let root = project_root(&state);
    let change_dir = workspace::change_path(root, &id);
    let state_file = change_dir.join("STATE.yaml");

    if !state_file.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("change '{}' not found", id) })),
        )
            .into_response();
    }

    match load_change_state(&state_file) {
        Ok(cs) => {
            let issue_ids: Vec<String> = cs
                .dag
                .as_ref()
                .map(|d| d.issues.iter().map(|i| i.number.to_string()).collect())
                .unwrap_or_default();

            // Collect spec IDs from the groups directory
            let spec_ids = collect_spec_ids(&change_dir);

            let resp = ChangeResponse {
                id: cs.change_id.clone(),
                description: cs.last_action.clone(),
                phase: serde_json::to_value(&cs.phase)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| map_phase_to_frontend(s).to_string()))
                    .unwrap_or_else(|| "init".to_string()),
                issue_ids,
                spec_ids,
                created_at: cs.created_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                updated_at: cs.updated_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            };
            Json(resp).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/lineage/:id — placeholder returning empty graph
pub async fn api_get_lineage(Path(_id): Path<String>) -> Json<LineageGraphResponse> {
    Json(LineageGraphResponse {
        nodes: vec![],
        edges: vec![],
    })
}

/// GET /api/project-info — project metadata
pub async fn api_project_info(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<ProjectInfoResponse> {
    let state = state.lock().await;
    let root = project_root(&state);
    let has_score = workspace::workspace_path(root).exists();
    let name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Json(ProjectInfoResponse {
        name,
        root: root.display().to_string(),
        has_score,
    })
}

// ============================================================================
// Filesystem helpers
// ============================================================================

fn collect_tech_design_files(root: &std::path::Path) -> anyhow::Result<Vec<(String, String)>> {
    let td_dir = workspace::tech_design_path(root);
    let mut results = Vec::new();

    if td_dir.exists() {
        results.extend(collect_md_files(&td_dir)?);
    }

    for (project, project_td_dir) in workspace::project_tech_design_paths(root) {
        if project_td_dir.starts_with(&td_dir) || !project_td_dir.exists() {
            continue;
        }
        for (rel_path, modified) in collect_md_files(&project_td_dir)? {
            results.push((format!("projects/{project}/{rel_path}"), modified));
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    results.dedup_by(|a, b| a.0 == b.0);
    Ok(results)
}

fn resolve_tech_design_file(root: &std::path::Path, rel_path: &str) -> Option<std::path::PathBuf> {
    let td_dir = workspace::tech_design_path(root);
    let base_candidate = td_dir.join(rel_path);
    if base_candidate.exists() && base_candidate.starts_with(&td_dir) {
        return Some(base_candidate);
    }

    let rest = rel_path.strip_prefix("projects/")?;
    let mut parts = rest.splitn(2, '/');
    let project = parts.next()?;
    let project_rel = parts.next()?;

    workspace::project_tech_design_paths(root)
        .into_iter()
        .find_map(|(name, td_root)| {
            if name == project {
                Some(td_root.join(project_rel))
            } else {
                None
            }
        })
}

/// Recursively collect all `.md` files under a directory, returning
/// `(relative_path, modified_iso)` pairs. Skips hidden files and README.md.
fn collect_md_files(base: &std::path::Path) -> anyhow::Result<Vec<(String, String)>> {
    let mut results = Vec::new();
    collect_md_files_recursive(base, base, &mut results)?;
    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

fn collect_md_files_recursive(
    base: &std::path::Path,
    dir: &std::path::Path,
    results: &mut Vec<(String, String)>,
) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files_recursive(base, &path, results)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            // Skip hidden files, README, AUTHORING, CHANGELOG
            if name.starts_with('.')
                || name == "README.md"
                || name == "AUTHORING.md"
                || name == "CHANGELOG.md"
            {
                continue;
            }
            if let Ok(rel) = path.strip_prefix(base) {
                let rel_str = rel.to_string_lossy().to_string();
                let modified = file_modified_iso(&path);
                results.push((rel_str, modified));
            }
        }
    }
    Ok(())
}

/// Get ISO 8601 modification time for a file, or empty string on error.
fn file_modified_iso(path: &std::path::Path) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Utc> = t.into();
            dt.to_rfc3339()
        })
        .unwrap_or_default()
}

/// Load and parse a STATE.yaml file.
fn load_change_state(path: &std::path::Path) -> anyhow::Result<ChangeState> {
    let content = std::fs::read_to_string(path)?;
    let state: ChangeState = serde_yaml::from_str(&content)?;
    Ok(state)
}

/// Collect spec file IDs from a change's groups/ directory.
fn collect_spec_ids(change_dir: &std::path::Path) -> Vec<String> {
    let groups_dir = change_dir.join("groups");
    if !groups_dir.exists() {
        return vec![];
    }
    let mut ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&groups_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Each subdirectory under groups/ may contain spec files
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if sub_path.extension().and_then(|e| e.to_str()) == Some("md") {
                            if let Some(stem) = sub_path.file_stem().and_then(|s| s.to_str()) {
                                ids.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    ids.sort();
    ids
}

/// Map internal StatePhase serialized string to the frontend's ChangePhase type.
/// The internal phases are more granular (22 variants); the frontend uses 9 buckets.
fn map_phase_to_frontend(internal: &str) -> &str {
    match internal {
        "change_inited" => "init",
        "input_restructured" => "restructure",
        "pre_clarifications_created" => "pre_clarify",
        "reference_context_created"
        | "reference_context_reviewed"
        | "reference_context_revised" => "reference_context",
        "post_clarifications_created" => "post_clarify",
        "change_spec_created" | "change_spec_reviewed" | "change_spec_revised" => "change_spec",
        "change_implementation_created"
        | "change_implementation_reviewed"
        | "change_implementation_revised" => "implementation",
        "docs_check" | "docs_created" | "docs_reviewed" | "docs_revised" => "review",
        "change_merge_created"
        | "change_merge_reviewed"
        | "change_merge_revised"
        | "change_archived" => "merge",
        "change_rejected" => "merge",
        _ => "init",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_from_filename() {
        assert_eq!(title_from_filename("architecture.md"), "Architecture");
        assert_eq!(
            title_from_filename("agent-protocols-spec.md"),
            "Agent Protocols Spec"
        );
        assert_eq!(title_from_filename("README.md"), "README");
    }

    #[test]
    fn test_crate_from_relative_path() {
        assert_eq!(
            crate_from_relative_path("crates/cclab-agent/architecture.md"),
            "cclab-agent"
        );
        assert_eq!(
            crate_from_relative_path("projects/agentic-workflow/state-machine.md"),
            "sdd"
        );
        // Top-level spec
        assert_eq!(crate_from_relative_path("change-spec-logic.md"), "");
    }

    #[test]
    fn test_priority_from_labels() {
        assert_eq!(
            priority_from_labels(&["priority:p0".into(), "type:bug".into()]),
            Some("critical".to_string())
        );
        assert_eq!(
            priority_from_labels(&["priority:p2".into()]),
            Some("medium".to_string())
        );
        assert_eq!(priority_from_labels(&["type:enhancement".into()]), None);
    }

    #[test]
    fn test_issue_status_str() {
        use crate::issues::types::IssueState;
        assert_eq!(issue_status_str(IssueState::Open), "open");
        assert_eq!(issue_status_str(IssueState::Draft), "open");
        assert_eq!(issue_status_str(IssueState::Closed), "closed");
    }

    #[test]
    fn test_map_phase_to_frontend() {
        assert_eq!(map_phase_to_frontend("change_inited"), "init");
        assert_eq!(map_phase_to_frontend("input_restructured"), "restructure");
        assert_eq!(
            map_phase_to_frontend("reference_context_created"),
            "reference_context"
        );
        assert_eq!(map_phase_to_frontend("change_spec_created"), "change_spec");
        assert_eq!(
            map_phase_to_frontend("change_implementation_created"),
            "implementation"
        );
        assert_eq!(map_phase_to_frontend("docs_created"), "review");
        assert_eq!(map_phase_to_frontend("change_archived"), "merge");
        assert_eq!(map_phase_to_frontend("change_rejected"), "merge");
        assert_eq!(map_phase_to_frontend("unknown_phase"), "init");
    }
}

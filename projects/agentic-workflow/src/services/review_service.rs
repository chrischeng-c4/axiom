// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/review_service_preamble_source.md#source
// CODEGEN-BEGIN
//! Review service — business logic for writing inline reviews.
//!
//! Extracted from `mcp/tools/review.rs`. Handles all artifact types with
//! full spec-aligned phase transition matrix.
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/review_service_phase_source.md#source
// CODEGEN-BEGIN
use crate::models::state::StatePhase;
use crate::tools::review_helpers::{
    build_review_section, remove_frontmatter_field, strip_review_section, upsert_frontmatter_field,
};
use crate::tools::workflow_common;
use crate::Result;
use serde_json::Value;
use std::path::Path;

/// Valid review file types (spec-aligned names + legacy aliases)
pub const VALID_FILES: &[&str] = &[
    // Pre-clarifications
    "pre_clarifications",
    "context_clarifications",
    // Reference context subtypes
    "reference_context",
    "spec_context",
    "knowledge_context",
    "codebase_context",
    "gap_codebase_spec",
    "gap_codebase_knowledge",
    "gap_spec_knowledge",
    // Post-clarifications
    "post_clarifications",
    "spec_clarifications",
    // Spec
    "change_spec",
    "spec",
    // Legacy
    "proposal",
    // Implementation
    "implementation",
    // Merge
    "merge",
];

/// Normalize artifact name: map legacy names to canonical spec names.
fn normalize_artifact(file: &str) -> &str {
    match file {
        "context_clarifications" => "pre_clarifications",
        "spec_clarifications" => "post_clarifications",
        "spec" => "change_spec",
        _ => file,
    }
}

/// Full phase transition matrix (spec-aligned: no approval states).
///
/// Maps (normalized_artifact, verdict) → target StatePhase.
/// Key invariant: both APPROVED and REVIEWED verdicts produce a *Reviewed phase.
/// route() reads verdict + revision_count to decide advancement vs. revision.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/review_service_phase_source.md#source
pub fn review_phase_transition(artifact: &str, verdict: &str) -> Option<StatePhase> {
    let normalized = normalize_artifact(artifact);
    match (normalized, verdict) {
        // Pre-clarifications: create-only, no phase transition from review
        ("pre_clarifications", _) => None,
        // Reference context: phase transition owned by sdd_run_change, not write_artifact
        (
            "reference_context"
            | "spec_context"
            | "knowledge_context"
            | "codebase_context"
            | "gap_codebase_spec"
            | "gap_codebase_knowledge"
            | "gap_spec_knowledge",
            _,
        ) => None,
        // Post-clarifications: create-only, no phase transition from review
        ("post_clarifications", _) => None,
        // Change spec: APPROVED|REVIEWED → ChangeSpecReviewed, REJECTED → ChangeRejected
        ("change_spec", "APPROVED" | "REVIEWED") => Some(StatePhase::ChangeSpecReviewed),
        ("change_spec", "REJECTED") => Some(StatePhase::ChangeRejected),
        // Implementation: APPROVED|REVIEWED → ChangeImplementationReviewed
        ("implementation", "APPROVED" | "REVIEWED") => {
            Some(StatePhase::ChangeImplementationReviewed)
        }
        // Merge: APPROVED|REVIEWED → ChangeMergeReviewed
        ("merge", "APPROVED" | "REVIEWED") => Some(StatePhase::ChangeMergeReviewed),
        // Proposal (legacy, no phase update)
        ("proposal", _) => None,
        _ => None,
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/review_service.md#schema
// CODEGEN-BEGIN
/// Input for writing a review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/review_service.md#schema
pub struct ReviewInput {
    /// Change identifier.
    pub change_id: String,
    /// File being reviewed.
    pub file: String,
    /// Review verdict (approved/needs-revision).
    pub verdict: String,
    /// Review summary.
    pub summary: String,
    /// Checklist entries.
    pub checklist: Vec<Value>,
    /// Identified issues.
    pub issues: Vec<Value>,
    /// Review iteration number.
    pub iteration: u64,
    /// Optional spec ID.
    pub spec_id: Option<String>,
    /// Optional task ID.
    pub task_id: Option<String>,
    /// Caller identifier.
    pub caller: String,
    /// Group ID for group-aware artifacts.
    pub group_id: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/review_service_runtime_source.md#source
// CODEGEN-BEGIN
/// Write an inline review and auto-update STATE.yaml phase.
///
/// Returns the status message string.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/review_service_runtime_source.md#source
pub fn write_review(input: ReviewInput, project_root: &Path) -> Result<String> {
    let file = &input.file;
    let verdict = &input.verdict;

    // Validate file type
    if !VALID_FILES.contains(&file.as_str()) {
        anyhow::bail!("Invalid file '{}'. Valid: {}", file, VALID_FILES.join(", "));
    }

    // Normalize legacy verdict names
    let verdict = match verdict.as_str() {
        "PASS" => "APPROVED".to_string(),
        "NEEDS_REVISION" => "REVIEWED".to_string(),
        _ => verdict.clone(),
    };

    if !["APPROVED", "REVIEWED", "REJECTED"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be APPROVED, REVIEWED, or REJECTED");
    }

    // spec_id required for spec/change_spec reviews
    if (file == "spec" || file == "change_spec") && input.spec_id.is_none() {
        anyhow::bail!("spec_id is required when file='spec' or file='change_spec'");
    }

    let change_dir = project_root.join(".aw/changes").join(&input.change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", input.change_id);
    }

    // Compute display name
    let display_name = if file == "spec" || file == "change_spec" {
        format!("spec:{}", input.spec_id.as_deref().unwrap_or("unknown"))
    } else if file == "implementation" && input.task_id.is_some() {
        format!("implementation:task_{}", input.task_id.as_deref().unwrap())
    } else {
        file.clone()
    };

    // Write inline review
    let filename = write_inline_review(
        &change_dir,
        file,
        &verdict,
        &input.summary,
        &input.checklist,
        &input.issues,
        input.iteration,
        &display_name,
        &input.change_id,
        input.spec_id.as_deref(),
        input.task_id.as_deref(),
        input.group_id.as_deref(),
    )?;

    // Auto-update STATE.yaml phase (full matrix)
    if let Some(phase) = review_phase_transition(file, &verdict) {
        workflow_common::update_phase(&change_dir, phase)?;
    }

    // Count issues
    let high = input
        .issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("HIGH"))
        .count();
    let medium = input
        .issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("MEDIUM"))
        .count();
    let low = input
        .issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("LOW"))
        .count();

    let status = format!(
        "Review written: {}\nVerdict: {}\nIssues: {} high, {} medium, {} low",
        filename, verdict, high, medium, low
    );
    if input.caller == "agent" {
        Ok(status)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            status
        ))
    }
}

/// Write inline review into the original artifact file.
fn write_inline_review(
    change_dir: &Path,
    file: &str,
    verdict: &str,
    summary: &str,
    checklist: &[Value],
    issues: &[Value],
    iteration: u64,
    display_name: &str,
    change_id: &str,
    spec_id: Option<&str>,
    task_id: Option<&str>,
    group_id: Option<&str>,
) -> Result<String> {
    // Merge is now programmatic — no review needed
    if file == "merge" {
        anyhow::bail!("Merge is now programmatic. Use sdd_workflow_create_change_merge instead.");
    }

    let target_path = if file == "spec" || file == "change_spec" {
        change_dir
            .join("specs")
            .join(format!("{}.md", spec_id.unwrap()))
    } else if file == "implementation" {
        match task_id {
            Some(tid) => change_dir.join(format!("impl_{}.md", tid)),
            None => change_dir.join("impl.md"),
        }
    } else {
        // Map spec-aligned names to file names
        let filename = match file {
            "pre_clarifications" | "context_clarifications" => "pre_clarifications",
            "post_clarifications" | "spec_clarifications" => "spec_clarifications",
            "reference_context" => "reference_context",
            _ => file,
        };
        // Group-aware path for reference_context
        if file == "reference_context"
            || file == "pre_clarifications"
            || file == "context_clarifications"
        {
            if let Some(gid) = group_id {
                change_dir
                    .join("groups")
                    .join(gid)
                    .join(format!("{}.md", filename))
            } else {
                change_dir.join(format!("{}.md", filename))
            }
        } else {
            change_dir.join(format!("{}.md", filename))
        }
    };

    if !target_path.exists() {
        anyhow::bail!("Artifact file not found: {}", target_path.display());
    }

    let original = std::fs::read_to_string(&target_path)?;

    // Strip existing # Reviews section
    let base = strip_review_section(&original);

    // Update frontmatter fields
    let updated = if verdict == "APPROVED" {
        let c = remove_frontmatter_field(&base, "review_verdict");
        remove_frontmatter_field(&c, "review_iteration")
    } else {
        let c = upsert_frontmatter_field(&base, "review_verdict", verdict);
        upsert_frontmatter_field(&c, "review_iteration", &iteration.to_string())
    };

    // Build final content
    let final_content = if verdict == "APPROVED" {
        format!("{}\n", updated)
    } else {
        let review = build_review_section(
            verdict,
            summary,
            checklist,
            issues,
            iteration,
            display_name,
            change_id,
        );
        format!("{}\n\n{}", updated, review)
    };

    std::fs::write(&target_path, &final_content)?;

    Ok(target_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change(change_id: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        let change_dir = project_root.join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), change_id);
        (temp_dir, project_root)
    }

    #[test]
    fn test_review_phase_transitions() {
        // Pre-clarifications: no phase transition (create-only)
        assert_eq!(
            review_phase_transition("pre_clarifications", "APPROVED"),
            None
        );
        assert_eq!(
            review_phase_transition("context_clarifications", "APPROVED"),
            None
        );
        // Reference context: phase transition owned by sdd_run_change → None
        assert_eq!(
            review_phase_transition("reference_context", "REVIEWED"),
            None
        );
        assert_eq!(review_phase_transition("spec_context", "REVIEWED"), None);
        // Post-clarifications: no phase transition (create-only)
        assert_eq!(
            review_phase_transition("post_clarifications", "APPROVED"),
            None
        );
        assert_eq!(
            review_phase_transition("spec_clarifications", "REVIEWED"),
            None
        );
        // Change spec: APPROVED|REVIEWED → ChangeSpecReviewed
        assert_eq!(
            review_phase_transition("change_spec", "APPROVED"),
            Some(StatePhase::ChangeSpecReviewed)
        );
        assert_eq!(
            review_phase_transition("spec", "REVIEWED"),
            Some(StatePhase::ChangeSpecReviewed)
        );
        // Implementation: APPROVED|REVIEWED → ChangeImplementationReviewed
        assert_eq!(
            review_phase_transition("implementation", "REVIEWED"),
            Some(StatePhase::ChangeImplementationReviewed)
        );
        assert_eq!(
            review_phase_transition("implementation", "APPROVED"),
            Some(StatePhase::ChangeImplementationReviewed)
        );
    }

    #[test]
    fn test_normalize_artifact() {
        assert_eq!(
            normalize_artifact("context_clarifications"),
            "pre_clarifications"
        );
        assert_eq!(
            normalize_artifact("spec_clarifications"),
            "post_clarifications"
        );
        assert_eq!(normalize_artifact("spec"), "change_spec");
        assert_eq!(normalize_artifact("implementation"), "implementation");
        assert_eq!(normalize_artifact("reference_context"), "reference_context");
    }

    #[test]
    fn test_valid_files_contains_all_types() {
        assert!(VALID_FILES.contains(&"pre_clarifications"));
        assert!(VALID_FILES.contains(&"context_clarifications"));
        assert!(VALID_FILES.contains(&"reference_context"));
        assert!(VALID_FILES.contains(&"post_clarifications"));
        assert!(VALID_FILES.contains(&"spec_clarifications"));
        assert!(VALID_FILES.contains(&"change_spec"));
        assert!(VALID_FILES.contains(&"spec"));
        assert!(VALID_FILES.contains(&"implementation"));
    }

    #[test]
    fn test_write_review_approved() {
        let (_tmp, root) = setup_change("test-review");
        let change_dir = root.join(".aw/changes/test-review");
        std::fs::write(
            change_dir.join("pre_clarifications.md"),
            "---\nid: test\ntype: clarifications\n---\n\n# Pre-Clarifications\n\nContent.\n",
        )
        .unwrap();

        let input = ReviewInput {
            change_id: "test-review".to_string(),
            file: "pre_clarifications".to_string(),
            verdict: "APPROVED".to_string(),
            summary: "All good".to_string(),
            checklist: vec![],
            issues: vec![],
            iteration: 1,
            spec_id: None,
            task_id: None,
            caller: "mainthread".to_string(),
            group_id: None,
        };
        let result = write_review(input, &root).unwrap();
        assert!(result.contains("APPROVED"));
    }

    #[test]
    fn test_merge_phase_transitions() {
        assert_eq!(
            review_phase_transition("merge", "APPROVED"),
            Some(StatePhase::ChangeMergeReviewed)
        );
        assert_eq!(
            review_phase_transition("merge", "REVIEWED"),
            Some(StatePhase::ChangeMergeReviewed)
        );
    }

    #[test]
    fn test_valid_files_contains_merge() {
        assert!(VALID_FILES.contains(&"merge"));
    }

    #[test]
    fn test_merge_review_bails() {
        let (_tmp, root) = setup_change("test-merge");

        let input = ReviewInput {
            change_id: "test-merge".to_string(),
            file: "merge".to_string(),
            verdict: "APPROVED".to_string(),
            summary: "test".to_string(),
            checklist: vec![],
            issues: vec![],
            iteration: 1,
            spec_id: None,
            task_id: None,
            caller: "agent".to_string(),
            group_id: None,
        };
        let result = write_review(input, &root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("programmatic"));
    }
}
// CODEGEN-END

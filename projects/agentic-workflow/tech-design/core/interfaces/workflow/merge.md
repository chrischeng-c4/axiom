---
id: projects-sdd-src-workflow-merge-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow state-machine interfaces drive TD/CB lifecycle transitions, review loops, merge, and validation gates."
---

# Standardized projects/agentic-workflow/src/workflow/merge.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/workflow/merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `handle` | projects/agentic-workflow/src/workflow/merge.rs | function | pub | 17 | handle(     change_dir: &Path,     change_id: &str,     _project_path: &str,     project_root: &Path, ) -> Result<Value> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/workflow/merge.rs -->
````rust
//! Merge flow: Begin/Resume/Review/Fix merge.
//!
//! Handles phases: ChangeMergeCreated, ChangeMergeReviewed, ChangeMergeRevised.

use super::helpers;
use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::path::Path;

/// Handle the merge flow.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/merge.md#source
pub fn handle(
    change_dir: &Path,
    change_id: &str,
    _project_path: &str,
    project_root: &Path,
) -> Result<Value> {
    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();

    let review_merge_path = change_dir.join("review_merge.md");
    let has_review_merge = review_merge_path.exists();

    let (review_verdict, review_issues) = if has_review_merge {
        helpers::extract_review_info(&review_merge_path)
    } else {
        (None, vec![])
    };

    let specs_to_merge = helpers::find_specs_to_merge(change_dir);
    let specs_merged = helpers::count_merged_specs(change_dir, project_root);

    let action = match &phase {
        // Entry: ChangeMergeCreated handles begin/resume/review routing internally
        StatePhase::ChangeMergeCreated => {
            let all_merged =
                specs_to_merge.is_empty() || specs_merged >= specs_to_merge.len();
            if !all_merged {
                Action::ResumeMerge
            } else if !has_review_merge {
                Action::ReviewMerge
            } else {
                match review_verdict.as_deref() {
                    Some("APPROVED") => Action::MergeComplete,
                    Some("REVIEWED") => Action::FixMerge,
                    Some("REJECTED") => {
                        return Ok(json!({
                            "action": "rejected",
                            "change_id": change_id,
                            "current_phase": "change_merge_created",
                            "message": "Merge was rejected.",
                            "prompt": "Merge was rejected due to fundamental problems.",
                        }));
                    }
                    _ => Action::ReviewMerge,
                }
            }
        }
        StatePhase::ChangeMergeReviewed => {
            let revision_count = sm.revision_count("merge");
            if revision_count >= 1 {
                Action::MergeComplete
            } else {
                Action::FixMerge
            }
        }
        StatePhase::ChangeMergeRevised => Action::ReviewMerge,
        // ChangeMergeApproved removed — MergeComplete is reached via
        // APPROVED verdict in ChangeMergeReviewed or ChangeMergeRevised
        _ => Action::BeginMerge,
    };

    let archive_path = format!(
        ".aw/archive/{}-{}",
        Utc::now().format("%Y%m%d"),
        change_id
    );

    let phase_str = helpers::phase_to_string(&phase);
    let mut base = json!({
        "change_id": change_id,
        "current_phase": phase_str,
        "has_review_merge": has_review_merge,
        "specs_to_merge": specs_to_merge,
        "specs_merged": specs_merged,
    });

    match action {
        Action::BeginMerge => {
            base["action"] = json!("begin_merge");
            base["message"] = json!("Start merging specs to main specs.");
            base["next_phase"] = json!("change_merge_created");
            base["target_dir"] = json!(format!("{}/.aw/tech-design", project_root.display()));
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Begin Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

                 ## Instructions\n\n\
                 1. Update STATE.yaml phase to 'merging'\n\
                 2. List specs in the change directory\n\
                 3. Read each spec's YAML frontmatter for `merge_strategy`\n\
                 4. Apply merge based on strategy:\n\
                    - `new` (default): Create a new spec file in .aw/tech-design/\n\
                    - `extend`: Add requirements/scenarios to existing main spec\n\
                    - `replace`: Replace the existing main spec entirely\n\
                    - `patch`: Partial update to specific sections of existing spec\n\
                 5. For each spec, include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths\n\
                 6. Call `score run-change` again when done\n\n\
                 ## Specs to merge\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = specs_to_merge.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ResumeMerge => {
            let remaining: Vec<_> = specs_to_merge.iter().skip(specs_merged).collect();
            base["action"] = json!("resume_merge");
            base["message"] = json!("Continue merging remaining specs.");
            base["remaining_specs"] = json!(&remaining);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Resume Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read `codebase_context.md` and `knowledge_context.md` via `score workflow read-artifact` (if not already read)\n\
                 2. Continue merging remaining specs to main specs\n\
                 3. For each remaining spec, call `score artifact write-artifact` — include in YAML frontmatter:\n\
                    - `codebase_paths`: list of relevant file paths from codebase_context\n\
                    - `knowledge_refs`: list of relevant knowledge doc paths from knowledge_context\n\
                 4. Call `score run-change` again when done\n\n\
                 ## Remaining specs\n{specs}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"codebase_context\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"knowledge_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                specs = remaining.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
                cid = change_id
            ));
        }
        Action::ReviewMerge => {
            base["action"] = json!("review_merge");
            base["message"] = json!("Create merge quality review.");
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Merge for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all merged specs from main specs directory\n\
                 2. Compare with original change specs\n\
                 3. Verify correctness, no conflicts, proper formatting\n\
                 4. Call `score artifact write-artifact` with artifact='merge', action='review'\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: All specs merged correctly\n\
                 - REVIEWED: Merge issues found\n\
                 - REJECTED: Fundamental merge problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:specs\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"list:main_specs\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id
            ));
        }
        Action::FixMerge => {
            base["action"] = json!("fix_merge");
            base["message"] = json!("Fix issues from merge review.");
            base["review_file"] = json!("review_merge.md");
            base["issues"] = json!(review_issues);
            base["issues_count"] = json!(review_issues.len());
            base["prompt"] = json!(format!(
                "# Task: Fix Merge Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_merge.md for issues\n\
                 2. Fix each issue in the merged specs\n\
                 3. Re-merge corrected specs via `score artifact write-artifact`\n\
                 4. Delete review_merge.md after fixing\n\
                 5. Update STATE.yaml phase to 'merge_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_merge\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 score run-change --change-id {cid}\n\
                 ```",
                cid = change_id
            ));
        }
        Action::MergeComplete => {
            base["action"] = json!("merge_complete");
            base["message"] = json!("Merge complete! Archive the change.");
            base["next_phase"] = json!("archived");
            base["archive_path"] = json!(archive_path);
            base["prompt"] = json!(format!(
                "# Merge Complete for Change '{}'\n\n\
                 Merge has been approved:\n\
                 - All specs merged to main\n\
                 - Merge review approved\n\n\
                 1. Move change directory to archive:\n\
                    mv .aw/changes/{cid} {archive}\n\
                 2. Update STATE.yaml phase to 'archived'\n\n\
                 SDD SDD workflow complete!",
                change_id, cid = change_id, archive = archive_path
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    BeginMerge,
    ResumeMerge,
    ReviewMerge,
    FixMerge,
    MergeComplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".aw/tech-design")).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    #[test]
    fn test_merge_created_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_with_specs_resumes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs/auth-flow.md"), "# Spec").unwrap();
        std::fs::write(change_dir.join("specs/user-model.md"), "# Spec").unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "resume_merge");
    }

    #[test]
    fn test_merging_all_merged_triggers_review() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "review_merge");
    }

    #[test]
    fn test_approved_merge_completes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: APPROVED\n---\n# Review\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
    }

    #[test]
    fn test_needs_revision_fixes() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_created");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(
            change_dir.join("review_merge.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Spec conflict\n",
        )
        .unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "fix_merge");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_merge_complete_has_archive_path() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        assert_eq!(result["action"], "merge_complete");
        assert!(result["archive_path"]
            .as_str()
            .unwrap()
            .contains("test-change"));
    }

    #[test]
    fn test_merge_complete_prompt_mentions_archive() {
        let (temp_dir, change_dir) = setup_change_dir("change_merge_reviewed");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        // Set revision_count >= 1 to trigger MergeComplete
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("merge");
        sm.save().unwrap();
        let pp = temp_dir.path().display().to_string();
        let result = handle(&change_dir, "test-change", &pp, temp_dir.path()).unwrap();
        let prompt = result["prompt"].as_str().unwrap();
        assert!(prompt.contains("archive"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/workflow/merge.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete merge workflow router.
```

---
id: sdd-tools-implementation-create-merge-review-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Create Merge Review Tool

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/implementation.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `create_merge_review_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 687 | create_merge_review_definition() -> ToolDefinition |
| `create_review_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 477 | create_review_definition() -> ToolDefinition |
| `execute_create_merge_review` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 781 | execute_create_merge_review(args: &Value, project_root: &Path) -> Result<String> |
| `execute_create_review` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 559 | execute_create_review(args: &Value, project_root: &Path) -> Result<String> |
| `execute_list_changed_files` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 356 | execute_list_changed_files(args: &Value, _project_root: &Path) -> Result<String> |
| `execute_read_all_requirements` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 117 | execute_read_all_requirements(args: &Value, project_root: &Path) -> Result<String> |
| `execute_read_implementation_summary` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 232 | execute_read_implementation_summary(args: &Value, _project_root: &Path) -> Result<String> |
| `list_changed_files_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 323 | list_changed_files_definition() -> ToolDefinition |
| `read_all_requirements_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 93 | read_all_requirements_definition() -> ToolDefinition |
| `read_implementation_summary_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 204 | read_implementation_summary_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Get the tool definition for review_merge
pub fn create_merge_review_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_review_merge".to_string(),
        description:
            "Create structured review_merge.md file with merge quality assessment and verdict"
                .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "iteration", "summary", "merge_quality", "verdict"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling: agent (via sdd_delegate_agent) or mainthread. Controls whether next dispatch hint is included in response."
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID being reviewed"
                },
                "iteration": {
                    "type": "integer",
                    "description": "Review iteration number (starts at 1)"
                },
                "summary": {
                    "type": "string",
                    "description": "Brief summary of the merge quality assessment"
                },
                "merge_quality": {
                    "type": "string",
                    "enum": ["CLEAN", "PARTIAL", "FAILED"],
                    "description": "Overall merge quality status"
                },
                "requirements_preserved": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether requirements are preserved in merged specs"
                },
                "scenarios_preserved": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether scenarios are preserved in merged specs"
                },
                "diagrams_preserved": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether diagrams are preserved in merged specs"
                },
                "changelog_present": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether CHANGELOG entry is present"
                },
                "changelog_accurate": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether CHANGELOG entry accurately describes the change"
                },
                "issues": {
                    "type": "array",
                    "default": [],
                    "description": "List of issues found during merge review",
                    "items": {
                        "type": "object",
                        "required": ["severity", "description"],
                        "properties": {
                            "severity": {
                                "type": "string",
                                "enum": ["HIGH", "MEDIUM"],
                                "description": "Issue severity"
                            },
                            "description": {
                                "type": "string",
                                "description": "Description of the issue"
                            }
                        }
                    }
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED", "REJECTED"],
                    "description": "Review verdict: APPROVED (ready for archive), REVIEWED (fixable issues), REJECTED (manual intervention needed)"
                }
            }
        }),
    }
}

/// Execute the create_merge_review tool
pub fn execute_create_merge_review(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let iteration = args.get("iteration").and_then(|v| v.as_u64()).unwrap_or(1) as u32;

    let summary = get_required_string(args, "summary")?;

    // Parse merge quality
    let merge_quality_str = get_required_string(args, "merge_quality")?;
    let merge_quality = match merge_quality_str.to_uppercase().as_str() {
        "CLEAN" => MergeQuality::Clean,
        "PARTIAL" => MergeQuality::Partial,
        "FAILED" => MergeQuality::Failed,
        _ => anyhow::bail!("Invalid merge_quality: {}", merge_quality_str),
    };

    // Parse preservation flags (default to true)
    let requirements_preserved = args
        .get("requirements_preserved")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let scenarios_preserved = args
        .get("scenarios_preserved")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let diagrams_preserved = args
        .get("diagrams_preserved")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let changelog_present = args
        .get("changelog_present")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let changelog_accurate = args
        .get("changelog_accurate")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Parse issues
    let issues_array = args
        .get("issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut issues = Vec::new();
    for issue_val in issues_array {
        let severity_str = issue_val
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("MEDIUM");
        let severity = match severity_str.to_uppercase().as_str() {
            "HIGH" => Severity::High,
            _ => Severity::Medium,
        };

        let description = issue_val
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        issues.push(MergeReviewIssue {
            severity,
            description,
        });
    }

    // Parse verdict
    let verdict_str = get_required_string(args, "verdict")?;
    let verdict = match verdict_str.to_uppercase().as_str() {
        "APPROVED" => MergeReviewVerdict::Approved,
        "REVIEWED" => MergeReviewVerdict::Reviewed,
        "REJECTED" => MergeReviewVerdict::Rejected,
        _ => anyhow::bail!("Invalid verdict: {}", verdict_str),
    };

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    let input = CreateMergeReviewInput {
        change_id,
        iteration,
        summary,
        merge_quality,
        requirements_preserved,
        scenarios_preserved,
        diagrams_preserved,
        changelog_present,
        changelog_accurate,
        issues,
        verdict,
    };

    let result = crate::services::implementation_service::create_merge_review(input, project_root)?;

    // Auto-update STATE.yaml phase
    super::workflow_common::update_phase(&change_dir, StatePhase::ChangeMergeReviewed)?;

    if caller == "agent" {
        Ok(result)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            result
        ))
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/implementation.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "create_merge_review_definition"
      - "execute_create_merge_review"
    description: "Tool definition and execution for creating structured merge review artifacts."
```

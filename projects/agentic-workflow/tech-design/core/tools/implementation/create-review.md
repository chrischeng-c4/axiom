---
id: sdd-tools-implementation-create-review-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Create Implementation Review Tool

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
/// Get the tool definition for review_implementation
pub fn create_review_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_review_implementation".to_string(),
        description: "Create structured review_impl.md file with test results, issues, and verdict"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "iteration", "verdict", "issues"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID being reviewed"
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling: agent (via sdd_delegate_agent) or mainthread. Controls whether next dispatch hint is included in response."
                },
                "iteration": {
                    "type": "integer",
                    "description": "Review iteration number (starts at 0)"
                },
                "test_results": {
                    "type": "object",
                    "description": "Test execution results",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["PASS", "FAIL", "PARTIAL", "UNKNOWN"],
                            "description": "Overall test status"
                        },
                        "total": { "type": "integer" },
                        "passed": { "type": "integer" },
                        "failed": { "type": "integer" },
                        "skipped": { "type": "integer" }
                    }
                },
                "security_status": {
                    "type": "string",
                    "enum": ["CLEAN", "WARNINGS", "VULNERABILITIES", "NOT_RUN"],
                    "description": "Security scan status"
                },
                "issues": {
                    "type": "array",
                    "description": "List of issues found",
                    "items": {
                        "type": "object",
                        "required": ["severity", "title", "description"],
                        "properties": {
                            "severity": {
                                "type": "string",
                                "enum": ["HIGH", "MEDIUM", "LOW"]
                            },
                            "title": { "type": "string" },
                            "description": { "type": "string" },
                            "file_path": { "type": "string" },
                            "line_number": { "type": "integer" },
                            "recommendation": { "type": "string" }
                        }
                    }
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED", "REJECTED"],
                    "description": "Review verdict"
                },
                "next_steps": {
                    "type": "string",
                    "description": "Suggested next steps"
                }
            }
        }),
    }
}

/// Execute the create_review tool
pub fn execute_create_review(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let iteration = args.get("iteration").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

    // Parse test results
    let test_results = if let Some(tr) = args.get("test_results") {
        TestResults {
            status: tr
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string(),
            total: tr.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            passed: tr.get("passed").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            failed: tr.get("failed").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            skipped: tr.get("skipped").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        }
    } else {
        TestResults {
            status: "UNKNOWN".to_string(),
            ..Default::default()
        }
    };

    let security_status =
        get_optional_string(args, "security_status").unwrap_or_else(|| "NOT_RUN".to_string());

    // Parse issues
    let issues_array = get_required_array(args, "issues")?;
    let mut issues = Vec::new();

    for issue_val in issues_array {
        let severity_str = issue_val
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("MEDIUM");
        let severity = match severity_str {
            "HIGH" => Severity::High,
            "LOW" => Severity::Low,
            _ => Severity::Medium,
        };

        let title = issue_val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Issue")
            .to_string();

        let description = issue_val
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let file_path = issue_val
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let line_number = issue_val
            .get("line_number")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);
        let recommendation = issue_val
            .get("recommendation")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        issues.push(ReviewIssue {
            severity,
            title,
            description,
            file_path,
            line_number,
            recommendation,
        });
    }

    // Parse verdict
    let verdict_str = get_required_string(args, "verdict")?;
    let verdict = match verdict_str.as_str() {
        "APPROVED" => ReviewVerdict::Approved,
        "REJECTED" => ReviewVerdict::Rejected,
        _ => ReviewVerdict::Reviewed,
    };

    let next_steps = get_optional_string(args, "next_steps");

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    let input = CreateReviewInput {
        change_id,
        iteration,
        test_results,
        security_status,
        issues,
        verdict,
        next_steps,
    };

    let result = crate::services::implementation_service::create_review(input, project_root)?;

    // Auto-update STATE.yaml phase
    super::workflow_common::update_phase(&change_dir, StatePhase::ChangeImplementationReviewed)?;

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
      - "create_review_definition"
      - "execute_create_review"
    description: "Tool definition and execution for creating structured implementation review artifacts."
```

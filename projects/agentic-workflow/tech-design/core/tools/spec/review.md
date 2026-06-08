---
id: sdd-tools-spec-review
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools spec review

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 41 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 238 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_review_spec` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 574 | execute_review_spec(args: &Value, project_root: &Path) -> Result<String> |
| `review_spec_definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 476 | review_spec_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ============================================================================
// JSON parsing helpers
// ============================================================================

/// Parse an optional string array from JSON args
fn parse_string_array_opt(args: &Value, field: &str) -> Vec<String> {
    args.get(field)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Parse changes array from JSON args
fn parse_changes(args: &Value) -> Vec<SpecChangeData> {
    args.get("changes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(SpecChangeData {
                        file: v.get("file")?.as_str()?.to_string(),
                        action: v.get("action")?.as_str()?.to_string(),
                        context_ref: v
                            .get("context_ref")
                            .and_then(|r| r.as_str())
                            .map(String::from),
                        description: v
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// ============================================================================
// New Review Tool: sdd_review_spec
// ============================================================================

/// Get the tool definition for review_spec
pub fn review_spec_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_review_spec".to_string(),
        description: "Create REVIEW_SPEC_{spec_id}.md with structured review verdict. Overwrites on each iteration."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "spec_id", "iteration", "summary", "verdict"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Unique identifier for the change (lowercase, hyphens allowed)"
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling: agent (via sdd_delegate_agent) or mainthread. Controls whether next dispatch hint is included in response."
                },
                "spec_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Spec ID being reviewed"
                },
                "iteration": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Review iteration number (starts at 1)"
                },
                "summary": {
                    "type": "string",
                    "minLength": 20,
                    "description": "Summary of the review findings"
                },
                "validation_passed": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether automated validation passed"
                },
                "missing_elements": {
                    "type": "array",
                    "default": [],
                    "items": { "type": "string" },
                    "description": "List of missing required elements"
                },
                "coverage": {
                    "type": "string",
                    "description": "Requirements-to-scenarios coverage (e.g., '5 scenarios for 3 requirements')"
                },
                "issues": {
                    "type": "array",
                    "default": [],
                    "description": "List of issues found",
                    "items": {
                        "type": "object",
                        "required": ["severity", "description"],
                        "properties": {
                            "severity": {
                                "type": "string",
                                "enum": ["HIGH", "MEDIUM", "LOW"],
                                "description": "Issue severity"
                            },
                            "requirement_id": {
                                "type": "string",
                                "description": "Related requirement ID (e.g., 'R1')"
                            },
                            "description": {
                                "type": "string",
                                "description": "Description of the issue"
                            },
                            "recommendation": {
                                "type": "string",
                                "description": "How to fix the issue"
                            }
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

/// Execute the review_spec tool
pub fn execute_review_spec(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let iteration = args.get("iteration").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let summary = get_required_string(args, "summary")?;
    let verdict_str = get_required_string(args, "verdict")?;
    let next_steps = get_optional_string(args, "next_steps");
    let coverage = get_optional_string(args, "coverage");
    let validation_passed = args
        .get("validation_passed")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Validate change_id format
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("change_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate spec_id format
    if !spec_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("spec_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate verdict (accept both old and new names)
    if !["APPROVED", "REVIEWED", "REJECTED", "NEEDS_REVISION"].contains(&verdict_str.as_str()) {
        anyhow::bail!("verdict must be 'APPROVED', 'REVIEWED', or 'REJECTED'");
    }

    // Normalize legacy verdict name
    let verdict_str = if verdict_str == "NEEDS_REVISION" {
        "REVIEWED".to_string()
    } else {
        verdict_str
    };

    // Get change directory
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Run create_proposal first.",
            change_id
        );
    }

    // Parse arrays
    let missing_elements: Vec<String> = args
        .get("missing_elements")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let issues_array = args
        .get("issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Build REVIEW_SPEC_{spec_id}.md content
    let mut content = String::new();

    // Header
    content.push_str(&format!(
        "# Spec Review: {} (Iteration {})\n\n",
        spec_id, iteration
    ));
    content.push_str(&format!("**Change ID**: {}\n\n", change_id));

    // Summary
    content.push_str("## Summary\n\n");
    content.push_str(&summary);
    content.push_str("\n\n");

    // Validation Results
    content.push_str("## Validation Results\n\n");
    content.push_str(&format!(
        "- **Completeness**: {}\n",
        if validation_passed { "PASS" } else { "FAIL" }
    ));
    if !missing_elements.is_empty() {
        content.push_str(&format!(
            "- **Missing elements**: {}\n",
            missing_elements.join(", ")
        ));
    }
    if let Some(ref cov) = coverage {
        content.push_str(&format!("- **Coverage**: {}\n", cov));
    }
    content.push('\n');

    // Issues
    content.push_str("## Issues\n\n");
    if issues_array.is_empty() {
        content.push_str("No issues found.\n\n");
    } else {
        for issue in &issues_array {
            let severity = issue
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("MEDIUM");
            let req_id = issue.get("requirement_id").and_then(|v| v.as_str());
            let description = issue
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let recommendation = issue.get("recommendation").and_then(|v| v.as_str());

            if let Some(rid) = req_id {
                content.push_str(&format!("- **[{}]** {}: {}\n", severity, rid, description));
            } else {
                content.push_str(&format!("- **[{}]** {}\n", severity, description));
            }
            if let Some(rec) = recommendation {
                content.push_str(&format!("  - *Recommendation*: {}\n", rec));
            }
        }
        content.push('\n');
    }

    // Verdict with checkbox format
    content.push_str("## Verdict\n\n");
    match verdict_str.as_str() {
        "APPROVED" => {
            content.push_str("- [x] APPROVED - Spec passes validation and manual review\n");
            content.push_str(
                "- [ ] REVIEWED - Missing elements, unclear requirements, insufficient scenarios\n",
            );
            content.push_str("- [ ] REJECTED - Fundamental design problems, wrong spec_type\n");
        }
        "REVIEWED" => {
            content.push_str("- [ ] APPROVED - Spec passes validation and manual review\n");
            content.push_str(
                "- [x] REVIEWED - Missing elements, unclear requirements, insufficient scenarios\n",
            );
            content.push_str("- [ ] REJECTED - Fundamental design problems, wrong spec_type\n");
        }
        "REJECTED" => {
            content.push_str("- [ ] APPROVED - Spec passes validation and manual review\n");
            content.push_str(
                "- [ ] REVIEWED - Missing elements, unclear requirements, insufficient scenarios\n",
            );
            content.push_str("- [x] REJECTED - Fundamental design problems, wrong spec_type\n");
        }
        _ => {}
    }
    content.push('\n');

    // Next steps
    if let Some(ref steps) = next_steps {
        content.push_str(&format!("**Next Steps**: {}\n", steps));
    } else {
        content.push_str("**Next Steps**: ");
        match verdict_str.as_str() {
            "APPROVED" => content.push_str("Spec is ready for implementation.\n"),
            "REVIEWED" => content.push_str("Address issues above and revise spec.\n"),
            "REJECTED" => {
                content.push_str("Redesign the spec with correct spec_type and structure.\n")
            }
            _ => content.push_str("Review the findings.\n"),
        }
    }

    // Write the file (overwrites each iteration)
    let review_path = change_dir.join(format!("REVIEW_SPEC_{}.md", spec_id));
    std::fs::write(&review_path, &content)?;

    // Count issues by severity
    let high_count = issues_array
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("HIGH"))
        .count();
    let medium_count = issues_array
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("MEDIUM"))
        .count();
    let low_count = issues_array
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("LOW"))
        .count();

    let status = format!(
        "REVIEW_SPEC_{}.md written: {}\n  Verdict: {}\n  Issues: {} high, {} medium, {} low",
        spec_id,
        review_path.display(),
        verdict_str,
        high_count,
        medium_count,
        low_count
    );
    if caller == "agent" {
        Ok(status)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            status
        ))
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - parse_string_array_opt
      - parse_changes
      - review_spec_definition
      - execute_review_spec
    description: "Review-spec helpers, schema, and execution path."
```

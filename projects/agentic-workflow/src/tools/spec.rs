// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec/preamble.md#source
// CODEGEN-BEGIN
//! create_spec MCP Tool
//!
//! Creates a validated spec file with requirements and acceptance criteria.
#![allow(deprecated)]
//!
//! ## Structured Diagrams
//!
//! The `diagrams` field accepts structured diagram definitions that are validated
//! against their corresponding Mermaid tool schemas. This ensures diagrams are
//! syntactically correct and enables semantic metadata for code generation.
//!
//! Supported diagram types:
//! - `flowchart` - Process flows, algorithms, decision trees (with semantic extensions)
//! - `sequence` - API interactions, message flows
//! - `class` - Data structures, domain models
//! - `state` - State machines, workflow states
//! - `erd` - Database schemas, entity relationships
//! - `mindmap` - Concept organization
//! - `requirement` - Requirement traceability
//! - `journey` - User journeys

use super::{get_optional_string, get_required_array, get_required_string, ToolDefinition};
use crate::models::spec_rules::{ApiSpecType, SpecType};
use crate::models::state::StatePhase;
use crate::services::spec_service::{
    create_spec, ApiSpecData, CreateSpecInput, DiagramData, RequirementData, ScenarioData,
    SpecChangeData,
};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::str::FromStr;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec/definition.md#source
// CODEGEN-BEGIN
/// Get the tool definition for create_spec
/// @spec projects/agentic-workflow/tech-design/core/tools/spec/definition.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_create_spec".to_string(),
        description: "Create a validated spec file with requirements and acceptance criteria. Supports structured diagrams for spec-to-code generation."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "spec_id", "title", "overview", "spec_type", "requirements", "scenarios"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID this spec belongs to"
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
                    "description": "Unique identifier for this spec (lowercase, hyphens allowed)"
                },
                "title": {
                    "type": "string",
                    "description": "Human-readable title for the spec"
                },
                "overview": {
                    "type": "string",
                    "minLength": 50,
                    "description": "Overview of what this spec covers"
                },
                "spec_type": {
                    "type": "string",
                    "enum": ["http-api", "event-driven", "data-model", "algorithm", "integration", "utility", "rpc-api", "workflow"],
                    "description": "Spec type classification. Determines required elements: http-api (sequence diagram + OpenAPI), event-driven (sequence + AsyncAPI), data-model (erd/class diagram), algorithm (flowchart/state), integration (sequence), utility (none), rpc-api (class diagram + OpenRPC), workflow (state/flowchart + Serverless Workflow)"
                },
                "requirements": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "description"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^R\\d+$",
                                "description": "Requirement ID (e.g., R1, R2)"
                            },
                            "title": {
                                "type": "string",
                                "description": "Short requirement title"
                            },
                            "description": {
                                "type": "string",
                                "description": "Detailed requirement description"
                            },
                            "priority": {
                                "enum": ["high", "medium", "low"],
                                "default": "medium",
                                "description": "Requirement priority"
                            }
                        }
                    },
                    "description": "List of requirements"
                },
                "scenarios": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["name", "when", "then"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Scenario name"
                            },
                            "given": {
                                "type": "string",
                                "description": "Optional precondition"
                            },
                            "when": {
                                "type": "string",
                                "description": "Action or trigger"
                            },
                            "then": {
                                "type": "string",
                                "description": "Expected outcome"
                            }
                        }
                    },
                    "description": "Acceptance scenarios in Given/When/Then format"
                },
                "diagrams": {
                    "type": "array",
                    "description": "Structured diagram definitions using Mermaid tool schemas. Preferred over flow_diagram.",
                    "items": {
                        "type": "object",
                        "required": ["type", "title", "input"],
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["flowchart", "sequence", "class", "state", "erd", "mindmap", "requirement", "journey"],
                                "description": "Diagram type (matches generate_mermaid_* tool)"
                            },
                            "title": {
                                "type": "string",
                                "description": "Human-readable title for the diagram"
                            },
                            "input": {
                                "type": "object",
                                "description": "Input matching the corresponding generate_mermaid_* tool schema"
                            }
                        }
                    }
                },
                "flow_diagram": {
                    "type": "string",
                    "description": "DEPRECATED: Use 'diagrams' field instead. Raw Mermaid diagram code."
                },
                "data_model": {
                    "type": "object",
                    "description": "Optional JSON Schema for data model"
                },
                "api_spec": {
                    "type": "object",
                    "description": "API specification (OpenAPI 3.1, AsyncAPI 2.6, JSON Schema, OpenRPC 1.3, or Serverless Workflow 0.8) for code generation",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["openapi-3.1", "asyncapi-2.6", "json-schema", "openrpc-1.3", "serverless-workflow-0.8"],
                            "description": "API specification format"
                        },
                        "spec": {
                            "type": "object",
                            "description": "Full API specification object"
                        }
                    },
                    "required": ["type", "spec"]
                },
                "spec_group": {
                    "type": "string",
                    "pattern": "^[a-z][a-z0-9-]*$",
                    "description": "Spec group for organizing specs (e.g., 'genesis', 'lens', 'auth'). Creates spec in specs/{spec_group}/ subdirectory. Omit for cross-cutting specs."
                },
                "group_id": {
                    "type": "string",
                    "description": "Change group ID for multi-group changes. When set, spec is written to groups/{group_id}/specs/ instead of specs/. Takes priority over spec_group."
                },
                "main_spec_ref": {
                    "type": "string",
                    "description": "Reference to existing main spec that this change spec extends/modifies. Used for traceability during merge. Example: 'auth-flow' means this extends .aw/tech-design/{spec_group}/auth-flow.md"
                },
                "merge_strategy": {
                    "type": "string",
                    "enum": ["new", "extend", "replace", "patch"],
                    "description": "Strategy for merging this spec back to main specs: new (create new), extend (add to existing), replace (overwrite), patch (partial update)"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Explicit tags (merged with auto-tags from spec_type). Values: api, http, rpc, events, async, data, logic, state, external"
                },
                "depends": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Dependency spec IDs (for topological ordering during creation)"
                },
                "changes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["file", "action"],
                        "properties": {
                            "file": { "type": "string", "description": "File path relative to project root" },
                            "action": { "type": "string", "enum": ["CREATE", "MODIFY", "DELETE"] },
                            "context_ref": { "type": "string", "description": "Reference to context artifact section" },
                            "description": { "type": "string", "description": "What changes in this file" }
                        }
                    },
                    "description": "File changes associated with this spec"
                }
            }
        }),
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec/execute.md#source
// CODEGEN-BEGIN
/// Execute the create_spec tool
/// @spec projects/agentic-workflow/tech-design/core/tools/spec/execute.md#source
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");

    // Extract required fields
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let title = get_required_string(args, "title")?;
    let overview = get_required_string(args, "overview")?;
    let requirements = get_required_array(args, "requirements")?;
    let scenarios = get_required_array(args, "scenarios")?;

    // Optional fields
    let flow_diagram = get_optional_string(args, "flow_diagram");
    let data_model = args.get("data_model").cloned();

    // Parse spec_type (required field)
    let spec_type_str = get_required_string(args, "spec_type")?;
    let spec_type = SpecType::from_str(&spec_type_str)
        .map_err(|_| anyhow::anyhow!(
            "Invalid spec_type '{}'. Valid types: http-api, event-driven, data-model, algorithm, integration, utility, rpc-api, workflow",
            spec_type_str
        ))?;

    // Parse structured diagrams
    let diagrams_array = args.get("diagrams").and_then(|v| v.as_array()).cloned();
    let diagrams_vec: Vec<DiagramData> = diagrams_array
        .unwrap_or_default()
        .iter()
        .filter_map(|d| {
            Some(DiagramData {
                diagram_type: d.get("type")?.as_str()?.to_string(),
                title: d.get("title")?.as_str()?.to_string(),
                input: d.get("input")?.clone(),
                rendered: None,
                semantic: None,
            })
        })
        .collect();

    // Validate diagram types
    let valid_diagram_types = [
        "flowchart",
        "sequence",
        "class",
        "state",
        "erd",
        "mindmap",
        "requirement",
        "journey",
    ];
    for diagram in &diagrams_vec {
        if !valid_diagram_types.contains(&diagram.diagram_type.as_str()) {
            anyhow::bail!(
                "Invalid diagram type '{}'. Valid types: {:?}",
                diagram.diagram_type,
                valid_diagram_types
            );
        }
    }

    // Parse API spec if provided
    let api_spec = if let Some(api_spec_obj) = args.get("api_spec") {
        let spec_type_str = api_spec_obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("api_spec missing 'type' field"))?;
        let spec_type = ApiSpecType::from_str(spec_type_str)
            .map_err(|_| anyhow::anyhow!("Invalid api_spec type: {}", spec_type_str))?;
        let spec = api_spec_obj
            .get("spec")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("api_spec missing 'spec' field"))?;
        Some(ApiSpecData { spec_type, spec })
    } else {
        None
    };

    // Parse spec_group if provided
    let spec_group = get_optional_string(args, "spec_group");

    // Parse group_id for multi-group change layout
    let group_id = get_optional_string(args, "group_id");

    // Parse main_spec_ref and merge_strategy (for main spec awareness)
    let main_spec_ref = get_optional_string(args, "main_spec_ref");
    let merge_strategy = get_optional_string(args, "merge_strategy");

    // Convert requirements JSON array to RequirementData (with explicit errors)
    let requirements_vec: Vec<RequirementData> = requirements
        .iter()
        .enumerate()
        .map(|(i, r)| {
            Ok(RequirementData {
                id: r
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("requirements[{}]: missing 'id'", i))?
                    .to_string(),
                title: r
                    .get("title")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("requirements[{}]: missing 'title'", i))?
                    .to_string(),
                description: r
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("requirements[{}]: missing 'description'", i))?
                    .to_string(),
                priority: r
                    .get("priority")
                    .and_then(|p| p.as_str())
                    .unwrap_or("medium")
                    .to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Convert scenarios JSON array to ScenarioData (with explicit errors)
    let scenarios_vec: Vec<ScenarioData> = scenarios
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Ok(ScenarioData {
                name: s
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("scenarios[{}]: missing 'name'", i))?
                    .to_string(),
                given: s.get("given").and_then(|g| g.as_str()).map(String::from),
                when: s
                    .get("when")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("scenarios[{}]: missing 'when'", i))?
                    .to_string(),
                then: s
                    .get("then")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("scenarios[{}]: missing 'then'", i))?
                    .to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Save for state update after artifact creation
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    // Create input struct and call service
    // Note: agent and duration_secs are set by the workflow engine post-processing
    let input = CreateSpecInput {
        change_id,
        spec_id,
        title,
        overview,
        requirements: requirements_vec,
        scenarios: scenarios_vec,
        spec_type,
        diagrams: diagrams_vec,
        flow_diagram,
        data_model,
        api_spec,
        agent: None,
        duration_secs: None,
        spec_group,
        group_id,
        main_spec_ref,
        merge_strategy,
        tags: parse_string_array_opt(args, "tags"),
        changes: parse_changes(args),
        depends: parse_string_array_opt(args, "depends"),
    };

    let result = create_spec(input, project_root)?;

    // Auto-update STATE.yaml phase
    super::workflow_common::update_phase(&change_dir, StatePhase::ChangeSpecCreated)?;

    if caller == "agent" {
        Ok(result)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            result
        ))
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec/review.md#source
// CODEGEN-BEGIN
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
/// @spec projects/agentic-workflow/tech-design/core/tools/spec/review.md#source
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
/// @spec projects/agentic-workflow/tech-design/core/tools/spec/review.md#source
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
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(project_root, "test-change");

        let args = json!({
            "change_id": "test-change",
            "spec_id": "mcp-protocol",
            "spec_type": "utility",
            "title": "MCP Protocol Implementation",
            "overview": "This specification covers the implementation of the Model Context Protocol (MCP) server for genesis, providing structured tools for proposal generation.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "JSON-RPC 2.0 Support",
                    "description": "The server must support JSON-RPC 2.0 protocol over stdio",
                    "priority": "high"
                },
                {
                    "id": "R2",
                    "title": "Tool Registration",
                    "description": "Tools must be registered and callable via tools/call method",
                    "priority": "high"
                }
            ],
            "scenarios": [
                {
                    "name": "Server Initialization",
                    "given": "MCP client is connected",
                    "when": "Client sends initialize request",
                    "then": "Server responds with capabilities"
                },
                {
                    "name": "Tool Execution",
                    "when": "Client calls create_proposal tool",
                    "then": "Server creates proposal.md and returns success"
                }
            ],
            "flow_diagram": "graph LR\n    A[Client] --> B[Server]\n    B --> C[Tool Registry]\n    C --> D[Execute Tool]"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created
        let spec_path = project_root.join(".aw/changes/test-change/specs/mcp-protocol.md");
        assert!(spec_path.exists());

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: mcp-protocol"));
        assert!(content.contains("spec_type: utility"));
        assert!(content.contains("## Requirements"));
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("### Scenario:"));
        assert!(content.contains("**WHEN**"));
        assert!(content.contains("**THEN**"));
        assert!(content.contains("```mermaid"));
    }

    // R6: Scenario - create_spec enforces http-api requires sequence diagram
    #[test]
    fn test_create_spec_http_api_missing_sequence_diagram() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create http-api spec WITHOUT sequence diagram but WITH OpenAPI
        let args = json!({
            "change_id": "test-change",
            "spec_id": "api-spec",
            "spec_type": "http-api",
            "title": "User API",
            "overview": "This specification defines the REST API for managing user resources and their profiles.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "List Users",
                    "description": "List all users"
                }
            ],
            "scenarios": [
                {
                    "name": "Get users",
                    "when": "GET /users is called",
                    "then": "user list is returned"
                }
            ],
            "api_spec": {
                "type": "openapi-3.1",
                "spec": {
                    "openapi": "3.1.0",
                    "info": {"title": "User API", "version": "1.0.0"},
                    "paths": {
                        "/users": {
                            "get": {"summary": "List users"}
                        }
                    }
                }
            }
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when http-api spec has no sequence diagram"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("sequence"),
            "Error should mention missing sequence diagram, got: {}",
            err_msg
        );
    }

    // R6: Scenario - create_spec enforces http-api requires OpenAPI spec
    #[test]
    fn test_create_spec_http_api_missing_openapi() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create http-api spec WITH sequence diagram but WITHOUT OpenAPI
        let args = json!({
            "change_id": "test-change",
            "spec_id": "api-spec",
            "spec_type": "http-api",
            "title": "User API",
            "overview": "This specification defines the REST API for managing user resources and their profiles.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "List Users",
                    "description": "List all users"
                }
            ],
            "scenarios": [
                {
                    "name": "Get users",
                    "when": "GET /users is called",
                    "then": "user list is returned"
                }
            ],
            "flow_diagram": "sequenceDiagram\n    Client->>API: GET /users\n    API-->>Client: 200 OK"
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when http-api spec has no OpenAPI spec"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("OpenAPI") || err_msg.contains("openapi-3.1"),
            "Error should mention missing OpenAPI spec, got: {}",
            err_msg
        );
    }

    // R6: Scenario - create_spec enforces data-model requires ERD or class diagram
    #[test]
    fn test_create_spec_data_model_missing_diagram() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create data-model spec WITHOUT ERD or class diagram
        let args = json!({
            "change_id": "test-change",
            "spec_id": "data-model",
            "spec_type": "data-model",
            "title": "User Data Model",
            "overview": "This specification defines the core data structures for user management including profiles, preferences, and relationships.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "User Entity",
                    "description": "User entity definition"
                }
            ],
            "scenarios": [
                {
                    "name": "User creation",
                    "when": "User is created",
                    "then": "Entity is stored"
                }
            ]
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when data-model spec has no ERD/class diagram"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("erd") || err_msg.contains("class"),
            "Error should mention missing erd or class diagram, got: {}",
            err_msg
        );
    }

    // R6: Scenario - create_spec enforces workflow requires Serverless Workflow spec
    #[test]
    fn test_create_spec_workflow_missing_serverless_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create workflow spec WITH state diagram but WITHOUT Serverless Workflow spec
        let args = json!({
            "change_id": "test-change",
            "spec_id": "workflow-spec",
            "spec_type": "workflow",
            "title": "Order Processing Workflow",
            "overview": "This specification defines the workflow for processing customer orders including validation, payment, and fulfillment stages.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "Order Processing",
                    "description": "Process orders"
                }
            ],
            "scenarios": [
                {
                    "name": "Process order",
                    "when": "order is submitted",
                    "then": "workflow executes"
                }
            ],
            "flow_diagram": "stateDiagram-v2\n    [*] --> Pending\n    Pending --> Done\n    Done --> [*]"
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when workflow spec has no Serverless Workflow spec"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("Serverless") || err_msg.contains("workflow-0.8"),
            "Error should mention missing Serverless Workflow spec, got: {}",
            err_msg
        );
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/spec_service_preamble_source.md#source
// CODEGEN-BEGIN
//! Spec service - Business logic for spec creation
//!
//! ## Structured Diagrams
#![allow(deprecated)]
//!
//! The service supports structured diagram definitions that are validated and rendered
//! using the embedded generate library. This ensures:
//! 1. Diagrams are syntactically correct
//! 2. Semantic metadata is preserved for code generation
//! 3. Consistent diagram output across the system

use crate::generate::diagrams::{
    class::{generate_class_diagram, ClassInput},
    erd::{generate_erd, ErdInput},
    flowchart::{generate_flowchart, FlowchartInput},
    journey::{generate_journey, JourneyInput},
    mindmap::{generate_mindmap, MindmapInput},
    requirement::{generate_requirement_diagram, RequirementInput},
    sequence::{generate_sequence, SequenceInput},
    state::{generate_state_diagram, StateInput},
};
use crate::models::spec_rules::{
    apply_section_optionality, ApiSpecType, SectionEntry, SectionType, SpecFormatRules, SpecType,
};
use crate::models::tech_stack::DesignSystem;
use crate::Result;
use chrono::Utc;
use serde_json::Value;
use std::path::Path;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/spec_service.md#schema
// CODEGEN-BEGIN
/// API specification data.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service.md#schema
#[derive(Debug, Clone)]
pub struct ApiSpecData {
    /// Specification type.
    pub spec_type: ApiSpecType,
    /// Full specification object.
    pub spec: Value,
}

/// Input structure for creating a spec.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service.md#schema
#[derive(Debug, Clone)]
pub struct CreateSpecInput {
    /// Change identifier.
    pub change_id: String,
    /// Spec identifier.
    pub spec_id: String,
    /// Spec title.
    pub title: String,
    /// Spec overview text.
    pub overview: String,
    /// Requirements.
    pub requirements: Vec<RequirementData>,
    /// Scenarios.
    pub scenarios: Vec<ScenarioData>,
    /// Spec type classification.
    pub spec_type: SpecType,
    /// Structured diagrams.
    pub diagrams: Vec<DiagramData>,
    /// DEPRECATED raw Mermaid diagram code.
    pub flow_diagram: Option<String>,
    /// Optional data model.
    pub data_model: Option<Value>,
    /// Optional API specification.
    pub api_spec: Option<ApiSpecData>,
    /// Optional agent name for history tracking.
    pub agent: Option<String>,
    /// Optional generation duration.
    pub duration_secs: Option<f64>,
    /// Spec group for organising specs.
    pub spec_group: Option<String>,
    /// Change group ID for group-scoped spec placement.
    pub group_id: Option<String>,
    /// Reference to existing main spec being extended.
    pub main_spec_ref: Option<String>,
    /// Strategy for merging back to main specs.
    pub merge_strategy: Option<String>,
    /// Explicit tags.
    pub tags: Vec<String>,
    /// File changes associated with this spec.
    pub changes: Vec<SpecChangeData>,
    /// Dependencies on other spec IDs.
    pub depends: Vec<String>,
}

/// Structured diagram data.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service.md#schema
#[derive(Debug, Clone)]
pub struct DiagramData {
    /// Diagram type matching generate_mermaid_* tool.
    pub diagram_type: String,
    /// Human-readable title.
    pub title: String,
    /// Input matching the corresponding Mermaid tool schema.
    pub input: Value,
    /// Rendered Mermaid code populated during creation.
    pub rendered: Option<String>,
    /// Extracted semantic data populated during creation.
    pub semantic: Option<Value>,
}

/// A requirement parsed from spec input.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service.md#schema
#[derive(Debug, Clone)]
pub struct RequirementData {
    /// Requirement identifier.
    pub id: String,
    /// Requirement title.
    pub title: String,
    /// Requirement description.
    pub description: String,
    /// Priority label.
    pub priority: String,
}

/// A scenario block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service.md#schema
#[derive(Debug, Clone)]
pub struct ScenarioData {
    /// Scenario name.
    pub name: String,
    /// Optional Given clause.
    pub given: Option<String>,
    /// When clause.
    pub when: String,
    /// Then clause.
    pub then: String,
}

/// File change data for spec creation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service.md#schema
#[derive(Debug, Clone)]
pub struct SpecChangeData {
    /// File path.
    pub file: String,
    /// Action verb (create, modify).
    pub action: String,
    /// Optional context reference.
    pub context_ref: Option<String>,
    /// Optional description.
    pub description: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/spec_service_runtime_source.md#source
// CODEGEN-BEGIN
/// Render a structured diagram using embedded generate library
fn render_diagram(diagram: &DiagramData) -> Result<(String, Option<Value>)> {
    let mermaid_code = match diagram.diagram_type.as_str() {
        "flowchart" => {
            let input: FlowchartInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid flowchart input: {}", e))?;
            generate_flowchart(&input)
                .map_err(|e| anyhow::anyhow!("Flowchart generation failed: {}", e))?
        }
        "sequence" => {
            let input: SequenceInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid sequence input: {}", e))?;
            generate_sequence(&input)
                .map_err(|e| anyhow::anyhow!("Sequence diagram generation failed: {}", e))?
        }
        "class" => {
            let input: ClassInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid class diagram input: {}", e))?;
            generate_class_diagram(&input)
                .map_err(|e| anyhow::anyhow!("Class diagram generation failed: {}", e))?
        }
        "state" => {
            let input: StateInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid state diagram input: {}", e))?;
            generate_state_diagram(&input)
                .map_err(|e| anyhow::anyhow!("State diagram generation failed: {}", e))?
        }
        "erd" => {
            let input: ErdInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid ERD input: {}", e))?;
            generate_erd(&input).map_err(|e| anyhow::anyhow!("ERD generation failed: {}", e))?
        }
        "mindmap" => {
            let input: MindmapInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid mindmap input: {}", e))?;
            generate_mindmap(&input)
                .map_err(|e| anyhow::anyhow!("Mindmap generation failed: {}", e))?
        }
        "requirement" => {
            let input: RequirementInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid requirement diagram input: {}", e))?;
            generate_requirement_diagram(&input)
                .map_err(|e| anyhow::anyhow!("Requirement diagram generation failed: {}", e))?
        }
        "journey" => {
            let input: JourneyInput = serde_json::from_value(diagram.input.clone())
                .map_err(|e| anyhow::anyhow!("Invalid journey diagram input: {}", e))?;
            generate_journey(&input)
                .map_err(|e| anyhow::anyhow!("Journey diagram generation failed: {}", e))?
        }
        _ => anyhow::bail!("Unknown diagram type: {}", diagram.diagram_type),
    };

    // Extract semantic data from flowchart input if present
    let semantic = if diagram.diagram_type == "flowchart" {
        diagram.input.get("metadata").cloned()
    } else {
        None
    };

    Ok((mermaid_code, semantic))
}

/// Validate an OpenAPI 3.1 specification
fn validate_openapi_spec(spec: &Value) -> Result<()> {
    // Check for required OpenAPI fields
    let openapi_version = spec
        .get("openapi")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("OpenAPI spec missing 'openapi' version field"))?;

    if !openapi_version.starts_with("3.1") {
        anyhow::bail!("Expected OpenAPI 3.1.x, got version '{}'", openapi_version);
    }

    // Verify required info field
    if spec.get("info").is_none() {
        anyhow::bail!("OpenAPI spec missing required 'info' field");
    }

    // Verify paths or webhooks exist (at least one required in OpenAPI 3.1)
    let has_paths = spec.get("paths").map_or(false, |p| !p.is_null());
    let has_webhooks = spec.get("webhooks").map_or(false, |w| !w.is_null());
    let has_components = spec.get("components").map_or(false, |c| !c.is_null());

    if !has_paths && !has_webhooks && !has_components {
        anyhow::bail!("OpenAPI spec must have at least 'paths', 'webhooks', or 'components'");
    }

    Ok(())
}

/// Validate an AsyncAPI 2.6 specification
fn validate_asyncapi_spec(spec: &Value) -> Result<()> {
    // Check for required AsyncAPI fields
    let asyncapi_version = spec
        .get("asyncapi")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("AsyncAPI spec missing 'asyncapi' version field"))?;

    if !asyncapi_version.starts_with("2.6") {
        anyhow::bail!(
            "Expected AsyncAPI 2.6.x, got version '{}'",
            asyncapi_version
        );
    }

    // Verify required info field
    if spec.get("info").is_none() {
        anyhow::bail!("AsyncAPI spec missing required 'info' field");
    }

    // Verify channels exist (required in AsyncAPI)
    if spec.get("channels").is_none() {
        anyhow::bail!("AsyncAPI spec missing required 'channels' field");
    }

    Ok(())
}

/// Validate an OpenRPC 1.3 specification
fn validate_openrpc_spec(spec: &Value) -> Result<()> {
    let version = spec
        .get("openrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("OpenRPC spec missing 'openrpc' version field"))?;

    if !version.starts_with("1.3") {
        anyhow::bail!("Expected OpenRPC 1.3.x, got '{}'", version);
    }

    if spec.get("info").is_none() {
        anyhow::bail!("OpenRPC spec missing 'info' field");
    }
    if spec.get("methods").is_none() {
        anyhow::bail!("OpenRPC spec missing 'methods' field");
    }
    Ok(())
}

/// Validate a Serverless Workflow 0.8 specification
fn validate_serverless_workflow_spec(spec: &Value) -> Result<()> {
    let version = spec
        .get("specVersion")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Serverless Workflow spec missing 'specVersion' field"))?;

    if !version.starts_with("0.8") {
        anyhow::bail!("Expected Serverless Workflow 0.8.x, got '{}'", version);
    }

    if spec.get("id").is_none() {
        anyhow::bail!("Serverless Workflow spec missing 'id' field");
    }
    if spec.get("start").is_none() {
        anyhow::bail!("Serverless Workflow spec missing 'start' field");
    }
    if spec.get("states").is_none() {
        anyhow::bail!("Serverless Workflow spec missing 'states' field");
    }
    Ok(())
}

/// Validate a JSON Schema specification
fn validate_json_schema(spec: &Value) -> Result<()> {
    // JSON Schema is very flexible. Minimal validation:
    // Must be a JSON object (Schema can be a boolean or object)
    match spec {
        Value::Object(_) | Value::Bool(_) => Ok(()),
        _ => anyhow::bail!("JSON Schema must be an object or boolean"),
    }
}

/// Validate API specification based on type
fn validate_api_spec(api_spec: &ApiSpecData) -> Result<()> {
    match api_spec.spec_type {
        ApiSpecType::OpenApi31 => validate_openapi_spec(&api_spec.spec),
        ApiSpecType::AsyncApi26 => validate_asyncapi_spec(&api_spec.spec),
        ApiSpecType::OpenRpc13 => validate_openrpc_spec(&api_spec.spec),
        ApiSpecType::ServerlessWorkflow08 => validate_serverless_workflow_spec(&api_spec.spec),
        ApiSpecType::JsonSchema => validate_json_schema(&api_spec.spec),
    }
}

/// Validate spec_type requirements are met
/// Per-tag diagram requirements. Tags with OR alternatives return multiple options.
fn tag_required_diagrams(tag: &str) -> Vec<Vec<&'static str>> {
    match tag {
        "api" => vec![],
        "http" => vec![vec!["sequence"]],
        "rpc" => vec![vec!["class"]],
        "events" => vec![vec!["sequence"]],
        "async" => vec![],
        "data" => vec![vec!["erd", "class"]], // erd OR class
        "logic" => vec![vec!["flowchart", "state"]], // flowchart OR state
        "state" => vec![vec!["state", "flowchart"]], // state OR flowchart
        "external" => vec![vec!["sequence"]],
        _ => vec![],
    }
}

/// Per-tag API spec requirements.
fn tag_required_api_spec(tag: &str) -> Option<ApiSpecType> {
    match tag {
        "http" => Some(ApiSpecType::OpenApi31),
        "rpc" => Some(ApiSpecType::OpenRpc13),
        "events" => Some(ApiSpecType::AsyncApi26),
        "data" => Some(ApiSpecType::JsonSchema),
        "state" => Some(ApiSpecType::ServerlessWorkflow08),
        _ => None,
    }
}

fn validate_spec_type_requirements(input: &CreateSpecInput) -> Result<()> {
    let resolved_tags = resolve_tags(&input.spec_type, &input.tags);

    // Collect provided diagram types from structured diagrams
    let mut diagram_types: Vec<&str> = input
        .diagrams
        .iter()
        .map(|d| d.diagram_type.as_str())
        .collect();

    // Also detect diagram type from legacy flow_diagram field
    if let Some(ref fd) = input.flow_diagram {
        let trimmed = fd.trim_start();
        if trimmed.starts_with("sequenceDiagram") {
            diagram_types.push("sequence");
        } else if trimmed.starts_with("classDiagram") {
            diagram_types.push("class");
        } else if trimmed.starts_with("erDiagram") {
            diagram_types.push("erd");
        } else if trimmed.starts_with("stateDiagram") {
            diagram_types.push("state");
        } else if trimmed.starts_with("flowchart") || trimmed.starts_with("graph") {
            diagram_types.push("flowchart");
        }
    }

    // For each tag, check required diagrams
    for tag in &resolved_tags {
        let requirements = tag_required_diagrams(tag);
        for alternatives in &requirements {
            if alternatives.is_empty() {
                continue;
            }
            let has_any = alternatives.iter().any(|req| diagram_types.contains(req));
            if !has_any {
                let alt_str = alternatives.join(" or ");
                anyhow::bail!("Tag '{}' requires diagram(s): {}", tag, alt_str);
            }
        }
    }

    // For each tag, check required API spec
    for tag in &resolved_tags {
        if let Some(required_api_type) = tag_required_api_spec(tag) {
            match &input.api_spec {
                Some(api_spec) if api_spec.spec_type == required_api_type => {}
                Some(api_spec) => {
                    anyhow::bail!(
                        "Tag '{}' requires {} API spec, but got {}",
                        tag,
                        required_api_type.as_str(),
                        api_spec.spec_type.as_str()
                    );
                }
                None => {
                    anyhow::bail!(
                        "Tag '{}' requires {} API spec",
                        tag,
                        required_api_type.as_str()
                    );
                }
            }
        }
    }

    Ok(())
}

/// Create a new spec with validation
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service_runtime_source.md#source
pub fn create_spec(input: CreateSpecInput, project_root: &Path) -> Result<String> {
    // Validate spec_id format
    if !input
        .spec_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("spec_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate spec_group format if provided
    if let Some(ref spec_group) = input.spec_group {
        // Must start with lowercase letter, then lowercase alphanumeric with hyphens
        let valid = !spec_group.is_empty()
            && spec_group
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_lowercase())
            && spec_group
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !valid {
            anyhow::bail!(
                "spec_group must start with lowercase letter and contain only lowercase alphanumeric with hyphens"
            );
        }
    }

    // Validate overview length
    if input.overview.len() < 50 {
        anyhow::bail!("overview must be at least 50 characters");
    }

    // Validate requirements
    if input.requirements.is_empty() {
        anyhow::bail!("At least one requirement is required");
    }

    // Validate scenarios
    if input.scenarios.is_empty() {
        anyhow::bail!("At least one scenario is required");
    }

    // Validate merge_strategy if provided
    if let Some(ref strategy) = input.merge_strategy {
        let valid_strategies = ["new", "extend", "replace", "patch"];
        if !valid_strategies.contains(&strategy.as_str()) {
            anyhow::bail!(
                "Invalid merge_strategy '{}'. Valid values: new, extend, replace, patch",
                strategy
            );
        }
    }

    // Check change directory exists
    let change_dir = project_root.join(".aw/changes").join(&input.change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Create proposal first.",
            input.change_id
        );
    }

    // Create specs directory (group-aware, backward compatible).
    // Priority: group_id (groups/{gid}/specs/) > spec_group (specs/{sg}/) > root (specs/)
    let specs_dir = if let Some(ref gid) = input.group_id {
        // New layout: multi-group change → groups/{group_id}/specs/
        change_dir.join("groups").join(gid).join("specs")
    } else if let Some(ref spec_group) = input.spec_group {
        // Legacy layout: single-group subdirectory → specs/{spec_group}/
        change_dir.join("specs").join(spec_group)
    } else {
        // Default: root specs directory
        change_dir.join("specs")
    };
    std::fs::create_dir_all(&specs_dir)?;

    // Render structured diagrams
    let mut rendered_diagrams: Vec<(String, String, Option<Value>)> = Vec::new();
    for diagram in &input.diagrams {
        let (mermaid_code, semantic) = render_diagram(diagram)
            .map_err(|e| anyhow::anyhow!("Failed to render diagram '{}': {}", diagram.title, e))?;
        rendered_diagrams.push((diagram.title.clone(), mermaid_code, semantic));
    }

    // Validate API spec if provided
    if let Some(ref api_spec) = input.api_spec {
        validate_api_spec(api_spec)
            .map_err(|e| anyhow::anyhow!("API spec validation failed: {}", e))?;
    }

    // Validate spec_type requirements
    validate_spec_type_requirements(&input)?;

    // Generate spec content
    let now = Utc::now();
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("id: {}\n", input.spec_id));
    content.push_str("type: spec\n");
    content.push_str(&format!(
        "title: \"{}\"\n",
        input.title.replace('"', "\\\"")
    ));
    content.push_str("version: 1\n");
    content.push_str(&format!("spec_type: {}\n", input.spec_type.as_str()));

    // Resolve tags: auto-tags from spec_type + explicit tags
    let resolved_tags = resolve_tags(&input.spec_type, &input.tags);
    if !resolved_tags.is_empty() {
        content.push_str(&format!("tags: [{}]\n", resolved_tags.join(", ")));
    }

    // Include spec_group in frontmatter if provided
    if let Some(ref spec_group) = input.spec_group {
        content.push_str(&format!("spec_group: {}\n", spec_group));
    }
    // Include main_spec_ref for traceability during merge
    if let Some(ref main_spec_ref) = input.main_spec_ref {
        content.push_str(&format!("main_spec_ref: {}\n", main_spec_ref));
    }
    // Include merge_strategy to specify how this spec should be merged
    if let Some(ref merge_strategy) = input.merge_strategy {
        content.push_str(&format!("merge_strategy: {}\n", merge_strategy));
    }
    content.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));

    // Requirements summary
    let requirement_ids: Vec<String> = input.requirements.iter().map(|r| r.id.clone()).collect();

    content.push_str("requirements:\n");
    content.push_str(&format!("  total: {}\n", input.requirements.len()));
    if !requirement_ids.is_empty() {
        content.push_str("  ids:\n");
        for id in &requirement_ids {
            content.push_str(&format!("    - {}\n", id));
        }
    }

    // Design elements - track both legacy and structured diagrams
    let has_mermaid = input.flow_diagram.is_some() || !input.diagrams.is_empty();
    let has_semantic = rendered_diagrams.iter().any(|(_, _, sem)| sem.is_some());

    content.push_str("design_elements:\n");
    content.push_str(&format!("  has_mermaid: {}\n", has_mermaid));
    content.push_str(&format!(
        "  has_json_schema: {}\n",
        input.data_model.is_some()
    ));
    content.push_str("  has_pseudo_code: false\n");
    content.push_str(&format!("  has_api_spec: {}\n", input.api_spec.is_some()));
    content.push_str(&format!("  has_semantic_diagrams: {}\n", has_semantic));

    // Track API spec type in frontmatter
    if let Some(ref api_spec) = input.api_spec {
        content.push_str(&format!(
            "  api_spec_type: {}\n",
            api_spec.spec_type.as_str()
        ));
    }

    // Track structured diagrams in frontmatter
    if !input.diagrams.is_empty() {
        content.push_str("  diagrams:\n");
        for diagram in &input.diagrams {
            content.push_str(&format!("    - type: {}\n", diagram.diagram_type));
            content.push_str(&format!(
                "      title: \"{}\"\n",
                diagram.title.replace('"', "\\\"")
            ));
        }
    }

    // Dependencies
    if !input.depends.is_empty() {
        content.push_str("depends:\n");
        for dep in &input.depends {
            content.push_str(&format!("  - {}\n", dep));
        }
    }

    // File changes
    if !input.changes.is_empty() {
        content.push_str("changes:\n");
        for ch in &input.changes {
            content.push_str(&format!("  - file: {}\n", ch.file));
            content.push_str(&format!("    action: {}\n", ch.action));
            if let Some(ref ctx) = ch.context_ref {
                content.push_str(&format!("    context_ref: \"{}\"\n", ctx));
            }
            if let Some(ref desc) = ch.description {
                content.push_str(&format!(
                    "    description: \"{}\"\n",
                    desc.replace('"', "\\\"")
                ));
            }
        }
    }

    // History entry
    content.push_str("history:\n");
    let agent_name = input.agent.as_deref().unwrap_or("mcp");
    content.push_str(&format!(
        "  - timestamp: {}\n    agent: \"{}\"\n    tool: \"create_spec\"\n    action: \"created\"",
        now.to_rfc3339(),
        agent_name
    ));
    if let Some(secs) = input.duration_secs {
        content.push_str(&format!("\n    duration_secs: {:.2}", secs));
    }
    content.push('\n');

    content.push_str("---\n\n");

    // Wrap spec content in XML
    content.push_str("<spec>\n\n");

    // Title
    content.push_str(&format!("# {}\n\n", input.title));

    // Overview
    content.push_str("## Overview\n\n");
    content.push_str(&format!("{}\n\n", input.overview));

    // Requirements section
    content.push_str("## Requirements\n\n");

    for req in &input.requirements {
        content.push_str(&format!("### {} - {}\n\n", req.id, req.title));
        content.push_str("```yaml\n");
        content.push_str(&format!("id: {}\n", req.id));
        content.push_str(&format!("priority: {}\n", req.priority));
        content.push_str("status: draft\n");
        content.push_str("```\n\n");
        content.push_str(&format!("{}\n\n", req.description));
    }

    // Acceptance Criteria section - use central format rules
    let spec_rules = SpecFormatRules::spec_defaults();

    // Find the "Acceptance Criteria" heading from required_headings
    let ac_heading = spec_rules
        .required_headings
        .iter()
        .find(|h| h.contains("Acceptance") || h.contains("Criteria"))
        .map(|s| s.as_str())
        .unwrap_or("Acceptance Criteria");

    content.push_str(&format!("## {}\n\n", ac_heading));

    for scenario in &input.scenarios {
        // Use scenario heading format from rules: ### {prefix} {name}
        let heading_hashes = "#".repeat(spec_rules.scenario_heading_level as usize);
        content.push_str(&format!(
            "{} {} {}\n\n",
            heading_hashes, spec_rules.scenario_heading_prefix, scenario.name
        ));

        // Use WHEN/THEN keywords from rules
        if let Some(given_text) = &scenario.given {
            content.push_str(&format!("- **GIVEN** {}\n", given_text));
        }
        content.push_str(&format!(
            "- **{}** {}\n",
            spec_rules.when_keyword, scenario.when
        ));
        content.push_str(&format!(
            "- **{}** {}\n\n",
            spec_rules.then_keyword, scenario.then
        ));
    }

    // Structured diagrams (preferred)
    if !rendered_diagrams.is_empty() {
        content.push_str("## Diagrams\n\n");
        for (title, mermaid_code, semantic) in &rendered_diagrams {
            content.push_str(&format!("### {}\n\n", title));
            content.push_str("```mermaid\n");
            content.push_str(mermaid_code);
            if !mermaid_code.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("```\n\n");

            // Include semantic data as structured comment for code generation
            if let Some(sem_data) = semantic {
                content.push_str("<semantic-data>\n\n");
                content.push_str("```json\n");
                content.push_str(&serde_json::to_string_pretty(sem_data)?);
                content.push_str("\n```\n\n");
                content.push_str("</semantic-data>\n\n");
            }
        }
    }

    // Legacy flow diagram (deprecated, for backward compatibility)
    if let Some(diagram) = &input.flow_diagram {
        if input.diagrams.is_empty() {
            // Only render if no structured diagrams provided
            content.push_str("## Flow Diagram\n\n");
            content.push_str("```mermaid\n");
            content.push_str(diagram);
            if !diagram.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("```\n\n");
        }
    }

    // Data model (optional)
    if let Some(model) = &input.data_model {
        content.push_str("## Data Model\n\n");
        content.push_str("```json\n");
        content.push_str(&serde_json::to_string_pretty(model)?);
        content.push_str("\n```\n\n");
    }

    // API Specification (OpenAPI, AsyncAPI, OpenRPC, or Serverless Workflow)
    if let Some(ref api_spec) = input.api_spec {
        let spec_type_label = match api_spec.spec_type {
            ApiSpecType::OpenApi31 => "OpenAPI 3.1",
            ApiSpecType::AsyncApi26 => "AsyncAPI 2.6",
            ApiSpecType::OpenRpc13 => "OpenRPC 1.3",
            ApiSpecType::ServerlessWorkflow08 => "Serverless Workflow 0.8",
            ApiSpecType::JsonSchema => "JSON Schema",
        };
        content.push_str(&format!("## API Specification ({})\n\n", spec_type_label));
        content.push_str("```yaml\n");
        // Convert JSON to YAML for better readability
        let yaml_str = serde_yaml::to_string(&api_spec.spec)?;
        content.push_str(&yaml_str);
        content.push_str("```\n\n");
    }

    // Close spec XML tag
    content.push_str("</spec>\n");

    // Write the file
    let spec_path = specs_dir.join(format!("{}.md", input.spec_id));
    std::fs::write(&spec_path, &content)?;

    // Generate YAML IR manifests (R1 from genesis-spec-generation spec)
    let ir_result = generate_spec_ir(&input, &change_dir, &spec_path);
    let ir_msg = match ir_result {
        Ok(result) if !result.files.is_empty() => {
            format!(" + {} YAML IR file(s)", result.files.len())
        }
        Ok(_) => String::new(),
        Err(e) => format!(" (YAML IR generation warning: {})", e),
    };

    Ok(format!(
        "Created spec '{}' for change '{}' at {}{}",
        input.spec_id,
        input.change_id,
        spec_path.display(),
        ir_msg
    ))
}

/// Generate YAML IR manifests from a CreateSpecInput.
///
/// Maps structured diagrams and API specs into SpecIR manifests
/// and writes them to `<change_dir>/spec_ir/`.
fn generate_spec_ir(
    input: &CreateSpecInput,
    change_dir: &Path,
    spec_path: &Path,
) -> Result<crate::spec_ir::generator::GenerateResult> {
    use crate::spec_ir::generator::{ApiSpecEntry, DiagramEntry, SpecIrInput};

    let source_file = spec_path
        .strip_prefix(change_dir.parent().unwrap_or(Path::new(".")))
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let diagrams: Vec<DiagramEntry> = input
        .diagrams
        .iter()
        .map(|d| {
            let content = if let Some(ref rendered) = d.rendered {
                serde_yaml::Value::String(rendered.clone())
            } else {
                // Convert JSON input to YAML value
                serde_yaml::to_value(&d.input).unwrap_or(serde_yaml::Value::Null)
            };
            DiagramEntry {
                diagram_type: d.diagram_type.clone(),
                title: d.title.clone(),
                content,
            }
        })
        .collect();

    let api_spec = input.api_spec.as_ref().map(|a| {
        let api_type = match a.spec_type {
            ApiSpecType::OpenApi31 => "openapi-3.1",
            ApiSpecType::AsyncApi26 => "asyncapi-2.6",
            ApiSpecType::OpenRpc13 => "openrpc-1.3",
            ApiSpecType::ServerlessWorkflow08 => "serverless-workflow-0.8",
            ApiSpecType::JsonSchema => "json-schema",
        };
        ApiSpecEntry {
            api_type: api_type.to_string(),
            content: serde_yaml::to_value(&a.spec).unwrap_or(serde_yaml::Value::Null),
        }
    });

    let ir_input = SpecIrInput {
        spec_id: input.spec_id.clone(),
        change_id: input.change_id.clone(),
        spec_group: input.spec_group.clone(),
        source_file,
        tags: input.tags.clone(),
        diagrams,
        api_spec,
    };

    crate::spec_ir::generator::generate(change_dir, &ir_input)
}

// ─── Section Selection Rule Engine ───────────────────────────────────────────

/// Keyword rules for section selection.
///
/// Each rule maps a regex pattern (applied to requirements text) to
/// suggested section types.
struct SectionRule {
    pattern: &'static str,
    sections: &'static [SectionType],
}

/// Keyword-based section rules. Matched against requirements text to suggest sections.
const SECTION_RULES: &[SectionRule] = &[
    SectionRule {
        pattern: "endpoint|route|api|REST|HTTP",
        sections: &[SectionType::RestApi, SectionType::Schema],
    },
    SectionRule {
        pattern: "rpc|json-rpc|MCP tool",
        sections: &[SectionType::RpcApi, SectionType::Schema],
    },
    SectionRule {
        pattern: "queue|pubsub|webhook|background|async",
        sections: &[SectionType::AsyncApi],
    },
    SectionRule {
        pattern: "database|model|table|migration|collection",
        sections: &[SectionType::DbModel],
    },
    SectionRule {
        pattern: "state|phase|lifecycle|transition",
        sections: &[SectionType::StateMachine],
    },
    SectionRule {
        pattern: "UI|page|component|layout|frontend",
        sections: &[SectionType::Wireframe, SectionType::Component],
    },
    SectionRule {
        pattern: "CLI|command|subcommand|flag",
        sections: &[SectionType::Cli],
    },
    SectionRule {
        pattern: r"config|env|settings|\.toml|\.env",
        sections: &[SectionType::Config],
    },
    SectionRule {
        pattern: "token|color|spacing|typography|theme",
        sections: &[SectionType::DesignToken],
    },
];

/// Resolve section types for a spec based on requirements text and tech stack.
///
/// Process:
/// 1. Always include `overview`
/// 2. Match requirements text against keyword rules
/// 3. Add conditional sections based on count thresholds
/// 4. Apply optionality filter based on `design_system` capabilities
/// 5. Sort by fill order
///
/// Returns `Vec<SectionEntry>` where each entry may be required or optional.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/spec_service_runtime_source.md#source
pub fn resolve_section_rules(
    requirements_text: &str,
    design_system: Option<&DesignSystem>,
) -> Vec<SectionEntry> {
    let mut matched: Vec<SectionType> = Vec::new();

    // Always-required sections
    matched.push(SectionType::Overview);

    // Keyword rule matching
    for rule in SECTION_RULES {
        let re = match regex::Regex::new(&format!("(?i){}", rule.pattern)) {
            Ok(re) => re,
            Err(_) => continue,
        };
        if re.is_match(requirements_text) {
            for &section in rule.sections {
                if !matched.contains(&section) {
                    matched.push(section);
                }
            }
        }
    }

    // Conditional sections based on count thresholds
    let keyword_count = matched.len();
    if keyword_count > 2 && !matched.contains(&SectionType::UnitTest) {
        matched.push(SectionType::UnitTest);
    }
    if keyword_count > 3 {
        if !matched.contains(&SectionType::Interaction) {
            matched.push(SectionType::Interaction);
        }
        if !matched.contains(&SectionType::Logic) {
            matched.push(SectionType::Logic);
        }
        if !matched.contains(&SectionType::Dependency) {
            matched.push(SectionType::Dependency);
        }
    }

    // Sort by fill order before applying optionality
    matched.sort_by_key(|st| st.fill_order());

    // Apply optionality filter based on design system
    apply_section_optionality(matched, design_system)
}

/// Resolve tags by combining auto-tags from spec_type with explicit tags.
///
/// Auto-tag mapping:
/// - http-api → [api, http]
/// - event-driven → [api, events, async]
/// - data-model → [data]
/// - algorithm → [logic]
/// - integration → [external]
/// - utility → []
/// - rpc-api → [api, rpc]
/// - workflow → [state, logic]
fn resolve_tags(spec_type: &SpecType, explicit_tags: &[String]) -> Vec<String> {
    let auto_tags: &[&str] = match spec_type {
        SpecType::HttpApi => &["api", "http"],
        SpecType::EventDriven => &["events", "async"],
        SpecType::DataModel => &["data"],
        SpecType::Algorithm => &["logic"],
        SpecType::Integration => &["external"],
        SpecType::Utility => &[],
        SpecType::RpcApi => &["api", "rpc"],
        SpecType::Workflow => &["state", "logic"],
    };

    let mut tags: Vec<String> = auto_tags.iter().map(|s| s.to_string()).collect();
    for tag in explicit_tags {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    tags
}

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

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "mcp-protocol".to_string(),
            title: "MCP Protocol Implementation".to_string(),
            overview: "This specification covers the implementation of the Model Context Protocol (MCP) server for genesis, providing structured tools for proposal generation.".to_string(),
            requirements: vec![
                RequirementData {
                    id: "R1".to_string(),
                    title: "JSON-RPC 2.0 Support".to_string(),
                    description: "The server must support JSON-RPC 2.0 protocol over stdio".to_string(),
                    priority: "high".to_string(),
                },
                RequirementData {
                    id: "R2".to_string(),
                    title: "Tool Registration".to_string(),
                    description: "Tools must be registered and callable via tools/call method".to_string(),
                    priority: "high".to_string(),
                },
            ],
            scenarios: vec![
                ScenarioData {
                    name: "Server Initialization".to_string(),
                    given: Some("MCP client is connected".to_string()),
                    when: "Client sends initialize request".to_string(),
                    then: "Server responds with capabilities".to_string(),
                },
                ScenarioData {
                    name: "Tool Execution".to_string(),
                    given: None,
                    when: "Client calls create_proposal tool".to_string(),
                    then: "Server creates proposal.md and returns success".to_string(),
                },
            ],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: Some("graph LR\n    A[Client] --> B[Server]\n    B --> C[Tool Registry]\n    C --> D[Execute Tool]".to_string()),
            data_model: None,
            api_spec: None,
            agent: Some("gemini-test".to_string()),
            duration_secs: Some(8.3),
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root).unwrap();
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
        // Verify history entry
        assert!(content.contains("history:"));
        assert!(content.contains("agent: \"gemini-test\""));
        assert!(content.contains("tool: \"create_spec\""));
        assert!(content.contains("action: \"created\""));
        assert!(content.contains("duration_secs: 8.30"));
    }

    #[test]
    fn test_create_spec_validation() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Test invalid spec_id
        let input = CreateSpecInput {
            change_id: "test".to_string(),
            spec_id: "Invalid_ID".to_string(),
            title: "Test".to_string(),
            overview: "Test overview that is long enough to pass validation requirements."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_spec_with_structured_diagrams() {
        use serde_json::json;

        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "api-flow".to_string(),
            title: "API Flow Specification".to_string(),
            overview: "This specification covers the API flow with structured diagrams for code generation support.".to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Create User Endpoint".to_string(),
                description: "POST /users endpoint for user creation".to_string(),
                priority: "high".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Successful User Creation".to_string(),
                given: Some("Valid user data provided".to_string()),
                when: "POST /users is called".to_string(),
                then: "User is created and returned".to_string(),
            }],
            diagrams: vec![DiagramData {
                diagram_type: "flowchart".to_string(),
                title: "Create User Flow".to_string(),
                input: json!({
                    "direction": "TB",
                    "nodes": [
                        {
                            "id": "start",
                            "label": "POST /users",
                            "shape": "rounded",
                            "semantic": {"type": "start"}
                        },
                        {
                            "id": "validate",
                            "label": "Validate Email",
                            "shape": "diamond",
                            "semantic": {
                                "type": "validation",
                                "error": {"code": 400, "message": "Invalid email"}
                            }
                        },
                        {
                            "id": "end",
                            "label": "Return User",
                            "shape": "rounded",
                            "semantic": {"type": "return"}
                        }
                    ],
                    "edges": [
                        {"from": "start", "to": "validate"},
                        {"from": "validate", "to": "end", "label": "valid"}
                    ],
                    "metadata": {
                        "endpoint": {
                            "method": "POST",
                            "path": "/users"
                        }
                    }
                }),
                rendered: None,
                semantic: None,
            }],
            spec_type: SpecType::Algorithm,
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: Some("test-agent".to_string()),
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created
        let spec_path = project_root.join(".aw/changes/test-change/specs/api-flow.md");
        assert!(spec_path.exists());

        let content = std::fs::read_to_string(&spec_path).unwrap();

        // Verify frontmatter contains diagram info
        assert!(content.contains("has_semantic_diagrams: true"));
        assert!(content.contains("diagrams:"));
        assert!(content.contains("type: flowchart"));
        assert!(content.contains("title: \"Create User Flow\""));

        // Verify diagram section
        assert!(content.contains("## Diagrams"));
        assert!(content.contains("### Create User Flow"));
        assert!(content.contains("flowchart TB"));
        assert!(content.contains("start(POST /users)"));
        assert!(content.contains("validate{Validate Email}"));

        // Verify semantic data is included
        assert!(content.contains("<semantic-data>"));
        assert!(content.contains("</semantic-data>"));
    }

    #[test]
    fn test_create_spec_with_openapi() {
        use serde_json::json;

        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "user-api".to_string(),
            title: "User API Specification".to_string(),
            overview: "This specification covers the User API with OpenAPI 3.1 specification for code generation.".to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Create User Endpoint".to_string(),
                description: "POST /users endpoint for user creation".to_string(),
                priority: "high".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Successful User Creation".to_string(),
                given: Some("Valid user data provided".to_string()),
                when: "POST /users is called".to_string(),
                then: "User is created and returned with 201 status".to_string(),
            }],
            spec_type: SpecType::Utility, // Using utility to skip validation for this test
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: Some(ApiSpecData {
                spec_type: ApiSpecType::OpenApi31,
                spec: json!({
                    "openapi": "3.1.0",
                    "info": {
                        "title": "User API",
                        "version": "1.0.0"
                    },
                    "paths": {
                        "/users": {
                            "post": {
                                "summary": "Create a new user",
                                "requestBody": {
                                    "content": {
                                        "application/json": {
                                            "schema": {
                                                "$ref": "#/components/schemas/UserCreate"
                                            }
                                        }
                                    }
                                },
                                "responses": {
                                    "201": {
                                        "description": "User created successfully"
                                    }
                                }
                            }
                        }
                    },
                    "components": {
                        "schemas": {
                            "UserCreate": {
                                "type": "object",
                                "properties": {
                                    "email": {"type": "string", "format": "email"},
                                    "name": {"type": "string"}
                                },
                                "required": ["email", "name"]
                            }
                        }
                    }
                }),
            }),
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created
        let spec_path = project_root.join(".aw/changes/test-change/specs/user-api.md");
        assert!(spec_path.exists());

        let content = std::fs::read_to_string(&spec_path).unwrap();

        // Verify API spec in frontmatter
        assert!(content.contains("has_api_spec: true"));
        assert!(content.contains("api_spec_type: openapi-3.1"));

        // Verify API spec section
        assert!(content.contains("## API Specification (OpenAPI 3.1)"));
        assert!(content.contains("openapi: 3.1.0"));
        assert!(content.contains("/users:"));
    }

    #[test]
    fn test_openapi_validation_fails_without_version() {
        use serde_json::json;

        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "bad-api".to_string(),
            title: "Bad API Specification".to_string(),
            overview: "This specification has an invalid OpenAPI spec without the version field."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::Utility, // Using utility to skip validation for this test
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: Some(ApiSpecData {
                spec_type: ApiSpecType::OpenApi31,
                spec: json!({
                    // Missing "openapi" field!
                    "info": {
                        "title": "Bad API",
                        "version": "1.0.0"
                    },
                    "paths": {}
                }),
            }),
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing 'openapi'"));
    }

    #[test]
    fn test_spec_type_http_api_requires_sequence_diagram() {
        // Test that http-api without sequence diagram fails validation
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "no-diagram".to_string(),
            title: "Missing Diagram Specification".to_string(),
            overview: "This specification is http-api type but missing required sequence diagram."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::HttpApi,
            diagrams: vec![],
            flow_diagram: None, // No sequence diagram
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires diagram(s): sequence"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_spec_type_http_api_requires_openapi() {
        // Test that http-api with sequence diagram but without OpenAPI fails validation
        // Note: We use flow_diagram (legacy field) to provide sequence diagram without
        // Mermaid tool dependency during unit tests
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "incomplete-api".to_string(),
            title: "Incomplete API Specification".to_string(),
            overview: "This specification is http-api type but missing required OpenAPI spec."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::HttpApi,
            diagrams: vec![],
            // Use flow_diagram (legacy) to provide sequence diagram without Mermaid tool
            flow_diagram: Some(
                "sequenceDiagram\n    Client->>Server: Request\n    Server-->>Client: Response"
                    .to_string(),
            ),
            data_model: None,
            api_spec: None, // Missing OpenAPI spec!
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires openapi-3.1 API spec"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_spec_type_data_model_requires_erd() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "incomplete-model".to_string(),
            title: "Incomplete Data Model".to_string(),
            overview: "This specification is data-model type but missing required ERD diagram."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::DataModel, // data-model requires erd or class
            diagrams: vec![],               // Missing ERD!
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires diagram(s): erd or class"));
    }

    #[test]
    fn test_spec_type_rpc_api_requires_class_diagram() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "rpc-no-diagram".to_string(),
            title: "RPC API Missing Diagram".to_string(),
            overview: "This specification is rpc-api type but missing required class diagram."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::RpcApi,
            diagrams: vec![],
            flow_diagram: None, // Missing class diagram
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires diagram(s): class"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_spec_type_rpc_api_requires_openrpc() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "rpc-no-openrpc".to_string(),
            title: "RPC API Missing OpenRPC".to_string(),
            overview: "This specification is rpc-api type but missing required OpenRPC spec."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::RpcApi,
            diagrams: vec![],
            flow_diagram: Some(
                "classDiagram\n    class Calculator {\n        +add(a, b) int\n    }".to_string(),
            ),
            data_model: None,
            api_spec: None, // Missing OpenRPC spec
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires openrpc-1.3 API spec"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_spec_type_workflow_requires_state_diagram() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "workflow-no-diagram".to_string(),
            title: "Workflow Missing Diagram".to_string(),
            overview: "This specification is workflow type but missing required state diagram."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::Workflow,
            diagrams: vec![],
            flow_diagram: None, // Missing state/flowchart diagram
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires diagram(s): state or flowchart"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_spec_type_workflow_requires_serverless_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "workflow-no-spec".to_string(),
            title: "Workflow Missing Serverless Workflow".to_string(),
            overview:
                "This specification is workflow type but missing required Serverless Workflow spec."
                    .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::Workflow,
            diagrams: vec![],
            flow_diagram: Some(
                "stateDiagram-v2\n    [*] --> Processing\n    Processing --> [*]".to_string(),
            ),
            data_model: None,
            api_spec: None, // Missing Serverless Workflow spec
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires serverless-workflow-0.8 API spec"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_create_spec_with_spec_group() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "workflow-spec".to_string(),
            title: "Workflow Enhancement".to_string(),
            overview:
                "This specification covers workflow enhancements for the sdd crate specifically."
                    .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Feature A".to_string(),
                description: "Implement feature A".to_string(),
                priority: "high".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Feature A works".to_string(),
                given: None,
                when: "user triggers feature A".to_string(),
                then: "feature A completes successfully".to_string(),
            }],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: Some("sdd".to_string()),
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created in spec_group subdirectory
        let spec_path = project_root.join(".aw/changes/test-change/specs/sdd/workflow-spec.md");
        assert!(
            spec_path.exists(),
            "Spec should be created in sdd subdirectory"
        );

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: workflow-spec"));
        assert!(content.contains("spec_group: sdd"));
    }

    #[test]
    fn test_create_spec_spec_group_validation() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Test invalid spec_group format (starts with number)
        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "test-spec".to_string(),
            title: "Test Spec".to_string(),
            overview:
                "This specification tests validation of spec_group format with invalid values."
                    .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: Some("123-invalid".to_string()),
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("spec_group must start with lowercase letter"),
            "Error message was: {}",
            err_msg
        );
    }

    #[test]
    fn test_create_spec_without_spec_group() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "cross-cutting-spec".to_string(),
            title: "Cross-cutting Spec".to_string(),
            overview: "This specification is cross-cutting and doesn't belong to a specific crate."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Shared Feature".to_string(),
                description: "Shared feature".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Shared feature works".to_string(),
                given: None,
                when: "shared feature used".to_string(),
                then: "works correctly".to_string(),
            }],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None, // No spec_group - should go to root specs/
            group_id: None,
            main_spec_ref: None,
            merge_strategy: None,
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created in root specs directory (no subdirectory)
        let spec_path = project_root.join(".aw/changes/test-change/specs/cross-cutting-spec.md");
        assert!(
            spec_path.exists(),
            "Spec should be created in root specs directory"
        );

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: cross-cutting-spec"));
        // Should NOT contain spec_group line
        assert!(!content.contains("spec_group:"));
    }

    #[test]
    fn test_create_spec_with_main_spec_ref_and_merge_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "extended-auth".to_string(),
            title: "Extended Authentication".to_string(),
            overview: "This specification extends the existing auth-flow spec with additional OAuth providers.".to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Add GitHub OAuth".to_string(),
                description: "Support GitHub as OAuth provider".to_string(),
                priority: "high".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "GitHub OAuth login".to_string(),
                given: Some("User clicks GitHub login".to_string()),
                when: "OAuth flow completes".to_string(),
                then: "User is authenticated".to_string(),
            }],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: Some("sdd".to_string()),
            group_id: None,
            main_spec_ref: Some("auth-flow".to_string()),
            merge_strategy: Some("extend".to_string()),
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created
        let spec_path = project_root.join(".aw/changes/test-change/specs/sdd/extended-auth.md");
        assert!(spec_path.exists());

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: extended-auth"));
        assert!(content.contains("spec_group: sdd"));
        assert!(content.contains("main_spec_ref: auth-flow"));
        assert!(content.contains("merge_strategy: extend"));
    }

    #[test]
    fn test_create_spec_invalid_merge_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "invalid-strategy".to_string(),
            title: "Invalid Strategy Test".to_string(),
            overview: "This specification tests validation of merge_strategy with invalid values."
                .to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None,
            group_id: None,
            main_spec_ref: None,
            merge_strategy: Some("invalid-strategy".to_string()),
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid merge_strategy"),
            "Error message was: {}",
            err_msg
        );
    }

    // ─── resolve_section_rules tests ────────────────────────────────────────

    #[test]
    fn test_resolve_section_rules_always_includes_overview_only() {
        let result = resolve_section_rules("some plain text", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::Overview),
            "must include overview"
        );
        assert!(
            !types.contains(&SectionType::Changes),
            "new TD section inference must not force legacy changes"
        );
    }

    #[test]
    fn test_resolve_section_rules_no_keywords_minimal() {
        // Plain text with no keyword matches → overview only
        let result = resolve_section_rules("nothing special here", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert_eq!(types, vec![SectionType::Overview]);
    }

    #[test]
    fn test_resolve_section_rules_rest_api_keyword() {
        let result = resolve_section_rules("Add a REST endpoint for users", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::RestApi),
            "should match REST keyword"
        );
        assert!(
            types.contains(&SectionType::Schema),
            "REST keyword also includes schema"
        );
    }

    #[test]
    fn test_resolve_section_rules_rpc_keyword() {
        let result = resolve_section_rules("Add an MCP tool for listing resources", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::RpcApi),
            "should match MCP tool keyword"
        );
        assert!(
            types.contains(&SectionType::Schema),
            "RPC keyword also includes schema"
        );
    }

    #[test]
    fn test_resolve_section_rules_ui_keyword_matches_frontend_sections() {
        let result = resolve_section_rules("Build a UI page for user dashboard", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::Wireframe),
            "should match UI keyword → wireframe"
        );
        assert!(
            types.contains(&SectionType::Component),
            "should match UI keyword → component"
        );
    }

    #[test]
    fn test_resolve_section_rules_design_token_keyword() {
        let result =
            resolve_section_rules("Define color tokens and typography for the theme", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::DesignToken),
            "should match token/color/typography keyword"
        );
    }

    #[test]
    fn test_resolve_section_rules_state_keyword() {
        let result = resolve_section_rules("Add lifecycle state transitions", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::StateMachine),
            "should match state keyword"
        );
    }

    #[test]
    fn test_resolve_section_rules_cli_keyword() {
        let result = resolve_section_rules("Add CLI command for data export", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::Cli),
            "should match CLI keyword"
        );
    }

    #[test]
    fn test_resolve_section_rules_database_keyword() {
        let result = resolve_section_rules("Create a database table for user sessions", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::DbModel),
            "should match database keyword"
        );
    }

    #[test]
    fn test_resolve_section_rules_config_keyword() {
        let result = resolve_section_rules("Add settings for .toml config file", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::Config),
            "should match config keyword"
        );
    }

    #[test]
    fn test_resolve_section_rules_async_keyword() {
        let result = resolve_section_rules("Add webhook background job", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::AsyncApi),
            "should match webhook/background keyword"
        );
    }

    #[test]
    fn test_resolve_section_rules_unit_test_added_when_gt_2_sections() {
        // overview + REST (rest-api + schema) = 3 keyword-matched sections
        let result = resolve_section_rules("Build a REST endpoint", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        // overview + rest-api + schema = 3 -> threshold > 2 met
        assert!(
            types.contains(&SectionType::UnitTest),
            "unit-test should be added when > 2 sections"
        );
    }

    #[test]
    fn test_resolve_section_rules_interaction_logic_dependency_added_when_gt_3() {
        // Trigger many keywords to get > 3 matched sections
        let result = resolve_section_rules(
            "Build a REST endpoint with state lifecycle and database table for the CLI command",
            None,
        );
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert!(
            types.contains(&SectionType::Interaction),
            "interaction added when > 3 sections"
        );
        assert!(
            types.contains(&SectionType::Logic),
            "logic added when > 3 sections"
        );
        assert!(
            types.contains(&SectionType::Dependency),
            "dependency added when > 3 sections"
        );
    }

    #[test]
    fn test_resolve_section_rules_sorted_by_fill_order() {
        let result = resolve_section_rules(
            "Add REST endpoint with database model and state machine for the UI component page with CLI command",
            None,
        );
        let orders: Vec<u8> = result
            .iter()
            .map(|e| e.section_type().fill_order())
            .collect();
        for window in orders.windows(2) {
            assert!(
                window[0] <= window[1],
                "sections must be sorted by fill_order, got {:?}",
                orders
            );
        }
    }

    #[test]
    fn test_resolve_section_rules_no_duplicates() {
        // keywords that overlap shouldn't produce duplicate section types
        let result = resolve_section_rules("REST api endpoint route HTTP", None);
        let types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        let unique: std::collections::HashSet<SectionType> = types.iter().copied().collect();
        assert_eq!(
            types.len(),
            unique.len(),
            "no duplicate section types allowed"
        );
    }

    // ─── resolve_section_rules + design system integration ───────────────────

    #[test]
    fn test_resolve_section_rules_no_design_system_all_required() {
        let result = resolve_section_rules(
            "Build a UI page with color tokens and typography theme",
            None,
        );
        for entry in &result {
            assert!(
                !entry.is_optional(),
                "{:?} should be required without design system",
                entry.section_type()
            );
        }
    }

    #[test]
    fn test_resolve_section_rules_with_design_system_provides_both() {
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let result = resolve_section_rules(
            "Build a UI page with color tokens and typography theme",
            Some(&ds),
        );
        let comp = result
            .iter()
            .find(|e| e.section_type() == SectionType::Component);
        let dt = result
            .iter()
            .find(|e| e.section_type() == SectionType::DesignToken);

        if let Some(comp) = comp {
            assert!(
                comp.is_optional(),
                "component should be optional with provides_components=true"
            );
        }
        if let Some(dt) = dt {
            assert!(
                dt.is_optional(),
                "design-token should be optional with provides_tokens=true"
            );
        }
    }

    #[test]
    fn test_resolve_section_rules_with_design_system_provides_tokens_only() {
        let ds = DesignSystem {
            library: "custom".to_string(),
            provides_tokens: true,
            provides_components: false,
        };
        let result = resolve_section_rules("Build a UI page with color tokens", Some(&ds));

        // design-token optional, component required
        if let Some(dt) = result
            .iter()
            .find(|e| e.section_type() == SectionType::DesignToken)
        {
            assert!(dt.is_optional(), "design-token should be optional");
        }
        if let Some(comp) = result
            .iter()
            .find(|e| e.section_type() == SectionType::Component)
        {
            assert!(!comp.is_optional(), "component should be required");
        }
    }

    #[test]
    fn test_resolve_section_rules_with_design_system_provides_components_only() {
        let ds = DesignSystem {
            library: "antd".to_string(),
            provides_tokens: false,
            provides_components: true,
        };
        let result = resolve_section_rules("Build a UI page with color tokens", Some(&ds));

        // component optional, design-token required
        if let Some(comp) = result
            .iter()
            .find(|e| e.section_type() == SectionType::Component)
        {
            assert!(comp.is_optional(), "component should be optional");
        }
        if let Some(dt) = result
            .iter()
            .find(|e| e.section_type() == SectionType::DesignToken)
        {
            assert!(!dt.is_optional(), "design-token should be required");
        }
    }

    #[test]
    fn test_resolve_section_rules_overview_never_optional_with_design_system() {
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let result = resolve_section_rules("Build a UI page with tokens", Some(&ds));

        let overview = result
            .iter()
            .find(|e| e.section_type() == SectionType::Overview)
            .unwrap();
        assert!(!overview.is_optional(), "overview must never be optional");

        assert!(
            result
                .iter()
                .all(|entry| entry.section_type() != SectionType::Changes),
            "new section resolution must not insert legacy changes"
        );
    }

    #[test]
    fn test_resolve_section_rules_design_system_does_not_affect_non_design_sections() {
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        // Trigger REST + state keywords
        let result = resolve_section_rules(
            "Build a REST endpoint with state lifecycle transitions",
            Some(&ds),
        );

        for entry in &result {
            match entry.section_type() {
                SectionType::Component | SectionType::DesignToken => {} // may be optional
                _ => assert!(
                    !entry.is_optional(),
                    "{:?} should be required (not a design section)",
                    entry.section_type()
                ),
            }
        }
    }

    #[test]
    fn test_create_spec_with_group_id_uses_groups_layout() {
        // Scenario: Spec file placed under group directory (R1)
        // GIVEN a multi-group change with group_id "group-directory-fix"
        // WHEN spec_service writes a spec file for spec_id "change-spec-logic"
        // THEN the file is created at groups/group-directory-fix/specs/change-spec-logic.md
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "change-spec-logic".to_string(),
            title: "Change Spec Logic".to_string(),
            overview: "Fix file placement bug where specs are written to the change root instead of groups/{group_id}/specs/ in multi-group changes.".to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Group-scoped path".to_string(),
                description: "Spec written to groups/{group_id}/specs/.".to_string(),
                priority: "high".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Spec placed under group directory".to_string(),
                given: Some("A multi-group change with group_id".to_string()),
                when: "spec_service writes a spec file".to_string(),
                then: "File created under groups/{group_id}/specs/".to_string(),
            }],
            spec_type: SpecType::Utility,
            diagrams: vec![],
            flow_diagram: None,
            data_model: None,
            api_spec: None,
            agent: None,
            duration_secs: None,
            spec_group: None, // group_id takes priority
            group_id: Some("group-directory-fix".to_string()),
            main_spec_ref: None,
            merge_strategy: Some("new".to_string()),
            tags: Vec::new(),
            changes: Vec::new(),
            depends: Vec::new(),
        };

        let result = create_spec(input, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file is under groups/group-directory-fix/specs/ NOT root specs/
        let group_spec_path = project_root
            .join(".aw/changes/test-change/groups/group-directory-fix/specs/change-spec-logic.md");
        assert!(
            group_spec_path.exists(),
            "Spec must be under groups/group-directory-fix/specs/"
        );

        // Verify root specs/ does NOT contain the file
        let root_spec_path =
            project_root.join(".aw/changes/test-change/specs/change-spec-logic.md");
        assert!(!root_spec_path.exists(), "Spec must NOT be in root specs/");

        let content = std::fs::read_to_string(&group_spec_path).unwrap();
        assert!(content.contains("id: change-spec-logic"));
    }
}
// CODEGEN-END

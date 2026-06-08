// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
// CODEGEN-BEGIN
//! SDD MCP Tool Handlers
//!
//! Implements handlers for diagram and spec generation tools.

use crate::generate::diagrams::{
    block_plus::{BlockDef, BlockPlusGenerator, BlockValidator},
    class::{generate_class_diagram, ClassInput},
    class_plus::{ClassDiagramDef, ClassPlusGenerator, ClassValidator},
    erd::{generate_erd, ErdInput},
    erd_plus::{ERDDef, ERDPlusGenerator, ERDValidator},
    flowchart::{generate_flowchart, FlowchartInput},
    // Plus modules
    flowchart_plus::{FlowchartDef, FlowchartPlusGenerator, FlowchartValidator},
    journey::{generate_journey, JourneyInput},
    journey_plus::{JourneyDef, JourneyPlusGenerator, JourneyValidator},
    mermaid_plus::{MermaidPlusGenerator, StateMachineDef, StateMachineValidator},
    mindmap::{generate_mindmap, MindmapInput},
    mindmap_plus::{MindmapDef, MindmapPlusGenerator, MindmapValidator},
    requirement::{generate_requirement_diagram, RequirementInput},
    requirement_plus::{RequirementDiagramDef, RequirementPlusGenerator, RequirementValidator},
    sequence::{generate_sequence, SequenceInput},
    sequence_plus::{SequenceDef, SequencePlusGenerator, SequenceValidator},
    state::{generate_state_diagram, StateInput},
};
use crate::generate::specs::{
    asyncapi::{generate_asyncapi, AsyncApiInput},
    openapi::{generate_openapi, OpenApiInput},
    openrpc::{generate_openrpc, OpenRpcInput},
    serverless::{generate_serverless_workflow, ServerlessWorkflowInput},
};
use serde_json::{json, Value};

/// Result type for handlers
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub type HandlerResult = Result<Value, String>;

/// Handle sdd_generate_flowchart
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_flowchart(args: &Value) -> HandlerResult {
    let input: FlowchartInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid flowchart input: {}", e))?;

    let mermaid =
        generate_flowchart(&input).map_err(|e| format!("Flowchart generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "flowchart"
    }))
}

/// Handle sdd_generate_sequence
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_sequence(args: &Value) -> HandlerResult {
    let input: SequenceInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid sequence input: {}", e))?;

    let mermaid =
        generate_sequence(&input).map_err(|e| format!("Sequence generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "sequence"
    }))
}

/// Handle sdd_generate_class
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_class(args: &Value) -> HandlerResult {
    let input: ClassInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid class diagram input: {}", e))?;

    let mermaid = generate_class_diagram(&input)
        .map_err(|e| format!("Class diagram generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "class"
    }))
}

/// Handle sdd_generate_state
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_state(args: &Value) -> HandlerResult {
    let input: StateInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid state diagram input: {}", e))?;

    let mermaid = generate_state_diagram(&input)
        .map_err(|e| format!("State diagram generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "state"
    }))
}

/// Handle sdd_generate_erd
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_erd(args: &Value) -> HandlerResult {
    let input: ErdInput =
        serde_json::from_value(args.clone()).map_err(|e| format!("Invalid ERD input: {}", e))?;

    let mermaid = generate_erd(&input).map_err(|e| format!("ERD generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "erd"
    }))
}

/// Handle sdd_generate_mindmap
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_mindmap(args: &Value) -> HandlerResult {
    let input: MindmapInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid mindmap input: {}", e))?;

    let mermaid =
        generate_mindmap(&input).map_err(|e| format!("Mindmap generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "mindmap"
    }))
}

/// Handle sdd_generate_requirement
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_requirement(args: &Value) -> HandlerResult {
    let input: RequirementInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid requirement input: {}", e))?;

    let mermaid = generate_requirement_diagram(&input)
        .map_err(|e| format!("Requirement diagram generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "requirement"
    }))
}

/// Handle sdd_generate_journey
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_journey(args: &Value) -> HandlerResult {
    let input: JourneyInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid journey input: {}", e))?;

    let mermaid =
        generate_journey(&input).map_err(|e| format!("Journey generation failed: {}", e))?;

    Ok(json!({
        "mermaid": mermaid,
        "type": "journey"
    }))
}

/// Handle sdd_generate_mermaid_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_mermaid_plus(args: &Value) -> HandlerResult {
    let machine: StateMachineDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid state machine definition: {}", e))?;

    let validator = StateMachineValidator::new();
    let validation = validator.validate(&machine);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = MermaidPlusGenerator::new();
    let output = generator
        .generate(&machine, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "mermaid_plus"
    }))
}

// === Plus Handlers ===

/// Handle sdd_generate_flowchart_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_flowchart_plus(args: &Value) -> HandlerResult {
    let flowchart: FlowchartDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid flowchart definition: {}", e))?;

    let validator = FlowchartValidator::new();
    let validation = validator.validate(&flowchart);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = FlowchartPlusGenerator::new();
    let output = generator
        .generate(&flowchart, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "flowchart_plus"
    }))
}

/// Handle sdd_generate_sequence_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_sequence_plus(args: &Value) -> HandlerResult {
    let sequence: SequenceDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid sequence definition: {}", e))?;

    let validator = SequenceValidator::new();
    let validation = validator.validate(&sequence);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = SequencePlusGenerator::new();
    let output = generator
        .generate(&sequence, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "sequence_plus"
    }))
}

/// Handle sdd_generate_class_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_class_plus(args: &Value) -> HandlerResult {
    let diagram: ClassDiagramDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid class diagram definition: {}", e))?;

    let validator = ClassValidator::new();
    let validation = validator.validate(&diagram);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = ClassPlusGenerator::new();
    let output = generator
        .generate(&diagram, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "class_plus"
    }))
}

/// Handle sdd_generate_erd_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_erd_plus(args: &Value) -> HandlerResult {
    let erd: ERDDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid ERD definition: {}", e))?;

    let validator = ERDValidator::new();
    let validation = validator.validate(&erd);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = ERDPlusGenerator::new();
    let output = generator
        .generate(&erd, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "erd_plus"
    }))
}

/// Handle sdd_generate_requirement_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_requirement_plus(args: &Value) -> HandlerResult {
    let diagram: RequirementDiagramDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid requirement diagram definition: {}", e))?;

    let validator = RequirementValidator::new();
    let validation = validator.validate(&diagram);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = RequirementPlusGenerator::new();
    let output = generator
        .generate(&diagram, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "requirement_plus"
    }))
}

/// Handle sdd_generate_mindmap_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_mindmap_plus(args: &Value) -> HandlerResult {
    let mindmap: MindmapDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid mindmap definition: {}", e))?;

    let validator = MindmapValidator::new();
    let validation = validator.validate(&mindmap);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = MindmapPlusGenerator::new();
    let output = generator
        .generate(&mindmap, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "mindmap_plus"
    }))
}

/// Handle sdd_generate_journey_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_journey_plus(args: &Value) -> HandlerResult {
    let journey: JourneyDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid journey definition: {}", e))?;

    let validator = JourneyValidator::new();
    let validation = validator.validate(&journey);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = JourneyPlusGenerator::new();
    let output = generator
        .generate(&journey, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "journey_plus"
    }))
}

/// Handle sdd_generate_block_plus
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_block_plus(args: &Value) -> HandlerResult {
    let diagram: BlockDef = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid block diagram definition: {}", e))?;

    let validator = BlockValidator::new();
    let validation = validator.validate(&diagram);

    if !validation.valid {
        let errors: Vec<String> = validation
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect();
        return Err(format!("Validation failed: {}", errors.join("; ")));
    }

    let generator = BlockPlusGenerator::new();
    let output = generator
        .generate(&diagram, validation)
        .map_err(|e| format!("Generation failed: {}", e))?;

    Ok(json!({
        "mermaid": output.diagram,
        "frontmatter": output.frontmatter,
        "combined": output.combined,
        "type": "block_plus"
    }))
}

// === Spec Handlers ===

/// Handle sdd_generate_openapi
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_openapi(args: &Value) -> HandlerResult {
    let input: OpenApiInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid OpenAPI input: {}", e))?;

    let spec = generate_openapi(&input).map_err(|e| format!("OpenAPI generation failed: {}", e))?;

    Ok(json!({
        "spec": spec,
        "type": "openapi"
    }))
}

/// Handle sdd_generate_asyncapi
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_asyncapi(args: &Value) -> HandlerResult {
    let input: AsyncApiInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid AsyncAPI input: {}", e))?;

    let spec =
        generate_asyncapi(&input).map_err(|e| format!("AsyncAPI generation failed: {}", e))?;

    Ok(json!({
        "spec": spec,
        "type": "asyncapi"
    }))
}

/// Handle sdd_generate_openrpc
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_openrpc(args: &Value) -> HandlerResult {
    let input: OpenRpcInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid OpenRPC input: {}", e))?;

    let spec = generate_openrpc(&input).map_err(|e| format!("OpenRPC generation failed: {}", e))?;

    Ok(json!({
        "spec": spec,
        "type": "openrpc"
    }))
}

/// Handle sdd_generate_serverless_workflow
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn handle_generate_serverless_workflow(args: &Value) -> HandlerResult {
    let input: ServerlessWorkflowInput = serde_json::from_value(args.clone())
        .map_err(|e| format!("Invalid Serverless Workflow input: {}", e))?;

    let spec = generate_serverless_workflow(&input)
        .map_err(|e| format!("Serverless Workflow generation failed: {}", e))?;

    Ok(json!({
        "spec": spec,
        "type": "serverless_workflow"
    }))
}

/// Route a tool call to the appropriate handler
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn call_tool(name: &str, args: &Value) -> HandlerResult {
    match name {
        // Simple diagram tools
        "sdd_generate_flowchart" => handle_generate_flowchart(args),
        "sdd_generate_sequence" => handle_generate_sequence(args),
        "sdd_generate_class" => handle_generate_class(args),
        "sdd_generate_state" => handle_generate_state(args),
        "sdd_generate_erd" => handle_generate_erd(args),
        "sdd_generate_mindmap" => handle_generate_mindmap(args),
        "sdd_generate_requirement" => handle_generate_requirement(args),
        "sdd_generate_journey" => handle_generate_journey(args),
        // Plus diagram tools
        "sdd_generate_mermaid_plus" => handle_generate_mermaid_plus(args),
        "sdd_generate_flowchart_plus" => handle_generate_flowchart_plus(args),
        "sdd_generate_sequence_plus" => handle_generate_sequence_plus(args),
        "sdd_generate_class_plus" => handle_generate_class_plus(args),
        "sdd_generate_erd_plus" => handle_generate_erd_plus(args),
        "sdd_generate_requirement_plus" => handle_generate_requirement_plus(args),
        "sdd_generate_mindmap_plus" => handle_generate_mindmap_plus(args),
        "sdd_generate_journey_plus" => handle_generate_journey_plus(args),
        "sdd_generate_block_plus" => handle_generate_block_plus(args),
        // Spec tools
        "sdd_generate_openapi" => handle_generate_openapi(args),
        "sdd_generate_asyncapi" => handle_generate_asyncapi(args),
        "sdd_generate_openrpc" => handle_generate_openrpc(args),
        "sdd_generate_serverless_workflow" => handle_generate_serverless_workflow(args),
        _ => Err(format!("Unknown SDD generate tool: {}", name)),
    }
}

/// Check if a tool name is an SDD generate tool
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/handlers.md#source
pub fn is_sdd_tool(name: &str) -> bool {
    name.starts_with("sdd_generate_")
}

// CODEGEN-END

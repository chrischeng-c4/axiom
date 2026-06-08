// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbEntity {
    pub fields: serde_json::Value,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbField {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub primary_key: Option<bool>,
    #[serde(default)]
    pub nullable: Option<bool>,
}

/// ERD-style entity/relationship payload (mirrors today's erd_plus schema family).
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbModelFrontmatter {
    pub id: String,
    pub entities: serde_json::Value,
    pub relationships: Vec<DbRelationship>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbModelIR {
    pub id: String,
    pub entities: serde_json::Value,
    pub relationships: Vec<DbRelationship>,
    /// entity-name -> list of entities reachable through FK relationships
    pub foreign_key_closure: serde_json::Value,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbRelationship {
    pub from: String,
    pub to: String,
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// Generic directed-graph payload (dependency / block_plus today).
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyFrontmatter {
    pub id: String,
    pub nodes: serde_json::Value,
    pub edges: Vec<serde_json::Value>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IREdge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
}

/// Discriminated union of lowered IR payloads. Produced by the lowering pass; consumed by emitters. Language-neutral by contract (R6): no Rust / TypeScript / Python types referenced from any payload.
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRFamily {
    pub kind: String,
    /// Back-reference to the `MermaidPlusBlockTyped.id` this IR was lowered from.
    pub source_id: String,
    #[serde(default)]
    pub logic_graph: Option<LogicGraphIR>,
    #[serde(default)]
    pub state_machine: Option<StateMachineIR>,
    #[serde(default)]
    pub interaction: Option<InteractionIR>,
    #[serde(default)]
    pub db_model: Option<DbModelIR>,
    #[serde(default)]
    pub requirement_set: Option<RequirementSetIR>,
    #[serde(default)]
    pub scenario_set: Option<ScenarioSetIR>,
    #[serde(default)]
    pub test_plan: Option<TestPlanIR>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRNode {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionActor {
    pub id: String,
    #[serde(default)]
    pub kind: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionFrontmatter {
    pub id: String,
    pub actors: Vec<InteractionActor>,
    pub messages: Vec<InteractionMessage>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionIR {
    pub id: String,
    pub actors: Vec<InteractionActor>,
    pub messages: Vec<InteractionMessage>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionMessage {
    pub from: String,
    pub to: String,
    pub name: String,
    #[serde(default)]
    pub returns: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicEdge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicFrontmatter {
    pub id: String,
    /// Node id of entry point
    pub entry: String,
    pub nodes: serde_json::Value,
    pub edges: Vec<LogicEdge>,
}

/// Directed control-flow graph. Lowered from `LogicFrontmatter`. Adds derived fields the raw frontmatter doesn't carry: `topological_order`, `unreachable_nodes`, `decision_branches`.
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicGraphIR {
    pub id: String,
    pub entry: String,
    pub nodes: Vec<IRNode>,
    pub edges: Vec<IREdge>,
    /// Node ids sorted topologically when the graph is acyclic; empty array when a cycle was detected (a lowering diagnostic accompanies the empty array).
    pub topological_order: Vec<String>,
    pub unreachable_nodes: Vec<String>,
    /// decision-node-id -> ordered list of outgoing edge labels
    #[serde(default)]
    pub decision_branches: Option<serde_json::Value>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicNode {
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// Diagnostic produced by the lowering pass. Distinct from `ParseDiagnostic` (which comes from frontmatter parse). Both carry the same shape so consumers can merge.
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LowerDiagnostic {
    /// Lowering diagnostic code, format `MP-LO-<n>`. Reserved codes:
    ///   MP-LO-001 unknown node id in edge
    ///   MP-LO-002 cycle detected in logic graph
    ///   MP-LO-003 unreachable node
    ///   MP-LO-004 initial state not in nodes
    ///   MP-LO-005 actor referenced in messages but not declared
    ///   MP-LO-006 FK target entity missing
    ///   MP-LO-007 test verifies unknown requirement id
    pub code: String,
    pub message: String,
    pub span: SourceSpan,
    pub severity: String,
}

/// Typed replacement for `TypedBody::MermaidPlus(MermaidPlusPayload)`. Preserves `frontmatter_raw` and `rendered_body` for round-trip fidelity (envelope migration spec invariant); adds stable id, source span, parsed typed frontmatter, and any diagnostics accumulated during parsing.
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MermaidPlusBlockTyped {
    /// Stable id derived from `frontmatter.id` when present, otherwise synthesised as `<section-type>:<line_start>`. Used by downstream IR nodes to back-reference the source AST node.
    pub id: String,
    pub span: SourceSpan,
    pub frontmatter: MermaidPlusFrontmatter,
    /// Canonical bytes between the `---` markers inside the mermaid fence. Excluded from `frontmatter` hash to preserve the envelope spec's hash-of-parsed-data invariant.
    pub frontmatter_raw: String,
    /// Mermaid diagram body after the closing `---`.
    pub rendered_body: String,
    /// Non-fatal parse diagnostics (recoverable). Fatal errors surface via `Result::Err(TdParseError)` at the parse layer and never produce a typed block.
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Discriminated union over diagram-kind. Replaces the untyped `serde_yaml::Value` carried in today's `MermaidPlusPayload`. `kind` is the discriminant; remaining fields are kind-specific payloads with closed schemas.
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MermaidPlusFrontmatter {
    pub kind: String,
    #[serde(default)]
    pub state_machine: Option<StateMachineFrontmatter>,
    #[serde(default)]
    pub logic: Option<LogicFrontmatter>,
    #[serde(default)]
    pub interaction: Option<InteractionFrontmatter>,
    #[serde(default)]
    pub db_model: Option<DbModelFrontmatter>,
    #[serde(default)]
    pub requirements: Option<RequirementsFrontmatter>,
    #[serde(default)]
    pub scenarios: Option<ScenariosFrontmatter>,
    #[serde(default)]
    pub test_plan: Option<TestPlanFrontmatter>,
    #[serde(default)]
    pub mindmap: Option<MindmapFrontmatter>,
    #[serde(default)]
    pub dependency: Option<DependencyFrontmatter>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MindmapFrontmatter {
    pub id: String,
    pub root: MindmapNode,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MindmapNode {
    pub label: String,
    pub children: Vec<MindmapNode>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    /// Stable diagnostic code. Format: `MP-<phase>-<n>` e.g. `MP-FM-001` (frontmatter), `MP-LO-007` (lowering), `MP-RB-003` (rendered-body).
    pub code: String,
    pub message: String,
    pub span: SourceSpan,
    pub severity: String,
    /// Optional suggested fix.
    #[serde(default)]
    pub hint: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementItem {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementSetIR {
    pub id: String,
    pub requirements: serde_json::Value,
    /// Stable iteration order: priority (must>should>could>wont) then id ASC.
    pub ordered_ids: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementsFrontmatter {
    pub id: String,
    pub requirements: serde_json::Value,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioItem {
    pub id: String,
    pub title: String,
    pub steps: Vec<serde_json::Value>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioSetIR {
    pub id: String,
    pub scenarios: Vec<ScenarioItem>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenariosFrontmatter {
    pub id: String,
    pub scenarios: Vec<ScenarioItem>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub line_start: u64,
    pub line_end: u64,
    pub byte_start: u64,
    pub byte_end: u64,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateEdge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub event: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateMachineFrontmatter {
    pub id: String,
    /// Node id of initial state
    pub initial: String,
    pub nodes: serde_json::Value,
    pub edges: Vec<StateEdge>,
}

/// Lowered state machine. Adds `reachability`, `terminal_states`, and `event_alphabet` derived during lowering.
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateMachineIR {
    pub id: String,
    pub initial: String,
    pub nodes: Vec<IRNode>,
    pub edges: Vec<IREdge>,
    pub terminal_states: Vec<String>,
    /// Distinct event labels appearing on any edge, sorted.
    pub event_alphabet: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateNode {
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestItem {
    #[serde(rename = "type")]
    pub r#type: String,
    pub name: String,
    #[serde(default)]
    pub file: Option<String>,
    pub verifies: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestPlanFrontmatter {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub tests: serde_json::Value,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestPlanIR {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub tests: serde_json::Value,
    /// requirement-id -> list of test-ids that verify it (inverse of TestItem.verifies)
    pub verifies_index: serde_json::Value,
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#logic
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#logic
pub fn enter() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-split_fence
    // TODO: Implement process step: Locate inner ``---`` markers; split raw into [frontmatter_raw, rendered_body]
    todo!("process: Locate inner ``---`` markers; split raw into [frontmatter_raw, rendered_body]");
    // Decision: Frontmatter markers found?
    if todo!("decision: Frontmatter markers found?")
    /* no */
    {
        todo!("terminal: Err(MP-FM-001 missing-frontmatter)");
    } else
    /* yes */
    {
        // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-parse_yaml
        // TODO: Implement process step: serde_yaml::from_str::<MermaidPlusFrontmatter>(frontmatter_raw) (typed deserializer dispatches on `kind`)
        todo!("process: serde_yaml::from_str::<MermaidPlusFrontmatter>(frontmatter_raw) (typed deserializer dispatches on `kind`)");
        // Decision: YAML parses into typed frontmatter?
        if todo!("decision: YAML parses into typed frontmatter?")
        /* no */
        {
            todo!("terminal: Err(MP-FM-002 invalid-yaml | MP-FM-003 unknown-kind | MP-FM-004 schema-mismatch)");
        } else
        /* yes */
        {
            // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-derive_id
            // TODO: Implement process step: id = frontmatter.id().unwrap_or(format!("{section_type}:{line_start}"))
            todo!("process: id = frontmatter.id().unwrap_or(format!(\"{{section_type}}:{{line_start}}\"))");
            // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-build_block
            // TODO: Implement process step: Construct MermaidPlusBlockTyped { id, span, frontmatter, frontmatter_raw, rendered_body, diagnostics: [] }
            todo!("process: Construct MermaidPlusBlockTyped {{ id, span, frontmatter, frontmatter_raw, rendered_body, diagnostics: [] }}");
            todo!("terminal: Ok(MermaidPlusBlockTyped) — parse layer done");
        }
    }
    // Terminal: err_no_fence -> Err(MP-FM-001 missing-frontmatter)
    // Terminal: err_yaml -> Err(MP-FM-002 invalid-yaml | MP-FM-003 unknown-kind | MP-FM-004 schema-mismatch)
    // Terminal: lower_unsupp -> (None, [info: MP-LO-999 unsupported-ir-family]) — mindmap / dependency carry no IR yet
    // Terminal: ret_block -> Ok(MermaidPlusBlockTyped) — parse layer done
    // Terminal: ret_ir -> (Some(IRFamily), diagnostics)
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-td-ast.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-td-ast.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END

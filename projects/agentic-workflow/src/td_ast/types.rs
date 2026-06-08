//! Core type declarations for the `crate::td_ast` module.
//!
//! @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema

use crate::generate::frontmatter::MermaidPlusBlock;
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Unified AST for a single tech-design spec file. Produced by parse_td(path) and consumed by entity-enumeration, hashing, and downstream tooling (R1, R2).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDAst {
    /// Parsed YAML frontmatter (id, fill_sections, summary, ...).
    pub frontmatter: serde_yaml::Value,
    /// Ordered list of parsed sections.
    #[serde(default)]
    pub sections: Vec<TDSection>,
}

/// One parsed section inside a TD spec. Joins annotation metadata (from parse_all_section_annotations) with the typed body (from section-type-driven dispatch). Satisfies R2, R7.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDSection {
    /// Section type identifier (canonical variant from SectionType enum).
    pub section_type: SectionType,
    /// Declared content language (e.g. yaml, mermaid).
    pub lang: String,
    /// Typed representation of the fenced code block.
    pub body: TypedBody,
    /// 1-based line number of the section heading.
    pub line_start: usize,
    /// 1-based line number of the last line in this section.
    pub line_end: usize,
    /// Deterministic hash over canonical body content (blake3 or xxh3, R8). None for Placeholder and Unsupported variants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<u64>,
}

/// Structured parse error returned when a section body cannot be parsed by its expected TypedBody parser. Satisfies R12 + Stage 1B R2.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdParseError {
    /// Discriminant: Frontmatter | TypedPayloadParse | Generic. Stage 1B adds TypedPayloadParse so callers can distinguish typed-payload deserialisation failures from frontmatter / IO failures.
    #[serde(default = "default_parse_kind")]
    pub kind: super::payloads::TdParseErrorKind,
    /// Section type whose dispatch attempted to parse the body. For TypedPayloadParse this is the `expected_type` from R2.
    pub section_type: SectionType,
    /// 1-based first line of the offending section.
    pub line_start: usize,
    /// 1-based last line of the offending section.
    pub line_end: usize,
    /// Human-readable parse failure description naming the section and line range.
    pub message: String,
    /// Optional verbatim error text from the underlying serde deserialiser. Carried for typed-payload-parse errors; None otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Discriminated body of a parsed TD section. Nine typed variants cover known section families (R3); the opaque `Unsupported` variant carries sections whose parser is not yet implemented (R9).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum TypedBody {
    /// Mermaid Plus block (state-machine, logic, interaction, dependency,
    /// db-model, scenarios, unit-test). Hash covers frontmatter only (R4, R8).
    MermaidPlus(MermaidPlusPayload),
    /// JSON Schema document (schema, config, wireframe, component,
    /// design-token, manifest, e2e-test). Typed in Stage 1B (sdd-td-ast-payloads).
    JsonSchema(super::payloads::JsonSchemaPayload),
    /// OpenRPC 1.3 document (rpc-api). Typed in Stage 1B.
    OpenRpc(super::payloads::OpenRpcPayload),
    /// OpenAPI 3.1 document (rest-api). Typed in Stage 1B.
    OpenApi(super::payloads::OpenApiPayload),
    /// AsyncAPI 2.6 document (async-api). Typed in Stage 1B.
    AsyncApi(super::payloads::AsyncApiPayload),
    /// CLI manifest YAML (cli). Typed in Stage 1B.
    CliManifest(super::payloads::CliManifestPayload),
    /// Config manifest YAML (config). Typed in Stage 1B.
    ConfigManifest(super::payloads::ConfigManifestPayload),
    /// Plain markdown body (doc, overview, changes-as-prose).
    Markdown(String),
    /// Section listed in `fill_sections` but not yet authored.
    Placeholder,
    /// Opaque carrier for section types without a typed parser. Carries
    /// the raw block string for round-trip fidelity (R9).
    Unsupported(String),
}
// CODEGEN-END

fn default_parse_kind() -> super::payloads::TdParseErrorKind {
    super::payloads::TdParseErrorKind::Generic
}

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#source
// CODEGEN-BEGIN
/// Serializable payload for [`TypedBody::MermaidPlus`].
///
/// We store the parsed frontmatter (used for hashing) and the raw rendered
/// body separately so consumers can re-render or re-hash without re-parsing.
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MermaidPlusPayload {
    /// Parsed YAML frontmatter (the structured part).
    pub frontmatter: serde_yaml::Value,
    /// Raw frontmatter string (canonical bytes between the `---` markers).
    pub frontmatter_raw: String,
    /// Mermaid diagram body (after the closing `---`). Excluded from hash.
    pub rendered_body: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#source
impl From<MermaidPlusBlock> for MermaidPlusPayload {
    fn from(b: MermaidPlusBlock) -> Self {
        Self {
            frontmatter: b.frontmatter,
            frontmatter_raw: b.frontmatter_raw,
            rendered_body: b.body,
        }
    }
}

/// Registry mapping each known [`SectionType`] variant to its [`TypedBody`]
/// constructor family. Replaces the keyword-match heuristic in the legacy
/// `try_parse_block`. Unknown variants fall through to [`SectionKind::Unsupported`] (R7, R9).
///
/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionKind {
    /// Mermaid Plus family — state-machine, logic, interaction, dependency,
    /// db-model, scenarios, unit-test, mindmap, requirements.
    MermaidFamily,
    /// JSON Schema family — schema, wireframe, component, design-token,
    /// manifest, e2e-test.
    JsonSchemaFamily,
    /// OpenRPC 1.3 family — rpc-api.
    OpenRpcFamily,
    /// OpenAPI 3.1 family — rest-api.
    OpenApiFamily,
    /// AsyncAPI 2.6 family — async-api.
    AsyncApiFamily,
    /// Plain markdown family — doc, overview.
    MarkdownFamily,
    /// CLI manifest family — cli.
    CliFamily,
    /// Config manifest family — config.
    ConfigFamily,
    /// Changes manifest family — changes (YAML schema).
    ChangesFamily,
    /// Section types without a typed parser at this implementation time (R9).
    Unsupported,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#source
impl SectionKind {
    /// Dispatch table: `SectionType` → `SectionKind`. Centralised so that
    /// adding a new section type is a one-line change at the registry.
    ///
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
    pub fn for_section_type(st: SectionType) -> Self {
        match st {
            SectionType::Interaction
            | SectionType::Logic
            | SectionType::Dependency
            | SectionType::StateMachine
            | SectionType::DbModel
            | SectionType::Mindmap
            | SectionType::Scenarios
            | SectionType::UnitTest
            | SectionType::Requirements => SectionKind::MermaidFamily,

            SectionType::Schema
            | SectionType::Wireframe
            | SectionType::Component
            | SectionType::DesignToken
            | SectionType::RuntimeImage
            | SectionType::Deployment
            | SectionType::Manifest
            | SectionType::E2eTest => SectionKind::JsonSchemaFamily,

            SectionType::RpcApi => SectionKind::OpenRpcFamily,
            SectionType::RestApi => SectionKind::OpenApiFamily,
            SectionType::AsyncApi => SectionKind::AsyncApiFamily,
            SectionType::Cli => SectionKind::CliFamily,
            SectionType::Config => SectionKind::ConfigFamily,
            SectionType::Changes => SectionKind::ChangesFamily,
            SectionType::Doc | SectionType::Overview => SectionKind::MarkdownFamily,
        }
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#schema
// CODEGEN-BEGIN

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
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-td-ast.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-td-ast.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END

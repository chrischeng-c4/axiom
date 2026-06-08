// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
// CODEGEN-BEGIN
//! Central Spec Format Rules
//!
//! This module defines the canonical format rules for genesis specifications.
//! Both MCP tools and validators derive their rules from this single source of truth.

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
use std::str::FromStr;

use super::tech_stack::DesignSystem;

// ─── Section Type System ──────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

/// Section type annotation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SectionType {
    Overview,
    Changes,
    Requirements,
    Scenarios,
    UnitTest,
    E2eTest,
    Interaction,
    Logic,
    Dependency,
    StateMachine,
    DbModel,
    Mindmap,
    RestApi,
    RpcApi,
    AsyncApi,
    Cli,
    Schema,
    Config,
    Wireframe,
    Component,
    DesignToken,
    RuntimeImage,
    Deployment,
    Doc,
    Manifest,
}

/// Mermaid diagram types that can be used in specs.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DiagramType {
    /// Sequence diagram.
    #[serde(rename = "sequence")]
    Sequence,
    /// Entity relationship diagram.
    #[serde(rename = "erd")]
    Erd,
    /// Class diagram.
    #[serde(rename = "class")]
    Class,
    /// Flowchart.
    #[serde(rename = "flowchart")]
    Flowchart,
    /// State diagram.
    #[serde(rename = "state")]
    State,
    /// Mind map.
    #[serde(rename = "mindmap")]
    MindMap,
    /// Requirement diagram.
    #[serde(rename = "requirement")]
    Requirement,
    /// Journey diagram.
    #[serde(rename = "journey")]
    Journey,
}

/// API specification types required for code generation.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiSpecType {
    /// OpenAPI 3.1 for REST APIs.
    #[serde(rename = "openapi-3.1")]
    OpenApi31,
    /// AsyncAPI 2.6 for event-driven systems.
    #[serde(rename = "asyncapi-2.6")]
    AsyncApi26,
    /// JSON Schema for data models.
    #[serde(rename = "json-schema")]
    JsonSchema,
    /// OpenRPC 1.3 for JSON-RPC APIs.
    #[serde(rename = "openrpc-1.3")]
    OpenRpc13,
    /// Serverless Workflow 0.8 for orchestration.
    #[serde(rename = "serverless-workflow-0.8")]
    ServerlessWorkflow08,
}

/// Specification types for genesis specs (deprecated; use SectionType to annotate sections).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[allow(deprecated)]
#[deprecated(
    since = "0.0.0",
    note = "Use SectionType to annotate individual sections instead"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
pub enum SpecType {
    /// HTTP REST API specification.
    #[serde(rename = "http-api")]
    HttpApi,
    /// Event-driven system specification.
    #[serde(rename = "event-driven")]
    EventDriven,
    /// Data model specification.
    #[serde(rename = "data-model")]
    DataModel,
    /// Algorithm or state machine specification.
    #[serde(rename = "algorithm")]
    Algorithm,
    /// Service integration specification.
    #[serde(rename = "integration")]
    Integration,
    /// JSON-RPC API specification.
    #[serde(rename = "rpc-api")]
    RpcApi,
    /// Workflow orchestration specification.
    #[serde(rename = "workflow")]
    Workflow,
    /// Utility/helper specification.
    #[serde(rename = "utility")]
    Utility,
}

/// Format style for scenarios.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioFormat {
    #[serde(rename = "SingleLine")]
    SingleLine,
    #[serde(rename = "MultiLine")]
    MultiLine,
}

/// Document type for spec format rules.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    #[serde(rename = "Prd")]
    Prd,
    #[serde(rename = "Spec")]
    Spec,
    #[serde(rename = "Task")]
    Task,
}

/// Section entry — either required or with explicit optionality.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SectionEntry {
    /// Required section — always included.
    Required(SectionType),
    /// Section with explicit optionality annotation.
    WithOptional {
        section_type: SectionType,
        optional: bool,
    },
}

/// Central specification format rules.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFormatRules {
    /// Document type these rules apply to.
    pub document_type: DocumentType,
    /// Required top-level headings.
    pub required_headings: Vec<String>,
    /// Scenario format style.
    pub scenario_format: ScenarioFormat,
    /// Heading level for scenarios.
    pub scenario_heading_level: u8,
    /// Scenario heading prefix.
    pub scenario_heading_prefix: String,
    /// Minimum number of scenarios required.
    pub min_scenarios: usize,
    /// WHEN keyword.
    pub when_keyword: String,
    /// THEN keyword.
    pub then_keyword: String,
    /// Whether scenarios must have both WHEN and THEN.
    pub require_when_then: bool,
    /// Optional requirement heading pattern.
    pub requirement_pattern: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl SectionType {
    /// Fill priority order (lower number = fill first).
    ///
    /// Used to produce deterministic section fill sequences.
    // @spec projects/agentic-workflow/tech-design/core/logic/spec-structure.md#R1
    pub fn fill_order(&self) -> u8 {
        match self {
            // 0: understand scope first
            SectionType::Overview => 0,
            // 1-2: requirements and behavior (top-down reasoning)
            SectionType::Requirements => 1,
            SectionType::Scenarios => 2,
            // 3-8: diagrams (structural overview before details)
            SectionType::Mindmap => 3,
            SectionType::StateMachine => 4,
            SectionType::Interaction => 5,
            SectionType::Logic => 6,
            SectionType::Dependency => 7,
            SectionType::DbModel => 8,
            // 9-13: data and API (defined after diagrams)
            SectionType::Schema => 9,
            SectionType::RestApi => 10,
            SectionType::RpcApi => 11,
            SectionType::AsyncApi => 12,
            SectionType::Cli => 13,
            // 14-16: UI
            SectionType::Wireframe => 14,
            SectionType::Component => 15,
            SectionType::DesignToken => 16,
            // 17-20: configuration, packaging, and operations
            SectionType::Config => 17,
            SectionType::Manifest => 18,
            SectionType::RuntimeImage => 19,
            SectionType::Deployment => 20,
            // 21-22: executable verification designs
            SectionType::UnitTest => 21,
            SectionType::E2eTest => 22,
            // 23-24: delta and doc (last)
            SectionType::Changes => 23,
            SectionType::Doc => 24,
        }
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R1
    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R4
    // @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R5
    /// Default content language for this section type.
    ///
    /// Three langs only: `markdown`, `yaml`, `mermaid`. JSON has been removed.
    ///
    /// - `markdown`: prose-only sections (overview, doc)
    /// - `mermaid`: all diagram sections + requirements (requirementDiagram) + unit-test
    /// - `yaml`: all structured data sections (APIs, schema, config, scenarios, etc.)
    pub fn default_lang(&self) -> &'static str {
        match self {
            // Prose-only sections
            SectionType::Overview | SectionType::Doc => "markdown",
            // Diagram sections (Mermaid Plus format — YAML frontmatter inside mermaid block)
            // requirements and unit-test use Mermaid Plus requirementDiagram (SysML v1.6)
            SectionType::Interaction
            | SectionType::Logic
            | SectionType::Dependency
            | SectionType::StateMachine
            | SectionType::DbModel
            | SectionType::Mindmap
            | SectionType::Requirements
            | SectionType::UnitTest => "mermaid",
            // Structured data sections — all use YAML (not JSON)
            // scenarios: YAML GWT format {id, given, when, then, diagram_ref?}
            // schema, rpc-api, config, component, design-token: YAML (was JSON)
            SectionType::RestApi
            | SectionType::AsyncApi
            | SectionType::Changes
            | SectionType::Wireframe
            | SectionType::Cli
            | SectionType::Scenarios
            | SectionType::RpcApi
            | SectionType::Schema
            | SectionType::Config
            | SectionType::Component
            | SectionType::DesignToken
            | SectionType::RuntimeImage
            | SectionType::Deployment
            | SectionType::Manifest
            | SectionType::E2eTest => "yaml",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SectionType::Overview => "overview",
            SectionType::Changes => "changes",
            SectionType::Requirements => "requirements",
            SectionType::Scenarios => "scenarios",
            SectionType::UnitTest => "unit-test",
            SectionType::E2eTest => "e2e-test",
            SectionType::Interaction => "interaction",
            SectionType::Logic => "logic",
            SectionType::Dependency => "dependency",
            SectionType::StateMachine => "state-machine",
            SectionType::DbModel => "db-model",
            SectionType::Mindmap => "mindmap",
            SectionType::RestApi => "rest-api",
            SectionType::RpcApi => "rpc-api",
            SectionType::AsyncApi => "async-api",
            SectionType::Cli => "cli",
            SectionType::Schema => "schema",
            SectionType::Config => "config",
            SectionType::Wireframe => "wireframe",
            SectionType::Component => "component",
            SectionType::DesignToken => "design-token",
            SectionType::RuntimeImage => "runtime-image",
            SectionType::Deployment => "deployment",
            SectionType::Doc => "doc",
            SectionType::Manifest => "manifest",
        }
    }

    /// Returns all section types in fill order.
    pub fn all_in_fill_order() -> Vec<SectionType> {
        let mut types = vec![
            SectionType::Overview,
            SectionType::Changes,
            SectionType::Requirements,
            SectionType::Scenarios,
            SectionType::UnitTest,
            SectionType::E2eTest,
            SectionType::Interaction,
            SectionType::Logic,
            SectionType::Dependency,
            SectionType::StateMachine,
            SectionType::DbModel,
            SectionType::Mindmap,
            SectionType::RestApi,
            SectionType::RpcApi,
            SectionType::AsyncApi,
            SectionType::Cli,
            SectionType::Schema,
            SectionType::Config,
            SectionType::Wireframe,
            SectionType::Component,
            SectionType::DesignToken,
            SectionType::Manifest,
            SectionType::RuntimeImage,
            SectionType::Deployment,
            SectionType::Doc,
        ];
        types.sort_by_key(|t| t.fill_order());
        types
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl FromStr for SectionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "overview" => Ok(SectionType::Overview),
            "changes" => Ok(SectionType::Changes),
            "requirements" => Ok(SectionType::Requirements),
            "scenarios" => Ok(SectionType::Scenarios),
            "unit-test" | "unit_test" | "test-plan" | "test_plan" | "tests" => {
                Ok(SectionType::UnitTest)
            }
            "e2e-test" | "e2e_test" | "e2e" | "end-to-end-test" | "end_to_end_test" => {
                Ok(SectionType::E2eTest)
            }
            "interaction" | "sequence" => Ok(SectionType::Interaction),
            "logic" | "flowchart" => Ok(SectionType::Logic),
            "dependency" | "class" => Ok(SectionType::Dependency),
            "state-machine" | "state" => Ok(SectionType::StateMachine),
            "db-model" | "erd" => Ok(SectionType::DbModel),
            "mindmap" => Ok(SectionType::Mindmap),
            "rest-api" | "openapi" => Ok(SectionType::RestApi),
            "rpc-api" | "openrpc" => Ok(SectionType::RpcApi),
            "async-api" | "asyncapi" => Ok(SectionType::AsyncApi),
            "cli" => Ok(SectionType::Cli),
            "schema" => Ok(SectionType::Schema),
            "config" => Ok(SectionType::Config),
            "wireframe" | "frontend" => Ok(SectionType::Wireframe),
            "component" => Ok(SectionType::Component),
            "design-token" => Ok(SectionType::DesignToken),
            "runtime-image" | "container-image" | "container" | "dockerfile" => {
                Ok(SectionType::RuntimeImage)
            }
            "deployment" | "deploy" | "kustomize" | "kubernetes" | "k8s" => {
                Ok(SectionType::Deployment)
            }
            "doc" => Ok(SectionType::Doc),
            "manifest" => Ok(SectionType::Manifest),
            _ => Err(format!("Unknown section type: {}", s)),
        }
    }
}

// ─── Section Entry (with optionality) ────────────────────────────────────────

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl SectionEntry {
    /// Get the underlying section type.
    pub fn section_type(&self) -> SectionType {
        match self {
            SectionEntry::Required(st) => *st,
            SectionEntry::WithOptional { section_type, .. } => *section_type,
        }
    }

    /// Whether this section is optional (may be skipped by fill loop).
    pub fn is_optional(&self) -> bool {
        match self {
            SectionEntry::Required(_) => false,
            SectionEntry::WithOptional { optional, .. } => *optional,
        }
    }

    /// Create a required section entry.
    pub fn required(section_type: SectionType) -> Self {
        SectionEntry::Required(section_type)
    }

    /// Create an optional section entry.
    pub fn optional(section_type: SectionType) -> Self {
        SectionEntry::WithOptional {
            section_type,
            optional: true,
        }
    }

    /// Serialize to `fill_sections` frontmatter format.
    ///
    /// Required sections → `"overview"`
    /// Optional sections → `"component (optional)"`
    pub fn to_fill_section_string(&self) -> String {
        let name = self.section_type().as_str();
        if self.is_optional() {
            format!("{} (optional)", name)
        } else {
            name.to_string()
        }
    }

    /// Parse from `fill_sections` frontmatter string.
    ///
    /// Supports both `"overview"` and `"component (optional)"` formats.
    pub fn from_fill_section_string(s: &str) -> Result<Self, String> {
        let trimmed = s.trim();
        if let Some(name) = trimmed.strip_suffix("(optional)") {
            let name = name.trim();
            let st = SectionType::from_str(name)?;
            Ok(SectionEntry::optional(st))
        } else {
            let st = SectionType::from_str(trimmed)?;
            Ok(SectionEntry::required(st))
        }
    }
}

// ─── Section Optionality Filter ──────────────────────────────────────────────

/// Sections that are never optional regardless of tech stack detection.
const NEVER_OPTIONAL: &[SectionType] = &[SectionType::Overview];

/// Apply section optionality filter based on detected design system.
///
/// After keyword rule matching produces candidate sections, this filter
/// checks `design_system` capabilities and marks `design-token` and
/// `component` sections as optional when the design system provides those.
///
/// # Rules
///
/// | Condition | `design-token` | `component` |
/// |-----------|---------------|-------------|
/// | No design system detected | required | required |
/// | `provides_tokens: true` | **optional** | required |
/// | `provides_components: true` | required | **optional** |
/// | Both `true` | **optional** | **optional** |
///
/// - `overview` is never optional (always-required rule)
/// - Optionality only applies to keyword-matched sections, not to
///   always-required or conditional-count sections
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
pub fn apply_section_optionality(
    sections: Vec<SectionType>,
    design_system: Option<&DesignSystem>,
) -> Vec<SectionEntry> {
    sections
        .into_iter()
        .map(|st| {
            // Never-optional sections stay required
            if NEVER_OPTIONAL.contains(&st) {
                return SectionEntry::required(st);
            }

            // Without a design system, all sections are required
            let ds = match design_system {
                Some(ds) => ds,
                None => return SectionEntry::required(st),
            };

            // Apply optionality based on design system capabilities
            match st {
                SectionType::DesignToken if ds.provides_tokens => SectionEntry::optional(st),
                SectionType::Component if ds.provides_components => SectionEntry::optional(st),
                _ => SectionEntry::required(st),
            }
        })
        .collect()
}

/// Parse a `fill_sections` frontmatter string into base section name and optional flag.
///
/// - `"component (optional)"` → `("component", true)`
/// - `"overview"` → `("overview", false)`
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
pub fn parse_fill_section_str(s: &str) -> (&str, bool) {
    let trimmed = s.trim();
    if let Some(name) = trimmed.strip_suffix("(optional)") {
        (name.trim(), true)
    } else {
        (trimmed, false)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl DiagramType {
    pub fn as_str(&self) -> &str {
        match self {
            DiagramType::Sequence => "sequence",
            DiagramType::Erd => "erd",
            DiagramType::Class => "class",
            DiagramType::Flowchart => "flowchart",
            DiagramType::State => "state",
            DiagramType::MindMap => "mindmap",
            DiagramType::Requirement => "requirement",
            DiagramType::Journey => "journey",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl ApiSpecType {
    pub fn as_str(&self) -> &str {
        match self {
            ApiSpecType::OpenApi31 => "openapi-3.1",
            ApiSpecType::AsyncApi26 => "asyncapi-2.6",
            ApiSpecType::JsonSchema => "json-schema",
            ApiSpecType::OpenRpc13 => "openrpc-1.3",
            ApiSpecType::ServerlessWorkflow08 => "serverless-workflow-0.8",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl FromStr for ApiSpecType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openapi-3.1" => Ok(ApiSpecType::OpenApi31),
            "asyncapi-2.6" => Ok(ApiSpecType::AsyncApi26),
            "json-schema" => Ok(ApiSpecType::JsonSchema),
            "openrpc-1.3" => Ok(ApiSpecType::OpenRpc13),
            "serverless-workflow-0.8" => Ok(ApiSpecType::ServerlessWorkflow08),
            _ => Err(format!("Unknown API spec type: {}", s)),
        }
    }
}

#[allow(deprecated)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl SpecType {
    /// Get the required Mermaid diagram types for this spec type as enum values
    pub fn required_diagrams(&self) -> Vec<DiagramType> {
        match self {
            SpecType::HttpApi => vec![DiagramType::Sequence],
            SpecType::EventDriven => vec![DiagramType::Sequence],
            SpecType::DataModel => vec![DiagramType::Erd, DiagramType::Class],
            SpecType::Algorithm => vec![DiagramType::Flowchart, DiagramType::State],
            SpecType::Integration => vec![DiagramType::Sequence],
            SpecType::RpcApi => vec![DiagramType::Class],
            SpecType::Workflow => vec![DiagramType::State, DiagramType::Flowchart],
            SpecType::Utility => vec![],
        }
    }

    /// Get the required Mermaid diagram types as string representation
    pub fn required_diagrams_as_strings(&self) -> Vec<&'static str> {
        match self {
            SpecType::HttpApi => vec!["sequence"],
            SpecType::EventDriven => vec!["sequence"],
            SpecType::DataModel => vec!["erd", "class"],
            SpecType::Algorithm => vec!["flowchart", "state"],
            SpecType::Integration => vec!["sequence"],
            SpecType::RpcApi => vec!["class"],
            SpecType::Workflow => vec!["state", "flowchart"],
            SpecType::Utility => vec![],
        }
    }

    /// Get the required API specification format for this spec type
    pub fn required_api_spec(&self) -> Option<ApiSpecType> {
        match self {
            SpecType::HttpApi => Some(ApiSpecType::OpenApi31),
            SpecType::EventDriven => Some(ApiSpecType::AsyncApi26),
            SpecType::DataModel => Some(ApiSpecType::JsonSchema),
            SpecType::Algorithm => None,
            SpecType::Integration => None,
            SpecType::RpcApi => Some(ApiSpecType::OpenRpc13),
            SpecType::Workflow => Some(ApiSpecType::ServerlessWorkflow08),
            SpecType::Utility => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SpecType::HttpApi => "http-api",
            SpecType::EventDriven => "event-driven",
            SpecType::DataModel => "data-model",
            SpecType::Algorithm => "algorithm",
            SpecType::Integration => "integration",
            SpecType::RpcApi => "rpc-api",
            SpecType::Workflow => "workflow",
            SpecType::Utility => "utility",
        }
    }
}

#[allow(deprecated)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl FromStr for SpecType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http-api" => Ok(SpecType::HttpApi),
            "event-driven" => Ok(SpecType::EventDriven),
            "data-model" => Ok(SpecType::DataModel),
            "algorithm" => Ok(SpecType::Algorithm),
            "integration" => Ok(SpecType::Integration),
            "rpc-api" => Ok(SpecType::RpcApi),
            "workflow" => Ok(SpecType::Workflow),
            "utility" => Ok(SpecType::Utility),
            _ => Err(format!("Unknown spec type: {}", s)),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/spec_rules.md#source
impl SpecFormatRules {
    /// Get default rules for PRD documents
    pub fn prd_defaults() -> Self {
        Self {
            document_type: DocumentType::Prd,
            required_headings: vec![
                "Summary".to_string(),
                "Why".to_string(),
                "What Changes".to_string(),
                "Impact".to_string(),
            ],
            scenario_format: ScenarioFormat::MultiLine,
            scenario_heading_level: 3,
            scenario_heading_prefix: "Scenario:".to_string(),
            min_scenarios: 0, // PRD doesn't require scenarios
            when_keyword: "WHEN".to_string(),
            then_keyword: "THEN".to_string(),
            require_when_then: false,
            requirement_pattern: None,
        }
    }

    /// Get default rules for Spec documents
    pub fn spec_defaults() -> Self {
        Self {
            document_type: DocumentType::Spec,
            required_headings: vec!["Overview".to_string(), "Acceptance Criteria".to_string()],
            scenario_format: ScenarioFormat::MultiLine,
            scenario_heading_level: 3,
            scenario_heading_prefix: "Scenario:".to_string(),
            min_scenarios: 1, // At least one scenario required
            when_keyword: "WHEN".to_string(),
            then_keyword: "THEN".to_string(),
            require_when_then: true,
            requirement_pattern: None, // Allow flexible requirement headings
        }
    }

    /// Get default rules for Task documents
    pub fn task_defaults() -> Self {
        Self {
            document_type: DocumentType::Task,
            required_headings: vec![], // Flexible task structure
            scenario_format: ScenarioFormat::MultiLine,
            scenario_heading_level: 3,
            scenario_heading_prefix: "Scenario:".to_string(),
            min_scenarios: 0,
            when_keyword: "WHEN".to_string(),
            then_keyword: "THEN".to_string(),
            require_when_then: false,
            requirement_pattern: None,
        }
    }

    /// Get rules for a specific document type
    pub fn for_document_type(doc_type: DocumentType) -> Self {
        match doc_type {
            DocumentType::Prd => Self::prd_defaults(),
            DocumentType::Spec => Self::spec_defaults(),
            DocumentType::Task => Self::task_defaults(),
        }
    }

    /// Generate regex pattern for matching scenarios based on format
    pub fn scenario_regex_pattern(&self) -> String {
        let heading_hashes = "#".repeat(self.scenario_heading_level as usize);
        let prefix = &self.scenario_heading_prefix;

        match self.scenario_format {
            ScenarioFormat::SingleLine => {
                // Old format: WHEN...THEN on same line
                format!(
                    r"{}\s*{}\s+.*?{}.*?{}",
                    heading_hashes, prefix, self.when_keyword, self.then_keyword
                )
            }
            ScenarioFormat::MultiLine => {
                // New format: Support both explicit scenario headings AND inline WHEN/THEN bullets
                // Match either:
                // 1. ### Scenario: heading (new format - simple match, just check heading exists)
                // 2. - WHEN...THEN pattern (old/compact format - single line bullet)
                // Use (?m) for multiline mode to match ^ and $ at line boundaries
                format!(
                    r"(?m)^{}\s*{}|^-\s*{}[^\n]*{}",
                    heading_hashes, prefix, self.when_keyword, self.then_keyword
                )
            }
        }
    }

    /// Generate skeleton markdown template for this document type
    pub fn to_markdown_skeleton(&self) -> String {
        match self.document_type {
            DocumentType::Spec => self.spec_markdown_skeleton(),
            DocumentType::Prd => self.prd_markdown_skeleton(),
            DocumentType::Task => self.task_markdown_skeleton(),
        }
    }

    fn spec_markdown_skeleton(&self) -> String {
        let heading_hashes = "#".repeat(self.scenario_heading_level as usize);

        format!(
            r#"# Specification: [Feature Name]

## Overview
[Brief description of what this spec covers and why it exists]

## Requirements

### R1: [Requirement Title]
[Description of what this requirement must do]

### R2: [Another Requirement]
[Description]

## Acceptance Criteria

{} {prefix} [Scenario Name]
- **{when}** [condition or action]
- **{then}** [expected outcome]

{} {prefix} [Another Scenario]
- **{when}** [condition]
- **{then}** [result]

{} {prefix} [Edge Case Scenario]
- **{when}** [edge condition]
- **{then}** [expected behavior]
"#,
            heading_hashes,
            heading_hashes,
            heading_hashes,
            prefix = self.scenario_heading_prefix,
            when = self.when_keyword,
            then = self.then_keyword,
        )
    }

    fn prd_markdown_skeleton(&self) -> String {
        r#"# Change: [change-id]

## Summary
[1-2 sentence description of the change]

## Why
[Problem statement and business motivation]

## What Changes
- [List of concrete changes]
- [Group by area: API, UI, Database, etc.]

## Impact
- Affected specs: [list]
- Affected code: [file paths]
- Breaking changes: [Yes/No with explanation]
"#
        .to_string()
    }

    fn task_markdown_skeleton(&self) -> String {
        r#"# Tasks

## Layer: [layer name]

- [ ] [Task ID] [Task name]
  - File: `path/to/file` (CREATE/MODIFY/DELETE)
  - Spec: `spec-name#section`
  - Do: [Description of what to implement]
  - Depends: [dependencies]
"#
        .to_string()
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_rules_defaults() {
        let rules = SpecFormatRules::spec_defaults();

        assert_eq!(rules.document_type, DocumentType::Spec);
        assert_eq!(rules.scenario_format, ScenarioFormat::MultiLine);
        assert_eq!(rules.min_scenarios, 1);
        assert!(rules.require_when_then);
        assert_eq!(rules.when_keyword, "WHEN");
        assert_eq!(rules.then_keyword, "THEN");
    }

    #[test]
    fn test_scenario_regex_multiline() {
        use regex::Regex;

        let rules = SpecFormatRules::spec_defaults();
        let pattern = rules.scenario_regex_pattern();

        // Should match scenario heading
        assert!(pattern.contains("###"));
        assert!(pattern.contains("Scenario:"));

        // Test with actual content
        let content = r#"### Scenario: Add two positive integers
- **WHEN** calling `add(10, 5)`
- **THEN** the result should be `15`

### Scenario: Add negative numbers
- **WHEN** calling `add(-5, -3)`
- **THEN** the result should be `-8`"#;

        let regex = Regex::new(&pattern).expect("Invalid regex pattern");
        let matches: Vec<_> = regex.find_iter(content).collect();

        assert!(
            matches.len() >= 2,
            "Should find at least 2 scenarios, found {}: {:?}",
            matches.len(),
            matches
        );
    }

    #[test]
    fn test_markdown_skeleton_generation() {
        let rules = SpecFormatRules::spec_defaults();
        let skeleton = rules.to_markdown_skeleton();

        assert!(skeleton.contains("## Overview"));
        assert!(skeleton.contains("## Acceptance Criteria"));
        assert!(skeleton.contains("### Scenario:"));
        assert!(skeleton.contains("**WHEN**"));
        assert!(skeleton.contains("**THEN**"));
    }

    #[test]
    fn test_document_type_specific_rules() {
        let prd = SpecFormatRules::for_document_type(DocumentType::Prd);
        let spec = SpecFormatRules::for_document_type(DocumentType::Spec);
        let task = SpecFormatRules::for_document_type(DocumentType::Task);

        assert_eq!(prd.min_scenarios, 0);
        assert_eq!(spec.min_scenarios, 1);
        assert_eq!(task.min_scenarios, 0);

        assert!(!prd.require_when_then);
        assert!(spec.require_when_then);
        assert!(!task.require_when_then);
    }

    // SpecType enforcement tests
    #[test]
    fn test_spec_type_required_diagrams_http_api() {
        let required = SpecType::HttpApi.required_diagrams();
        assert!(
            required.contains(&DiagramType::Sequence),
            "http-api requires sequence diagram"
        );
    }

    #[test]
    fn test_spec_type_required_diagrams_data_model() {
        let required = SpecType::DataModel.required_diagrams();
        assert!(
            required.contains(&DiagramType::Erd) || required.contains(&DiagramType::Class),
            "data-model requires ERD or class diagram"
        );
    }

    #[test]
    fn test_spec_type_required_api_spec_http_api() {
        let api_spec = SpecType::HttpApi.required_api_spec();
        assert_eq!(
            api_spec,
            Some(ApiSpecType::OpenApi31),
            "http-api requires OpenAPI 3.1"
        );
    }

    #[test]
    fn test_spec_type_required_api_spec_event_driven() {
        let api_spec = SpecType::EventDriven.required_api_spec();
        assert_eq!(
            api_spec,
            Some(ApiSpecType::AsyncApi26),
            "event-driven requires AsyncAPI 2.6"
        );
    }

    #[test]
    fn test_spec_type_utility_no_requirements() {
        let diagrams = SpecType::Utility.required_diagrams();
        let api_spec = SpecType::Utility.required_api_spec();
        assert!(diagrams.is_empty(), "utility requires no diagrams");
        assert_eq!(api_spec, None, "utility requires no API spec");
    }

    #[test]
    fn test_spec_type_from_str() {
        assert_eq!(SpecType::from_str("http-api").unwrap(), SpecType::HttpApi);
        assert_eq!(
            SpecType::from_str("event-driven").unwrap(),
            SpecType::EventDriven
        );
        assert_eq!(
            SpecType::from_str("data-model").unwrap(),
            SpecType::DataModel
        );
        assert_eq!(
            SpecType::from_str("algorithm").unwrap(),
            SpecType::Algorithm
        );
        assert_eq!(
            SpecType::from_str("integration").unwrap(),
            SpecType::Integration
        );
        assert_eq!(SpecType::from_str("rpc-api").unwrap(), SpecType::RpcApi);
        assert_eq!(SpecType::from_str("workflow").unwrap(), SpecType::Workflow);
        assert_eq!(SpecType::from_str("utility").unwrap(), SpecType::Utility);
        assert!(SpecType::from_str("invalid").is_err());
    }

    #[test]
    #[allow(deprecated)]
    fn test_spec_type_as_str() {
        assert_eq!(SpecType::HttpApi.as_str(), "http-api");
        assert_eq!(SpecType::EventDriven.as_str(), "event-driven");
        assert_eq!(SpecType::DataModel.as_str(), "data-model");
        assert_eq!(SpecType::Algorithm.as_str(), "algorithm");
        assert_eq!(SpecType::Integration.as_str(), "integration");
        assert_eq!(SpecType::RpcApi.as_str(), "rpc-api");
        assert_eq!(SpecType::Workflow.as_str(), "workflow");
        assert_eq!(SpecType::Utility.as_str(), "utility");
    }

    #[test]
    fn test_api_spec_type_from_str() {
        assert_eq!(
            ApiSpecType::from_str("openapi-3.1").unwrap(),
            ApiSpecType::OpenApi31
        );
        assert_eq!(
            ApiSpecType::from_str("asyncapi-2.6").unwrap(),
            ApiSpecType::AsyncApi26
        );
        assert_eq!(
            ApiSpecType::from_str("json-schema").unwrap(),
            ApiSpecType::JsonSchema
        );
        assert_eq!(
            ApiSpecType::from_str("openrpc-1.3").unwrap(),
            ApiSpecType::OpenRpc13
        );
        assert_eq!(
            ApiSpecType::from_str("serverless-workflow-0.8").unwrap(),
            ApiSpecType::ServerlessWorkflow08
        );
        assert!(ApiSpecType::from_str("invalid").is_err());
    }

    // ─── SectionType tests ──────────────────────────────────────────────────

    #[test]
    fn test_section_type_count() {
        // Updated as section types are added. Change together with
        // `AUTHORING.md`'s "Section Type Registry" header count.
        // 21 → +Manifest → +UnitTest/+E2eTest → +RuntimeImage/+Deployment = 25.
        assert_eq!(SectionType::all_in_fill_order().len(), 25);
    }

    #[test]
    fn test_section_type_fill_order_sorted() {
        let types = SectionType::all_in_fill_order();
        for window in types.windows(2) {
            assert!(
                window[0].fill_order() <= window[1].fill_order(),
                "fill_order not sorted: {:?} > {:?}",
                window[0],
                window[1]
            );
        }
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/spec-structure.md#R1
    #[test]
    fn test_fill_order_requirements_before_schema() {
        // R1: Top-down fill order — requirements before data/API sections
        assert!(
            SectionType::Requirements.fill_order() < SectionType::Schema.fill_order(),
            "requirements should fill before schema"
        );
        assert!(
            SectionType::Scenarios.fill_order() < SectionType::Schema.fill_order(),
            "scenarios should fill before schema"
        );
        assert!(
            SectionType::Requirements.fill_order() < SectionType::DbModel.fill_order(),
            "requirements should fill before db-model"
        );
        assert!(
            SectionType::Requirements.fill_order() < SectionType::RestApi.fill_order(),
            "requirements should fill before rest-api"
        );
        // unit-test fills after all diagrams and data
        assert!(
            SectionType::UnitTest.fill_order() > SectionType::Schema.fill_order(),
            "unit-test should fill after schema"
        );
        assert!(
            SectionType::Manifest.fill_order() < SectionType::RuntimeImage.fill_order(),
            "manifest should fill before runtime-image"
        );
        assert!(
            SectionType::RuntimeImage.fill_order() < SectionType::Deployment.fill_order(),
            "runtime-image should fill before deployment"
        );
        // changes and doc are last
        assert!(
            SectionType::E2eTest.fill_order() < SectionType::Changes.fill_order(),
            "e2e-test should fill before changes"
        );
        assert!(
            SectionType::Changes.fill_order() < SectionType::Doc.fill_order(),
            "changes should fill before doc"
        );
    }

    #[test]
    fn test_section_type_overview_first() {
        let types = SectionType::all_in_fill_order();
        assert_eq!(types[0], SectionType::Overview);
    }

    #[test]
    fn test_section_type_from_str_roundtrip() {
        for st in SectionType::all_in_fill_order() {
            let s = st.as_str();
            let parsed = SectionType::from_str(s).expect("failed to parse");
            assert_eq!(parsed, st, "roundtrip failed for {:?}", st);
        }
    }

    #[test]
    fn test_section_type_from_str_invalid() {
        assert!(SectionType::from_str("invalid-type").is_err());
    }

    #[test]
    fn test_operations_section_type_aliases() {
        assert_eq!(
            SectionType::from_str("dockerfile").unwrap(),
            SectionType::RuntimeImage
        );
        assert_eq!(
            SectionType::from_str("container").unwrap(),
            SectionType::RuntimeImage
        );
        assert_eq!(
            SectionType::from_str("kustomize").unwrap(),
            SectionType::Deployment
        );
        assert_eq!(
            SectionType::from_str("k8s").unwrap(),
            SectionType::Deployment
        );
    }

    #[test]
    fn test_legacy_test_section_aliases_parse_to_unit_test() {
        assert_eq!(
            SectionType::from_str("test-plan").unwrap(),
            SectionType::UnitTest
        );
        assert_eq!(
            SectionType::from_str("tests").unwrap(),
            SectionType::UnitTest
        );
    }

    #[test]
    fn test_section_type_default_lang() {
        // Prose-only sections
        assert_eq!(SectionType::Overview.default_lang(), "markdown");
        assert_eq!(SectionType::Doc.default_lang(), "markdown");
        // YAML sections (structured data — no JSON)
        assert_eq!(SectionType::Changes.default_lang(), "yaml");
        assert_eq!(SectionType::RestApi.default_lang(), "yaml");
        assert_eq!(SectionType::RpcApi.default_lang(), "yaml"); // was "json"
        assert_eq!(SectionType::Schema.default_lang(), "yaml"); // was "json"
        assert_eq!(SectionType::Config.default_lang(), "yaml"); // was "json"
        assert_eq!(SectionType::Component.default_lang(), "yaml"); // was "json"
        assert_eq!(SectionType::DesignToken.default_lang(), "yaml"); // was "json"
        assert_eq!(SectionType::Scenarios.default_lang(), "yaml"); // was "markdown"
        assert_eq!(SectionType::Wireframe.default_lang(), "yaml");
        assert_eq!(SectionType::RuntimeImage.default_lang(), "yaml");
        assert_eq!(SectionType::Deployment.default_lang(), "yaml");
        // Mermaid sections (diagrams + requirements + unit-test)
        assert_eq!(SectionType::Interaction.default_lang(), "mermaid");
        assert_eq!(SectionType::Requirements.default_lang(), "mermaid"); // was "markdown"
        assert_eq!(SectionType::UnitTest.default_lang(), "mermaid");
        assert_eq!(SectionType::E2eTest.default_lang(), "yaml");
    }

    // ─── SectionEntry tests ──────────────────────────────────────────────────

    #[test]
    fn test_section_entry_required_is_not_optional() {
        let entry = SectionEntry::required(SectionType::Overview);
        assert_eq!(entry.section_type(), SectionType::Overview);
        assert!(!entry.is_optional());
    }

    #[test]
    fn test_section_entry_optional_is_optional() {
        let entry = SectionEntry::optional(SectionType::Component);
        assert_eq!(entry.section_type(), SectionType::Component);
        assert!(entry.is_optional());
    }

    #[test]
    fn test_section_entry_with_optional_false_is_not_optional() {
        let entry = SectionEntry::WithOptional {
            section_type: SectionType::DesignToken,
            optional: false,
        };
        assert_eq!(entry.section_type(), SectionType::DesignToken);
        assert!(!entry.is_optional());
    }

    #[test]
    fn test_section_entry_to_fill_section_string_required() {
        let entry = SectionEntry::required(SectionType::Overview);
        assert_eq!(entry.to_fill_section_string(), "overview");
    }

    #[test]
    fn test_section_entry_to_fill_section_string_optional() {
        let entry = SectionEntry::optional(SectionType::Component);
        assert_eq!(entry.to_fill_section_string(), "component (optional)");
    }

    #[test]
    fn test_section_entry_to_fill_section_string_design_token_optional() {
        let entry = SectionEntry::optional(SectionType::DesignToken);
        assert_eq!(entry.to_fill_section_string(), "design-token (optional)");
    }

    #[test]
    fn test_section_entry_from_fill_section_string_required() {
        let entry = SectionEntry::from_fill_section_string("overview").unwrap();
        assert_eq!(entry.section_type(), SectionType::Overview);
        assert!(!entry.is_optional());
    }

    #[test]
    fn test_section_entry_from_fill_section_string_optional() {
        let entry = SectionEntry::from_fill_section_string("component (optional)").unwrap();
        assert_eq!(entry.section_type(), SectionType::Component);
        assert!(entry.is_optional());
    }

    #[test]
    fn test_section_entry_from_fill_section_string_design_token_optional() {
        let entry = SectionEntry::from_fill_section_string("design-token (optional)").unwrap();
        assert_eq!(entry.section_type(), SectionType::DesignToken);
        assert!(entry.is_optional());
    }

    #[test]
    fn test_section_entry_from_fill_section_string_with_whitespace() {
        let entry = SectionEntry::from_fill_section_string("  wireframe  ").unwrap();
        assert_eq!(entry.section_type(), SectionType::Wireframe);
        assert!(!entry.is_optional());
    }

    #[test]
    fn test_section_entry_from_fill_section_string_invalid() {
        let result = SectionEntry::from_fill_section_string("nonexistent-type");
        assert!(result.is_err());
    }

    #[test]
    fn test_section_entry_roundtrip_required() {
        let original = SectionEntry::required(SectionType::Logic);
        let serialized = original.to_fill_section_string();
        let parsed = SectionEntry::from_fill_section_string(&serialized).unwrap();
        assert_eq!(parsed.section_type(), original.section_type());
        assert_eq!(parsed.is_optional(), original.is_optional());
    }

    #[test]
    fn test_section_entry_roundtrip_optional() {
        let original = SectionEntry::optional(SectionType::Component);
        let serialized = original.to_fill_section_string();
        let parsed = SectionEntry::from_fill_section_string(&serialized).unwrap();
        assert_eq!(parsed.section_type(), original.section_type());
        assert_eq!(parsed.is_optional(), original.is_optional());
    }

    #[test]
    fn test_section_entry_serde_required_as_plain_string() {
        // Required variant serializes as plain string via serde
        let entry = SectionEntry::Required(SectionType::Overview);
        let json = serde_json::to_string(&entry).unwrap();
        assert_eq!(json, "\"overview\"");
        let parsed: SectionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.section_type(), SectionType::Overview);
        assert!(!parsed.is_optional());
    }

    #[test]
    fn test_section_entry_serde_optional_as_object() {
        // WithOptional variant serializes as object via serde
        let entry = SectionEntry::optional(SectionType::Component);
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: SectionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.section_type(), SectionType::Component);
        assert!(parsed.is_optional());
    }

    // ─── apply_section_optionality tests ─────────────────────────────────────

    #[test]
    fn test_optionality_no_design_system_all_required() {
        let sections = vec![
            SectionType::Overview,
            SectionType::Wireframe,
            SectionType::Component,
            SectionType::DesignToken,
            SectionType::Changes,
        ];
        let result = apply_section_optionality(sections, None);
        for entry in &result {
            assert!(
                !entry.is_optional(),
                "{:?} should be required when no design system",
                entry.section_type()
            );
        }
    }

    #[test]
    fn test_optionality_provides_tokens_true_marks_design_token_optional() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "chakra".to_string(),
            provides_tokens: true,
            provides_components: false,
        };
        let sections = vec![
            SectionType::Overview,
            SectionType::Component,
            SectionType::DesignToken,
            SectionType::Changes,
        ];
        let result = apply_section_optionality(sections, Some(&ds));

        // design-token should be optional
        let dt = result
            .iter()
            .find(|e| e.section_type() == SectionType::DesignToken)
            .unwrap();
        assert!(
            dt.is_optional(),
            "design-token should be optional when provides_tokens=true"
        );

        // component should remain required
        let comp = result
            .iter()
            .find(|e| e.section_type() == SectionType::Component)
            .unwrap();
        assert!(
            !comp.is_optional(),
            "component should be required when provides_components=false"
        );
    }

    #[test]
    fn test_optionality_provides_components_true_marks_component_optional() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "antd".to_string(),
            provides_tokens: false,
            provides_components: true,
        };
        let sections = vec![
            SectionType::Overview,
            SectionType::Component,
            SectionType::DesignToken,
            SectionType::Changes,
        ];
        let result = apply_section_optionality(sections, Some(&ds));

        // component should be optional
        let comp = result
            .iter()
            .find(|e| e.section_type() == SectionType::Component)
            .unwrap();
        assert!(
            comp.is_optional(),
            "component should be optional when provides_components=true"
        );

        // design-token should remain required
        let dt = result
            .iter()
            .find(|e| e.section_type() == SectionType::DesignToken)
            .unwrap();
        assert!(
            !dt.is_optional(),
            "design-token should be required when provides_tokens=false"
        );
    }

    #[test]
    fn test_optionality_both_true_marks_both_optional() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let sections = vec![
            SectionType::Overview,
            SectionType::Wireframe,
            SectionType::Component,
            SectionType::DesignToken,
            SectionType::Changes,
        ];
        let result = apply_section_optionality(sections, Some(&ds));

        let comp = result
            .iter()
            .find(|e| e.section_type() == SectionType::Component)
            .unwrap();
        assert!(
            comp.is_optional(),
            "component should be optional when provides_components=true"
        );

        let dt = result
            .iter()
            .find(|e| e.section_type() == SectionType::DesignToken)
            .unwrap();
        assert!(
            dt.is_optional(),
            "design-token should be optional when provides_tokens=true"
        );

        // wireframe should remain required (not affected by design system)
        let wf = result
            .iter()
            .find(|e| e.section_type() == SectionType::Wireframe)
            .unwrap();
        assert!(!wf.is_optional(), "wireframe should remain required");
    }

    #[test]
    fn test_optionality_overview_never_optional() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let sections = vec![SectionType::Overview, SectionType::Changes];
        let result = apply_section_optionality(sections, Some(&ds));

        let overview = result
            .iter()
            .find(|e| e.section_type() == SectionType::Overview)
            .unwrap();
        assert!(!overview.is_optional(), "overview must never be optional");
    }

    #[test]
    fn test_optionality_legacy_changes_remains_required_when_present() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let sections = vec![SectionType::Overview, SectionType::Changes];
        let result = apply_section_optionality(sections, Some(&ds));

        let changes = result
            .iter()
            .find(|e| e.section_type() == SectionType::Changes)
            .unwrap();
        assert!(
            !changes.is_optional(),
            "legacy changes stays required when an older caller includes it"
        );
    }

    #[test]
    fn test_optionality_non_design_sections_always_required() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let sections = vec![
            SectionType::Logic,
            SectionType::RestApi,
            SectionType::Schema,
            SectionType::StateMachine,
            SectionType::Interaction,
        ];
        let result = apply_section_optionality(sections, Some(&ds));

        for entry in &result {
            assert!(
                !entry.is_optional(),
                "{:?} should remain required (not design-token or component)",
                entry.section_type()
            );
        }
    }

    #[test]
    fn test_optionality_empty_sections_returns_empty() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let result = apply_section_optionality(vec![], Some(&ds));
        assert!(result.is_empty());
    }

    #[test]
    fn test_optionality_preserves_section_order() {
        use crate::models::tech_stack::DesignSystem;
        let ds = DesignSystem {
            library: "mui".to_string(),
            provides_tokens: true,
            provides_components: true,
        };
        let sections = vec![
            SectionType::Overview,
            SectionType::Wireframe,
            SectionType::Component,
            SectionType::DesignToken,
            SectionType::Changes,
        ];
        let result = apply_section_optionality(sections.clone(), Some(&ds));

        let result_types: Vec<SectionType> = result.iter().map(|e| e.section_type()).collect();
        assert_eq!(result_types, sections, "order must be preserved");
    }

    // ─── parse_fill_section_str tests ────────────────────────────────────────

    #[test]
    fn test_parse_fill_section_str_required() {
        let (name, optional) = parse_fill_section_str("overview");
        assert_eq!(name, "overview");
        assert!(!optional);
    }

    #[test]
    fn test_parse_fill_section_str_optional() {
        let (name, optional) = parse_fill_section_str("component (optional)");
        assert_eq!(name, "component");
        assert!(optional);
    }

    #[test]
    fn test_parse_fill_section_str_design_token_optional() {
        let (name, optional) = parse_fill_section_str("design-token (optional)");
        assert_eq!(name, "design-token");
        assert!(optional);
    }

    #[test]
    fn test_parse_fill_section_str_with_surrounding_whitespace() {
        let (name, optional) = parse_fill_section_str("  wireframe  ");
        assert_eq!(name, "wireframe");
        assert!(!optional);
    }

    #[test]
    fn test_parse_fill_section_str_changes_never_optional_format() {
        let (name, optional) = parse_fill_section_str("changes");
        assert_eq!(name, "changes");
        assert!(!optional);
    }
}

// CODEGEN-END

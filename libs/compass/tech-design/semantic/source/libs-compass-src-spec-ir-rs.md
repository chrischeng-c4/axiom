---
id: libs-compass-src-spec-ir-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/spec/ir.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/spec/ir.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/spec/ir.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SpecIR` | libs/compass/src/spec/ir.rs | enum | pub | 10 | pub enum SpecIR { |
| `DataModelSpec` | libs/compass/src/spec/ir.rs | struct | pub | 25 | pub struct DataModelSpec { |
| `ModelDef` | libs/compass/src/spec/ir.rs | struct | pub | 36 | pub struct ModelDef { |
| `FieldDef` | libs/compass/src/spec/ir.rs | struct | pub | 75 | pub struct FieldDef { |
| `FieldConstraints` | libs/compass/src/spec/ir.rs | struct | pub | 123 | pub struct FieldConstraints { |
| `StringFormat` | libs/compass/src/spec/ir.rs | enum | pub | 145 | pub enum StringFormat { |
| `ForeignKey` | libs/compass/src/spec/ir.rs | struct | pub | 164 | pub struct ForeignKey { |
| `ForeignKeyAction` | libs/compass/src/spec/ir.rs | enum | pub | 177 | pub enum ForeignKeyAction { |
| `MethodDef` | libs/compass/src/spec/ir.rs | struct | pub | 188 | pub struct MethodDef { |
| `ParamDef` | libs/compass/src/spec/ir.rs | struct | pub | 207 | pub struct ParamDef { |
| `Visibility` | libs/compass/src/spec/ir.rs | enum | pub | 215 | pub enum Visibility { |
| `TypeParam` | libs/compass/src/spec/ir.rs | struct | pub | 224 | pub struct TypeParam { |
| `EnumDef` | libs/compass/src/spec/ir.rs | struct | pub | 232 | pub struct EnumDef { |
| `EnumVariant` | libs/compass/src/spec/ir.rs | struct | pub | 240 | pub struct EnumVariant { |
| `EnumValue` | libs/compass/src/spec/ir.rs | enum | pub | 248 | pub enum EnumValue { |
| `Relationship` | libs/compass/src/spec/ir.rs | struct | pub | 255 | pub struct Relationship { |
| `RelationType` | libs/compass/src/spec/ir.rs | enum | pub | 270 | pub enum RelationType { |
| `RestApiSpec` | libs/compass/src/spec/ir.rs | struct | pub | 287 | pub struct RestApiSpec { |
| `ServerDef` | libs/compass/src/spec/ir.rs | struct | pub | 306 | pub struct ServerDef { |
| `EndpointDef` | libs/compass/src/spec/ir.rs | struct | pub | 313 | pub struct EndpointDef { |
| `HttpMethod` | libs/compass/src/spec/ir.rs | enum | pub | 342 | pub enum HttpMethod { |
| `QueryParam` | libs/compass/src/spec/ir.rs | struct | pub | 368 | pub struct QueryParam { |
| `RequestBody` | libs/compass/src/spec/ir.rs | struct | pub | 378 | pub struct RequestBody { |
| `ResponseDef` | libs/compass/src/spec/ir.rs | struct | pub | 391 | pub struct ResponseDef { |
| `SecurityScheme` | libs/compass/src/spec/ir.rs | struct | pub | 404 | pub struct SecurityScheme { |
| `SecuritySchemeType` | libs/compass/src/spec/ir.rs | enum | pub | 412 | pub enum SecuritySchemeType { |
| `EventApiSpec` | libs/compass/src/spec/ir.rs | struct | pub | 435 | pub struct EventApiSpec { |
| `ChannelDef` | libs/compass/src/spec/ir.rs | struct | pub | 450 | pub struct ChannelDef { |
| `OperationDef` | libs/compass/src/spec/ir.rs | struct | pub | 463 | pub struct OperationDef { |
| `StateMachineSpec` | libs/compass/src/spec/ir.rs | struct | pub | 477 | pub struct StateMachineSpec { |
| `StateDef` | libs/compass/src/spec/ir.rs | struct | pub | 492 | pub struct StateDef { |
| `TransitionDef` | libs/compass/src/spec/ir.rs | struct | pub | 505 | pub struct TransitionDef { |
| `ControlFlowSpec` | libs/compass/src/spec/ir.rs | struct | pub | 519 | pub struct ControlFlowSpec { |
| `FlowNode` | libs/compass/src/spec/ir.rs | struct | pub | 530 | pub struct FlowNode { |
| `FlowNodeType` | libs/compass/src/spec/ir.rs | enum | pub | 538 | pub enum FlowNodeType { |
| `FlowEdge` | libs/compass/src/spec/ir.rs | struct | pub | 550 | pub struct FlowEdge { |
| `new` | libs/compass/src/spec/ir.rs | function | pub | 563 | pub fn new() -> Self { |
| `add_model` | libs/compass/src/spec/ir.rs | function | pub | 567 | pub fn add_model(&mut self, model: ModelDef) { |
| `add_enum` | libs/compass/src/spec/ir.rs | function | pub | 571 | pub fn add_enum(&mut self, enum_def: EnumDef) { |
| `add_relationship` | libs/compass/src/spec/ir.rs | function | pub | 575 | pub fn add_relationship(&mut self, rel: Relationship) { |
| `get_model` | libs/compass/src/spec/ir.rs | function | pub | 580 | pub fn get_model(&self, name: &str) -> Option<&ModelDef> { |
| `get_enum` | libs/compass/src/spec/ir.rs | function | pub | 585 | pub fn get_enum(&self, name: &str) -> Option<&EnumDef> { |
| `with_description` | libs/compass/src/spec/ir.rs | function | pub | 598 | pub fn with_description(mut self, desc: impl Into<String>) -> Self { |
| `add_field` | libs/compass/src/spec/ir.rs | function | pub | 603 | pub fn add_field(&mut self, field: FieldDef) { |
| `add_method` | libs/compass/src/spec/ir.rs | function | pub | 607 | pub fn add_method(&mut self, method: MethodDef) { |
| `optional` | libs/compass/src/spec/ir.rs | function | pub | 621 | pub fn optional(mut self) -> Self { |
| `with_default` | libs/compass/src/spec/ir.rs | function | pub | 626 | pub fn with_default(mut self, default: impl Into<String>) -> Self { |
| `primary_key` | libs/compass/src/spec/ir.rs | function | pub | 637 | pub fn primary_key(mut self) -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Intermediate representation for spec-to-code generation
//!
//! SpecIR serves as the common representation between spec parsers
//! (JSON Schema, OpenAPI, AsyncAPI, Mermaid) and code generators.

use crate::type_inference::Type;

/// Main SpecIR enum representing different spec types
#[derive(Debug, Clone)]
pub enum SpecIR {
    /// Data model specification (from JSON Schema, Mermaid classDiagram/ERD)
    DataModel(DataModelSpec),
    /// REST API specification (from OpenAPI)
    RestApi(RestApiSpec),
    /// Event-driven API specification (from AsyncAPI)
    EventApi(EventApiSpec),
    /// State machine specification (from Mermaid stateDiagram)
    StateMachine(StateMachineSpec),
    /// Control flow specification (from Mermaid flowchart)
    ControlFlow(ControlFlowSpec),
}

/// Data model specification containing models, enums, and relationships
#[derive(Debug, Clone, Default)]
pub struct DataModelSpec {
    /// Model definitions
    pub models: Vec<ModelDef>,
    /// Enum definitions
    pub enums: Vec<EnumDef>,
    /// Relationships between models (for ORM generation)
    pub relationships: Vec<Relationship>,
}

/// Model definition (class/struct/interface)
#[derive(Debug, Clone)]
pub struct ModelDef {
    /// Model name (PascalCase)
    pub name: String,
    /// Optional description/documentation
    pub description: Option<String>,
    /// Field definitions
    pub fields: Vec<FieldDef>,
    /// Method definitions (from Mermaid classDiagram)
    pub methods: Vec<MethodDef>,
    /// Base classes/interfaces this model extends
    pub extends: Vec<String>,
    /// Generic type parameters
    pub type_params: Vec<TypeParam>,
    /// Whether this is an abstract class/protocol
    pub is_abstract: bool,
    /// Database table name (for ORM generation)
    pub table_name: Option<String>,
    /// Collection name (for MongoDB)
    pub collection_name: Option<String>,
}

impl Default for ModelDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            fields: Vec::new(),
            methods: Vec::new(),
            extends: Vec::new(),
            type_params: Vec::new(),
            is_abstract: false,
            table_name: None,
            collection_name: None,
        }
    }
}

/// Field definition with type, constraints, and metadata
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Field name (snake_case)
    pub name: String,
    /// Field type (uses existing Type IR)
    pub ty: Type,
    /// Whether the field is required
    pub required: bool,
    /// Default value expression (as string)
    pub default: Option<String>,
    /// Field description
    pub description: Option<String>,
    /// Field constraints
    pub constraints: FieldConstraints,
    /// Database column name (if different from field name)
    pub column_name: Option<String>,
    /// Whether this is a primary key
    pub primary_key: bool,
    /// Whether this field has a unique constraint
    pub unique: bool,
    /// Whether this field is indexed
    pub indexed: bool,
    /// Foreign key reference
    pub foreign_key: Option<ForeignKey>,
    /// Alias for serialization
    pub alias: Option<String>,
}

impl Default for FieldDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            ty: Type::Any,
            required: true,
            default: None,
            description: None,
            constraints: FieldConstraints::default(),
            column_name: None,
            primary_key: false,
            unique: false,
            indexed: false,
            foreign_key: None,
            alias: None,
        }
    }
}

/// Field constraints for validation
#[derive(Debug, Clone, Default)]
pub struct FieldConstraints {
    // String constraints
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub format: Option<StringFormat>,

    // Numeric constraints
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub exclusive_minimum: Option<f64>,
    pub exclusive_maximum: Option<f64>,
    pub multiple_of: Option<f64>,

    // Array constraints
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub unique_items: bool,
}

/// String format types
#[derive(Debug, Clone, PartialEq)]
pub enum StringFormat {
    Email,
    Uri,
    Url,
    Uuid,
    DateTime,
    Date,
    Time,
    Duration,
    Hostname,
    Ipv4,
    Ipv6,
    Regex,
    JsonPointer,
    Custom(String),
}

/// Foreign key reference
#[derive(Debug, Clone)]
pub struct ForeignKey {
    /// Referenced model name
    pub model: String,
    /// Referenced field name
    pub field: String,
    /// On delete action
    pub on_delete: ForeignKeyAction,
    /// On update action
    pub on_update: ForeignKeyAction,
}

/// Foreign key actions
#[derive(Debug, Clone, Default)]
pub enum ForeignKeyAction {
    #[default]
    NoAction,
    Cascade,
    SetNull,
    SetDefault,
    Restrict,
}

/// Method definition (from Mermaid classDiagram)
#[derive(Debug, Clone)]
pub struct MethodDef {
    /// Method name
    pub name: String,
    /// Method parameters
    pub params: Vec<ParamDef>,
    /// Return type
    pub return_type: Type,
    /// Visibility
    pub visibility: Visibility,
    /// Whether this is a static method
    pub is_static: bool,
    /// Whether this is an async method
    pub is_async: bool,
    /// Method description
    pub description: Option<String>,
}

/// Parameter definition
#[derive(Debug, Clone)]
pub struct ParamDef {
    pub name: String,
    pub ty: Type,
    pub default: Option<String>,
}

/// Visibility modifier
#[derive(Debug, Clone, Default)]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Protected,
}

/// Generic type parameter
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: String,
    pub bound: Option<Type>,
    pub default: Option<Type>,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub description: Option<String>,
    pub variants: Vec<EnumVariant>,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<EnumValue>,
    pub description: Option<String>,
}

/// Enum value type
#[derive(Debug, Clone)]
pub enum EnumValue {
    Int(i64),
    String(String),
}

/// Relationship between models (for ERD/ORM)
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Source model name
    pub from_model: String,
    /// Source field name
    pub from_field: String,
    /// Target model name
    pub to_model: String,
    /// Target field name (usually 'id')
    pub to_field: String,
    /// Relationship type
    pub rel_type: RelationType,
}

/// Relationship cardinality
#[derive(Debug, Clone)]
pub enum RelationType {
    /// 1:1 relationship
    OneToOne,
    /// 1:N relationship
    OneToMany,
    /// N:1 relationship
    ManyToOne,
    /// N:M relationship
    ManyToMany,
}

// ============================================================================
// REST API Specification (from OpenAPI)
// ============================================================================

/// REST API specification
#[derive(Debug, Clone, Default)]
pub struct RestApiSpec {
    /// API title
    pub title: String,
    /// API version
    pub version: String,
    /// API description
    pub description: Option<String>,
    /// Base URL/server
    pub servers: Vec<ServerDef>,
    /// API endpoints
    pub endpoints: Vec<EndpointDef>,
    /// Shared schemas (referenced by endpoints)
    pub schemas: DataModelSpec,
    /// Security schemes
    pub security_schemes: Vec<SecurityScheme>,
}

/// Server definition
#[derive(Debug, Clone)]
pub struct ServerDef {
    pub url: String,
    pub description: Option<String>,
}

/// API endpoint definition
#[derive(Debug, Clone)]
pub struct EndpointDef {
    /// HTTP path (e.g., "/users/{id}")
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Operation ID (for function naming)
    pub operation_id: Option<String>,
    /// Summary
    pub summary: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Tags for grouping
    pub tags: Vec<String>,
    /// Path parameters
    pub path_params: Vec<ParamDef>,
    /// Query parameters
    pub query_params: Vec<QueryParam>,
    /// Request body
    pub request_body: Option<RequestBody>,
    /// Responses
    pub responses: Vec<ResponseDef>,
    /// Security requirements
    pub security: Vec<String>,
    /// Whether endpoint is deprecated
    pub deprecated: bool,
}

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

/// Query parameter
#[derive(Debug, Clone)]
pub struct QueryParam {
    pub name: String,
    pub ty: Type,
    pub required: bool,
    pub description: Option<String>,
    pub default: Option<String>,
}

/// Request body definition
#[derive(Debug, Clone)]
pub struct RequestBody {
    /// Content type (e.g., "application/json")
    pub content_type: String,
    /// Body schema (model name or inline type)
    pub schema: Type,
    /// Whether body is required
    pub required: bool,
    /// Description
    pub description: Option<String>,
}

/// Response definition
#[derive(Debug, Clone)]
pub struct ResponseDef {
    /// HTTP status code
    pub status_code: u16,
    /// Description
    pub description: String,
    /// Response body schema
    pub schema: Option<Type>,
    /// Content type
    pub content_type: Option<String>,
}

/// Security scheme
#[derive(Debug, Clone)]
pub struct SecurityScheme {
    pub name: String,
    pub scheme_type: SecuritySchemeType,
    pub description: Option<String>,
}

/// Security scheme types
#[derive(Debug, Clone)]
pub enum SecuritySchemeType {
    ApiKey {
        in_header: bool,
        key_name: String,
    },
    Http {
        scheme: String,
        bearer_format: Option<String>,
    },
    OAuth2 {
        flows: Vec<String>,
    },
    OpenIdConnect {
        url: String,
    },
}

// ============================================================================
// Event API Specification (from AsyncAPI)
// ============================================================================

/// Event-driven API specification
#[derive(Debug, Clone, Default)]
pub struct EventApiSpec {
    /// API title
    pub title: String,
    /// API version
    pub version: String,
    /// Description
    pub description: Option<String>,
    /// Channels (topics/queues)
    pub channels: Vec<ChannelDef>,
    /// Message schemas
    pub messages: DataModelSpec,
}

/// Channel (topic/queue) definition
#[derive(Debug, Clone)]
pub struct ChannelDef {
    /// Channel name/path
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Subscribe operation (consuming messages)
    pub subscribe: Option<OperationDef>,
    /// Publish operation (producing messages)
    pub publish: Option<OperationDef>,
}

/// Channel operation definition
#[derive(Debug, Clone)]
pub struct OperationDef {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    /// Message schema
    pub message: Type,
}

// ============================================================================
// State Machine Specification (from Mermaid stateDiagram)
// ============================================================================

/// State machine specification
#[derive(Debug, Clone, Default)]
pub struct StateMachineSpec {
    /// State machine name
    pub name: String,
    /// States
    pub states: Vec<StateDef>,
    /// Transitions
    pub transitions: Vec<TransitionDef>,
    /// Initial state
    pub initial_state: Option<String>,
    /// Final states
    pub final_states: Vec<String>,
}

/// State definition
#[derive(Debug, Clone)]
pub struct StateDef {
    pub name: String,
    pub description: Option<String>,
    /// Entry action
    pub on_enter: Option<String>,
    /// Exit action
    pub on_exit: Option<String>,
    /// Nested state machine
    pub nested: Option<Box<StateMachineSpec>>,
}

/// State transition
#[derive(Debug, Clone)]
pub struct TransitionDef {
    pub from: String,
    pub to: String,
    pub event: Option<String>,
    pub guard: Option<String>,
    pub action: Option<String>,
}

// ============================================================================
// Control Flow Specification (from Mermaid flowchart)
// ============================================================================

/// Control flow specification
#[derive(Debug, Clone, Default)]
pub struct ControlFlowSpec {
    /// Flow name
    pub name: String,
    /// Nodes
    pub nodes: Vec<FlowNode>,
    /// Edges
    pub edges: Vec<FlowEdge>,
}

/// Flow node types
#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub node_type: FlowNodeType,
}

/// Flow node type
#[derive(Debug, Clone)]
pub enum FlowNodeType {
    Start,
    End,
    Process,
    Decision,
    SubProcess,
    InputOutput,
    Database,
}

/// Flow edge
#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    /// For decision nodes: true/false branch
    pub condition: Option<bool>,
}

// ============================================================================
// Utility implementations
// ============================================================================

impl DataModelSpec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_model(&mut self, model: ModelDef) {
        self.models.push(model);
    }

    pub fn add_enum(&mut self, enum_def: EnumDef) {
        self.enums.push(enum_def);
    }

    pub fn add_relationship(&mut self, rel: Relationship) {
        self.relationships.push(rel);
    }

    /// Get a model by name
    pub fn get_model(&self, name: &str) -> Option<&ModelDef> {
        self.models.iter().find(|m| m.name == name)
    }

    /// Get an enum by name
    pub fn get_enum(&self, name: &str) -> Option<&EnumDef> {
        self.enums.iter().find(|e| e.name == name)
    }
}

impl ModelDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn add_field(&mut self, field: FieldDef) {
        self.fields.push(field);
    }

    pub fn add_method(&mut self, method: MethodDef) {
        self.methods.push(method);
    }
}

impl FieldDef {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
            ..Default::default()
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self.required = false;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }
}

impl RestApiSpec {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            ..Default::default()
        }
    }
}

impl StateMachineSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_builder() {
        let mut model = ModelDef::new("User").with_description("User account model");

        model.add_field(FieldDef::new("id", Type::Int).primary_key());
        model.add_field(FieldDef::new("email", Type::Str));
        model.add_field(
            FieldDef::new("name", Type::Str)
                .optional()
                .with_description("Display name"),
        );

        assert_eq!(model.name, "User");
        assert_eq!(model.fields.len(), 3);
        assert!(model.fields[0].primary_key);
        assert!(!model.fields[2].required);
    }

    #[test]
    fn test_data_model_spec() {
        let mut spec = DataModelSpec::new();

        let user = ModelDef::new("User");
        let post = ModelDef::new("Post");

        spec.add_model(user);
        spec.add_model(post);

        spec.add_relationship(Relationship {
            from_model: "Post".into(),
            from_field: "author_id".into(),
            to_model: "User".into(),
            to_field: "id".into(),
            rel_type: RelationType::ManyToOne,
        });

        assert_eq!(spec.models.len(), 2);
        assert!(spec.get_model("User").is_some());
        assert!(spec.get_model("Post").is_some());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/spec/ir.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/spec/ir.rs` captured during libs codegen standardization.
```

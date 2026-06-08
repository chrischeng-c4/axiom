// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
// CODEGEN-BEGIN
//! SpecIR type definitions
//!
//! The universal contract between SDD generate (spec ownership) and Lens (code generation).

use crate::generate::diagrams::{
    ClassDiagramDef, ERDDef, FlowchartDef, RequirementDiagramDef, SequenceDef, StateMachineDef,
};
use crate::generate::schema::JsonSchema;

// ---------------------------------------------------------------------------
// New spec types (deploy, wireframe, component, design-token section types)
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

/// Kubernetes deployment specification (for the deploy section type). Targets k8s Deployment + Service manifest generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploySpec {
    /// Application name — used as metadata.name in k8s resources.
    #[serde(default)]
    pub name: String,
    /// Container image reference (e.g. nginx:1.21).
    #[serde(default)]
    pub image: String,
    /// Port the container listens on. Defaults to 8080.
    #[serde(default = "default_deploy_port")]
    pub port: u16,
    /// Number of desired pod replicas. Defaults to 1.
    #[serde(default = "default_replicas")]
    pub replicas: u32,
    /// Container environment variables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVar>,
    /// Optional CPU/memory resource limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceLimits>,
}

/// A single environment variable entry.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Variable name.
    pub name: String,
    /// Literal value (mutually exclusive with value_from).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Source reference, e.g. secretKeyRef:my-secret:key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_from: Option<String>,
}

/// CPU / memory resource constraints.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit, e.g. 500m.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    /// Memory limit, e.g. 256Mi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

/// Wireframe specification for React component scaffold generation (for the wireframe section type).
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WireframeSpec {
    /// Component name in PascalCase (e.g. UserCard).
    #[serde(default)]
    pub name: String,
    /// High-level component type: page, layout, card, form, etc.
    #[serde(default)]
    pub component_type: String,
    /// TypeScript props exposed by the component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<PropDef>,
    /// Top-level layout nodes rendered by the component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout: Vec<WireframeNode>,
}

/// A TypeScript prop definition for the React component.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropDef {
    /// Prop name in camelCase.
    pub name: String,
    /// TypeScript type string, e.g. string, number, User.
    pub prop_type: String,
    /// Whether the prop is required (no ? in the interface).
    #[serde(default)]
    pub required: bool,
    /// Default value expression as a string (used in destructuring).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    /// Optional JSDoc description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A single layout node in a wireframe tree.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WireframeNode {
    /// Element kind: text, button, input, list, container, etc.
    pub kind: String,
    /// Display label or placeholder text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Nested child nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WireframeNode>,
}

/// Component Element Model (CEM) spec for TypeScript interface + component skeleton generation (for the component section type).
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentSpec {
    /// Kebab-case custom element tag name (e.g. my-button).
    #[serde(default)]
    pub tag_name: String,
    /// One-line summary shown in generated JSDoc.
    #[serde(default)]
    pub summary: String,
    /// Reflected HTML attributes / observed properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<AttributeDef>,
    /// Named slots.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<SlotDef>,
    /// Custom events emitted by this component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EventDef>,
}

/// A component attribute / property definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDef {
    /// Attribute name in kebab-case.
    pub name: String,
    /// TypeScript type string.
    #[serde(rename = "type", default)]
    pub attr_type: String,
    /// Whether the attribute is required.
    #[serde(default)]
    pub required: bool,
    /// Optional description for JSDoc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A named slot definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlotDef {
    /// Slot name (empty string for the default slot).
    pub name: String,
    /// Optional slot description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A custom event emitted by the component.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    /// Event name in kebab-case.
    pub name: String,
    /// TypeScript type for CustomEvent<T> detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail_type: Option<String>,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Design Token Community Group (DTCG) spec for CSS custom property / Tailwind token generation (for the design-token section type).
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignTokenSpec {
    /// Token collection name used as a CSS prefix (e.g. theme).
    #[serde(default)]
    pub name: String,
    /// Flat list of token entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<DesignTokenEntry>,
}

/// A single DTCG design token.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTokenEntry {
    /// Dot-separated path, e.g. color.primary.500.
    pub path: String,
    /// Resolved token value as a string, e.g. #3B82F6.
    pub value: String,
    /// DTCG type: color, dimension, fontWeight, etc.
    pub token_type: String,
    /// Optional description for generated comments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Common metadata carried by every SpecIR variant. Enables generators to make routing decisions (can_generate) without parsing the full spec payload.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecMetadata {
    /// Source file path (relative to project root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    /// Spec group (e.g. sdd).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_group: Option<String>,
    /// Spec identifier within the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    /// Tags for filtering / routing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Specification Intermediate Representation. Universal input type for code generators. Each variant wraps an existing SDD schema type, carrying it along with SpecMetadata for routing.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpecIR {
    /// API specification from OpenAPI / JSON Schema.
    Api {
        schema: JsonSchema,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// StateMachine+ with states, transitions, guards, and actions.
    StateMachinePlus {
        def: StateMachineDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Flowchart+ with SemanticType annotations for codegen.
    FlowchartPlus {
        def: FlowchartDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Class+ with DDD stereotypes (entity, valueObject, aggregate, etc.).
    ClassPlus {
        def: ClassDiagramDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// ERD+ with PK/FK/UK key types.
    ErdPlus {
        def: ERDDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Sequence+ with loops, alt blocks, and activation.
    SequencePlus {
        def: SequenceDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Requirement+ with N:M mapping for test generation.
    RequirementPlus {
        def: RequirementDiagramDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Kubernetes deployment specification (deploy section type).
    Deploy {
        spec: DeploySpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Wireframe specification for React component scaffold (wireframe section type).
    Wireframe {
        spec: WireframeSpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Component Element Model for TypeScript interface generation (component section type).
    Component {
        spec: ComponentSpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Design Token Community Group format for CSS/Tailwind generation (design-token section type).
    DesignToken {
        spec: DesignTokenSpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
}

/// Metadata for a SpecBundle.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// Change ID this bundle was produced for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_id: Option<String>,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A bundle of related SpecIR items with a dependency graph. Allows generators to receive the complete context for a change — e.g. an ERD+ alongside the API spec that references its entities.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecBundle {
    /// Ordered list of specs.
    pub specs: Vec<SpecIR>,
    /// Dependency edges as (from_index, to_index) pairs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<(usize, usize)>,
    /// Bundle-level metadata.
    #[serde(default)]
    pub metadata: BundleMetadata,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl Default for DeploySpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            image: String::new(),
            port: default_deploy_port(),
            replicas: default_replicas(),
            env: Vec::new(),
            resources: None,
        }
    }
}

fn default_deploy_port() -> u16 {
    8080
}
fn default_replicas() -> u32 {
    1
}

// ---------------------------------------------------------------------------
// SpecIR helper methods
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl SpecIR {
    /// Return the kind tag as a static str.
    pub fn kind(&self) -> &'static str {
        match self {
            SpecIR::Api { .. } => "api",
            SpecIR::StateMachinePlus { .. } => "state_machine_plus",
            SpecIR::FlowchartPlus { .. } => "flowchart_plus",
            SpecIR::ClassPlus { .. } => "class_plus",
            SpecIR::ErdPlus { .. } => "erd_plus",
            SpecIR::SequencePlus { .. } => "sequence_plus",
            SpecIR::RequirementPlus { .. } => "requirement_plus",
            SpecIR::Deploy { .. } => "deploy",
            SpecIR::Wireframe { .. } => "wireframe",
            SpecIR::Component { .. } => "component",
            SpecIR::DesignToken { .. } => "design_token",
        }
    }

    /// Access the metadata regardless of variant.
    pub fn metadata(&self) -> &SpecMetadata {
        match self {
            SpecIR::Api { metadata, .. }
            | SpecIR::StateMachinePlus { metadata, .. }
            | SpecIR::FlowchartPlus { metadata, .. }
            | SpecIR::ClassPlus { metadata, .. }
            | SpecIR::ErdPlus { metadata, .. }
            | SpecIR::SequencePlus { metadata, .. }
            | SpecIR::RequirementPlus { metadata, .. }
            | SpecIR::Deploy { metadata, .. }
            | SpecIR::Wireframe { metadata, .. }
            | SpecIR::Component { metadata, .. }
            | SpecIR::DesignToken { metadata, .. } => metadata,
        }
    }

    /// Access the metadata mutably.
    pub fn metadata_mut(&mut self) -> &mut SpecMetadata {
        match self {
            SpecIR::Api { metadata, .. }
            | SpecIR::StateMachinePlus { metadata, .. }
            | SpecIR::FlowchartPlus { metadata, .. }
            | SpecIR::ClassPlus { metadata, .. }
            | SpecIR::ErdPlus { metadata, .. }
            | SpecIR::SequencePlus { metadata, .. }
            | SpecIR::RequirementPlus { metadata, .. }
            | SpecIR::Deploy { metadata, .. }
            | SpecIR::Wireframe { metadata, .. }
            | SpecIR::Component { metadata, .. }
            | SpecIR::DesignToken { metadata, .. } => metadata,
        }
    }
}

// ---------------------------------------------------------------------------
// From impls: generate types → SpecIR (R3)
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<JsonSchema> for SpecIR {
    fn from(schema: JsonSchema) -> Self {
        SpecIR::Api {
            schema,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<StateMachineDef> for SpecIR {
    fn from(def: StateMachineDef) -> Self {
        SpecIR::StateMachinePlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<FlowchartDef> for SpecIR {
    fn from(def: FlowchartDef) -> Self {
        SpecIR::FlowchartPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<ClassDiagramDef> for SpecIR {
    fn from(def: ClassDiagramDef) -> Self {
        SpecIR::ClassPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<ERDDef> for SpecIR {
    fn from(def: ERDDef) -> Self {
        SpecIR::ErdPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<SequenceDef> for SpecIR {
    fn from(def: SequenceDef) -> Self {
        SpecIR::SequencePlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<RequirementDiagramDef> for SpecIR {
    fn from(def: RequirementDiagramDef) -> Self {
        SpecIR::RequirementPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<DeploySpec> for SpecIR {
    fn from(spec: DeploySpec) -> Self {
        SpecIR::Deploy {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<WireframeSpec> for SpecIR {
    fn from(spec: WireframeSpec) -> Self {
        SpecIR::Wireframe {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<ComponentSpec> for SpecIR {
    fn from(spec: ComponentSpec) -> Self {
        SpecIR::Component {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<DesignTokenSpec> for SpecIR {
    fn from(spec: DesignTokenSpec) -> Self {
        SpecIR::DesignToken {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// SpecBundle helpers
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl SpecBundle {
    /// Create an empty bundle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a spec and return its index.
    pub fn push(&mut self, spec: SpecIR) -> usize {
        let idx = self.specs.len();
        self.specs.push(spec);
        idx
    }

    /// Add a dependency edge (from_index depends on to_index).
    pub fn add_dependency(&mut self, from: usize, to: usize) {
        self.dependencies.push((from, to));
    }

    /// Number of specs in the bundle.
    pub fn len(&self) -> usize {
        self.specs.len()
    }

    /// Whether the bundle is empty.
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_spec_ir_from_json_schema() {
        let schema = JsonSchema {
            title: Some("User".into()),
            ..Default::default()
        };
        let ir = SpecIR::from(schema);
        assert_eq!(ir.kind(), "api");
        assert!(ir.metadata().source_path.is_none());
    }

    #[test]
    fn test_spec_ir_from_flowchart_def() {
        let def = FlowchartDef {
            id: "test-flow".into(),
            direction: Default::default(),
            nodes: IndexMap::new(),
            edges: vec![],
            subgraphs: vec![],
            description: None,
        };
        let ir = SpecIR::from(def);
        assert_eq!(ir.kind(), "flowchart_plus");
    }

    #[test]
    fn test_spec_bundle_with_dependencies() {
        let schema1 = JsonSchema {
            title: Some("Entity".into()),
            ..Default::default()
        };
        let schema2 = JsonSchema {
            title: Some("API".into()),
            ..Default::default()
        };

        let mut bundle = SpecBundle::new();
        let idx0 = bundle.push(SpecIR::from(schema1));
        let idx1 = bundle.push(SpecIR::from(schema2));
        bundle.add_dependency(idx1, idx0); // API depends on Entity

        assert_eq!(bundle.len(), 2);
        assert_eq!(bundle.dependencies, vec![(1, 0)]);
    }

    #[test]
    fn test_spec_ir_serialize_roundtrip() {
        let schema = JsonSchema {
            title: Some("Test".into()),
            ..Default::default()
        };
        let ir = SpecIR::Api {
            schema,
            metadata: SpecMetadata {
                source_path: Some("specs/test.json".into()),
                spec_group: Some("test-group".into()),
                spec_id: Some("test-spec".into()),
                tags: vec!["api".into()],
            },
        };

        let json = serde_json::to_value(&ir).unwrap();
        assert_eq!(json["kind"], "api");
        assert_eq!(json["source_path"], "specs/test.json");
        assert_eq!(json["tags"], serde_json::json!(["api"]));

        // Roundtrip
        let deserialized: SpecIR = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.kind(), "api");
        assert_eq!(
            deserialized.metadata().source_path.as_deref(),
            Some("specs/test.json")
        );
    }

    #[test]
    fn test_metadata_access() {
        let schema = JsonSchema::default();
        let mut ir = SpecIR::from(schema);
        ir.metadata_mut().tags.push("generated".into());
        assert_eq!(ir.metadata().tags, vec!["generated"]);
    }
}

// CODEGEN-END

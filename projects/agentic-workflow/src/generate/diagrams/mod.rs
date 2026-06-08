// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/mod.md#source
// CODEGEN-BEGIN
//! Mermaid Diagram Generation
//!
//! Provides functions for generating various types of Mermaid diagrams.
//! Includes both simple generators and Plus (validated, YAML frontmatter) variants.
//!
//! New codegen types (per-diagram Content structs, XState-free):
//! - `envelope` — common `Diagram<C>` envelope + `DiagramFrontmatter` trait
//! - `content` — per-diagram Content types (state_machine, interaction, logic, requirement)

// New codegen envelope and content types
pub mod content;
pub mod envelope;

// Simple generators
pub mod class;
pub mod erd;
pub mod flowchart;
pub mod journey;
pub mod mindmap;
pub mod requirement;
pub mod sequence;
pub mod state;

// Plus generators (schema + validator + YAML frontmatter)
pub mod block_plus;
pub mod class_plus;
pub mod erd_plus;
pub mod flowchart_plus;
pub mod journey_plus;
pub mod mermaid_plus;
pub mod mindmap_plus;
pub mod requirement_plus;
pub mod sequence_plus;
pub mod state_plus;

// Simple generator exports
pub use class::{generate_class_diagram, ClassDef, ClassInput, ClassRelationship};
pub use erd::{generate_erd, Entity, ErdInput, ErdRelationship};
pub use flowchart::{generate_flowchart, FlowchartEdge, FlowchartInput, FlowchartNode};
pub use journey::{generate_journey, JourneyInput, JourneySection, JourneyTask};
pub use mindmap::{generate_mindmap, MindmapInput, MindmapNode, MindmapRoot};
pub use requirement::{generate_requirement_diagram, RequirementDef, RequirementInput};
pub use sequence::{generate_sequence, Message, Participant, SequenceInput};
pub use state::{generate_state_diagram, StateDef, StateInput, StateTransition};

// Mermaid+ (original state machine format, kept for backward compatibility)
pub use mermaid_plus::{
    ActionDef, ActionRef, GuardDef, MermaidPlusGenerator, MermaidPlusOutput, Severity,
    StateMachineDef, StateMachineValidator, StateNodeDef, StateType, TransitionDetail,
    TransitionInput, ValidationError, ValidationResult,
};

// State+ re-export (same as mermaid_plus, aliased for consistency)
pub use state_plus as state_machine_plus;

// Flowchart+ exports
pub use flowchart_plus::{
    EdgeDef as FlowchartEdgeDef, EdgeStyle, FlowDirection, FlowchartDef, FlowchartPlusGenerator,
    FlowchartPlusOutput, FlowchartSeverity, FlowchartValidationError, FlowchartValidationResult,
    FlowchartValidator, NodeDef as FlowchartNodeDef, NodeShape, PrimitiveKind, SemanticType,
    SubgraphDef,
};

// Sequence+ exports
pub use sequence_plus::{
    AltBlockType, AltDef, ArrowType, LoopDef, MessageDef, NoteDef, NotePosition, ParticipantDef,
    ParticipantType, SequenceDef, SequencePlusGenerator, SequencePlusOutput, SequenceSeverity,
    SequenceValidationError, SequenceValidationResult, SequenceValidator,
};

// Class+ exports
pub use class_plus::{
    AttributeDef, ClassDef as ClassDefPlus, ClassDiagramDef, ClassPlusGenerator, ClassPlusOutput,
    ClassSeverity, ClassStereotype, ClassValidationError, ClassValidationResult, ClassValidator,
    MethodDef, NamespaceDef, ParameterDef, RelationshipDef as ClassRelationshipDef,
    RelationshipType, Visibility,
};

// ERD+ exports
pub use erd_plus::{
    Cardinality, ERDAttributeDef, ERDDef, ERDPlusGenerator, ERDPlusOutput, ERDRelationshipDef,
    ERDSeverity, ERDValidationError, ERDValidationResult, ERDValidator, EntityDef, KeyType,
};

// Requirement+ exports
pub use requirement_plus::{
    ElementDef, ReqDirection, ReqRelationshipDef, ReqRelationshipTypePlus, RequirementDefPlus,
    RequirementDiagramDef, RequirementPlusGenerator, RequirementPlusOutput, RequirementSeverity,
    RequirementTypePlus, RequirementValidationError, RequirementValidationResult,
    RequirementValidator, RiskLevelPlus, VerificationMethodPlus,
};

// Mindmap+ exports
pub use mindmap_plus::{
    MindmapDef, MindmapNodeDef, MindmapPlusGenerator, MindmapPlusOutput, MindmapSeverity,
    MindmapShapePlus, MindmapValidationError, MindmapValidationResult, MindmapValidator,
};

// Journey+ exports
pub use journey_plus::{
    JourneyDef, JourneyPlusGenerator, JourneyPlusOutput, JourneySeverity, JourneyValidationError,
    JourneyValidationResult, JourneyValidator, SectionDef, TaskDef,
};

// Block+ exports
pub use block_plus::{
    BlockDef, BlockEdgeDef, BlockEdgeStyle, BlockNodeDef, BlockPlusGenerator, BlockPlusOutput,
    BlockSeverity, BlockShape, BlockValidationError, BlockValidationResult, BlockValidator,
};

// CODEGEN-END

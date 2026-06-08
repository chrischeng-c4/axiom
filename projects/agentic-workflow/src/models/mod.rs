// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/mod.md#source
// CODEGEN-BEGIN
pub mod annotation;
pub mod archive_review;
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
pub mod artifact_quality;
pub mod challenge;
pub mod change;
pub mod context;
pub mod frontmatter;
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-preflight-gates.md#schema
pub mod preflight;
pub mod project;
pub mod reference_context_sections;
pub mod requirement;
pub mod review;
pub mod scenario;
pub mod section;
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
pub mod source_reference;
pub mod spec_rules;
pub mod state;
pub mod task_graph;
pub mod tech_stack;
pub mod validation;
pub mod verification;

pub use archive_review::{
    ArchiveIssueCategory, ArchiveReview, ArchiveReviewIssue, ArchiveReviewVerdict,
};
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
pub use artifact_quality::{
    ArtifactKind, ArtifactQualityProfile, ArtifactSourceMode, ArtifactSourcePolicy,
    PreflightGateSet, QualityDial,
};
pub use challenge::{Challenge, ChallengeIssue, ChallengeVerdict, IssueSeverity};
pub use change::{
    Change, ChangePhase, ConfigLanguage, ProjectConfig, ProjectModule, RepoPlatformConfig,
    SddConfig, SddInterface, SpecsConfig, TechDesignPlatformConfig, WorkflowArtifact,
    WorkflowConfig,
};
pub use frontmatter::{
    // Document frontmatter types
    DesignElements,
    HistoryEntry,
    // Inline block types
    IssueBlock,
    IssueLocation,
    IssueSeverity as FrontmatterIssueSeverity,
    LayerBreakdown,
    LayerInfo,
    MainSpecFrontmatter,
    MergeStrategy,
    PriorityBreakdown,
    RequirementBlock,
    RequirementPriority,
    RequirementStatus,
    RequirementsSummary,
    SpecFrontmatter,
    SpecReference,
    TaskAction,
    TaskBlock,
    TaskStatus,
    TasksFrontmatter,
    TasksSummary,
};
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-preflight-gates.md#schema
pub use preflight::{
    default_preflight_gates, PreFlightEvidence, PreFlightEvidenceKind, PreFlightEvidenceStatus,
    PreFlightGate, PreFlightGateReport, PreFlightGateResult, PreFlightGateSeverity,
    PreFlightGateStatus,
};
pub use reference_context_sections::REFERENCE_CONTEXT_SECTIONS;
pub use requirement::{Requirement, RequirementDelta};
pub use review::{IssueCategory, ReviewIssue, ReviewVerdict};
pub use scenario::Scenario;
pub use section::{parse_all_section_annotations, parse_section_annotation, SectionMeta};
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
pub use source_reference::{
    evaluate_source_references, SourceFailureMode, SourceReference, SourceReferenceAvailability,
    SourceReferenceKind, SourceReferencePolicy, SourceReferenceRequirement, SourceReferenceReview,
    SourceReviewFinding, SourceReviewSeverity,
};
pub use spec_rules::{
    DocumentType as SpecDocumentType, ScenarioFormat, SectionEntry, SectionType, SpecFormatRules,
};
pub use state::{
    ChecksumEntry, DagIssue, DagState, LlmCall, State, StatePhase, Telemetry, ValidationEntry,
    ValidationMode,
};
pub use task_graph::{Layer, SpecGroup, TaskGraph, TaskRef};
pub use tech_stack::{DesignSystem, TechStack};
pub use validation::{
    DocumentType, ErrorCategory, JsonValidationError, Severity, SeverityMap, ValidationCounts,
    ValidationError, ValidationJsonOutput, ValidationOptions, ValidationResult, ValidationRules,
};
pub use verification::{TestResult, TestStatus, Verification};

pub use annotation::{
    get_author_name, Annotation, AnnotationError, AnnotationResult, AnnotationStore,
};
pub use context::{
    CodebaseContext, ContextType, DocRef, FileRef, KnowledgeContext, LensResult, PatternRef,
    ReviewFeedback, ReviewVerdict as ContextReviewVerdict, SpecContext, SpecRef,
};

// CODEGEN-END

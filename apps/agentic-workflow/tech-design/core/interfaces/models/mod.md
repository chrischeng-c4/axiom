---
id: projects-sdd-src-models-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Standardized apps/agentic-workflow/src/models/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/models/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `annotation` | apps/agentic-workflow/src/models/mod.rs | module | pub | 3 |  |
| `archive_review` | apps/agentic-workflow/src/models/mod.rs | module | pub | 4 |  |
| `artifact_quality` | apps/agentic-workflow/src/models/mod.rs | module | pub | 6 |  |
| `challenge` | apps/agentic-workflow/src/models/mod.rs | module | pub | 7 |  |
| `change` | apps/agentic-workflow/src/models/mod.rs | module | pub | 8 |  |
| `context` | apps/agentic-workflow/src/models/mod.rs | module | pub | 9 |  |
| `frontmatter` | apps/agentic-workflow/src/models/mod.rs | module | pub | 10 |  |
| `preflight` | apps/agentic-workflow/src/models/mod.rs | module | pub | 12 |  |
| `project` | apps/agentic-workflow/src/models/mod.rs | module | pub | 13 |  |
| `reference_context_sections` | apps/agentic-workflow/src/models/mod.rs | module | pub | 14 |  |
| `requirement` | apps/agentic-workflow/src/models/mod.rs | module | pub | 15 |  |
| `review` | apps/agentic-workflow/src/models/mod.rs | module | pub | 16 |  |
| `scenario` | apps/agentic-workflow/src/models/mod.rs | module | pub | 17 |  |
| `section` | apps/agentic-workflow/src/models/mod.rs | module | pub | 18 |  |
| `source_reference` | apps/agentic-workflow/src/models/mod.rs | module | pub | 20 |  |
| `spec_rules` | apps/agentic-workflow/src/models/mod.rs | module | pub | 21 |  |
| `state` | apps/agentic-workflow/src/models/mod.rs | module | pub | 22 |  |
| `task_graph` | apps/agentic-workflow/src/models/mod.rs | module | pub | 23 |  |
| `tech_stack` | apps/agentic-workflow/src/models/mod.rs | module | pub | 24 |  |
| `validation` | apps/agentic-workflow/src/models/mod.rs | module | pub | 25 |  |
| `verification` | apps/agentic-workflow/src/models/mod.rs | module | pub | 26 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/models/mod.rs -->
```rust
pub mod annotation;
pub mod archive_review;
pub mod artifact_quality;
pub mod challenge;
pub mod change;
pub mod context;
pub mod frontmatter;
pub mod preflight;
pub mod project;
pub mod reference_context_sections;
pub mod requirement;
pub mod review;
pub mod scenario;
pub mod section;
pub mod spec_rules;
pub mod state;
pub mod task_graph;
pub mod tech_stack;
pub mod validation;
pub mod verification;

pub use archive_review::{
    ArchiveIssueCategory, ArchiveReview, ArchiveReviewIssue, ArchiveReviewVerdict,
};
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/models/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete model module facade.
```

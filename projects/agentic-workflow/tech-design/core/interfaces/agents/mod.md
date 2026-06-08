---
id: sdd-agents-mod-autoapprove
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# AutoApproveHandler Type

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AutoApproveHandler` | projects/agentic-workflow/src/agents/mod.rs | struct | pub | 106 |  |
| `change_spec` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 7 |  |
| `code_agent` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 8 |  |
| `codebase_to_spec` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 9 |  |
| `crr` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 11 |  |
| `reference_codebase_context` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 12 |  |
| `reference_spec_context` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 13 |  |
| `restructure` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 14 |  |
| `restructure_codebase` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 15 |  |
| `review` | projects/agentic-workflow/src/agents/mod.rs | module | pub | 16 |  |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  AutoApproveHandler:
    type: object
    required: []
    properties: {}
    description: |
      Default approval handler that auto-approves everything. Unit
      struct; behaviour lives on the generated
      `impl ApprovalHandler for AutoApproveHandler` block.
    x-rust-struct:
      derive: []
      unit: true
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/mod.rs -->
```rust
//! Agent module - defines the Agent trait and agent implementations.

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/mod.md#source
mod analyst;
pub mod change_spec;
pub mod code_agent;
pub mod codebase_to_spec;
mod coding;
pub mod crr;
pub mod reference_codebase_context;
pub mod reference_spec_context;
pub mod restructure;
pub mod restructure_codebase;
pub mod review;

// Keep the old module path accessible so existing code compiles without changes.
// `reference_context` is now an alias for `reference_spec_context`.
pub use reference_spec_context as reference_context;

pub use crate::spec_store::FileSystemSpecStore;
pub use analyst::{AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig};
pub use change_spec::{
    ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput,
};
pub use code_agent::{
    CodeAgent, CodeAgentBuilder, CodeAgentConfig, FileBlock, ImplementationTask, TaskAction,
    TaskCategory,
};
pub use codebase_to_spec::{
    CodebaseToSpecAgent, CodebaseToSpecAgentBuilder, CodebaseToSpecAgentConfig, CodebaseToSpecInput,
};
pub use coding::{CodingAgent, CodingAgentBuilder, CodingAgentConfig};
pub use crr::{CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType};
pub use reference_codebase_context::{
    CodebaseDependency, ComponentRelationship, KeyFile, ReferenceCodebaseArtifact,
    ReferenceCodebaseContextAgent, ReferenceCodebaseContextAgentBuilder,
    ReferenceCodebaseContextAgentConfig,
};
pub use reference_spec_context::{
    Contradiction, ReferenceContextOutput, ReferenceSpecContextAgent,
    ReferenceSpecContextAgentBuilder, ReferenceSpecContextAgentConfig, RelevanceLevel,
    SpecReferenceEntry,
};
pub use restructure::{
    Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
    RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
};
pub use restructure_codebase::{
    RestructureCodebaseAgent, RestructureCodebaseAgentBuilder, RestructureCodebaseAgentConfig,
};
pub use review::{
    ReviewAgent, ReviewAgentBuilder, ReviewAgentConfig, ReviewIssue, ReviewType, ReviewVerdict,
    Reviewer, Severity,
};

use agent::error::NovaResult;
use agent::stream::StreamHandler;
use async_trait::async_trait;

/// Core trait for all agent implementations.
///
/// Agents are autonomous LLM-powered assistants that can execute tools
/// to accomplish tasks. This trait defines the common interface for
/// running agents with different configurations.
#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/mod.md#source
pub trait Agent: Send + Sync {
    /// Run the agent with user input and return the final response.
    async fn run(&self, input: &str) -> NovaResult<String>;

    /// Run the agent with a custom stream handler for real-time events.
    async fn run_with_handler(
        &self,
        input: &str,
        handler: &dyn StreamHandler,
    ) -> NovaResult<String>;
}

/// Approval handler trait for human-in-the-loop approval.
#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/mod.md#source
pub trait ApprovalHandler: Send + Sync {
    async fn request_approval(
        &self,
        request: agent::security::ApprovalRequest,
    ) -> NovaResult<agent::security::ApprovalResponse>;
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/mod.md#source
impl ApprovalHandler for AutoApproveHandler {
    async fn request_approval(
        &self,
        _request: agent::security::ApprovalRequest,
    ) -> NovaResult<agent::security::ApprovalResponse> {
        Ok(agent::security::ApprovalResponse::Approved)
    }
}

/// Default approval handler that auto-approves everything. Unit
/// struct; behaviour lives on the generated
/// `impl ApprovalHandler for AutoApproveHandler` block.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/mod.md#schema
pub struct AutoApproveHandler;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete agent module facade, including module
      declarations, public re-exports, async traits, the auto-approve handler
      unit struct, and its approval implementation.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Correctly identifies the unit struct, its trait impl, and the hand-written boundary (all module-level items, two trait decls, async_trait impl).
- [schema] Definition is well-formed: `properties: {}` + `required: []` + `x-rust-struct.derive: []` + `unit: true` produces a bare `pub struct X;`. Matches the 17 generator unit structs already dogfooded.
- [changes] Two entries cleanly split codegen vs hand-written. `replaces` lists the single struct name; hand-written entry covers all out-of-block items including the async_trait impl.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Promotes the module facade to full source ownership without adding a Rust-specific section type for module declarations or trait impls.
- [source] Uses `strip-managed-markers` so the current Rust module remains the source template while removing the HANDWRITE/CODEGEN wrapper split.
- [changes] Correctly routes the target file through the `source` section with `impl_mode: codegen`.

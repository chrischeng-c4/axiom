// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/mod.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END

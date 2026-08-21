//! # agent
//!
//! High-performance LLM agent framework with Rust core. Provides LLM
//! provider abstractions, tool execution, session storage, and
//! third-party integrations (GitHub, GitLab, Jira).

pub mod agent;
pub mod cancellation_timeout;
pub mod context;
pub mod error;
pub mod eval;
pub mod events;
pub mod integrations;
pub mod llm;
pub mod observability;
pub mod protocols;
pub mod schema;
pub mod security;
pub mod storage;
pub mod stream;
pub mod structured;
pub mod sync_adapter;
pub mod tokenizer;
pub mod tools;

pub mod types;

// Re-export Agent runtime (#2030, #2033, #2034)
pub use agent::{
    Agent, AgentBuilder, RunContext, SystemPromptFn, ToolHandler, ToolHandlerFuture, ToolSpec,
};

// Re-export context
pub use context::{ContextManager, ContextStats};

// Re-export error types
pub use error::{NovaError, NovaResult};

// Re-export observability bootstrap and trace export surface (#2057)
pub use observability::{
    agent_event_to_trace_record, agent_events_to_trace_records, init_stdout_subscriber,
    TraceContext, TraceRecord, TraceRecordKind,
};

// Re-export structured event log (#2058)
pub use events::{AgentEvent, EventBus, EventEmitter, Subscriber as EventSubscriber};

// Re-export eval harness (#2060)
pub use eval::{
    Dataset, DatasetRunner, Eval, EvalCase, EvalCaseResult, EvalReport, ExactMatchScorer,
    JsonEqScorer, Score, Scorer,
};

// Re-export cancellation + timeout primitives (#2070)
pub use cancellation_timeout::{
    run_until_cancelled, run_with_deadline, with_timeout, CancellationSnapshot, CancellationToken,
};

// Re-export schema builder + validator (pydantic-style typed I/O foundation, #1949, #1950)
pub use schema::{ObjectSchemaBuilder, Schema, SchemaMetadata, ValidationError};

// Re-export `#[derive(AgentSchema)]` proc-macro (#1952)
pub use agent_derive::AgentSchema;

// Re-export LLM types
pub use llm::{
    ClaudeProvider, CompletionRequest, CompletionResponse, GeminiProvider, LLMProvider,
    OpenAIProvider, StreamChunk, StreamResponse, ToolDefinition,
};

// Re-export security types
pub use security::{ApprovalRequest, ApprovalResponse, SecurityPolicy, SecurityPolicyBuilder};

// Re-export stream types
pub use stream::{
    CallbackHandler, CollectingHandler, NoOpHandler, PrintHandler, StreamEvent, StreamHandler,
};

// Re-export coding tools
pub use tools::{
    BashTool, EditFileTool, GlobTool, GrepTool, ReadFileTool, StreamingBashTool, Tool,
    ToolExecutor, ToolParameter, ToolRegistry, WriteFileTool,
};

// Re-export analysis tools
pub use tools::{AskUserTool, RecordFindingTool, TakeNoteTool, WebFetchTool, WebSearchTool};

// Re-export codebase restructuring tools
pub use tools::{
    EstimateTokensTool, GroupingState, ListFolderSummaryTool, ReadManifestTool, SetGroupingTool,
    SpecGroup,
};

// Re-export core types
pub use types::{AgentId, Message, Role, TokenUsage, ToolCall, ToolResult};

// Re-export storage types
pub use storage::{
    FileStorage, Finding, FindingSeverity, MemoryStorage, Note, SessionInfo, SessionState,
    SessionStatus, Storage,
};

// Re-export integration types
pub use integrations::{
    CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, GitHubIntegration,
    GitLabIntegration, Issue, IssueComment, IssueFilter, IssueState, IssueSummary, JiraIntegration,
    PlatformIntegration, PullRequestParams,
};

// Re-export protocol types
pub use protocols::{
    ChangeProtocol, ChangeStatus, CodeIndexProtocol, IssuePriority, IssueProtocol, IssueStatus,
    Platform, ProjectProtocol, SpecFormat, SpecProtocol,
};

// Re-export sync adapter types
pub use sync_adapter::{
    ConfluenceSyncAdapter, GDocsSyncAdapter, GitHubSyncAdapter, GitLabSyncAdapter, JiraSyncAdapter,
    SyncAction, SyncAdapter, SyncResult,
};
// SPEC-MANAGED: .aw/tech-design/projects/agentkit/specs/unified-inner-core.md#dependency
// CODEGEN-BEGIN
// SPEC-REF: .aw/tech-design/projects/agentkit/specs/unified-inner-core.md#dependency
// TODO: Implement projects/agentkit/core/src/lib.rs
// CODEGEN-END
// SPEC-MANAGED: .aw/tech-design/projects/agentkit/specs/cross-surface-rpc-contract.md#changes
// CODEGEN-BEGIN
// SPEC-REF: .aw/tech-design/projects/agentkit/specs/cross-surface-rpc-contract.md#changes
// TODO: Implement projects/agentkit/core/src/lib.rs
// CODEGEN-END

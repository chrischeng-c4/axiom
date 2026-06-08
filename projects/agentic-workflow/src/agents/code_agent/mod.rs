// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
// CODEGEN-BEGIN
//! CodeAgent — transforms approved specifications into code implementations.
//!
//! # Flow
//!
//! ```text
//! execute(spec)
//!   │
//!   ├─ decompose_spec → ordered ImplementationTask list
//!   │
//!   ├─ CRRCycle(creator=LLM, reviewer=ReviewAgent, reviser=LLM)
//!   │     └─ artifact = multi-file XML blob
//!   │
//!   ├─ parse_file_blocks → Vec<FileBlock>
//!   │
//!   ├─ PlatformIntegration::create_branch
//!   ├─ PlatformIntegration::create_commit
//!   └─ PlatformIntegration::create_pull_request → PR URL
//! ```

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
mod parser;
mod tasks;

pub use parser::{parse_file_blocks, FileBlock};
pub use tasks::{decompose_spec, ImplementationTask, TaskAction, TaskCategory};

use crate::agents::crr::CRRCycle;
use crate::agents::review::{ReviewAgent, ReviewType, Reviewer};
use crate::agents::Agent;
use agent::error::{NovaError, NovaResult};
use agent::integrations::{CommitFile, PlatformIntegration, PullRequestParams};
use agent::llm::{CompletionRequest, LLMProvider};
use agent::stream::{NoOpHandler, StreamHandler};
use agent::types::Message;
use async_trait::async_trait;
use std::sync::Arc;

// ============================================================
// Config
// ============================================================

/// Configuration for CodeAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#schema
#[derive(Debug, Clone)]
pub struct CodeAgentConfig {
    /// LLM model identifier.
    pub model: String,
    /// Maximum tokens per LLM call.
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Maximum CRR revision rounds.
    pub max_revisions: u32,
    /// Base branch to branch from.
    pub base_branch: String,
    /// Prefix for auto-generated branch names.
    pub branch_prefix: String,
}

/// Autonomous agent that implements approved specs end-to-end.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#schema
pub struct CodeAgent {
    /// Agent configuration.
    config: CodeAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
    /// CRR reviewer.
    reviewer: Arc<dyn Reviewer>,
    /// Platform integration.
    platform: Arc<dyn PlatformIntegration>,
}

/// Builder for CodeAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#schema
pub struct CodeAgentBuilder {
    /// Agent configuration.
    config: CodeAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
    /// Optional reviewer.
    reviewer: Option<Arc<dyn Reviewer>>,
    /// Optional platform integration.
    platform: Option<Arc<dyn PlatformIntegration>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
impl Default for CodeAgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(8192),
            temperature: Some(0.0),
            max_revisions: 3,
            base_branch: "main".to_string(),
            branch_prefix: "feature/code-agent-".to_string(),
        }
    }
}

// ============================================================
// CodeAgent
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
impl std::fmt::Debug for CodeAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
impl CodeAgent {
    /// Return a builder for this agent.
    pub fn builder() -> CodeAgentBuilder {
        CodeAgentBuilder::new()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
impl Agent for CodeAgent {
    /// Run with `input` as the full specification text.
    ///
    /// Returns the URL of the opened pull / merge request on success.
    async fn run(&self, input: &str) -> NovaResult<String> {
        let handler = NoOpHandler;
        self.run_with_handler(input, &handler).await
    }

    async fn run_with_handler(
        &self,
        input: &str,
        _handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        // 1. Decompose spec into topologically sorted tasks.
        let tasks = decompose_spec(input);

        // 2. Build the generation prompt from the tasks and spec.
        let generation_prompt = build_generation_prompt(input, &tasks);

        // 3. Wire up CRR cycle.
        let creator = Arc::new(InlineLLMAgent {
            provider: self.provider.clone(),
            system_prompt: CODE_GENERATION_SYSTEM_PROMPT.to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        });

        let reviser = Arc::new(InlineLLMAgent {
            provider: self.provider.clone(),
            system_prompt: CODE_REVISION_SYSTEM_PROMPT.to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        });

        let crr = CRRCycle::new()
            .creator_arc(creator)
            .reviewer_arc(self.reviewer.clone())
            .reviser_arc(reviser)
            .max_revisions(self.config.max_revisions)
            .build()?;

        // 4. Run CRR — get approved code artifact (multi-file XML blob).
        let crr_result = crr.run(&generation_prompt).await?;

        // 5. Parse XML file blocks.
        let file_blocks = parse_file_blocks(&crr_result.artifact)?;

        // 6. Create a remote branch.
        let branch_name = format!(
            "{}{}",
            self.config.branch_prefix,
            chrono::Utc::now().timestamp()
        );
        self.platform
            .create_branch(&branch_name, &self.config.base_branch)
            .await
            .map_err(|e| NovaError::PlatformError(format!("create_branch failed: {}", e)))?;

        // 7. Commit all generated files in a single commit.
        let commit_files: Vec<CommitFile> = file_blocks
            .into_iter()
            .map(|fb| CommitFile {
                path: fb.path,
                content: fb.content,
            })
            .collect();

        self.platform
            .create_commit(
                &branch_name,
                "feat: implement spec via CodeAgent",
                &commit_files,
            )
            .await
            .map_err(|e| NovaError::PlatformError(format!("create_commit failed: {}", e)))?;

        // 8. Open a pull / merge request.
        let pr = self
            .platform
            .create_pull_request(&PullRequestParams {
                title: "feat: implement spec via CodeAgent".to_string(),
                body: format!(
                    "Automated implementation generated by CodeAgent.\n\n\
                     CRR revisions: {}",
                    crr_result.revision_count
                ),
                head: branch_name,
                base: self.config.base_branch.clone(),
            })
            .await
            .map_err(|e| NovaError::PlatformError(format!("create_pull_request failed: {}", e)))?;

        Ok(pr.url)
    }
}

// ============================================================
// Prompt construction
// ============================================================

fn build_generation_prompt(spec: &str, tasks: &[ImplementationTask]) -> String {
    let mut prompt = String::from(
        "Implement the following specification. \
         Output ALL files using the XML format below — one block per file:\n\n\
         <file path=\"relative/path/to/file.ext\">\n\
         // full file content here\n\
         </file>\n\n\
         Do NOT include any prose outside the <file> blocks.\n\n\
         ## Specification\n\n",
    );
    prompt.push_str(spec);

    if !tasks.is_empty() {
        prompt.push_str("\n\n## Implementation Order\n\n");
        for task in tasks {
            prompt.push_str(&format!(
                "- [{:?}] {} → `{}`\n",
                task.action, task.description, task.file_path
            ));
        }
    }

    prompt
}

const CODE_GENERATION_SYSTEM_PROMPT: &str = r#"You are an expert Rust/software engineer.
Your task: implement a specification exactly as written.

Rules:
1. Output ONLY <file path="...">...</file> blocks — no commentary outside them.
2. Implement every file listed in the ## Changes section.
3. Follow existing code style and module conventions.
4. Include #[cfg(test)] unit tests for all public functions.
5. Handle errors with the existing NovaError type."#;

const CODE_REVISION_SYSTEM_PROMPT: &str = r#"You are an expert Rust/software engineer performing a revision.
You will receive a previous implementation and a list of review issues.

Rules:
1. Address every review issue listed.
2. Output the FULL revised files as <file path="...">...</file> blocks.
3. Do not omit unchanged files — include all files from the original output.
4. Do not add commentary outside the <file> blocks."#;

// ============================================================
// InlineLLMAgent — minimal Agent wrapper around a single LLM call
// ============================================================

/// A minimal agent that forwards the input directly to the LLM.
struct InlineLLMAgent {
    provider: Arc<dyn LLMProvider>,
    system_prompt: String,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
impl Agent for InlineLLMAgent {
    async fn run(&self, input: &str) -> NovaResult<String> {
        let handler = NoOpHandler;
        self.run_with_handler(input, &handler).await
    }

    async fn run_with_handler(
        &self,
        input: &str,
        _handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        let messages = vec![Message::system(&self.system_prompt), Message::user(input)];

        let mut request = CompletionRequest::new(messages, &self.model);
        if let Some(temp) = self.temperature {
            request = request.with_temperature(temp);
        }
        if let Some(max_tokens) = self.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }

        let response = self.provider.complete(request).await?;
        Ok(response.content)
    }
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
impl CodeAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: CodeAgentConfig::default(),
            provider: None,
            reviewer: None,
            platform: None,
        }
    }

    /// Set the LLM provider used for code generation and revision.
    pub fn with_provider<P: LLMProvider + 'static>(mut self, provider: P) -> Self {
        self.provider = Some(Arc::new(provider));
        self
    }

    /// Set the LLM provider from an `Arc`.
    pub fn with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set a custom reviewer.  Defaults to a [`ReviewAgent`] using `Code` mode.
    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    /// Set the reviewer from an `Arc`.
    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
        self
    }

    /// Set the platform integration for source control operations.
    pub fn with_platform<P: PlatformIntegration + 'static>(mut self, platform: P) -> Self {
        self.platform = Some(Arc::new(platform));
        self
    }

    /// Set the platform integration from an `Arc`.
    pub fn with_platform_arc(mut self, platform: Arc<dyn PlatformIntegration>) -> Self {
        self.platform = Some(platform);
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    pub fn with_max_revisions(mut self, n: u32) -> Self {
        self.config.max_revisions = n;
        self
    }

    pub fn with_base_branch(mut self, branch: impl Into<String>) -> Self {
        self.config.base_branch = branch.into();
        self
    }

    pub fn with_branch_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.branch_prefix = prefix.into();
        self
    }

    /// Build the [`CodeAgent`].
    ///
    /// # Errors
    ///
    /// Returns [`NovaError::ConfigError`] if `provider` or `platform` is missing.
    pub fn build(self) -> NovaResult<CodeAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("CodeAgent: provider is required".to_string()))?;

        let platform = self
            .platform
            .ok_or_else(|| NovaError::ConfigError("CodeAgent: platform is required".to_string()))?;

        // Default reviewer: ReviewAgent with Code review type using the same provider.
        let reviewer: Arc<dyn Reviewer> = match self.reviewer {
            Some(r) => r,
            None => {
                let review_agent = ReviewAgent::builder()
                    .with_provider_arc(provider.clone())
                    .with_review_type(ReviewType::Code)
                    .with_model(self.config.model.clone())
                    .build()?;
                Arc::new(review_agent)
            }
        };

        Ok(CodeAgent {
            config: self.config,
            provider,
            reviewer,
            platform,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/mod.md#source
impl Default for CodeAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::review::{ReviewVerdict, Reviewer, Severity};
    use agent::integrations::{
        CommitFile, CreatedBranch, CreatedCommit, CreatedPullRequest, Issue, IssueComment,
        IssueFilter, IssueSummary, PostedComment, PullRequestParams,
    };
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use std::collections::HashMap;

    // ---- mock LLM provider ----

    struct MockProvider {
        response: String,
    }

    #[async_trait::async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            "mock"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["mock".to_string()]
        }
        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
            Ok(CompletionResponse {
                content: self.response.clone(),
                tool_calls: None,
                finish_reason: "stop".to_string(),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
                metadata: HashMap::new(),
            })
        }
        async fn complete_stream(&self, _req: CompletionRequest) -> NovaResult<StreamResponse> {
            unimplemented!()
        }
    }

    // ---- mock reviewer that always approves ----

    struct ApproveReviewer;

    #[async_trait::async_trait]
    impl Reviewer for ApproveReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            Ok(ReviewVerdict::Approved)
        }
    }

    // ---- mock platform ----

    struct MockPlatform;

    #[async_trait::async_trait]
    impl agent::integrations::PlatformIntegration for MockPlatform {
        fn name(&self) -> &str {
            "mock"
        }
        async fn get_issue(&self, _id: &str) -> NovaResult<Issue> {
            unimplemented!()
        }
        async fn list_issues(&self, _filter: &IssueFilter) -> NovaResult<Vec<IssueSummary>> {
            unimplemented!()
        }
        async fn get_comments(&self, _issue_id: &str) -> NovaResult<Vec<IssueComment>> {
            unimplemented!()
        }
        async fn post_comment(&self, _issue_id: &str, _body: &str) -> NovaResult<PostedComment> {
            unimplemented!()
        }
        async fn create_branch(
            &self,
            branch_name: &str,
            _from_ref: &str,
        ) -> NovaResult<CreatedBranch> {
            Ok(CreatedBranch {
                name: branch_name.to_string(),
                sha: "abc123".to_string(),
            })
        }
        async fn create_commit(
            &self,
            _branch: &str,
            _message: &str,
            _files: &[CommitFile],
        ) -> NovaResult<CreatedCommit> {
            Ok(CreatedCommit {
                sha: "def456".to_string(),
                url: "https://example.com/commit/def456".to_string(),
            })
        }
        async fn create_pull_request(
            &self,
            params: &PullRequestParams,
        ) -> NovaResult<CreatedPullRequest> {
            Ok(CreatedPullRequest {
                id: "1".to_string(),
                url: format!("https://example.com/pr/1?branch={}", params.head),
                number: 1,
            })
        }
        fn into_tools(self: Box<Self>) -> Vec<Box<dyn agent::tools::Tool>> {
            vec![]
        }
    }

    // ---- tests ----

    #[tokio::test]
    async fn test_code_agent_success() {
        let xml = "<file path=\"src/lib.rs\">\npub fn hello() {}\n</file>";

        let agent = CodeAgent::builder()
            .with_provider(MockProvider {
                response: xml.to_string(),
            })
            .with_reviewer(ApproveReviewer)
            .with_platform(MockPlatform)
            .build()
            .unwrap();

        let result = agent
            .run("## Changes\n- `src/lib.rs`:\n  - **CREATE**: Hello function.\n")
            .await
            .unwrap();

        assert!(result.contains("https://example.com/pr/1"));
    }

    #[tokio::test]
    async fn test_code_agent_malformed_xml_error() {
        let agent = CodeAgent::builder()
            .with_provider(MockProvider {
                response: "no xml blocks here".to_string(),
            })
            .with_reviewer(ApproveReviewer)
            .with_platform(MockPlatform)
            .build()
            .unwrap();

        let err = agent
            .run("## Changes\n- `src/lib.rs`:\n  - **CREATE**: x\n")
            .await
            .unwrap_err();
        assert!(
            matches!(err, NovaError::MalformedLLMResponse(_)),
            "expected MalformedLLMResponse, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_code_agent_max_revisions_exceeded() {
        use crate::agents::review::ReviewIssue;

        struct AlwaysRejectReviewer;

        #[async_trait::async_trait]
        impl Reviewer for AlwaysRejectReviewer {
            async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
                Ok(ReviewVerdict::NeedsRevision {
                    issues: vec![ReviewIssue {
                        severity: Severity::High,
                        description: "always broken".to_string(),
                        suggestion: "fix it".to_string(),
                        location: None,
                    }],
                })
            }
        }

        let xml = "<file path=\"src/lib.rs\">\npub fn hello() {}\n</file>";

        let agent = CodeAgent::builder()
            .with_provider(MockProvider {
                response: xml.to_string(),
            })
            .with_reviewer(AlwaysRejectReviewer)
            .with_platform(MockPlatform)
            .with_max_revisions(2)
            .build()
            .unwrap();

        let err = agent
            .run("## Changes\n- `src/lib.rs`:\n  - **CREATE**: x\n")
            .await
            .unwrap_err();
        assert!(
            matches!(err, NovaError::MaxRevisionsExceeded(2)),
            "expected MaxRevisionsExceeded(2), got {:?}",
            err
        );
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = CodeAgent::builder()
            .with_platform(MockPlatform)
            .build()
            .unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
    }

    #[test]
    fn test_builder_missing_platform() {
        let err = CodeAgent::builder()
            .with_provider(MockProvider {
                response: String::new(),
            })
            .build()
            .unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
    }

    #[test]
    fn test_default_config() {
        let config = CodeAgentConfig::default();
        assert_eq!(config.max_revisions, 3);
        assert_eq!(config.base_branch, "main");
        assert!(!config.branch_prefix.is_empty());
    }
}

// CODEGEN-END

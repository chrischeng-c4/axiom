---
id: implementation
type: change_implementation
change_id: change-spec-agent
---

# Implementation

## Summary

New ChangeSpecAgent in crates/cclab-agent across 3 files: (1) agents/change_spec.rs (new, ~754 lines): opinionated spec generation agent implementing the Agent trait; ChangeSpecInput (StructuredIssue + ReferenceContextOutput), ChangeSpecAgentConfig with defaults (claude-sonnet-4-20250514, max_tokens=8192, temp=0.3, max_retries=2), ChangeSpecAgent with generate_spec() and revise_spec() methods, complete_text() with retry logic on empty LLM responses, SDD system prompt enforcing format priority (OpenRPC>JSON Schema>Mermaid>YAML>Markdown>Prose), diagram selection heuristics (FSM→stateDiagram-v2, DAG→flowchart, objects→classDiagram, actors→sequenceDiagram), mandatory 7-section structure, dual CRR role dispatch via Agent::run() (JSON→creator, plain-text→reviser), ChangeSpecAgentBuilder with fluent API, 12 unit tests with MockProvider and SequenceProvider covering creator/reviser roles, retry exhaustion, JSON round-trips, and builder validation; (2) agents/mod.rs: added pub mod change_spec and re-exports for ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput; (3) lib.rs: re-exported all 4 types at crate root. Also added cclab/specs/cclab-agent/agents/change-spec-agent.md (new): formal main spec with R1-R7 requirements, 6 scenarios, sequence+class diagrams, JSON Schema for ChangeSpecInput and ChangeSpecAgentConfig, 12-item test plan, and changes list.

## Diff

```diff
diff --git a/crates/cclab-agent/src/agents/mod.rs b/crates/cclab-agent/src/agents/mod.rs
index 590c0dde..1d38c4a5 100644
--- a/crates/cclab-agent/src/agents/mod.rs
+++ b/crates/cclab-agent/src/agents/mod.rs
@@ -1,6 +1,7 @@
 //! Agent module - defines the Agent trait and agent implementations.
 
 mod analyst;
+pub mod change_spec;
 mod coding;
 pub mod crr;
 pub mod reference_context;
@@ -8,6 +9,7 @@ pub mod review;
 mod restructure;
 
 pub use analyst::{AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig};
+pub use change_spec::{ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput};
 pub use coding::{CodingAgent, CodingAgentBuilder, CodingAgentConfig};
 pub use crr::{CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType};
 pub use reference_context::{
diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
index 9c4ae458..bd6715b4 100644
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -80,7 +80,9 @@ mod types;
 // Re-export agent types
 pub use agents::{
     Agent, AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig, ApprovalHandler,
-    AutoApproveHandler, CodingAgent, CodingAgentBuilder, CodingAgentConfig,
+    AutoApproveHandler,
+    ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput,
+    CodingAgent, CodingAgentBuilder, CodingAgentConfig,
     CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType,
     Clarification, Contradiction, Question,
     ReferenceContextAgent, ReferenceContextAgentBuilder, ReferenceContextAgentConfig,
diff --git a/crates/cclab-agent/src/agents/change_spec.rs b/crates/cclab-agent/src/agents/change_spec.rs
new file mode 100644
index 00000000..db69a913
--- /dev/null
+++ b/crates/cclab-agent/src/agents/change_spec.rs
@@ -0,0 +1,754 @@
+//! ChangeSpecAgent — generates formal technical specifications from structured issues.
+//!
+//! Operates during SDD phase 6 (change spec). Takes a [`ChangeSpecInput`] containing
+//! a [`StructuredIssue`] and [`ReferenceContextOutput`], generates a complete cclab-sdd
+//! spec document, and integrates with the CRR cycle as both creator and reviser.
+//!
+//! # CRR Integration
+//!
+//! The agent implements [`Agent`] so it can fill both creator and reviser roles in a
+//! [`CRRCycle`]:
+//!
+//! - **Creator**: `run()` receives a JSON-encoded [`ChangeSpecInput`] → calls
+//!   [`generate_spec`][ChangeSpecAgent::generate_spec].
+//! - **Reviser**: `run()` receives the CRR revision prompt (artifact + issues text) →
+//!   calls the LLM directly with the SDD system prompt prepended.
+//!
+//! # SDD Format Rules Enforced
+//!
+//! 1. Format priority: OpenRPC > JSON Schema > Mermaid > YAML > Markdown table > Prose
+//! 2. Diagram selection: FSM → `stateDiagram-v2`, DAG → `flowchart`,
+//!    actors → `sequenceDiagram`, objects → `classDiagram`
+//! 3. Mandatory sections: Overview, Requirements, Scenarios, Diagrams,
+//!    API Spec, Test Plan, Changes
+//! 4. No real Rust/Python/TypeScript implementation code in output
+//! 5. Natural language ≤ 10% of spec content
+//!
+//! # Example
+//!
+//! ```rust,ignore
+//! use cclab_agent::{ChangeSpecAgent, ChangeSpecInput, ClaudeProvider};
+//!
+//! let agent = ChangeSpecAgent::builder()
+//!     .with_provider(provider)
+//!     .build()?;
+//!
+//! let input = ChangeSpecInput { issue, context };
+//! let spec = agent.generate_spec(&input).await?;
+//! println!("{}", spec);
+//! ```
+
+use crate::agents::reference_context::ReferenceContextOutput;
+use crate::agents::restructure::StructuredIssue;
+use crate::agents::review::ReviewIssue;
+use crate::agents::Agent;
+use crate::error::{NovaError, NovaResult};
+use crate::llm::{CompletionRequest, LLMProvider};
+use crate::stream::StreamHandler;
+use crate::types::Message;
+use async_trait::async_trait;
+use serde::{Deserialize, Serialize};
+use std::sync::Arc;
+
+// ============================================================
+// Input type
+// ============================================================
+
+/// Typed input for [`ChangeSpecAgent`] when used as a CRR creator.
+///
+/// Serialized as JSON and passed to [`Agent::run`] by the orchestrator.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ChangeSpecInput {
+    /// The structured issue to resolve with a formal spec.
+    pub issue: StructuredIssue,
+    /// Synthesized reference context from existing project specs.
+    pub context: ReferenceContextOutput,
+}
+
+// ============================================================
+// Agent config
+// ============================================================
+
+/// Configuration for [`ChangeSpecAgent`].
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ChangeSpecAgentConfig {
+    pub model: String,
+    pub max_tokens: Option<u32>,
+    pub temperature: Option<f32>,
+    /// Number of LLM completion retries on empty or error response.
+    pub max_retries: u32,
+}
+
+impl Default for ChangeSpecAgentConfig {
+    fn default() -> Self {
+        Self {
+            model: "claude-sonnet-4-20250514".to_string(),
+            max_tokens: Some(8192),
+            temperature: Some(0.3),
+            max_retries: 2,
+        }
+    }
+}
+
+// ============================================================
+// Agent
+// ============================================================
+
+/// Generates and revises formal technical specifications for SDD phase 6.
+///
+/// # CRR Roles
+///
+/// - **Creator**: receives JSON-encoded [`ChangeSpecInput`] via [`Agent::run`], calls
+///   [`generate_spec`][ChangeSpecAgent::generate_spec].
+/// - **Reviser**: receives CRR revision prompt (artifact + issues) via [`Agent::run`],
+///   produces a revised spec addressing all flagged issues.
+pub struct ChangeSpecAgent {
+    config: ChangeSpecAgentConfig,
+    provider: Arc<dyn LLMProvider>,
+}
+
+impl std::fmt::Debug for ChangeSpecAgent {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("ChangeSpecAgent")
+            .field("config", &self.config)
+            .finish_non_exhaustive()
+    }
+}
+
+impl ChangeSpecAgent {
+    /// Create a new builder.
+    pub fn builder() -> ChangeSpecAgentBuilder {
+        ChangeSpecAgentBuilder::new()
+    }
+
+    /// Generate an initial formal spec from a structured issue and reference context.
+    pub async fn generate_spec(&self, input: &ChangeSpecInput) -> NovaResult<String> {
+        let (system_msg, user_msg) = build_generate_prompt(input);
+        self.complete_text(vec![system_msg, user_msg]).await
+    }
+
+    /// Revise an existing spec to address review issues.
+    pub async fn revise_spec(&self, spec: &str, issues: &[ReviewIssue]) -> NovaResult<String> {
+        let user_content = build_revise_prompt(spec, issues);
+        self.complete_text(vec![
+            Message::system(SYSTEM_PROMPT),
+            Message::user(user_content),
+        ])
+        .await
+    }
+
+    // ---- private ----
+
+    /// Call the LLM and return the text response, retrying on empty responses.
+    async fn complete_text(&self, messages: Vec<Message>) -> NovaResult<String> {
+        let mut request = CompletionRequest::new(messages, &self.config.model);
+        if let Some(temp) = self.config.temperature {
+            request = request.with_temperature(temp);
+        }
+        if let Some(max_tokens) = self.config.max_tokens {
+            request = request.with_max_tokens(max_tokens);
+        }
+
+        let mut last_error = String::new();
+
+        for attempt in 0..=self.config.max_retries {
+            match self.provider.complete(request.clone()).await {
+                Ok(response) => {
+                    let content = response.content.trim().to_string();
+                    if !content.is_empty() {
+                        return Ok(content);
+                    }
+                    last_error = "empty response from LLM".to_string();
+                }
+                Err(e) => {
+                    last_error = e.to_string();
+                }
+            }
+
+            if attempt < self.config.max_retries {
+                request.messages.push(Message::user(format!(
+                    "The previous response was invalid ({}). \
+                     Please generate the complete specification again.",
+                    last_error
+                )));
+            }
+        }
+
+        Err(NovaError::Other(anyhow::anyhow!(
+            "ChangeSpecAgent: failed after {} retries. Last error: {}",
+            self.config.max_retries,
+            last_error
+        )))
+    }
+}
+
+#[async_trait]
+impl Agent for ChangeSpecAgent {
+    /// Run in creator or reviser role.
+    ///
+    /// - **Creator**: input is JSON-encoded [`ChangeSpecInput`] → [`generate_spec`][ChangeSpecAgent::generate_spec].
+    /// - **Reviser**: input is a CRR revision prompt → `complete_text` with system prompt.
+    async fn run(&self, input: &str) -> NovaResult<String> {
+        match serde_json::from_str::<ChangeSpecInput>(input) {
+            Ok(spec_input) => self.generate_spec(&spec_input).await,
+            Err(_) => {
+                // Reviser role: CRR built the revision prompt; prepend the SDD system prompt.
+                self.complete_text(vec![
+                    Message::system(SYSTEM_PROMPT),
+                    Message::user(input.to_string()),
+                ])
+                .await
+            }
+        }
+    }
+
+    async fn run_with_handler(
+        &self,
+        input: &str,
+        _handler: &dyn StreamHandler,
+    ) -> NovaResult<String> {
+        self.run(input).await
+    }
+}
+
+// ============================================================
+// System prompt
+// ============================================================
+
+const SYSTEM_PROMPT: &str = r#"You are an expert specification author for spec-driven development (SDD).
+
+Generate formal technical specifications following these strict rules.
+
+## Format Priority (highest to lowest)
+
+1. OpenRPC JSON — for MCP/API tool definitions
+2. JSON Schema — for data models, state, payloads
+3. Mermaid diagrams — for structure visualization
+4. YAML — for config schemas, CLI command trees
+5. Markdown tables — for mappings, enums, checklists
+6. Prose — ONLY for context that cannot be expressed above
+
+## Diagram Selection
+
+| Structure                    | Diagram           |
+|------------------------------|-------------------|
+| State machine / lifecycle    | `stateDiagram-v2` |
+| Algorithm / DAG / decisions  | `flowchart`       |
+| Object relations / models    | `classDiagram`    |
+| Actor interactions / APIs    | `sequenceDiagram` |
+
+## Mandatory Section Structure (in order)
+
+1. **Overview** — one paragraph summary only
+2. **Requirements** — `R1:`, `R2:`, ... — one sentence each using MUST/MUST NOT/SHOULD
+3. **Scenarios** — WHEN/THEN format, one scenario per acceptance criterion
+4. **Diagrams** — at least one Mermaid diagram using correct type selection above
+5. **API Spec** — OpenRPC or JSON Schema if applicable; omit section if not applicable
+6. **Test Plan** — acceptance criteria as a Markdown checklist
+7. **Changes** — file-level list: `path/to/file.rs` (Create/Modify/Delete): description
+
+## Quality Gates
+
+- Natural language MUST NOT exceed 10% of total spec content
+- NEVER include real Rust, Python, or TypeScript implementation code
+- Use pseudocode blocks if behavior logic must be described:
+  ```
+  FUNCTION name(param: Type) -> Result
+    INPUT: ...
+    OUTPUT: ...
+    ERRORS: ...
+  ```
+- FSMs MUST use `stateDiagram-v2` — NEVER use flowchart for state machines
+- No sections beyond the 7 listed above"#;
+
+// ============================================================
+// Prompt builders
+// ============================================================
+
+fn build_generate_prompt(input: &ChangeSpecInput) -> (Message, Message) {
+    let system_msg = Message::system(SYSTEM_PROMPT);
+
+    let issue = &input.issue;
+    let mut content = format!(
+        "## Structured Issue\n\n\
+         - **Title**: {}\n\
+         - **Type**: {}\n\
+         - **Priority**: {}\n\
+         - **Scope**: {}\n\n\
+         **Description**:\n{}\n\n\
+         **Acceptance Criteria**:\n",
+        issue.title, issue.issue_type, issue.priority, issue.scope, issue.description
+    );
+
+    for (i, criterion) in issue.acceptance_criteria.iter().enumerate() {
+        content.push_str(&format!("{}. {}\n", i + 1, criterion));
+    }
+
+    if !issue.labels.is_empty() {
+        content.push_str("\n**Labels**: ");
+        content.push_str(&issue.labels.join(", "));
+        content.push('\n');
+    }
+
+    if !issue.depends_on.is_empty() {
+        content.push_str("\n**Depends On**: ");
+        content.push_str(&issue.depends_on.join(", "));
+        content.push('\n');
+    }
+
+    content.push_str("\n## Reference Context\n\n");
+
+    if input.context.specs.is_empty() {
+        content.push_str("(No relevant specs found)\n\n");
+    } else {
+        for spec in &input.context.specs {
+            let relevance = format!("{:?}", spec.relevance).to_lowercase();
+            if spec.key_requirements.is_empty() {
+                content.push_str(&format!("- `{}` ({})\n", spec.spec_id, relevance));
+            } else {
+                content.push_str(&format!(
+                    "- `{}` ({}): {}\n",
+                    spec.spec_id,
+                    relevance,
+                    spec.key_requirements.join("; ")
+                ));
+            }
+        }
+        content.push('\n');
+    }
+
+    if !input.context.contradictions.is_empty() {
+        content.push_str("### Contradictions to Resolve\n\n");
+        for c in &input.context.contradictions {
+            content.push_str(&format!(
+                "- `{}`: `{}` — Conflict: {} — Resolution: {}\n",
+                c.spec_id, c.requirement, c.conflict, c.resolution
+            ));
+        }
+        content.push('\n');
+    }
+
+    content.push_str(
+        "Generate a complete technical specification for this issue \
+         following all SDD format rules and the mandatory section structure.",
+    );
+
+    (system_msg, Message::user(content))
+}
+
+fn build_revise_prompt(spec: &str, issues: &[ReviewIssue]) -> String {
+    let mut prompt = format!(
+        "Revise the following specification to address all review issues listed below.\n\n\
+         ## Original Spec\n\n{}\n\n\
+         ## Review Issues\n",
+        spec
+    );
+
+    for (i, issue) in issues.iter().enumerate() {
+        prompt.push_str(&format!(
+            "\n{}. [{}] {}\n   Suggestion: {}\n",
+            i + 1,
+            issue.severity,
+            issue.description,
+            issue.suggestion,
+        ));
+        if let Some(ref loc) = issue.location {
+            prompt.push_str(&format!("   Location: {}\n", loc));
+        }
+    }
+
+    prompt.push_str(
+        "\nAddress every issue above. Return the fully revised specification, \
+         maintaining the mandatory section structure and all SDD format rules.",
+    );
+    prompt
+}
+
+// ============================================================
+// Builder
+// ============================================================
+
+/// Builder for [`ChangeSpecAgent`].
+pub struct ChangeSpecAgentBuilder {
+    config: ChangeSpecAgentConfig,
+    provider: Option<Arc<dyn LLMProvider>>,
+}
+
+impl ChangeSpecAgentBuilder {
+    pub fn new() -> Self {
+        Self {
+            config: ChangeSpecAgentConfig::default(),
+            provider: None,
+        }
+    }
+
+    pub fn with_provider<P: LLMProvider + 'static>(mut self, provider: P) -> Self {
+        self.provider = Some(Arc::new(provider));
+        self
+    }
+
+    pub fn with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self {
+        self.provider = Some(provider);
+        self
+    }
+
+    pub fn with_model(mut self, model: impl Into<String>) -> Self {
+        self.config.model = model.into();
+        self
+    }
+
+    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
+        self.config.max_tokens = Some(max_tokens);
+        self
+    }
+
+    pub fn with_temperature(mut self, temperature: f32) -> Self {
+        self.config.temperature = Some(temperature);
+        self
+    }
+
+    pub fn with_max_retries(mut self, n: u32) -> Self {
+        self.config.max_retries = n;
+        self
+    }
+
+    pub fn build(self) -> NovaResult<ChangeSpecAgent> {
+        let provider = self.provider.ok_or_else(|| {
+            NovaError::ConfigError("ChangeSpecAgent: provider is required".to_string())
+        })?;
+        Ok(ChangeSpecAgent { config: self.config, provider })
+    }
+}
+
+impl Default for ChangeSpecAgentBuilder {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+// ============================================================
+// Tests
+// ============================================================
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::agents::reference_context::{
+        Contradiction, RelevanceLevel, ReferenceContextOutput, SpecReferenceEntry,
+    };
+    use crate::agents::restructure::StructuredIssue;
+    use crate::agents::review::{ReviewIssue, Severity};
+    use crate::llm::{CompletionRequest, CompletionResponse, StreamResponse};
+    use crate::types::TokenUsage;
+    use async_trait::async_trait;
+    use std::collections::HashMap;
+    use std::sync::Mutex;
+
+    // ---- Mock LLM ----
+
+    struct MockProvider {
+        response: String,
+    }
+
+    #[async_trait]
+    impl LLMProvider for MockProvider {
+        fn provider_name(&self) -> &str {
+            "openai"
+        }
+        fn supported_models(&self) -> Vec<String> {
+            vec!["mock-model".to_string()]
+        }
+        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
+            Ok(CompletionResponse {
+                content: self.response.clone(),
+                tool_calls: None,
+                finish_reason: "stop".to_string(),
+                usage: TokenUsage::default(),
+                model: "mock-model".to_string(),
+                metadata: HashMap::new(),
+            })
+        }
+        async fn complete_stream(
+            &self,
+            _req: CompletionRequest,
+        ) -> NovaResult<StreamResponse> {
+            unimplemented!()
+        }
+    }
+
+    /// Provider that returns responses from a scripted sequence.
+    struct SequenceProvider {
+        responses: Mutex<Vec<String>>,
+        fallback: String,
+    }
+
+    impl SequenceProvider {
+        fn new(responses: Vec<String>, fallback: &str) -> Self {
+            Self {
+                responses: Mutex::new(responses),
+                fallback: fallback.to_string(),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl LLMProvider for SequenceProvider {
+        fn provider_name(&self) -> &str {
+            "openai"
+        }
+        fn supported_models(&self) -> Vec<String> {
+            vec!["mock-model".to_string()]
+        }
+        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
+            let mut responses = self.responses.lock().unwrap();
+            let content = if responses.is_empty() {
+                self.fallback.clone()
+            } else {
+                responses.remove(0)
+            };
+            Ok(CompletionResponse {
+                content,
+                tool_calls: None,
+                finish_reason: "stop".to_string(),
+                usage: TokenUsage::default(),
+                model: "mock-model".to_string(),
+                metadata: HashMap::new(),
+            })
+        }
+        async fn complete_stream(
+            &self,
+            _req: CompletionRequest,
+        ) -> NovaResult<StreamResponse> {
+            unimplemented!()
+        }
+    }
+
+    // ---- Helpers ----
+
+    const DRAFT_SPEC: &str = "\
+## Overview\n\
+\n\
+Draft spec overview.\n\
+\n\
+## Requirements\n\
+\n\
+R1: The agent MUST generate specs.\n\
+\n\
+## Changes\n\
+\n\
+- `crates/foo/src/lib.rs` (Create): Initial implementation.";
+
+    fn make_issue() -> StructuredIssue {
+        StructuredIssue {
+            title: "feat(agent): add ChangeSpecAgent".to_string(),
+            description: "Implement ChangeSpecAgent for SDD phase 6.".to_string(),
+            issue_type: "feature".to_string(),
+            priority: "P1".to_string(),
+            labels: vec!["crate:agent".to_string()],
+            acceptance_criteria: vec![
+                "Agent generates spec from StructuredIssue and ReferenceContext".to_string(),
+                "Agent revises spec given ReviewIssues".to_string(),
+            ],
+            depends_on: vec![],
+            scope: "medium".to_string(),
+        }
+    }
+
+    fn make_context() -> ReferenceContextOutput {
+        ReferenceContextOutput {
+            specs: vec![SpecReferenceEntry {
+                spec_id: "cclab-agent/agents.md".to_string(),
+                spec_group: "cclab-agent".to_string(),
+                relevance: RelevanceLevel::High,
+                key_requirements: vec!["Agent trait must be implemented".to_string()],
+            }],
+            contradictions: vec![],
+        }
+    }
+
+    fn make_review_issue(msg: &str) -> ReviewIssue {
+        ReviewIssue {
+            severity: Severity::High,
+            description: msg.to_string(),
+            suggestion: "Fix it.".to_string(),
+            location: None,
+        }
+    }
+
+    // ---- Tests ----
+
+    #[tokio::test]
+    async fn test_generate_spec_returns_llm_content() {
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(MockProvider { response: DRAFT_SPEC.to_string() })
+            .build()
+            .unwrap();
+
+        let input = ChangeSpecInput { issue: make_issue(), context: make_context() };
+        let result = agent.generate_spec(&input).await.unwrap();
+        assert_eq!(result, DRAFT_SPEC.trim());
+    }
+
+    #[tokio::test]
+    async fn test_revise_spec_returns_revised_content() {
+        let revised = "## Overview\n\nRevised overview.\n\n## Requirements\n\nR1: Updated.";
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(MockProvider { response: revised.to_string() })
+            .build()
+            .unwrap();
+
+        let issues = vec![make_review_issue("Missing diagram section")];
+        let result = agent.revise_spec(DRAFT_SPEC, &issues).await.unwrap();
+        assert_eq!(result, revised.trim());
+    }
+
+    #[tokio::test]
+    async fn test_run_creator_role_via_json_input() {
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(MockProvider { response: DRAFT_SPEC.to_string() })
+            .build()
+            .unwrap();
+
+        let input = ChangeSpecInput { issue: make_issue(), context: make_context() };
+        let json = serde_json::to_string(&input).unwrap();
+        let result = agent.run(&json).await.unwrap();
+        assert_eq!(result, DRAFT_SPEC.trim());
+    }
+
+    #[tokio::test]
+    async fn test_run_reviser_role_via_text_prompt() {
+        let revised = "## Overview\n\nRevised.\n\n## Requirements\n\nR1: Fixed.";
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(MockProvider { response: revised.to_string() })
+            .build()
+            .unwrap();
+
+        // CRR revision prompt — plain text, not JSON
+        let prompt = format!(
+            "Revise the following artifact based on the review issues listed below.\n\n\
+             ## Original Artifact\n\n{}\n\n\
+             ## Review Issues\n\n\
+             1. [High] Missing diagrams\n   Suggestion: Add them",
+            DRAFT_SPEC
+        );
+        let result = agent.run(&prompt).await.unwrap();
+        assert_eq!(result, revised.trim());
+    }
+
+    #[tokio::test]
+    async fn test_empty_response_triggers_retry() {
+        // First call returns empty; second call returns valid content.
+        let provider = SequenceProvider::new(
+            vec!["   ".to_string()],
+            DRAFT_SPEC,
+        );
+
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(provider)
+            .with_max_retries(1)
+            .build()
+            .unwrap();
+
+        let input = ChangeSpecInput { issue: make_issue(), context: make_context() };
+        let result = agent.generate_spec(&input).await.unwrap();
+        assert_eq!(result, DRAFT_SPEC.trim());
+    }
+
+    #[tokio::test]
+    async fn test_all_retries_exhausted_returns_error() {
+        // Always returns empty.
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(MockProvider { response: "   ".to_string() })
+            .with_max_retries(1)
+            .build()
+            .unwrap();
+
+        let input = ChangeSpecInput { issue: make_issue(), context: make_context() };
+        let err = agent.generate_spec(&input).await.unwrap_err();
+        let msg = err.to_string();
+        assert!(msg.contains("ChangeSpecAgent"), "expected agent error, got: {}", msg);
+    }
+
+    #[tokio::test]
+    async fn test_generate_spec_with_contradictions() {
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(MockProvider { response: DRAFT_SPEC.to_string() })
+            .build()
+            .unwrap();
+
+        let context = ReferenceContextOutput {
+            specs: vec![],
+            contradictions: vec![Contradiction {
+                spec_id: "agent.md".to_string(),
+                requirement: "must use async".to_string(),
+                conflict: "existing is sync-only".to_string(),
+                resolution: "migrate to async_trait".to_string(),
+            }],
+        };
+
+        let input = ChangeSpecInput { issue: make_issue(), context };
+        let result = agent.generate_spec(&input).await.unwrap();
+        assert!(!result.is_empty());
+    }
+
+    #[tokio::test]
+    async fn test_revise_spec_with_location_in_issue() {
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(MockProvider { response: DRAFT_SPEC.to_string() })
+            .build()
+            .unwrap();
+
+        let issues = vec![ReviewIssue {
+            severity: Severity::High,
+            description: "Missing classDiagram".to_string(),
+            suggestion: "Add a classDiagram for the agent".to_string(),
+            location: Some("spec.md:Diagrams".to_string()),
+        }];
+
+        let result = agent.revise_spec(DRAFT_SPEC, &issues).await.unwrap();
+        assert!(!result.is_empty());
+    }
+
+    #[test]
+    fn test_builder_missing_provider_returns_config_error() {
+        let err = ChangeSpecAgent::builder().build().unwrap_err();
+        assert!(matches!(err, NovaError::ConfigError(_)));
+        assert!(err.to_string().contains("provider"));
+    }
+
+    #[test]
+    fn test_change_spec_input_round_trips_json() {
+        let input = ChangeSpecInput { issue: make_issue(), context: make_context() };
+        let json = serde_json::to_string(&input).unwrap();
+        let parsed: ChangeSpecInput = serde_json::from_str(&json).unwrap();
+        assert_eq!(parsed.issue.title, input.issue.title);
+        assert_eq!(parsed.context.specs.len(), 1);
+    }
+
+    #[test]
+    fn test_config_defaults() {
+        let config = ChangeSpecAgentConfig::default();
+        assert_eq!(config.model, "claude-sonnet-4-20250514");
+        assert_eq!(config.max_tokens, Some(8192));
+        assert_eq!(config.temperature, Some(0.3));
+        assert_eq!(config.max_retries, 2);
+    }
+
+    #[test]
+    fn test_builder_with_overrides() {
+        let provider = MockProvider { response: String::new() };
+        let agent = ChangeSpecAgent::builder()
+            .with_provider(provider)
+            .with_model("gemini-2.0-flash")
+            .with_max_tokens(4096)
+            .with_temperature(0.5)
+            .with_max_retries(1)
+            .build()
+            .unwrap();
+
+        assert_eq!(agent.config.model, "gemini-2.0-flash");
+        assert_eq!(agent.config.max_tokens, Some(4096));
+        assert_eq!(agent.config.temperature, Some(0.5));
+        assert_eq!(agent.config.max_retries, 1);
+    }
+}
diff --git a/cclab/specs/cclab-agent/agents/change-spec-agent.md b/cclab/specs/cclab-agent/agents/change-spec-agent.md
new file mode 100644
index 00000000..c17faa7b
--- /dev/null
+++ b/cclab/specs/cclab-agent/agents/change-spec-agent.md
@@ -0,0 +1,189 @@
+# ChangeSpecAgent
+
+## Overview
+
+`ChangeSpecAgent` is a builder agent for SDD phase 6. It accepts a `ChangeSpecInput`
+(containing a `StructuredIssue` and `ReferenceContextOutput`) and produces a formal
+technical specification conforming to the cclab-sdd format rules. It operates as both
+creator and reviser in a CRR cycle coordinated by an external orchestrator.
+
+## Requirements
+
+R1: The agent MUST accept a `ChangeSpecInput` containing a `StructuredIssue` and
+`ReferenceContextOutput` and return a complete formal specification string.
+
+R2: The agent MUST enforce cclab-sdd format priority:
+OpenRPC > JSON Schema > Mermaid > YAML > Markdown table > Prose.
+Natural language MUST NOT exceed 10% of spec content.
+
+R3: The agent MUST select Mermaid diagram types by structure:
+FSM → `stateDiagram-v2`, DAG → `flowchart`, objects → `classDiagram`,
+actors → `sequenceDiagram`.
+
+R4: Generated specs MUST contain all mandatory sections in order:
+Overview, Requirements, Scenarios, Diagrams, API Spec, Test Plan, Changes.
+Extraneous sections MUST NOT appear.
+
+R5: The agent MUST implement the `Agent` trait to integrate with `CRRCycle` as both
+creator (JSON-encoded `ChangeSpecInput` input) and reviser (CRR revision prompt input).
+
+R6: The agent MUST NEVER emit real Rust, Python, or TypeScript implementation code.
+Logic MUST be expressed as pseudocode, interfaces, or formal schemas.
+
+R7: The agent MUST retry up to `max_retries` times when the LLM returns an empty response.
+
+## Scenarios
+
+### Scenario: Happy Path Spec Generation
+
+- **WHEN** a `ChangeSpecInput` with a valid `StructuredIssue` and populated
+  `ReferenceContextOutput` is provided
+- **THEN** the agent returns a non-empty spec string containing all mandatory sections
+
+### Scenario: Creator Role via JSON Input
+
+- **WHEN** `Agent::run()` receives a JSON-encoded `ChangeSpecInput`
+- **THEN** the agent calls `generate_spec` and returns the initial draft spec
+
+### Scenario: Reviser Role via CRR Prompt
+
+- **WHEN** `Agent::run()` receives a plain-text CRR revision prompt (not valid JSON)
+- **THEN** the agent calls the LLM with the SDD system prompt prepended and returns
+  the revised spec
+
+### Scenario: CRR Rejection Recovery (Excessive Prose)
+
+- **WHEN** the ReviewAgent returns `NeedsRevision` with a "excessive prose" issue
+- **THEN** `revise_spec` is called with the flagged issues and the agent replaces
+  natural language paragraphs with Mermaid diagrams or JSON Schema blocks
+
+### Scenario: Abstraction Enforcement Recovery
+
+- **WHEN** the ReviewAgent flags real implementation code in the spec with severity HIGH
+- **THEN** `revise_spec` rewrites the section using pseudocode blocks
+  (`FUNCTION name(params) -> Result`) with INPUT/OUTPUT/ERRORS annotations
+
+### Scenario: Empty Response Retry
+
+- **WHEN** the LLM returns an empty response on the first attempt
+- **THEN** the agent retries up to `max_retries` times; on success returns valid content;
+  on exhaustion returns `NovaError::Other`
+
+## Diagrams
+
+### Sequence Diagram: CRR Orchestration
+
+```mermaid
+sequenceDiagram
+    participant O as Orchestrator
+    participant S as ChangeSpecAgent
+    participant R as ReviewAgent
+
+    O->>S: run(ChangeSpecInput JSON)
+    S-->>O: Draft Spec
+    O->>R: review(Draft Spec)
+
+    alt NeedsRevision
+        R-->>O: ReviewVerdict::NeedsRevision(issues)
+        O->>S: run(CRR revision prompt)
+        S-->>O: Revised Spec
+        O->>R: review(Revised Spec)
+        R-->>O: ReviewVerdict::Approved
+    else Approved
+        R-->>O: ReviewVerdict::Approved
+    end
+```
+
+### Class Diagram: Agent Structure
+
+```mermaid
+classDiagram
+    class ChangeSpecAgent {
+        -config: ChangeSpecAgentConfig
+        -provider: Arc~dyn LLMProvider~
+        +builder() ChangeSpecAgentBuilder
+        +generate_spec(input: ChangeSpecInput) String
+        +revise_spec(spec: String, issues: Vec~ReviewIssue~) String
+        -complete_text(messages: Vec~Message~) String
+    }
+
+    class ChangeSpecInput {
+        +issue: StructuredIssue
+        +context: ReferenceContextOutput
+    }
+
+    class ChangeSpecAgentConfig {
+        +model: String
+        +max_tokens: Option~u32~
+        +temperature: Option~f32~
+        +max_retries: u32
+    }
+
+    class Agent {
+        <<trait>>
+        +run(input: str) String
+        +run_with_handler(input: str, handler: StreamHandler) String
+    }
+
+    Agent <|.. ChangeSpecAgent
+    ChangeSpecAgent --> ChangeSpecInput : consumes
+    ChangeSpecAgent --> ChangeSpecAgentConfig : configured by
+```
+
+## API Spec
+
+### `ChangeSpecInput` — JSON Schema
+
+```json
+{
+  "type": "object",
+  "required": ["issue", "context"],
+  "properties": {
+    "issue": { "$ref": "#/definitions/StructuredIssue" },
+    "context": { "$ref": "#/definitions/ReferenceContextOutput" }
+  },
+  "additionalProperties": false
+}
+```
+
+### `ChangeSpecAgentConfig` — JSON Schema
+
+```json
+{
+  "type": "object",
+  "required": ["model", "max_retries"],
+  "properties": {
+    "model":       { "type": "string", "default": "claude-sonnet-4-20250514" },
+    "max_tokens":  { "type": "integer", "minimum": 1 },
+    "temperature": { "type": "number", "minimum": 0.0, "maximum": 2.0 },
+    "max_retries": { "type": "integer", "minimum": 0, "default": 2 }
+  },
+  "additionalProperties": false
+}
+```
+
+## Test Plan
+
+- [ ] `test_generate_spec_returns_llm_content` — happy path: returns trimmed LLM output
+- [ ] `test_revise_spec_returns_revised_content` — revise path: returns revised content
+- [ ] `test_run_creator_role_via_json_input` — `run()` routes to `generate_spec` on valid JSON
+- [ ] `test_run_reviser_role_via_text_prompt` — `run()` routes to LLM on plain-text prompt
+- [ ] `test_empty_response_triggers_retry` — retries on empty LLM response
+- [ ] `test_all_retries_exhausted_returns_error` — returns `NovaError::Other` on exhaustion
+- [ ] `test_generate_spec_with_contradictions` — handles contradictions in context
+- [ ] `test_revise_spec_with_location_in_issue` — includes location in revision prompt
+- [ ] `test_builder_missing_provider_returns_config_error` — builder validates required fields
+- [ ] `test_change_spec_input_round_trips_json` — `ChangeSpecInput` JSON round-trip
+- [ ] `test_config_defaults` — `ChangeSpecAgentConfig::default()` values
+- [ ] `test_builder_with_overrides` — builder setters apply correctly
+
+## Changes
+
+- `crates/cclab-agent/src/agents/change_spec.rs` (Create): `ChangeSpecAgent`,
+  `ChangeSpecAgentBuilder`, `ChangeSpecAgentConfig`, `ChangeSpecInput` with full
+  `Agent` trait impl, `generate_spec`, `revise_spec`, and unit tests.
+- `crates/cclab-agent/src/agents/mod.rs` (Modify): Export `change_spec` module and
+  `ChangeSpecAgent`, `ChangeSpecAgentBuilder`, `ChangeSpecAgentConfig`, `ChangeSpecInput`.
+- `crates/cclab-agent/src/lib.rs` (Modify): Re-export `ChangeSpecAgent`,
+  `ChangeSpecAgentBuilder`, `ChangeSpecAgentConfig`, `ChangeSpecInput` at crate root.
+- `cclab/specs/cclab-agent/agents/change-spec-agent.md` (Create): This spec file.

```

## Review: change-spec-agent-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: change-spec-agent

**Summary**: 12 tests pass, compiles clean, 147 total.


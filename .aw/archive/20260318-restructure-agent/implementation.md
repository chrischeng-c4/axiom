---
id: implementation
type: change_implementation
change_id: restructure-agent
---

# Implementation

## Summary

New RestructureAgent in crates/cclab-agent across 3 files: (1) agents/restructure.rs (new, 663 lines): stateless agent with typed I/O (RestructureInput, RestructureOutput discriminated union), SpecStore trait for spec context injection, prompt assembly with intent + clarification history + spec excerpts, complete_structured() call with JSON Schema-enforced output, RestructureAgentBuilder with fluent API; (2) agents/mod.rs: added mod restructure and re-exports for all public types; (3) lib.rs: re-exported RestructureAgent and all associated types at crate root. 8 unit tests cover NeedClarification/CreateIssues variants, mock LLM and SpecStore injection, builder validation, and serde round-trips.

## Diff

```diff
diff --git a/crates/cclab-agent/src/agents/mod.rs b/crates/cclab-agent/src/agents/mod.rs
index 2dfca020..ebf139d1 100644
--- a/crates/cclab-agent/src/agents/mod.rs
+++ b/crates/cclab-agent/src/agents/mod.rs
@@ -2,9 +2,14 @@
 
 mod analyst;
 mod coding;
+mod restructure;
 
 pub use analyst::{AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig};
 pub use coding::{CodingAgent, CodingAgentBuilder, CodingAgentConfig};
+pub use restructure::{
+    Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
+    RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
+};
 
 use crate::error::NovaResult;
 use crate::stream::StreamHandler;
diff --git a/crates/cclab-agent/src/agents/restructure.rs b/crates/cclab-agent/src/agents/restructure.rs
new file mode 100644
index 00000000..f77f03a3
--- /dev/null
+++ b/crates/cclab-agent/src/agents/restructure.rs
@@ -0,0 +1,663 @@
+//! RestructureAgent — LLM-based prompt refinement with structured I/O.
+//!
+//! Takes a user's vague intent, enriches it with spec context, and produces
+//! either clarifying questions or well-structured issues.
+
+use crate::error::{NovaError, NovaResult};
+use crate::llm::{CompletionRequest, LLMProvider};
+use crate::structured::complete_structured;
+use crate::types::Message;
+use async_trait::async_trait;
+use serde::{Deserialize, Serialize};
+use serde_json::Value;
+use std::sync::Arc;
+
+// ============================================================
+// Input types
+// ============================================================
+
+/// A single clarification Q&A pair from a previous round.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Clarification {
+    pub question: String,
+    pub answer: String,
+}
+
+/// Typed input for the RestructureAgent.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct RestructureInput {
+    /// The user's raw intent or feature request.
+    pub intent: String,
+    /// Project identifier used for SpecStore lookup.
+    pub project_id: String,
+    /// Q&A clarifications from prior rounds (empty on first call).
+    pub clarifications: Vec<Clarification>,
+}
+
+// ============================================================
+// Output types
+// ============================================================
+
+/// A spec-informed clarification question produced when more info is needed.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Question {
+    pub id: String,
+    pub question: String,
+    pub why: String,
+    pub suggestions: Vec<String>,
+}
+
+/// A structured issue produced when sufficient information is available.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct StructuredIssue {
+    pub title: String,
+    pub description: String,
+    pub issue_type: String,
+    pub priority: String,
+    pub labels: Vec<String>,
+    pub acceptance_criteria: Vec<String>,
+    pub depends_on: Vec<String>,
+    pub scope: String,
+}
+
+/// Discriminated-union output from RestructureAgent.
+///
+/// Serializes as `{"type": "need_clarification", ...}` or
+/// `{"type": "create_issues", ...}`.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+#[serde(tag = "type", rename_all = "snake_case")]
+pub enum RestructureOutput {
+    NeedClarification { questions: Vec<Question> },
+    CreateIssues { issues: Vec<StructuredIssue>, summary: String },
+}
+
+// ============================================================
+// SpecStore trait
+// ============================================================
+
+/// A spec excerpt returned by a SpecStore search.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SpecExcerpt {
+    /// File path relative to the project root.
+    pub path: String,
+    /// Relevant excerpt from the spec file.
+    pub content: String,
+    /// Relevance score in [0, 1].
+    pub relevance: f32,
+}
+
+/// Trait for retrieving relevant spec context.
+///
+/// A production implementation (issue #901) queries the cclab spec index.
+/// Tests can inject a mock.
+#[async_trait]
+pub trait SpecStore: Send + Sync {
+    /// Return spec excerpts ranked by relevance to the query.
+    async fn search(&self, query: &str) -> NovaResult<Vec<SpecExcerpt>>;
+}
+
+// ============================================================
+// Agent config
+// ============================================================
+
+/// Configuration for RestructureAgent.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct RestructureAgentConfig {
+    pub model: String,
+    pub max_tokens: Option<u32>,
+    pub temperature: Option<f32>,
+    /// Maximum number of spec excerpts to include in the prompt.
+    pub max_spec_excerpts: usize,
+    /// Number of structured-output retries on validation failure.
+    pub max_retries: u32,
+}
+
+impl Default for RestructureAgentConfig {
+    fn default() -> Self {
+        Self {
+            model: "claude-sonnet-4-20250514".to_string(),
+            max_tokens: Some(4096),
+            temperature: Some(0.3),
+            max_spec_excerpts: 10,
+            max_retries: 2,
+        }
+    }
+}
+
+// ============================================================
+// Agent
+// ============================================================
+
+/// Stateless agent that refines vague user intent into well-formed issues.
+///
+/// # Flow
+///
+/// 1. Search `SpecStore` for relevant spec excerpts.
+/// 2. Assemble a prompt: intent + clarification history + spec excerpts.
+/// 3. Call `complete_structured()` with the output JSON Schema.
+/// 4. Deserialize the validated JSON into [`RestructureOutput`].
+///
+/// The agent has no internal mutable state and is safe to share across tasks.
+pub struct RestructureAgent {
+    config: RestructureAgentConfig,
+    provider: Arc<dyn LLMProvider>,
+    spec_store: Arc<dyn SpecStore>,
+}
+
+impl RestructureAgent {
+    /// Create a new builder.
+    pub fn builder() -> RestructureAgentBuilder {
+        RestructureAgentBuilder::new()
+    }
+
+    /// Run the agent with a typed input and return a typed output.
+    pub async fn run(&self, input: &RestructureInput) -> NovaResult<RestructureOutput> {
+        // 1. Retrieve relevant spec context
+        let specs = self.spec_store.search(&input.intent).await?;
+        let spec_count = specs.len().min(self.config.max_spec_excerpts);
+        let specs = &specs[..spec_count];
+
+        // 2. Build prompt
+        let (system_msg, user_msg) = build_prompt(input, specs);
+        let messages = vec![system_msg, user_msg];
+
+        // 3. Build completion request
+        let mut request = CompletionRequest::new(messages, &self.config.model);
+        if let Some(temp) = self.config.temperature {
+            request = request.with_temperature(temp);
+        }
+        if let Some(max_tokens) = self.config.max_tokens {
+            request = request.with_max_tokens(max_tokens);
+        }
+
+        // 4. Call LLM with JSON Schema-enforced structured output
+        let schema = output_schema();
+        let (_response, value) =
+            complete_structured(self.provider.as_ref(), request, &schema, self.config.max_retries)
+                .await?;
+
+        // 5. Deserialize validated JSON into typed output
+        let output: RestructureOutput = serde_json::from_value(value).map_err(|e| {
+            NovaError::SchemaValidationError(format!("Failed to deserialize output: {}", e))
+        })?;
+
+        Ok(output)
+    }
+}
+
+// ============================================================
+// Prompt assembly
+// ============================================================
+
+const SYSTEM_PROMPT: &str = r#"You are an expert software requirements analyst.
+
+Your job:
+1. Analyze the user's intent against existing project specs.
+2. Decide: is there enough information to produce well-formed issues?
+   - If NO: produce specific, spec-informed clarifying questions.
+   - If YES: produce structured issues with acceptance criteria.
+
+Rules for questions:
+- Questions must reference concrete existing specs (traits, types, modules).
+- Never ask generic questions like "what feature do you need?".
+- Include suggestions to help the user choose.
+
+Rules for issues:
+- Each issue title must follow conventional commit format (e.g. "feat(auth): ...").
+- Acceptance criteria must be testable and concrete.
+- Scope: "small" (<1 day), "medium" (1-3 days), "large" (>3 days).
+- Priority: "P0" (critical), "P1" (high), "P2" (medium), "P3" (low).
+
+Output MUST be a JSON object matching the provided schema."#;
+
+fn build_prompt(input: &RestructureInput, specs: &[SpecExcerpt]) -> (Message, Message) {
+    let system_msg = Message::system(SYSTEM_PROMPT);
+
+    let mut content = format!("## Intent\n\n{}\n\n", input.intent);
+
+    if !input.clarifications.is_empty() {
+        content.push_str("## Clarification History\n\n");
+        for c in &input.clarifications {
+            content.push_str(&format!("**Q:** {}\n**A:** {}\n\n", c.question, c.answer));
+        }
+    }
+
+    if !specs.is_empty() {
+        content.push_str("## Relevant Spec Context\n\n");
+        for spec in specs {
+            content.push_str(&format!(
+                "### `{}`\n\n```\n{}\n```\n\n",
+                spec.path, spec.content
+            ));
+        }
+    } else {
+        content.push_str("## Spec Context\n\n(No matching specs found)\n\n");
+    }
+
+    content.push_str("Based on the above, produce a structured JSON output.");
+    (system_msg, Message::user(content))
+}
+
+// ============================================================
+// JSON Schema for structured output
+// ============================================================
+
+fn output_schema() -> Value {
+    serde_json::json!({
+        "type": "object",
+        "required": ["type"],
+        "properties": {
+            "type": {
+                "type": "string",
+                "enum": ["need_clarification", "create_issues"]
+            },
+            "questions": {
+                "type": "array",
+                "items": {
+                    "type": "object",
+                    "required": ["id", "question", "why", "suggestions"],
+                    "properties": {
+                        "id": { "type": "string" },
+                        "question": { "type": "string" },
+                        "why": { "type": "string" },
+                        "suggestions": {
+                            "type": "array",
+                            "items": { "type": "string" }
+                        }
+                    },
+                    "additionalProperties": false
+                }
+            },
+            "issues": {
+                "type": "array",
+                "items": {
+                    "type": "object",
+                    "required": [
+                        "title", "description", "issue_type", "priority",
+                        "labels", "acceptance_criteria", "depends_on", "scope"
+                    ],
+                    "properties": {
+                        "title": { "type": "string" },
+                        "description": { "type": "string" },
+                        "issue_type": { "type": "string" },
+                        "priority": {
+                            "type": "string",
+                            "enum": ["P0", "P1", "P2", "P3"]
+                        },
+                        "labels": {
+                            "type": "array",
+                            "items": { "type": "string" }
+                        },
+                        "acceptance_criteria": {
+                            "type": "array",
+                            "items": { "type": "string" }
+                        },
+                        "depends_on": {
+                            "type": "array",
+                            "items": { "type": "string" }
+                        },
+                        "scope": {
+                            "type": "string",
+                            "enum": ["small", "medium", "large"]
+                        }
+                    },
+                    "additionalProperties": false
+                }
+            },
+            "summary": { "type": "string" }
+        }
+    })
+}
+
+// ============================================================
+// Builder
+// ============================================================
+
+/// Builder for [`RestructureAgent`].
+pub struct RestructureAgentBuilder {
+    config: RestructureAgentConfig,
+    provider: Option<Arc<dyn LLMProvider>>,
+    spec_store: Option<Arc<dyn SpecStore>>,
+}
+
+impl RestructureAgentBuilder {
+    pub fn new() -> Self {
+        Self {
+            config: RestructureAgentConfig::default(),
+            provider: None,
+            spec_store: None,
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
+    pub fn with_spec_store<S: SpecStore + 'static>(mut self, store: S) -> Self {
+        self.spec_store = Some(Arc::new(store));
+        self
+    }
+
+    pub fn with_spec_store_arc(mut self, store: Arc<dyn SpecStore>) -> Self {
+        self.spec_store = Some(store);
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
+    pub fn with_max_spec_excerpts(mut self, n: usize) -> Self {
+        self.config.max_spec_excerpts = n;
+        self
+    }
+
+    pub fn with_max_retries(mut self, n: u32) -> Self {
+        self.config.max_retries = n;
+        self
+    }
+
+    pub fn build(self) -> NovaResult<RestructureAgent> {
+        let provider = self
+            .provider
+            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
+        let spec_store = self
+            .spec_store
+            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
+        Ok(RestructureAgent { config: self.config, provider, spec_store })
+    }
+}
+
+impl Default for RestructureAgentBuilder {
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
+    use crate::llm::{CompletionRequest, CompletionResponse, StreamResponse};
+    use crate::types::TokenUsage;
+    use std::collections::HashMap;
+
+    // --- Mock LLM ---
+
+    struct MockProvider {
+        response_json: String,
+    }
+
+    #[async_trait]
+    impl LLMProvider for MockProvider {
+        fn provider_name(&self) -> &str {
+            // "openai" causes content to be parsed directly as JSON
+            "openai"
+        }
+
+        fn supported_models(&self) -> Vec<String> {
+            vec!["mock-model".to_string()]
+        }
+
+        async fn complete(&self, _request: CompletionRequest) -> NovaResult<CompletionResponse> {
+            Ok(CompletionResponse {
+                content: self.response_json.clone(),
+                tool_calls: None,
+                finish_reason: "stop".to_string(),
+                usage: TokenUsage::default(),
+                model: "mock-model".to_string(),
+                metadata: HashMap::new(),
+            })
+        }
+
+        async fn complete_stream(
+            &self,
+            _request: CompletionRequest,
+        ) -> NovaResult<StreamResponse> {
+            unimplemented!()
+        }
+    }
+
+    // --- Mock SpecStores ---
+
+    struct EmptySpecStore;
+
+    #[async_trait]
+    impl SpecStore for EmptySpecStore {
+        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
+            Ok(vec![])
+        }
+    }
+
+    struct FakeSpecStore;
+
+    #[async_trait]
+    impl SpecStore for FakeSpecStore {
+        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
+            Ok(vec![SpecExcerpt {
+                path: "specs/auth.md".to_string(),
+                content: "AuthProvider trait with login/logout methods.".to_string(),
+                relevance: 0.9,
+            }])
+        }
+    }
+
+    // --- Helpers ---
+
+    fn make_input() -> RestructureInput {
+        RestructureInput {
+            intent: "Add OAuth2 support with Google and GitHub".to_string(),
+            project_id: "cclab".to_string(),
+            clarifications: vec![],
+        }
+    }
+
+    // --- Tests ---
+
+    #[tokio::test]
+    async fn test_run_returns_need_clarification() {
+        let response_json = serde_json::json!({
+            "type": "need_clarification",
+            "questions": [
+                {
+                    "id": "q1",
+                    "question": "Extend AuthProvider or create new OAuth2Provider?",
+                    "why": "Affects whether this modifies existing spec or creates new one",
+                    "suggestions": ["Extend existing", "Create new trait"]
+                }
+            ]
+        })
+        .to_string();
+
+        let agent = RestructureAgent::builder()
+            .with_provider(MockProvider { response_json })
+            .with_spec_store(EmptySpecStore)
+            .build()
+            .unwrap();
+
+        let result = agent.run(&make_input()).await.unwrap();
+
+        match result {
+            RestructureOutput::NeedClarification { questions } => {
+                assert_eq!(questions.len(), 1);
+                assert_eq!(questions[0].id, "q1");
+                assert_eq!(questions[0].suggestions.len(), 2);
+            }
+            _ => panic!("Expected NeedClarification"),
+        }
+    }
+
+    #[tokio::test]
+    async fn test_run_returns_create_issues() {
+        let response_json = serde_json::json!({
+            "type": "create_issues",
+            "issues": [
+                {
+                    "title": "feat(auth): add OAuth2 provider trait",
+                    "description": "Define OAuth2Provider trait with authorize_url, exchange_code, refresh_token.",
+                    "issue_type": "feature",
+                    "priority": "P1",
+                    "labels": ["auth", "oauth2"],
+                    "acceptance_criteria": [
+                        "OAuth2Provider trait compiles",
+                        "Unit tests cover trait contract"
+                    ],
+                    "depends_on": [],
+                    "scope": "medium"
+                }
+            ],
+            "summary": "1 trait definition issue"
+        })
+        .to_string();
+
+        let agent = RestructureAgent::builder()
+            .with_provider(MockProvider { response_json })
+            .with_spec_store(EmptySpecStore)
+            .build()
+            .unwrap();
+
+        let result = agent.run(&make_input()).await.unwrap();
+
+        match result {
+            RestructureOutput::CreateIssues { issues, summary } => {
+                assert_eq!(issues.len(), 1);
+                assert_eq!(issues[0].priority, "P1");
+                assert_eq!(issues[0].scope, "medium");
+                assert_eq!(issues[0].acceptance_criteria.len(), 2);
+                assert!(!summary.is_empty());
+            }
+            _ => panic!("Expected CreateIssues"),
+        }
+    }
+
+    #[tokio::test]
+    async fn test_spec_context_and_clarifications_assembled() {
+        // Use FakeSpecStore to verify specs are fetched; use clarifications in input.
+        let response_json = serde_json::json!({
+            "type": "create_issues",
+            "issues": [
+                {
+                    "title": "feat(auth): add OAuth2",
+                    "description": "OAuth2 support via extension of AuthProvider.",
+                    "issue_type": "feature",
+                    "priority": "P1",
+                    "labels": ["auth"],
+                    "acceptance_criteria": ["Tests pass"],
+                    "depends_on": [],
+                    "scope": "small"
+                }
+            ],
+            "summary": "1 issue"
+        })
+        .to_string();
+
+        let agent = RestructureAgent::builder()
+            .with_provider(MockProvider { response_json })
+            .with_spec_store(FakeSpecStore)
+            .build()
+            .unwrap();
+
+        let input = RestructureInput {
+            intent: "Add OAuth2 support".to_string(),
+            project_id: "cclab".to_string(),
+            clarifications: vec![Clarification {
+                question: "Extend existing trait?".to_string(),
+                answer: "Yes, extend".to_string(),
+            }],
+        };
+
+        let result = agent.run(&input).await.unwrap();
+        assert!(matches!(result, RestructureOutput::CreateIssues { .. }));
+    }
+
+    #[test]
+    fn test_builder_missing_provider_returns_err() {
+        let result = RestructureAgent::builder().with_spec_store(EmptySpecStore).build();
+        assert!(result.is_err());
+        let err = result.err().unwrap().to_string();
+        assert!(err.contains("provider"));
+    }
+
+    #[test]
+    fn test_builder_missing_spec_store_returns_err() {
+        let result = RestructureAgent::builder()
+            .with_provider(MockProvider { response_json: "{}".to_string() })
+            .build();
+        assert!(result.is_err());
+        let err = result.err().unwrap().to_string();
+        assert!(err.contains("SpecStore"));
+    }
+
+    #[test]
+    fn test_restructure_input_round_trips() {
+        let input = RestructureInput {
+            intent: "Add feature X".to_string(),
+            project_id: "proj1".to_string(),
+            clarifications: vec![Clarification {
+                question: "Which module?".to_string(),
+                answer: "auth".to_string(),
+            }],
+        };
+        let json = serde_json::to_string(&input).unwrap();
+        let parsed: RestructureInput = serde_json::from_str(&json).unwrap();
+        assert_eq!(parsed.intent, "Add feature X");
+        assert_eq!(parsed.clarifications.len(), 1);
+        assert_eq!(parsed.clarifications[0].answer, "auth");
+    }
+
+    #[test]
+    fn test_output_serializes_need_clarification_tag() {
+        let output = RestructureOutput::NeedClarification {
+            questions: vec![Question {
+                id: "q1".to_string(),
+                question: "How?".to_string(),
+                why: "Because".to_string(),
+                suggestions: vec!["Option A".to_string()],
+            }],
+        };
+        let json = serde_json::to_value(&output).unwrap();
+        assert_eq!(json["type"], "need_clarification");
+        assert!(json["questions"].is_array());
+    }
+
+    #[test]
+    fn test_output_serializes_create_issues_tag() {
+        let output = RestructureOutput::CreateIssues {
+            issues: vec![StructuredIssue {
+                title: "feat: add X".to_string(),
+                description: "Description".to_string(),
+                issue_type: "feature".to_string(),
+                priority: "P1".to_string(),
+                labels: vec!["enhancement".to_string()],
+                acceptance_criteria: vec!["Test passes".to_string()],
+                depends_on: vec![],
+                scope: "small".to_string(),
+            }],
+            summary: "1 issue".to_string(),
+        };
+        let json = serde_json::to_value(&output).unwrap();
+        assert_eq!(json["type"], "create_issues");
+        assert_eq!(json["summary"], "1 issue");
+        assert_eq!(json["issues"][0]["priority"], "P1");
+    }
+}
diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
index 844af358..13dc766b 100644
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -81,6 +81,8 @@ mod types;
 pub use agents::{
     Agent, AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig, ApprovalHandler,
     AutoApproveHandler, CodingAgent, CodingAgentBuilder, CodingAgentConfig,
+    Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
+    RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
 };
 
 // Re-export context

```

## Review: restructure-agent-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: restructure-agent

**Summary**: Implementation matches spec. RestructureAgent with TypedAgent pattern, SpecStore trait, structured I/O (discriminated union), builder pattern, 8 unit tests all passing. Code compiles with no new warnings.


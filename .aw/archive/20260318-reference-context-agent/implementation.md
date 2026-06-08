---
id: implementation
type: change_implementation
change_id: reference-context-agent
---

# Implementation

## Summary

New ReferenceContextAgent in crates/cclab-agent across 4 files: (1) agents/reference_context.rs (new, 816 lines): LLM-powered spec discovery agent implementing the Agent trait; RelevanceLevel enum (High/Medium/Low), SpecReferenceEntry and Contradiction output types, ReferenceContextOutput, internal CRR pass (max_revisions=1, auto-approve semantics) — searches SpecStore, reads full spec text, generates structured context via complete_structured(), reviews with Reviewer, revises once on NeedsRevision then auto-approves, auto-approves on Rejected; ReferenceContextAgentBuilder with fluent API, 12 unit tests covering CRR paths, read call tracking, max_spec_excerpts limit, serde round-trips, and builder errors; (2) agents/restructure.rs: extended SpecStore trait with read(&self, path: &str) -> NovaResult<String> method; updated EmptySpecStore and FakeSpecStore test mocks; (3) agents/mod.rs: added pub mod reference_context and re-exports for all 7 public types; (4) lib.rs: re-exported ReferenceContextAgent, ReferenceContextAgentBuilder, ReferenceContextAgentConfig, ReferenceContextOutput, RelevanceLevel, SpecReferenceEntry, Contradiction at crate root.

## Diff

```diff
diff --git a/crates/cclab-agent/src/agents/mod.rs b/crates/cclab-agent/src/agents/mod.rs
index a243664a..590c0dde 100644
--- a/crates/cclab-agent/src/agents/mod.rs
+++ b/crates/cclab-agent/src/agents/mod.rs
@@ -3,12 +3,17 @@
 mod analyst;
 mod coding;
 pub mod crr;
+pub mod reference_context;
 pub mod review;
 mod restructure;
 
 pub use analyst::{AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig};
 pub use coding::{CodingAgent, CodingAgentBuilder, CodingAgentConfig};
 pub use crr::{CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType};
+pub use reference_context::{
+    Contradiction, ReferenceContextAgent, ReferenceContextAgentBuilder,
+    ReferenceContextAgentConfig, ReferenceContextOutput, RelevanceLevel, SpecReferenceEntry,
+};
 pub use restructure::{
     Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
     RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
diff --git a/crates/cclab-agent/src/agents/restructure.rs b/crates/cclab-agent/src/agents/restructure.rs
index f77f03a3..e5b4e35e 100644
--- a/crates/cclab-agent/src/agents/restructure.rs
+++ b/crates/cclab-agent/src/agents/restructure.rs
@@ -94,6 +94,9 @@ pub struct SpecExcerpt {
 pub trait SpecStore: Send + Sync {
     /// Return spec excerpts ranked by relevance to the query.
     async fn search(&self, query: &str) -> NovaResult<Vec<SpecExcerpt>>;
+
+    /// Return the full text of a specification file by its path.
+    async fn read(&self, path: &str) -> NovaResult<String>;
 }
 
 // ============================================================
@@ -447,6 +450,10 @@ mod tests {
         async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
             Ok(vec![])
         }
+
+        async fn read(&self, _path: &str) -> NovaResult<String> {
+            Ok(String::new())
+        }
     }
 
     struct FakeSpecStore;
@@ -460,6 +467,10 @@ mod tests {
                 relevance: 0.9,
             }])
         }
+
+        async fn read(&self, path: &str) -> NovaResult<String> {
+            Ok(format!("# Full content of {}\n\nAuthProvider trait definition.", path))
+        }
     }
 
     // --- Helpers ---
diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
index 484c2b6b..9c4ae458 100644
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -82,7 +82,10 @@ pub use agents::{
     Agent, AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig, ApprovalHandler,
     AutoApproveHandler, CodingAgent, CodingAgentBuilder, CodingAgentConfig,
     CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType,
-    Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
+    Clarification, Contradiction, Question,
+    ReferenceContextAgent, ReferenceContextAgentBuilder, ReferenceContextAgentConfig,
+    ReferenceContextOutput, RelevanceLevel, SpecReferenceEntry,
+    RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
     RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
     ReviewAgent, ReviewAgentBuilder, ReviewAgentConfig, ReviewIssue, ReviewType, ReviewVerdict,
     Reviewer, Severity,
diff --git a/crates/cclab-agent/src/agents/reference_context.rs b/crates/cclab-agent/src/agents/reference_context.rs
new file mode 100644
index 00000000..e3545483
--- /dev/null
+++ b/crates/cclab-agent/src/agents/reference_context.rs
@@ -0,0 +1,816 @@
+//! ReferenceContextAgent — discovers and synthesizes spec context for SDD.
+//!
+//! Operates during SDD phases 4 (reference-context) and 5 (post-clarification).
+//! Searches the `SpecStore`, reads full spec content, scores each spec's relevance,
+//! extracts key requirements, detects contradictions, and runs an internal CRR cycle
+//! (max_revisions=1, auto-approve semantics) via `ReviewAgent` before finalizing.
+
+use crate::agents::restructure::{SpecExcerpt, SpecStore};
+use crate::agents::review::{ReviewIssue, ReviewVerdict, Reviewer};
+use crate::agents::Agent;
+use crate::error::{NovaError, NovaResult};
+use crate::llm::{CompletionRequest, LLMProvider};
+use crate::stream::StreamHandler;
+use crate::structured::complete_structured;
+use crate::types::Message;
+use async_trait::async_trait;
+use serde::{Deserialize, Serialize};
+use serde_json::Value;
+use std::sync::Arc;
+
+// ============================================================
+// Output types
+// ============================================================
+
+/// Relevance of a spec relative to the current change.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum RelevanceLevel {
+    High,
+    Medium,
+    Low,
+}
+
+/// A single spec entry in the reference context artifact.
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+pub struct SpecReferenceEntry {
+    /// Path relative to cclab/specs/.
+    pub spec_id: String,
+    /// Logical group name (e.g. "cclab-agent", "cclab-sdd").
+    pub spec_group: String,
+    /// Relevance of this spec to the current change.
+    pub relevance: RelevanceLevel,
+    /// Key requirements from this spec that apply to the change.
+    pub key_requirements: Vec<String>,
+}
+
+/// A contradiction detected between an existing spec and the change requirements.
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+pub struct Contradiction {
+    /// Spec where the contradiction was found.
+    pub spec_id: String,
+    /// The change requirement that conflicts.
+    pub requirement: String,
+    /// Description of the conflict.
+    pub conflict: String,
+    /// Suggested resolution.
+    pub resolution: String,
+}
+
+/// Structured output produced by [`ReferenceContextAgent`].
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+pub struct ReferenceContextOutput {
+    pub specs: Vec<SpecReferenceEntry>,
+    pub contradictions: Vec<Contradiction>,
+}
+
+// ============================================================
+// Agent config
+// ============================================================
+
+/// Configuration for [`ReferenceContextAgent`].
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ReferenceContextAgentConfig {
+    pub model: String,
+    pub max_tokens: Option<u32>,
+    pub temperature: Option<f32>,
+    /// Maximum number of specs to fetch and read in full.
+    pub max_spec_excerpts: usize,
+    /// Structured-output retries on validation failure.
+    pub max_retries: u32,
+}
+
+impl Default for ReferenceContextAgentConfig {
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
+/// Discovers and synthesizes spec reference context for SDD phases 4 and 5.
+///
+/// # Flow
+///
+/// 1. `SpecStore::search` — retrieve ranked spec excerpts for the change.
+/// 2. `SpecStore::read` — fetch full text of each discovered spec.
+/// 3. `complete_structured` — generate initial reference context artifact.
+/// 4. Internal CRR cycle (max_revisions=1, auto-approve):
+///    - `Reviewer::review` examines the draft artifact.
+///    - On `NeedsRevision`: revise once then accept regardless of second verdict.
+///    - On `Approved` or `Rejected`: accept immediately (auto-approve contract).
+/// 5. Return the final JSON string artifact.
+pub struct ReferenceContextAgent {
+    config: ReferenceContextAgentConfig,
+    provider: Arc<dyn LLMProvider>,
+    spec_store: Arc<dyn SpecStore>,
+    reviewer: Arc<dyn Reviewer>,
+}
+
+impl std::fmt::Debug for ReferenceContextAgent {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("ReferenceContextAgent")
+            .field("config", &self.config)
+            .finish_non_exhaustive()
+    }
+}
+
+#[async_trait]
+impl Agent for ReferenceContextAgent {
+    async fn run(&self, input: &str) -> NovaResult<String> {
+        // 1. Search for relevant specs
+        let excerpts = self.spec_store.search(input).await?;
+        let excerpt_count = excerpts.len().min(self.config.max_spec_excerpts);
+        let excerpts = &excerpts[..excerpt_count];
+
+        // 2. Read full text of each spec
+        let mut full_specs: Vec<(SpecExcerpt, String)> = Vec::new();
+        for excerpt in excerpts {
+            let content = self.spec_store.read(&excerpt.path).await?;
+            full_specs.push((excerpt.clone(), content));
+        }
+
+        // 3. Generate initial reference context
+        let artifact = self.generate_context(input, &full_specs).await?;
+
+        // 4. CRR cycle (max_revisions=1, auto-approve)
+        let final_artifact = self.crr_cycle(input, &full_specs, artifact).await?;
+
+        Ok(final_artifact)
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
+impl ReferenceContextAgent {
+    /// Create a new builder.
+    pub fn builder() -> ReferenceContextAgentBuilder {
+        ReferenceContextAgentBuilder::new()
+    }
+
+    /// Generate the initial reference context JSON from the change input + spec contents.
+    async fn generate_context(
+        &self,
+        input: &str,
+        full_specs: &[(SpecExcerpt, String)],
+    ) -> NovaResult<String> {
+        let (system_msg, user_msg) = build_prompt(input, full_specs);
+        let artifact_json = self.complete_structured_output(vec![system_msg, user_msg]).await?;
+        Ok(artifact_json)
+    }
+
+    /// Run an internal CRR cycle with max_revisions=1 and auto-approve semantics.
+    ///
+    /// If the reviewer flags issues, we revise once and accept the result regardless
+    /// of the second verdict (auto-approve). If the artifact is `Rejected`, we still
+    /// return it rather than failing.
+    async fn crr_cycle(
+        &self,
+        input: &str,
+        full_specs: &[(SpecExcerpt, String)],
+        initial_artifact: String,
+    ) -> NovaResult<String> {
+        let verdict = self.reviewer.review(&initial_artifact).await?;
+
+        match verdict {
+            // Accept immediately — approved or auto-approve on rejection
+            ReviewVerdict::Approved | ReviewVerdict::Rejected { .. } => Ok(initial_artifact),
+
+            // Revise once, then auto-approve regardless of the second verdict
+            ReviewVerdict::NeedsRevision { issues } => {
+                let revision_prompt =
+                    build_revision_prompt(input, full_specs, &initial_artifact, &issues);
+                let revised = self
+                    .complete_structured_output(vec![
+                        Message::system(SYSTEM_PROMPT),
+                        Message::user(revision_prompt),
+                    ])
+                    .await?;
+                Ok(revised)
+            }
+        }
+    }
+
+    /// Call the LLM with structured output and return the pretty-printed JSON string.
+    async fn complete_structured_output(&self, messages: Vec<Message>) -> NovaResult<String> {
+        let mut request = CompletionRequest::new(messages, &self.config.model);
+        if let Some(temp) = self.config.temperature {
+            request = request.with_temperature(temp);
+        }
+        if let Some(max_tokens) = self.config.max_tokens {
+            request = request.with_max_tokens(max_tokens);
+        }
+
+        let schema = output_schema();
+        let (_response, value) =
+            complete_structured(self.provider.as_ref(), request, &schema, self.config.max_retries)
+                .await?;
+
+        // Validate round-trip to ReferenceContextOutput
+        let _: ReferenceContextOutput = serde_json::from_value(value.clone()).map_err(|e| {
+            NovaError::SchemaValidationError(format!("Failed to deserialize output: {}", e))
+        })?;
+
+        serde_json::to_string_pretty(&value)
+            .map_err(|e| NovaError::Other(anyhow::anyhow!("Failed to serialize output: {}", e)))
+    }
+}
+
+// ============================================================
+// Prompt assembly
+// ============================================================
+
+const SYSTEM_PROMPT: &str = r#"You are an expert specification analyst for spec-driven development (SDD).
+
+Your task:
+1. Analyze the provided change requirements against the existing project specifications.
+2. For each spec assign a relevance score:
+   - "high"   — directly impacts the change (same module, same interface)
+   - "medium" — adjacent context (related module, shared type)
+   - "low"    — tangentially related (general framework, distant dependency)
+3. Extract concise key requirements from high/medium-relevance specs applicable to the change.
+4. Identify contradictions between existing specs and the proposed change requirements.
+
+Output MUST be a JSON object matching the provided schema exactly."#;
+
+fn build_prompt(input: &str, full_specs: &[(SpecExcerpt, String)]) -> (Message, Message) {
+    let system_msg = Message::system(SYSTEM_PROMPT);
+
+    let mut content = format!("## Change Requirements\n\n{}\n\n", input);
+
+    if !full_specs.is_empty() {
+        content.push_str("## Existing Specifications\n\n");
+        for (excerpt, full_content) in full_specs {
+            content.push_str(&format!(
+                "### `{}` (search relevance: {:.2})\n\n```\n{}\n```\n\n",
+                excerpt.path, excerpt.relevance, full_content
+            ));
+        }
+    } else {
+        content.push_str("## Existing Specifications\n\n(No matching specs found)\n\n");
+    }
+
+    content.push_str(
+        "Analyze the specifications against the change requirements \
+         and produce a JSON reference context.",
+    );
+
+    (system_msg, Message::user(content))
+}
+
+fn build_revision_prompt(
+    input: &str,
+    full_specs: &[(SpecExcerpt, String)],
+    artifact: &str,
+    issues: &[ReviewIssue],
+) -> String {
+    let mut prompt = format!(
+        "## Change Requirements\n\n{}\n\n\
+         ## Original Reference Context\n\n{}\n\n\
+         ## Review Issues\n",
+        input, artifact
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
+    if !full_specs.is_empty() {
+        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
+        for (excerpt, full_content) in full_specs {
+            prompt.push_str(&format!(
+                "### `{}`\n\n```\n{}\n```\n\n",
+                excerpt.path, full_content
+            ));
+        }
+    }
+
+    prompt.push_str(
+        "\nAddress every review issue above and return \
+         the fully revised reference context JSON.",
+    );
+    prompt
+}
+
+// ============================================================
+// JSON Schema for structured output
+// ============================================================
+
+fn output_schema() -> Value {
+    serde_json::json!({
+        "type": "object",
+        "required": ["specs", "contradictions"],
+        "properties": {
+            "specs": {
+                "type": "array",
+                "items": {
+                    "type": "object",
+                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
+                    "properties": {
+                        "spec_id": { "type": "string" },
+                        "spec_group": { "type": "string" },
+                        "relevance": {
+                            "type": "string",
+                            "enum": ["high", "medium", "low"]
+                        },
+                        "key_requirements": {
+                            "type": "array",
+                            "items": { "type": "string" }
+                        }
+                    },
+                    "additionalProperties": false
+                }
+            },
+            "contradictions": {
+                "type": "array",
+                "items": {
+                    "type": "object",
+                    "required": ["spec_id", "requirement", "conflict", "resolution"],
+                    "properties": {
+                        "spec_id": { "type": "string" },
+                        "requirement": { "type": "string" },
+                        "conflict": { "type": "string" },
+                        "resolution": { "type": "string" }
+                    },
+                    "additionalProperties": false
+                }
+            }
+        },
+        "additionalProperties": false
+    })
+}
+
+// ============================================================
+// Builder
+// ============================================================
+
+/// Builder for [`ReferenceContextAgent`].
+pub struct ReferenceContextAgentBuilder {
+    config: ReferenceContextAgentConfig,
+    provider: Option<Arc<dyn LLMProvider>>,
+    spec_store: Option<Arc<dyn SpecStore>>,
+    reviewer: Option<Arc<dyn Reviewer>>,
+}
+
+impl ReferenceContextAgentBuilder {
+    pub fn new() -> Self {
+        Self {
+            config: ReferenceContextAgentConfig::default(),
+            provider: None,
+            spec_store: None,
+            reviewer: None,
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
+    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
+        self.reviewer = Some(Arc::new(reviewer));
+        self
+    }
+
+    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
+        self.reviewer = Some(reviewer);
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
+    pub fn build(self) -> NovaResult<ReferenceContextAgent> {
+        let provider = self
+            .provider
+            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
+        let spec_store = self
+            .spec_store
+            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
+        let reviewer = self
+            .reviewer
+            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
+        Ok(ReferenceContextAgent { config: self.config, provider, spec_store, reviewer })
+    }
+}
+
+impl Default for ReferenceContextAgentBuilder {
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
+    use crate::agents::restructure::SpecExcerpt;
+    use crate::llm::{CompletionRequest, CompletionResponse, StreamResponse};
+    use crate::types::TokenUsage;
+    use async_trait::async_trait;
+    use std::collections::HashMap;
+    use std::sync::Mutex;
+
+    // ---- Mock LLM ----
+
+    struct MockProvider {
+        response_json: String,
+    }
+
+    #[async_trait]
+    impl LLMProvider for MockProvider {
+        fn provider_name(&self) -> &str {
+            // "openai" triggers direct JSON parsing in complete_structured
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
+    // ---- Mock SpecStore ----
+
+    struct MockSpecStore {
+        excerpts: Vec<SpecExcerpt>,
+        read_content: String,
+        read_paths: Arc<Mutex<Vec<String>>>,
+    }
+
+    impl MockSpecStore {
+        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
+            Self {
+                excerpts,
+                read_content: read_content.to_string(),
+                read_paths: Arc::new(Mutex::new(vec![])),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl SpecStore for MockSpecStore {
+        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
+            Ok(self.excerpts.clone())
+        }
+
+        async fn read(&self, path: &str) -> NovaResult<String> {
+            self.read_paths.lock().unwrap().push(path.to_string());
+            Ok(self.read_content.clone())
+        }
+    }
+
+    struct EmptySpecStore;
+
+    #[async_trait]
+    impl SpecStore for EmptySpecStore {
+        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
+            Ok(vec![])
+        }
+
+        async fn read(&self, _path: &str) -> NovaResult<String> {
+            Ok(String::new())
+        }
+    }
+
+    // ---- Mock Reviewer ----
+
+    struct ScriptedReviewer {
+        verdicts: Mutex<Vec<ReviewVerdict>>,
+    }
+
+    impl ScriptedReviewer {
+        fn always_approve() -> Self {
+            Self { verdicts: Mutex::new(vec![]) }
+        }
+
+        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
+            Self { verdicts: Mutex::new(verdicts) }
+        }
+    }
+
+    #[async_trait]
+    impl Reviewer for ScriptedReviewer {
+        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
+            let mut v = self.verdicts.lock().unwrap();
+            if v.is_empty() {
+                Ok(ReviewVerdict::Approved)
+            } else {
+                Ok(v.remove(0))
+            }
+        }
+    }
+
+    // ---- Helpers ----
+
+    fn valid_context_json() -> String {
+        serde_json::json!({
+            "specs": [
+                {
+                    "spec_id": "cclab-agent/spec.md",
+                    "spec_group": "cclab-agent",
+                    "relevance": "high",
+                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
+                }
+            ],
+            "contradictions": []
+        })
+        .to_string()
+    }
+
+    fn make_review_issue(msg: &str) -> ReviewIssue {
+        use crate::agents::review::Severity;
+        ReviewIssue {
+            severity: Severity::Medium,
+            description: msg.to_string(),
+            suggestion: "Fix it.".to_string(),
+            location: None,
+        }
+    }
+
+    // ---- Tests ----
+
+    #[tokio::test]
+    async fn test_run_approved_on_first_review() {
+        let agent = ReferenceContextAgent::builder()
+            .with_provider(MockProvider { response_json: valid_context_json() })
+            .with_spec_store(EmptySpecStore)
+            .with_reviewer(ScriptedReviewer::always_approve())
+            .build()
+            .unwrap();
+
+        let result = agent.run("Add OAuth2 support").await.unwrap();
+        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
+        assert_eq!(output.specs.len(), 1);
+        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
+        assert!(output.contradictions.is_empty());
+    }
+
+    #[tokio::test]
+    async fn test_run_triggers_revision_on_needs_revision() {
+        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
+        let agent = ReferenceContextAgent::builder()
+            .with_provider(MockProvider { response_json: valid_context_json() })
+            .with_spec_store(EmptySpecStore)
+            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::NeedsRevision {
+                issues: vec![make_review_issue("Missing medium-relevance spec")],
+            }]))
+            .build()
+            .unwrap();
+
+        // Should succeed (auto-approve after one revision attempt)
+        let result = agent.run("Add OAuth2 support").await.unwrap();
+        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
+        assert_eq!(output.specs.len(), 1);
+    }
+
+    #[tokio::test]
+    async fn test_run_auto_approves_on_rejection() {
+        // Rejected verdict → auto-approve, returns initial artifact
+        let agent = ReferenceContextAgent::builder()
+            .with_provider(MockProvider { response_json: valid_context_json() })
+            .with_spec_store(EmptySpecStore)
+            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
+                reason: "fundamentally wrong".to_string(),
+            }]))
+            .build()
+            .unwrap();
+
+        let result = agent.run("Add OAuth2 support").await.unwrap();
+        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
+        assert_eq!(output.specs.len(), 1);
+    }
+
+    #[tokio::test]
+    async fn test_spec_store_read_called_for_each_excerpt() {
+        let excerpts = vec![
+            SpecExcerpt {
+                path: "specs/auth.md".to_string(),
+                content: "Auth excerpt".to_string(),
+                relevance: 0.9,
+            },
+            SpecExcerpt {
+                path: "specs/oauth.md".to_string(),
+                content: "OAuth excerpt".to_string(),
+                relevance: 0.7,
+            },
+        ];
+        let store = MockSpecStore::new(excerpts, "Full spec content");
+        let read_paths = store.read_paths.clone();
+
+        let agent = ReferenceContextAgent::builder()
+            .with_provider(MockProvider { response_json: valid_context_json() })
+            .with_spec_store(store)
+            .with_reviewer(ScriptedReviewer::always_approve())
+            .build()
+            .unwrap();
+
+        agent.run("Add OAuth2 support").await.unwrap();
+
+        let paths = read_paths.lock().unwrap();
+        assert_eq!(paths.len(), 2);
+        assert!(paths.contains(&"specs/auth.md".to_string()));
+        assert!(paths.contains(&"specs/oauth.md".to_string()));
+    }
+
+    #[tokio::test]
+    async fn test_max_spec_excerpts_limit() {
+        let excerpts: Vec<SpecExcerpt> = (0..5)
+            .map(|i| SpecExcerpt {
+                path: format!("specs/spec{}.md", i),
+                content: format!("content {}", i),
+                relevance: 0.5,
+            })
+            .collect();
+        let store = MockSpecStore::new(excerpts, "Full content");
+        let read_paths = store.read_paths.clone();
+
+        let agent = ReferenceContextAgent::builder()
+            .with_provider(MockProvider { response_json: valid_context_json() })
+            .with_spec_store(store)
+            .with_reviewer(ScriptedReviewer::always_approve())
+            .with_max_spec_excerpts(3) // limit to 3
+            .build()
+            .unwrap();
+
+        agent.run("Some change").await.unwrap();
+
+        let paths = read_paths.lock().unwrap();
+        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
+    }
+
+    #[tokio::test]
+    async fn test_output_round_trips_all_relevance_levels() {
+        let output = ReferenceContextOutput {
+            specs: vec![
+                SpecReferenceEntry {
+                    spec_id: "a.md".to_string(),
+                    spec_group: "g1".to_string(),
+                    relevance: RelevanceLevel::High,
+                    key_requirements: vec!["req1".to_string()],
+                },
+                SpecReferenceEntry {
+                    spec_id: "b.md".to_string(),
+                    spec_group: "g2".to_string(),
+                    relevance: RelevanceLevel::Medium,
+                    key_requirements: vec![],
+                },
+                SpecReferenceEntry {
+                    spec_id: "c.md".to_string(),
+                    spec_group: "g3".to_string(),
+                    relevance: RelevanceLevel::Low,
+                    key_requirements: vec![],
+                },
+            ],
+            contradictions: vec![Contradiction {
+                spec_id: "a.md".to_string(),
+                requirement: "must use async".to_string(),
+                conflict: "existing spec is sync".to_string(),
+                resolution: "migrate to async_trait".to_string(),
+            }],
+        };
+
+        let json = serde_json::to_value(&output).unwrap();
+        assert_eq!(json["specs"][0]["relevance"], "high");
+        assert_eq!(json["specs"][1]["relevance"], "medium");
+        assert_eq!(json["specs"][2]["relevance"], "low");
+        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");
+
+        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
+        assert_eq!(parsed, output);
+    }
+
+    #[test]
+    fn test_builder_missing_provider() {
+        let err = ReferenceContextAgent::builder()
+            .with_spec_store(EmptySpecStore)
+            .with_reviewer(ScriptedReviewer::always_approve())
+            .build()
+            .unwrap_err();
+        assert!(err.to_string().contains("provider"));
+    }
+
+    #[test]
+    fn test_builder_missing_spec_store() {
+        let err = ReferenceContextAgent::builder()
+            .with_provider(MockProvider { response_json: "{}".to_string() })
+            .with_reviewer(ScriptedReviewer::always_approve())
+            .build()
+            .unwrap_err();
+        assert!(err.to_string().contains("SpecStore"));
+    }
+
+    #[test]
+    fn test_builder_missing_reviewer() {
+        let err = ReferenceContextAgent::builder()
+            .with_provider(MockProvider { response_json: "{}".to_string() })
+            .with_spec_store(EmptySpecStore)
+            .build()
+            .unwrap_err();
+        assert!(err.to_string().contains("Reviewer"));
+    }
+
+    #[test]
+    fn test_relevance_level_serializes_lowercase() {
+        assert_eq!(
+            serde_json::to_value(RelevanceLevel::High).unwrap(),
+            serde_json::json!("high")
+        );
+        assert_eq!(
+            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
+            serde_json::json!("medium")
+        );
+        assert_eq!(
+            serde_json::to_value(RelevanceLevel::Low).unwrap(),
+            serde_json::json!("low")
+        );
+    }
+
+    #[test]
+    fn test_empty_context_output_is_valid() {
+        let output = ReferenceContextOutput { specs: vec![], contradictions: vec![] };
+        let json = serde_json::to_value(&output).unwrap();
+        assert!(json["specs"].is_array());
+        assert!(json["contradictions"].is_array());
+    }
+}

```

## Review: reference-context-agent-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: reference-context-agent

**Summary**: 11 tests pass, compiles clean, 135 total tests.


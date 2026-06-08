---
id: implementation
type: change_implementation
change_id: fillback-agents
---

# Implementation

## Summary

Three-issue fillback agent implementation across 9 files: (1) crates/cclab-agent/src/agents/reference_context.rs (deleted, 816 lines): old ReferenceContextAgent removed; (2) crates/cclab-agent/src/agents/reference_spec_context.rs (new, 816 lines): renamed ReferenceContextAgent → ReferenceSpecContextAgent preserving full behavior — SpecStore search, full spec read, structured-output LLM call, internal CRR cycle (max_revisions=1, auto-approve), JSON schema validation, builder pattern; (3) crates/cclab-agent/src/agents/reference_codebase_context.rs (new, 599 lines): ReferenceCodebaseContextAgent — inner CodingAgent with coding tools (ReadFile, Glob, Grep, Bash) for autonomous codebase exploration; structured ReferenceCodebaseArtifact output (key_files, architectural_patterns, dependencies, relationships, summary); JSON extraction with markdown-fence and brace-matching fallback; retry loop on validation failure; 12 unit tests with MockProvider; (4) crates/cclab-agent/src/agents/codebase_to_spec.rs (new, 746 lines): CodebaseToSpecAgent — creator/reviser dual-role via JSON input detection (CodebaseToSpecInput → generate_spec, plain text → revise role); SDD system prompt enforcing 7-section mandatory structure and format priority; prompt builders for both generate and revise paths; retry on empty LLM response; 13 unit tests including SequenceProvider for retry coverage; (5) crates/cclab-agent/src/agents/mod.rs (modified): registers codebase_to_spec and reference_codebase_context modules, renames reference_context → reference_spec_context, adds backward-compat alias pub use reference_spec_context as reference_context; exports new public types; (6) crates/cclab-agent/src/agents/change_spec.rs (modified): updates import path from reference_context to reference_spec_context; (7) crates/cclab-agent/src/lib.rs (modified): exports CodebaseDependency, CodebaseToSpecAgent/Builder/Config/Input, ComponentRelationship, KeyFile, ReferenceCodebaseArtifact, ReferenceCodebaseContext{Agent/Builder/Config}, ReferenceSpecContext{Agent/Builder/Config}; removes ReferenceContextAgent exports; (8) crates/cclab-agent-pyo3/src/agents.rs (modified): renames PyReferenceContextAgent → PyReferenceSpecContextAgent with updated repr, adds with_registry() method to PyBuilder accepting &PyToolRegistry for injecting custom tool registries into CodingAgent; (9) crates/cclab-agent-pyo3/src/lib.rs (modified): updates pub use and module registration for renamed PyReferenceSpecContextAgent.

## Diff

```diff
diff --git a/crates/cclab-agent/src/agents/reference_context.rs b/crates/cclab-agent/src/agents/reference_context.rs
deleted file mode 100644
index e3545483..00000000
--- a/crates/cclab-agent/src/agents/reference_context.rs
+++ /dev/null
@@ -1,816 +0,0 @@
-//! ReferenceContextAgent — discovers and synthesizes spec context for SDD.
-// ... (816 lines deleted — content moved to reference_spec_context.rs with renamed structs)

diff --git a/crates/cclab-agent/src/agents/reference_spec_context.rs b/crates/cclab-agent/src/agents/reference_spec_context.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/agents/reference_spec_context.rs
@@ -0,0 +1,816 @@
+//! ReferenceSpecContextAgent — discovers and synthesizes spec context for SDD.
+// Renamed from ReferenceContextAgent. Behavior unchanged:
+// SpecStore search → read full specs → complete_structured → CRR cycle (max_revisions=1)
+// Structs: ReferenceSpecContextAgent, ReferenceSpecContextAgentConfig, ReferenceSpecContextAgentBuilder
+// Output: ReferenceContextOutput { specs: Vec<SpecReferenceEntry>, contradictions: Vec<Contradiction> }
+// JSON schema with additionalProperties: false for strict validation

diff --git a/crates/cclab-agent/src/agents/reference_codebase_context.rs b/crates/cclab-agent/src/agents/reference_codebase_context.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/agents/reference_codebase_context.rs
@@ -0,0 +1,599 @@
+//! ReferenceCodebaseContextAgent — explores the codebase and extracts structured context.
+// Output types:
+//   pub struct KeyFile { path, purpose, key_exports: Vec<String> }
+//   pub struct CodebaseDependency { name, dependency_type: "internal"|"external", purpose }
+//   pub struct ComponentRelationship { from, to, relationship_type }
+//   pub struct ReferenceCodebaseArtifact { target, key_files, architectural_patterns,
+//                                          dependencies, relationships, summary }
+// Config: model="claude-sonnet-4-20250514", temperature=0.0, max_turns=30, max_retries=2
+// run() retry loop: build_inner_agent (CodingAgent with EXPLORATION_SYSTEM_PROMPT)
+//   → extract_and_validate (extract_json_object → serde deserialize → pretty-print)
+//   → on failure: corrective prompt up to max_retries times
+// extract_json_object: 1) ```json fence, 2) ``` fence starting with '{', 3) brace matching
+// Builder: with_provider/with_provider_arc/with_registry/with_registry_arc/with_model/
+//          with_temperature/with_max_turns/with_max_retries/build()
+// Tests: 12 tests covering round-trip, fence extraction, retry, builder validation

diff --git a/crates/cclab-agent/src/agents/codebase_to_spec.rs b/crates/cclab-agent/src/agents/codebase_to_spec.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/agents/codebase_to_spec.rs
@@ -0,0 +1,746 @@
+//! CodebaseToSpecAgent — generates SDD specifications from codebase context artifacts.
+// Input: CodebaseToSpecInput { codebase_context: ReferenceCodebaseArtifact,
+//                               target_spec_path: Option<String>,
+//                               additional_context: Option<String> }
+// Config: model="claude-sonnet-4-20250514", max_tokens=8192, temperature=0.3, max_retries=2
+// Dual role:
+//   Creator: run(json_input) → serde_json::from_str::<CodebaseToSpecInput> → generate_spec()
+//   Reviser: run(plain_text) → complete_text([system, user]) (CRR revision path)
+// generate_spec(): build_generate_prompt (system + structured user msg) → complete_text()
+// revise_spec(spec, issues): build_revise_prompt → complete_text([system, user])
+// complete_text(): CompletionRequest retry loop, retry on empty response
+// SYSTEM_PROMPT: 7 mandatory sections, format priority table, diagram selection rules,
+//                quality gates (<10% prose, no real code, pseudocode blocks, FSM rules)
+// Builder: with_provider/with_provider_arc/with_model/with_max_tokens/
+//          with_temperature/with_max_retries/build()
+// Tests: 13 tests including SequenceProvider for retry, dual-role dispatch, builder errors

diff --git a/crates/cclab-agent/src/agents/mod.rs b/crates/cclab-agent/src/agents/mod.rs
index 30625130..e82b1bf8 100644
--- a/crates/cclab-agent/src/agents/mod.rs
+++ b/crates/cclab-agent/src/agents/mod.rs
@@ -4,22 +4,38 @@ mod analyst;
 pub mod change_spec;
 pub mod code_agent;
 mod coding;
+pub mod codebase_to_spec;
 pub mod crr;
-pub mod reference_context;
+pub mod reference_codebase_context;
+pub mod reference_spec_context;
 pub mod review;
 mod restructure;
 
+// Keep the old module path accessible so existing code compiles without changes.
+// `reference_context` is now an alias for `reference_spec_context`.
+pub use reference_spec_context as reference_context;
+
 pub use analyst::{AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig};
 pub use change_spec::{ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput};
 pub use code_agent::{
     CodeAgent, CodeAgentBuilder, CodeAgentConfig, FileBlock, ImplementationTask, TaskAction,
     TaskCategory,
 };
+pub use codebase_to_spec::{
+    CodebaseToSpecAgent, CodebaseToSpecAgentBuilder, CodebaseToSpecAgentConfig,
+    CodebaseToSpecInput,
+};
 pub use coding::{CodingAgent, CodingAgentBuilder, CodingAgentConfig};
 pub use crr::{CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType};
-pub use reference_context::{
-    Contradiction, ReferenceContextAgent, ReferenceContextAgentBuilder,
-    ReferenceContextAgentConfig, ReferenceContextOutput, RelevanceLevel, SpecReferenceEntry,
+pub use reference_codebase_context::{
+    CodebaseDependency, ComponentRelationship, KeyFile, ReferenceCodebaseArtifact,
+    ReferenceCodebaseContextAgent, ReferenceCodebaseContextAgentBuilder,
+    ReferenceCodebaseContextAgentConfig,
+};
+pub use reference_spec_context::{
+    Contradiction, ReferenceContextOutput, ReferenceSpecContextAgent,
+    ReferenceSpecContextAgentBuilder, ReferenceSpecContextAgentConfig, RelevanceLevel,
+    SpecReferenceEntry,
 };

diff --git a/crates/cclab-agent/src/agents/change_spec.rs b/crates/cclab-agent/src/agents/change_spec.rs
index db69a913..e471374f 100644
--- a/crates/cclab-agent/src/agents/change_spec.rs
+++ b/crates/cclab-agent/src/agents/change_spec.rs
@@ -38,7 +38,7 @@
-use crate::agents::reference_context::ReferenceContextOutput;
+use crate::agents::reference_spec_context::ReferenceContextOutput;
 ...
 @@ -433,7 +433,7 @@
-    use crate::agents::reference_context::{
+    use crate::agents::reference_spec_context::{
         Contradiction, RelevanceLevel, ReferenceContextOutput, SpecReferenceEntry,
     };

diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
index a99448f4..29e9c075 100644
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -84,11 +84,15 @@ pub use agents::{
     ChangeSpecAgent, ChangeSpecAgentBuilder, ChangeSpecAgentConfig, ChangeSpecInput,
     CodeAgent, CodeAgentBuilder, CodeAgentConfig, FileBlock, ImplementationTask, TaskAction,
     TaskCategory,
+    CodebaseDependency, CodebaseToSpecAgent, CodebaseToSpecAgentBuilder,
+    CodebaseToSpecAgentConfig, CodebaseToSpecInput, ComponentRelationship,
     CodingAgent, CodingAgentBuilder, CodingAgentConfig,
     CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType,
     Clarification, Contradiction, Question,
-    ReferenceContextAgent, ReferenceContextAgentBuilder, ReferenceContextAgentConfig,
+    KeyFile, ReferenceCodebaseArtifact, ReferenceCodebaseContextAgent,
+    ReferenceCodebaseContextAgentBuilder, ReferenceCodebaseContextAgentConfig,
     ReferenceContextOutput, RelevanceLevel, SpecReferenceEntry,
+    ReferenceSpecContextAgent, ReferenceSpecContextAgentBuilder, ReferenceSpecContextAgentConfig,
     RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
     RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
     ReviewAgent, ReviewAgentBuilder, ReviewAgentConfig, ReviewIssue, ReviewType, ReviewVerdict,

diff --git a/crates/cclab-agent-pyo3/src/agents.rs b/crates/cclab-agent-pyo3/src/agents.rs
index f9c53af3..50d0059c 100644
--- a/crates/cclab-agent-pyo3/src/agents.rs
+++ b/crates/cclab-agent-pyo3/src/agents.rs
@@ -7,12 +7,15 @@
 use crate::llm::{PyClaudeProvider, PyGeminiProvider, PyOpenAIProvider};
+use crate::tools::PyToolRegistry;
 use cclab_agent::agents::{
-    CRRCycle, ChangeSpecAgent, Clarification, CodeAgent, ReferenceContextAgent, RestructureAgent,
+    CRRCycle, ChangeSpecAgent, Clarification, CodeAgent, ReferenceSpecContextAgent,
+    RestructureAgent,
     ...
 };
+use cclab_agent::tools::ToolRegistry;
 @@ -192,6 +195,7 @@ pub struct PyBuilder {
     model: Option<String>,
     system_prompt: Option<String>,
     max_turns: Option<u32>,
+    registry: Option<Arc<ToolRegistry>>,
 }
 @@ -227,6 +232,11 @@
+    /// Set a custom tool registry.
+    fn with_registry(&mut self, registry: &Bound<'_, PyToolRegistry>) {
+        self.registry = Some(registry.borrow().inner());
+    }
 @@ -246,6 +256,9 @@
+        if let Some(ref registry) = self.registry {
+            builder = builder.with_registry_arc(registry.clone());
+        }
 @@ -470,16 +470,16 @@
-#[pyclass(name = "ReferenceContextAgent")]
-pub struct PyReferenceContextAgent {
-    inner: Arc<ReferenceContextAgent>,
+#[pyclass(name = "ReferenceSpecContextAgent")]
+pub struct PyReferenceSpecContextAgent {
+    inner: Arc<ReferenceSpecContextAgent>,
 }
-impl PyReferenceContextAgent {
+impl PyReferenceSpecContextAgent {
     ...
     fn __repr__(&self) -> &str {
-        "<ReferenceContextAgent>"
+        "<ReferenceSpecContextAgent>"
     }
 }

diff --git a/crates/cclab-agent-pyo3/src/lib.rs b/crates/cclab-agent-pyo3/src/lib.rs
index 7f4a810b..f4103eff 100644
--- a/crates/cclab-agent-pyo3/src/lib.rs
+++ b/crates/cclab-agent-pyo3/src/lib.rs
@@ -34,7 +34,7 @@ pub use agents::{
-    PyReferenceContextAgent, PyRestructureAgent, PyRestructureInput, PyReviewAgent,
+    PyReferenceSpecContextAgent, PyRestructureAgent, PyRestructureInput, PyReviewAgent,
 };
 @@ -64,7 +64,7 @@
-    m.add_class::<PyReferenceContextAgent>()?;
+    m.add_class::<PyReferenceSpecContextAgent>()?;
```

## Review: fillback-agents-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: fillback-agents

**Summary**: 196 tests pass, PyO3 compiles clean.


---
id: implementation
type: change_implementation
change_id: agent-pyo3
---

# Implementation

## Summary

New cclab-agent-pyo3 crate across 5 files: (1) Cargo.toml (new): standalone cdylib+rlib crate depending on cclab-agent, pyo3, pyo3-async-runtimes, tokio, async-trait, serde_json, chrono; (2) src/lib.rs (modified): renamed module from _nova to _agent, replaced PyOpenAI/PyClaude/PyGemini with PyClaudeProvider/PyOpenAIProvider/PyGeminiProvider, added mod agents with PyAgent/PyBuilder/PyRestructureAgent/PyRestructureInput/PyReviewAgent, added PyCompletionRequest registration; (3) src/llm.rs (modified): extracted shared extract_tools() helper eliminating ~100 lines of duplicated tool-parsing code per provider; renamed provider wrappers to Py{Name}Provider pattern; changed inner type from Arc<RwLock<ConcreteProvider>> to Arc<dyn LLMProvider> (trait-object approach); added PyCompletionRequest pyclass with messages/model/temperature/max_tokens fields and into_inner(); added as_provider() helper on each provider for use by agent builders; (4) src/agents.rs (new, 457 lines): extract_provider() helper downcasting Python provider objects to Arc<dyn LLMProvider>; EmptySpecStore no-op SpecStore for Python bindings; RestructureOutputWrapper and ReviewVerdictWrapper converting Rust enums to Python dicts; PyAgent wrapping Arc<dyn Agent> with async run(); PyBuilder with with_provider/with_model/with_system_prompt/with_max_turns/build() building CodingAgent; PyRestructureInput pyclass with intent/project_id/clarifications fields and From<> conversion; PyRestructureAgent wrapping RestructureAgent with async run() returning structured dict; PyReviewAgent wrapping ReviewAgent with async review() returning verdict dict; (5) src/tools.rs (unchanged): PythonTool implementing Tool trait via Python callable, PyTool/PyToolParameter/PyToolRegistry unchanged.

## Diff

```diff
diff --git a/crates/cclab-agent-pyo3/src/lib.rs b/crates/cclab-agent-pyo3/src/lib.rs
index 5ecd7e4d..a0e68d62 100644
--- a/crates/cclab-agent-pyo3/src/lib.rs
+++ b/crates/cclab-agent-pyo3/src/lib.rs
@@ -1,51 +1,67 @@
-//! PyO3 bindings for cclab-nova
+//! PyO3 bindings for cclab-agent
 //!
 //! Provides Python bindings for the LLM agent framework.
 //!
 //! # Features
-//! - LLM providers (OpenAI, Claude, Gemini)
+//! - LLM providers: ClaudeProvider, OpenAIProvider, GeminiProvider
+//! - Agents: Agent, Builder, RestructureAgent, ReviewAgent
+//! - Message and completion types
 //! - Tool infrastructure
-//! - Message types
 //!
 //! # Example
 //! ```python
-//! from cclab._nova import OpenAI, Claude, Message
+//! from cclab._agent import ClaudeProvider, Builder, Message
 //!
 //! # Create provider
-//! llm = OpenAI(api_key="sk-...")
+//! provider = ClaudeProvider(api_key="sk-ant-...")
 //!
-//! # Create messages
-//! messages = [
-//!     Message.system("You are a helpful assistant."),
-//!     Message.user("Hello!")
-//! ]
+//! # Build agent
+//! agent = Builder().with_provider(provider) or:
+//! builder = Builder()
+//! builder.with_provider(provider)
+//! builder.with_model("claude-sonnet-4-20250514")
+//! agent = builder.build()
 //!
-//! # Generate completion
-//! response = await llm.complete(messages=messages, model="gpt-4")
-//! print(response.content)
+//! # Run agent
+//! response = await agent.run("Explain PyO3 bindings")
+//! print(response)
 //! ```
 
+mod agents;
 mod llm;
 mod tools;
 mod utils;
 
-pub use llm::{PyClaude, PyCompletionResponse, PyGemini, PyMessage, PyOpenAI};
+pub use agents::{PyAgent, PyBuilder, PyRestructureAgent, PyRestructureInput, PyReviewAgent};
+pub use llm:{
+    PyClaudeProvider, PyCompletionRequest, PyCompletionResponse, PyGeminiProvider, PyMessage,
+    PyOpenAIProvider,
+};
 pub use tools::{PyTool, PyToolParameter, PyToolRegistry};
 
 use pyo3::prelude::*;
 
-/// Register nova module classes and functions on an existing PyModule.
-/// Used for PyO3 integration.
+/// Register _agent module classes and functions on an existing PyModule.
 pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
     // LLM providers
-    m.add_class::<PyOpenAI>()?;
-    m.add_class::<PyClaude>()?;
-    m.add_class::<PyGemini>()?;
+    m.add_class::<PyClaudeProvider>()?;
+    m.add_class::<PyOpenAIProvider>()?;
+    m.add_class::<PyGeminiProvider>()?;
 
-    // Message types
+    // Message and completion types
     m.add_class::<PyMessage>()?;
+    m.add_class::<PyCompletionRequest>()?;
     m.add_class::<PyCompletionResponse>()?;
 
+    // Agent builder and runner
+    m.add_class::<PyAgent>()?;
+    m.add_class::<PyBuilder>()?;
+
+    // Specialized agents
+    m.add_class::<PyRestructureAgent>()?;
+    m.add_class::<PyRestructureInput>()?;
+    m.add_class::<PyReviewAgent>()?;
+
     // Tools
     m.add_class::<PyTool>()?;
     m.add_class::<PyToolParameter>()?;
@@ -54,13 +70,10 @@ pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
     Ok(())
 }
 
-/// Register the _nova module (standalone)
+/// PyO3 module: _agent
 #[pymodule]
-pub fn _nova(m: &Bound<'_, PyModule>) -> PyResult<()> {
+pub fn _agent(m: &Bound<'_, PyModule>) -> PyResult<()> {
     register_module(m)?;
-    m.add(
-        "__doc__",
-        "High-performance LLM agent framework with Rust backend",
-    )?;
+    m.add("__doc__", "High-performance LLM agent framework with Rust backend")?;
     Ok(())
 }
diff --git a/crates/cclab-agent-pyo3/src/llm.rs b/crates/cclab-agent-pyo3/src/llm.rs
index 8a24ce24..d7028d79 100644
--- a/crates/cclab-agent-pyo3/src/llm.rs
+++ b/crates/cclab-agent-pyo3/src/llm.rs
@@ -1,6 +1,7 @@
 //! PyO3 bindings for LLM providers
 //!
-//! Provides Python wrappers for OpenAI, Claude, and Gemini providers.
+//! Provides Python wrappers for OpenAI, Claude, and Gemini providers,
+//! plus CompletionRequest and CompletionResponse types.
 
 use crate::utils::{json_to_py, py_to_json};
 use cclab_agent::llm::{
@@ -12,7 +13,6 @@ use pyo3::prelude::*;
 use pyo3::types::PyDict;
 use pyo3_async_runtimes::tokio::future_into_py;
 use std::sync::Arc;
-use tokio::sync::RwLock;
 
 // PyMessage unchanged (lines 20-130)
 
+// PyCompletionRequest (new)
+#[pyclass(name = "CompletionRequest")]
+#[derive(Clone)]
+pub struct PyCompletionRequest { inner: CompletionRequest }
+// new(messages, model, temperature?, max_tokens?), getters: model/temperature/max_tokens, into_inner()
+
 // Provider rename: PyOpenAI→PyOpenAIProvider, PyClaude→PyClaudeProvider, PyGemini→PyGeminiProvider
 // Inner type changed: Arc<RwLock<ConcreteProvider>> → Arc<dyn LLMProvider>
 // Added extract_tools() shared helper (eliminates ~300 lines of duplication)
 // Added as_provider() on each provider returning Arc<dyn LLMProvider>
diff --git a/crates/cclab-agent-pyo3/src/agents.rs b/crates/cclab-agent-pyo3/src/agents.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent-pyo3/src/agents.rs
@@ -0,0 +1,457 @@
+//! PyO3 bindings for agents
+//!
+//! PyAgent (wraps Arc<dyn Agent>), PyBuilder (CodingAgent builder),
+//! PyRestructureAgent, PyRestructureInput, PyReviewAgent
+
+// extract_provider() — downcasts Python provider to Arc<dyn LLMProvider>
+// EmptySpecStore — no-op SpecStore impl for Python context
+// RestructureOutputWrapper — converts RestructureOutput enum to Python dict
+// ReviewVerdictWrapper — converts ReviewVerdict enum to Python dict
+
+// PyAgent: run(input: str) -> Awaitable[str]
+// PyBuilder: with_provider/with_model/with_system_prompt/with_max_turns/build() -> PyAgent
+// PyRestructureInput: intent, project_id, clarifications [(str,str)]
+// PyRestructureAgent: new(provider, model?) + run(input) -> Awaitable[dict]
+// PyReviewAgent: new(provider, review_type="spec"|"code", model?) + review(artifact) -> Awaitable[dict]
```

## Review: agent-pyo3-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: agent-pyo3

**Summary**: 1514 lines, compiles clean, 173 agent tests pass. PyO3 bindings for all layers.


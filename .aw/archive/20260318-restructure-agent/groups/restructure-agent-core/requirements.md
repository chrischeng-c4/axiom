---
change: restructure-agent
group: restructure-agent-core
date: 2026-03-18
---

# Requirements

Implement `RestructureAgent` in `crates/cclab-agent` alongside existing `AnalystAgent` and `CodingAgent`. The agent takes a typed `RestructureInput { intent, project_id, clarifications }`, assembles a prompt enriched with SpecStore context, calls the LLM via existing `complete_structured()` (in `structured.rs`) with a JSON Schema-enforced output, and returns a discriminated union `RestructureOutput` of either `NeedClarification { questions: Vec<Question> }` or `CreateIssues { issues: Vec<StructuredIssue>, summary: String }`. The agent is stateless (no internal mutable state), platform-agnostic (works with Claude, OpenAI, Gemini via the existing `LLMProvider` trait), and depends on a `SpecStore` trait (defined in issue #901, not yet implemented). Key acceptance criteria: (1) typed I/O schema with serde derives; (2) prompt assembles intent + clarification history + spec excerpts; (3) structured output validated against JSON Schema; (4) unit tests with mock LLM covering both output variants; (5) integration test with real LLM.

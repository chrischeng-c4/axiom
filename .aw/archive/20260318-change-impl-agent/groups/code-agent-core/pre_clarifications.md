---
change: change-impl-agent
group: code-agent-core
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Platform Integration
- **Answer**: Extend the existing PlatformIntegration trait in cclab-agent/src/integrations/mod.rs. This ensures CodeAgent can remain platform-agnostic while utilizing a unified interface for all remote project operations (fetching issues, writing files, creating branches, and opening PRs/MRs).

### Q2: Task Decomposition
- **Answer**: CodeAgent should have its own specialized decomposition logic tailored for transforming specifications into actionable implementation tasks. While it should reference the existing TaskGraph and TaskBlock models in cclab-sdd/src/models/task_graph.rs for structure, the actual transformation from Spec -> Tasks is a core responsibility of the CodeAgent.

### Q3: Task Management
- **Answer**: The agent should be responsible for managing dependencies and performing topological sorting internally. It can leverage the sorting logic pattern found in cclab-sdd/src/models/task_graph.rs (get_execution_order) to ensure tasks are implemented in the correct order (e.g., Data -> Logic -> Integration -> Testing).

### Q4: Code Generation Format
- **Answer**: For multi-file code generation, the LLM should output a single response containing multiple file blocks, each wrapped in a specific XML tag (e.g., <file path="path/to/file.rs">...</file>). This format is more reliable for parsing than raw markdown and avoids the overhead of multiple LLM calls for a single logical implementation unit.


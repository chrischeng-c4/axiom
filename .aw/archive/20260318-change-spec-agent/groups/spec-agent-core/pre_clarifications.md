---
change: change-spec-agent
group: spec-agent-core
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Naming
- **Answer**: `SpecAgent`. It aligns with the existing naming convention for specialized agents in `cclab-agent` (e.g., `RestructureAgent`, `CodingAgent`, `AnalystAgent`).

### Q2: SpecStore Integration
- **Answer**: The existing `SpecStore` trait in `cclab-agent` (with `search` and `read`) is sufficient for retrieving the necessary context. No further extension is needed at this stage.

### Q3: Input Structure
- **Answer**: The input should be a `SpecInput` struct containing the `StructuredIssue` objects (produced by `RestructureAgent`) and the `ReferenceContext` (produced by `ReferenceContextAgent`).

### Q4: Output Artifacts
- **Answer**: The agent should produce a structured `SpecOutput` object containing one or more `Spec` objects. Each `Spec` object will represent a formal specification with its respective sections and diagrams. This structured data will later be serialized to Markdown by a downstream service or the CLI.


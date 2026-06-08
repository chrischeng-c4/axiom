---
change: fillback-agents
group: fillback-agents
date: 2026-03-19
---

# Requirements

Implement two new agents, ReferenceCodebaseContextAgent and CodebaseToSpecAgent, to support the fillback flow by reverse-engineering specs from existing code. ReferenceCodebaseContextAgent will explore the codebase to build context, while CodebaseToSpecAgent will use that context to produce formal specifications. Additionally, rename the existing ReferenceContextAgent to ReferenceSpecContextAgent (and its corresponding file and Python bindings) to distinguish it from the new codebase-focused agent. Ensure all changes follow established patterns in crates/cclab-agent and crates/cclab-agent-pyo3.

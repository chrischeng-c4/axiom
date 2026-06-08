---
change: reference-context-agent
group: reference-context-agent
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: architecture
- **Answer**: The ReferenceContextAgent should be implemented within the existing cclab-agent crate, specifically as crates/cclab-agent/src/agents/reference.rs. It should leverage the existing Agent trait and CRRCycle pattern.

### Q2: relevance-scoring
- **Answer**: Relevance should be scored based on the following criteria: High (specs directly modified or providing core interfaces/models for the change), Medium (specs for dependent systems or relevant architectural patterns), Low (indirectly related specs providing general project context/conventions).

### Q3: user-interaction
- **Answer**: When contradictions are identified, the agent should produce a 'NeedClarification' output structure similar to the RestructureAgent, allowing the SDD workflow to prompt the user for resolution.

### Q4: data-extraction
- **Answer**: The agent should dynamically identify 'key_requirements' from the specs using the LLM, focusing on elements that define interfaces, data models, and state transitions relevant to the change context.


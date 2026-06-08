---
change: impl-merge-mcp-tools
date: 2026-02-05
---

# Clarifications

## Q1: Impl Mode
- **Question**: For genesis_impl_change, should it execute implementation directly or just return instructions for mainthread to execute?
- **Answer**: It depends on config - check workflow.agents.implement to decide
- **Rationale**: Flexibility to support both mainthread execution and external LLM delegation based on user configuration

## Q2: Merge Mode
- **Question**: For genesis_merge_change, should specs be merged automatically or require user confirmation before merge?
- **Answer**: Auto merge after review passes
- **Rationale**: Streamlined workflow - if review passes, no need for additional confirmation

## Q3: Agent API
- **Question**: Should these tools use the new per-artifact agent config (AgentsConfig) or the legacy WorkflowStage API?
- **Answer**: New AgentsConfig with WorkflowArtifact
- **Rationale**: Use the new per-artifact configuration for consistency with recent refactoring


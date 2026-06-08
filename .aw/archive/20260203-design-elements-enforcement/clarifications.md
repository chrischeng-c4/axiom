---
change: design-elements-enforcement
date: 2026-01-26
---

# Clarifications

## Q1: Migration
- **Question**: How should we handle existing specs that don't have required diagrams?
- **Answer**: Validation only - only validate new specs, existing specs remain unchanged
- **Rationale**: Avoids breaking existing workflows while ensuring new specs meet requirements

## Q2: Generation
- **Question**: Should we auto-generate missing diagrams or require explicit generation?
- **Answer**: Require explicit - LLM must call diagram tools explicitly
- **Rationale**: Ensures quality by requiring intentional diagram creation rather than auto-generated stubs

## Q3: Architecture
- **Question**: Where should spec_type validation logic live?
- **Answer**: MCP tools layer - validate in create_spec tool
- **Rationale**: Single point of enforcement ensures consistency and prevents bypassing validation

## Q4: Git Workflow
- **Question**: What git workflow do you prefer for this change?
- **Answer**: New branch - genesis/design-elements-enforcement
- **Rationale**: Standard branch workflow for isolated development


---
change: second-clarification
date: 2026-01-27
---

# Clarifications

## Q1: State Machine
- **Question**: How should the workflow state machine handle second clarification?
- **Answer**: New phase: 'needs-second-clarification'
- **Rationale**: Explicit phase transition (exploring → needs-second-clarification → drafting) allows the engine to pause cleanly and wait for Claude to handle user clarification. This is consistent with how the initial clarification works.

## Q2: Storage
- **Question**: Where should second clarifications be stored?
- **Answer**: Append to clarifications.md
- **Rationale**: Single file keeps all clarifications together. Use '## Post-Exploration Clarifications' section header to distinguish phases. The append_clarifications MCP tool will handle this.

## Q3: Integration
- **Question**: How should second clarification integrate with exploration results?
- **Answer**: Use needs_clarification flag in exploration.md output
- **Rationale**: The exploration phase sets needs_clarification=true with clarification_questions array when it discovers decision points. This triggers the phase transition to 'needs-second-clarification'.


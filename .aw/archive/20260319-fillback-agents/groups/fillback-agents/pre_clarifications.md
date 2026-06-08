---
change: fillback-agents
group: fillback-agents
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Code organization
- **Answer**: Flat files in agents/ directory. reference_codebase_context.rs and codebase_to_spec.rs.

### Q2: General
- **Question**: Traits
- **Answer**: Implement Agent trait. Use LLMProvider + complete_structured(). CodebaseToSpecAgent uses CRRCycle with ReviewAgent internally.

### Q3: General
- **Question**: External consumers
- **Answer**: Conductor is the only consumer, on feature branch. Safe to rename.


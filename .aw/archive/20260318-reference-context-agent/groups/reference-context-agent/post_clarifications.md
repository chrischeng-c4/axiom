---
change: reference-context-agent
group: reference-context-agent
date: 2026-03-18
status: clarified
---

# Post-Clarifications

## Questions

### Q1: spec-reading-logic
- **Question**: How should the agent handle reading full spec content if SpecStore only supports search()?
- **Answer**: The ReferenceContextAgent will follow the stateless RestructureAgent pattern, but it will require a way to read full spec content to build the reference context. Extending SpecStore with a read() method is the preferred approach.
- **Rationale**: Consistency with the RestructureAgent's dependency on SpecStore while meeting the new need for full spec summaries.

## Contradictions

### C1: restructure-agent.md vs requirement
- **Spec**: restructure-agent.md
- **Requirement**: 1. Interface with SpecStore (search/read) to identify related specifications.
- **Conflict**: The requirements specify that SpecStore should support both search() and read(), but the current implementation in restructure.rs only defines search().
- **Resolution**: Extend the SpecStore trait to include a read() method to support full spec retrieval.


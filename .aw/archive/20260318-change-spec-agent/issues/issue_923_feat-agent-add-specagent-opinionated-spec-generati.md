---
number: 923
title: "feat(agent): Add SpecAgent — opinionated spec generation with CRR cycle"
state: open
labels: [enhancement, crate:agent, P1]
group: "spec-agent-core"
---

# #923 — feat(agent): Add SpecAgent — opinionated spec generation with CRR cycle

**Parent**: #920

## Summary

Opinionated agent that generates formal specs from issues/requirements, embedding cclab-sdd best practices. Includes built-in CRR (Create-Review-Revise) cycle.

## Embedded Knowledge (from cclab-sdd)

### Format Priority
1. OpenRPC JSON — MCP tool interfaces
2. JSON Schema — data models, payloads
3. Mermaid — state machines, flowcharts, sequences
4. YAML — config, CLI command trees
5. Markdown table — mappings, enum listings
6. Prose (minimal) — only for context

### Diagram Selection
| Structure | Diagram | Identify by |
|-----------|---------|-------------|
| FSM | stateDiagram-v2 | Fixed states + transitions |
| DAG | flowchart | Conditional branches |
| Actors | sequenceDiagram | Request/response in time |
| Entities | erDiagram | Cardinality constraints |
| Hierarchy | classDiagram | Inheritance, traits |

### Section Structure
- Overview, Requirements, Scenarios, Diagrams, API Spec, Changes, Test Plan

### Quality Standards
- < 10% natural language
- No real code in specs
- Near-zero interpretive space

## CRR Cycle

```
Create (SpecAgent) → Review (ReviewAgent) → verdict
    ↑                                          ↓
    └── Revise (SpecAgent) ←── NEEDS_REVISION ─┘
                                  APPROVED → done
```

Max revisions configurable (default: 2).

## API

```rust
let spec_agent = SpecAgent::builder()
    .provider(provider)
    .spec_store(spec_store)  // read existing specs for context
    .reviewer(review_agent)  // for CRR
    .max_revisions(2)
    .build()?;

let spec = spec_agent.run(issue_input).await?;
// spec.sections, spec.diagrams, spec.format
```

## Dependencies
- #900 RestructureAgent (done)
- ReviewAgent (for CRR)
- SpecStore trait (defined in RestructureAgent)

## Test Plan
- [ ] Unit: format selection per spec type
- [ ] Unit: diagram selection per structure
- [ ] Unit: CRR cycle — approved on first review
- [ ] Unit: CRR cycle — 2 revisions then approve
- [ ] Integration: end-to-end spec generation

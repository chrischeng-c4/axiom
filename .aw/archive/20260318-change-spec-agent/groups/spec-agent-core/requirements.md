---
change: change-spec-agent
group: spec-agent-core
date: 2026-03-18
---

# Requirements

Implement `SpecAgent` (or `ChangeSpecAgent`) in `cclab-agent` to generate formal specifications from requirements following `cclab-sdd` best practices.

### Key Requirements:
- **Opinionated Generation**: Embed format priority (OpenRPC > JSON Schema > Mermaid > YAML > Markdown > Prose) and diagram selection logic (FSM -> stateDiagram, DAG -> flowchart, Actors -> sequenceDiagram, etc.).
- **Section Structure**: Ensure specs include Overview, Requirements, Scenarios, Diagrams, API Spec, Changes, and Test Plan.
- **Quality Standards**: Enforce high signal-to-noise ratio (< 10% natural language, no real code, near-zero interpretive space).
- **CRR Cycle**: Integrate with `ReviewAgent` via `CRRCycle` for an automated Create-Review-Revise loop (max revisions configurable, default 2).
- **API & Initialization**: Provide a `SpecAgentBuilder` and support `.run(issue_input).await?` returning structured specs.
- **Dependencies**: Utilize `SpecStore` for existing spec context and `ReviewAgent` for quality reviews.

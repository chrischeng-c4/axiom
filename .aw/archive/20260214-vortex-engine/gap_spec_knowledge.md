---
change_id: vortex-engine
type: gap_spec_knowledge
created_at: 2026-02-14T06:37:13.155913+00:00
updated_at: 2026-02-14T06:37:13.155913+00:00
---

## Contradictions & Misalignments
- **Persistence Layer Architecture**: The "Data Mapper Pattern" mandated in `05-titan/architecture-guide.md` for complex systems is not reflected in any of the relevant specs (`analyst-agent`, `workflow-state-machine`). This creates a risk of architectural inconsistency in how Vortex engine state is persisted. (Severity: High)
- **Agent Boundary Misalignment**: There is a lack of distinction between the task-oriented `analyst-agent` interface (from `cclab-nova`) and the real-time behavioral AI requirements for the Vortex engine. The current specs do not define the boundary or coordination between these two fundamentally different agent archetypes. (Severity: Medium)
- **State Machine Performance**: The `workflow-state-machine` pattern from `cclab-meteor` is designed for high-level task workflows. Its application to real-time game state transitions lacks a performance-oriented specification as required by the `orbit/performance-tuning.md` and `cclab-core/02-architecture-principles` baseline. (Severity: Medium)

## Missing Patterns in Specs
- **Dynamic Tooling Configuration**: The "Dynamic MCP Configuration" pattern from `40-mcp/index.md` is not reflected in the specs. There is no technical design for how the Vortex engine will dynamically expose its internal tools or engine hooks via MCP. (Severity: Medium)
- **Spec-to-Code Mapping Alignment**: The relevant specs (`02-architecture-principles`, `core-safety-standards`) are currently unstructured documents. They have not yet been adapted to the 6 core spec types required by the `spec-to-code/spec-model.md` mapping architecture. (Severity: Low)

## Archetype Gaps
- **ECS Spec Archetype**: While the lack of an ECS spec is identified as a gap, there is also no established "Archetype" in the knowledge base (`spec-to-code/spec-model.md`) for high-performance Entity Component Systems, which may lead to an ad-hoc implementation that deviates from the standard pipeline. (Severity: Medium)
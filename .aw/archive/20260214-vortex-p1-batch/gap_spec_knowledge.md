---
change_id: vortex-p1-batch
type: gap_spec_knowledge
created_at: 2026-02-14T10:25:56.143744+00:00
updated_at: 2026-02-14T10:25:56.143744+00:00
---

# Gap Analysis: Spec vs Knowledge (vortex-p1-batch)

## Architectural Gaps & Misalignments

### 1. ECS Storage vs. Data Mapper Pattern (HIGH SEVERITY)
- **Spec:** `vortex-ecs-engine` utilizes "Sparse Set Storage" for core data management.
- **Knowledge:** `05-titan/architecture-guide.md` explicitly recommends the **Data Mapper Pattern** to decouple domain models from persistence.
- **Gap:** ECS is fundamentally a data-driven, flat architecture that often conflicts with the abstraction layers of a Data Mapper. Implementing ECS without reconciling it with the project's preferred Data Mapper pattern may lead to inconsistent data access layers.

### 2. GIL Management in Hybrid Core Loop (HIGH SEVERITY)
- **Spec:** `vortex-core-architecture` defines a "Hybrid Time Step Loop" integrating rendering and state.
- **Knowledge:** `orbit/bridge-internals.md` mandates a **GIL Release Strategy** for long-running Rust tasks and cross-thread sync.
- **Gap:** The core architecture specs do not explicitly address the critical pitfall of GIL contention. In a multi-language engine (Rust/Python), failure to release the GIL during the render/update loop is a known cause of deadlocks and performance degradation.

### 3. Event Bus vs. Orbit Event Loop (MEDIUM SEVERITY)
- **Spec:** Identifies a gap for a "Formal specification for internal Event Bus for ECS and non-ECS messaging."
- **Knowledge:** `orbit/performance-tuning.md` and `orbit/bridge-internals.md` describe an existing **Orbit Event Loop** and bridge architecture.
- **Gap:** There is a risk of architectural fragmentation or redundancy. The spec context seeks an Event Bus without clarifying if it should leverage or integrate with the existing Orbit infrastructure described in the knowledge base.

### 4. Missing Nova Integration Patterns (MEDIUM SEVERITY)
- **Spec:** `vortex-agent-bt` depends on "Nova Agent Sync" for agent-driven AI.
- **Knowledge:** No patterns or documents related to **Nova** or **Behavior Tree (BT)** integration are present in the scanned knowledge categories.
- **Gap:** The specification relies on a foundational system (Nova) that has no corresponding architectural guidance or safety patterns in the current knowledge context.

### 5. Render Batching vs. Performance Baselines (LOW SEVERITY)
- **Spec:** `vortex-render-wgpu` specifies "Batch Rendering."
- **Knowledge:** `orbit/performance-tuning.md` provides performance tuning for event loops.
- **Gap:** While rendering is performance-critical, the specs do not reference the performance baselines or "io_uring" backends suggested in the Orbit knowledge docs for low-latency processing.

### 6. Operational Misalignment with MCP Strategies (LOW SEVERITY)
- **Spec:** Gaps listed for "Detailed input mapping" and "interaction logic."
- **Knowledge:** `40-mcp/index.md` suggests a strategy for **stage-specific MCP tool loading**.
- **Gap:** The spec development process for new features (Input/Interaction) does not yet reflect the operational knowledge of limiting MCP tool exposure to reduce LLM cognitive load.

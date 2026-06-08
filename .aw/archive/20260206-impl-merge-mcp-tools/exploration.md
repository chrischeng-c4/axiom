---
id: impl-merge-mcp-tools
type: exploration
created_at: 2026-02-05T15:37:34.189174+00:00
needs_clarification: false
---

# Codebase Exploration

## Architecture Overview

The genesis crate uses a **state-aware, phase-driven workflow** architecture:
- Each change has a `STATE.yaml` file tracking current phase
- MCP tools are **decision-making orchestrators** that return the next action
- The mainthread executes actions based on tool responses
- Services provide business logic for file generation and state updates

**Existing Workflow Tools**:
- `genesis_decide_change`: Orchestrates decide-change workflow
- `genesis_plan_change`: Orchestrates plan-change workflow (952 lines)
- **Missing**: `genesis_impl_change` and `genesis_merge_change`

## Relevant Files

### Reference Implementations
- `src/mcp/tools/plan_change.rs` - Main template (PlanAction enum, determine_action(), build_response())
- `src/mcp/tools/decide_change.rs` - Simpler pattern reference

### Existing Services
- `src/services/implementation_service.rs` - Has `create_review()` for REVIEW_IMPL.md
- `src/services/implementation_service.rs` - Has `create_merge_review()` for REVIEW_MERGE.md

### State Management
- `src/models/frontmatter.rs:621` - StatePhase enum with impl phases: `Implementing → Testing → CodeReviewing → Implemented`
- `src/models/frontmatter.rs:621` - StatePhase enum with merge phases: `Merging → Archived`
- `src/state/manager.rs` - StateManager for loading/updating STATE.yaml

### CLI Reference (workflow logic)
- `src/cli/implement.rs` (871 lines) - Full impl-change workflow
- `src/cli/merge_change.rs` (746 lines) - Full merge-change workflow

### Registration
- `src/mcp/tools/mod.rs` - Tool registry with all_tools_vec() and call_tool()

## Impact Analysis

**Files to Create**:
- `src/mcp/tools/impl_change.rs` (~400-500 lines)
- `src/mcp/tools/merge_change.rs` (~400-500 lines)

**Files to Modify**:
- `src/mcp/tools/mod.rs` - Add module declarations, tool definitions, call_tool match arms

## Technical Considerations

### impl_change Actions
```rust
pub enum ImplAction {
    BeginImplementation,     // From Planned → start implementing
    ResumeImplementation,    // Continue after partial work
    ReviewImplementation,    // Trigger code review
    ResolveImplementation,   // Fix issues from review
    ImplementationComplete,  // APPROVED verdict
    AlreadyImplemented,      // Already at Implemented phase
    NotReady,                // Not at Planned phase yet
}
```

### merge_change Actions
```rust
pub enum MergeAction {
    BeginMerge,       // From Implemented → start merging
    ResumeMerge,      // Continue after partial merge
    ReviewMerge,      // Trigger merge quality review
    FixMerge,         // Fix issues from review
    MergeComplete,    // APPROVED verdict → archive
    AlreadyArchived,  // Already archived
    NotReady,         // Not at Implemented phase yet
}
```

### Key Patterns to Follow
1. Analyze STATE.yaml phase to determine current state
2. Check REVIEW_*.md files to extract verdict
3. Return JSON with action type + detailed metadata
4. Include validation (change_id format, security)
5. Use new AgentsConfig API (WorkflowArtifact)

## Spec Recommendations

| spec_id | spec_type | Description |
|---------|-----------|-------------|
| impl-change-tool | mcp-tool | genesis_impl_change MCP workflow tool |
| merge-change-tool | mcp-tool | genesis_merge_change MCP workflow tool |

Dependencies: `merge-change-tool` depends on `impl-change-tool` (shared patterns)

## Risk Assessment

**Low Risk**:
- Well-established patterns from plan_change.rs and decide_change.rs
- Services and state management already exist
- Clear StatePhase definitions for impl/merge phases

**Medium Risk**:
- Integration with existing CLI implementations may need coordination
- Need to ensure consistent behavior between MCP tool and CLI

## Open Questions

None - clarifications already captured:
1. Impl mode: Depends on config (workflow.agents.implement)
2. Merge mode: Auto merge after review passes
3. Agent API: Use new AgentsConfig with WorkflowArtifact

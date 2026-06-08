# Tasks: genesis-aurora-fixes

Generated from spec: bug-fixes

## Task List

### T1: Fix Aurora Mermaid+ frontmatter format (R1)
**Priority**: high
**Files**:
- `crates/cclab-aurora/src/diagrams/state_plus/generator.rs`
- `crates/cclab-aurora/src/diagrams/flowchart_plus/generator.rs`
- `crates/cclab-aurora/src/diagrams/sequence_plus/generator.rs`
- `crates/cclab-aurora/src/diagrams/class_plus/generator.rs`
- `crates/cclab-aurora/src/diagrams/erd_plus/generator.rs`
- `crates/cclab-aurora/src/diagrams/requirement_plus/generator.rs`
- `crates/cclab-aurora/src/diagrams/mindmap_plus/generator.rs`
- `crates/cclab-aurora/src/diagrams/journey_plus/generator.rs`

**Description**:
Move YAML frontmatter inside the mermaid code block. Change from:
```
---
frontmatter
---

```mermaid
diagram
```
```

To:
```
```mermaid
---
frontmatter
---
diagram
```
```

### T2: Remove Bash permission from impl-change (R2)
**Priority**: high
**Files**:
- `crates/cclab-genesis/src/orchestrator/claude.rs`

**Description**:
Remove `Bash` from the allowed tools list at line ~103:
```rust
// Before
LlmArg::AllowedTools("Write,Edit,Read,Bash,Glob,Grep".to_string()),

// After
LlmArg::AllowedTools("Write,Edit,Read,Glob,Grep".to_string()),
```

### T3: Remove XState references from specs (R3)
**Priority**: medium
**Files**:
- `cclab/specs/cclab-genesis/plan-change.md`
- `cclab/specs/cclab-aurora/mermaid-plus-format.md`
- `cclab/specs/cclab-aurora/mermaid-plus-conversion.md`

**Description**:
- Remove all XState integration references
- Keep Mermaid+ format documentation
- Update R1-R5 requirements in mermaid-plus-format.md to reflect only implemented features

### T4: Handle no-specs case in plan-change workflow (R4)
**Priority**: high
**Files**:
- `crates/cclab-genesis/src/orchestrator/plan_change.rs` (or relevant orchestrator file)

**Description**:
When `specs/` directory is empty or no specs are found:
1. Skip the "generate tasks from specs" step
2. Generate a basic tasks.md from proposal's `impact.affected_code` list
3. Each affected file becomes a task item
4. Continue workflow without erroring

## Acceptance Criteria

- [ ] All Mermaid+ generators output frontmatter inside code block
- [ ] `cargo test` passes for aurora crate
- [ ] impl-change allowed tools list excludes Bash
- [ ] Specs contain no XState references
- [ ] `cclab gen plan-change` succeeds for changes without specs

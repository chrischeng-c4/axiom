---
id: projects-sdd-src-workflow-post-clarifications-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow state-machine interfaces drive TD/CB lifecycle transitions, review loops, merge, and validation gates."
---

# Standardized projects/agentic-workflow/src/workflow/post_clarifications.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/workflow/post_clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `handle_route` | projects/agentic-workflow/src/workflow/post_clarifications.rs | function | pub | 21 | handle_route(change_dir: &Path, change_id: &str, interface: SddInterface) -> Result<Value> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/workflow/post_clarifications.rs -->
```rust
//! Post-clarifications flow: PostClarificationsCreated routing.
//!
//! With groups removed, this simply advances to the spec phase.
//! Kept as a module for compatibility with the workflow router.

use crate::models::change::SddInterface;
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Route PostClarificationsCreated phase.
///
/// With groups removed, post-clarifications is a single artifact at the
/// change root. If the file exists, advance to spec; otherwise create it.
#[allow(dead_code)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/post_clarifications.md#source
pub fn handle_route(change_dir: &Path, change_id: &str, interface: SddInterface) -> Result<Value> {
    let sm = StateManager::load(change_dir)?;
    let post_clar_path = change_dir.join("post_clarifications.md");
    let done = post_clar_path.exists();

    if done {
        let phase_str = workflow_common::phase_to_string(sm.phase());
        let na = super::helpers::next_action(
            interface,
            "sdd_workflow_create_change_spec",
            json!({"change_id": change_id}),
        );
        Ok(json!({
            "change_id": change_id,
            "current_phase": phase_str,
            "action": "delegate_to_per_action_tools",
            "message": "Post-clarifications done. Spec lifecycle managed by per-action tools.",
            "executor": ["mainthread"],
            "next_actions": [na]
        }))
    } else {
        let phase_str = workflow_common::phase_to_string(sm.phase());
        let mut na = super::helpers::next_action(
            interface,
            "sdd_workflow_create_post_clarifications",
            json!({"change_id": change_id}),
        );
        na["when"] = json!("immediate");
        na["executor"] = json!("mainthread");
        Ok(json!({
            "change_id": change_id,
            "current_phase": phase_str,
            "action": "create_post_clarifications",
            "executor": ["mainthread"],
            "message": "Post-clarifications incomplete. Create at change root.",
            "next_actions": [na],
        }))
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/workflow/post_clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete post-clarifications workflow router.
```

---
id: sdd-tools-create-change-spec-workflow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change spec workflow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 55 | artifact_definition() -> ToolDefinition |
| `build_fill_prompt` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 736 | build_fill_prompt(     change_id: &str,     spec_id: &str,     section: &str,     group_id: Option<&str>,     project_root: &Path, ) -> Result<String> |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 338 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 120 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_spec.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute sdd_workflow_create_change_spec.
///
/// Sub-state router that determines the next action for spec creation:
/// 1. Resolve which spec needs work (topological ordering)
/// 2. Within a spec: skeleton → analyze → fill sections → prune → done
/// 3. After all specs: redirect to review or advance to implementation
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    match common::resolve_next_spec(&change_dir, &change_id)? {
        SpecSubState::Create { spec_id, depends } => {
            // Resolve group_id per-spec: prefer existing spec location, then spec_plan assignment
            let group_id = common::resolve_group_id_for_spec(&change_dir, &spec_id)
                .or_else(|| workflow_common::resolve_single_group_id(&change_dir));
            handle_create_sub_state(
                &change_id,
                &spec_id,
                &depends,
                &change_dir,
                project_root,
                interface,
                group_id.as_deref(),
            )
            .await
        }
        SpecSubState::Review { spec_id } => {
            let result = json!({
                "status": "ok",
                "spec_id": spec_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        SpecSubState::Revise { spec_id } => {
            let result = json!({
                "status": "ok",
                "spec_id": spec_id,
                "next_actions": [
                    workflow_common::next_action(interface, "sdd_workflow_revise_change_spec", json!({"change_id": change_id}))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
        SpecSubState::MainthreadMustFix { spec_id } => {
            let sm = crate::state::StateManager::load(&change_dir)?;
            let phase = sm.phase().clone();
            let resp = helpers::mainthread_must_fix(
                &change_id,
                &phase,
                &format!("spec:{}", spec_id),
                &format!("review_spec_{}", spec_id),
                interface,
            );
            Ok(serde_json::to_string_pretty(&resp)?)
        }
        SpecSubState::AdvanceToImplementation => {
            workflow_common::update_phase(&change_dir, StatePhase::ChangeImplementationCreated)?;
            let result = json!({
                "status": "phase_complete",
                "prompt": "All specs created and reviewed. Advancing to implementation.",
                "next_actions": [
                    workflow_common::next_action(interface, helpers::RUN_CHANGE_TOOL, json!({
                        "change_id": change_id,
                    }))
                ]
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Handle the create sub-state machine for a single spec.
///
/// Progression: no file → skeleton → analyze → fill per section → prune → done
///
/// Each call returns ONE prompt for the host agent (Claude Code mainthread
/// or subagent) to execute. The host agent writes artifacts via the
/// `sdd_artifact_create_change_spec` tool, then calls back to this workflow
/// tool for the next prompt. State is persisted in spec frontmatter
/// (`fill_sections`, `filled_sections`, `create_complete`).
async fn handle_create_sub_state(
    change_id: &str,
    spec_id: &str,
    depends: &[String],
    change_dir: &Path,
    project_root: &Path,
    interface: SddInterface,
    group_id: Option<&str>,
) -> Result<String> {
    let specs_dir = common::get_specs_dir(change_dir, group_id);
    let spec_path = specs_dir.join(format!("{}.md", spec_id));

    if !spec_path.exists() {
        // Step 1: Generate skeleton + return analyze prompt
        std::fs::create_dir_all(&specs_dir)?;

        let title = spec_id
            .split('-')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let skeleton = common::generate_skeleton(spec_id, &title, None, None, project_root);
        std::fs::write(&spec_path, &skeleton)?;

        // Update phase to ChangeSpecCreated
        workflow_common::update_phase(change_dir, StatePhase::ChangeSpecCreated)?;

        return build_analyze_prompt(change_id, spec_id, depends, group_id, project_root).await;
    }

    // File exists — check sub-state from frontmatter
    let content = std::fs::read_to_string(&spec_path)?;

    if common::is_create_complete(&content) {
        // Create done — redirect to review
        let result = json!({
            "status": "ok",
            "spec_id": spec_id,
            "next_actions": [
                workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))
            ]
        });
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    let fill_sections = common::read_fill_sections(&content);
    let filled_sections = common::read_filled_sections(&content);

    if fill_sections.is_empty() {
        // Analyze not done yet — return analyze prompt
        return build_analyze_prompt(change_id, spec_id, depends, group_id, project_root).await;
    }

    // Find next unfilled section (compare using base name, stripping "(optional)")
    let next_section = fill_sections.iter().find(|s| {
        let base_name = common::fill_section_base_name(s);
        !filled_sections.contains(&base_name.to_string())
    });

    if let Some(section_str) = next_section {
        // Extract base section name for the fill prompt (strip optional marker)
        let base_name = common::fill_section_base_name(section_str);
        build_fill_prompt(change_id, spec_id, base_name, group_id, project_root).await
    } else {
        // All sections filled — prune and mark complete
        let pruned = common::prune_todo_sections(&content);
        let marked = review_helpers::upsert_frontmatter_field(&pruned, "create_complete", "true");
        std::fs::write(&spec_path, &marked)?;

        // Redirect to review
        let result = json!({
            "status": "ok",
            "spec_id": spec_id,
            "message": "Spec creation complete. All sections filled and pruned.",
            "next_actions": [
                workflow_common::next_action(interface, "sdd_workflow_review_change_spec", json!({"change_id": change_id}))
            ]
        });
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_workflow"
      - "handle_create_sub_state"
    description: "Workflow orchestration for create-change-spec lifecycle routing."
```

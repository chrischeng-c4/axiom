---
id: sdd-tools-create-change-spec-artifact
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change spec artifact

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
/// Execute sdd_artifact_create_change_spec.
///
/// Writes one section at a time into the spec file. Used for both initial
/// creation and revision. After writing, marks the section as filled in
/// frontmatter and returns next_actions pointing back to the workflow tool.
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let section = get_required_string(args, "section")?;
    let content = get_required_string(args, "content")?;

    // change_id: prefer explicit arg, fall back to active change on current branch
    let explicit_change_id = get_optional_string(args, "change_id");
    // spec_id: prefer explicit arg, fall back to STATE.yaml current_task_id
    let explicit_spec_id = get_optional_string(args, "spec_id");

    let change_id = if let Some(id) = explicit_change_id {
        workflow_common::validate_change_id(&id)?;
        id
    } else {
        workflow_common::resolve_active_change_id(project_root)?
    };

    let interface = workflow_common::load_interface(project_root);

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    let spec_id = if let Some(id) = explicit_spec_id {
        id
    } else {
        // Read current_task_id from STATE.yaml
        let sm = crate::state::StateManager::load(&change_dir)?;
        sm.state().current_task_id.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "No spec_id in payload and no current_task_id in STATE.yaml. \
                 Either pass spec_id or run the workflow tool first."
            )
        })?
    };

    // Validate section name
    if !common::ALL_SECTIONS.contains(&section.as_str()) {
        anyhow::bail!(
            "Invalid section '{}'. Valid: {:?}",
            section,
            common::ALL_SECTIONS
        );
    }

    // Auto-resolve spec path from change directory structure.
    // No group_id or spec_id needed from caller — state machine knows.
    let spec_path = resolve_spec_path(&change_dir, &spec_id)?;

    // Read current content
    let current = std::fs::read_to_string(&spec_path)?;

    // Replace the target section
    let updated = common::replace_section(&current, &section, &content);

    // Mark section as filled in frontmatter
    let mut filled = common::read_filled_sections(&updated);
    if !filled.contains(&section) {
        filled.push(section.clone());
    }
    let filled_str = format!("[{}]", filled.join(", "));
    let mut final_content =
        review_helpers::upsert_frontmatter_field(&updated, "filled_sections", &filled_str);

    // Persist fill_sections if provided (from analyze step)
    if let Some(fs) = args.get("fill_sections").and_then(|v| v.as_array()) {
        let fs_list: Vec<String> = fs
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        // Reject umbrella names — only ALL_SECTIONS leaf names allowed.
        // `fill_section_base_name` strips "(optional)" markers before comparison.
        let invalid: Vec<String> = fs_list
            .iter()
            .filter(|s| {
                let base = common::fill_section_base_name(s);
                !common::ALL_SECTIONS.contains(&base)
            })
            .cloned()
            .collect();
        if !invalid.is_empty() {
            anyhow::bail!(
                "Invalid fill_sections entries: {:?}. Use leaf names only (no umbrella names like 'diagrams'/'api_spec'/'test_plan'). Valid: {:?}",
                invalid,
                common::ALL_SECTIONS
            );
        }
        if !fs_list.is_empty() {
            let fs_str = format!("[{}]", fs_list.join(", "));
            final_content =
                review_helpers::upsert_frontmatter_field(&final_content, "fill_sections", &fs_str);
        }
    }

    // Persist main_spec_ref if provided
    if let Some(ref_path) = get_optional_string(args, "main_spec_ref") {
        final_content = review_helpers::upsert_frontmatter_field(
            &final_content,
            "main_spec_ref",
            &format!("\"{}\"", ref_path),
        );
    }

    std::fs::write(&spec_path, &final_content)?;

    // ── Post-write alignment validation ──────────────────────────────────
    // Gating: only run on complete specs (create_complete: true).
    // Rationale: incomplete specs contain unfilled <!-- TODO --> sections
    // that trigger MissingSectionAnnotation / FormatPriorityViolation false
    // positives. The spec says "after writing section content to disk, call
    // spec_alignment::check()" — the create_complete gate narrows this to
    // the final write (after prune), which is the first moment the spec
    // is structurally valid. See also: test_artifact_alignment_skipped_when_incomplete.
    //
    // Phase 1 format violations → revert and return error.
    // Phase 2 coverage gaps → allow write, return as warnings.
    let alignment_warnings: Option<Vec<Value>> = if common::is_create_complete(&final_content) {
        match crate::spec_alignment::check(&spec_path) {
            result if result.total_violations > 0 => {
                let mut format_violations = Vec::new();
                let mut coverage_warnings = Vec::new();

                for file_result in &result.files {
                    for violation in &file_result.violations {
                        let v_json = json!({
                            "kind": violation.kind.to_string(),
                            "message": &violation.message,
                            "heading": violation.heading.as_deref(),
                            "line": violation.line,
                            "file": &file_result.path,
                        });
                        if violation.kind.is_format_violation() {
                            format_violations.push(v_json);
                        } else {
                            coverage_warnings.push(v_json);
                        }
                    }
                }

                if !format_violations.is_empty() {
                    // Revert to pre-write content
                    std::fs::write(&spec_path, &current)?;
                    let err_result = json!({
                        "status": "error",
                        "message": "Alignment check failed: format violations found. File reverted.",
                        "violations": format_violations,
                        "next_actions": [
                            workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
                        ]
                    });
                    return Ok(serde_json::to_string_pretty(&err_result)?);
                }

                if coverage_warnings.is_empty() {
                    None
                } else {
                    Some(coverage_warnings)
                }
            }
            _ => None,
        }
    } else {
        None
    };

    // Derive relative path from the resolved spec_path
    let rel_spec_path = spec_path
        .strip_prefix(&change_dir)
        .unwrap_or(&spec_path)
        .to_string_lossy()
        .to_string();
    let artifacts_written = vec![rel_spec_path];

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "alignment_warnings": alignment_warnings,
        "next_actions": [
            workflow_common::next_action(interface, "sdd_workflow_create_change_spec", json!({"change_id": change_id}))
        ]
    });

    Ok(serde_json::to_string_pretty(&result)?)
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
      - "<handwrite-gap:missing-generator:sdd-artifact-write-flow>"
    description: "Artifact write flow for one-section change-spec updates and alignment gating."
```

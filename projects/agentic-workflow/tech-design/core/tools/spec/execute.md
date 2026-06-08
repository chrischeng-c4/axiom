---
id: sdd-tools-spec-execute
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools spec execute

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 41 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 238 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_review_spec` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 574 | execute_review_spec(args: &Value, project_root: &Path) -> Result<String> |
| `review_spec_definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 476 | review_spec_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute the create_spec tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");

    // Extract required fields
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let title = get_required_string(args, "title")?;
    let overview = get_required_string(args, "overview")?;
    let requirements = get_required_array(args, "requirements")?;
    let scenarios = get_required_array(args, "scenarios")?;

    // Optional fields
    let flow_diagram = get_optional_string(args, "flow_diagram");
    let data_model = args.get("data_model").cloned();

    // Parse spec_type (required field)
    let spec_type_str = get_required_string(args, "spec_type")?;
    let spec_type = SpecType::from_str(&spec_type_str)
        .map_err(|_| anyhow::anyhow!(
            "Invalid spec_type '{}'. Valid types: http-api, event-driven, data-model, algorithm, integration, utility, rpc-api, workflow",
            spec_type_str
        ))?;

    // Parse structured diagrams
    let diagrams_array = args.get("diagrams").and_then(|v| v.as_array()).cloned();
    let diagrams_vec: Vec<DiagramData> = diagrams_array
        .unwrap_or_default()
        .iter()
        .filter_map(|d| {
            Some(DiagramData {
                diagram_type: d.get("type")?.as_str()?.to_string(),
                title: d.get("title")?.as_str()?.to_string(),
                input: d.get("input")?.clone(),
                rendered: None,
                semantic: None,
            })
        })
        .collect();

    // Validate diagram types
    let valid_diagram_types = [
        "flowchart",
        "sequence",
        "class",
        "state",
        "erd",
        "mindmap",
        "requirement",
        "journey",
    ];
    for diagram in &diagrams_vec {
        if !valid_diagram_types.contains(&diagram.diagram_type.as_str()) {
            anyhow::bail!(
                "Invalid diagram type '{}'. Valid types: {:?}",
                diagram.diagram_type,
                valid_diagram_types
            );
        }
    }

    // Parse API spec if provided
    let api_spec = if let Some(api_spec_obj) = args.get("api_spec") {
        let spec_type_str = api_spec_obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("api_spec missing 'type' field"))?;
        let spec_type = ApiSpecType::from_str(spec_type_str)
            .map_err(|_| anyhow::anyhow!("Invalid api_spec type: {}", spec_type_str))?;
        let spec = api_spec_obj
            .get("spec")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("api_spec missing 'spec' field"))?;
        Some(ApiSpecData { spec_type, spec })
    } else {
        None
    };

    // Parse spec_group if provided
    let spec_group = get_optional_string(args, "spec_group");

    // Parse group_id for multi-group change layout
    let group_id = get_optional_string(args, "group_id");

    // Parse main_spec_ref and merge_strategy (for main spec awareness)
    let main_spec_ref = get_optional_string(args, "main_spec_ref");
    let merge_strategy = get_optional_string(args, "merge_strategy");

    // Convert requirements JSON array to RequirementData (with explicit errors)
    let requirements_vec: Vec<RequirementData> = requirements
        .iter()
        .enumerate()
        .map(|(i, r)| {
            Ok(RequirementData {
                id: r
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("requirements[{}]: missing 'id'", i))?
                    .to_string(),
                title: r
                    .get("title")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("requirements[{}]: missing 'title'", i))?
                    .to_string(),
                description: r
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("requirements[{}]: missing 'description'", i))?
                    .to_string(),
                priority: r
                    .get("priority")
                    .and_then(|p| p.as_str())
                    .unwrap_or("medium")
                    .to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Convert scenarios JSON array to ScenarioData (with explicit errors)
    let scenarios_vec: Vec<ScenarioData> = scenarios
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Ok(ScenarioData {
                name: s
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("scenarios[{}]: missing 'name'", i))?
                    .to_string(),
                given: s.get("given").and_then(|g| g.as_str()).map(String::from),
                when: s
                    .get("when")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("scenarios[{}]: missing 'when'", i))?
                    .to_string(),
                then: s
                    .get("then")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("scenarios[{}]: missing 'then'", i))?
                    .to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Save for state update after artifact creation
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);

    // Create input struct and call service
    // Note: agent and duration_secs are set by the workflow engine post-processing
    let input = CreateSpecInput {
        change_id,
        spec_id,
        title,
        overview,
        requirements: requirements_vec,
        scenarios: scenarios_vec,
        spec_type,
        diagrams: diagrams_vec,
        flow_diagram,
        data_model,
        api_spec,
        agent: None,
        duration_secs: None,
        spec_group,
        group_id,
        main_spec_ref,
        merge_strategy,
        tags: parse_string_array_opt(args, "tags"),
        changes: parse_changes(args),
        depends: parse_string_array_opt(args, "depends"),
    };

    let result = create_spec(input, project_root)?;

    // Auto-update STATE.yaml phase
    super::workflow_common::update_phase(&change_dir, StatePhase::ChangeSpecCreated)?;

    if caller == "agent" {
        Ok(result)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            result
        ))
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - execute
    description: "Create-spec execution path."
```

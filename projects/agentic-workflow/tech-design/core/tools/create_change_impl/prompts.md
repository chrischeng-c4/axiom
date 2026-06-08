---
id: projects-sdd-src-tools-create-change-impl-rs-prompts
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd create change implementation prompts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 46 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 348 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 83 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `parse_test_plan_count` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 869 | parse_test_plan_count(spec_content: &str) -> Option<usize> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_impl.rs | function | pub | 21 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ─── Prompt Builders ─────────────────────────────────────────────────────────

async fn build_implement_code_prompt(
    change_id: &str,
    spec_id: &str,
    is_first: bool,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();
    let title = if is_first {
        format!("Begin Implementation for Change '{}'", change_id)
    } else {
        format!("Implement Spec '{}' for Change '{}'", spec_id, change_id)
    };

    // Group-aware spec path
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };

    let instructions = if is_first {
        format!(
            "1. List all change specs in `.aw/changes/{cid}/`\n\
             2. Read spec **{sid}** to understand requirements: `{spec_path}`\n\
             3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **{sid}**\n\
             4. When done with {sid}, run `score workflow create-change-implementation {cid}` to advance",
            cid = change_id, sid = spec_id, spec_path = spec_path
        )
    } else {
        format!(
            "1. Read spec **{sid}**: `{spec_path}`\n\
             2. Implement **production code only** (no `#[test]` functions) according to spec requirements\n\
             3. When done, run `score workflow create-change-implementation {cid}` to advance",
            cid = change_id, sid = spec_id, spec_path = spec_path
        )
    };

    // Extract targets from spec's changes section for enriched guidance
    let spec_full_path = project_root.join(&spec_path);
    let targets_section = extract_change_targets(&spec_full_path);

    // Resolve executor before building prompt
    let action = if is_first {
        "begin_implementation"
    } else {
        "implement_spec"
    };
    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    // Code intelligence hints — all executors (mainthread + Claude Code subagents)
    // have Bash tool access, so hints are always included for change implementation.
    let cli_hints = "\n\
         \n\
         # Code intelligence — explore codebase before making changes\n\
         score symbols <file>              # list symbols in a file\n\
         score hover <file> <line> <col>   # type info for a symbol\n\
         score references <file> <line> <col>  # find all references\n\
         score impact <file> <line> <col>  # analyze change impact\n\
         score context <file:symbol...> [--depth N]  # cross-ref context";

    let prompt = format!(
        "# Task: {title}\n\n\
         ## Instructions\n\n\
         {instructions}\n\n\
         {targets}\
         ## Spec Annotations\n\n\
         Add `@spec` annotations to public functions that implement spec requirements.\n\
         For each public function or method,\n\
         add a comment: `// @spec {{spec_path}}#R{{N}}` where `{{spec_path}}` is the\n\
         spec file path and `R{{N}}` is the requirement ID from the spec's Requirements table.\n\n\
         Use the comment syntax appropriate for the language:\n\
         ```\n\
         // @spec {spec_path}#R1   (Rust, JS, TS, Go, C)\n\
         #  @spec {spec_path}#R1   (Python, Ruby, Shell, YAML)\n\
         -- @spec {spec_path}#R1   (SQL)\n\
         <!-- @spec {spec_path}#R1 --> (HTML, Markdown)\n\
         /* @spec {spec_path}#R1 */    (CSS, C block)\n\
         ```\n\n\
         This annotation enables automated spec↔code traceability.\n\
         Place the annotation on the line immediately above the function signature.\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Read spec\n\
         Read file: {spec_path}\n\
         \n\
         # Advance implementation workflow\n\
         score workflow create-change-implementation {cid}\
         {cli_hints}\n\
         ```",
        cid = change_id,
        spec_path = spec_path,
        targets = targets_section,
        cli_hints = cli_hints
    );

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        action,
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

/// Extract change targets from spec's `changes` YAML section.
/// Returns a formatted string for the implementation prompt, or empty if no targets found.
fn extract_change_targets(spec_path: &std::path::Path) -> String {
    let content = match std::fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    // Find ```yaml block after <!-- type: changes lang: yaml -->
    let changes_marker = "<!-- type: changes lang: yaml -->";
    let Some(marker_pos) = content.find(changes_marker) else {
        return String::new();
    };
    let after_marker = &content[marker_pos..];

    // Extract YAML between ```yaml and ```
    let Some(yaml_start) = after_marker.find("```yaml").map(|p| p + 7) else {
        return String::new();
    };
    let Some(yaml_end) = after_marker[yaml_start..].find("```") else {
        return String::new();
    };
    let yaml_content = &after_marker[yaml_start..yaml_start + yaml_end];

    // Parse YAML and look for targets
    let doc: serde_json::Value = match serde_yaml::from_str(yaml_content) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    let changes = match doc.get("changes").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return String::new(),
    };

    let mut sections = Vec::new();
    for entry in changes {
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let action = entry.get("action").and_then(|v| v.as_str()).unwrap_or("");

        if action != "MODIFY" {
            continue;
        }

        let targets = entry.get("targets").and_then(|v| v.as_array());
        let do_not_touch = entry.get("do_not_touch").and_then(|v| v.as_array());

        if targets.is_none() && do_not_touch.is_none() {
            continue;
        }

        let mut section = format!("### {}\n", path);

        if let Some(targets) = targets {
            for t in targets {
                let kind = t.get("type").and_then(|v| v.as_str()).unwrap_or("function");
                let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let change = t.get("change").and_then(|v| v.as_str()).unwrap_or("");
                section.push_str(&format!("- **{} `{}`**: {}\n", kind, name, change));
            }
        }

        if let Some(dnt) = do_not_touch {
            let names: Vec<&str> = dnt.iter().filter_map(|v| v.as_str()).collect();
            if !names.is_empty() {
                section.push_str(&format!("- **DO NOT MODIFY**: {}\n", names.join(", ")));
            }
        }

        sections.push(section);
    }

    if sections.is_empty() {
        return String::new();
    }

    format!("## Change Targets\n\n{}\n", sections.join("\n"))
}

async fn build_implement_tests_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };

    let prompt = format!(
        "# Task: Implement Tests for Spec '{sid}' (Change '{cid}')\n\n\
         ## Instructions\n\n\
         Production code for spec '{sid}' has been implemented and verified to compile.\n\
         Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).\n\n\
         1. Read spec **{sid}**: `{spec_path}`\n\
         2. Read the `## Unit Test` section to understand required test cases\n\
         3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the unit-test design\n\
         4. Run `cargo test` to verify tests pass\n\
         5. When done, run `score workflow create-change-implementation {cid}` to advance\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Read spec\n\
         Read file: {spec_path}\n\
         \n\
         # Run tests\n\
         cargo test\n\
         \n\
         # Advance implementation workflow\n\
         score workflow create-change-implementation {cid}\n\
         ```",
        sid = spec_id,
        cid = change_id,
        spec_path = spec_path
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    let mut extra = json!({ "spec_id": spec_id, "phase": "tests" });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("implement_tests_{}", spec_id),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

async fn build_codegen_prompt(
    change_id: &str,
    spec_id: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };

    let prompt = format!(
        "# Task: Implement Spec '{spec_id}' with Codegen for Change '{change_id}'\n\n\
         ## Structured Codegen Path\n\n\
         Spec '{spec_id}' has a JSON schema or API spec. Use Lens's code generation pipeline:\n\n\
         1. Read the spec: `{spec_path}`\n\
         2. Call `cclab lens gen-from-spec` with spec content to generate code\n\
         3. Review generated output against spec requirements\n\
         4. Apply manual adjustments as needed\n\
         5. Run tests to verify\n\
         6. When done, run `score workflow create-change-implementation {change_id}` to advance\n\n\
         ## Fallback\n\n\
         If codegen fails, fall back to manual implementation.\n\n\
         ## CLI Commands\n\n\
         ```\n\
         # Read spec\n\
         Read file: {spec_path}\n\
         \n\
         # Generate code from spec\n\
         cclab lens gen-from-spec <spec_json>\n\
         \n\
         # Advance implementation workflow\n\
         score workflow create-change-implementation {change_id}\n\
         ```",
        spec_path = spec_path
    );

    let change_dir = workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreateChangeImplementation,
    );

    let mut extra = json!({ "spec_id": spec_id, "codegen": true });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        "implement_spec_with_codegen",
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-create-change-impl-rs-prompts>"
    description: "Implementation workflow prompt builders."
```

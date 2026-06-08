---
id: sdd-tools-create-change-spec-prompts
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change spec prompts

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
/// Build ANALYZE prompt — agent reads context and decides which sections to fill.
async fn build_analyze_prompt(
    change_id: &str,
    spec_id: &str,
    depends: &[String],
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    let depends_note = if depends.is_empty() {
        String::new()
    } else {
        format!(
            "\n## Dependencies\n\nThis spec depends on: {}. Read these specs first.\n",
            depends.join(", ")
        )
    };

    // Group-aware paths
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };
    let payload_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/payloads/create-change-spec.json",
            change_id, gid
        ),
        None => format!(".aw/changes/{}/payloads/create-change-spec.json", change_id),
    };
    let group_id_hint = match group_id {
        Some(gid) => format!(
            "\n**group_id**: `{}` (pass this to the artifact CLI as `group_id` parameter)\n",
            gid
        ),
        None => String::new(),
    };

    let prompt = format!(
        r#"# Task: Analyze Spec '{spec_id}' for Change '{change_id}'

A skeleton has been generated at `{spec_path}`.
{group_id_hint}
## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED and you will have to redo the work.

## Instructions

1. Read context:
   - Read the temp issue working copy that initiated this change (see user_input.md for the slug)
   - The issue's ## Problem, ## Requirements, ## Scope, and ## Reference Context sections are your primary context
2. Read the skeleton: `{spec_path}`
3. **IMPORTANT — `main_spec_ref`**: Check the spec frontmatter. If `main_spec_ref` is `~` (null),
   you MUST determine the target path in `.aw/tech-design/` where this spec will be merged.
   Format: `<scope>/<category>/<spec-id>.md` (e.g., `sdd/tools/new-feature.md`).
   Browse `.aw/tech-design/` to see existing spec groups.
   Pass it as the `main_spec_ref` parameter when calling the artifact CLI.
4. Decide which sections to fill based on the artifact being changed. Pick ONLY leaf section names from this list — NEVER pass umbrella words like `diagrams`, `api_spec`, or `test_plan`:
   Always fill: `changes`
   Verification artifacts (pick those that apply): `unit-test`, `e2e-test`
   Diagrams (pick those that apply): `interaction`, `logic`, `state-machine`, `mindmap`, `dependency`, `db-model`
   API shape (pick those that apply): `rest-api`, `rpc-api`, `async-api`, `cli`, `schema`, `config`
   UI (pick those that apply): `wireframe`, `component`, `design-token`
   Optional migration/prose sections only when maintaining legacy TD: `overview`, `requirements`, `scenarios`
   Docs: `doc`
5. Write a JSON payload file to `{payload_path}` then run the artifact CLI.

## Expected Action

Write the **overview** section first via artifact CLI. Pass the `fill_sections`
array as a parameter — USE LEAF NAMES ONLY from the allowed list above.
Example (adapt to this change): `fill_sections=["cli", "unit-test", "e2e-test", "changes"]`.
Never pass `diagrams`, `api_spec`, or `test_plan` (umbrella names).
Also pass `main_spec_ref` as a parameter if determined above.
The system persists it to frontmatter automatically.

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Read artifacts
Read file: .aw/changes/{change_id}/proposal.md
Read file: {spec_path}

# Write each section (MUST use this — do NOT edit spec files directly)
# Step 1: Write payload JSON to the EXACT path below (do NOT write to other locations)
# Step 2: Run artifact CLI
score artifact create-change-spec {change_id} {payload_path}
```
{depends_note}"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("analyze_spec_{}", spec_id),
        prompt,
        executor,
        extra,
        interface,
        project_root,
    )
    .await
}

/// Return fill guidance text for a section.
///
/// When a `SectionType` can be parsed from `section`, returns type-aware
/// guidance including the annotation format. Falls back to the legacy
/// string-match guidance for unrecognized names.
fn section_fill_guidance(section: &str) -> &'static str {
    // Attempt to resolve via SectionType for type-aware guidance
    if let Ok(st) = SectionType::from_str(section) {
        return match st {
            SectionType::Overview =>
                "Write a comprehensive overview (>= 50 chars) describing what this spec covers.\n\
                 Begin with `<!-- type: overview lang: markdown -->` on its own line.",
            SectionType::Requirements =>
                "Write requirements as Mermaid Plus requirementDiagram.\n\
                 Begin with `<!-- type: requirements lang: mermaid -->`.",
            SectionType::Scenarios =>
                "Write acceptance scenarios as YAML entries with id, given, when, then, and optional diagram_ref.\n\
                 Begin with `<!-- type: scenarios lang: yaml -->`.",
            SectionType::Changes =>
                "List files that will change. For MODIFY entries, include function/type-level `targets`:\n\
                 ```yaml\nchanges:\n  - path: foo.rs\n    action: CREATE\n    description: new file\n\
                   - path: bar.rs\n    action: MODIFY\n    targets:\n\
                       - type: function\n        name: handle_request\n        change: add error handling\n\
                       - type: struct\n        name: Config\n        change: add timeout field\n\
                     do_not_touch: [validate_input, parse_args]\n```\n\
                 Target type values: function, struct, enum, trait, impl, method.\n\
                 `targets` is required for MODIFY, optional for CREATE/DELETE.\n\
                 `do_not_touch` lists functions/types the agent must NOT modify.\n\
                 Begin with `<!-- type: changes lang: yaml -->`.",
            SectionType::UnitTest =>
                "Define unit-test cases as Mermaid Plus requirementDiagram with test elements and verifies links.\n\
                 Begin with `<!-- type: unit-test lang: mermaid -->`.",
            SectionType::E2eTest =>
                "Define product-flow E2E cases in YAML, including command/input, expected output, and artifact side-effect assertions.\n\
                 Begin with `<!-- type: e2e-test lang: yaml -->`.",
            SectionType::Interaction =>
                "Draw a Mermaid sequence diagram. Begin with `<!-- type: interaction lang: mermaid -->`.",
            SectionType::Logic =>
                "Draw a Mermaid flowchart. Begin with `<!-- type: logic lang: mermaid -->`.",
            SectionType::Dependency =>
                "Draw a Mermaid class diagram. Begin with `<!-- type: dependency lang: mermaid -->`.",
            SectionType::StateMachine =>
                "Draw a Mermaid stateDiagram-v2. Begin with `<!-- type: state-machine lang: mermaid -->`.",
            SectionType::DbModel =>
                "Draw a Mermaid erDiagram. Begin with `<!-- type: db-model lang: mermaid -->`.",
            SectionType::Mindmap =>
                "Draw a Mermaid mindmap. Begin with `<!-- type: mindmap lang: mermaid -->`.",
            SectionType::RestApi =>
                "Write OpenAPI 3.1 YAML. Begin with `<!-- type: rest-api lang: yaml -->`.",
            SectionType::RpcApi =>
                "Write OpenRPC 1.3 YAML. Begin with `<!-- type: rpc-api lang: yaml -->`.",
            SectionType::AsyncApi =>
                "Write AsyncAPI 2.6 YAML. Begin with `<!-- type: async-api lang: yaml -->`.",
            SectionType::Cli =>
                "Define CLI command tree in YAML. Begin with `<!-- type: cli lang: yaml -->`.",
            SectionType::Schema =>
                "Write JSON Schema for interface/data models as YAML. Begin with `<!-- type: schema lang: yaml -->`.",
            SectionType::Config =>
                "Write config file schema as YAML. Begin with `<!-- type: config lang: yaml -->`.",
            SectionType::Wireframe =>
                "Describe the UI layout in wireframe YAML. Begin with `<!-- type: wireframe lang: yaml -->`.",
            SectionType::Component =>
                "Define UI component contract as YAML. Begin with `<!-- type: component lang: yaml -->`.",
            SectionType::DesignToken =>
                "Define design tokens in W3C DTCG-compatible YAML. Begin with `<!-- type: design-token lang: yaml -->`.",
            SectionType::RuntimeImage =>
                "Define a container image build contract in YAML: base image, package installs, copy layout, build args, env, entrypoint/command, and build context inputs. Begin with `<!-- type: runtime-image lang: yaml -->`.",
            SectionType::Deployment =>
                "Define deployment/runtime operations manifests in YAML: Kubernetes/Kustomize resources, overlays, services, scaling, routing, policy, and rollout expectations. Begin with `<!-- type: deployment lang: yaml -->`.",
            SectionType::Doc =>
                "Write user-facing documentation in markdown. Begin with `<!-- type: doc lang: markdown -->`.",
            SectionType::Manifest =>
                "Declare package manifest entries (Cargo.toml dependencies, etc.) in YAML.\n\
                 Shape: `dependencies: [{ name, spec: workspace|version|path, features?: [..] }]`.\n\
                 Begin with `<!-- type: manifest lang: yaml -->`.",
        };
    }

    // Fallback for sections without SectionType-specific guidance (e.g. overview, requirements).
    match section {
        "overview" => "Write a comprehensive overview (>= 50 chars) describing what this spec covers.",
        "requirements" => "Write requirements in markdown format:\n\n### R1: Title\n\nDescription.\n\n**Priority**: high/medium/low",
        "scenarios" => "Write acceptance scenarios in Given/When/Then format:\n\n### Scenario: Name\n\n**GIVEN** precondition\n**WHEN** action\n**THEN** expected outcome",
        "changes" => "List the files that will be changed:\n\n| File | Action | Description |\n|------|--------|-------------|",
        _ => "Fill in this section with appropriate content.",
    }
}

/// Build FILL prompt for a specific section.
pub(crate) async fn build_fill_prompt(
    change_id: &str,
    spec_id: &str,
    section: &str,
    group_id: Option<&str>,
    project_root: &Path,
) -> Result<String> {
    let _pp = project_root.display();

    // Base section guidance, augmented with SectionType-specific hints
    let section_guidance = section_fill_guidance(section);

    // Group-aware paths
    let spec_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/specs/{}.md",
            change_id, gid, spec_id
        ),
        None => format!(".aw/changes/{}/specs/{}.md", change_id, spec_id),
    };
    let payload_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/payloads/create-change-spec.json",
            change_id, gid
        ),
        None => format!(".aw/changes/{}/payloads/create-change-spec.json", change_id),
    };
    let group_id_hint = match group_id {
        Some(gid) => format!(
            "\n**group_id**: `{}` (pass this to the artifact CLI)\n",
            gid
        ),
        None => String::new(),
    };

    let prompt = format!(
        r#"# Task: Fill Section '{section}' for Spec '{spec_id}' (Change '{change_id}')
{group_id_hint}
## Instructions

1. Read the current spec: `{spec_path}`
2. Read relevant context if needed
3. Write content for the **{section}** section

## Section Guidance

{section_guidance}

## Action

Run `score artifact create-change-spec` with section="{section}" and your content.

## CLI Commands

```
# Read spec
Read file: {spec_path}

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec {change_id} {payload_path}
```"#,
    );

    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    let interface = workflow_common::load_interface(project_root);
    let executor =
        workflow_common::get_executor_chain(project_root, WorkflowArtifact::CreateChangeSpec);

    let mut extra = json!({ "spec_id": spec_id });
    if let Some(gid) = group_id {
        extra["group_id"] = json!(gid);
    }

    workflow_common::build_workflow_response(
        &change_dir,
        change_id,
        &format!("fill_spec_{}_{}", spec_id, section),
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
  - path: projects/agentic-workflow/src/tools/create_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "build_analyze_prompt"
      - "section_fill_guidance"
      - "build_fill_prompt"
    description: "Prompt builders and section-specific fill guidance for create-change-spec."
```

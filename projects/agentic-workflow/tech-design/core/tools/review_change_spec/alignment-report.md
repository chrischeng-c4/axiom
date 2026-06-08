---
id: sdd-tools-review-change-spec-alignment-report
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change spec alignment report

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 48 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 186 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 129 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 24 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Build alignment report section for spec review prompts.
///
/// Calls `spec_alignment::check()` on the change-spec file being reviewed.
/// Returns a markdown section to inject into the review prompt, or empty string on error.
fn build_alignment_report(spec_abs_path: &Path) -> String {
    let check_result =
        match std::panic::catch_unwind(|| crate::spec_alignment::check(spec_abs_path)) {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!(
                    path = %spec_abs_path.display(),
                    "alignment check panicked — skipping injection"
                );
                return String::new();
            }
        };

    if check_result.total_violations == 0 {
        return "## Alignment Report\n\nNo alignment violations found.\n\n".to_string();
    }

    let mut table =
        String::from("## Alignment Report\n\n| File | Kind | Message |\n|------|------|---------|");
    for file_result in &check_result.files {
        for violation in &file_result.violations {
            table.push_str(&format!(
                "\n| {} | {} | {} |",
                file_result.path, violation.kind, violation.message
            ));
        }
    }
    table.push_str("\n\n");
    table
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "build_alignment_report"
    description: "Alignment report injection helper for review prompts."
```

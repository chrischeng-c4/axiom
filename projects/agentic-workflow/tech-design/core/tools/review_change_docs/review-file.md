---
id: sdd-tools-review-change-docs-review-file
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change docs review file

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_change_docs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 44 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 150 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 97 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_change_docs.rs | function | pub | 19 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Build review file content.
fn build_review_file(verdict: &str, review_notes: &str, cli_results: &[Value]) -> String {
    let mut md = String::new();
    md.push_str("# Docs Review\n\n");
    md.push_str(&format!("verdict: {}\n\n", verdict));
    md.push_str("## Review Notes\n\n");
    md.push_str(review_notes);
    md.push_str("\n\n");

    if !cli_results.is_empty() {
        md.push_str("## CLI Verification Results\n\n");
        md.push_str("| Command | Expected | Actual | Pass |\n");
        md.push_str("|---------|----------|--------|------|\n");
        for result in cli_results {
            let cmd = result.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let expected = result
                .get("expected")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let actual = result.get("actual").and_then(|v| v.as_str()).unwrap_or("");
            let pass = result
                .get("pass")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let icon = if pass { "PASS" } else { "FAIL" };
            md.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                cmd, expected, actual, icon
            ));
        }
        md.push('\n');
    }

    md
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "build_review_file"
    description: "Markdown review file renderer for docs review output."
```

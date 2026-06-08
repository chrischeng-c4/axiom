---
id: sdd-tools-review-inline-writer
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review inline writer

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/review.rs | function | pub | 40 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/review.rs | function | pub | 135 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// Write inline review into the original artifact file.
/// Returns the filename of the modified artifact.
fn write_inline_review(
    change_dir: &Path,
    file: &str,
    verdict: &str,
    summary: &str,
    checklist: &[Value],
    issues: &[Value],
    iteration: u64,
    display_name: &str,
    change_id: &str,
    spec_id: Option<&str>,
    task_id: Option<&str>,
) -> Result<String> {
    let target_path = if file == "spec" {
        change_dir
            .join("specs")
            .join(format!("{}.md", spec_id.unwrap()))
    } else if file == "implementation" {
        match task_id {
            Some(tid) => change_dir.join(format!("impl_{}.md", tid)),
            None => change_dir.join("impl.md"),
        }
    } else {
        change_dir.join(format!("{}.md", file))
    };

    if !target_path.exists() {
        anyhow::bail!("Artifact file not found: {}", target_path.display());
    }

    let original = std::fs::read_to_string(&target_path)?;

    // Strip existing # Reviews section
    let base = strip_review_section(&original);

    // Update frontmatter fields
    let updated = if verdict == "APPROVED" {
        let c = remove_frontmatter_field(&base, "review_verdict");
        remove_frontmatter_field(&c, "review_iteration")
    } else {
        let c = upsert_frontmatter_field(&base, "review_verdict", verdict);
        upsert_frontmatter_field(&c, "review_iteration", &iteration.to_string())
    };

    // Build final content
    let final_content = if verdict == "APPROVED" {
        format!("{}\n", updated)
    } else {
        let review = build_review_section(
            verdict,
            summary,
            checklist,
            issues,
            iteration,
            display_name,
            change_id,
        );
        format!("{}\n\n{}", updated, review)
    };

    std::fs::write(&target_path, &final_content)?;

    Ok(target_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string())
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "write_inline_review"
    description: "Inline review writer for contexts, gaps, proposals, specs, and implementation artifacts."
```

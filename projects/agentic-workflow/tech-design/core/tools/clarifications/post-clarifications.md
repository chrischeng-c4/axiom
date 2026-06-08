---
id: sdd-tools-clarifications-post-clarifications
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools clarifications post clarifications

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 15 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 68 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_append` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 312 | execute_append(args: &Value, project_root: &Path) -> Result<String> |
| `execute_post_clarifications` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 198 | execute_post_clarifications(     args: &Value,     project_root: &Path,     _action: &str, ) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// Write spec_clarifications.md (post-gap cross-reference artifact).
///
/// Called by `artifact_write` for `artifact="spec_clarifications"`.
/// Writes to `spec_clarifications.md` and sets the appropriate post_clarifications phase.
pub fn execute_post_clarifications(
    args: &Value,
    project_root: &Path,
    _action: &str,
) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let questions = args
        .get("questions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let contradictions = args
        .get("contradictions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let file_path = change_dir.join("spec_clarifications.md");
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut content = format!(
        "---\nchange: {}\ndate: {}\n---\n\n# Spec Clarifications\n\n",
        change_id, today
    );

    // Questions section
    if !questions.is_empty() {
        content.push_str("## Questions\n\n");
        for (i, qa) in questions.iter().enumerate() {
            let topic = qa
                .get("topic")
                .and_then(|v| v.as_str())
                .unwrap_or("General");
            let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
            let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
            let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");

            content.push_str(&format!("### Q{}: {}\n", i + 1, topic));
            content.push_str(&format!("- **Question**: {}\n", question));
            content.push_str(&format!("- **Answer**: {}\n", answer));
            content.push_str(&format!("- **Rationale**: {}\n\n", rationale));
        }
    }

    // Contradictions section
    if !contradictions.is_empty() {
        content.push_str("## Contradictions\n\n");
        for (i, c) in contradictions.iter().enumerate() {
            let source = c
                .get("source_artifact")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let original = c
                .get("original_claim")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let contradicting = c
                .get("contradicting_finding")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolution = c.get("resolution").and_then(|v| v.as_str()).unwrap_or("");

            content.push_str(&format!("### C{}: {} vs gathered context\n", i + 1, source));
            content.push_str(&format!("- **Original claim**: {}\n", original));
            content.push_str(&format!("- **Contradicting finding**: {}\n", contradicting));
            content.push_str(&format!("- **Resolution**: {}\n\n", resolution));
        }
    }

    if questions.is_empty() && contradictions.is_empty() {
        content.push_str("All original clarifications remain valid. No contradictions found.\n");
    }

    std::fs::write(&file_path, content)?;

    // Legacy PostClarifications phase removed; spec_clarifications artifact
    // still written but no phase update needed (handled by run_change flow)

    let phase_str = "spec_clarifications_written";

    let result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifacts": [
            format!(".aw/changes/{}/spec_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "phase": phase_str,
        "questions_count": questions.len(),
        "contradictions_count": contradictions.len(),
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute_post_clarifications"
    description: "Post-gap spec clarifications artifact writer."
```

---
id: sdd-tools-review-execute
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review execute

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
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");
    let change_id = get_required_string(args, "change_id")?;
    let file = get_required_string(args, "file")?;
    let verdict = get_required_string(args, "verdict")?;
    let summary = get_required_string(args, "summary")?;
    let spec_id = get_optional_string(args, "spec_id");
    let task_id = get_optional_string(args, "task_id");
    let iteration = args.get("iteration").and_then(|v| v.as_u64()).unwrap_or(1);

    // Validate change_id
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("change_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate file type
    if !VALID_FILES.contains(&file.as_str()) {
        anyhow::bail!("Invalid file '{}'. Valid: {}", file, VALID_FILES.join(", "));
    }

    // Validate verdict
    if !["APPROVED", "REVIEWED", "REJECTED", "PASS", "NEEDS_REVISION"].contains(&verdict.as_str()) {
        anyhow::bail!("verdict must be APPROVED, REVIEWED, or REJECTED");
    }

    // Normalize legacy verdict names to unified standard
    let verdict = match verdict.as_str() {
        "PASS" => "APPROVED".to_string(),
        "NEEDS_REVISION" => "REVIEWED".to_string(),
        _ => verdict,
    };

    // spec_id required for spec reviews
    if file == "spec" && spec_id.is_none() {
        anyhow::bail!("spec_id is required when file='spec'");
    }

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", change_id);
    }

    // Parse checklist and issues
    let checklist = args
        .get("checklist_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let issues = args
        .get("issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Compute display name
    let display_name = if file == "spec" {
        format!("spec:{}", spec_id.as_deref().unwrap_or("unknown"))
    } else if file == "implementation" && task_id.is_some() {
        format!("implementation:task_{}", task_id.as_deref().unwrap())
    } else {
        file.clone()
    };

    // Write review: inline (# Reviews section appended to artifact)
    let filename = write_inline_review(
        &change_dir,
        &file,
        &verdict,
        &summary,
        &checklist,
        &issues,
        iteration,
        &display_name,
        &change_id,
        spec_id.as_deref(),
        task_id.as_deref(),
    )?;

    // Auto-update STATE.yaml phase based on file type
    let target_phase = match file.as_str() {
        "spec" => Some(StatePhase::ChangeSpecReviewed),
        "implementation" => Some(StatePhase::ChangeImplementationReviewed),
        // Legacy context/gap/proposal types: no phase update (handled by run_change)
        _ => None,
    };
    if let Some(phase) = target_phase {
        super::workflow_common::update_phase(&change_dir, phase)?;
    }

    // Count issues
    let high = issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("HIGH"))
        .count();
    let medium = issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("MEDIUM"))
        .count();
    let low = issues
        .iter()
        .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("LOW"))
        .count();

    let status = format!(
        "Review written: {}\nVerdict: {}\nIssues: {} high, {} medium, {} low",
        filename, verdict, high, medium, low
    );
    if caller == "agent" {
        Ok(status)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            status
        ))
    }
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
      - "execute"
    description: "Review tool execution flow and phase update behavior."
```

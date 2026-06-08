---
id: sdd-spec-plan-file-preparation-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Spec Plan File Preparation

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/spec_plan.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SpecPlanEntry` | projects/agentic-workflow/src/tools/spec_plan.rs | struct | pub | 203 |  |
| `deduplicate_spec_plans` | projects/agentic-workflow/src/tools/spec_plan.rs | function | pub | 262 | deduplicate_spec_plans(change_dir: &Path) -> Result<()> |
| `prepare_specs_from_plan` | projects/agentic-workflow/src/tools/spec_plan.rs | function | pub | 319 | prepare_specs_from_plan(change_dir: &Path, project_root: &Path) -> Result<Vec<String>> |
| `suggest_sections` | projects/agentic-workflow/src/tools/spec_plan.rs | function | pub | 51 | suggest_sections(requirements: &str) -> Vec<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// Read all `groups/*/spec_plan.yaml` files and return `(group_id, entries)` pairs.
///
/// Returns groups sorted alphabetically by group_id.
fn read_all_spec_plans(change_dir: &Path) -> Result<Vec<(String, Vec<SpecPlanEntry>)>> {
    let groups_dir = change_dir.join("groups");
    if !groups_dir.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&groups_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let plan_path = entry.path().join("spec_plan.yaml");
        if !plan_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&plan_path)?;
        let plan: Vec<SpecPlanEntry> = serde_yaml::from_str(&content).unwrap_or_default();
        if !plan.is_empty() {
            let group_id = entry.file_name().to_string_lossy().to_string();
            result.push((group_id, plan));
        }
    }

    Ok(result)
}

// ─── Deduplication ──────────────────────────────────────────────────────────

/// Deduplicate spec_plan entries across groups.
///
/// If two groups target the same `main_spec_ref`, keep the entry in the
/// earlier group (by alphabetical order) and remove it from later groups.
/// Updated `spec_plan.yaml` files are written back to disk.
/// @spec projects/agentic-workflow/tech-design/core/tools/spec_plan_entry.md#changes
pub fn deduplicate_spec_plans(change_dir: &Path) -> Result<()> {
    let all_plans = read_all_spec_plans(change_dir)?;
    if all_plans.len() <= 1 {
        return Ok(());
    }

    let mut seen_refs: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut modified = false;

    for (group_id, plan) in &all_plans {
        let mut deduplicated: Vec<SpecPlanEntry> = Vec::new();
        let mut group_modified = false;

        for entry in plan {
            if seen_refs.contains(&entry.main_spec_ref) {
                group_modified = true;
                modified = true;
                eprintln!(
                    "[deduplicate_spec_plans] Removing duplicate '{}' from group '{}'",
                    entry.main_spec_ref, group_id
                );
                continue;
            }
            seen_refs.insert(entry.main_spec_ref.clone());
            deduplicated.push(entry.clone());
        }

        if group_modified {
            let group_dir = change_dir.join("groups").join(group_id);
            let plan_path = group_dir.join("spec_plan.yaml");
            if deduplicated.is_empty() {
                let _ = std::fs::remove_file(&plan_path);
            } else {
                let yaml = serde_yaml::to_string(&deduplicated)?;
                std::fs::write(&plan_path, yaml)?;
            }
        }
    }

    if modified {
        eprintln!("[deduplicate_spec_plans] Deduplication complete.");
    }

    Ok(())
}

// ─── Preparation ────────────────────────────────────────────────────────────

/// Prepare spec files from spec_plan entries across all groups.
///
/// 1. Reads all `groups/*/spec_plan.yaml` files
/// 2. Runs cross-group deduplication
/// 3. For each entry:
///    - `action: modify` -> copy from `.aw/tech-design/{source}` (or `main_spec_ref`)
///    - `action: create` -> write skeleton with `fill_sections` from `sections`
/// 4. Returns list of prepared spec file relative paths
/// @spec projects/agentic-workflow/tech-design/core/tools/spec_plan_entry.md#changes
pub fn prepare_specs_from_plan(change_dir: &Path, project_root: &Path) -> Result<Vec<String>> {
    // Step 1: Deduplicate across groups
    deduplicate_spec_plans(change_dir)?;

    // Step 2: Read deduplicated plans
    let all_plans = read_all_spec_plans(change_dir)?;
    if all_plans.is_empty() {
        return Ok(vec![]);
    }

    // Validate all spec_plan entries before preparing any files (hard error on failure).
    for (_group_id, plan) in &all_plans {
        for entry in plan {
            let components: Vec<&str> = entry
                .main_spec_ref
                .split('/')
                .filter(|c| !c.is_empty())
                .collect();
            if components.len() < 4 {
                anyhow::bail!(
                    "main_spec_ref must be under a subdirectory (got: {})",
                    entry.main_spec_ref
                );
            }
        }
    }

    let mut prepared = Vec::new();

    for (group_id, plan) in &all_plans {
        let specs_dir = change_dir.join("groups").join(group_id).join("specs");
        std::fs::create_dir_all(&specs_dir)?;

        for entry in plan {
            let spec_path = specs_dir.join(format!("{}.md", entry.spec_id));

            // Skip if already prepared
            if spec_path.exists() {
                continue;
            }

            let (content, base_content) = match entry.action.as_str() {
                "modify" => {
                    let source_ref = entry.source.as_deref().unwrap_or(&entry.main_spec_ref);
                    prepare_modify_spec(
                        &entry.spec_id,
                        source_ref,
                        &entry.main_spec_ref,
                        &entry.sections,
                        project_root,
                    )
                }
                "create" | _ => (
                    prepare_create_spec(&entry.spec_id, &entry.main_spec_ref, &entry.sections),
                    None,
                ),
            };

            // Write base snapshot (.base.md) for modify specs with existing source
            if let Some(ref base) = base_content {
                let base_path = specs_dir.join(format!("{}.base.md", entry.spec_id));
                std::fs::write(&base_path, base)?;
            }

            std::fs::write(&spec_path, &content)?;
            let rel_path = format!("groups/{}/specs/{}.md", group_id, entry.spec_id);
            prepared.push(rel_path);
        }
    }

    Ok(prepared)
}

// ─── Internal Helpers ───────────────────────────────────────────────────────

/// Prepare a "modify" spec: copy from main specs and update frontmatter.
///
/// Returns `(working_content, Option<base_content>)`:
/// - `base_content` is `Some(raw_source)` when the source file exists (unmodified snapshot)
/// - `base_content` is `None` when the source file is missing (fallback to create)
fn prepare_modify_spec(
    spec_id: &str,
    source_ref: &str,
    main_spec_ref: &str,
    sections: &[String],
    project_root: &Path,
) -> (String, Option<String>) {
    let specs_root = crate::shared::workspace::tech_design_path(project_root);
    let source_path = resolve_main_spec_path(project_root, &specs_root, source_ref);

    let raw_source = if source_path.exists() {
        std::fs::read_to_string(&source_path).unwrap_or_default()
    } else {
        // Source file missing — fallback to create, no base snapshot
        return (prepare_create_spec(spec_id, main_spec_ref, sections), None);
    };

    // Clone raw source before any frontmatter mutation — this is the base snapshot
    let base_content = raw_source.clone();

    let mut content = raw_source;

    content = super::review_helpers::upsert_frontmatter_field(&content, "id", spec_id);
    content =
        super::review_helpers::upsert_frontmatter_field(&content, "main_spec_ref", main_spec_ref);
    content = super::review_helpers::upsert_frontmatter_field(&content, "merge_strategy", "extend");

    if !sections.is_empty() {
        let sections_yaml = format!(
            "[{}]",
            sections
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        content = super::review_helpers::upsert_frontmatter_field(
            &content,
            "fill_sections",
            &sections_yaml,
        );
    }

    if !content.contains("\n# Reviews") {
        content.push_str("\n\n# Reviews\n");
    }

    (content, Some(base_content))
}

fn resolve_main_spec_path(project_root: &Path, specs_root: &Path, source_ref: &str) -> PathBuf {
    if let Some((group, rest)) = source_ref.split_once('/') {
        if let Ok(resolved) =
            crate::services::project_registry::resolve_td_root_from_config(project_root, group)
        {
            return PathBuf::from(resolved.root).join(rest);
        }
    }
    specs_root.join(source_ref)
}

/// Prepare a "create" spec: write skeleton with frontmatter.
fn prepare_create_spec(spec_id: &str, main_spec_ref: &str, sections: &[String]) -> String {
    let title = spec_id
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => {
                    let upper: String = f.to_uppercase().collect();
                    upper + c.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut content = UNIVERSAL_SKELETON
        .replace("{spec_id}", spec_id)
        .replace("{title}", &title);

    content =
        super::review_helpers::upsert_frontmatter_field(&content, "main_spec_ref", main_spec_ref);

    if !sections.is_empty() {
        let sections_yaml = format!(
            "[{}]",
            sections
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        content = super::review_helpers::upsert_frontmatter_field(
            &content,
            "fill_sections",
            &sections_yaml,
        );
    }

    content
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/spec_plan.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:standardize-gap-sdd-spec-plan-file-preparation>"
    description: "Spec plan reading, cross-group deduplication, and create/modify spec file preparation flows."
```

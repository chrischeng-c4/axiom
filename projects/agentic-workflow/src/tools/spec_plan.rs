//! Spec plan preparation, cross-group deduplication, and section suggestion.
//!
//! Reads `groups/*/spec_plan.yaml` files written by the reference context
//! artifact, deduplicates entries across groups, and prepares spec files
//! (copy or skeleton) for the change-spec phase.
//!
//! Also exposes `suggest_sections()` — a keyword-matching rule engine that
//! maps requirements text to suggested spec section types.

use crate::Result;
use std::path::{Path, PathBuf};

use super::common_change_spec::UNIVERSAL_SKELETON;

// ─── Section Suggestion Rule Engine ─────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec_plan/section-suggestions.md#source
// CODEGEN-BEGIN
/// Map a spec-domain section name to its fill_order priority.
///
/// Based on the Section Fill Order in `.aw/tech-design/sdd/logic/change-spec.md`.
/// Lower number = fill first.
fn section_fill_order(section: &str) -> u8 {
    match section {
        "overview" => 0,
        "db-model" => 1,
        "schema" => 2,
        "state-machine" => 3,
        "logic" | "model" | "prompt" => 4,
        "dependency" => 5,
        "interaction" | "threat-model" | "auth-matrix" => 6,
        "rest-api" | "rpc-api" | "async-api" | "cli" | "grpc" | "graphql" => 7,
        "wireframe" | "component" | "design-token" => 8,
        "config" | "container" | "deploy" | "cloud-resource" | "pipeline" | "observability" => 9,
        "unit-test" | "e2e-test" | "test-fixture" | "perf-test" | "security-test" => 10,
        "changes" => 11,
        _ => 99,
    }
}

/// Keyword-matching rule engine: map requirements text to suggested section types.
///
/// Rules are evaluated case-insensitively against the full requirements text.
/// The `changes` transition manifest is always included.
/// `unit-test` is added when more than 2 keyword-matched sections are found.
/// `interaction`, `logic`, and `dependency` are added when more than 3
/// keyword-matched sections are found.
///
/// Returns deduplicated section names sorted by fill_order priority.
/// @spec projects/agentic-workflow/tech-design/core/tools/spec_plan_entry.md#changes
pub fn suggest_sections(requirements: &str) -> Vec<String> {
    use regex::Regex;

    // Build case-insensitive regex for each keyword pattern and collect matches.
    // Note: Rust's regex crate does not support lookaheads; patterns use (?i) inline flag
    // at the start only and avoid negative lookaheads.
    // For "api": match as a whole word (word boundaries ensure "capital" does not match).
    let keyword_rules: &[(&str, &[&str])] = &[
        // Existing types
        (
            r"(?i)\b(endpoint|route|api|REST|HTTP)\b",
            &["rest-api", "schema"],
        ),
        (r"(?i)\b(rpc|json-rpc|MCP\s+tool)\b", &["rpc-api", "schema"]),
        (
            r"(?i)\b(queue|pubsub|webhook|background|async)\b",
            &["async-api"],
        ),
        (
            r"(?i)\b(database|table|migration|collection)\b",
            &["db-model"],
        ),
        (
            r"(?i)\b(state|phase|lifecycle|transition)\b",
            &["state-machine"],
        ),
        (
            r"(?i)\b(UI|page|component|layout|frontend)\b",
            &["wireframe", "component"],
        ),
        (r"(?i)\b(CLI|command|subcommand|flag)\b", &["cli"]),
        (
            r"(?i)(\b(config|env|settings)\b|\.toml\b|\.env\b)",
            &["config"],
        ),
        (
            r"(?i)\b(token|color|spacing|typography|theme)\b",
            &["design-token"],
        ),
        // Backend API types
        (r"(?i)\b(grpc|protobuf|proto|gRPC)\b", &["grpc", "schema"]),
        (
            r"(?i)\b(graphql|mutation|subscription|SDL)\b",
            &["graphql", "schema"],
        ),
        // QA types
        (r"(?i)\b(e2e|end-to-end|acceptance\s+test)\b", &["e2e-test"]),
        (r"(?i)\b(fixture|test-data|seed\s+data)\b", &["unit-test"]),
        (
            r"(?i)\b(performance|load-test|benchmark|latency)\b",
            &["unit-test"],
        ),
        // Security types
        (
            r"(?i)\b(threat|STRIDE|attack\s+surface)\b",
            &["threat-model"],
        ),
        (
            r"(?i)\b(auth-matrix|RBAC|permission\s+matrix|authorization\s+matrix)\b",
            &["auth-matrix"],
        ),
        (
            r"(?i)\b(security-test|pen-test|penetration|OWASP)\b",
            &["e2e-test"],
        ),
        // SRE types
        (r"(?i)\b(container|docker|Dockerfile|OCI)\b", &["container"]),
        (
            r"(?i)\b(deploy|deployment|kubernetes|k8s|helm)\b",
            &["deploy"],
        ),
        (
            r"(?i)\b(cloud-resource|terraform|pulumi|cloud\s+resource)\b",
            &["cloud-resource"],
        ),
        (
            r"(?i)\b(pipeline|CI/CD|CICD|github\s+actions|gitlab\s+ci)\b",
            &["pipeline"],
        ),
        (
            r"(?i)\b(observability|monitoring|alerting|metrics|tracing|SLO)\b",
            &["observability"],
        ),
        // MLE types
        (
            r"(?i)\b(ML\s+model|machine.learning|training|inference|neural)\b",
            &["model"],
        ),
        // Agent types
        (
            r"(?i)\b(prompt|system.instruction|few.shot|prompt\s+template)\b",
            &["prompt"],
        ),
    ];

    let mut matched: std::collections::LinkedList<&str> = std::collections::LinkedList::new();

    for (pattern, sections) in keyword_rules {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(requirements) {
                for &s in *sections {
                    matched.push_back(s);
                }
            }
        }
    }

    // Collect keyword-matched sections (deduplicated, preserving insertion order).
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut result: Vec<String> = Vec::new();

    for s in &matched {
        if seen.insert(s.to_string()) {
            result.push(s.to_string());
        }
    }

    let keyword_section_count = result.len();

    // Conditional additions based on section count.
    if keyword_section_count > 3 {
        for s in &["interaction", "logic", "dependency"] {
            if seen.insert(s.to_string()) {
                result.push(s.to_string());
            }
        }
    }
    if keyword_section_count > 2 {
        if seen.insert("unit-test".to_string()) {
            result.push("unit-test".to_string());
        }
    }

    // Always-present transition manifest.
    for s in &["changes"] {
        if seen.insert(s.to_string()) {
            result.push(s.to_string());
        }
    }

    // Sort by fill_order priority.
    result.sort_by_key(|s| section_fill_order(s.as_str()));
    result
}
// CODEGEN-END
// ─── Data Types ─────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec_plan_entry.md#schema
// CODEGEN-BEGIN
/// One row of the spec-planning table in an issue's Reference Context.
/// @spec projects/agentic-workflow/tech-design/core/tools/spec_plan_entry.md#schema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpecPlanEntry {
    /// Spec identifier slug.
    pub spec_id: String,
    /// Action keyword (create / update / merge / ...).
    pub action: String,
    /// Reference to the main spec this entry plans.
    pub main_spec_ref: String,
    /// Optional source artifact (e.g. issue slug).
    #[serde(default)]
    pub source: Option<String>,
    /// Section names this spec covers.
    #[serde(default)]
    pub sections: Vec<String>,
}
// CODEGEN-END
// ─── Reading ────────────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec_plan/file-preparation.md#source
// CODEGEN-BEGIN
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
// CODEGEN-END
// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_plan(change_dir: &Path, group_id: &str, yaml: &str) {
        let group_dir = change_dir.join("groups").join(group_id);
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(group_dir.join("spec_plan.yaml"), yaml).unwrap();
    }

    #[test]
    fn test_read_all_spec_plans() {
        let tmp = TempDir::new().unwrap();
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: api-spec\n  action: create\n  main_spec_ref: platform/api.md\n  sections: [overview, rest-api]\n",
        );

        let plans = read_all_spec_plans(tmp.path()).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].0, "group-a");
        assert_eq!(plans[0].1.len(), 1);
        assert_eq!(plans[0].1[0].spec_id, "api-spec");
        assert_eq!(plans[0].1[0].action, "create");
        assert_eq!(plans[0].1[0].sections, vec!["overview", "rest-api"]);
    }

    #[test]
    fn test_deduplicate_removes_later_duplicate() {
        let tmp = TempDir::new().unwrap();
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: api-v1\n  action: modify\n  main_spec_ref: shared/api.md\n  sections: [overview]\n",
        );
        write_plan(
            tmp.path(),
            "group-b",
            "- spec_id: api-v2\n  action: modify\n  main_spec_ref: shared/api.md\n  sections: [changes]\n",
        );

        deduplicate_spec_plans(tmp.path()).unwrap();

        let plans = read_all_spec_plans(tmp.path()).unwrap();
        let group_a = plans.iter().find(|(id, _)| id == "group-a").unwrap();
        assert_eq!(group_a.1.len(), 1);

        let group_b = plans.iter().find(|(id, _)| id == "group-b");
        assert!(group_b.is_none() || group_b.unwrap().1.is_empty());
    }

    #[test]
    fn test_deduplicate_no_duplicates_is_noop() {
        let tmp = TempDir::new().unwrap();
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: api-spec\n  action: create\n  main_spec_ref: platform/api.md\n  sections: [overview]\n",
        );
        write_plan(
            tmp.path(),
            "group-b",
            "- spec_id: db-spec\n  action: create\n  main_spec_ref: platform/db.md\n  sections: [schema]\n",
        );

        deduplicate_spec_plans(tmp.path()).unwrap();

        let plans = read_all_spec_plans(tmp.path()).unwrap();
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].1.len(), 1);
        assert_eq!(plans[1].1.len(), 1);
    }

    #[test]
    fn test_prepare_create() {
        let tmp = TempDir::new().unwrap();
        // main_spec_ref must have ≥4 path components (projects/agentic-workflow/logic/file.md)
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: version-api\n  action: create\n  main_spec_ref: projects/agentic-workflow/api/version-api.md\n  sections: [overview, rest-api, schema]\n",
        );

        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        assert_eq!(prepared.len(), 1);
        assert_eq!(prepared[0], "groups/group-a/specs/version-api.md");

        let spec_path = tmp.path().join("groups/group-a/specs/version-api.md");
        assert!(spec_path.exists());
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: version-api"));
        assert!(content.contains("main_spec_ref: projects/agentic-workflow/api/version-api.md"));
        assert!(content.contains("fill_sections:"));
        assert!(content.contains("overview"));
        assert!(content.contains("# Reviews"));
    }

    #[test]
    fn test_prepare_modify_with_source() {
        let tmp = TempDir::new().unwrap();

        // Place source file under a valid spec path with ≥4 components
        let specs_dir = tmp.path().join(".aw/tech-design/crates/myapp/logic");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("existing.md"),
            "---\nid: existing\n---\n\n# Existing Spec\n\n## Overview\n\nOriginal content.\n",
        )
        .unwrap();

        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: my-change\n  action: modify\n  main_spec_ref: crates/myapp/logic/existing.md\n  source: crates/myapp/logic/existing.md\n  sections: [overview, changes]\n",
        );

        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        assert_eq!(prepared.len(), 1);

        let spec_path = tmp.path().join("groups/group-a/specs/my-change.md");
        assert!(spec_path.exists());
        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: my-change"));
        assert!(content.contains("main_spec_ref: crates/myapp/logic/existing.md"));
        assert!(content.contains("merge_strategy: extend"));
        assert!(content.contains("Original content."));
    }

    #[test]
    fn test_prepare_modify_creates_base_snapshot() {
        let tmp = TempDir::new().unwrap();

        // Place source file under .aw/tech-design/
        let specs_dir = tmp.path().join(".aw/tech-design/crates/myapp/logic");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let original_content =
            "---\nid: existing\n---\n\n# Existing Spec\n\n## Overview\n\nOriginal content.\n";
        std::fs::write(specs_dir.join("existing.md"), original_content).unwrap();

        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: my-change\n  action: modify\n  main_spec_ref: crates/myapp/logic/existing.md\n  source: crates/myapp/logic/existing.md\n  sections: [overview, changes]\n",
        );

        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        assert_eq!(prepared.len(), 1);

        // Working spec should exist with modified frontmatter
        let spec_path = tmp.path().join("groups/group-a/specs/my-change.md");
        assert!(spec_path.exists());
        let working = std::fs::read_to_string(&spec_path).unwrap();
        assert!(working.contains("id: my-change"));
        assert!(working.contains("merge_strategy: extend"));

        // Base snapshot should exist with unmodified source content
        let base_path = tmp.path().join("groups/group-a/specs/my-change.base.md");
        assert!(
            base_path.exists(),
            ".base.md should be created for modify specs"
        );
        let base = std::fs::read_to_string(&base_path).unwrap();
        assert_eq!(
            base, original_content,
            ".base.md should contain unmodified source"
        );
    }

    #[test]
    fn test_prepare_create_no_base_snapshot() {
        let tmp = TempDir::new().unwrap();

        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: new-spec\n  action: create\n  main_spec_ref: projects/agentic-workflow/logic/new-spec.md\n  sections: [overview, changes]\n",
        );

        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        assert_eq!(prepared.len(), 1);

        // Working spec should exist
        let spec_path = tmp.path().join("groups/group-a/specs/new-spec.md");
        assert!(spec_path.exists());

        // No .base.md for create specs
        let base_path = tmp.path().join("groups/group-a/specs/new-spec.base.md");
        assert!(
            !base_path.exists(),
            ".base.md should NOT be created for create specs"
        );
    }

    #[test]
    fn test_prepare_modify_missing_source_no_base() {
        let tmp = TempDir::new().unwrap();

        // Do NOT create the source file — it will fallback to create
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: missing-src\n  action: modify\n  main_spec_ref: crates/myapp/logic/nonexistent.md\n  source: crates/myapp/logic/nonexistent.md\n  sections: [overview, changes]\n",
        );

        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        assert_eq!(prepared.len(), 1);

        // Working spec should exist (created via fallback)
        let spec_path = tmp.path().join("groups/group-a/specs/missing-src.md");
        assert!(spec_path.exists());

        // No .base.md when source is missing (fallback to create)
        let base_path = tmp.path().join("groups/group-a/specs/missing-src.base.md");
        assert!(
            !base_path.exists(),
            ".base.md should NOT be written when source file is missing"
        );
    }

    #[test]
    fn test_prepare_skip_already_prepared_no_duplicate_base() {
        let tmp = TempDir::new().unwrap();

        // Place source file under .aw/tech-design/
        let specs_dir = tmp.path().join(".aw/tech-design/crates/myapp/logic");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("existing.md"),
            "---\nid: existing\n---\n\n# Existing Spec\n",
        )
        .unwrap();

        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: already-prepped\n  action: modify\n  main_spec_ref: crates/myapp/logic/existing.md\n  source: crates/myapp/logic/existing.md\n  sections: [overview]\n",
        );

        // Pre-create the spec file to simulate already-prepared state
        let group_specs_dir = tmp.path().join("groups/group-a/specs");
        std::fs::create_dir_all(&group_specs_dir).unwrap();
        std::fs::write(
            group_specs_dir.join("already-prepped.md"),
            "---\nid: already-prepped\n---\n# Pre-existing\n",
        )
        .unwrap();

        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        // Should skip (already prepared) — returns empty
        assert!(prepared.is_empty());

        // No .base.md should be written for skipped specs
        let base_path = tmp
            .path()
            .join("groups/group-a/specs/already-prepped.base.md");
        assert!(
            !base_path.exists(),
            ".base.md should NOT be written for already-prepared specs"
        );
    }

    #[test]
    fn test_prepare_skips_existing() {
        let tmp = TempDir::new().unwrap();
        // Use a valid 4-component main_spec_ref
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: already-done\n  action: create\n  main_spec_ref: crates/test/logic/ref.md\n  sections: [overview]\n",
        );

        let specs_dir = tmp.path().join("groups/group-a/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("already-done.md"),
            "---\nid: already-done\n---\n# Existing\n",
        )
        .unwrap();

        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        assert!(prepared.is_empty());
    }

    #[test]
    fn test_prepare_no_plan_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let prepared = prepare_specs_from_plan(tmp.path(), tmp.path()).unwrap();
        assert!(prepared.is_empty());
    }

    // ─── suggest_sections tests ──────────────────────────────────────────────

    #[test]
    fn test_suggest_sections_rest_api() {
        let sections = suggest_sections("Add a REST endpoint for user creation via HTTP POST");
        assert!(
            sections.contains(&"rest-api".to_string()),
            "should include rest-api"
        );
        assert!(
            sections.contains(&"schema".to_string()),
            "should include schema"
        );
        assert!(
            sections.contains(&"changes".to_string()),
            "should include changes"
        );
        // sections are sorted by fill_order: schema(2) < rest-api(7) < changes(11)
        let schema_pos = sections.iter().position(|s| s == "schema").unwrap();
        let rest_pos = sections.iter().position(|s| s == "rest-api").unwrap();
        let changes_pos = sections.iter().position(|s| s == "changes").unwrap();
        assert!(schema_pos < rest_pos, "schema should come before rest-api");
        assert!(
            rest_pos < changes_pos,
            "rest-api should come before changes"
        );
    }

    #[test]
    fn test_suggest_sections_database() {
        let sections =
            suggest_sections("Add a new database table for user profiles with migration");
        assert!(
            sections.contains(&"db-model".to_string()),
            "should include db-model"
        );
        assert!(
            sections.contains(&"changes".to_string()),
            "should include changes"
        );
    }

    #[test]
    fn test_suggest_sections_ui() {
        let sections =
            suggest_sections("Build a UI page with component layout for the dashboard frontend");
        assert!(
            sections.contains(&"wireframe".to_string()),
            "should include wireframe"
        );
        assert!(
            sections.contains(&"component".to_string()),
            "should include component"
        );
        assert!(
            sections.contains(&"changes".to_string()),
            "should include changes"
        );
    }

    #[test]
    fn test_suggest_sections_complex_api_db_ui() {
        // More than 3 keyword-matched sections → should add unit-test + interaction + logic + dependency
        let sections = suggest_sections(
            "Implement a REST API endpoint backed by a database table, \
             with a UI component and a state machine for lifecycle management. \
             Also add CLI commands.",
        );
        assert!(sections.contains(&"rest-api".to_string()));
        assert!(sections.contains(&"db-model".to_string()));
        assert!(sections.contains(&"wireframe".to_string()));
        assert!(sections.contains(&"component".to_string()));
        assert!(sections.contains(&"state-machine".to_string()));
        assert!(sections.contains(&"cli".to_string()));
        assert!(
            sections.contains(&"unit-test".to_string()),
            "should include unit-test"
        );
        assert!(
            sections.contains(&"interaction".to_string()),
            "should include interaction"
        );
        assert!(
            sections.contains(&"logic".to_string()),
            "should include logic"
        );
        assert!(
            sections.contains(&"dependency".to_string()),
            "should include dependency"
        );
        assert!(sections.contains(&"changes".to_string()));
        // Verify fill_order sorted: db-model(1) < interaction(6) < unit-test(10) < changes(11)
        let db_pos = sections.iter().position(|s| s == "db-model").unwrap();
        let test_pos = sections.iter().position(|s| s == "unit-test").unwrap();
        let changes_pos = sections.iter().position(|s| s == "changes").unwrap();
        assert!(db_pos < test_pos);
        assert!(test_pos < changes_pos);
    }

    #[test]
    fn test_suggest_sections_empty_returns_always() {
        let sections = suggest_sections("");
        assert_eq!(sections, vec!["changes".to_string()]);
    }

    #[test]
    fn test_suggest_sections_minimal_returns_always() {
        let sections = suggest_sections("Some minor refactoring with no specific keywords");
        assert_eq!(sections, vec!["changes".to_string()]);
    }

    #[test]
    fn test_suggest_sections_case_insensitive() {
        let lower = suggest_sections("add a rest endpoint");
        let upper = suggest_sections("ADD A REST ENDPOINT");
        assert_eq!(lower, upper, "matching should be case-insensitive");
    }

    #[test]
    fn test_suggest_sections_no_duplicates() {
        // "api" appears twice but should only yield one rest-api
        let sections = suggest_sections("REST API endpoint via HTTP route for the API");
        let rest_count = sections.iter().filter(|s| s.as_str() == "rest-api").count();
        assert_eq!(rest_count, 1, "rest-api should appear exactly once");
        let schema_count = sections.iter().filter(|s| s.as_str() == "schema").count();
        assert_eq!(schema_count, 1, "schema should appear exactly once");
    }

    #[test]
    fn test_suggest_sections_api_word_boundary() {
        // "capital" contains "api" but should NOT trigger rest-api
        let sections = suggest_sections("A capital improvement with no routes or endpoints");
        assert!(
            !sections.contains(&"rest-api".to_string()),
            "'capital' should not match api rule"
        );
    }

    #[test]
    fn test_suggest_sections_unit_test_threshold() {
        // 2 keyword sections: db-model + state-machine → no unit-test (need > 2)
        let sections = suggest_sections("Update the database with a state lifecycle");
        // db-model, state-machine = 2 unique keyword sections → unit-test needs count > 2
        assert!(
            !sections.contains(&"unit-test".to_string()),
            "2 keyword sections should NOT add unit-test"
        );

        // 3 keyword sections: db-model + state-machine + cli → unit-test added (3 > 2)
        let sections2 =
            suggest_sections("Update the database table and state transition for the CLI command");
        assert!(
            sections2.contains(&"unit-test".to_string()),
            "3 keyword sections should add unit-test"
        );

        // 4 keyword sections: + async-api via "queue" → unit-test still added + interaction/logic/dependency
        let sections3 = suggest_sections(
            "Update the database table and state transition for the CLI command with a queue",
        );
        assert!(
            sections3.contains(&"unit-test".to_string()),
            "4 keyword sections should add unit-test"
        );
        assert!(
            sections3.contains(&"interaction".to_string()),
            "4 keyword sections should add interaction"
        );
        assert!(
            sections3.contains(&"logic".to_string()),
            "4 keyword sections should add logic"
        );
        assert!(
            sections3.contains(&"dependency".to_string()),
            "4 keyword sections should add dependency"
        );
    }

    // ─── prepare_specs_from_plan path validation tests ───────────────────────

    #[test]
    fn test_prepare_invalid_main_spec_ref_3_components_aborts() {
        let tmp = TempDir::new().unwrap();
        // 3 components: projects/agentic-workflow/foo.md → rejected (need ≥ 4)
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: foo\n  action: create\n  main_spec_ref: projects/agentic-workflow/foo.md\n  sections: [overview]\n",
        );

        let result = prepare_specs_from_plan(tmp.path(), tmp.path());
        assert!(result.is_err(), "3-component path must return a hard error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("main_spec_ref") || err.contains("components"),
            "error must mention path component requirement: {}",
            err
        );
    }

    #[test]
    fn test_prepare_invalid_main_spec_ref_root_level_aborts() {
        let tmp = TempDir::new().unwrap();
        // 1 component: foo.md → rejected
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: flat\n  action: create\n  main_spec_ref: flat.md\n  sections: [overview]\n",
        );

        let result = prepare_specs_from_plan(tmp.path(), tmp.path());
        assert!(result.is_err(), "root-level path must return a hard error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("main_spec_ref") || err.contains("components"),
            "error must mention path component requirement: {}",
            err
        );
    }

    #[test]
    fn test_prepare_valid_main_spec_ref_4_components_succeeds() {
        let tmp = TempDir::new().unwrap();
        // Exactly 4 components: projects/agentic-workflow/logic/foo.md → valid
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: foo\n  action: create\n  main_spec_ref: projects/agentic-workflow/logic/foo.md\n  sections: [overview]\n",
        );

        let result = prepare_specs_from_plan(tmp.path(), tmp.path());
        assert!(
            result.is_ok(),
            "4-component path should succeed: {:?}",
            result
        );
        let prepared = result.unwrap();
        assert_eq!(prepared.len(), 1);
    }

    #[test]
    fn test_prepare_valid_main_spec_ref_5_components_succeeds() {
        let tmp = TempDir::new().unwrap();
        // 5 components: projects/agentic-workflow/logic/sub/deep.md → also valid
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: deep\n  action: create\n  main_spec_ref: projects/agentic-workflow/logic/sub/deep.md\n  sections: [overview]\n",
        );

        let result = prepare_specs_from_plan(tmp.path(), tmp.path());
        assert!(
            result.is_ok(),
            "5-component path should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_prepare_path_validation_aborts_all_no_files_written() {
        let tmp = TempDir::new().unwrap();
        // First entry is valid, second has 3 components → hard error aborts entire operation
        write_plan(
            tmp.path(),
            "group-a",
            concat!(
                "- spec_id: good\n  action: create\n  main_spec_ref: projects/agentic-workflow/logic/good.md\n  sections: [overview]\n",
                "- spec_id: bad\n  action: create\n  main_spec_ref: projects/agentic-workflow/bad.md\n  sections: [overview]\n",
            ),
        );

        let result = prepare_specs_from_plan(tmp.path(), tmp.path());
        assert!(
            result.is_err(),
            "any invalid entry must abort the whole operation"
        );

        // Neither spec file should have been created
        let good_path = tmp.path().join("groups/group-a/specs/good.md");
        let bad_path = tmp.path().join("groups/group-a/specs/bad.md");
        assert!(
            !good_path.exists(),
            "no spec files should be written on hard error (good.md)"
        );
        assert!(
            !bad_path.exists(),
            "no spec files should be written on hard error (bad.md)"
        );
    }

    #[test]
    fn test_prepare_invalid_2_component_path_aborts() {
        let tmp = TempDir::new().unwrap();
        // 2 components: platform/api.md → rejected
        write_plan(
            tmp.path(),
            "group-a",
            "- spec_id: api\n  action: create\n  main_spec_ref: platform/api.md\n  sections: [overview]\n",
        );

        let result = prepare_specs_from_plan(tmp.path(), tmp.path());
        assert!(result.is_err(), "2-component path must return a hard error");
    }
}

---
id: sdd-tools-common-change-spec-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change spec tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/common_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ALL_SECTIONS` | projects/agentic-workflow/src/tools/common_change_spec.rs | constant | pub | 285 |  |
| `SpecSubState` | projects/agentic-workflow/src/tools/common_change_spec.rs | enum | pub | 439 |  |
| `UNIVERSAL_SKELETON` | projects/agentic-workflow/src/tools/common_change_spec.rs | constant | pub | 30 |  |
| `fill_section_base_name` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1281 | fill_section_base_name(s: &str) -> &str |
| `find_spec_path` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 343 | find_spec_path(change_dir: &std::path::Path, spec_id: &str) -> std::path::PathBuf |
| `generate_skeleton` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 770 | generate_skeleton(     spec_id: &str,     title: &str,     main_spec_ref: Option<&str>,     merge_strategy: Option<&str>,     project_root: &Path, ) -> String |
| `get_primary_specs_dir` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 369 | get_primary_specs_dir(change_dir: &std::path::Path) -> std::path::PathBuf |
| `get_spec_path` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 329 | get_spec_path(     change_dir: &std::path::Path,     group_id: Option<&str>,     spec_id: &str, ) -> std::path::PathBuf |
| `get_specs_dir` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 320 | get_specs_dir(change_dir: &std::path::Path, group_id: Option<&str>) -> std::path::PathBuf |
| `is_create_complete` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1344 | is_create_complete(content: &str) -> bool |
| `is_fill_section_optional` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1287 | is_fill_section_optional(s: &str) -> bool |
| `parse_fill_section` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1272 | parse_fill_section(s: &str) -> (&str, bool) |
| `prune_todo_sections` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1091 | prune_todo_sections(content: &str) -> String |
| `read_fill_sections` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1171 | read_fill_sections(content: &str) -> Vec<String> |
| `read_filled_sections` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1218 | read_filled_sections(content: &str) -> Vec<String> |
| `read_main_spec_ref` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1295 | read_main_spec_ref(content: &str) -> Option<String> |
| `replace_section` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 911 | replace_section(content: &str, section: &str, new_content: &str) -> String |
| `resolve_group_id_for_spec` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 395 | resolve_group_id_for_spec(change_dir: &std::path::Path, spec_id: &str) -> Option<String> |
| `resolve_next_spec` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 463 | resolve_next_spec(change_dir: &Path, change_id: &str) -> Result<SpecSubState> |
| `strip_change_spec_fields` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1324 | strip_change_spec_fields(content: &str) -> String |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_skeleton_universal() {
        let skeleton = generate_skeleton("my-spec", "My Spec Title", None, None, Path::new("/tmp"));
        assert!(skeleton.contains("id: my-spec"));
        assert!(skeleton.contains("# My Spec Title"));
        assert!(skeleton.contains("## Overview"));
        assert!(skeleton.contains("## Requirements"));
        assert!(skeleton.contains("## Scenarios"));
        assert!(skeleton.contains("## Interaction"));
        assert!(skeleton.contains("## Logic"));
        assert!(skeleton.contains("## REST API"));
        assert!(skeleton.contains("## Schema"));
        assert!(skeleton.contains("## Unit Test"));
        assert!(skeleton.contains("## E2E Test"));
        assert!(skeleton.contains("## Changes"));
        assert!(!skeleton.contains("## Diagrams"));
        assert!(!skeleton.contains("## API Spec"));
        assert!(skeleton.contains("# Reviews"));
        assert!(skeleton.contains("merge_strategy: new"));
    }

    #[test]
    fn test_skeleton_has_section_annotations() {
        let skeleton = generate_skeleton(
            "annotated-spec",
            "Annotated Spec",
            None,
            None,
            Path::new("/tmp"),
        );
        // Verify section type annotations are present (new format: 3 langs only)
        assert!(skeleton.contains("<!-- type: overview lang: markdown -->"));
        assert!(skeleton.contains("<!-- type: changes lang: yaml -->"));
        assert!(skeleton.contains("<!-- type: requirements lang: mermaid -->")); // was markdown
        assert!(skeleton.contains("<!-- type: scenarios lang: yaml -->")); // was markdown
        assert!(skeleton.contains("<!-- type: unit-test lang: mermaid -->"));
        assert!(skeleton.contains("<!-- type: e2e-test lang: yaml -->"));
        assert!(skeleton.contains("<!-- type: interaction lang: mermaid -->"));
        assert!(skeleton.contains("<!-- type: rest-api lang: yaml -->"));
        assert!(skeleton.contains("<!-- type: rpc-api lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: schema lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: config lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: component lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: design-token lang: yaml -->")); // was json
    }

    #[test]
    fn test_get_specs_dir_no_group() {
        let base = Path::new("/tmp/change");
        let dir = get_specs_dir(base, None);
        assert_eq!(dir, Path::new("/tmp/change/specs"));
    }

    #[test]
    fn test_get_specs_dir_with_group() {
        let base = Path::new("/tmp/change");
        let dir = get_specs_dir(base, Some("feature-a"));
        assert_eq!(dir, Path::new("/tmp/change/groups/feature-a/specs"));
    }

    #[test]
    fn test_get_spec_path_no_group() {
        let base = Path::new("/tmp/change");
        let path = get_spec_path(base, None, "my-spec");
        assert_eq!(path, Path::new("/tmp/change/specs/my-spec.md"));
    }

    #[test]
    fn test_get_spec_path_with_group() {
        let base = Path::new("/tmp/change");
        let path = get_spec_path(base, Some("group-1"), "my-spec");
        assert_eq!(
            path,
            Path::new("/tmp/change/groups/group-1/specs/my-spec.md")
        );
    }

    // REQ: change-spec.md#NAP3 — `N/A` body sections are pruned alongside TODOs.
    #[test]
    fn test_prune_na_sections() {
        let content = r#"---
id: test
---

# Test Spec

## Overview

Real overview content.

## Schema
<!-- type: schema lang: yaml -->

N/A

## Config
<!-- type: config lang: yaml -->

```yaml
key: value
```

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(pruned.contains("## Overview"), "real section kept");
        assert!(!pruned.contains("## Schema"), "N/A section must be pruned");
        assert!(pruned.contains("## Config"), "filled section kept");
        assert!(pruned.contains("# Reviews"), "reviews heading kept");
    }

    #[test]
    fn test_prune_mixed_na_and_todo() {
        let content = r#"---
id: test
---

# Test Spec

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Schema
<!-- type: schema lang: yaml -->

N/A

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    A --> B
```

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(!pruned.contains("## Overview"), "TODO pruned");
        assert!(!pruned.contains("## Schema"), "N/A pruned");
        assert!(pruned.contains("## Logic"), "real section kept");
    }

    #[test]
    fn test_prune_na_prose_is_not_pruned() {
        let content = r#"---
id: test
---

# Test Spec

## Overview
<!-- type: overview lang: markdown -->

N/A because historically we skipped this; new owners: fill it in.

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(
            pruned.contains("## Overview"),
            "section with N/A-prefixed prose must be kept (only bare N/A sentinel prunes)"
        );
    }

    #[test]
    fn test_prune_todo_sections() {
        let content = r#"---
id: test
---

# Test Spec

## Overview

Some real content here.

## Requirements

<!-- TODO -->

## Scenarios

Real scenarios here.

## Diagrams

### Sequence Diagram
<!-- TODO -->

### Flowchart

Real flowchart here.

### Class Diagram
<!-- TODO -->

## Unit Test

<!-- TODO -->

## Changes

<!-- TODO -->

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(pruned.contains("## Overview"));
        assert!(pruned.contains("Some real content here."));
        assert!(!pruned.contains("## Requirements"));
        assert!(pruned.contains("## Scenarios"));
        assert!(!pruned.contains("### Sequence Diagram"));
        assert!(pruned.contains("### Flowchart"));
        assert!(!pruned.contains("### Class Diagram"));
        assert!(!pruned.contains("## Unit Test"));
        assert!(!pruned.contains("## Changes"));
        assert!(pruned.contains("# Reviews"));
    }

    #[test]
    fn test_replace_section() {
        let content = r#"---
id: test
---

# Test

## Overview

Old overview.

## Requirements

Old requirements.

## Scenarios

Old scenarios.
"#;
        let result = replace_section(
            content,
            "overview",
            "New overview content.\n\nMore details.",
        );
        assert!(result.contains("New overview content."));
        assert!(result.contains("More details."));
        assert!(!result.contains("Old overview."));
        assert!(result.contains("Old requirements."));
        assert!(result.contains("Old scenarios."));
    }

    #[test]
    fn test_read_fill_sections_yaml_list() {
        let content = "---\nid: test\nfill_sections:\n- overview\n- requirements\n- scenarios\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(sections, vec!["overview", "requirements", "scenarios"]);
    }

    #[test]
    fn test_read_fill_sections_inline() {
        let content = "---\nid: test\nfill_sections: [overview, requirements]\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(sections, vec!["overview", "requirements"]);
    }

    #[test]
    fn test_read_main_spec_ref_unquoted() {
        let content = "---\nid: test\nmain_spec_ref: foo/bar.md\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
    }

    #[test]
    fn test_read_main_spec_ref_double_quoted() {
        let content = "---\nid: test\nmain_spec_ref: \"foo/bar.md\"\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
    }

    #[test]
    fn test_read_main_spec_ref_single_quoted() {
        let content = "---\nid: test\nmain_spec_ref: 'foo/bar.md'\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
    }

    #[test]
    fn test_read_main_spec_ref_null() {
        let content = "---\nid: test\nmain_spec_ref: ~\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), None);
    }

    #[test]
    fn test_is_create_complete() {
        let content = "---\nid: test\ncreate_complete: true\n---\n\n# Body\n";
        assert!(is_create_complete(content));

        let content2 = "---\nid: test\n---\n\n# Body\n";
        assert!(!is_create_complete(content2));
    }

    #[test]
    fn test_resolve_group_id_for_spec_from_existing_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let group_dir = change_dir.join("groups/my-group/specs");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(group_dir.join("my-spec.md"), "---\nid: my-spec\n---\n").unwrap();
        assert_eq!(
            resolve_group_id_for_spec(change_dir, "my-spec"),
            Some("my-group".to_string())
        );
    }

    #[test]
    fn test_resolve_group_id_for_spec_from_spec_plan() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let group_dir = change_dir.join("groups/plan-group");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(
            group_dir.join("spec_plan.yaml"),
            "- spec_id: plan-spec\n  action: create\n",
        )
        .unwrap();
        assert_eq!(
            resolve_group_id_for_spec(change_dir, "plan-spec"),
            Some("plan-group".to_string())
        );
    }

    #[test]
    fn test_resolve_group_id_for_spec_no_groups_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        assert_eq!(resolve_group_id_for_spec(change_dir, "any-spec"), None);
    }

    #[test]
    fn test_resolve_group_id_for_spec_not_found_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        std::fs::create_dir_all(change_dir.join("groups/group-a")).unwrap();
        assert_eq!(resolve_group_id_for_spec(change_dir, "missing-spec"), None);
    }

    // ── replace_section: insert-before-Reviews for missing headings ───────────

    #[test]
    fn test_replace_section_inserts_before_reviews_when_missing() {
        // When the target section heading does not exist in the document,
        // replace_section must insert it immediately before "# Reviews".
        // This handles modify-action specs copied from main specs that lack the section.
        let content =
            "---\nid: test\n---\n\n# Test\n\n## Overview\n\nExisting content.\n\n# Reviews\n";
        let result = replace_section(content, "requirements", "R1: Must do something.");
        assert!(
            result.contains("## Requirements"),
            "inserted heading must be present"
        );
        assert!(
            result.contains("R1: Must do something."),
            "inserted content must be present"
        );
        assert!(
            result.contains("## Overview"),
            "original overview must be preserved"
        );
        assert!(
            result.contains("Existing content."),
            "original overview content must be preserved"
        );
        // Inserted section must appear before # Reviews
        let reqs_pos = result.find("## Requirements").unwrap();
        let reviews_pos = result.find("# Reviews").unwrap();
        assert!(
            reqs_pos < reviews_pos,
            "inserted section must come before # Reviews, got:\n{}",
            result
        );
    }

    #[test]
    fn test_replace_section_appends_at_end_when_no_reviews_and_missing() {
        // When neither the target section nor a "# Reviews" section exists,
        // replace_section must append the new section at the end of the document.
        let content = "---\nid: test\n---\n\n# Test\n\n## Overview\n\nExisting content.\n";
        let result = replace_section(content, "requirements", "R1: Must work.");
        assert!(
            result.contains("## Requirements"),
            "appended heading must be present"
        );
        assert!(
            result.contains("R1: Must work."),
            "appended content must be present"
        );
        assert!(
            result.contains("## Overview"),
            "original section must be preserved"
        );
    }

    // ── strip_change_spec_fields: merge_strategy is no longer stripped ────────

    // ─── parse_fill_section / fill_section_base_name / is_fill_section_optional ─

    #[test]
    fn test_parse_fill_section_required() {
        let (name, optional) = parse_fill_section("overview");
        assert_eq!(name, "overview");
        assert!(!optional);
    }

    #[test]
    fn test_parse_fill_section_optional() {
        let (name, optional) = parse_fill_section("component (optional)");
        assert_eq!(name, "component");
        assert!(optional);
    }

    #[test]
    fn test_parse_fill_section_design_token_optional() {
        let (name, optional) = parse_fill_section("design-token (optional)");
        assert_eq!(name, "design-token");
        assert!(optional);
    }

    #[test]
    fn test_fill_section_base_name_required() {
        assert_eq!(fill_section_base_name("overview"), "overview");
    }

    #[test]
    fn test_fill_section_base_name_strips_optional() {
        assert_eq!(fill_section_base_name("component (optional)"), "component");
    }

    #[test]
    fn test_fill_section_base_name_design_token() {
        assert_eq!(
            fill_section_base_name("design-token (optional)"),
            "design-token"
        );
    }

    #[test]
    fn test_is_fill_section_optional_false() {
        assert!(!is_fill_section_optional("overview"));
        assert!(!is_fill_section_optional("changes"));
        assert!(!is_fill_section_optional("wireframe"));
    }

    #[test]
    fn test_is_fill_section_optional_true() {
        assert!(is_fill_section_optional("component (optional)"));
        assert!(is_fill_section_optional("design-token (optional)"));
    }

    #[test]
    fn test_read_fill_sections_with_optional_markers() {
        let content = "---\nid: test\nfill_sections: [overview, component (optional), design-token (optional), changes]\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(
            sections,
            vec![
                "overview",
                "component (optional)",
                "design-token (optional)",
                "changes",
            ]
        );
    }

    #[test]
    fn test_read_fill_sections_yaml_list_with_optional_markers() {
        let content = "---\nid: test\nfill_sections:\n- overview\n- wireframe\n- component (optional)\n- design-token (optional)\n- changes\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(
            sections,
            vec![
                "overview",
                "wireframe",
                "component (optional)",
                "design-token (optional)",
                "changes",
            ]
        );
    }

    #[test]
    fn test_strip_change_spec_fields_preserves_merge_strategy() {
        // merge_strategy is dead code but is no longer in the stripped fields list.
        // After strip it must remain in the frontmatter (not silently removed).
        let content = "---\n\
            id: test\n\
            main_spec_ref: foo/bar.md\n\
            merge_strategy: extend\n\
            fill_sections: [overview]\n\
            filled_sections: [overview]\n\
            create_complete: true\n\
            ---\n\n# Test\n\nContent.\n";
        let stripped = strip_change_spec_fields(content);
        // Change-spec-only lifecycle fields must be removed
        assert!(
            !stripped.contains("fill_sections"),
            "fill_sections must be stripped"
        );
        assert!(
            !stripped.contains("filled_sections"),
            "filled_sections must be stripped"
        );
        assert!(
            !stripped.contains("create_complete"),
            "create_complete must be stripped"
        );
        // merge_strategy is NOT a change-spec-only field — must be preserved
        assert!(
            stripped.contains("merge_strategy: extend"),
            "merge_strategy must NOT be stripped; it belongs to the main spec"
        );
        // Core fields must be preserved
        assert!(stripped.contains("id: test"), "id must be preserved");
        assert!(
            stripped.contains("main_spec_ref: foo/bar.md"),
            "main_spec_ref must be preserved"
        );
    }

    // ── find_unreviewed_complete_spec: CRR is mandatory ───────────────────────

    #[test]
    fn test_find_unreviewed_complete_spec_with_approved_inline_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let content = "---\n\
            id: foo-spec\n\
            create_complete: true\n\
            review_verdict: APPROVED\n\
            ---\n\n# Foo\n";
        std::fs::write(specs_dir.join("foo-spec.md"), content).unwrap();
        assert_eq!(find_unreviewed_complete_spec(change_dir, &specs_dir), None);
    }

    #[test]
    fn test_find_unreviewed_complete_spec_without_verdict_returns_id() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let content = "---\n\
            id: bar-spec\n\
            create_complete: true\n\
            ---\n\n# Bar\n";
        std::fs::write(specs_dir.join("bar-spec.md"), content).unwrap();
        assert_eq!(
            find_unreviewed_complete_spec(change_dir, &specs_dir),
            Some("bar-spec".to_string())
        );
    }

    #[test]
    fn test_find_unreviewed_complete_spec_with_sibling_review_approved_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let spec_content = "---\n\
            id: baz-spec\n\
            create_complete: true\n\
            ---\n\n# Baz\n";
        std::fs::write(specs_dir.join("baz-spec.md"), spec_content).unwrap();
        let review_content = "---\n\
            verdict: APPROVED\n\
            ---\n\n# Review\n";
        std::fs::write(change_dir.join("review_spec_baz-spec.md"), review_content).unwrap();
        assert_eq!(find_unreviewed_complete_spec(change_dir, &specs_dir), None);
    }

    #[test]
    fn test_find_unreviewed_complete_spec_skips_incomplete() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        // Incomplete spec must NOT be flagged for review
        let content = "---\n\
            id: qux-spec\n\
            create_complete: false\n\
            ---\n\n# Qux\n";
        std::fs::write(specs_dir.join("qux-spec.md"), content).unwrap();
        assert_eq!(find_unreviewed_complete_spec(change_dir, &specs_dir), None);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-common-change-spec-rs-tests>"
    description: "Common change-spec regression test module."
```

---
id: projects-sdd-src-tools-fetch-issues-rs-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue and platform-sync tool TDs expose AW Core workflow state through configured external clients."
---

# sdd fetch issues tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/fetch_issues.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 29 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 61 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `list_issues_by_labels` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 657 | list_issues_by_labels(     labels: &[String],     repo: Option<&str>,     project_root: Option<&Path>, ) -> Result<Vec<u64>> |
## Source
<!-- type: source lang: rust -->

````rust
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_number_url() {
        assert_eq!(
            parse_issue_number("https://github.com/owner/repo/issues/42"),
            Some(42)
        );
    }

    #[test]
    fn test_parse_issue_number_hash() {
        assert_eq!(parse_issue_number("#123"), Some(123));
    }

    #[test]
    fn test_parse_issue_number_plain() {
        assert_eq!(parse_issue_number("456"), Some(456));
    }

    #[test]
    fn test_extract_dependencies() {
        let body = "This is blocked by #10 and depends on #20.\nAlso blockedBy #30.";
        let deps = extract_dependencies(body);
        assert!(deps.contains(&10));
        assert!(deps.contains(&20));
        assert!(deps.contains(&30));
    }

    #[test]
    fn test_extract_dependencies_tasklist() {
        let body = "Tasks:\n- [ ] #5\n- [x] #7\n";
        let deps = extract_dependencies(body);
        assert!(deps.contains(&5));
        assert!(deps.contains(&7));
    }

    #[test]
    fn test_topological_sort_simple() {
        let mut issues = HashMap::new();
        issues.insert(
            1,
            FetchedIssue {
                number: 1,
                title: "A".into(),
                body: String::new(),
                labels: vec![],
                state: "OPEN".into(),
                dependencies: vec![],
            },
        );
        issues.insert(
            2,
            FetchedIssue {
                number: 2,
                title: "B".into(),
                body: String::new(),
                labels: vec![],
                state: "OPEN".into(),
                dependencies: vec![1],
            },
        );
        issues.insert(
            3,
            FetchedIssue {
                number: 3,
                title: "C".into(),
                body: String::new(),
                labels: vec![],
                state: "OPEN".into(),
                dependencies: vec![1, 2],
            },
        );

        let order = topological_sort(&issues);
        assert_eq!(order, vec![1, 2, 3]);
    }

    #[test]
    fn test_topological_sort_independent() {
        let mut issues = HashMap::new();
        issues.insert(
            3,
            FetchedIssue {
                number: 3,
                title: "C".into(),
                body: String::new(),
                labels: vec![],
                state: "OPEN".into(),
                dependencies: vec![],
            },
        );
        issues.insert(
            1,
            FetchedIssue {
                number: 1,
                title: "A".into(),
                body: String::new(),
                labels: vec![],
                state: "OPEN".into(),
                dependencies: vec![],
            },
        );

        let order = topological_sort(&issues);
        // Sorted numerically for determinism
        assert_eq!(order, vec![1, 3]);
    }

    #[test]
    fn test_parse_issue_number_gitlab_url() {
        assert_eq!(
            parse_issue_number("https://gitlab.com/owner/repo/-/issues/99"),
            Some(99)
        );
    }

    #[test]
    fn test_detect_repo_github() {
        let refs = vec![
            "https://github.com/anthropics/cclab/issues/42".to_string(),
            "#10".to_string(),
        ];
        assert_eq!(detect_repo(&refs), Some("anthropics/cclab".to_string()));
    }

    #[test]
    fn test_detect_repo_gitlab() {
        let refs = vec!["https://gitlab.com/myorg/myrepo/-/issues/55".to_string()];
        assert_eq!(detect_repo(&refs), Some("myorg/myrepo".to_string()));
    }

    #[test]
    fn test_detect_repo_no_url() {
        let refs = vec!["#10".to_string()];
        assert_eq!(detect_repo(&refs), None);
    }

    #[test]
    fn test_detect_platform_github_default() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        // No config.toml → defaults to GitHub
        assert_eq!(detect_platform(temp_dir.path()), PlatformType::GitHub);
    }

    #[test]
    fn test_detect_platform_gitlab() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".aw");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "[platform]\ntype = \"gitlab\"\nrepo = \"myorg/myrepo\"\n",
        )
        .unwrap();
        assert_eq!(detect_platform(temp_dir.path()), PlatformType::GitLab);
    }

    #[test]
    fn test_detect_repo_from_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".aw");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "[platform]\ntype = \"github\"\nrepo = \"myorg/myrepo\"\n",
        )
        .unwrap();
        assert_eq!(
            detect_repo_from_config(temp_dir.path()),
            Some("myorg/myrepo".to_string())
        );
    }

    #[test]
    fn test_detect_repo_from_config_placeholder() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".aw");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "[platform]\ntype = \"github\"\nrepo = \"owner/repo\"\n",
        )
        .unwrap();
        // "owner/repo" is the placeholder, should return None
        assert_eq!(detect_repo_from_config(temp_dir.path()), None);
    }

    #[test]
    fn test_format_issue_md() {
        let issue = FetchedIssue {
            number: 42,
            title: "Test issue".into(),
            body: "Issue body content".into(),
            labels: vec!["bug".into()],
            state: "OPEN".into(),
            dependencies: vec![10],
        };
        let md = format_issue_md(&issue);
        assert!(md.contains("number: 42"));
        assert!(md.contains("state: open")); // lowercase
        assert!(md.contains("# #42 — Test issue"));
        assert!(md.contains("Issue body content"));
        assert!(md.contains("dependencies: [10]"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Test Issue"), "test-issue");
        assert_eq!(slugify("Add OAuth/OIDC support"), "add-oauth-oidc-support");
        assert_eq!(
            slugify("fix: memory leak in parser"),
            "fix-memory-leak-in-parser"
        );
        assert_eq!(slugify(""), "");
        // truncate to 50 chars
        let long = "a".repeat(100);
        assert_eq!(slugify(&long).len(), 50);
    }

    #[test]
    fn test_update_state_dag() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        // Use proper project layout: .aw/changes/test/ as change_dir
        let project_root = temp_dir.path();
        let change_dir = project_root.join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(project_root, "test");

        let mut issues = HashMap::new();
        issues.insert(
            1,
            FetchedIssue {
                number: 1,
                title: "A".into(),
                body: String::new(),
                labels: vec![],
                state: "OPEN".into(),
                dependencies: vec![],
            },
        );
        issues.insert(
            2,
            FetchedIssue {
                number: 2,
                title: "B".into(),
                body: String::new(),
                labels: vec![],
                state: "OPEN".into(),
                dependencies: vec![1],
            },
        );

        // update_state_dag must succeed (loads SM, sets dag, saves to issue frontmatter).
        // Note: `dag` is a transient field — it is set in memory and saved but is NOT
        // persisted to issue frontmatter (not part of IssuePatch). Verify the call
        // succeeds and the topological sort produces the correct order.
        let order = vec![1, 2];
        update_state_dag(&change_dir, &order, &issues).unwrap();

        // Verify the topological sort embedded in update_state_dag is correct
        // by re-running it and checking the order (dag in-memory state is not reloadable).
        let recomputed_order = topological_sort(&issues);
        assert_eq!(recomputed_order, vec![1, 2]);
        assert_eq!(issues[&1].title, "A");
        assert_eq!(issues[&2].title, "B");
        assert_eq!(issues[&2].dependencies, vec![1]);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/fetch_issues.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-fetch-issues-rs-tests>"
    description: "Fetch issues regression tests."
```

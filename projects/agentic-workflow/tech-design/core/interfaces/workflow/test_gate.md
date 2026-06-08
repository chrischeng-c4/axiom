---
id: sdd-workflow-test-gate
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow state-machine interfaces drive TD/CB lifecycle transitions, review loops, merge, and validation gates."
---

# TestGateResult Type

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/workflow/test_gate.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TestGateResult` | projects/agentic-workflow/src/workflow/test_gate.rs | struct | pub | 20 |  |
| `check_requirement_coverage` | projects/agentic-workflow/src/workflow/test_gate.rs | function | pub | 86 | check_requirement_coverage(     spec_reqs: &[String],     test_markers: &HashSet<String>, ) -> std::result::Result<(), Vec<String>> |
| `get_changed_files` | projects/agentic-workflow/src/workflow/test_gate.rs | function | pub | 204 | get_changed_files(project_root: &Path) -> Vec<String> |
| `match_scopes` | projects/agentic-workflow/src/workflow/test_gate.rs | function | pub | 110 | match_scopes(changed_files: &[String], config: &'a TestConfig) -> Vec<&'a TestScope> |
| `parse_requirement_ids` | projects/agentic-workflow/src/workflow/test_gate.rs | function | pub | 34 | parse_requirement_ids(spec_content: &str) -> Vec<String> |
| `run_full_test_gate` | projects/agentic-workflow/src/workflow/test_gate.rs | function | pub | 227 | run_full_test_gate(change_dir: &Path, project_root: &Path) -> Result<TestGateResult> |
| `run_test_gate` | projects/agentic-workflow/src/workflow/test_gate.rs | function | pub | 133 | run_test_gate(     scope: &TestScope,     global: &TestConfig,     project_root: &Path, ) -> Result<String> |
| `scan_test_markers` | projects/agentic-workflow/src/workflow/test_gate.rs | function | pub | 68 | scan_test_markers(test_files: &[PathBuf]) -> HashSet<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TestGateResult:
    type: object
    required: [passed, messages, skipped]
    description: Result of running the test gate.
    properties:
      passed:
        type: boolean
        description: "Whether the gate passed."
      messages:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Human-readable summary messages."
      skipped:
        type: boolean
        description: "Whether gate was skipped (no config or no matching scopes)."
    x-rust-struct:
      derive: [Debug]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/workflow/test_gate.rs -->
~~~rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/test_gate.md#source
// CODEGEN-BEGIN
//! TDD test gate logic.
//!
//! Two gates that run between ChangeImplementationReviewed and DocsCheck:
//! - Gate 1: Requirement coverage — spec requirementDiagram IDs vs test file REQ markers
//! - Gate 2: Test execution — match changed files against scope glob patterns, run test commands

use crate::models::change::{SddConfig, TestConfig, TestScope};
use crate::Result;
use globset::{Glob, GlobSetBuilder};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of running the test gate.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/test_gate.md#schema
#[derive(Debug)]
pub struct TestGateResult {
    /// Whether the gate passed.
    pub passed: bool,
    /// Human-readable summary messages.
    pub messages: Vec<String>,
    /// Whether gate was skipped (no config or no matching scopes).
    pub skipped: bool,
}
// ---------------------------------------------------------------------------
// Gate 1: Requirement coverage check
// ---------------------------------------------------------------------------

/// Parse requirement IDs from Mermaid `requirementDiagram` blocks in spec content.
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R5
pub fn parse_requirement_ids(spec_content: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let re = Regex::new(r"(?m)^\s*requirement\s+.*?\{\s*\n\s*id:\s*(REQ-\w+)").unwrap();

    // Also match simple REQ-{id} patterns within requirementDiagram blocks
    let block_re = Regex::new(r"(?s)```mermaid\s*\n\s*requirementDiagram(.*?)```").unwrap();
    let id_re = Regex::new(r"(REQ-\w+)").unwrap();

    for block_cap in block_re.captures_iter(spec_content) {
        let block_content = &block_cap[1];
        for id_cap in id_re.captures_iter(block_content) {
            let id = id_cap[1].to_string();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    // Fallback: also match top-level requirement IDs outside code blocks
    // (for specs that list REQ-xxx in tables or prose)
    if ids.is_empty() {
        for cap in re.captures_iter(spec_content) {
            let id = cap[1].to_string();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    ids
}

/// Scan test files for `REQ: REQ-{id}` comment markers.
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R5
pub fn scan_test_markers(test_files: &[PathBuf]) -> HashSet<String> {
    let re = Regex::new(r"REQ:\s*(REQ-\w+)").unwrap();
    let mut markers = HashSet::new();

    for path in test_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            for cap in re.captures_iter(&content) {
                markers.insert(cap[1].to_string());
            }
        }
    }

    markers
}

/// Check that all spec requirement IDs are covered by test markers.
/// Returns `Err` with the list of uncovered requirement IDs.
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R5
pub fn check_requirement_coverage(
    spec_reqs: &[String],
    test_markers: &HashSet<String>,
) -> std::result::Result<(), Vec<String>> {
    let missing: Vec<String> = spec_reqs
        .iter()
        .filter(|req| !test_markers.contains(req.as_str()))
        .cloned()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

// ---------------------------------------------------------------------------
// Gate 2: Test execution
// ---------------------------------------------------------------------------

/// Match changed files against scope `changes` glob patterns.
/// Returns references to scopes whose patterns match at least one changed file.
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R6
pub fn match_scopes<'a>(changed_files: &[String], config: &'a TestConfig) -> Vec<&'a TestScope> {
    config
        .scope
        .iter()
        .filter(|scope| {
            let mut builder = GlobSetBuilder::new();
            for pattern in &scope.changes {
                if let Ok(glob) = Glob::new(pattern) {
                    builder.add(glob);
                }
            }
            if let Ok(set) = builder.build() {
                changed_files.iter().any(|f| set.is_match(f))
            } else {
                false
            }
        })
        .collect()
}

/// Run setup/test/teardown for a single matched scope.
/// Uses the scope's own commands, falling back to global defaults.
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R6
pub fn run_test_gate(
    scope: &TestScope,
    global: &TestConfig,
    project_root: &Path,
) -> Result<String> {
    let mut output_log = String::new();

    // Setup: scope override > global default
    let setup_cmd = scope.setup.as_deref().or(global.setup.as_deref());
    if let Some(cmd) = setup_cmd {
        output_log.push_str(&format!("[{}] Running setup: {}\n", scope.name, cmd));
        run_shell_command(cmd, project_root)?;
    }

    // Test command: scope override > global default
    let test_cmd = scope.test_cmd.as_deref().or(global.test_cmd.as_deref());

    let test_result = if let Some(cmd) = test_cmd {
        output_log.push_str(&format!("[{}] Running test: {}\n", scope.name, cmd));
        run_shell_command(cmd, project_root)
    } else {
        output_log.push_str(&format!(
            "[{}] WARNING: No test_cmd configured (scope or global)\n",
            scope.name
        ));
        Ok(())
    };

    // Teardown: scope override > global default (always runs, even on test failure)
    let teardown_cmd = scope.teardown.as_deref().or(global.teardown.as_deref());
    if let Some(cmd) = teardown_cmd {
        output_log.push_str(&format!("[{}] Running teardown: {}\n", scope.name, cmd));
        let _ = run_shell_command(cmd, project_root); // Don't fail on teardown
    }

    // Propagate test failure
    test_result?;

    Ok(output_log)
}

/// Execute a shell command and return error on non-zero exit.
fn run_shell_command(cmd: &str, working_dir: &Path) -> Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(working_dir)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "Command '{}' exited with code {}.\nstdout:\n{}\nstderr:\n{}",
            cmd,
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Orchestrator: run full test gate
// ---------------------------------------------------------------------------

/// Get changed files for a change by reading git diff.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/test_gate.md#source
pub fn get_changed_files(project_root: &Path) -> Vec<String> {
    // Try git diff against main/HEAD
    let output = Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .current_dir(project_root)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Run the full test gate for a change.
///
/// 1. Load config — skip if no `[agentic_workflow.test]` section
/// 2. Match changed files against scopes — skip if no matches
/// 3. Gate 1: Requirement coverage (if specs have requirementDiagram)
/// 4. Gate 2: Execute test commands for matched scopes
// @spec projects/agentic-workflow/tech-design/core/logic/tdd-gate.md#R4
pub fn run_full_test_gate(change_dir: &Path, project_root: &Path) -> Result<TestGateResult> {
    let config = SddConfig::load(project_root)?;

    // Skip if no [agentic_workflow.test] config
    let test_config = match config.test {
        Some(ref tc) => tc,
        None => {
            return Ok(TestGateResult {
                passed: true,
                messages: vec!["TestCheck: skipped (no [agentic_workflow.test] config)".to_string()],
                skipped: true,
            });
        }
    };

    // Get changed files
    let changed_files = get_changed_files(project_root);

    // Match scopes
    let matched_scopes = match_scopes(&changed_files, test_config);
    if matched_scopes.is_empty() {
        return Ok(TestGateResult {
            passed: true,
            messages: vec!["TestCheck: skipped (no changed files match any test scope)".to_string()],
            skipped: true,
        });
    }

    let mut messages = Vec::new();
    messages.push(format!(
        "TestCheck: {} scope(s) matched: {}",
        matched_scopes.len(),
        matched_scopes
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    // Gate 1: Requirement coverage check
    let specs_dir = change_dir.join("specs");
    if specs_dir.exists() {
        let mut all_req_ids = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|e| e == "md") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        all_req_ids.extend(parse_requirement_ids(&content));
                    }
                }
            }
        }

        if !all_req_ids.is_empty() {
            // Scan all source files in the change directory for test markers
            let mut test_files = Vec::new();
            collect_source_files(project_root, &changed_files, &mut test_files);

            let test_markers = scan_test_markers(&test_files);

            match check_requirement_coverage(&all_req_ids, &test_markers) {
                Ok(()) => {
                    messages.push(format!(
                        "Gate 1 PASS: all {} requirement(s) covered by test markers",
                        all_req_ids.len()
                    ));
                }
                Err(missing) => {
                    messages.push(format!(
                        "Gate 1 FAIL: {} uncovered requirement(s): {}",
                        missing.len(),
                        missing.join(", ")
                    ));
                    return Ok(TestGateResult {
                        passed: false,
                        messages,
                        skipped: false,
                    });
                }
            }
        } else {
            messages.push("Gate 1: skipped (no requirementDiagram in specs)".to_string());
        }
    }

    // Gate 2: Test execution
    for scope in &matched_scopes {
        match run_test_gate(scope, test_config, project_root) {
            Ok(log) => {
                messages.push(format!("Gate 2 PASS [{}]: tests passed", scope.name));
                if !log.is_empty() {
                    messages.push(log);
                }
            }
            Err(e) => {
                messages.push(format!("Gate 2 FAIL [{}]: {}", scope.name, e));
                return Ok(TestGateResult {
                    passed: false,
                    messages,
                    skipped: false,
                });
            }
        }
    }

    Ok(TestGateResult {
        passed: true,
        messages,
        skipped: false,
    })
}

/// Collect source files from changed file paths.
fn collect_source_files(project_root: &Path, changed_files: &[String], out: &mut Vec<PathBuf>) {
    for file in changed_files {
        let path = project_root.join(file);
        if path.exists() && path.is_file() {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirement_ids_from_requirement_diagram() {
        let spec = r#"
## Requirements

```mermaid
requirementDiagram

    requirement "TestConfig" {
        id: REQ-001
        text: Add TestConfig struct
        risk: low
    }

    requirement "TOML parsing" {
        id: REQ-002
        text: Parse config
        risk: low
    }
```
"#;
        let ids = parse_requirement_ids(spec);
        assert_eq!(ids, vec!["REQ-001".to_string(), "REQ-002".to_string()]);
    }

    #[test]
    fn test_parse_requirement_ids_empty() {
        let spec = "## Overview\n\nNo requirements here.";
        let ids = parse_requirement_ids(spec);
        assert!(ids.is_empty());
    }

    #[test]
    fn test_scan_test_markers() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test_foo.rs");
        std::fs::write(&test_file, "// REQ: REQ-001\n// REQ: REQ-002\nfn test() {}").unwrap();

        let markers = scan_test_markers(&[test_file]);
        assert!(markers.contains("REQ-001"));
        assert!(markers.contains("REQ-002"));
        assert_eq!(markers.len(), 2);
    }

    #[test]
    fn test_check_requirement_coverage_pass() {
        let reqs = vec!["REQ-001".to_string(), "REQ-002".to_string()];
        let mut markers = HashSet::new();
        markers.insert("REQ-001".to_string());
        markers.insert("REQ-002".to_string());
        assert!(check_requirement_coverage(&reqs, &markers).is_ok());
    }

    #[test]
    fn test_check_requirement_coverage_fail() {
        let reqs = vec!["REQ-001".to_string(), "REQ-002".to_string()];
        let mut markers = HashSet::new();
        markers.insert("REQ-001".to_string());
        let result = check_requirement_coverage(&reqs, &markers);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing, vec!["REQ-002".to_string()]);
    }

    #[test]
    fn test_match_scopes() {
        let config = TestConfig {
            test_cmd: Some("cargo test".to_string()),
            setup: None,
            teardown: None,
            scope: vec![
                TestScope {
                    name: "conductor".to_string(),
                    changes: vec!["projects/conductor/**".to_string()],
                    test_cmd: None,
                    setup: None,
                    teardown: None,
                },
                TestScope {
                    name: "sdd".to_string(),
                    changes: vec!["projects/agentic-workflow/**".to_string()],
                    test_cmd: None,
                    setup: None,
                    teardown: None,
                },
            ],
        };

        // Match conductor
        let changed = vec!["projects/conductor/fe/src/App.tsx".to_string()];
        let matched = match_scopes(&changed, &config);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "conductor");

        // Match sdd
        let changed = vec!["projects/agentic-workflow/src/models/change.rs".to_string()];
        let matched = match_scopes(&changed, &config);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "sdd");

        // No match
        let changed = vec!["docs/readme.md".to_string()];
        let matched = match_scopes(&changed, &config);
        assert!(matched.is_empty());
    }

    #[test]
    fn test_match_scopes_multiple() {
        let config = TestConfig {
            test_cmd: None,
            setup: None,
            teardown: None,
            scope: vec![
                TestScope {
                    name: "all-rs".to_string(),
                    changes: vec!["projects/**".to_string()],
                    test_cmd: None,
                    setup: None,
                    teardown: None,
                },
                TestScope {
                    name: "sdd".to_string(),
                    changes: vec!["projects/agentic-workflow/**".to_string()],
                    test_cmd: None,
                    setup: None,
                    teardown: None,
                },
            ],
        };

        let changed = vec!["projects/agentic-workflow/src/lib.rs".to_string()];
        let matched = match_scopes(&changed, &config);
        assert_eq!(matched.len(), 2);
    }
}

// CODEGEN-END
~~~

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/workflow/test_gate.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete workflow test gate module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single struct with Debug only.
- [schema] All in `required:`; standard pattern.
- [changes] Standard split.

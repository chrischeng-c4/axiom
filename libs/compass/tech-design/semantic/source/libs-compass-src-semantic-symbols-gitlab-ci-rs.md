---
id: libs-compass-src-semantic-symbols-gitlab-ci-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/semantic/symbols/gitlab_ci.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/semantic/symbols/gitlab_ci.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/semantic/symbols/gitlab_ci.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `visit_gitlab_ci_lines` | libs/compass/src/semantic/symbols/gitlab_ci.rs | function | pub | 24 | pub(crate) fn visit_gitlab_ci_lines(&mut self, source: &str) { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! GitLab CI YAML symbol extraction (line-based)
//!
//! Extracts: jobs, stages, variables, templates from `.gitlab-ci.yml`.

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::{Position, Range};

/// Reserved top-level keywords that are NOT jobs
const RESERVED: &[&str] = &[
    "stages",
    "variables",
    "include",
    "image",
    "services",
    "before_script",
    "after_script",
    "cache",
    "default",
    "workflow",
    "pages",
];

impl SymbolTableBuilder {
    pub(crate) fn visit_gitlab_ci_lines(&mut self, source: &str) {
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Top-level key (no indent, has colon)
            if !line.starts_with(' ') && !line.starts_with('\t') {
                if let Some(key) = yaml_key(trimmed) {
                    let ln = i as u32;
                    match key.as_str() {
                        "stages" => {
                            i += 1;
                            i = self.collect_stages(&lines, i);
                            continue;
                        }
                        "variables" => {
                            i += 1;
                            i = self.collect_vars(&lines, i, "global");
                            continue;
                        }
                        _ if !RESERVED.contains(&key.as_str()) => {
                            let kind = if key.starts_with('.') {
                                SymbolKind::Template
                            } else {
                                SymbolKind::Job
                            };
                            let col = line.find(&key).unwrap_or(0) as u32;
                            self.table.add_symbol(
                                key.clone(),
                                kind,
                                mk_range(ln, col, key.len()),
                                None,
                                None,
                                self.current_scope,
                            );
                            i += 1;
                            i = self.scan_job_body(&lines, i, &key);
                            continue;
                        }
                        _ => {}
                    }
                }
            }
            i += 1;
        }
    }

    fn collect_stages(&mut self, lines: &[&str], mut i: usize) -> usize {
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
                break;
            }
            if let Some(item) = trimmed.strip_prefix("- ") {
                let name = item.trim().to_string();
                if !name.is_empty() {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.clone(),
                        SymbolKind::Stage,
                        mk_range(i as u32, col, name.len()),
                        None,
                        Some("stage".into()),
                        self.current_scope,
                    );
                }
            }
            i += 1;
        }
        i
    }

    fn collect_vars(&mut self, lines: &[&str], mut i: usize, ctx: &str) -> usize {
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
                break;
            }
            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }
            if !trimmed.starts_with('-') {
                if let Some(k) = yaml_key(trimmed) {
                    let col = line.find(&k).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        k.clone(),
                        SymbolKind::Variable,
                        mk_range(i as u32, col, k.len()),
                        None,
                        Some(format!("{} variable", ctx)),
                        self.current_scope,
                    );
                }
            }
            i += 1;
        }
        i
    }

    fn scan_job_body(&mut self, lines: &[&str], mut i: usize, job: &str) -> usize {
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
                break;
            }
            if trimmed == "variables:" {
                i += 1;
                i = self.collect_job_vars(lines, i, job);
                continue;
            }
            i += 1;
        }
        i
    }

    fn collect_job_vars(&mut self, lines: &[&str], mut i: usize, job: &str) -> usize {
        let base = if i < lines.len() {
            indent(lines[i])
        } else {
            return i;
        };
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }
            if indent(line) < base {
                break;
            }
            if let Some(k) = yaml_key(trimmed) {
                let col = line.find(&k).unwrap_or(0) as u32;
                self.table.add_symbol(
                    k.clone(),
                    SymbolKind::Variable,
                    mk_range(i as u32, col, k.len()),
                    None,
                    Some(format!("{} variable", job)),
                    self.current_scope,
                );
            }
            i += 1;
        }
        i
    }
}

fn yaml_key(line: &str) -> Option<String> {
    let pos = line.find(':')?;
    let key = line[..pos].trim();
    if key.is_empty() || key.starts_with('-') || key.starts_with('#') {
        return None;
    }
    Some(key.trim_matches('"').trim_matches('\'').to_string())
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn mk_range(line: u32, col: u32, len: usize) -> Range {
    Range::new(
        Position::new(line, col),
        Position::new(line, col + len as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};

    fn build(source: &str) -> super::super::SymbolTable {
        let mut b = SymbolTableBuilder::new();
        b.visit_gitlab_ci_lines(source);
        b.table
    }

    #[test]
    fn test_stages() {
        let t = build("stages:\n  - build\n  - test\n  - deploy\n");
        let names: Vec<&str> = t
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Stage)
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(names, vec!["build", "test", "deploy"]);
    }

    #[test]
    fn test_jobs_and_templates() {
        let t = build(
            ".base:\n  image: alpine\nbuild_job:\n  stage: build\ntest_job:\n  stage: test\n",
        );
        let jobs: Vec<&str> = t
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Job)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            jobs.contains(&"build_job") && jobs.contains(&"test_job"),
            "got: {:?}",
            jobs
        );
        let tpls: Vec<&str> = t
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Template)
            .map(|s| s.name.as_str())
            .collect();
        assert!(tpls.contains(&".base"), "got: {:?}", tpls);
    }

    #[test]
    fn test_variables() {
        let t = build("variables:\n  GLOBAL_VAR: value\nbuild:\n  variables:\n    BUILD_VAR: val\n  script: echo\n");
        let vars: Vec<&str> = t
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            vars.contains(&"GLOBAL_VAR") && vars.contains(&"BUILD_VAR"),
            "got: {:?}",
            vars
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/semantic/symbols/gitlab_ci.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/semantic/symbols/gitlab_ci.rs` captured during libs codegen standardization.
```

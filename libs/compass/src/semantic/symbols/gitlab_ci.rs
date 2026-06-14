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

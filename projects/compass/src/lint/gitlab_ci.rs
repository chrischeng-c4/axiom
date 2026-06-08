//! GitLab CI lint checker (source-line analysis on YAML)

use super::gitlab_ci_rules;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};
use std::collections::HashSet;

/// GitLab CI checker — line-based YAML analysis
pub struct GitlabCiChecker;

/// Represents a parsed CI job extracted from YAML source
#[derive(Debug)]
pub(super) struct CiJob {
    pub(super) name: String,
    pub(super) start_line: usize,
    pub(super) end_line: usize,
    pub(super) stage: Option<String>,
    pub(super) has_script: bool,
    pub(super) has_rules: bool,
    pub(super) has_only: bool,
    pub(super) has_except: bool,
    pub(super) has_timeout: bool,
    pub(super) has_when_manual: bool,
    pub(super) has_allow_failure: bool,
    pub(super) extends: Option<String>,
    pub(super) needs: Vec<String>,
    pub(super) keywords: Vec<String>,
    pub(super) variable_lines: Vec<(usize, String)>,
}

impl GitlabCiChecker {
    pub fn new() -> Self {
        Self
    }

    /// GL001: Syntax errors from tree-sitter YAML parse
    fn check_syntax_errors(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "GL001",
                        DiagnosticCategory::Syntax,
                        "YAML syntax error in CI configuration",
                    ));
                }
                true
            });
        }
        diagnostics
    }

    /// Parse the YAML source into stages list, job definitions, and template names
    pub(super) fn parse_ci_structure(lines: &[&str]) -> (Vec<String>, Vec<CiJob>, Vec<String>) {
        let mut stages: Vec<String> = Vec::new();
        let mut jobs: Vec<CiJob> = Vec::new();
        let mut templates: Vec<String> = Vec::new();
        let mut current_job: Option<CiJob> = None;
        let mut in_stages_block = false;
        let mut in_variables_block = false;
        let mut in_needs_block = false;
        let mut job_indent: usize = 0;

        let reserved_keys: HashSet<&str> = [
            "stages",
            "variables",
            "default",
            "include",
            "image",
            "services",
            "before_script",
            "after_script",
            "cache",
            "workflow",
            "pages",
        ]
        .into_iter()
        .collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indent = line.len() - line.trim_start().len();

            if indent == 0 && !trimmed.starts_with('-') {
                if let Some(mut job) = current_job.take() {
                    job.end_line = line_num.saturating_sub(1);
                    jobs.push(job);
                }
                in_stages_block = false;
                in_variables_block = false;

                if let Some(key) = trimmed.strip_suffix(':') {
                    let key = key.trim();
                    if key == "stages" {
                        in_stages_block = true;
                    } else if key.starts_with('.') {
                        templates.push(key.to_string());
                    } else if !reserved_keys.contains(key) {
                        current_job = Some(CiJob {
                            name: key.to_string(),
                            start_line: line_num,
                            end_line: line_num,
                            stage: None,
                            has_script: false,
                            has_rules: false,
                            has_only: false,
                            has_except: false,
                            has_timeout: false,
                            has_when_manual: false,
                            has_allow_failure: false,
                            extends: None,
                            needs: Vec::new(),
                            keywords: Vec::new(),
                            variable_lines: Vec::new(),
                        });
                        job_indent = 0;
                    }
                }
                continue;
            }

            if in_stages_block && indent > 0 {
                if let Some(stage_name) = trimmed.strip_prefix("- ") {
                    stages.push(
                        stage_name
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                }
                continue;
            }

            if let Some(ref mut job) = current_job {
                if indent > job_indent {
                    if in_needs_block && trimmed.starts_with("- ") {
                        let need_name = trimmed
                            .strip_prefix("- ")
                            .unwrap_or("")
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'');
                        if !need_name.is_empty() && !need_name.contains(':') {
                            job.needs.push(need_name.to_string());
                        }
                        continue;
                    }

                    if let Some(key) = Self::extract_yaml_key(trimmed) {
                        job.keywords.push(key.to_string());
                        in_needs_block = false;
                        match key {
                            "script" | "trigger" => job.has_script = true,
                            "rules" => job.has_rules = true,
                            "only" => job.has_only = true,
                            "except" => job.has_except = true,
                            "timeout" => job.has_timeout = true,
                            "allow_failure" => job.has_allow_failure = true,
                            "needs" => {
                                in_needs_block = true;
                                if let Some(val) = Self::extract_yaml_value(trimmed) {
                                    let val = val.trim_matches('[').trim_matches(']');
                                    for item in val.split(',') {
                                        let n = item.trim().trim_matches('"').trim_matches('\'');
                                        if !n.is_empty() {
                                            job.needs.push(n.to_string());
                                        }
                                    }
                                }
                            }
                            "when" => {
                                if let Some(val) = Self::extract_yaml_value(trimmed) {
                                    if val == "manual" {
                                        job.has_when_manual = true;
                                    }
                                }
                            }
                            "extends" => {
                                if let Some(val) = Self::extract_yaml_value(trimmed) {
                                    job.extends = Some(val);
                                }
                            }
                            "stage" => {
                                if let Some(val) = Self::extract_yaml_value(trimmed) {
                                    job.stage = Some(val);
                                }
                            }
                            "variables" => {
                                in_variables_block = true;
                            }
                            _ => {
                                if in_variables_block && indent > 2 {
                                    job.variable_lines.push((line_num, trimmed.to_string()));
                                } else {
                                    in_variables_block = false;
                                }
                            }
                        }

                        if in_variables_block && key != "variables" && indent > 2 {
                            job.variable_lines.push((line_num, trimmed.to_string()));
                        }
                    }
                }
            }
        }

        if let Some(mut job) = current_job {
            job.end_line = lines.len().saturating_sub(1);
            jobs.push(job);
        }

        (stages, jobs, templates)
    }

    /// GL003: Invalid stage reference
    fn check_invalid_stages(stages: &[String], jobs: &[CiJob], lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if stages.is_empty() {
            return diagnostics;
        }

        let valid_stages: HashSet<&str> = stages.iter().map(|s| s.as_str()).collect();
        for job in jobs {
            if let Some(ref stage) = job.stage {
                if !valid_stages.contains(stage.as_str()) {
                    let line = job.start_line;
                    let col = lines.get(line).map(|l| l.len()).unwrap_or(0);
                    diagnostics.push(Diagnostic::error(
                        Range::new(
                            Position::new(line as u32, 0),
                            Position::new(line as u32, col as u32),
                        ),
                        "GL003",
                        DiagnosticCategory::Logic,
                        format!(
                            "Job '{}' references undefined stage '{}' — define it in 'stages:'",
                            job.name, stage,
                        ),
                    ));
                }
            }
        }
        diagnostics
    }

    /// GL004: Missing script in job
    fn check_missing_script(jobs: &[CiJob], lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for job in jobs {
            if !job.has_script {
                let col = lines.get(job.start_line).map(|l| l.len()).unwrap_or(0);
                diagnostics.push(Diagnostic::error(
                    Range::new(
                        Position::new(job.start_line as u32, 0),
                        Position::new(job.start_line as u32, col as u32),
                    ),
                    "GL004",
                    DiagnosticCategory::Logic,
                    format!("Job '{}' is missing a 'script' key", job.name),
                ));
            }
        }
        diagnostics
    }

    /// GL007: Mixing `rules` with `only`/`except`
    fn check_rules_only_mixed(jobs: &[CiJob], lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for job in jobs {
            if job.has_rules && (job.has_only || job.has_except) {
                let col = lines.get(job.start_line).map(|l| l.len()).unwrap_or(0);
                diagnostics.push(Diagnostic::warning(
                    Range::new(
                        Position::new(job.start_line as u32, 0),
                        Position::new(job.start_line as u32, col as u32),
                    ),
                    "GL007",
                    DiagnosticCategory::Logic,
                    format!(
                        "Job '{}' mixes 'rules' with 'only/except' — use 'rules' exclusively",
                        job.name,
                    ),
                ));
            }
        }
        diagnostics
    }

    /// GL008: Hardcoded secrets in variables
    fn check_hardcoded_secrets(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        const SECRET_KEYWORDS: &[&str] = &[
            "PASSWORD",
            "SECRET",
            "TOKEN",
            "API_KEY",
            "APIKEY",
            "PRIVATE_KEY",
            "ACCESS_KEY",
            "CREDENTIAL",
        ];

        let mut in_variables = false;
        let mut variables_indent: Option<usize> = None;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();

            if trimmed == "variables:" {
                in_variables = true;
                variables_indent = Some(indent);
                continue;
            }

            if in_variables {
                if let Some(vi) = variables_indent {
                    if indent <= vi && !trimmed.is_empty() {
                        in_variables = false;
                        variables_indent = None;
                    } else {
                        let upper = trimmed.to_uppercase();
                        for keyword in SECRET_KEYWORDS {
                            if upper.contains(keyword) {
                                if let Some(val) = Self::extract_yaml_value(trimmed) {
                                    let val_trimmed = val.trim();
                                    if !val_trimmed.is_empty()
                                        && !val_trimmed.starts_with('$')
                                        && !val_trimmed.starts_with("${")
                                    {
                                        diagnostics.push(Diagnostic::new(
                                            Range::new(
                                                Position::new(line_num as u32, 0),
                                                Position::new(line_num as u32, line.len() as u32),
                                            ),
                                            DiagnosticSeverity::Error,
                                            "GL008",
                                            DiagnosticCategory::Security,
                                            format!(
                                                "Hardcoded secret in CI variable — '{}' should use CI/CD masked variables",
                                                keyword,
                                            ),
                                        ));
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }

    // ===== Helpers =====

    /// Extract the key from a YAML line like "key: value" -> "key"
    pub(super) fn extract_yaml_key(line: &str) -> Option<&str> {
        let trimmed = line.trim().trim_start_matches("- ");
        let colon_pos = trimmed.find(':')?;
        let key = trimmed[..colon_pos].trim();
        if key.is_empty() {
            None
        } else {
            Some(key)
        }
    }

    /// Extract the value from a YAML line like "key: value" -> "value"
    pub(super) fn extract_yaml_value(line: &str) -> Option<String> {
        let trimmed = line.trim();
        let colon_pos = trimmed.find(':')?;
        let value = trimmed[colon_pos + 1..].trim();
        let value = value.trim_matches('"').trim_matches('\'');
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    /// Quick check: does this look like a GitLab CI file?
    pub(super) fn is_gitlab_ci(source: &str) -> bool {
        let has_stages = source.contains("stages:");
        let has_script = source.contains("script:");
        let has_job_stage = source.contains("stage:");
        has_stages || (has_script && has_job_stage)
    }
}

impl Default for GitlabCiChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for GitlabCiChecker {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        if !Self::is_gitlab_ci(&file.source) {
            return Vec::new();
        }

        let lines: Vec<&str> = file.source.lines().collect();
        let mut diagnostics = Vec::new();

        diagnostics.extend(self.check_syntax_errors(file));

        let (stages, jobs, templates) = Self::parse_ci_structure(&lines);

        diagnostics.extend(Self::check_invalid_stages(&stages, &jobs, &lines));
        diagnostics.extend(Self::check_missing_script(&jobs, &lines));
        diagnostics.extend(Self::check_rules_only_mixed(&jobs, &lines));
        diagnostics.extend(Self::check_hardcoded_secrets(&lines));
        // Delegated rules (gitlab_ci_rules module)
        diagnostics.extend(gitlab_ci_rules::check_unknown_keywords(&jobs, &lines));
        diagnostics.extend(gitlab_ci_rules::check_needs_references(&jobs, &lines));
        diagnostics.extend(gitlab_ci_rules::check_circular_needs(&jobs, &lines));
        diagnostics.extend(gitlab_ci_rules::check_missing_timeout(&jobs, &lines));
        diagnostics.extend(gitlab_ci_rules::check_allow_failure_without_manual(
            &jobs, &lines,
        ));
        diagnostics.extend(gitlab_ci_rules::check_unused_templates(
            &jobs, &templates, &lines,
        ));
        diagnostics.extend(gitlab_ci_rules::check_invalid_includes(&lines));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "GL001", // Syntax errors
            "GL003", // Invalid stage reference
            "GL004", // Missing script in job
            "GL007", // rules + only/except mixed
            "GL008", // Hardcoded secrets in variables
            "GL002", // Unknown job keywords
            "GL005", // needs referencing non-existent job
            "GL006", // Circular needs dependencies
            "GL009", // Missing timeout on jobs
            "GL010", // allow_failure without when: manual
            "GL011", // Unused extends templates
            "GL012", // Invalid include references
        ]
    }
}

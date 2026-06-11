//! Output formatters for lint results

use std::path::{Path, PathBuf};

use crate::checker::FileResult;
use crate::diagnostic::DiagnosticSeverity;
use crate::graph::ImportGraph;
use crate::semantic::symbols::SymbolTable;
use serde::Serialize;

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Markdown,
    Console,
    /// SARIF 2.1.0 for code scanning dashboards (--format sarif)
    Sarif,
    /// GitHub Actions inline annotations (--format github)
    GitHub,
    /// GitLab Code Quality JSON array (--format gitlab)
    GitLab,
    /// Symbol-centric JSON optimized for LLM agent consumption (--format agent)
    Agent,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "markdown" | "md" => Some(OutputFormat::Markdown),
            "console" | "text" | "default" => Some(OutputFormat::Console),
            "sarif" => Some(OutputFormat::Sarif),
            "github" => Some(OutputFormat::GitHub),
            "gitlab" => Some(OutputFormat::GitLab),
            "agent" => Some(OutputFormat::Agent),
            _ => None,
        }
    }
}

/// Reporter for generating output
pub struct Reporter {
    format: OutputFormat,
}

impl Reporter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn generate(&self, results: &[FileResult]) -> String {
        match self.format {
            OutputFormat::Json => self.generate_json(results),
            OutputFormat::Markdown => self.generate_markdown(results),
            OutputFormat::Console => self.generate_console(results),
            OutputFormat::Sarif => self.generate_sarif(results),
            OutputFormat::GitHub => self.generate_github(results),
            OutputFormat::GitLab => self.generate_gitlab(results),
            OutputFormat::Agent => {
                // Agent format requires additional data (SymbolTable, ImportGraph)
                // that cannot be passed through the standard generate() signature.
                // Callers must use Reporter::generate_agent() instead, which accepts
                // the extra parameters needed for symbol-centric output.
                eprintln!(
                    "warning: Agent format requires SymbolTable and ImportGraph data. \
                     Use Reporter::generate_agent() for agent output."
                );
                r#"{"error":"Agent format requires SymbolTable and ImportGraph. Use Reporter::generate_agent()."}"#.to_string()
            }
        }
    }

    /// Generate agent-format output with additional SymbolTable and ImportGraph data.
    ///
    /// Agent format requires per-file symbol tables and a project-wide import graph
    /// in addition to the lint results. This method delegates to `AgentOutputBuilder`
    /// to produce symbol-centric JSON optimized for LLM agent consumption.
    ///
    /// # Arguments
    /// - `results` — lint/check results per file
    /// - `symbol_tables` — per-file symbol tables (keyed by absolute path)
    /// - `import_graph` — project-wide import dependency graph
    /// - `project_root` — project root for computing relative paths
    pub fn generate_agent(
        &self,
        results: &[FileResult],
        symbol_tables: &[(PathBuf, SymbolTable)],
        import_graph: &ImportGraph,
        project_root: &Path,
    ) -> String {
        let builder = super::agent::AgentOutputBuilder::new(project_root);
        let agent_output = builder.build(results, symbol_tables, import_graph);
        // Compact JSON — no pretty-printing by default (R9)
        serde_json::to_string(&agent_output).unwrap_or_default()
    }

    fn generate_json(&self, results: &[FileResult]) -> String {
        #[derive(Serialize)]
        struct JsonOutput<'a> {
            files: Vec<JsonFile<'a>>,
            summary: JsonSummary,
        }

        #[derive(Serialize)]
        struct JsonFile<'a> {
            path: String,
            language: &'a str,
            diagnostics: &'a [crate::diagnostic::Diagnostic],
        }

        #[derive(Serialize)]
        struct JsonSummary {
            files_checked: usize,
            files_with_issues: usize,
            total_errors: usize,
            total_warnings: usize,
        }

        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut files_with_issues = 0;

        let files: Vec<JsonFile> = results
            .iter()
            .map(|r| {
                if !r.diagnostics.is_empty() {
                    files_with_issues += 1;
                }
                for d in &r.diagnostics {
                    match d.severity {
                        DiagnosticSeverity::Error => total_errors += 1,
                        DiagnosticSeverity::Warning => total_warnings += 1,
                        _ => {}
                    }
                }
                JsonFile {
                    path: r.path.to_string_lossy().to_string(),
                    language: r.language.as_str(),
                    diagnostics: &r.diagnostics,
                }
            })
            .collect();

        let output = JsonOutput {
            files,
            summary: JsonSummary {
                files_checked: results.len(),
                files_with_issues,
                total_errors,
                total_warnings,
            },
        };

        serde_json::to_string_pretty(&output).unwrap_or_default()
    }

    fn generate_markdown(&self, results: &[FileResult]) -> String {
        let mut output = String::new();
        output.push_str("# Lint Report\n\n");

        let mut total_errors = 0;
        let mut total_warnings = 0;

        for result in results {
            if result.diagnostics.is_empty() {
                continue;
            }

            output.push_str(&format!("## {}\n\n", result.path.display()));

            for diag in &result.diagnostics {
                let emoji = match diag.severity {
                    DiagnosticSeverity::Error => {
                        total_errors += 1;
                        "❌"
                    }
                    DiagnosticSeverity::Warning => {
                        total_warnings += 1;
                        "⚠️"
                    }
                    DiagnosticSeverity::Information => "ℹ️",
                    DiagnosticSeverity::Hint => "💡",
                };

                output.push_str(&format!(
                    "- {} **{}** (line {}): {}\n",
                    emoji,
                    diag.code,
                    diag.range.start.line + 1,
                    diag.message
                ));
            }

            output.push('\n');
        }

        output.push_str("## Summary\n\n");
        output.push_str(&format!("- Files checked: {}\n", results.len()));
        output.push_str(&format!("- Errors: {}\n", total_errors));
        output.push_str(&format!("- Warnings: {}\n", total_warnings));

        output
    }

    fn generate_console(&self, results: &[FileResult]) -> String {
        let mut output = String::new();
        let mut total_errors = 0;
        let mut total_warnings = 0;

        for result in results {
            if result.diagnostics.is_empty() {
                continue;
            }

            for diag in &result.diagnostics {
                let severity_str = match diag.severity {
                    DiagnosticSeverity::Error => {
                        total_errors += 1;
                        "error"
                    }
                    DiagnosticSeverity::Warning => {
                        total_warnings += 1;
                        "warning"
                    }
                    DiagnosticSeverity::Information => "info",
                    DiagnosticSeverity::Hint => "hint",
                };

                output.push_str(&format!(
                    "{}:{}:{}: {} [{}]: {}\n",
                    result.path.display(),
                    diag.range.start.line + 1,
                    diag.range.start.character + 1,
                    severity_str,
                    diag.code,
                    diag.message
                ));
            }
        }

        if total_errors > 0 || total_warnings > 0 {
            output.push_str(&format!(
                "\n{} error(s), {} warning(s)\n",
                total_errors, total_warnings
            ));
        }

        output
    }

    /// Generate SARIF 2.1.0 output for code scanning dashboards.
    fn generate_sarif(&self, results: &[FileResult]) -> String {
        use serde_json::{json, Value};

        let mut rules: Vec<Value> = Vec::new();
        let mut rule_ids: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut sarif_results: Vec<Value> = Vec::new();

        for file_result in results {
            let path_str = file_result.path.to_string_lossy();

            for diag in &file_result.diagnostics {
                // Register rule if not seen yet
                let rule_idx = if let Some(&idx) = rule_ids.get(&diag.code) {
                    idx
                } else {
                    let idx = rules.len();
                    rule_ids.insert(diag.code.clone(), idx);
                    rules.push(json!({
                        "id": diag.code,
                        "shortDescription": { "text": diag.code },
                        "defaultConfiguration": {
                            "level": sarif_level(diag.severity)
                        }
                    }));
                    idx
                };

                sarif_results.push(json!({
                    "ruleId": diag.code,
                    "ruleIndex": rule_idx,
                    "level": sarif_level(diag.severity),
                    "message": { "text": diag.message },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": { "uri": path_str },
                            "region": {
                                "startLine": diag.range.start.line + 1,
                                "startColumn": diag.range.start.character + 1,
                                "endLine": diag.range.end.line + 1,
                                "endColumn": diag.range.end.character + 1
                            }
                        }
                    }]
                }));
            }
        }

        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "cclab-lens",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": rules
                    }
                },
                "results": sarif_results
            }]
        });

        serde_json::to_string_pretty(&sarif).unwrap_or_default()
    }

    /// Generate GitHub Actions annotation commands.
    ///
    /// Each diagnostic becomes a `::error` or `::warning` workflow command
    /// that GitHub renders as inline annotations on pull requests.
    fn generate_github(&self, results: &[FileResult]) -> String {
        let mut output = String::new();

        for file_result in results {
            let path_str = file_result.path.to_string_lossy();

            for diag in &file_result.diagnostics {
                let level = match diag.severity {
                    DiagnosticSeverity::Error => "error",
                    DiagnosticSeverity::Warning => "warning",
                    _ => "notice",
                };

                output.push_str(&format!(
                    "::{} file={},line={},col={},endLine={},endColumn={},title={}::{}\n",
                    level,
                    path_str,
                    diag.range.start.line + 1,
                    diag.range.start.character + 1,
                    diag.range.end.line + 1,
                    diag.range.end.character + 1,
                    diag.code,
                    diag.message
                ));
            }
        }

        output
    }

    /// Generate GitLab Code Quality JSON array.
    ///
    /// See https://docs.gitlab.com/ee/ci/testing/code_quality.html#implement-a-custom-tool
    fn generate_gitlab(&self, results: &[FileResult]) -> String {
        use serde_json::{json, Value};

        let mut issues: Vec<Value> = Vec::new();

        for file_result in results {
            let path_str = file_result.path.to_string_lossy();

            for diag in &file_result.diagnostics {
                let severity = match diag.severity {
                    DiagnosticSeverity::Error => "critical",
                    DiagnosticSeverity::Warning => "major",
                    DiagnosticSeverity::Information => "minor",
                    DiagnosticSeverity::Hint => "info",
                };

                // Fingerprint: stable hash of (path, code, line)
                let fp_input = format!("{}:{}:{}", path_str, diag.code, diag.range.start.line + 1);
                let fingerprint = format!("{:x}", simple_hash(fp_input.as_bytes()));

                issues.push(json!({
                    "type": "issue",
                    "check_name": diag.code,
                    "description": diag.message,
                    "content": {
                        "body": format!("[{}] {}", diag.code, diag.message)
                    },
                    "categories": ["Style"],
                    "location": {
                        "path": path_str,
                        "lines": {
                            "begin": diag.range.start.line + 1,
                            "end": diag.range.end.line + 1
                        }
                    },
                    "severity": severity,
                    "fingerprint": fingerprint
                }));
            }
        }

        serde_json::to_string_pretty(&issues).unwrap_or_default()
    }
}

/// Map DiagnosticSeverity to a SARIF level string.
fn sarif_level(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "error",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Information => "note",
        DiagnosticSeverity::Hint => "none",
    }
}

/// Compute a simple 64-bit FNV-1a hash for stable fingerprints.
fn simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

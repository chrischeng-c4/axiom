//! Configuration-driven custom linting engine (R3)
//!
//! Loads user-defined lint rules from `cclab/.index/rules.toml` and evaluates
//! them against source files. All custom rule IDs are namespaced with the
//! `CUSTOM_` prefix so they never collide with built-in rule codes.
//!
//! # Rule kinds
//! - `regex` — match a regular expression against the raw source text
//! - `query` — run a tree-sitter named query against the parsed AST
//!
//! # Example `cclab/.index/rules.toml`
//! ```toml
//! [[rule]]
//! id       = "NO_TODO"
//! kind     = "regex"
//! pattern  = "TODO"
//! severity = "warning"
//! message  = "TODO comment found — resolve before merging"
//!
//! [[rule]]
//! id       = "NO_DBG"
//! kind     = "query"
//! pattern  = "(macro_invocation macro: (identifier) @name (#eq? @name \"dbg\"))"
//! severity = "error"
//! message  = "dbg!() macro must not appear in production code"
//! fix      = "Remove the dbg!() call before merging"
//! ```

use std::path::Path;

use regex_lite::Regex;
use serde::Deserialize;
use tree_sitter::StreamingIterator;

use crate::diagnostic::{
    Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, QuickFix, Range,
};
use crate::syntax::ParsedFile;

// ============================================================================
// Rule configuration types (deserialized from rules.toml)
// ============================================================================

/// Matching strategy for a custom rule
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleKind {
    /// Match a regular expression against each source line
    Regex,
    /// Run a tree-sitter named query against the parsed AST
    Query,
}

/// A single user-defined rule entry from `rules.toml`
#[derive(Debug, Clone, Deserialize)]
pub struct CustomRuleConfig {
    /// Short rule identifier — will be surfaced as `CUSTOM_<id>` in diagnostics
    pub id: String,
    /// Matching strategy
    pub kind: RuleKind,
    /// Regex pattern or tree-sitter query string
    pub pattern: String,
    /// Severity string (`"error"`, `"warning"`, `"information"`, `"hint"`)
    #[serde(default = "default_severity")]
    pub severity: String,
    /// Diagnostic message shown to the user
    pub message: String,
    /// Optional auto-fix hint surfaced as a quick-fix title (no edits)
    #[serde(default)]
    pub fix: Option<String>,
}

fn default_severity() -> String {
    "warning".to_string()
}

impl CustomRuleConfig {
    /// Returns the canonical diagnostic code: `CUSTOM_<ID>`
    pub fn code(&self) -> String {
        format!("CUSTOM_{}", self.id)
    }

    /// Parses the `severity` string into `DiagnosticSeverity`
    pub fn diagnostic_severity(&self) -> DiagnosticSeverity {
        match self.severity.to_ascii_lowercase().as_str() {
            "error" => DiagnosticSeverity::Error,
            "warning" | "warn" => DiagnosticSeverity::Warning,
            "information" | "info" => DiagnosticSeverity::Information,
            "hint" => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Warning,
        }
    }
}

/// Top-level structure of `rules.toml`
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CustomRulesFile {
    /// List of rules (TOML array-of-tables `[[rule]]`)
    #[serde(default, rename = "rule")]
    pub rules: Vec<CustomRuleConfig>,
}

// ============================================================================
// Compiled rule variants
// ============================================================================

struct CompiledRegexRule {
    config: CustomRuleConfig,
    regex: Regex,
}

struct CompiledQueryRule {
    config: CustomRuleConfig,
    /// Raw query string — compiled lazily per-language at check time
    query_str: String,
}

// ============================================================================
// CustomLintEngine
// ============================================================================

/// Engine that evaluates all loaded custom rules against source files.
///
/// Create once with `from_rules_file()` or `load_from_workspace()`, then call
/// `check()` for each file you want to lint.
pub struct CustomLintEngine {
    regex_rules: Vec<CompiledRegexRule>,
    query_rules: Vec<CompiledQueryRule>,
}

impl CustomLintEngine {
    /// Build an engine from an already-parsed `CustomRulesFile`.
    ///
    /// Regex rules are compiled eagerly; invalid patterns are skipped with a
    /// warning rather than panicking.
    pub fn from_rules_file(rules_file: &CustomRulesFile) -> Self {
        let mut regex_rules = Vec::new();
        let mut query_rules = Vec::new();

        for rule in &rules_file.rules {
            match rule.kind {
                RuleKind::Regex => match Regex::new(&rule.pattern) {
                    Ok(regex) => regex_rules.push(CompiledRegexRule {
                        config: rule.clone(),
                        regex,
                    }),
                    Err(e) => {
                        tracing::warn!(
                            "Custom rule '{}': invalid regex '{}': {}",
                            rule.id,
                            rule.pattern,
                            e
                        );
                    }
                },
                RuleKind::Query => query_rules.push(CompiledQueryRule {
                    config: rule.clone(),
                    query_str: rule.pattern.clone(),
                }),
            }
        }

        Self {
            regex_rules,
            query_rules,
        }
    }

    /// Load rules from an explicit file path.
    pub fn load_from_path(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let rules_file: CustomRulesFile = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self::from_rules_file(&rules_file))
    }

    /// Convenience loader: reads from `{workspace_root}/cclab/.index/rules.toml`.
    ///
    /// Returns `None` if the file does not exist or cannot be parsed (errors
    /// are logged at `WARN` level so CI pipelines can still proceed).
    pub fn load_from_workspace(workspace_root: &Path) -> Option<Self> {
        let rules_path = workspace_root
            .join("cclab")
            .join(".index")
            .join("rules.toml");

        if !rules_path.exists() {
            return None;
        }

        match Self::load_from_path(&rules_path) {
            Ok(engine) => Some(engine),
            Err(e) => {
                tracing::warn!("Failed to load custom rules from {:?}: {}", rules_path, e);
                None
            }
        }
    }

    /// Total number of loaded (valid) rules.
    pub fn rule_count(&self) -> usize {
        self.regex_rules.len() + self.query_rules.len()
    }

    /// All custom rule codes exposed by this engine (`CUSTOM_<ID>`).
    pub fn rule_codes(&self) -> Vec<String> {
        let mut codes: Vec<String> = self.regex_rules.iter().map(|r| r.config.code()).collect();
        codes.extend(self.query_rules.iter().map(|r| r.config.code()));
        codes
    }

    /// Apply all custom rules to a parsed file and return diagnostics.
    pub fn check(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.apply_regex_rules(file, &mut diagnostics);
        self.apply_query_rules(file, &mut diagnostics);
        diagnostics
    }

    // -----------------------------------------------------------------------
    // Regex rules — operate on raw source lines
    // -----------------------------------------------------------------------

    fn apply_regex_rules(&self, file: &ParsedFile, diagnostics: &mut Vec<Diagnostic>) {
        for rule in &self.regex_rules {
            self.apply_single_regex_rule(rule, file, diagnostics);
        }
    }

    fn apply_single_regex_rule(
        &self,
        rule: &CompiledRegexRule,
        file: &ParsedFile,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for (line_idx, line) in file.source.lines().enumerate() {
            if !rule.regex.is_match(line) {
                continue;
            }

            let line_num = line_idx as u32;
            let range = Range::new(
                Position::new(line_num, 0),
                Position::new(line_num, line.len() as u32),
            );

            let mut diag = Diagnostic::new(
                range,
                rule.config.diagnostic_severity(),
                rule.config.code(),
                DiagnosticCategory::Custom,
                rule.config.message.clone(),
            );

            if let Some(fix_title) = &rule.config.fix {
                diag.quick_fixes.push(QuickFix {
                    title: fix_title.clone(),
                    edits: Vec::new(), // hint only — no edits
                });
            }

            diagnostics.push(diag);
        }
    }

    // -----------------------------------------------------------------------
    // Tree-sitter query rules — require a real AST
    // -----------------------------------------------------------------------

    fn apply_query_rules(&self, file: &ParsedFile, diagnostics: &mut Vec<Diagnostic>) {
        // Line-based files have a dummy HTML tree — skip query rules entirely.
        if file.is_line_based {
            return;
        }

        let language = file.tree.language();

        for rule in &self.query_rules {
            match tree_sitter::Query::new(&language, &rule.query_str) {
                Ok(query) => {
                    self.apply_single_query_rule(rule, file, &query, diagnostics);
                }
                Err(e) => {
                    // Query may be valid for a different language — skip silently at
                    // debug level to avoid noisy output on polyglot repos.
                    tracing::debug!(
                        "Custom query rule '{}': query compile error for this language: {}",
                        rule.config.id,
                        e
                    );
                }
            }
        }
    }

    fn apply_single_query_rule(
        &self,
        rule: &CompiledQueryRule,
        file: &ParsedFile,
        query: &tree_sitter::Query,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let source_bytes = file.source.as_bytes();
        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(query, file.tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            // Use the first capture node as the diagnostic anchor.
            let Some(capture) = m.captures.first() else {
                continue;
            };

            let node = capture.node;
            let range = Range::from_node(&node);

            let mut diag = Diagnostic::new(
                range,
                rule.config.diagnostic_severity(),
                rule.config.code(),
                DiagnosticCategory::Custom,
                rule.config.message.clone(),
            );

            if let Some(fix_title) = &rule.config.fix {
                diag.quick_fixes.push(QuickFix {
                    title: fix_title.clone(),
                    edits: Vec::new(),
                });
            }

            diagnostics.push(diag);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Language;

    fn line_file(src: &str) -> ParsedFile {
        ParsedFile::line_based(src.to_string(), Language::Markdown)
    }

    fn load_engine(toml: &str) -> CustomLintEngine {
        let f: CustomRulesFile = ::toml::from_str(toml).expect("valid toml");
        CustomLintEngine::from_rules_file(&f)
    }

    // -----------------------------------------------------------------------

    #[test]
    fn test_regex_rule_matches_todo() {
        let engine = load_engine(
            r#"
[[rule]]
id       = "NO_TODO"
kind     = "regex"
pattern  = "TODO"
severity = "warning"
message  = "TODO comment found"
"#,
        );
        let file = line_file("// TODO: fix this\nfn main() {}");
        let diags = engine.check(&file);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "CUSTOM_NO_TODO");
        assert_eq!(diags[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(diags[0].category, DiagnosticCategory::Custom);
    }

    #[test]
    fn test_no_match_returns_empty() {
        let engine = load_engine(
            r#"
[[rule]]
id      = "NO_HACK"
kind    = "regex"
pattern = "HACK"
message = "HACK comment"
"#,
        );
        let file = line_file("fn clean_code() {}");
        assert!(engine.check(&file).is_empty());
    }

    #[test]
    fn test_custom_prefix() {
        let cfg = CustomRuleConfig {
            id: "MY_RULE".to_string(),
            kind: RuleKind::Regex,
            pattern: "x".to_string(),
            severity: "error".to_string(),
            message: "test".to_string(),
            fix: None,
        };
        assert_eq!(cfg.code(), "CUSTOM_MY_RULE");
    }

    #[test]
    fn test_rule_count() {
        let engine = load_engine(
            r#"
[[rule]]
id = "A"
kind = "regex"
pattern = "foo"
message = "a"

[[rule]]
id = "B"
kind = "regex"
pattern = "bar"
message = "b"
"#,
        );
        assert_eq!(engine.rule_count(), 2);
    }

    #[test]
    fn test_invalid_regex_skipped() {
        let engine = load_engine(
            r#"
[[rule]]
id      = "BAD"
kind    = "regex"
pattern = "[unclosed"
message = "bad rule"
"#,
        );
        // Invalid regex must be silently skipped — engine stays functional.
        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn test_fix_hint_attached() {
        let engine = load_engine(
            r#"
[[rule]]
id      = "NO_PRINT"
kind    = "regex"
pattern = "println!"
message = "No println! in production"
fix     = "Replace println! with tracing::info!"
"#,
        );
        let file = line_file("println!(\"hello\");");
        let diags = engine.check(&file);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].quick_fixes.len(), 1);
        assert_eq!(
            diags[0].quick_fixes[0].title,
            "Replace println! with tracing::info!"
        );
    }

    #[test]
    fn test_severity_parsing() {
        let cfgs = [
            ("error", DiagnosticSeverity::Error),
            ("warning", DiagnosticSeverity::Warning),
            ("information", DiagnosticSeverity::Information),
            ("hint", DiagnosticSeverity::Hint),
            ("WARN", DiagnosticSeverity::Warning),
            ("unknown", DiagnosticSeverity::Warning),
        ];
        for (sev_str, expected) in &cfgs {
            let cfg = CustomRuleConfig {
                id: "X".to_string(),
                kind: RuleKind::Regex,
                pattern: "x".to_string(),
                severity: sev_str.to_string(),
                message: "m".to_string(),
                fix: None,
            };
            assert_eq!(
                cfg.diagnostic_severity(),
                *expected,
                "failed for '{}'",
                sev_str
            );
        }
    }

    #[test]
    fn test_multiple_matches_on_same_file() {
        let engine = load_engine(
            r#"
[[rule]]
id      = "NO_TODO"
kind    = "regex"
pattern = "TODO"
message = "TODO found"
"#,
        );
        let file = line_file("// TODO: first\n// normal\n// TODO: second\n");
        let diags = engine.check(&file);
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn test_rule_codes() {
        let engine = load_engine(
            r#"
[[rule]]
id = "ALPHA"
kind = "regex"
pattern = "alpha"
message = "a"
"#,
        );
        let codes = engine.rule_codes();
        assert!(codes.contains(&"CUSTOM_ALPHA".to_string()));
    }
}

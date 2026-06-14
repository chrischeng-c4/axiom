//! SQL lint checker (line-based)
//!
//! Rules: SQ001-SQ005, PG001, MY001
//! Also provides `detect_sql_injection` for Python/JS/Go code.

use super::Checker;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

pub struct SqlChecker;

impl SqlChecker {
    pub fn new() -> Self {
        Self
    }

    /// SQ001: Unmatched parentheses and missing trailing semicolons.
    fn check_syntax(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut depth: i32 = 0;
        let mut open_line: Option<u32> = None;
        for (idx, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("--") || t.is_empty() {
                continue;
            }
            for ch in t.chars() {
                if ch == '(' {
                    if depth == 0 {
                        open_line = Some(idx as u32);
                    }
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth < 0 {
                        diags.push(Diagnostic::new(
                            lr(idx as u32),
                            DiagnosticSeverity::Error,
                            "SQ001",
                            DiagnosticCategory::Syntax,
                            "Unmatched closing parenthesis",
                        ));
                        depth = 0;
                    }
                }
            }
        }
        if depth > 0 {
            diags.push(Diagnostic::new(
                lr(open_line.unwrap_or(0)),
                DiagnosticSeverity::Error,
                "SQ001",
                DiagnosticCategory::Syntax,
                format!("Unmatched opening parenthesis ({} unclosed)", depth),
            ));
        }
        let lines_vec: Vec<(usize, &str)> = source.lines().enumerate().collect();
        let last = lines_vec
            .iter()
            .rev()
            .find(|(_, l)| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with("--")
            })
            .map(|&(i, l)| (i, l));
        if let Some((i, l)) = last {
            let t = l.trim();
            if !t.ends_with(';')
                && !t.to_uppercase().starts_with("BEGIN")
                && !t.to_uppercase().starts_with("END")
            {
                diags.push(Diagnostic::new(
                    lr(i as u32),
                    DiagnosticSeverity::Warning,
                    "SQ001",
                    DiagnosticCategory::Syntax,
                    "Statement does not end with a semicolon",
                ));
            }
        }
        diags
    }

    /// SQ002: SELECT * usage
    fn check_select_star(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let u = line.to_uppercase();
            let t = u.trim();
            if t.starts_with("--") {
                continue;
            }
            if t.contains("SELECT *") || t.contains("SELECT  *") {
                diags.push(Diagnostic::new(
                    lr(i as u32),
                    DiagnosticSeverity::Warning,
                    "SQ002",
                    DiagnosticCategory::Style,
                    "Avoid SELECT * — specify column names explicitly",
                ));
            }
        }
        diags
    }

    /// SQ003: Missing WHERE on UPDATE/DELETE.
    fn check_missing_where(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for stmt in &split_statements(source) {
            let u = stmt.text.to_uppercase();
            let t = u.trim();
            if (t.starts_with("UPDATE ") || t.starts_with("DELETE ")) && !u.contains("WHERE") {
                let kind = if t.starts_with("UPDATE") {
                    "UPDATE"
                } else {
                    "DELETE"
                };
                diags.push(Diagnostic::new(
                    lr(stmt.start_line),
                    DiagnosticSeverity::Warning,
                    "SQ003",
                    DiagnosticCategory::Logic,
                    format!("{} without WHERE clause — this affects all rows", kind),
                ));
            }
        }
        diags
    }

    /// SQ004: Implicit join (FROM table1, table2).
    fn check_implicit_join(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let u = line.to_uppercase();
            let t = u.trim();
            if t.starts_with("--") {
                continue;
            }
            if let Some(fp) = t.find("FROM ") {
                let after = &t[fp + 5..];
                let bw = after.find("WHERE").map_or(after, |w| &after[..w]);
                if !bw.contains("JOIN") && bw.contains(',') {
                    diags.push(Diagnostic::new(
                        lr(i as u32),
                        DiagnosticSeverity::Warning,
                        "SQ004",
                        DiagnosticCategory::Style,
                        "Implicit join — use explicit JOIN syntax",
                    ));
                }
            }
        }
        diags
    }

    /// SQ005: Deprecated CONVERT -> CAST.
    fn check_deprecated_functions(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let t = line.to_uppercase();
            if t.trim().starts_with("--") {
                continue;
            }
            if t.contains("CONVERT(") {
                diags.push(Diagnostic::new(
                    lr(i as u32),
                    DiagnosticSeverity::Hint,
                    "SQ005",
                    DiagnosticCategory::Style,
                    "CONVERT() deprecated — prefer CAST(expr AS type)",
                ));
            }
        }
        diags
    }

    /// PG001: SERIAL -> GENERATED ALWAYS AS IDENTITY.
    fn check_serial_type(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let u = line.to_uppercase();
            let t = u.trim();
            if t.starts_with("--") {
                continue;
            }
            if t.contains("SERIAL") && !t.contains("GENERATED") && !t.contains("SERIALIZABLE") {
                diags.push(Diagnostic::new(
                    lr(i as u32),
                    DiagnosticSeverity::Hint,
                    "PG001",
                    DiagnosticCategory::Style,
                    "Consider GENERATED ALWAYS AS IDENTITY instead of SERIAL",
                ));
            }
        }
        diags
    }

    /// MY001: Missing ENGINE= on CREATE TABLE.
    fn check_missing_engine(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for stmt in &split_statements(source) {
            let u = stmt.text.to_uppercase();
            if u.trim().starts_with("CREATE TABLE") && !u.contains("ENGINE") {
                diags.push(Diagnostic::new(
                    lr(stmt.start_line),
                    DiagnosticSeverity::Hint,
                    "MY001",
                    DiagnosticCategory::Style,
                    "CREATE TABLE without ENGINE= — consider specifying ENGINE=InnoDB",
                ));
            }
        }
        diags
    }

    // =========================================================================
    // AST-based checks (tree-sitter-sql node kinds) — R3
    // =========================================================================

    /// SQ001 via AST: map tree-sitter parse errors to diagnostics.
    fn ast_check_syntax(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        file.collect_errors()
            .into_iter()
            .map(|err| {
                let (row, col) = err.start_position;
                let pos = Position::new(
                    (row.saturating_sub(1)) as u32,
                    (col.saturating_sub(1)) as u32,
                );
                Diagnostic::new(
                    Range::new(pos, pos),
                    DiagnosticSeverity::Error,
                    "SQ001",
                    DiagnosticCategory::Syntax,
                    "SQL syntax error",
                )
            })
            .collect()
    }

    /// SQ002 via AST: detect SELECT * via wildcard nodes in select clauses.
    fn ast_check_select_star(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        file.walk(|node, _depth| {
            if matches!(node.kind(), "wildcard" | "star" | "asterisk" | "all_fields") {
                let in_select = node
                    .parent()
                    .map(|p| {
                        matches!(
                            p.kind(),
                            "select_clause" | "select_expression" | "select_item"
                        )
                    })
                    .unwrap_or(false);
                if in_select {
                    let line = node.start_position().row as u32;
                    diags.push(Diagnostic::new(
                        lr(line),
                        DiagnosticSeverity::Warning,
                        "SQ002",
                        DiagnosticCategory::Style,
                        "Avoid SELECT * — list explicit columns for clarity and safety",
                    ));
                }
            }
            true
        });
        diags
    }

    /// SQ003 via AST: DELETE/UPDATE statements without a WHERE clause.
    fn ast_check_missing_where(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        file.walk(|node, _depth| {
            if matches!(
                node.kind(),
                "delete_statement" | "update_statement" | "delete" | "update"
            ) {
                let has_where = (0..node.child_count())
                    .filter_map(|i| node.child(i))
                    .any(|c| matches!(c.kind(), "where_clause" | "where"));
                if !has_where {
                    let line = node.start_position().row as u32;
                    let stmt = if node.kind().starts_with("delete") {
                        "DELETE"
                    } else {
                        "UPDATE"
                    };
                    diags.push(Diagnostic::new(
                        lr(line),
                        DiagnosticSeverity::Warning,
                        "SQ003",
                        DiagnosticCategory::Logic,
                        format!("{} without WHERE clause — will affect all rows", stmt),
                    ));
                }
            }
            true
        });
        diags
    }

    /// SQ004 via AST: implicit JOIN via comma-separated tables in FROM clause.
    fn ast_check_implicit_join(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        file.walk(|node, _depth| {
            if matches!(node.kind(), "from_clause" | "from") {
                let mut cursor = node.walk();
                let comma_count = node
                    .children(&mut cursor)
                    .filter(|c| c.kind() == ",")
                    .count();
                if comma_count > 0 {
                    let line = node.start_position().row as u32;
                    diags.push(Diagnostic::new(
                        lr(line),
                        DiagnosticSeverity::Warning,
                        "SQ004",
                        DiagnosticCategory::Style,
                        "Implicit JOIN (comma-separated tables) — use explicit JOIN syntax",
                    ));
                }
            }
            true
        });
        diags
    }
}

impl Default for SqlChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect SQL injection patterns in Python/JS/Go source code.
pub fn detect_sql_injection(source: &str, language: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let kws = ["SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "CREATE"];
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.starts_with('#') || t.starts_with("//") {
            continue;
        }
        let u = t.to_uppercase();
        if !kws.iter().any(|kw| u.contains(kw)) {
            continue;
        }
        let ln = i as u32;
        match language {
            "python" | "py" => {
                if t.contains("f\"") || t.contains("f'") {
                    diags.push(inj(ln, "SQL in f-string — use parameterized queries"));
                }
                if t.contains("\" %") || t.contains("' %") {
                    diags.push(inj(ln, "SQL with % formatting — use parameterized queries"));
                }
                if t.contains(".format(") {
                    diags.push(inj(ln, "SQL with .format() — use parameterized queries"));
                }
            }
            "javascript" | "js" | "typescript" | "ts" => {
                if t.contains('`') && t.contains("${") {
                    diags.push(inj(
                        ln,
                        "SQL in template literal — use parameterized queries",
                    ));
                }
                if (t.contains("\"SELECT") || t.contains("'SELECT")) && t.contains(" + ") {
                    diags.push(inj(
                        ln,
                        "SQL string concatenation — use parameterized queries",
                    ));
                }
            }
            "go" => {
                if t.contains("fmt.Sprintf") || t.contains("fmt.Fprintf") {
                    diags.push(inj(ln, "SQL with fmt.Sprintf — use parameterized queries"));
                }
                if (t.contains("\"SELECT") || t.contains("\"INSERT")) && t.contains(" + ") {
                    diags.push(inj(
                        ln,
                        "SQL string concatenation — use parameterized queries",
                    ));
                }
            }
            _ => {}
        }
    }
    diags
}

fn inj(line: u32, msg: &str) -> Diagnostic {
    Diagnostic::new(
        lr(line),
        DiagnosticSeverity::Warning,
        "SQL-INJ",
        DiagnosticCategory::Security,
        msg,
    )
}

struct StatementSpan {
    text: String,
    start_line: u32,
}

fn split_statements(source: &str) -> Vec<StatementSpan> {
    let mut stmts = Vec::new();
    let mut cur = String::new();
    let mut start: u32 = 0;
    let mut found = false;
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("--") || t.is_empty() {
            continue;
        }
        if !found {
            start = i as u32;
            found = true;
        }
        cur.push(' ');
        cur.push_str(t);
        if t.ends_with(';') {
            stmts.push(StatementSpan {
                text: cur.clone(),
                start_line: start,
            });
            cur.clear();
            found = false;
        }
    }
    if !cur.trim().is_empty() {
        stmts.push(StatementSpan {
            text: cur,
            start_line: start,
        });
    }
    stmts
}

fn lr(line: u32) -> Range {
    Range::new(Position::new(line, 0), Position::new(line, u32::MAX))
}

impl Checker for SqlChecker {
    fn language(&self) -> Language {
        Language::Sql
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut d = Vec::new();

        // R3: When a real tree-sitter SQL tree is available, derive syntax errors
        // directly from AST parse errors and use AST node walking for semantic checks.
        if file.language == Language::Sql && !file.has_errors && !file.is_line_based {
            // SQ001 via AST: tree-sitter syntax errors
            d.extend(self.ast_check_syntax(file));
            // Semantic checks use AST node matching (SELECT *, missing WHERE, etc.)
            d.extend(self.ast_check_select_star(file));
            d.extend(self.ast_check_missing_where(file));
            d.extend(self.ast_check_implicit_join(file));
            // Rule-based checks still line-based as they match on keyword patterns
            d.extend(self.check_deprecated_functions(&file.source));
            d.extend(self.check_serial_type(&file.source));
            d.extend(self.check_missing_engine(&file.source));
        } else {
            // Line-based fallback (dummy tree / has_errors)
            d.extend(self.check_syntax(&file.source));
            d.extend(self.check_select_star(&file.source));
            d.extend(self.check_missing_where(&file.source));
            d.extend(self.check_implicit_join(&file.source));
            d.extend(self.check_deprecated_functions(&file.source));
            d.extend(self.check_serial_type(&file.source));
            d.extend(self.check_missing_engine(&file.source));
        }

        d
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "SQ001", "SQ002", "SQ003", "SQ004", "SQ005", "PG001", "MY001",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::LintConfig;

    fn make(s: &str) -> ParsedFile {
        ParsedFile::line_based(s.to_string(), Language::Sql)
    }
    fn codes(d: &[Diagnostic]) -> Vec<&str> {
        d.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn test_unmatched_parens() {
        let d = SqlChecker::new().check(
            &make("SELECT (id, name FROM users;\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"SQ001"),
            "expected SQ001, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_select_star() {
        let d = SqlChecker::new().check(&make("SELECT * FROM users;\n"), &LintConfig::default());
        assert!(
            codes(&d).contains(&"SQ002"),
            "expected SQ002, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_missing_where_delete() {
        let d = SqlChecker::new().check(&make("DELETE FROM users;\n"), &LintConfig::default());
        assert!(
            codes(&d).contains(&"SQ003"),
            "expected SQ003, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_implicit_join() {
        let d = SqlChecker::new().check(
            &make("SELECT a.id FROM users a, orders b WHERE a.id = b.user_id;\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"SQ004"),
            "expected SQ004, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_deprecated_convert() {
        let d = SqlChecker::new().check(
            &make("SELECT CONVERT(varchar, d) FROM t;\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"SQ005"),
            "expected SQ005, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_serial_type() {
        let d = SqlChecker::new().check(
            &make("CREATE TABLE t (\n  id SERIAL PRIMARY KEY\n);\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"PG001"),
            "expected PG001, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_missing_engine() {
        let d = SqlChecker::new().check(
            &make("CREATE TABLE users (\n  id INT PRIMARY KEY\n);\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"MY001"),
            "expected MY001, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_injection_python() {
        let d = detect_sql_injection("q = f\"SELECT * FROM users WHERE id = {uid}\"\n", "python");
        assert!(d.iter().any(|d| d.code == "SQL-INJ"), "expected SQL-INJ");
    }

    #[test]
    fn test_injection_js() {
        let d = detect_sql_injection(
            "const q = `SELECT * FROM users WHERE id = ${uid}`;\n",
            "javascript",
        );
        assert!(d.iter().any(|d| d.code == "SQL-INJ"), "expected SQL-INJ");
    }

    #[test]
    fn test_injection_go() {
        let d = detect_sql_injection(
            "q := fmt.Sprintf(\"SELECT * FROM u WHERE id = %s\", id)\n",
            "go",
        );
        assert!(d.iter().any(|d| d.code == "SQL-INJ"), "expected SQL-INJ");
    }
}

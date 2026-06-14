//! Protocol Buffer (proto3) lint checker (line-based)
//!
//! Rules: PB001-PB007

use super::Checker;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};
use std::collections::{HashMap, HashSet};

pub struct ProtoChecker;

impl ProtoChecker {
    pub fn new() -> Self {
        Self
    }

    /// PB001: Brace/semicolon validation.
    fn check_syntax(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut depth: i32 = 0;
        let mut open_line: Option<u32> = None;
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.is_empty() {
                continue;
            }
            for ch in t.chars() {
                if ch == '{' {
                    if depth == 0 {
                        open_line = Some(i as u32);
                    }
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth < 0 {
                        diags.push(Diagnostic::new(
                            lr(i as u32),
                            DiagnosticSeverity::Error,
                            "PB001",
                            DiagnosticCategory::Syntax,
                            "Unmatched closing brace",
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
                "PB001",
                DiagnosticCategory::Syntax,
                format!("Unmatched opening brace ({} unclosed)", depth),
            ));
        }
        diags
    }

    /// PB002: Duplicate field numbers within a message.
    fn check_duplicate_field_numbers(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_msg = false;
        let mut nums: HashMap<String, u32> = HashMap::new();
        let mut msg_name = String::new();
        let mut depth: i32 = 0;
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.is_empty() {
                continue;
            }
            if t.starts_with("message ") {
                in_msg = true;
                msg_name = t
                    .strip_prefix("message ")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                nums.clear();
            }
            for ch in t.chars() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        in_msg = false;
                    }
                }
            }
            if in_msg && depth == 1 && t.contains('=') && t.ends_with(';') {
                if let Some(n) = field_number(t) {
                    if let Some(&prev) = nums.get(&n) {
                        diags.push(Diagnostic::new(
                            lr(i as u32),
                            DiagnosticSeverity::Error,
                            "PB002",
                            DiagnosticCategory::Logic,
                            format!(
                                "Duplicate field number {} in '{}' (first at line {})",
                                n,
                                msg_name,
                                prev + 1
                            ),
                        ));
                    } else {
                        nums.insert(n, i as u32);
                    }
                }
            }
        }
        diags
    }

    /// PB003: Reserved field number used.
    fn check_reserved_fields(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut reserved: HashSet<u32> = HashSet::new();
        let mut in_msg = false;
        let mut depth: i32 = 0;
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.is_empty() {
                continue;
            }
            if t.starts_with("message ") {
                in_msg = true;
                reserved.clear();
            }
            for ch in t.chars() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        in_msg = false;
                    }
                }
            }
            if in_msg && t.starts_with("reserved ") {
                let nums_part = t
                    .strip_prefix("reserved ")
                    .unwrap_or("")
                    .trim_end_matches(';');
                for part in nums_part.split(',') {
                    let p = part.trim().trim_matches('"');
                    if let Ok(n) = p.parse::<u32>() {
                        reserved.insert(n);
                    } else if p.contains(" to ") {
                        let ps: Vec<&str> = p.split(" to ").collect();
                        if ps.len() == 2 {
                            if let (Ok(a), Ok(b)) =
                                (ps[0].trim().parse::<u32>(), ps[1].trim().parse::<u32>())
                            {
                                for n in a..=b {
                                    reserved.insert(n);
                                }
                            }
                        }
                    }
                }
                continue;
            }
            if in_msg && depth == 1 && t.contains('=') && t.ends_with(';') {
                if let Some(ns) = field_number(t) {
                    if let Ok(n) = ns.parse::<u32>() {
                        if reserved.contains(&n) {
                            diags.push(Diagnostic::new(
                                lr(i as u32),
                                DiagnosticSeverity::Error,
                                "PB003",
                                DiagnosticCategory::Logic,
                                format!("Field number {} is reserved", n),
                            ));
                        }
                    }
                }
            }
        }
        diags
    }

    /// PB004: Missing package declaration.
    fn check_missing_package(&self, source: &str) -> Vec<Diagnostic> {
        if source.lines().any(|l| l.trim().starts_with("package ")) {
            Vec::new()
        } else {
            vec![Diagnostic::new(
                lr(0),
                DiagnosticSeverity::Warning,
                "PB004",
                DiagnosticCategory::Style,
                "Missing 'package' declaration",
            )]
        }
    }

    /// PB005: Service without rpc methods.
    fn check_empty_service(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut svc: Option<(String, u32)> = None;
        let mut has_rpc = false;
        let mut depth: i32 = 0;
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.is_empty() {
                continue;
            }
            if t.starts_with("service ") && svc.is_none() {
                let name = t
                    .strip_prefix("service ")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                svc = Some((name, i as u32));
                has_rpc = false;
            }
            for ch in t.chars() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        if let Some((ref name, sl)) = svc {
                            if !has_rpc {
                                diags.push(Diagnostic::new(
                                    lr(sl),
                                    DiagnosticSeverity::Warning,
                                    "PB005",
                                    DiagnosticCategory::Logic,
                                    format!("Service '{}' has no rpc methods", name),
                                ));
                            }
                        }
                        svc = None;
                    }
                }
            }
            if svc.is_some() && t.starts_with("rpc ") {
                has_rpc = true;
            }
        }
        diags
    }

    /// PB006: Import not used.
    fn check_unused_imports(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut imports: Vec<(String, u32)> = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("import ") {
                if let Some(path) = import_path(t) {
                    imports.push((path, i as u32));
                }
            }
        }
        let rest: String = source
            .lines()
            .filter(|l| !l.trim().starts_with("import "))
            .collect::<Vec<_>>()
            .join("\n");
        for (path, line) in &imports {
            let base = path
                .rsplit('/')
                .next()
                .unwrap_or(path)
                .strip_suffix(".proto")
                .unwrap_or(path);
            let pascal = to_pascal(base);
            if !rest.contains(&pascal) && !rest.contains(base) {
                diags.push(Diagnostic::new(
                    lr(*line),
                    DiagnosticSeverity::Warning,
                    "PB006",
                    DiagnosticCategory::Logic,
                    format!("Import '{}' appears unused", path),
                ));
            }
        }
        diags
    }

    /// PB007: Field naming convention (should be snake_case).
    fn check_field_naming(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_block = false;
        let mut depth: i32 = 0;
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.is_empty() {
                continue;
            }
            if t.starts_with("message ") || t.starts_with("enum ") {
                in_block = true;
            }
            for ch in t.chars() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        in_block = false;
                    }
                }
            }
            if in_block && depth >= 1 && t.contains('=') && t.ends_with(';') {
                if let Some(name) = field_name(t) {
                    if !is_snake_case(name) {
                        diags.push(Diagnostic::new(
                            lr(i as u32),
                            DiagnosticSeverity::Warning,
                            "PB007",
                            DiagnosticCategory::Style,
                            format!("Field '{}' should be snake_case", name),
                        ));
                    }
                }
            }
        }
        diags
    }
    // =========================================================================
    // AST-based checks (tree-sitter-proto node kinds) — R3
    // =========================================================================

    /// PB001 via AST: map tree-sitter parse errors to diagnostics.
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
                    "PB001",
                    DiagnosticCategory::Syntax,
                    "Protobuf syntax error",
                )
            })
            .collect()
    }

    /// PB002 via AST: detect duplicate field numbers within the same message.
    fn ast_check_duplicate_field_numbers(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();

        file.walk(|node, _depth| {
            // message_body or message_block contains field definitions
            if matches!(node.kind(), "message_body" | "message_block" | "message") {
                let mut seen: std::collections::HashMap<String, u32> =
                    std::collections::HashMap::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if matches!(child.kind(), "field" | "message_field") {
                        // Field number is typically in a child named "number" or "field_number"
                        if let Some(num_node) = child
                            .child_by_field_name("number")
                            .or_else(|| child.child_by_field_name("field_number"))
                        {
                            if let Ok(num) = num_node.utf8_text(source) {
                                let line = child.start_position().row as u32;
                                if let Some(&prev) = seen.get(num) {
                                    diags.push(Diagnostic::new(
                                        lr(line),
                                        DiagnosticSeverity::Error,
                                        "PB002",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "Duplicate field number {} (first at line {})",
                                            num,
                                            prev + 1
                                        ),
                                    ));
                                } else {
                                    seen.insert(num.to_string(), line);
                                }
                            }
                        }
                    }
                }
            }
            true
        });
        diags
    }

    /// PB004 via AST: missing `package` declaration at top level.
    fn ast_check_missing_package(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut has_package = false;
        file.walk(|node, _depth| {
            if node.kind() == "package" || node.kind() == "package_statement" {
                has_package = true;
            }
            true
        });
        if !has_package {
            vec![Diagnostic::new(
                lr(0),
                DiagnosticSeverity::Warning,
                "PB004",
                DiagnosticCategory::Style,
                "Missing package declaration",
            )]
        } else {
            vec![]
        }
    }

    /// PB005 via AST: service definitions with no RPC methods.
    fn ast_check_empty_service(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();

        file.walk(|node, _depth| {
            if matches!(node.kind(), "service" | "service_definition") {
                let has_rpc = (0..node.child_count())
                    .filter_map(|i| node.child(i))
                    .any(|c| matches!(c.kind(), "rpc" | "rpc_definition" | "rpc_statement"));
                if !has_rpc {
                    let name = node
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source).ok())
                        .unwrap_or("unknown");
                    let line = node.start_position().row as u32;
                    diags.push(Diagnostic::new(
                        lr(line),
                        DiagnosticSeverity::Warning,
                        "PB005",
                        DiagnosticCategory::Logic,
                        format!("Service '{}' has no RPC methods", name),
                    ));
                }
            }
            true
        });
        diags
    }
}

impl Default for ProtoChecker {
    fn default() -> Self {
        Self::new()
    }
}

fn lr(line: u32) -> Range {
    Range::new(Position::new(line, 0), Position::new(line, u32::MAX))
}

fn field_number(line: &str) -> Option<String> {
    let after = line[line.find('=')? + 1..]
        .trim()
        .trim_end_matches(';')
        .trim();
    if after.chars().all(|c| c.is_ascii_digit()) {
        Some(after.to_string())
    } else {
        None
    }
}

fn field_name(line: &str) -> Option<&str> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 && parts.iter().any(|p| p.contains('=')) {
        Some(parts[1])
    } else {
        None
    }
}

fn import_path(line: &str) -> Option<String> {
    let after = line.trim().strip_prefix("import ")?;
    let after = after.strip_prefix("public ").unwrap_or(after);
    let p = after
        .trim()
        .trim_matches('"')
        .trim_end_matches(';')
        .trim_matches('"');
    if p.is_empty() {
        None
    } else {
        Some(p.to_string())
    }
}

fn to_pascal(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(ch) => ch.to_uppercase().to_string() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect()
}

fn is_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !s.starts_with('_')
        && !s.ends_with('_')
        && !s.contains("__")
}

impl Checker for ProtoChecker {
    fn language(&self) -> Language {
        Language::Proto
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut d = Vec::new();

        // R3: AST-based syntax error detection when a real tree is present;
        // semantic checks use AST node walking for field numbers, services, etc.
        if file.language == Language::Proto && !file.has_errors && !file.is_line_based {
            // PB001 via AST: tree-sitter parse errors
            d.extend(self.ast_check_syntax(file));
            // Semantic checks using AST node matching
            d.extend(self.ast_check_duplicate_field_numbers(file));
            d.extend(self.ast_check_empty_service(file));
            d.extend(self.ast_check_missing_package(file));
            // Field naming and reserved checks still use line-based patterns
            d.extend(self.check_reserved_fields(&file.source));
            d.extend(self.check_unused_imports(&file.source));
            d.extend(self.check_field_naming(&file.source));
        } else {
            // Line-based fallback
            d.extend(self.check_syntax(&file.source));
            d.extend(self.check_duplicate_field_numbers(&file.source));
            d.extend(self.check_reserved_fields(&file.source));
            d.extend(self.check_missing_package(&file.source));
            d.extend(self.check_empty_service(&file.source));
            d.extend(self.check_unused_imports(&file.source));
            d.extend(self.check_field_naming(&file.source));
        }

        d
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "PB001", "PB002", "PB003", "PB004", "PB005", "PB006", "PB007",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::LintConfig;

    fn make(s: &str) -> ParsedFile {
        ParsedFile::line_based(s.to_string(), Language::Proto)
    }
    fn codes(d: &[Diagnostic]) -> Vec<&str> {
        d.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn test_missing_package() {
        let d = ProtoChecker::new().check(
            &make("syntax = \"proto3\";\nmessage Foo {\n  string name = 1;\n}\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"PB004"),
            "expected PB004, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_duplicate_field_number() {
        let d = ProtoChecker::new().check(
            &make("syntax = \"proto3\";\npackage test;\nmessage Foo {\n  string a = 1;\n  string b = 1;\n}\n"),
            &LintConfig::default());
        assert!(
            codes(&d).contains(&"PB002"),
            "expected PB002, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_empty_service() {
        let d = ProtoChecker::new().check(
            &make("syntax = \"proto3\";\npackage test;\nservice MyService {\n}\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"PB005"),
            "expected PB005, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_field_naming() {
        let d = ProtoChecker::new().check(
            &make("syntax = \"proto3\";\npackage test;\nmessage Foo {\n  string myField = 1;\n}\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"PB007"),
            "expected PB007, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_clean_proto() {
        let src = "syntax = \"proto3\";\npackage test;\nmessage Foo {\n  string my_field = 1;\n  int32 count = 2;\n}\nservice FooService {\n  rpc GetFoo (Foo) returns (Foo);\n}\n";
        let d = ProtoChecker::new().check(&make(src), &LintConfig::default());
        assert!(d.is_empty(), "unexpected diagnostics: {:?}", codes(&d));
    }

    #[test]
    fn test_snake_case_helper() {
        assert!(is_snake_case("my_field"));
        assert!(is_snake_case("name"));
        assert!(!is_snake_case("myField"));
        assert!(!is_snake_case("MyField"));
    }
}

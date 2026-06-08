//! Extract Rust tests and convert to Python tests.
//!
//! This module extracts `#[test]` functions from Rust source files
//! and translates them to equivalent Python pytest tests.

use anyhow::{Context, Result};
use std::path::Path;

/// Configuration for test extraction
#[derive(Debug, Clone, Default)]
pub struct TestExtractorConfig {
    /// Rust struct name to Python class name mapping
    pub type_mapping: Vec<(String, String)>,
    /// Module import path (e.g., "cclab.titan")
    pub python_module: String,
}

/// Extracted Rust test
#[derive(Debug, Clone)]
pub struct RustTest {
    pub name: String,
    pub body: String,
    pub source_file: String,
    pub line_number: usize,
    pub is_async: bool,
}

/// Test extractor and translator
pub struct TestExtractor {
    config: TestExtractorConfig,
}

impl TestExtractor {
    pub fn new(config: TestExtractorConfig) -> Self {
        Self { config }
    }

    /// Extract tests from a Rust source file
    pub fn extract_tests(&self, path: &Path) -> Result<Vec<RustTest>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;

        self.extract_tests_from_source(&content, path.to_string_lossy().to_string())
    }

    /// Extract tests from source string
    pub fn extract_tests_from_source(
        &self,
        content: &str,
        source_file: String,
    ) -> Result<Vec<RustTest>> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .context("Failed to set Rust language")?;

        let tree = parser
            .parse(content, None)
            .context("Failed to parse Rust source")?;

        let mut tests = Vec::new();
        self.visit_node(tree.root_node(), content, &source_file, &mut tests)?;

        Ok(tests)
    }

    fn visit_node(
        &self,
        node: tree_sitter::Node,
        source: &str,
        source_file: &str,
        tests: &mut Vec<RustTest>,
    ) -> Result<()> {
        if node.kind() == "function_item" {
            if let Some(test) = self.extract_test_function(node, source, source_file)? {
                tests.push(test);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child, source, source_file, tests)?;
        }

        Ok(())
    }

    fn extract_test_function(
        &self,
        node: tree_sitter::Node,
        source: &str,
        source_file: &str,
    ) -> Result<Option<RustTest>> {
        // Check for #[test] attribute
        let has_test_attr = self.has_test_attribute(node, source);
        if !has_test_attr {
            return Ok(None);
        }

        // Get function name
        let name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();

        // Check if async
        let is_async = source[node.byte_range()].contains("async fn");

        // Get function body
        let body = node
            .child_by_field_name("body")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();

        let line_number = node.start_position().row + 1;

        Ok(Some(RustTest {
            name,
            body,
            source_file: source_file.to_string(),
            line_number,
            is_async,
        }))
    }

    fn has_test_attribute(&self, node: tree_sitter::Node, source: &str) -> bool {
        let mut prev = node.prev_sibling();
        while let Some(sibling) = prev {
            if sibling.kind() == "attribute_item" {
                let text = self.node_text(sibling, source);
                if text.contains("#[test]") || text.contains("#[tokio::test]") {
                    return true;
                }
            } else if sibling.kind() != "attribute_item" && sibling.kind() != "line_comment" {
                break;
            }
            prev = sibling.prev_sibling();
        }
        false
    }

    fn node_text(&self, node: tree_sitter::Node, source: &str) -> String {
        source[node.byte_range()].to_string()
    }

    /// Convert a Rust test to Python test code
    pub fn translate_to_python(&self, test: &RustTest) -> String {
        let mut py_body = self.translate_rust_to_python(&test.body);

        // Indent the body
        py_body = py_body
            .lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n");

        let decorator = if test.is_async {
            "@pytest.mark.asyncio\n"
        } else {
            ""
        };
        let async_kw = if test.is_async { "async " } else { "" };

        format!(
            r#"{decorator}{async_kw}def {name}():
    """Translated from: {source}:{line}"""
{body}
"#,
            decorator = decorator,
            async_kw = async_kw,
            name = test.name,
            source = test
                .source_file
                .split('/')
                .last()
                .unwrap_or(&test.source_file),
            line = test.line_number,
            body = py_body,
        )
    }

    /// Translate Rust code block to Python
    fn translate_rust_to_python(&self, rust_code: &str) -> String {
        let mut code = rust_code.to_string();

        // Remove outer braces
        code = code.trim().to_string();
        if code.starts_with('{') && code.ends_with('}') {
            code = code[1..code.len() - 1].to_string();
        }

        // Apply translations
        code = self.translate_statements(&code);

        code
    }

    fn translate_statements(&self, code: &str) -> String {
        let mut lines: Vec<String> = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let translated = self.translate_line(trimmed);
            if !translated.is_empty() {
                // Handle method chaining - if line starts with '.', append to previous
                if translated.starts_with('.') && !lines.is_empty() {
                    let last = lines.pop().unwrap();
                    lines.push(format!("{}{}", last, translated));
                } else {
                    lines.push(translated);
                }
            }
        }

        lines.join("\n")
    }

    fn translate_line(&self, line: &str) -> String {
        let mut result = line.to_string();

        // Remove trailing semicolons
        result = result.trim_end_matches(';').to_string();

        // let (a, b) = ... -> a, b = ... (tuple unpacking)
        if result.starts_with("let (") {
            if let Some(eq_pos) = result.find(" = ") {
                let pattern = &result[4..eq_pos]; // Skip "let "
                let value = &result[eq_pos + 3..];
                // Remove outer parens from pattern if present
                let pattern = pattern.trim();
                let pattern = if pattern.starts_with('(') && pattern.ends_with(')') {
                    &pattern[1..pattern.len() - 1]
                } else {
                    pattern
                };
                result = format!("{} = {}", pattern, value);
            }
        }
        // let x = ... -> x = ...
        else if result.starts_with("let ") {
            result = result.replacen("let ", "", 1);
            // Remove mut
            result = result.replacen("mut ", "", 1);
        }

        // .unwrap() -> (remove)
        result = result.replace(".unwrap()", "");

        // .expect("...") -> (remove)
        if let Some(idx) = result.find(".expect(") {
            if let Some(end_idx) = result[idx..].find(')') {
                result = format!("{}{}", &result[..idx], &result[idx + end_idx + 1..]);
            }
        }

        // vec!["a", "b"] -> ["a", "b"]
        result = self.translate_vec_macro(&result);

        // "string".to_string() -> "string"
        result = result.replace(".to_string()", "");

        // assert_eq!(a, b) -> assert a == b
        result = self.translate_assert_eq(&result);

        // assert!(x) -> assert x
        result = self.translate_assert(&result);

        // Operator::Eq -> "="
        result = self.translate_operators(&result);

        // ExtractedValue::Int(42) -> 42
        result = self.translate_extracted_value(&result);

        // OrderDirection::Desc -> "desc" or OrderDirection enum
        result = self.translate_order_direction(&result);

        // Type::new(...) -> Type(...)
        result = self.translate_constructor(&result);

        // x.len() -> len(x)
        result = self.translate_len_method(&result);

        // Apply custom type mappings
        for (rust_type, py_type) in &self.config.type_mapping {
            result = result.replace(rust_type, py_type);
        }

        result.trim().to_string()
    }

    fn translate_len_method(&self, code: &str) -> String {
        let mut result = code.to_string();

        // Simple pattern: identifier.len() -> len(identifier)
        while let Some(len_pos) = result.find(".len()") {
            // Find the start of the identifier
            let before = &result[..len_pos];
            let mut start = len_pos;
            for (i, c) in before.chars().rev().enumerate() {
                if c.is_alphanumeric() || c == '_' {
                    start = len_pos - i - 1;
                } else {
                    break;
                }
            }
            let ident = &result[start..len_pos];
            let after = &result[len_pos + 6..];
            result = format!("{}len({}){}", &result[..start], ident, after);
        }

        result
    }

    fn translate_vec_macro(&self, code: &str) -> String {
        let mut result = code.to_string();

        // Simple vec![] replacement
        while let Some(start) = result.find("vec![") {
            let rest = &result[start + 5..];
            if let Some(end) = self.find_matching_bracket(rest, '[', ']') {
                let inner = &rest[..end];
                let replacement = format!("[{}]", inner);
                result = format!("{}{}{}", &result[..start], replacement, &rest[end + 1..]);
            } else {
                break;
            }
        }

        result
    }

    fn find_matching_bracket(&self, s: &str, open: char, close: char) -> Option<usize> {
        let mut depth = 1;
        for (i, c) in s.chars().enumerate() {
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
        None
    }

    fn translate_assert_eq(&self, code: &str) -> String {
        let mut result = code.to_string();

        // assert_eq!(a, b) -> assert a == b
        while let Some(start) = result.find("assert_eq!(") {
            let rest = &result[start + 11..];
            if let Some(end) = self.find_matching_bracket(rest, '(', ')') {
                let inner = &rest[..end];
                // Split on first top-level comma
                if let Some((left, right)) = self.split_on_comma(inner) {
                    let replacement = format!("assert {} == {}", left.trim(), right.trim());
                    result = format!("{}{}{}", &result[..start], replacement, &rest[end + 1..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        result
    }

    fn translate_assert(&self, code: &str) -> String {
        let mut result = code.to_string();

        // assert!(x) -> assert x (but not assert_eq which we already handled)
        if result.contains("assert!(") && !result.contains("assert_eq") {
            while let Some(start) = result.find("assert!(") {
                let rest = &result[start + 8..];
                if let Some(end) = self.find_matching_bracket(rest, '(', ')') {
                    let inner = &rest[..end];
                    let replacement = format!("assert {}", inner.trim());
                    result = format!("{}{}{}", &result[..start], replacement, &rest[end + 1..]);
                } else {
                    break;
                }
            }
        }

        result
    }

    fn split_on_comma(&self, s: &str) -> Option<(String, String)> {
        let mut depth = 0;
        for (i, c) in s.chars().enumerate() {
            match c {
                '(' | '[' | '{' | '<' => depth += 1,
                ')' | ']' | '}' | '>' => depth -= 1,
                ',' if depth == 0 => {
                    return Some((s[..i].to_string(), s[i + 1..].to_string()));
                }
                _ => {}
            }
        }
        None
    }

    fn translate_operators(&self, code: &str) -> String {
        code.replace("Operator::Eq", "\"=\"")
            .replace("Operator::Ne", "\"!=\"")
            .replace("Operator::Gt", "\">\"")
            .replace("Operator::Gte", "\">=\"")
            .replace("Operator::Lt", "\"<\"")
            .replace("Operator::Lte", "\"<=\"")
            .replace("Operator::Like", "\"LIKE\"")
            .replace("Operator::In", "\"IN\"")
    }

    fn translate_extracted_value(&self, code: &str) -> String {
        let mut result = code.to_string();

        // ExtractedValue::Int(42) -> 42
        result = self.replace_enum_variant(&result, "ExtractedValue::Int");
        result = self.replace_enum_variant(&result, "ExtractedValue::Float");
        result = self.replace_enum_variant(&result, "ExtractedValue::Bool");
        result = self.replace_enum_variant(&result, "ExtractedValue::String");

        result
    }

    fn translate_order_direction(&self, code: &str) -> String {
        code.replace("OrderDirection::Asc", "\"asc\"")
            .replace("OrderDirection::Desc", "\"desc\"")
    }

    fn replace_enum_variant(&self, code: &str, pattern: &str) -> String {
        let mut result = code.to_string();
        let search = format!("{}(", pattern);

        while let Some(start) = result.find(&search) {
            let rest = &result[start + search.len()..];
            if let Some(end) = self.find_matching_bracket(rest, '(', ')') {
                let inner = &rest[..end];
                result = format!("{}{}{}", &result[..start], inner, &rest[end + 1..]);
            } else {
                break;
            }
        }

        result
    }

    fn translate_constructor(&self, code: &str) -> String {
        let mut result = code.to_string();

        // QueryBuilder::new("table") -> QueryBuilder("table")
        // But only for types we know about
        let constructors = ["QueryBuilder::new", "WindowSpec::new"];
        for ctor in constructors {
            result = result.replace(ctor, &ctor.replace("::new", ""));
        }

        result
    }

    /// Generate a complete Python test file from extracted tests
    pub fn generate_test_file(&self, tests: &[RustTest]) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            r#"""Auto-generated Python tests from Rust tests.

DO NOT EDIT - Regenerate with `cclab lens gen-python-test`
"""
import pytest
from {} import *

"#,
            self.config.python_module
        ));

        // Generate test functions
        for test in tests {
            output.push_str(&self.translate_to_python(test));
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_extractor() -> TestExtractor {
        TestExtractor::new(TestExtractorConfig {
            type_mapping: vec![("QueryBuilder".to_string(), "RustQueryBuilder".to_string())],
            python_module: "cclab.titan".to_string(),
        })
    }

    #[test]
    fn test_extract_simple_test() {
        let extractor = make_extractor();
        let source = r#"
#[test]
fn test_simple() {
    let x = 1;
    assert_eq!(x, 1);
}
"#;

        let tests = extractor
            .extract_tests_from_source(source, "test.rs".to_string())
            .unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "test_simple");
    }

    #[test]
    fn test_translate_let() {
        let extractor = make_extractor();
        assert_eq!(extractor.translate_line("let x = 1;"), "x = 1");
        assert_eq!(extractor.translate_line("let mut x = 1;"), "x = 1");
    }

    #[test]
    fn test_translate_vec() {
        let extractor = make_extractor();
        let result = extractor.translate_vec_macro(r#"vec!["a", "b"]"#);
        assert_eq!(result, r#"["a", "b"]"#);
    }

    #[test]
    fn test_translate_assert_eq() {
        let extractor = make_extractor();
        let result = extractor.translate_assert_eq("assert_eq!(x, 1)");
        assert_eq!(result, "assert x == 1");
    }

    #[test]
    fn test_translate_assert() {
        let extractor = make_extractor();
        let result = extractor.translate_assert("assert!(x > 0)");
        assert_eq!(result, "assert x > 0");
    }

    #[test]
    fn test_translate_unwrap() {
        let extractor = make_extractor();
        let result = extractor.translate_line("let x = foo().unwrap();");
        assert_eq!(result, "x = foo()");
    }

    #[test]
    fn test_translate_to_string() {
        let extractor = make_extractor();
        let result = extractor.translate_line(r#"let x = "hello".to_string();"#);
        assert_eq!(result, r#"x = "hello""#);
    }

    #[test]
    fn test_translate_operator() {
        let extractor = make_extractor();
        let result = extractor.translate_operators("Operator::Eq");
        assert_eq!(result, "\"=\"");
    }

    #[test]
    fn test_translate_extracted_value() {
        let extractor = make_extractor();
        let result = extractor.translate_extracted_value("ExtractedValue::Int(42)");
        assert_eq!(result, "42");
    }

    #[test]
    fn test_translate_full_test() {
        let extractor = make_extractor();
        let source = r#"
#[test]
fn test_simple_select() {
    let qb = QueryBuilder::new("users").unwrap();
    let (sql, params) = qb.build_select();
    assert_eq!(sql, "SELECT * FROM \"users\"");
    assert_eq!(params.len(), 0);
}
"#;

        let tests = extractor
            .extract_tests_from_source(source, "test.rs".to_string())
            .unwrap();
        let python = extractor.translate_to_python(&tests[0]);

        assert!(python.contains("def test_simple_select():"));
        assert!(python.contains("qb = RustQueryBuilder(\"users\")"));
        assert!(python.contains("sql, params = qb.build_select()"));
        assert!(python.contains("assert sql == \"SELECT * FROM \\\"users\\\"\""));
    }

    #[test]
    fn test_generate_multiple_tests() {
        let extractor = make_extractor();
        let source = r#"
#[test]
fn test_select_with_columns() {
    let qb = QueryBuilder::new("users").unwrap()
        .select(vec!["id".to_string(), "name".to_string()]).unwrap();
    let (sql, params) = qb.build_select();
    assert_eq!(sql, "SELECT \"id\", \"name\" FROM \"users\"");
    assert_eq!(params.len(), 0);
}

#[test]
fn test_select_with_where() {
    let qb = QueryBuilder::new("users").unwrap()
        .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
    let (sql, params) = qb.build_select();
    assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"id\" = $1");
    assert_eq!(params.len(), 1);
}
"#;

        let tests = extractor
            .extract_tests_from_source(source, "query.rs".to_string())
            .unwrap();
        assert_eq!(tests.len(), 2);

        let python = extractor.generate_test_file(&tests);

        // Check header
        assert!(python.contains("Auto-generated Python tests"));
        assert!(python.contains("from cclab.titan import *"));

        // Check first test
        assert!(python.contains("def test_select_with_columns():"));
        assert!(python.contains(r#".select(["id", "name"])"#));

        // Check second test
        assert!(python.contains("def test_select_with_where():"));
        assert!(python.contains(r#".where_clause("id", "=", 42)"#));

        // Print for visual inspection
        println!("\n=== Generated Python Test File ===\n{}", python);
    }
}

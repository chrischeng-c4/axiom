//! Multi-language refactoring support
//!
//! Provides Rust and TypeScript refactoring operations alongside Python.

use std::collections::HashSet;
use std::path::PathBuf;

use super::mutable_ast::Span;
#[allow(unused_imports)]
use super::refactoring::{
    DiagnosticLevel, RefactorOptions, RefactorRequest, RefactorResult, TextEdit,
};

/// Language for refactoring operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactorLanguage {
    Python,
    TypeScript,
    Rust,
}

impl RefactorLanguage {
    /// Detect language from file extension
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "py" => Some(Self::Python),
            "ts" | "tsx" | "js" | "jsx" => Some(Self::TypeScript),
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }

    /// Get keywords for this language
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            Self::Python => &[
                "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
                "continue", "def", "del", "elif", "else", "except", "finally", "for", "from",
                "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass",
                "raise", "return", "try", "while", "with", "yield",
            ],
            Self::TypeScript => &[
                "break",
                "case",
                "catch",
                "class",
                "const",
                "continue",
                "debugger",
                "default",
                "delete",
                "do",
                "else",
                "enum",
                "export",
                "extends",
                "false",
                "finally",
                "for",
                "function",
                "if",
                "import",
                "in",
                "instanceof",
                "new",
                "null",
                "return",
                "super",
                "switch",
                "this",
                "throw",
                "true",
                "try",
                "typeof",
                "var",
                "void",
                "while",
                "with",
                "yield",
                "let",
                "static",
                "implements",
                "interface",
                "package",
                "private",
                "protected",
                "public",
                "async",
                "await",
                "type",
                "as",
            ],
            Self::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
                "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            ],
        }
    }

    /// Check if name is a keyword
    pub fn is_keyword(&self, name: &str) -> bool {
        self.keywords().contains(&name)
    }
}

/// Multi-language refactoring engine
pub struct MultiLangRefactorer;

impl MultiLangRefactorer {
    pub fn new() -> Self {
        Self
    }

    /// Extract function for any supported language
    pub fn extract_function(
        &self,
        request: &RefactorRequest,
        name: &str,
        source: &str,
    ) -> RefactorResult {
        let lang = match RefactorLanguage::from_path(&request.file) {
            Some(l) => l,
            None => {
                let mut result = RefactorResult::empty();
                result.add_diagnostic(
                    DiagnosticLevel::Error,
                    "Unsupported file type for refactoring",
                    Some(request.file.clone()),
                    None,
                );
                return result;
            }
        };

        match lang {
            RefactorLanguage::Python => self.extract_function_python(request, name, source),
            RefactorLanguage::TypeScript => self.extract_function_typescript(request, name, source),
            RefactorLanguage::Rust => self.extract_function_rust(request, name, source),
        }
    }

    /// Extract function (TypeScript)
    fn extract_function_typescript(
        &self,
        request: &RefactorRequest,
        name: &str,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Validate function name
        if name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Function name cannot be empty",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        if RefactorLanguage::TypeScript.is_keyword(name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("'{}' is a reserved keyword", name),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Analyze data flow
        let data_flow = self.analyze_data_flow(request.span, source);
        let selected_code = &source[request.span.start..request.span.end];

        // Build parameters
        let params: Vec<String> = data_flow.external_vars.clone();
        let params_str = if request.options.add_type_annotations {
            params
                .iter()
                .map(|p| format!("{}: any", p))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            params.join(", ")
        };

        // Build function body
        let body_lines: Vec<String> = selected_code
            .lines()
            .map(|line| format!("  {}", line))
            .collect();
        let mut body = body_lines.join("\n");

        // Add return if needed
        if !data_flow.returned_vars.is_empty() {
            let return_vars = data_flow.returned_vars.join(", ");
            if data_flow.returned_vars.len() == 1 {
                body.push_str(&format!("\n  return {};", return_vars));
            } else {
                body.push_str(&format!("\n  return {{ {} }};", return_vars));
            }
        }

        // Generate function
        let return_type = if request.options.add_type_annotations {
            if data_flow.returned_vars.is_empty() {
                ": void"
            } else {
                ": any"
            }
        } else {
            ""
        };

        let func_def = format!(
            "function {}({}){} {{\n{}\n}}\n\n",
            name, params_str, return_type, body
        );

        // Generate call
        let call_str = if params.is_empty() {
            if data_flow.returned_vars.is_empty() {
                format!("{}();", name)
            } else if data_flow.returned_vars.len() == 1 {
                format!("const {} = {}();", data_flow.returned_vars[0], name)
            } else {
                format!(
                    "const {{ {} }} = {}();",
                    data_flow.returned_vars.join(", "),
                    name
                )
            }
        } else {
            let call_params = params.join(", ");
            if data_flow.returned_vars.is_empty() {
                format!("{}({});", name, call_params)
            } else if data_flow.returned_vars.len() == 1 {
                format!(
                    "const {} = {}({});",
                    data_flow.returned_vars[0], name, call_params
                )
            } else {
                format!(
                    "const {{ {} }} = {}({});",
                    data_flow.returned_vars.join(", "),
                    name,
                    call_params
                )
            }
        };

        // Find insertion point
        let insert_pos = self.find_ts_insertion_point(source, request.span);

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: Span::new(insert_pos, insert_pos),
                new_text: func_def,
            },
        );

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: request.span,
                new_text: call_str,
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Extracted TypeScript function '{}' with {} parameter(s)",
                name,
                params.len()
            ),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Extract function (Rust)
    fn extract_function_rust(
        &self,
        request: &RefactorRequest,
        name: &str,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Validate function name
        if name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Function name cannot be empty",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        if RefactorLanguage::Rust.is_keyword(name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("'{}' is a reserved keyword", name),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Analyze data flow
        let data_flow = self.analyze_data_flow(request.span, source);
        let selected_code = &source[request.span.start..request.span.end];

        // Build parameters with type annotations (required in Rust)
        let params: Vec<String> = data_flow.external_vars.clone();
        let params_str = params
            .iter()
            .map(|p| {
                if data_flow.mutable_vars.contains(p) {
                    format!("{}: &mut _", p)
                } else {
                    format!("{}: &_", p)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        // Build function body
        let body_lines: Vec<String> = selected_code
            .lines()
            .map(|line| format!("    {}", line))
            .collect();
        let mut body = body_lines.join("\n");

        // Add return if needed
        if !data_flow.returned_vars.is_empty() {
            let return_vars = data_flow.returned_vars.join(", ");
            if data_flow.returned_vars.len() == 1 {
                body.push_str(&format!("\n    {}", return_vars));
            } else {
                body.push_str(&format!("\n    ({})", return_vars));
            }
        }

        // Generate function
        let func_def = format!("fn {}({}) {{\n{}\n}}\n\n", name, params_str, body);

        // Generate call
        let call_str = if params.is_empty() {
            if data_flow.returned_vars.is_empty() {
                format!("{}();", name)
            } else if data_flow.returned_vars.len() == 1 {
                format!("let {} = {}();", data_flow.returned_vars[0], name)
            } else {
                format!("let ({}) = {}();", data_flow.returned_vars.join(", "), name)
            }
        } else {
            let call_params = params
                .iter()
                .map(|p| {
                    if data_flow.mutable_vars.contains(p) {
                        format!("&mut {}", p)
                    } else {
                        format!("&{}", p)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            if data_flow.returned_vars.is_empty() {
                format!("{}({});", name, call_params)
            } else if data_flow.returned_vars.len() == 1 {
                format!(
                    "let {} = {}({});",
                    data_flow.returned_vars[0], name, call_params
                )
            } else {
                format!(
                    "let ({}) = {}({});",
                    data_flow.returned_vars.join(", "),
                    name,
                    call_params
                )
            }
        };

        // Find insertion point
        let insert_pos = self.find_rust_insertion_point(source, request.span);

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: Span::new(insert_pos, insert_pos),
                new_text: func_def,
            },
        );

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: request.span,
                new_text: call_str,
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Extracted Rust function '{}' with {} parameter(s)",
                name,
                params.len()
            ),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Extract function (Python) - uses existing logic
    fn extract_function_python(
        &self,
        request: &RefactorRequest,
        name: &str,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Validate function name
        if name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Function name cannot be empty",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        if RefactorLanguage::Python.is_keyword(name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("'{}' is a reserved keyword", name),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Analyze data flow
        let data_flow = self.analyze_data_flow(request.span, source);
        let selected_code = &source[request.span.start..request.span.end];

        // Build parameters
        let params: Vec<String> = data_flow.external_vars.clone();
        let params_str = if request.options.add_type_annotations {
            params
                .iter()
                .map(|p| format!("{}: Any", p))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            params.join(", ")
        };

        // Build function body
        let body_lines: Vec<String> = selected_code
            .lines()
            .map(|line| format!("    {}", line))
            .collect();
        let mut body = body_lines.join("\n");

        // Add return if needed
        if !data_flow.returned_vars.is_empty() {
            let return_vars = data_flow.returned_vars.join(", ");
            if data_flow.returned_vars.len() == 1 {
                body.push_str(&format!("\n    return {}", return_vars));
            } else {
                body.push_str(&format!("\n    return ({})", return_vars));
            }
        }

        // Generate function
        let func_def = format!("def {}({}):\n{}\n\n", name, params_str, body);

        // Generate call
        let call_str = if params.is_empty() {
            if data_flow.returned_vars.is_empty() {
                format!("{}()", name)
            } else if data_flow.returned_vars.len() == 1 {
                format!("{} = {}()", data_flow.returned_vars[0], name)
            } else {
                format!("{} = {}()", data_flow.returned_vars.join(", "), name)
            }
        } else {
            let call_params = params.join(", ");
            if data_flow.returned_vars.is_empty() {
                format!("{}({})", name, call_params)
            } else if data_flow.returned_vars.len() == 1 {
                format!("{} = {}({})", data_flow.returned_vars[0], name, call_params)
            } else {
                format!(
                    "{} = {}({})",
                    data_flow.returned_vars.join(", "),
                    name,
                    call_params
                )
            }
        };

        // Find insertion point
        let insert_pos = self.find_python_insertion_point(source, request.span);

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: Span::new(insert_pos, insert_pos),
                new_text: func_def,
            },
        );

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: request.span,
                new_text: call_str,
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Extracted Python function '{}' with {} parameter(s)",
                name,
                params.len()
            ),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Simple data flow analysis
    fn analyze_data_flow(&self, span: Span, source: &str) -> DataFlowResult {
        let selected = &source[span.start..span.end];

        // Find identifiers in selection
        let mut defined = HashSet::new();
        let mut used = HashSet::new();
        let mut mutated = HashSet::new();

        // Simple regex-like patterns
        for word in selected.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if !word.is_empty()
                && word
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic() || c == '_')
                    .unwrap_or(false)
            {
                used.insert(word.to_string());
            }
        }

        // Find assignments
        for line in selected.lines() {
            let trimmed = line.trim();
            // Python: x = ...
            // Rust: let x = ... or let mut x = ...
            // TS: const x = ... or let x = ... or var x = ...
            if let Some(idx) = trimmed.find('=') {
                let before = &trimmed[..idx].trim();
                // Handle let/const/var/mut
                let var_part = before
                    .trim_start_matches("let ")
                    .trim_start_matches("const ")
                    .trim_start_matches("var ")
                    .trim_start_matches("mut ")
                    .trim();

                if !var_part.is_empty() {
                    for word in var_part.split(|c: char| !c.is_alphanumeric() && c != '_') {
                        if !word.is_empty() {
                            defined.insert(word.to_string());
                        }
                    }
                }
            }

            // Check for mutation (&mut, += etc)
            if trimmed.contains("&mut ") || trimmed.contains("+=") || trimmed.contains("-=") {
                for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if !word.is_empty() {
                        mutated.insert(word.to_string());
                    }
                }
            }
        }

        // External vars = used but not defined in selection
        let external_vars: Vec<String> = used
            .difference(&defined)
            .filter(|v| {
                !["self", "this", "true", "false", "None", "null", "undefined"]
                    .contains(&v.as_str())
            })
            .cloned()
            .collect();

        // Variables defined that may need to be returned
        let returned_vars: Vec<String> = defined.iter().cloned().collect();

        DataFlowResult {
            external_vars,
            returned_vars,
            mutable_vars: mutated,
        }
    }

    /// Find insertion point for TypeScript
    fn find_ts_insertion_point(&self, source: &str, span: Span) -> usize {
        let selection_pos = span.start;
        let mut current_pos = 0;
        let mut function_start_pos = 0;
        let mut _function_indent = 0;

        for line in source.lines() {
            let line_end = current_pos + line.len();

            if current_pos > selection_pos {
                break;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with("function ")
                || trimmed.contains("=>") && trimmed.contains("const ")
            {
                function_start_pos = current_pos;
                _function_indent = line.len() - trimmed.len();
            }

            current_pos = line_end + 1;
        }

        // Find end of current function
        current_pos = function_start_pos;
        let mut brace_count = 0;
        let mut found_start = false;

        for line in source[function_start_pos..].lines() {
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_start = true;
                } else if ch == '}' {
                    brace_count -= 1;
                    if found_start && brace_count == 0 {
                        return current_pos + line.len() + 1;
                    }
                }
            }
            current_pos += line.len() + 1;
        }

        source.len()
    }

    /// Find insertion point for Rust
    fn find_rust_insertion_point(&self, source: &str, span: Span) -> usize {
        let selection_pos = span.start;
        let mut current_pos = 0;
        let mut function_start_pos = 0;

        for line in source.lines() {
            let line_end = current_pos + line.len();

            if current_pos > selection_pos {
                break;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                function_start_pos = current_pos;
            }

            current_pos = line_end + 1;
        }

        // Find end of current function (matching braces)
        current_pos = function_start_pos;
        let mut brace_count = 0;
        let mut found_start = false;

        for line in source[function_start_pos..].lines() {
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_start = true;
                } else if ch == '}' {
                    brace_count -= 1;
                    if found_start && brace_count == 0 {
                        return current_pos + line.len() + 1;
                    }
                }
            }
            current_pos += line.len() + 1;
        }

        source.len()
    }

    /// Find insertion point for Python
    fn find_python_insertion_point(&self, source: &str, span: Span) -> usize {
        let selection_pos = span.start;
        let mut current_pos = 0;
        let mut function_start_pos = 0;
        let mut function_indent = 0;

        for line in source.lines() {
            let line_end = current_pos + line.len();

            if current_pos > selection_pos {
                break;
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                function_start_pos = current_pos;
                function_indent = line.len() - trimmed.len();
            }

            current_pos = line_end + 1;
        }

        // Find end of current function (next def/class at same indent)
        current_pos = function_start_pos;
        let mut found_start = false;

        for line in source[function_start_pos..].lines() {
            let line_end = current_pos + line.len();

            if !found_start {
                found_start = true;
                current_pos = line_end + 1;
                continue;
            }

            let trimmed = line.trim_start();
            let line_indent = line.len() - trimmed.len();

            if (trimmed.starts_with("def ") || trimmed.starts_with("class "))
                && line_indent <= function_indent
            {
                return current_pos;
            }

            current_pos = line_end + 1;
        }

        source.len()
    }

    /// Cross-file rename for any language
    pub fn rename_symbol(
        &self,
        file: &PathBuf,
        old_name: &str,
        new_name: &str,
        _source: &str,
        all_usages: &[(PathBuf, Span)],
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        let lang = match RefactorLanguage::from_path(file) {
            Some(l) => l,
            None => {
                result.add_diagnostic(
                    DiagnosticLevel::Error,
                    "Unsupported file type",
                    Some(file.clone()),
                    None,
                );
                return result;
            }
        };

        // Validate new name
        if new_name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "New name cannot be empty",
                Some(file.clone()),
                None,
            );
            return result;
        }

        if lang.is_keyword(new_name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("'{}' is a reserved keyword", new_name),
                Some(file.clone()),
                None,
            );
            return result;
        }

        // Create edits for all usages
        for (usage_file, span) in all_usages {
            result.add_edit(
                usage_file.clone(),
                TextEdit {
                    span: *span,
                    new_text: new_name.to_string(),
                },
            );
        }

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Renamed '{}' to '{}' across {} file(s)",
                old_name,
                new_name,
                result.file_edits.len()
            ),
            Some(file.clone()),
            None,
        );

        result
    }
}

impl Default for MultiLangRefactorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Data flow analysis result
struct DataFlowResult {
    external_vars: Vec<String>,
    returned_vars: Vec<String>,
    mutable_vars: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(
            RefactorLanguage::from_path(&PathBuf::from("test.py")),
            Some(RefactorLanguage::Python)
        );
        assert_eq!(
            RefactorLanguage::from_path(&PathBuf::from("test.ts")),
            Some(RefactorLanguage::TypeScript)
        );
        assert_eq!(
            RefactorLanguage::from_path(&PathBuf::from("test.rs")),
            Some(RefactorLanguage::Rust)
        );
        assert_eq!(
            RefactorLanguage::from_path(&PathBuf::from("test.txt")),
            None
        );
    }

    #[test]
    fn test_keyword_detection() {
        assert!(RefactorLanguage::Python.is_keyword("def"));
        assert!(RefactorLanguage::TypeScript.is_keyword("function"));
        assert!(RefactorLanguage::Rust.is_keyword("fn"));

        assert!(!RefactorLanguage::Python.is_keyword("foo"));
        assert!(!RefactorLanguage::TypeScript.is_keyword("bar"));
        assert!(!RefactorLanguage::Rust.is_keyword("baz"));
    }

    #[test]
    fn test_extract_function_rust() {
        let refactorer = MultiLangRefactorer::new();
        let source = r#"
fn main() {
    let x = 1;
    let y = 2;
    let sum = x + y;
    println!("{}", sum);
}
"#;
        let request = RefactorRequest {
            kind: super::super::refactoring::RefactorKind::ExtractFunction {
                name: "add_numbers".to_string(),
            },
            file: PathBuf::from("test.rs"),
            span: Span::new(35, 55),
            options: RefactorOptions::default(),
        };

        let result = refactorer.extract_function(&request, "add_numbers", source);
        assert!(!result.file_edits.is_empty());
    }

    #[test]
    fn test_extract_function_typescript() {
        let refactorer = MultiLangRefactorer::new();
        let source = r#"
function main() {
    const x = 1;
    const y = 2;
    const sum = x + y;
    console.log(sum);
}
"#;
        let request = RefactorRequest {
            kind: super::super::refactoring::RefactorKind::ExtractFunction {
                name: "addNumbers".to_string(),
            },
            file: PathBuf::from("test.ts"),
            span: Span::new(35, 60),
            options: RefactorOptions::default(),
        };

        let result = refactorer.extract_function(&request, "addNumbers", source);
        assert!(!result.file_edits.is_empty());
    }

    #[test]
    fn test_keyword_validation_rust() {
        let refactorer = MultiLangRefactorer::new();
        let source = "fn main() { let x = 1; }";
        let request = RefactorRequest {
            kind: super::super::refactoring::RefactorKind::ExtractFunction {
                name: "fn".to_string(), // Rust keyword
            },
            file: PathBuf::from("test.rs"),
            span: Span::new(12, 22),
            options: RefactorOptions::default(),
        };

        let result = refactorer.extract_function(&request, "fn", source);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.level == DiagnosticLevel::Error));
    }
}

//! Python security lint rules (PY301-PY305)

use crate::diagnostic::{Diagnostic, DiagnosticCategory, Position, Range};
use crate::syntax::ParsedFile;

/// PY301: `eval()` usage
pub(super) fn check_eval_usage(file: &ParsedFile) -> Vec<Diagnostic> {
    check_function_call(
        file,
        "eval",
        "PY301",
        "Use of eval() is a security risk — consider ast.literal_eval() or safer alternatives",
    )
}

/// PY302: `exec()` usage
pub(super) fn check_exec_usage(file: &ParsedFile) -> Vec<Diagnostic> {
    check_function_call(
        file,
        "exec",
        "PY302",
        "Use of exec() is a security risk — avoid executing dynamic code",
    )
}

/// PY303: `pickle.loads()` / `pickle.load()` usage
pub(super) fn check_pickle_usage(file: &ParsedFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() == "call" {
            if let Some(func) = node.child_by_field_name("function") {
                let func_text = file.node_text(&func);
                if func_text == "pickle.loads"
                    || func_text == "pickle.load"
                    || func_text == "cPickle.loads"
                    || func_text == "cPickle.load"
                {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "PY303",
                        DiagnosticCategory::Security,
                        format!(
                            "{}() can execute arbitrary code during deserialization — use a safer format",
                            func_text,
                        ),
                    ));
                }
            }
        }
        true
    });

    diagnostics
}

/// PY304: `subprocess` with `shell=True`
pub(super) fn check_subprocess_shell(file: &ParsedFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() == "call" {
            if let Some(func) = node.child_by_field_name("function") {
                let func_text = file.node_text(&func);
                let is_subprocess = func_text.starts_with("subprocess.")
                    || func_text == "Popen"
                    || func_text == "call"
                    || func_text == "check_output"
                    || func_text == "check_call";

                if !is_subprocess {
                    return true;
                }

                // Check for shell=True in keyword arguments
                if let Some(args) = node.child_by_field_name("arguments") {
                    let mut cursor = args.walk();
                    for child in args.children(&mut cursor) {
                        if child.kind() == "keyword_argument" {
                            if let Some(name) = child.child_by_field_name("name") {
                                if file.node_text(&name) == "shell" {
                                    if let Some(value) = child.child_by_field_name("value") {
                                        if file.node_text(&value) == "True" {
                                            diagnostics.push(Diagnostic::warning(
                                                Range::from_node(node),
                                                "PY304",
                                                DiagnosticCategory::Security,
                                                format!(
                                                    "{}() with shell=True is a security risk — use a list of arguments instead",
                                                    func_text,
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        true
    });

    diagnostics
}

/// PY305: Hardcoded secrets (strings matching patterns like `password=`, `secret=`, `api_key=`)
pub(super) fn check_hardcoded_secrets(lines: &[&str]) -> Vec<Diagnostic> {
    const SECRET_PATTERNS: &[&str] = &[
        "password=",
        "password =",
        "secret=",
        "secret =",
        "api_key=",
        "api_key =",
        "apikey=",
        "apikey =",
        "token=",
        "token =",
        "private_key=",
        "private_key =",
        "access_key=",
        "access_key =",
        "secret_key=",
        "secret_key =",
    ];

    let mut diagnostics = Vec::new();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let lower = trimmed.to_lowercase();
        for pattern in SECRET_PATTERNS {
            if lower.contains(pattern) {
                // Check that this looks like an assignment with a literal value
                // (not just referencing os.environ or a variable)
                let after_pat = &lower[lower.find(pattern).unwrap() + pattern.len()..];
                let after_trimmed = after_pat.trim();
                // Skip if the value references an env var or is empty
                if after_trimmed.starts_with("os.environ")
                    || after_trimmed.starts_with("os.getenv")
                    || after_trimmed.starts_with("environ")
                    || after_trimmed.starts_with("none")
                    || after_trimmed.starts_with("\"\"")
                    || after_trimmed.starts_with("''")
                    || after_trimmed.is_empty()
                {
                    continue;
                }
                // Only flag if it looks like there's a string literal value
                if after_trimmed.starts_with('"')
                    || after_trimmed.starts_with('\'')
                    || after_trimmed.starts_with("b\"")
                    || after_trimmed.starts_with("b'")
                {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "PY305",
                        DiagnosticCategory::Security,
                        format!(
                            "Possible hardcoded secret matching '{}' — use environment variables or a secrets manager",
                            pattern.trim(),
                        ),
                    ));
                    break; // One diagnostic per line
                }
            }
        }
    }

    diagnostics
}

/// Helper: check for a simple function call by name
fn check_function_call(
    file: &ParsedFile,
    func_name: &str,
    code: &str,
    message: &str,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() == "call" {
            if let Some(func) = node.child_by_field_name("function") {
                let func_text = file.node_text(&func);
                if func_text == func_name {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        code,
                        DiagnosticCategory::Security,
                        message,
                    ));
                }
            }
        }
        true
    });

    diagnostics
}

//! Test file detection by naming conventions per language (R5, R10).
//!
//! Supports Python, TypeScript, Rust, and Go naming conventions for test files.

use std::path::Path;

use super::types::{ContextEntry, ContextReason};

/// Test file score is always 0.7, regardless of depth (per spec).
const TEST_FILE_SCORE: f64 = 0.7;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/context_builder/test_detection.md#schema
// CODEGEN-BEGIN
/// Language-specific test file detection configuration.
/// @spec projects/agentic-workflow/tech-design/core/logic/context_builder/test_detection.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestLanguage {
    /// Python (.py).
    Python,
    /// TypeScript / JavaScript (.ts/.tsx/.js/.jsx).
    TypeScript,
    /// Rust (.rs).
    Rust,
    /// Go (.go).
    Go,
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/context_builder/test_detection.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/logic/context_builder/test_detection.md#source
impl TestLanguage {
    /// Detect language from file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "py" => Some(Self::Python),
            "ts" | "tsx" | "js" | "jsx" => Some(Self::TypeScript),
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            _ => None,
        }
    }
}

/// Detect test files that likely cover a given target file.
///
/// Scans `project_files` for files matching test naming conventions in the
/// target file's language. Returns `ContextEntry` items with reason `TestFile`.
///
/// # Convention table (from spec)
///
/// | Language   | Pattern                                |
/// |------------|----------------------------------------|
/// | Python     | `test_{name}.py`, `{name}_test.py`     |
/// | TypeScript | `{name}.test.ts`, `{name}.spec.tsx`    |
/// | Rust       | `tests/{name}.rs`                      |
/// | Go         | `{name}_test.go`                       |
/// @spec projects/agentic-workflow/tech-design/core/logic/context_builder/test_detection.md#source
pub fn detect_test_files(
    project_files: &[String],
    target_file: &str,
    language: TestLanguage,
) -> Vec<ContextEntry> {
    let target_path = Path::new(target_file);
    let stem = match target_path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut results = Vec::new();

    for file in project_files {
        let file_path = Path::new(file.as_str());
        if is_test_file_for(file_path, stem, language) {
            results.push(ContextEntry {
                path: file.clone(),
                reason: ContextReason::TestFile,
                symbols: Vec::new(),
                depth: 0,
                score: TEST_FILE_SCORE,
            });
        }
    }

    results
}

/// Check if `candidate` is a test file for a module with the given `stem`.
fn is_test_file_for(candidate: &Path, stem: &str, language: TestLanguage) -> bool {
    let candidate_name = match candidate.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    match language {
        TestLanguage::Python => is_python_test(candidate, candidate_name, stem),
        TestLanguage::TypeScript => is_typescript_test(candidate_name, stem),
        TestLanguage::Rust => is_rust_test(candidate, candidate_name, stem),
        TestLanguage::Go => is_go_test(candidate_name, stem),
    }
}

/// Python: `test_{name}.py` or `{name}_test.py` or `tests/test_{name}.py`
fn is_python_test(_candidate: &Path, name: &str, stem: &str) -> bool {
    if !name.ends_with(".py") {
        return false;
    }
    let base = &name[..name.len() - 3]; // strip .py
    base == format!("test_{}", stem) || base == format!("{}_test", stem)
}

/// TypeScript: `{name}.test.ts`, `{name}.test.tsx`, `{name}.spec.ts`, `{name}.spec.tsx`
fn is_typescript_test(name: &str, stem: &str) -> bool {
    let expected_patterns = [
        format!("{}.test.ts", stem),
        format!("{}.test.tsx", stem),
        format!("{}.spec.ts", stem),
        format!("{}.spec.tsx", stem),
        format!("{}.test.js", stem),
        format!("{}.test.jsx", stem),
        format!("{}.spec.js", stem),
        format!("{}.spec.jsx", stem),
    ];
    expected_patterns.iter().any(|p| name == p)
}

/// Rust: `tests/{name}.rs`
fn is_rust_test(candidate: &Path, name: &str, stem: &str) -> bool {
    if !name.ends_with(".rs") {
        return false;
    }
    // Check if the file is in a `tests/` directory
    let in_tests_dir = candidate
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|n| n == "tests")
        .unwrap_or(false);

    if in_tests_dir {
        let base = &name[..name.len() - 3]; // strip .rs
        return base == stem;
    }

    false
}

/// Go: `{name}_test.go`
fn is_go_test(name: &str, stem: &str) -> bool {
    if !name.ends_with("_test.go") {
        return false;
    }
    let base = &name[..name.len() - 8]; // strip _test.go
    base == stem
}

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // Python test detection (R5, R10)
    // ====================================================================

    /// Matches `test_{name}.py` and `{name}_test.py` patterns.
    #[test]
    fn test_detect_python_tests() {
        let project_files = vec![
            "src/user.py".to_string(),
            "tests/test_user.py".to_string(),
            "tests/user_test.py".to_string(),
            "tests/test_other.py".to_string(),
            "tests/unrelated.py".to_string(),
        ];

        let entries = detect_test_files(&project_files, "src/user.py", TestLanguage::Python);

        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(
            paths.contains(&"tests/test_user.py"),
            "should match test_{{name}}.py"
        );
        assert!(
            paths.contains(&"tests/user_test.py"),
            "should match {{name}}_test.py"
        );
        assert!(
            !paths.contains(&"tests/test_other.py"),
            "should not match unrelated test"
        );
        assert!(
            !paths.contains(&"tests/unrelated.py"),
            "should not match non-test file"
        );
        assert_eq!(entries.len(), 2);

        // All entries should have TestFile reason and 0.7 score
        for entry in &entries {
            assert_eq!(entry.reason, ContextReason::TestFile);
            assert!(
                (entry.score - 0.7).abs() < 1e-6,
                "test file score should be 0.7"
            );
        }
    }

    // ====================================================================
    // TypeScript test detection (R5, R10)
    // ====================================================================

    /// Matches `*.test.ts` and `*.spec.ts` patterns.
    #[test]
    fn test_detect_typescript_tests() {
        let project_files = vec![
            "src/App.ts".to_string(),
            "src/App.test.ts".to_string(),
            "src/App.spec.ts".to_string(),
            "src/App.test.tsx".to_string(),
            "src/App.spec.tsx".to_string(),
            "src/Other.test.ts".to_string(),
            "src/utils.ts".to_string(),
        ];

        let entries = detect_test_files(&project_files, "src/App.ts", TestLanguage::TypeScript);

        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"src/App.test.ts"), "should match .test.ts");
        assert!(paths.contains(&"src/App.spec.ts"), "should match .spec.ts");
        assert!(
            paths.contains(&"src/App.test.tsx"),
            "should match .test.tsx"
        );
        assert!(
            paths.contains(&"src/App.spec.tsx"),
            "should match .spec.tsx"
        );
        assert!(
            !paths.contains(&"src/Other.test.ts"),
            "should not match different name"
        );
        assert!(
            !paths.contains(&"src/utils.ts"),
            "should not match non-test file"
        );
        assert_eq!(entries.len(), 4);
    }

    // ====================================================================
    // Rust test detection (R5, R10)
    // ====================================================================

    /// Matches `tests/{name}.rs` pattern.
    #[test]
    fn test_detect_rust_tests() {
        let project_files = vec![
            "src/parser.rs".to_string(),
            "tests/parser.rs".to_string(),
            "tests/other.rs".to_string(),
            "src/tests/parser.rs".to_string(), // nested tests/ dir
        ];

        let entries = detect_test_files(&project_files, "src/parser.rs", TestLanguage::Rust);

        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(
            paths.contains(&"tests/parser.rs"),
            "should match tests/{{name}}.rs"
        );
        assert!(
            paths.contains(&"src/tests/parser.rs"),
            "should match nested tests/ dir"
        );
        assert!(
            !paths.contains(&"tests/other.rs"),
            "should not match different name"
        );
    }

    // ====================================================================
    // Go test detection (R5, R10)
    // ====================================================================

    /// Matches `{name}_test.go` pattern.
    #[test]
    fn test_detect_go_tests() {
        let project_files = vec![
            "pkg/handler.go".to_string(),
            "pkg/handler_test.go".to_string(),
            "pkg/utils_test.go".to_string(),
            "pkg/handler.go.bak".to_string(),
        ];

        let entries = detect_test_files(&project_files, "pkg/handler.go", TestLanguage::Go);

        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(
            paths.contains(&"pkg/handler_test.go"),
            "should match {{name}}_test.go"
        );
        assert!(
            !paths.contains(&"pkg/utils_test.go"),
            "should not match different name"
        );
        assert_eq!(entries.len(), 1);
    }

    // ====================================================================
    // Language detection from path
    // ====================================================================

    #[test]
    fn test_language_from_path() {
        assert_eq!(
            TestLanguage::from_path(Path::new("a.py")),
            Some(TestLanguage::Python)
        );
        assert_eq!(
            TestLanguage::from_path(Path::new("b.ts")),
            Some(TestLanguage::TypeScript)
        );
        assert_eq!(
            TestLanguage::from_path(Path::new("c.tsx")),
            Some(TestLanguage::TypeScript)
        );
        assert_eq!(
            TestLanguage::from_path(Path::new("d.js")),
            Some(TestLanguage::TypeScript)
        );
        assert_eq!(
            TestLanguage::from_path(Path::new("e.jsx")),
            Some(TestLanguage::TypeScript)
        );
        assert_eq!(
            TestLanguage::from_path(Path::new("f.rs")),
            Some(TestLanguage::Rust)
        );
        assert_eq!(
            TestLanguage::from_path(Path::new("g.go")),
            Some(TestLanguage::Go)
        );
        assert_eq!(TestLanguage::from_path(Path::new("h.rb")), None);
        assert_eq!(TestLanguage::from_path(Path::new("no_ext")), None);
    }

    // ====================================================================
    // Edge cases
    // ====================================================================

    #[test]
    fn test_empty_project_files() {
        let entries = detect_test_files(&[], "src/user.py", TestLanguage::Python);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_no_matching_tests() {
        let project_files = vec!["src/user.py".to_string(), "tests/test_other.py".to_string()];
        let entries = detect_test_files(&project_files, "src/user.py", TestLanguage::Python);
        assert!(entries.is_empty());
    }
}

// CODEGEN-END

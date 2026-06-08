//! EstimateTokensTool — heuristic token count for a file or directory.
//!
//! Calculates an approximate token count by counting total lines in a path
//! and multiplying by `TOKENS_PER_LINE` (3). The result informs
//! [`RestructureCodebaseAgent`]'s budget-constrained grouping loop.
//!
//! [`RestructureCodebaseAgent`]: crate::agents::restructure_codebase::RestructureCodebaseAgent

use crate::error::NovaResult;
use crate::tools::tool::{Tool, ToolParameter};
use async_trait::async_trait;
use serde::Deserialize;
use std::io::BufRead as _;
use std::path::Path;

/// Heuristic multiplier: tokens per line.
const TOKENS_PER_LINE: u64 = 3;

/// Tool that returns an estimated token count for a path.
pub struct EstimateTokensTool;

impl EstimateTokensTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EstimateTokensTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct EstimateTokensArgs {
    path: String,
}

#[async_trait]
impl Tool for EstimateTokensTool {
    fn name(&self) -> &str {
        "estimate_tokens"
    }

    fn description(&self) -> &str {
        "Estimate the number of tokens for a file or directory by counting total lines \
         and applying a heuristic multiplier (lines × 3). Use this to determine whether \
         a directory fits within the token budget before assigning it to a spec group."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".to_string(),
            description: "File or directory path to estimate tokens for".to_string(),
            required: true,
            parameter_type: "string".to_string(),
        }]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: EstimateTokensArgs = serde_json::from_value(arguments)?;
        let path = Path::new(&args.path);

        if !path.exists() {
            return Ok(serde_json::json!({
                "path": args.path,
                "error": format!("Path does not exist: {}", args.path)
            }));
        }

        let (file_count, line_count) = count_recursive(path);
        let estimated_tokens = line_count * TOKENS_PER_LINE;

        Ok(serde_json::json!({
            "path": args.path,
            "file_count": file_count,
            "line_count": line_count,
            "estimated_tokens": estimated_tokens,
            "heuristic": format!("lines × {}", TOKENS_PER_LINE)
        }))
    }
}

// ============================================================
// Helpers
// ============================================================

/// Recursively count files and total lines.
/// Skips hidden entries (dot-prefixed names).
fn count_recursive(path: &Path) -> (u64, u64) {
    if path.is_file() {
        return (1, count_lines(path));
    }

    let mut files: u64 = 0;
    let mut lines: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            if fname.to_string_lossy().starts_with('.') {
                continue;
            }
            let (f, l) = count_recursive(&entry.path());
            files += f;
            lines += l;
        }
    }

    (files, lines)
}

/// Count lines in a single file using a buffered reader.
/// Returns 0 for binary or unreadable files.
fn count_lines(path: &Path) -> u64 {
    let Ok(file) = std::fs::File::open(path) else {
        return 0;
    };
    std::io::BufReader::new(file).lines().count() as u64
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_count_lines_single_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rs");
        fs::write(&path, "a\nb\nc\n").unwrap();
        assert_eq!(count_lines(&path), 3);
    }

    #[test]
    fn test_count_recursive_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rs");
        fs::write(&path, "line1\nline2\n").unwrap();
        let (files, lines) = count_recursive(&path);
        assert_eq!(files, 1);
        assert_eq!(lines, 2);
    }

    #[test]
    fn test_count_recursive_directory() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("a.rs"), "1\n2\n3\n").unwrap();
        fs::write(src.join("b.rs"), "x\n").unwrap();

        let (files, lines) = count_recursive(dir.path());
        assert_eq!(files, 2);
        assert_eq!(lines, 4);
    }

    #[test]
    fn test_hidden_files_skipped() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".hidden"), "secret\n").unwrap();
        fs::write(dir.path().join("visible.rs"), "pub fn foo() {}\n").unwrap();
        let (files, lines) = count_recursive(dir.path());
        assert_eq!(files, 1);
        assert_eq!(lines, 1);
    }

    #[tokio::test]
    async fn test_execute_returns_estimate() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\nfn foo() {}\n").unwrap();

        let tool = EstimateTokensTool::new();
        let result = tool
            .execute(serde_json::json!({ "path": dir.path().to_str().unwrap() }))
            .await
            .unwrap();

        assert_eq!(result["file_count"], 1);
        assert_eq!(result["line_count"], 2);
        assert_eq!(result["estimated_tokens"], 6); // 2 × 3
    }

    #[tokio::test]
    async fn test_execute_nonexistent_path() {
        let tool = EstimateTokensTool::new();
        let result = tool
            .execute(serde_json::json!({ "path": "/nonexistent/xyz/abc" }))
            .await
            .unwrap();
        assert!(result["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_execute_single_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("code.py");
        fs::write(&file, "def foo():\n    pass\n").unwrap();

        let tool = EstimateTokensTool::new();
        let result = tool
            .execute(serde_json::json!({ "path": file.to_str().unwrap() }))
            .await
            .unwrap();

        assert_eq!(result["file_count"], 1);
        assert_eq!(result["line_count"], 2);
        assert_eq!(result["estimated_tokens"], 6);
    }
}

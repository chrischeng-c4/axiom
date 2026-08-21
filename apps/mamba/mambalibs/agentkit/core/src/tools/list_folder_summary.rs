//! ListFolderSummaryTool — summarizes directory structure, file counts, and line counts.
//!
//! Provides a high-level structural view of a directory up to a given depth,
//! accumulating file counts and line counts per subtree without reading full
//! file contents. Used by [`RestructureCodebaseAgent`] to analyse component size
//! before committing to a grouping.
//!
//! [`RestructureCodebaseAgent`]: crate::agents::restructure_codebase::RestructureCodebaseAgent

use crate::error::NovaResult;
use crate::tools::tool::{Tool, ToolParameter};
use async_trait::async_trait;
use serde::Deserialize;
use std::io::BufRead as _;
use std::path::Path;

/// Tool that returns a folder tree summary up to a given depth.
pub struct ListFolderSummaryTool;

impl ListFolderSummaryTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListFolderSummaryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ListFolderSummaryArgs {
    path: String,
    depth: Option<u32>,
}

/// A single node in the folder summary tree.
#[derive(Debug, serde::Serialize)]
struct TreeNode {
    name: String,
    path: String,
    is_dir: bool,
    file_count: u64,
    line_count: u64,
    children: Vec<TreeNode>,
}

#[async_trait]
impl Tool for ListFolderSummaryTool {
    fn name(&self) -> &str {
        "list_folder_summary"
    }

    fn description(&self) -> &str {
        "Return a folder tree summary for the given path up to the specified depth. \
         Reports file count and line count per directory node to enable token \
         budget estimation without reading full file contents."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                description: "Directory path to summarize".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "depth".to_string(),
                description: "Maximum depth to traverse (default: 2)".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: ListFolderSummaryArgs = serde_json::from_value(arguments)?;
        let depth = args.depth.unwrap_or(2);
        let base = Path::new(&args.path);

        if !base.exists() {
            return Ok(serde_json::json!({
                "path": args.path,
                "error": format!("Path does not exist: {}", args.path)
            }));
        }

        if !base.is_dir() {
            let line_count = count_lines(base);
            return Ok(serde_json::json!({
                "path": args.path,
                "is_dir": false,
                "file_count": 1,
                "line_count": line_count
            }));
        }

        let node = build_tree(base, 0, depth);
        let total_files = node.file_count;
        let total_lines = node.line_count;

        Ok(serde_json::json!({
            "path": args.path,
            "depth": depth,
            "total_file_count": total_files,
            "total_line_count": total_lines,
            "tree": node
        }))
    }
}

// ============================================================
// Tree builder
// ============================================================

/// Recursively build a `TreeNode` for the given path up to `max_depth`.
///
/// At `max_depth`, directories are still counted (file_count / line_count)
/// but children are not expanded.
fn build_tree(path: &Path, current_depth: u32, max_depth: u32) -> TreeNode {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let path_str = path.to_string_lossy().to_string();

    if path.is_file() {
        let lines = count_lines(path);
        return TreeNode {
            name,
            path: path_str,
            is_dir: false,
            file_count: 1,
            line_count: lines,
            children: vec![],
        };
    }

    // Directory
    let mut children: Vec<TreeNode> = Vec::new();
    let mut total_files: u64 = 0;
    let mut total_lines: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
        let mut sorted_entries: Vec<_> = entries.flatten().collect();
        sorted_entries.sort_by_key(|e| e.file_name());

        for entry in sorted_entries {
            let entry_path = entry.path();

            // Skip hidden files/dirs (dot-prefixed) except at root
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            if fname_str.starts_with('.') {
                continue;
            }

            if current_depth < max_depth {
                let child = build_tree(&entry_path, current_depth + 1, max_depth);
                total_files += child.file_count;
                total_lines += child.line_count;
                children.push(child);
            } else {
                // At max depth: count recursively but don't expand
                let (fc, lc) = count_recursive(&entry_path);
                total_files += fc;
                total_lines += lc;
            }
        }
    }

    TreeNode {
        name,
        path: path_str,
        is_dir: true,
        file_count: total_files,
        line_count: total_lines,
        children,
    }
}

/// Count files and lines recursively without building a tree structure.
fn count_recursive(path: &Path) -> (u64, u64) {
    if path.is_file() {
        return (1, count_lines(path));
    }

    let mut files: u64 = 0;
    let mut lines: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            if fname_str.starts_with('.') {
                continue;
            }
            let (f, l) = count_recursive(&entry_path);
            files += f;
            lines += l;
        }
    }

    (files, lines)
}

/// Count the number of lines in a file efficiently using a buffered reader.
/// Returns 0 for binary or unreadable files.
fn count_lines(path: &Path) -> u64 {
    let Ok(file) = std::fs::File::open(path) else {
        return 0;
    };
    let reader = std::io::BufReader::new(file);
    reader.lines().count() as u64
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_temp_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // root/
        //   src/
        //     main.rs      (3 lines)
        //     lib.rs       (2 lines)
        //   README.md      (1 line)
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    println!();\n}\n",
        )
        .unwrap();
        fs::write(root.join("src/lib.rs"), "pub mod foo;\n// comment\n").unwrap();
        fs::write(root.join("README.md"), "# Hello\n").unwrap();

        dir
    }

    #[test]
    fn test_count_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rs");
        fs::write(&path, "line1\nline2\nline3\n").unwrap();
        assert_eq!(count_lines(&path), 3);
    }

    #[test]
    fn test_count_recursive() {
        let dir = make_temp_tree();
        let (files, lines) = count_recursive(dir.path());
        assert_eq!(files, 3);
        // main.rs=3, lib.rs=2, README.md=1
        assert_eq!(lines, 6);
    }

    #[test]
    fn test_build_tree_depth_1() {
        let dir = make_temp_tree();
        let node = build_tree(dir.path(), 0, 1);
        assert!(node.is_dir);
        assert_eq!(node.file_count, 3);
        assert_eq!(node.line_count, 6);
        // At depth 1 we have src/ and README.md as children
        assert_eq!(node.children.len(), 2);
        let src = node.children.iter().find(|c| c.name == "src").unwrap();
        assert!(src.is_dir);
        // src/ children not expanded at depth 1
        assert!(src.children.is_empty());
        assert_eq!(src.file_count, 2);
    }

    #[test]
    fn test_build_tree_depth_2() {
        let dir = make_temp_tree();
        let node = build_tree(dir.path(), 0, 2);
        let src = node.children.iter().find(|c| c.name == "src").unwrap();
        // At depth 2, src children ARE expanded
        assert!(!src.children.is_empty());
        assert_eq!(src.children.len(), 2);
    }

    #[tokio::test]
    async fn test_execute_returns_summary() {
        let dir = make_temp_tree();
        let tool = ListFolderSummaryTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": dir.path().to_str().unwrap(),
                "depth": 2
            }))
            .await
            .unwrap();

        assert_eq!(result["total_file_count"], 3);
        assert_eq!(result["total_line_count"], 6);
        assert_eq!(result["depth"], 2);
    }

    #[tokio::test]
    async fn test_execute_nonexistent_path() {
        let tool = ListFolderSummaryTool::new();
        let result = tool
            .execute(serde_json::json!({ "path": "/nonexistent/path/xyz" }))
            .await
            .unwrap();
        assert!(result["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_execute_file_path() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "a\nb\nc\n").unwrap();

        let tool = ListFolderSummaryTool::new();
        let result = tool
            .execute(serde_json::json!({ "path": file_path.to_str().unwrap() }))
            .await
            .unwrap();

        assert_eq!(result["is_dir"], false);
        assert_eq!(result["file_count"], 1);
        assert_eq!(result["line_count"], 3);
    }
}

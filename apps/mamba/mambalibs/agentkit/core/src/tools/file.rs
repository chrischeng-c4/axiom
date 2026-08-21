use crate::error::{NovaError, NovaResult};
use crate::tools::tool::{Tool, ToolParameter};
use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Tool for reading file contents
pub struct ReadFileTool {
    max_file_size: usize,
    default_line_limit: usize,
}

impl ReadFileTool {
    pub fn new() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024,
            default_line_limit: 2000,
        }
    }

    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    async fn read_file_impl(&self, args: ReadFileArgs) -> NovaResult<serde_json::Value> {
        let path = PathBuf::from(&args.file_path);

        if !path.exists() {
            return Err(NovaError::FileNotFound(args.file_path));
        }

        let metadata = tokio::fs::metadata(&path).await?;
        if metadata.len() > self.max_file_size as u64 {
            return Err(NovaError::InvalidArguments(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                self.max_file_size
            )));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let lines: Vec<&str> = content.lines().collect();

        let offset = args.offset.unwrap_or(0);
        let limit = args.limit.unwrap_or(self.default_line_limit);

        let start = offset.min(lines.len());
        let end = (start + limit).min(lines.len());
        let selected_lines = &lines[start..end];

        let formatted: String = selected_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line_num = start + i + 1;
                format!("{:>6}\t{}", line_num, line)
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(serde_json::json!({
            "content": formatted,
            "total_lines": lines.len(),
            "offset": start,
            "limit": end - start,
            "truncated": end < lines.len()
        }))
    }
}

impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ReadFileArgs {
    file_path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns content with line numbers."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "file_path".to_string(),
                description: "The absolute path to the file to read".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "offset".to_string(),
                description: "Line number to start reading from (0-indexed)".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
            ToolParameter {
                name: "limit".to_string(),
                description: "Maximum number of lines to read".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: ReadFileArgs = serde_json::from_value(arguments)?;
        self.read_file_impl(args).await
    }
}

/// Tool for writing file contents
pub struct WriteFileTool;

impl WriteFileTool {
    pub fn new() -> Self {
        Self
    }

    async fn write_file_impl(&self, args: WriteFileArgs) -> NovaResult<serde_json::Value> {
        let path = PathBuf::from(&args.file_path);

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, &args.content).await?;

        let lines = args.content.lines().count();
        let bytes = args.content.len();

        Ok(serde_json::json!({
            "success": true,
            "file_path": args.file_path,
            "lines_written": lines,
            "bytes_written": bytes
        }))
    }
}

impl Default for WriteFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct WriteFileArgs {
    file_path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates the file if it doesn't exist."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "file_path".to_string(),
                description: "The absolute path to the file to write".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "content".to_string(),
                description: "The content to write to the file".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: WriteFileArgs = serde_json::from_value(arguments)?;
        self.write_file_impl(args).await
    }
}

/// Tool for editing files with precise text replacement
pub struct EditFileTool;

impl EditFileTool {
    pub fn new() -> Self {
        Self
    }

    async fn edit_file_impl(&self, args: EditFileArgs) -> NovaResult<serde_json::Value> {
        let path = PathBuf::from(&args.file_path);

        if !path.exists() {
            return Err(NovaError::FileNotFound(args.file_path));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let occurrences = content.matches(&args.old_string).count();

        if occurrences == 0 {
            return Err(NovaError::EditFailed(format!(
                "old_string not found in file: '{}'",
                if args.old_string.len() > 50 {
                    format!("{}...", &args.old_string[..50])
                } else {
                    args.old_string.clone()
                }
            )));
        }

        if !args.replace_all.unwrap_or(false) && occurrences > 1 {
            return Err(NovaError::EditFailed(format!(
                "old_string found {} times. Use replace_all=true or provide more context.",
                occurrences
            )));
        }

        let new_content = if args.replace_all.unwrap_or(false) {
            content.replace(&args.old_string, &args.new_string)
        } else {
            content.replacen(&args.old_string, &args.new_string, 1)
        };

        tokio::fs::write(&path, &new_content).await?;

        Ok(serde_json::json!({
            "success": true,
            "file_path": args.file_path,
            "replacements": if args.replace_all.unwrap_or(false) { occurrences } else { 1 }
        }))
    }
}

impl Default for EditFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct EditFileArgs {
    file_path: String,
    old_string: String,
    new_string: String,
    replace_all: Option<bool>,
}

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing old_string with new_string."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "file_path".to_string(),
                description: "The absolute path to the file to edit".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "old_string".to_string(),
                description: "The exact text to replace".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "new_string".to_string(),
                description: "The text to replace old_string with".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "replace_all".to_string(),
                description: "If true, replace all occurrences".to_string(),
                required: false,
                parameter_type: "boolean".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: EditFileArgs = serde_json::from_value(arguments)?;
        self.edit_file_impl(args).await
    }
}

/// Tool for finding files matching a glob pattern
pub struct GlobTool {
    max_results: usize,
}

impl GlobTool {
    pub fn new() -> Self {
        Self { max_results: 1000 }
    }

    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    async fn glob_impl(&self, args: GlobArgs) -> NovaResult<serde_json::Value> {
        let base_path = args.path.as_deref().unwrap_or(".");
        let pattern = &args.pattern;

        let glob_pattern =
            glob::Pattern::new(pattern).map_err(|e| NovaError::PatternError(e.to_string()))?;

        let mut matches = Vec::new();
        let walker = WalkDir::new(base_path).follow_links(false);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if matches.len() >= self.max_results {
                break;
            }

            let path = entry.path();
            if let Some(path_str) = path.to_str() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if glob_pattern.matches(file_name) || glob_pattern.matches(path_str) {
                    matches.push(path_str.to_string());
                }
            }
        }

        matches.sort();

        Ok(serde_json::json!({
            "matches": matches,
            "count": matches.len(),
            "truncated": matches.len() >= self.max_results
        }))
    }
}

impl Default for GlobTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct GlobArgs {
    pattern: String,
    path: Option<String>,
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "pattern".to_string(),
                description: "The glob pattern to match files against".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "path".to_string(),
                description: "The directory to search in".to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: GlobArgs = serde_json::from_value(arguments)?;
        self.glob_impl(args).await
    }
}

/// Tool for searching file contents with regex
pub struct GrepTool {
    max_matches: usize,
    default_context: usize,
}

impl GrepTool {
    pub fn new() -> Self {
        Self {
            max_matches: 500,
            default_context: 2,
        }
    }

    pub fn with_max_matches(mut self, max: usize) -> Self {
        self.max_matches = max;
        self
    }

    async fn grep_impl(&self, args: GrepArgs) -> NovaResult<serde_json::Value> {
        let base_path = args.path.as_deref().unwrap_or(".");
        let regex =
            Regex::new(&args.pattern).map_err(|e| NovaError::PatternError(e.to_string()))?;

        let context = args.context.unwrap_or(self.default_context);
        let mut matches = Vec::new();

        let walker = WalkDir::new(base_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        let type_filter = args.file_type.as_ref();

        for entry in walker {
            if matches.len() >= self.max_matches {
                break;
            }

            let path = entry.path();

            if let Some(filter) = type_filter {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !self.matches_file_type(ext, filter) {
                    continue;
                }
            }

            if let Ok(content) = tokio::fs::read_to_string(path).await {
                let lines: Vec<&str> = content.lines().collect();

                for (i, line) in lines.iter().enumerate() {
                    if regex.is_match(line) {
                        let start = i.saturating_sub(context);
                        let end = (i + context + 1).min(lines.len());

                        let context_lines: Vec<_> = lines[start..end]
                            .iter()
                            .enumerate()
                            .map(|(j, l)| {
                                let line_num = start + j + 1;
                                let is_match = start + j == i;
                                serde_json::json!({
                                    "line_number": line_num,
                                    "content": l,
                                    "is_match": is_match
                                })
                            })
                            .collect();

                        matches.push(serde_json::json!({
                            "file": path.to_string_lossy(),
                            "line_number": i + 1,
                            "content": line,
                            "context": context_lines
                        }));

                        if matches.len() >= self.max_matches {
                            break;
                        }
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "matches": matches,
            "count": matches.len(),
            "truncated": matches.len() >= self.max_matches
        }))
    }

    fn matches_file_type(&self, ext: &str, filter: &str) -> bool {
        match filter {
            "rs" | "rust" => ext == "rs",
            "ts" | "typescript" => ext == "ts" || ext == "tsx",
            "js" | "javascript" => ext == "js" || ext == "jsx",
            "py" | "python" => ext == "py",
            "go" => ext == "go",
            "java" => ext == "java",
            "c" | "cpp" => ext == "c" || ext == "cpp" || ext == "h" || ext == "hpp",
            _ => ext == filter,
        }
    }
}

impl Default for GrepTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct GrepArgs {
    pattern: String,
    path: Option<String>,
    file_type: Option<String>,
    context: Option<usize>,
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using a regex pattern."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "pattern".to_string(),
                description: "The regex pattern to search for".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "path".to_string(),
                description: "The directory to search in".to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "file_type".to_string(),
                description: "Filter by file type (e.g., 'rs', 'py', 'ts')".to_string(),
                required: false,
                parameter_type: "string".to_string(),
            },
            ToolParameter {
                name: "context".to_string(),
                description: "Number of context lines".to_string(),
                required: false,
                parameter_type: "integer".to_string(),
            },
        ]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: GrepArgs = serde_json::from_value(arguments)?;
        self.grep_impl(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_read_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "line 1\nline 2\nline 3").unwrap();

        let tool = ReadFileTool::new();
        let result = tool
            .execute(serde_json::json!({
                "file_path": file_path.to_string_lossy()
            }))
            .await
            .unwrap();

        assert_eq!(result["total_lines"], 3);
    }

    #[tokio::test]
    async fn test_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("new_file.txt");

        let tool = WriteFileTool::new();
        let result = tool
            .execute(serde_json::json!({
                "file_path": file_path.to_string_lossy(),
                "content": "Hello, world!"
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_edit_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("edit.txt");
        std::fs::write(&file_path, "Hello, world!").unwrap();

        let tool = EditFileTool::new();
        let result = tool
            .execute(serde_json::json!({
                "file_path": file_path.to_string_lossy(),
                "old_string": "world",
                "new_string": "Rust"
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, Rust!");
    }
}

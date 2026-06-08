---
id: projects-sdd-src-tools-analyze-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Analyze-code tool TDs support brownfield semantic coverage and standardization readiness."
---

# Standardized projects/agentic-workflow/src/tools/analyze/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/analyze/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AnalysisResult` | projects/agentic-workflow/src/tools/analyze/mod.rs | struct | pub | 98 |  |
| `ClassInfo` | projects/agentic-workflow/src/tools/analyze/mod.rs | struct | pub | 79 |  |
| `FieldInfo` | projects/agentic-workflow/src/tools/analyze/mod.rs | struct | pub | 90 |  |
| `FunctionInfo` | projects/agentic-workflow/src/tools/analyze/mod.rs | struct | pub | 58 |  |
| `ParamInfo` | projects/agentic-workflow/src/tools/analyze/mod.rs | struct | pub | 70 |  |
| `definition` | projects/agentic-workflow/src/tools/analyze/mod.rs | function | pub | 21 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/analyze/mod.rs | function | pub | 106 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
mod python;
mod rust_lang;
mod suggestions;
mod typescript;

use super::{get_required_array, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for analyze_code_for_spec
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_analyze_code_for_spec".to_string(),
        description: "Analyze code files with tree-sitter and suggest spec structure for code generation. Extracts functions, classes, and patterns to recommend spec_type, diagrams, and requirements.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "file_paths", "analysis_type"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "file_paths": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of file paths to analyze (relative to project root)"
                },
                "analysis_type": {
                    "type": "string",
                    "enum": ["api", "data-model", "module", "auto"],
                    "description": "Type of analysis: api (HTTP endpoints), data-model (schemas/classes), module (general), auto (detect automatically)"
                },
                "quick": {
                    "type": "boolean",
                    "default": false,
                    "description": "Fast-path: return AST-only analysis without LLM enrichment prompts"
                }
            }
        }),
    }
}

// Shared types used across language-specific modules

/// Extracted function information
#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub return_type: Option<String>,
    pub decorators: Vec<String>,
    pub is_async: bool,
    pub doc: Option<String>,
}

/// Extracted parameter information
#[derive(Debug, Clone)]
pub(crate) struct ParamInfo {
    pub name: String,
    pub type_annotation: Option<String>,
}

/// Extracted class information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ClassInfo {
    pub name: String,
    pub bases: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<FunctionInfo>,
    pub doc: Option<String>,
}

/// Extracted field information
#[derive(Debug, Clone)]
pub(crate) struct FieldInfo {
    pub name: String,
    pub type_annotation: Option<String>,
}

/// Analysis result from a single language parser
#[derive(Debug)]
pub(crate) struct AnalysisResult {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub detected_patterns: Vec<String>,
}

/// Execute the analyze_code_for_spec tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let file_paths = get_required_array(args, "file_paths")?;
    let analysis_type = get_required_string(args, "analysis_type")?;
    let quick = args.get("quick").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut all_functions: Vec<FunctionInfo> = Vec::new();
    let mut all_classes: Vec<ClassInfo> = Vec::new();
    let mut detected_patterns: Vec<String> = Vec::new();
    let mut skipped_files: Vec<Value> = Vec::new();

    for file_value in &file_paths {
        let file_path_str = file_value
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("file_paths must contain strings"))?;

        let full_path = project_root.join(file_path_str);

        if !full_path.exists() {
            skipped_files.push(json!({"path": file_path_str, "reason": "file_not_found"}));
            continue;
        }

        let source = std::fs::read_to_string(&full_path)?;
        let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let result = match ext {
            "py" => python::analyze(&source)?,
            "ts" | "tsx" | "js" | "jsx" => typescript::analyze(&source)?,
            "rs" => rust_lang::analyze(&source)?,
            _ => {
                skipped_files.push(json!({
                    "path": file_path_str,
                    "reason": format!("unsupported_extension: .{}", ext)
                }));
                continue;
            }
        };

        all_functions.extend(result.functions);
        all_classes.extend(result.classes);
        detected_patterns.extend(result.detected_patterns);
    }

    let suggested_spec_type = if analysis_type == "auto" {
        suggestions::detect_spec_type(&all_functions, &all_classes, &detected_patterns)
    } else {
        analysis_type.clone()
    };

    let mut output = suggestions::generate_suggestions(
        &suggested_spec_type,
        &all_functions,
        &all_classes,
        &detected_patterns,
    );

    if !skipped_files.is_empty() {
        output["skipped_files"] = json!(skipped_files);
    }

    // Add diagram inputs and enrichment prompt when not in quick mode
    if !quick {
        output["diagram_inputs"] = suggestions::generate_diagram_inputs(
            &suggested_spec_type,
            &all_functions,
            &all_classes,
        );
        output["enrichment_prompt"] = json!(suggestions::generate_enrichment_prompt(
            &suggested_spec_type,
            &all_functions,
            &all_classes
        ));
    }

    output["quick"] = json!(quick);

    Ok(serde_json::to_string_pretty(&output)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_python_api() {
        let source = r#"
from fastapi import APIRouter

router = APIRouter()

@router.get("/users/{user_id}")
async def get_user(user_id: int) -> User:
    """Get a user by ID."""
    pass

class User(BaseModel):
    id: int
    name: str
    email: str
"#;
        let result = python::analyze(source).unwrap();

        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "get_user");
        assert!(result.functions[0].is_async);

        assert_eq!(result.classes.len(), 1);
        assert_eq!(result.classes[0].name, "User");

        assert!(result.detected_patterns.contains(&"http-api".to_string()));
        assert!(result.detected_patterns.contains(&"data-model".to_string()));
    }

    #[test]
    fn test_analyze_python_data_model() {
        let source = r#"
from pydantic import BaseModel

class Order(BaseModel):
    id: int
    user_id: int
    items: list[OrderItem]
    total: float

class OrderItem(BaseModel):
    product_id: int
    quantity: int
    price: float
"#;
        let result = python::analyze(source).unwrap();

        assert_eq!(result.classes.len(), 2);
        assert!(result.detected_patterns.contains(&"data-model".to_string()));
    }

    #[test]
    fn test_detect_spec_type() {
        let patterns = vec!["http-api".to_string()];
        assert_eq!(
            suggestions::detect_spec_type(&[], &[], &patterns),
            "http-api"
        );

        let patterns = vec!["data-model".to_string()];
        assert_eq!(
            suggestions::detect_spec_type(&[], &[], &patterns),
            "data-model"
        );

        let classes = vec![
            ClassInfo {
                name: "A".to_string(),
                bases: vec![],
                fields: vec![],
                methods: vec![],
                doc: None,
            },
            ClassInfo {
                name: "B".to_string(),
                bases: vec![],
                fields: vec![],
                methods: vec![],
                doc: None,
            },
        ];
        assert_eq!(
            suggestions::detect_spec_type(&[], &classes, &[]),
            "data-model"
        );
    }

    #[test]
    fn test_execute_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        std::fs::create_dir_all(project_root.join("sdd")).unwrap();

        let py_content = r#"
class UserService:
    async def create_user(self, data: dict) -> User:
        """Create a new user."""
        pass
"#;
        std::fs::write(project_root.join("service.py"), py_content).unwrap();

        let args = json!({
            "project_path": project_root.to_str().unwrap(),
            "file_paths": ["service.py"],
            "analysis_type": "auto"
        });

        let result = execute(&args, project_root).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert!(output.get("suggested_spec_type").is_some());
        assert!(output.get("extracted_functions").is_some());
    }

    #[test]
    fn test_quick_mode_skips_enrichment() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        std::fs::write(project_root.join("main.py"), "def hello(): pass").unwrap();

        let args = json!({
            "project_path": project_root.to_str().unwrap(),
            "file_paths": ["main.py"],
            "analysis_type": "auto",
            "quick": true
        });

        let result = execute(&args, project_root).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(output["quick"], true);
        assert!(output.get("enrichment_prompt").is_none());
        assert!(output.get("diagram_inputs").is_none());
    }

    #[test]
    fn test_full_mode_includes_enrichment() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        std::fs::write(project_root.join("main.py"), "def hello(): pass").unwrap();

        let args = json!({
            "project_path": project_root.to_str().unwrap(),
            "file_paths": ["main.py"],
            "analysis_type": "auto",
            "quick": false
        });

        let result = execute(&args, project_root).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(output["quick"], false);
        assert!(output.get("enrichment_prompt").is_some());
        assert!(output.get("diagram_inputs").is_some());
    }

    #[test]
    fn test_skipped_files_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        std::fs::write(project_root.join("readme.txt"), "Hello").unwrap();

        let args = json!({
            "project_path": project_root.to_str().unwrap(),
            "file_paths": ["readme.txt", "nonexistent.py"],
            "analysis_type": "auto",
            "quick": true
        });

        let result = execute(&args, project_root).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        let skipped = output["skipped_files"].as_array().unwrap();
        assert_eq!(skipped.len(), 2);
        assert_eq!(skipped[0]["reason"], "unsupported_extension: .txt");
        assert_eq!(skipped[1]["reason"], "file_not_found");
    }

    #[test]
    fn test_analyze_typescript() {
        let source = r#"
interface User {
    id: number;
    name: string;
}

class UserService {
    async getUser(id: number): Promise<User> {
        return {} as User;
    }
}
"#;
        let result = typescript::analyze(source).unwrap();

        assert!(!result.classes.is_empty());
        // Interface detected as class + data-model pattern
        assert!(result.classes.iter().any(|c| c.name == "User"));
        assert!(result.classes.iter().any(|c| c.name == "UserService"));
        assert!(result.detected_patterns.contains(&"data-model".to_string()));
    }

    #[test]
    fn test_analyze_rust() {
        let source = r#"
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub port: u16,
}

pub fn create_config(name: &str) -> Config {
    Config { name: name.to_string(), port: 8080 }
}

pub async fn load_config(path: &str) -> Config {
    todo!()
}
"#;
        let result = rust_lang::analyze(source).unwrap();

        assert!(result.classes.iter().any(|c| c.name == "Config"));
        assert!(result.functions.iter().any(|f| f.name == "create_config"));
        assert!(result
            .functions
            .iter()
            .any(|f| f.name == "load_config" && f.is_async));
        assert!(result.detected_patterns.contains(&"data-model".to_string()));
    }

    #[test]
    fn test_multi_language_execution() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        std::fs::write(project_root.join("app.py"), "def hello(): pass").unwrap();
        std::fs::write(project_root.join("app.ts"), "function greet() {}").unwrap();
        std::fs::write(project_root.join("app.rs"), "fn main() {}").unwrap();

        let args = json!({
            "project_path": project_root.to_str().unwrap(),
            "file_paths": ["app.py", "app.ts", "app.rs"],
            "analysis_type": "auto",
            "quick": true
        });

        let result = execute(&args, project_root).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        let funcs = output["extracted_functions"].as_array().unwrap();
        assert!(
            funcs.len() >= 3,
            "Should extract functions from all 3 languages"
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/analyze/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects/agentic-workflow/tech-design/core/tools/analyze/mod.md#changes>"
    description: "Analyze-code-for-spec dispatcher, shared types, response assembly, and parser fixture tests."
```

//! ReadManifestTool — parses workspace manifest files.
//!
//! Reads known workspace manifest files (`Cargo.toml`, `package.json`,
//! `pyproject.toml`) at a given directory path, extracting top-level components
//! and workspace members for use by [`RestructureCodebaseAgent`].
//!
//! [`RestructureCodebaseAgent`]: crate::agents::restructure_codebase::RestructureCodebaseAgent

use crate::error::NovaResult;
use crate::tools::tool::{Tool, ToolParameter};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

/// Tool that reads workspace manifest files and extracts component information.
pub struct ReadManifestTool;

impl ReadManifestTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReadManifestTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ReadManifestArgs {
    path: String,
}

#[async_trait]
impl Tool for ReadManifestTool {
    fn name(&self) -> &str {
        "read_manifest"
    }

    fn description(&self) -> &str {
        "Read workspace manifest files (Cargo.toml, package.json, pyproject.toml) \
         at the given directory path. Returns workspace members and package \
         information to identify top-level codebase components."
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".to_string(),
            description: "Directory path to search for manifest files".to_string(),
            required: true,
            parameter_type: "string".to_string(),
        }]
    }

    async fn execute(&self, arguments: serde_json::Value) -> NovaResult<serde_json::Value> {
        let args: ReadManifestArgs = serde_json::from_value(arguments)?;
        let base = Path::new(&args.path);

        let mut manifests: Vec<serde_json::Value> = Vec::new();

        // --- Cargo.toml ---
        let cargo_path = base.join("Cargo.toml");
        if cargo_path.exists() {
            let content = tokio::fs::read_to_string(&cargo_path).await?;
            let entry = parse_cargo_toml(&content)
                .unwrap_or_else(|e| serde_json::json!({ "file": "Cargo.toml", "error": e }));
            manifests.push(entry);
        }

        // --- package.json ---
        let pkg_path = base.join("package.json");
        if pkg_path.exists() {
            let content = tokio::fs::read_to_string(&pkg_path).await?;
            let entry = parse_package_json(&content)
                .unwrap_or_else(|e| serde_json::json!({ "file": "package.json", "error": e }));
            manifests.push(entry);
        }

        // --- pyproject.toml ---
        let py_path = base.join("pyproject.toml");
        if py_path.exists() {
            let content = tokio::fs::read_to_string(&py_path).await?;
            let entry = parse_pyproject_toml(&content)
                .unwrap_or_else(|e| serde_json::json!({ "file": "pyproject.toml", "error": e }));
            manifests.push(entry);
        }

        if manifests.is_empty() {
            return Ok(serde_json::json!({
                "path": args.path,
                "manifests": [],
                "message": "No manifest files found (Cargo.toml, package.json, pyproject.toml). \
                            Proceed with directory structure analysis."
            }));
        }

        Ok(serde_json::json!({
            "path": args.path,
            "manifests": manifests
        }))
    }
}

// ============================================================
// Private parsers
// ============================================================

/// Parse `Cargo.toml` content and return a JSON summary.
fn parse_cargo_toml(content: &str) -> Result<serde_json::Value, String> {
    let parsed: toml::Value =
        toml::from_str(content).map_err(|e| format!("TOML parse error: {}", e))?;

    let package_name = parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());

    let mut workspace_members: Vec<String> = Vec::new();
    if let Some(workspace) = parsed.get("workspace") {
        if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(s) = member.as_str() {
                    workspace_members.push(s.to_string());
                }
            }
        }
    }

    let is_workspace = !workspace_members.is_empty();

    Ok(serde_json::json!({
        "file": "Cargo.toml",
        "type": "cargo",
        "package_name": package_name,
        "workspace_members": workspace_members,
        "is_workspace": is_workspace
    }))
}

/// Parse `package.json` content and return a JSON summary.
fn parse_package_json(content: &str) -> Result<serde_json::Value, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("JSON parse error: {}", e))?;

    let package_name = parsed
        .get("name")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());

    let mut workspace_members: Vec<String> = Vec::new();
    if let Some(ws) = parsed.get("workspaces") {
        // workspaces can be an array of globs or {"packages": [...]}
        if let Some(arr) = ws.as_array() {
            for w in arr {
                if let Some(s) = w.as_str() {
                    workspace_members.push(s.to_string());
                }
            }
        } else if let Some(packages) = ws.get("packages").and_then(|p| p.as_array()) {
            for p in packages {
                if let Some(s) = p.as_str() {
                    workspace_members.push(s.to_string());
                }
            }
        }
    }

    let is_workspace = !workspace_members.is_empty();

    Ok(serde_json::json!({
        "file": "package.json",
        "type": "npm",
        "package_name": package_name,
        "workspace_members": workspace_members,
        "is_workspace": is_workspace
    }))
}

/// Parse `pyproject.toml` content and return a JSON summary.
fn parse_pyproject_toml(content: &str) -> Result<serde_json::Value, String> {
    let parsed: toml::Value =
        toml::from_str(content).map_err(|e| format!("TOML parse error: {}", e))?;

    let package_name = parsed
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());

    // Python projects rarely have workspace members in pyproject.toml,
    // but hatch build targets are surfaced as sub-packages.
    let mut workspace_members: Vec<String> = Vec::new();
    if let Some(tool) = parsed.get("tool") {
        if let Some(hatch) = tool.get("hatch") {
            if let Some(build) = hatch.get("build") {
                if let Some(targets) = build.get("targets").and_then(|t| t.as_table()) {
                    for key in targets.keys() {
                        workspace_members.push(key.clone());
                    }
                }
            }
        }
    }

    Ok(serde_json::json!({
        "file": "pyproject.toml",
        "type": "python",
        "package_name": package_name,
        "workspace_members": workspace_members,
        "is_workspace": false
    }))
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_toml_workspace() {
        let content = r#"
[workspace]
members = ["crates/agent", "crates/cli"]

[workspace.dependencies]
serde = "1"
"#;
        let result = parse_cargo_toml(content).unwrap();
        assert_eq!(result["type"], "cargo");
        assert_eq!(result["is_workspace"], true);
        let members = result["workspace_members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0], "crates/agent");
        assert_eq!(members[1], "crates/cli");
    }

    #[test]
    fn test_parse_cargo_toml_package() {
        let content = r#"
[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"
"#;
        let result = parse_cargo_toml(content).unwrap();
        assert_eq!(result["package_name"], "my-crate");
        assert_eq!(result["is_workspace"], false);
        let members = result["workspace_members"].as_array().unwrap();
        assert!(members.is_empty());
    }

    #[test]
    fn test_parse_package_json_workspace() {
        let content = r#"{
  "name": "my-monorepo",
  "workspaces": ["packages/*", "apps/*"]
}"#;
        let result = parse_package_json(content).unwrap();
        assert_eq!(result["type"], "npm");
        assert_eq!(result["package_name"], "my-monorepo");
        assert_eq!(result["is_workspace"], true);
        let members = result["workspace_members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn test_parse_package_json_workspace_object() {
        let content = r#"{
  "name": "root",
  "workspaces": { "packages": ["packages/a", "packages/b"] }
}"#;
        let result = parse_package_json(content).unwrap();
        let members = result["workspace_members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn test_parse_pyproject_toml() {
        let content = r#"
[project]
name = "mylib"
version = "1.0.0"
"#;
        let result = parse_pyproject_toml(content).unwrap();
        assert_eq!(result["type"], "python");
        assert_eq!(result["package_name"], "mylib");
        assert_eq!(result["is_workspace"], false);
    }

    #[test]
    fn test_parse_cargo_toml_invalid() {
        let result = parse_cargo_toml("not valid toml !@#");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_package_json_invalid() {
        let result = parse_package_json("not valid json");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_no_manifests() {
        let tool = ReadManifestTool::new();
        let dir = tempfile::tempdir().unwrap();

        let result = tool
            .execute(serde_json::json!({ "path": dir.path() }))
            .await
            .unwrap();
        let manifests = result["manifests"].as_array().unwrap();
        assert!(manifests.is_empty());
        assert!(result["message"].as_str().is_some());
    }
}

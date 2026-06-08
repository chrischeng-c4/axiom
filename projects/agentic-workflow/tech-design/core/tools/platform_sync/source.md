---
id: sdd-tools-platform-sync-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue and platform-sync tool TDs expose AW Core workflow state through configured external clients."
---

# sdd tools platform sync source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/platform_sync.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/platform_sync.rs | function | pub | 16 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/platform_sync.rs | function | pub | 42 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
//! Platform Sync MCP Tool
//!
//! Syncs SDD change artifacts to external platforms.
//! Configuration is read from `.aw/config.toml` (preferred) or `.aw/config.yaml`.

use super::{get_required_string, ToolDefinition};
use crate::services::platform_sync::{PlatformSyncService, SyncStatus};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for platform_sync
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_platform_sync".to_string(),
        description: "Sync SDD change artifacts to external platforms (GitHub/GitLab). \
            Configuration is read from .aw/config.toml (preferred) or .aw/config.yaml, \
            with token from .env file. Labels (including crate:xxx scope labels) are auto-generated from config."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID to sync"
                }
            }
        }),
    }
}

/// Execute the platform_sync tool
pub async fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;

    // Create service and sync
    let service = PlatformSyncService::new(project_root.to_path_buf());

    // Load config to show in response
    let config = service.load_config()?;

    let result = service.sync(&change_id).await?;

    // Format response
    let status_emoji = match result.status {
        SyncStatus::Created => "✨",
        SyncStatus::Updated => "🔄",
        SyncStatus::Partial => "⚠️",
        SyncStatus::Error => "❌",
    };

    let mut response = format!(
        "{} {} - {}\n\n",
        status_emoji,
        format!("{:?}", result.status).to_uppercase(),
        result.message
    );

    if let Some(url) = &result.issue_url {
        response.push_str(&format!("**Issue URL**: {}\n", url));
    }

    if let Some(num) = result.issue_number {
        response.push_str(&format!("**Issue Number**: #{}\n", num));
    }

    // Show spec results if any
    if !result.spec_results.is_empty() {
        response.push_str("\n**Spec Issues**:\n");
        for spec_result in &result.spec_results {
            let spec_emoji = match spec_result.status {
                SyncStatus::Created => "✨",
                SyncStatus::Updated => "🔄",
                SyncStatus::Partial => "⚠️",
                SyncStatus::Error => "❌",
            };
            if let Some(num) = spec_result.issue_number {
                response.push_str(&format!(
                    "  {} #{} `{}`\n",
                    spec_emoji, num, spec_result.spec_id
                ));
            } else {
                response.push_str(&format!(
                    "  {} `{}` (failed)\n",
                    spec_emoji, spec_result.spec_id
                ));
            }
        }
    }

    response.push_str(&format!("\n**Platform**: {}\n", config.platform_type));
    response.push_str(&format!("**Repository**: {}\n", config.repo));

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_execute_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create cclab directory without config
        std::fs::create_dir_all(project_root.join(".aw/changes/test")).unwrap();

        let args = json!({
            "change_id": "test"
        });

        let result = execute(&args, project_root).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("config"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/platform_sync.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: "Platform sync MCP tool definition, execution, response formatting, and regression test."
```

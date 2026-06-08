// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/context.md#source
// CODEGEN-BEGIN
//! sdd_create_context MCP Tool
//!
//! Creates structured context artifact files (spec_context.md, knowledge_context.md,
//! codebase_context.md) for the v2 decide-change workflow.
//!
//! Each context type has its own structured JSON input — no more free-form content.

use super::{get_required_string, ToolDefinition};
use crate::models::context::{DocRef, FileRef, LensResult, PatternRef, SpecRef};
use crate::models::state::StatePhase;
use crate::services::reference_context_service::{
    CreateCodebaseContextInput, CreateContextInput, CreateKnowledgeContextInput,
    CreateSpecContextInput,
};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for sdd_create_context
/// @spec projects/agentic-workflow/tech-design/core/tools/context.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_create_context".to_string(),
        description: "Create a structured context artifact file for the v2 decide-change workflow. Each context_type requires type-specific structured fields instead of free-form content. Required per type: spec_context needs scanned_groups + specs; knowledge_context needs scanned_categories + docs; codebase_context needs lens_tools_used + files.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "context_type"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID (lowercase, hyphens allowed)"
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling: agent (via sdd_delegate_agent) or mainthread. Controls whether next dispatch hint is included in response."
                },
                "context_type": {
                    "type": "string",
                    "enum": ["spec_context", "knowledge_context", "codebase_context", "gap_codebase_spec", "gap_codebase_knowledge", "gap_spec_knowledge", "reference_context"],
                    "description": "Type of context artifact to create"
                },
                "content": {
                    "type": "string",
                    "description": "[gap_*] Free-form markdown content for gap analysis artifacts"
                },
                "complexity": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "critical"],
                    "description": "Complexity level of the change (default: high)"
                },
                "iteration": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Iteration number (default: 1)"
                },
                // Spec context fields
                "scanned_groups": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "[spec_context] Spec groups that were scanned (completeness proof)"
                },
                "specs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "relevance"],
                        "properties": {
                            "id": { "type": "string" },
                            "group": { "type": "string" },
                            "relevance": { "type": "string", "enum": ["high", "medium", "low"] },
                            "reason": { "type": "string" },
                            "key_sections": { "type": "array", "items": { "type": "string" } }
                        }
                    },
                    "description": "[spec_context] Relevant specs found"
                },
                "dependencies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "[spec_context] Dependencies between specs"
                },
                "gaps": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "[spec_context] Identified spec gaps"
                },
                // Knowledge context fields
                "scanned_categories": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "[knowledge_context] Knowledge categories scanned"
                },
                "docs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["path", "summary"],
                        "properties": {
                            "path": { "type": "string" },
                            "summary": { "type": "string" },
                            "relevant_sections": { "type": "array", "items": { "type": "string" } }
                        }
                    },
                    "description": "[knowledge_context] Relevant knowledge documents"
                },
                "patterns": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name", "source", "description"],
                        "properties": {
                            "name": { "type": "string" },
                            "source": { "type": "string" },
                            "description": { "type": "string" }
                        }
                    },
                    "description": "[knowledge_context] Patterns found in knowledge base"
                },
                "pitfalls": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "[knowledge_context] Known pitfalls"
                },
                // Codebase context fields
                "lens_tools_used": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "[codebase_context] Lens tools that were invoked"
                },
                "files": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["path"],
                        "properties": {
                            "path": { "type": "string" },
                            "symbols": { "type": "array", "items": { "type": "string" } },
                            "role": { "type": "string" }
                        }
                    },
                    "description": "[codebase_context] Analyzed files with symbols and roles"
                },
                "lens_results": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["tool", "query", "summary"],
                        "properties": {
                            "tool": { "type": "string" },
                            "query": { "type": "string" },
                            "summary": { "type": "string" }
                        }
                    },
                    "description": "[codebase_context] Results from Lens tool invocations"
                },
                "dependency_graph": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "[codebase_context] Dependency relationships between files/modules"
                }
            }
        }),
    }
}

/// Execute the sdd_create_context tool
/// @spec projects/agentic-workflow/tech-design/core/tools/context.md#source
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let caller = args
        .get("caller")
        .and_then(|v| v.as_str())
        .unwrap_or("mainthread");
    let change_id = get_required_string(args, "change_id")?;
    let context_type = get_required_string(args, "context_type")?;
    let complexity = args
        .get("complexity")
        .and_then(|v| v.as_str())
        .unwrap_or("high")
        .to_string();
    let iteration_u64 = args.get("iteration").and_then(|v| v.as_u64()).unwrap_or(1);
    let iteration = u32::try_from(iteration_u64).map_err(|_| {
        anyhow::anyhow!(
            "iteration {} is too large (max {})",
            iteration_u64,
            u32::MAX
        )
    })?;

    // Save for state update after artifact creation
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    let context_type_key = context_type.clone();

    let input = match context_type.as_str() {
        "spec_context" => CreateContextInput::Spec(CreateSpecContextInput {
            change_id,
            complexity,
            iteration,
            scanned_groups: parse_string_array(args, "scanned_groups")?,
            specs: parse_spec_refs(args)?,
            dependencies: parse_string_array(args, "dependencies")?,
            gaps: parse_string_array(args, "gaps")?,
        }),
        "knowledge_context" => CreateContextInput::Knowledge(CreateKnowledgeContextInput {
            change_id,
            complexity,
            iteration,
            scanned_categories: parse_string_array(args, "scanned_categories")?,
            docs: parse_doc_refs(args)?,
            patterns: parse_pattern_refs(args)?,
            pitfalls: parse_string_array(args, "pitfalls")?,
        }),
        "codebase_context" => CreateContextInput::Codebase(CreateCodebaseContextInput {
            change_id,
            complexity,
            iteration,
            lens_tools_used: parse_string_array(args, "lens_tools_used")?,
            files: parse_file_refs(args)?,
            lens_results: parse_lens_results(args)?,
            dependency_graph: parse_string_array(args, "dependency_graph")?,
        }),
        "gap_codebase_spec" | "gap_codebase_knowledge" | "gap_spec_knowledge" => {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("content is required for gap analysis context types"))?;
            CreateContextInput::Gap {
                change_id,
                context_type: context_type.clone(),
                content: content.to_string(),
            }
        }
        "reference_context" => {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("content is required for reference_context"))?;
            CreateContextInput::Gap {
                change_id,
                context_type: context_type.clone(),
                content: content.to_string(),
            }
        }
        _ => anyhow::bail!(
            "Invalid context_type '{}': must be 'spec_context', 'knowledge_context', 'codebase_context', 'gap_codebase_spec', 'gap_codebase_knowledge', 'gap_spec_knowledge', or 'reference_context'",
            context_type
        ),
    };

    let result = crate::services::reference_context_service::create_context(input, project_root)?;

    // Auto-update STATE.yaml phase based on context type and current phase
    // If currently in Reviewed state (needs revision), set to Revised + increment counter
    // Otherwise, set to Created (initial creation)
    {
        let mut sm = crate::state::StateManager::load(&change_dir)?;
        let (target_phase, is_revision) = match (context_type_key.as_str(), sm.phase()) {
            // Reference context (unified exploration) — absorbed by issue lifecycle,
            // phase stays at PostClarificationsCreated
            ("reference_context", StatePhase::ChangeInited) => (StatePhase::ChangeInited, false),
            ("reference_context", StatePhase::ChangeRejected) => (StatePhase::ChangeInited, true),
            ("reference_context", _) => (StatePhase::ChangeInited, false),
            // Legacy context types: no phase update needed
            _ => {
                sm.save()?;
                if caller == "agent" {
                    return Ok(result);
                } else {
                    return Ok(format!(
                        "{}\n\n→ Next: call `sdd_run_change` to continue.",
                        result
                    ));
                }
            }
        };
        sm.set_phase(target_phase)?;
        if is_revision {
            sm.increment_revision_count(context_type_key.as_str());
        }
        sm.save()?;
    }

    if caller == "agent" {
        Ok(result)
    } else {
        Ok(format!(
            "{}\n\n→ Next: call `sdd_run_change` to continue.",
            result
        ))
    }
}

// ---------------------------------------------------------------------------
// JSON parsing helpers
// ---------------------------------------------------------------------------

fn parse_string_array(args: &Value, field: &str) -> Result<Vec<String>> {
    let arr = match args.get(field).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let s = v
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("{}[{}]: expected string, got {}", field, i, v))?;
        result.push(s.to_string());
    }
    Ok(result)
}

fn parse_spec_refs(args: &Value) -> Result<Vec<SpecRef>> {
    let arr = match args.get("specs").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let id = v
            .get("id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow::anyhow!("specs[{}]: missing required field 'id' (string)", i))?
            .to_string();
        let relevance = v
            .get("relevance")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("specs[{}]: missing required field 'relevance' (string)", i)
            })?
            .to_string();
        result.push(SpecRef {
            id,
            group: v
                .get("group")
                .and_then(|g| g.as_str())
                .unwrap_or("")
                .to_string(),
            relevance,
            reason: v
                .get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string(),
            key_sections: parse_nested_string_array(v, "specs", i, "key_sections")?,
        });
    }
    Ok(result)
}

fn parse_doc_refs(args: &Value) -> Result<Vec<DocRef>> {
    let arr = match args.get("docs").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let path = v
            .get("path")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow::anyhow!("docs[{}]: missing required field 'path' (string)", i))?
            .to_string();
        let summary = v
            .get("summary")
            .and_then(|s| s.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("docs[{}]: missing required field 'summary' (string)", i)
            })?
            .to_string();
        result.push(DocRef {
            path,
            summary,
            relevant_sections: parse_nested_string_array(v, "docs", i, "relevant_sections")?,
        });
    }
    Ok(result)
}

fn parse_pattern_refs(args: &Value) -> Result<Vec<PatternRef>> {
    let arr = match args.get("patterns").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let name = v
            .get("name")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("patterns[{}]: missing required field 'name' (string)", i)
            })?
            .to_string();
        let source = v
            .get("source")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("patterns[{}]: missing required field 'source' (string)", i)
            })?
            .to_string();
        let description = v
            .get("description")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "patterns[{}]: missing required field 'description' (string)",
                    i
                )
            })?
            .to_string();
        result.push(PatternRef {
            name,
            source,
            description,
        });
    }
    Ok(result)
}

fn parse_file_refs(args: &Value) -> Result<Vec<FileRef>> {
    let arr = match args.get("files").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let path = v
            .get("path")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow::anyhow!("files[{}]: missing required field 'path' (string)", i))?
            .to_string();
        result.push(FileRef {
            path,
            symbols: parse_nested_string_array(v, "files", i, "symbols")?,
            role: v
                .get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    Ok(result)
}

fn parse_lens_results(args: &Value) -> Result<Vec<LensResult>> {
    let arr = match args.get("lens_results").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let tool = v
            .get("tool")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "lens_results[{}]: missing required field 'tool' (string)",
                    i
                )
            })?
            .to_string();
        let query = v
            .get("query")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "lens_results[{}]: missing required field 'query' (string)",
                    i
                )
            })?
            .to_string();
        let summary = v
            .get("summary")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "lens_results[{}]: missing required field 'summary' (string)",
                    i
                )
            })?
            .to_string();
        result.push(LensResult {
            tool,
            query,
            summary,
        });
    }
    Ok(result)
}

/// Parse a nested string array from a JSON object field
fn parse_nested_string_array(
    obj: &Value,
    parent: &str,
    idx: usize,
    field: &str,
) -> Result<Vec<String>> {
    let arr = match obj.get(field).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };
    let mut result = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let s = v.as_str().ok_or_else(|| {
            anyhow::anyhow!(
                "{}[{}].{}[{}]: expected string, got {}",
                parent,
                idx,
                field,
                i,
                v
            )
        })?;
        result.push(s.to_string());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_spec_context_structured() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        std::fs::create_dir_all(project_root.join(".aw/changes")).unwrap();
        crate::test_util::write_minimal_issue(project_root, "test-change");

        let args = json!({
            "change_id": "test-change",
            "context_type": "spec_context",
            "complexity": "high",
            "iteration": 1,
            "scanned_groups": ["sdd", "cli"],
            "specs": [{
                "id": "decide-change-workflow",
                "group": "sdd",
                "relevance": "high",
                "reason": "Directly modified",
                "key_sections": ["State Transitions"]
            }],
            "dependencies": ["decide-change → state-machine"],
            "gaps": ["Missing context schema spec"]
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("spec_context.md"));

        let file_path = project_root.join(".aw/changes/test-change/spec_context.md");
        let written = std::fs::read_to_string(&file_path).unwrap();
        assert!(written.contains("type: spec_context"));
        assert!(written.contains("stage: spec"));
        assert!(written.contains("scanned_groups:"));
        assert!(written.contains("  - sdd"));
        assert!(written.contains("**decide-change-workflow**"));
        assert!(written.contains("relevance: high"));
    }

    #[test]
    fn test_create_knowledge_context_structured() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        std::fs::create_dir_all(project_root.join(".aw/changes")).unwrap();
        crate::test_util::write_minimal_issue(project_root, "test-change");

        let args = json!({
            "change_id": "test-change",
            "context_type": "knowledge_context",
            "scanned_categories": ["architecture", "patterns"],
            "docs": [{
                "path": "00-architecture/01-overview.md",
                "summary": "Project overview",
                "relevant_sections": ["MCP Tools"]
            }],
            "patterns": [{
                "name": "Service pattern",
                "source": "knowledge/patterns.md",
                "description": "Input + validate + render"
            }],
            "pitfalls": ["Do not mix CLI and MCP"]
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("knowledge_context.md"));

        let file_path = project_root.join(".aw/changes/test-change/knowledge_context.md");
        let written = std::fs::read_to_string(&file_path).unwrap();
        assert!(written.contains("type: knowledge_context"));
        assert!(written.contains("scanned_categories:"));
        assert!(written.contains("## Patterns"));
        assert!(written.contains("## Pitfalls"));
    }

    #[test]
    fn test_create_codebase_context_structured() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        std::fs::create_dir_all(project_root.join(".aw/changes")).unwrap();
        crate::test_util::write_minimal_issue(project_root, "test-change");

        let args = json!({
            "change_id": "test-change",
            "context_type": "codebase_context",
            "lens_tools_used": ["lens_symbols", "lens_references"],
            "files": [{
                "path": "src/mcp/tools/context.rs",
                "symbols": ["execute", "definition"],
                "role": "MCP tool entry point"
            }],
            "lens_results": [{
                "tool": "lens_symbols",
                "query": "context",
                "summary": "Found 5 symbols"
            }],
            "dependency_graph": ["context.rs → context_service.rs"]
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("codebase_context.md"));

        let file_path = project_root.join(".aw/changes/test-change/codebase_context.md");
        let written = std::fs::read_to_string(&file_path).unwrap();
        assert!(written.contains("type: codebase_context"));
        assert!(written.contains("lens_tools_used:"));
        assert!(written.contains("## Analyzed Files"));
        assert!(written.contains("## Lens Results"));
    }

    #[test]
    fn test_invalid_context_type() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test",
            "context_type": "invalid_type",
            "scanned_groups": ["x"],
            "specs": [{"id": "x", "relevance": "high"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid context_type"));
    }

    #[test]
    fn test_missing_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // spec_context without scanned_groups → validation error
        let args = json!({
            "change_id": "test",
            "context_type": "spec_context",
            "specs": [{"id": "x", "relevance": "high"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("scanned_groups"));
    }

    #[test]
    fn test_malformed_spec_element() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // spec missing 'relevance' → parse error with index
        let args = json!({
            "change_id": "test",
            "context_type": "spec_context",
            "scanned_groups": ["sdd"],
            "specs": [{"id": "good", "relevance": "high"}, {"id": "bad"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("specs[1]"),
            "Error should pinpoint index: {}",
            err
        );
        assert!(
            err.contains("relevance"),
            "Error should name missing field: {}",
            err
        );
    }

    #[test]
    fn test_iteration_overflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "test",
            "context_type": "spec_context",
            "iteration": 5_000_000_000_u64,
            "scanned_groups": ["sdd"],
            "specs": [{"id": "x", "relevance": "high"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_malformed_string_array_element() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        std::fs::create_dir_all(project_root.join(".aw/changes")).unwrap();

        // scanned_groups with non-string element
        let args = json!({
            "change_id": "test",
            "context_type": "spec_context",
            "scanned_groups": ["sdd", 42],
            "specs": [{"id": "x", "relevance": "high"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("scanned_groups[1]"),
            "Error should pinpoint index: {}",
            err
        );
        assert!(
            err.contains("expected string"),
            "Error should describe type mismatch: {}",
            err
        );
    }

    #[test]
    fn test_malformed_nested_string_array_element() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        std::fs::create_dir_all(project_root.join(".aw/changes")).unwrap();

        // key_sections with non-string element
        let args = json!({
            "change_id": "test",
            "context_type": "spec_context",
            "scanned_groups": ["sdd"],
            "specs": [{"id": "x", "relevance": "high", "key_sections": ["ok", true]}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("specs[0].key_sections[1]"),
            "Error should pinpoint nested index: {}",
            err
        );
    }

    #[test]
    fn test_missing_doc_summary() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // docs element without summary
        let args = json!({
            "change_id": "test",
            "context_type": "knowledge_context",
            "scanned_categories": ["arch"],
            "docs": [{"path": "some/doc.md"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("docs[0]"),
            "Error should pinpoint index: {}",
            err
        );
        assert!(
            err.contains("summary"),
            "Error should name missing field: {}",
            err
        );
    }

    #[test]
    fn test_malformed_file_element() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // file missing 'path' → parse error with index
        let args = json!({
            "change_id": "test",
            "context_type": "codebase_context",
            "lens_tools_used": ["lens_symbols"],
            "files": [{"symbols": ["x"]}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("files[0]"),
            "Error should pinpoint index: {}",
            err
        );
        assert!(
            err.contains("path"),
            "Error should name missing field: {}",
            err
        );
    }
}
// CODEGEN-END

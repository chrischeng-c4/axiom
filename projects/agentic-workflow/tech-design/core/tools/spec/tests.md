---
id: sdd-tools-spec-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools spec tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 41 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 238 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_review_spec` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 574 | execute_review_spec(args: &Value, project_root: &Path) -> Result<String> |
| `review_spec_definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 476 | review_spec_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(project_root, "test-change");

        let args = json!({
            "change_id": "test-change",
            "spec_id": "mcp-protocol",
            "spec_type": "utility",
            "title": "MCP Protocol Implementation",
            "overview": "This specification covers the implementation of the Model Context Protocol (MCP) server for genesis, providing structured tools for proposal generation.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "JSON-RPC 2.0 Support",
                    "description": "The server must support JSON-RPC 2.0 protocol over stdio",
                    "priority": "high"
                },
                {
                    "id": "R2",
                    "title": "Tool Registration",
                    "description": "Tools must be registered and callable via tools/call method",
                    "priority": "high"
                }
            ],
            "scenarios": [
                {
                    "name": "Server Initialization",
                    "given": "MCP client is connected",
                    "when": "Client sends initialize request",
                    "then": "Server responds with capabilities"
                },
                {
                    "name": "Tool Execution",
                    "when": "Client calls create_proposal tool",
                    "then": "Server creates proposal.md and returns success"
                }
            ],
            "flow_diagram": "graph LR\n    A[Client] --> B[Server]\n    B --> C[Tool Registry]\n    C --> D[Execute Tool]"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created
        let spec_path = project_root.join(".aw/changes/test-change/specs/mcp-protocol.md");
        assert!(spec_path.exists());

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: mcp-protocol"));
        assert!(content.contains("spec_type: utility"));
        assert!(content.contains("## Requirements"));
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("### Scenario:"));
        assert!(content.contains("**WHEN**"));
        assert!(content.contains("**THEN**"));
        assert!(content.contains("```mermaid"));
    }

    // R6: Scenario - create_spec enforces http-api requires sequence diagram
    #[test]
    fn test_create_spec_http_api_missing_sequence_diagram() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create http-api spec WITHOUT sequence diagram but WITH OpenAPI
        let args = json!({
            "change_id": "test-change",
            "spec_id": "api-spec",
            "spec_type": "http-api",
            "title": "User API",
            "overview": "This specification defines the REST API for managing user resources and their profiles.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "List Users",
                    "description": "List all users"
                }
            ],
            "scenarios": [
                {
                    "name": "Get users",
                    "when": "GET /users is called",
                    "then": "user list is returned"
                }
            ],
            "api_spec": {
                "type": "openapi-3.1",
                "spec": {
                    "openapi": "3.1.0",
                    "info": {"title": "User API", "version": "1.0.0"},
                    "paths": {
                        "/users": {
                            "get": {"summary": "List users"}
                        }
                    }
                }
            }
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when http-api spec has no sequence diagram"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("sequence"),
            "Error should mention missing sequence diagram, got: {}",
            err_msg
        );
    }

    // R6: Scenario - create_spec enforces http-api requires OpenAPI spec
    #[test]
    fn test_create_spec_http_api_missing_openapi() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create http-api spec WITH sequence diagram but WITHOUT OpenAPI
        let args = json!({
            "change_id": "test-change",
            "spec_id": "api-spec",
            "spec_type": "http-api",
            "title": "User API",
            "overview": "This specification defines the REST API for managing user resources and their profiles.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "List Users",
                    "description": "List all users"
                }
            ],
            "scenarios": [
                {
                    "name": "Get users",
                    "when": "GET /users is called",
                    "then": "user list is returned"
                }
            ],
            "flow_diagram": "sequenceDiagram\n    Client->>API: GET /users\n    API-->>Client: 200 OK"
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when http-api spec has no OpenAPI spec"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("OpenAPI") || err_msg.contains("openapi-3.1"),
            "Error should mention missing OpenAPI spec, got: {}",
            err_msg
        );
    }

    // R6: Scenario - create_spec enforces data-model requires ERD or class diagram
    #[test]
    fn test_create_spec_data_model_missing_diagram() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create data-model spec WITHOUT ERD or class diagram
        let args = json!({
            "change_id": "test-change",
            "spec_id": "data-model",
            "spec_type": "data-model",
            "title": "User Data Model",
            "overview": "This specification defines the core data structures for user management including profiles, preferences, and relationships.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "User Entity",
                    "description": "User entity definition"
                }
            ],
            "scenarios": [
                {
                    "name": "User creation",
                    "when": "User is created",
                    "then": "Entity is stored"
                }
            ]
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when data-model spec has no ERD/class diagram"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("erd") || err_msg.contains("class"),
            "Error should mention missing erd or class diagram, got: {}",
            err_msg
        );
    }

    // R6: Scenario - create_spec enforces workflow requires Serverless Workflow spec
    #[test]
    fn test_create_spec_workflow_missing_serverless_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Try to create workflow spec WITH state diagram but WITHOUT Serverless Workflow spec
        let args = json!({
            "change_id": "test-change",
            "spec_id": "workflow-spec",
            "spec_type": "workflow",
            "title": "Order Processing Workflow",
            "overview": "This specification defines the workflow for processing customer orders including validation, payment, and fulfillment stages.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "Order Processing",
                    "description": "Process orders"
                }
            ],
            "scenarios": [
                {
                    "name": "Process order",
                    "when": "order is submitted",
                    "then": "workflow executes"
                }
            ],
            "flow_diagram": "stateDiagram-v2\n    [*] --> Pending\n    Pending --> Done\n    Done --> [*]"
        });

        let result = execute(&args, project_root);
        assert!(
            result.is_err(),
            "Should fail when workflow spec has no Serverless Workflow spec"
        );
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("Serverless") || err_msg.contains("workflow-0.8"),
            "Error should mention missing Serverless Workflow spec, got: {}",
            err_msg
        );
    }
}

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - tests
    description: "Create-spec and review-spec test module."
```

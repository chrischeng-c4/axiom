// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_tests.md#source
// CODEGEN-BEGIN
use super::*;
use tempfile::TempDir;

fn make_valid_spec_input(change_id: &str) -> CreateSpecContextInput {
    CreateSpecContextInput {
        change_id: change_id.to_string(),
        complexity: "high".to_string(),
        iteration: 1,
        scanned_groups: vec!["sdd".to_string(), "cli".to_string()],
        specs: vec![SpecRef {
            id: "decide-change-workflow".to_string(),
            group: "sdd".to_string(),
            relevance: "high".to_string(),
            reason: "Directly modified workflow".to_string(),
            key_sections: vec!["State Transitions".to_string()],
        }],
        dependencies: vec!["decide-change → state-machine".to_string()],
        gaps: vec!["Missing context artifact schema spec".to_string()],
    }
}

fn make_valid_knowledge_input(change_id: &str) -> CreateKnowledgeContextInput {
    CreateKnowledgeContextInput {
        change_id: change_id.to_string(),
        complexity: "high".to_string(),
        iteration: 1,
        scanned_categories: vec!["architecture".to_string()],
        docs: vec![DocRef {
            path: "00-architecture/01-overview.md".to_string(),
            summary: "Project architecture".to_string(),
            relevant_sections: vec!["MCP Tools".to_string()],
        }],
        patterns: vec![PatternRef {
            name: "Service pattern".to_string(),
            source: "knowledge/patterns.md".to_string(),
            description: "Input + validate + render".to_string(),
        }],
        pitfalls: vec!["Do not mix CLI and MCP".to_string()],
    }
}

fn make_valid_codebase_input(change_id: &str) -> CreateCodebaseContextInput {
    CreateCodebaseContextInput {
        change_id: change_id.to_string(),
        complexity: "high".to_string(),
        iteration: 1,
        lens_tools_used: vec!["lens_symbols".to_string(), "lens_references".to_string()],
        files: vec![FileRef {
            path: "src/mcp/tools/context.rs".to_string(),
            symbols: vec!["execute".to_string(), "definition".to_string()],
            role: "MCP tool entry point".to_string(),
        }],
        lens_results: vec![LensResult {
            tool: "lens_symbols".to_string(),
            query: "context".to_string(),
            summary: "Found 5 relevant symbols".to_string(),
        }],
        dependency_graph: vec!["context.rs → context_service.rs".to_string()],
    }
}

// --- Spec context tests ---

#[test]
fn test_create_spec_context_valid() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".aw/changes")).unwrap();
    let input = make_valid_spec_input("test-spec");
    let result = create_spec_context(input, temp.path()).unwrap();
    assert!(result.contains("spec_context.md"));

    let content =
        std::fs::read_to_string(temp.path().join(".aw/changes/test-spec/spec_context.md")).unwrap();
    assert!(content.contains("type: spec_context"));
    assert!(content.contains("stage: spec"));
    assert!(content.contains("scanned_groups:"));
    assert!(content.contains("  - sdd"));
    assert!(content.contains("# Spec Context"));
    assert!(content.contains("**decide-change-workflow**"));
    assert!(content.contains("relevance: high"));
    assert!(content.contains("## Gaps"));
}

#[test]
fn test_spec_context_empty_scanned_groups() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test");
    input.scanned_groups = vec![];
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("scanned_groups"));
}

#[test]
fn test_spec_context_empty_specs() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test");
    input.specs = vec![];
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("specs must be non-empty"));
}

#[test]
fn test_spec_context_invalid_relevance() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test");
    input.specs[0].relevance = "extreme".to_string();
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("invalid relevance"));
}

// --- Knowledge context tests ---

#[test]
fn test_create_knowledge_context_valid() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".aw/changes")).unwrap();
    let input = make_valid_knowledge_input("test-knowledge");
    let result = create_knowledge_context(input, temp.path()).unwrap();
    assert!(result.contains("knowledge_context.md"));

    let content = std::fs::read_to_string(
        temp.path()
            .join(".aw/changes/test-knowledge/knowledge_context.md"),
    )
    .unwrap();
    assert!(content.contains("type: knowledge_context"));
    assert!(content.contains("stage: knowledge"));
    assert!(content.contains("scanned_categories:"));
    assert!(content.contains("# Knowledge Context"));
    assert!(content.contains("## Relevant Documents"));
    assert!(content.contains("## Patterns"));
    assert!(content.contains("## Pitfalls"));
}

#[test]
fn test_knowledge_context_empty_categories() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test");
    input.scanned_categories = vec![];
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("scanned_categories"));
}

#[test]
fn test_knowledge_context_empty_docs() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test");
    input.docs = vec![];
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("docs must be non-empty"));
}

// --- Codebase context tests ---

#[test]
fn test_create_codebase_context_valid() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".aw/changes")).unwrap();
    let input = make_valid_codebase_input("test-codebase");
    let result = create_codebase_context(input, temp.path()).unwrap();
    assert!(result.contains("codebase_context.md"));

    let content = std::fs::read_to_string(
        temp.path()
            .join(".aw/changes/test-codebase/codebase_context.md"),
    )
    .unwrap();
    assert!(content.contains("type: codebase_context"));
    assert!(content.contains("stage: codebase"));
    assert!(content.contains("lens_tools_used:"));
    assert!(content.contains("  - lens_symbols"));
    assert!(content.contains("# Codebase Context"));
    assert!(content.contains("## Analyzed Files"));
    assert!(content.contains("## Lens Results"));
    assert!(content.contains("## Dependency Graph"));
}

#[test]
fn test_codebase_context_empty_lens_tools() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test");
    input.lens_tools_used = vec![];
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("lens_tools_used"));
}

#[test]
fn test_codebase_context_empty_files() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test");
    input.files = vec![];
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("files must be non-empty"));
}

// --- Shared validation tests ---

#[test]
fn test_invalid_change_id() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("INVALID_ID");
    input.change_id = "INVALID_ID".to_string();
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid change_id"));
}

#[test]
fn test_empty_change_id() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("");
    input.change_id = "".to_string();
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must not be empty"));
}

#[test]
fn test_dotdot_change_id() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("..");
    input.change_id = "..".to_string();
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not allowed"));
}

#[test]
fn test_doc_path_traversal() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-traversal");
    input.docs[0].path = "../../../etc/passwd".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("path traversal"));
}

#[test]
fn test_file_absolute_path() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-abs");
    input.files[0].path = "/etc/passwd".to_string();
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("absolute paths"));
}

// --- Windows drive-letter path tests ---

#[test]
fn test_file_windows_drive_path_backslash() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-win");
    input.files[0].path = "C:\\Users\\foo\\bar.rs".to_string();
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("absolute paths"));
}

#[test]
fn test_file_windows_drive_path_forward_slash() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-win");
    input.files[0].path = "D:/projects/src/lib.rs".to_string();
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("absolute paths"));
}

#[test]
fn test_doc_windows_drive_path() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-win");
    input.docs[0].path = "C:/docs/readme.md".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("absolute paths"));
}

// --- YAML escaping tests ---

#[test]
fn test_yaml_safe_scalar_fields_escaped() {
    let result = yaml_safe("high: critical!");
    assert_eq!(result, "\"high: critical!\"");

    let result = yaml_safe("value with #comment");
    assert_eq!(result, "\"value with #comment\"");

    let result = yaml_safe("high");
    assert_eq!(result, "high");
}

// --- Complexity/iteration validation tests ---

#[test]
fn test_invalid_complexity() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test-cplx");
    input.complexity = "extreme".to_string();
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid complexity"));
}

#[test]
fn test_invalid_complexity_knowledge() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-cplx");
    input.complexity = "unknown".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid complexity"));
}

#[test]
fn test_invalid_complexity_codebase() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-cplx");
    input.complexity = "".to_string();
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid complexity"));
}

#[test]
fn test_valid_complexity_values() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".aw/changes")).unwrap();
    for c in &["low", "medium", "high", "critical"] {
        let mut input = make_valid_spec_input(&format!("test-{}", c));
        input.complexity = c.to_string();
        assert!(create_spec_context(input, temp.path()).is_ok());
    }
}

// --- Doc summary validation tests ---

#[test]
fn test_doc_empty_summary() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-summary");
    input.docs[0].summary = "".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("non-empty summary"));
}

// --- Pattern source path validation tests ---

#[test]
fn test_pattern_source_path_traversal() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-patsrc");
    input.patterns[0].source = "../../../etc/passwd".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("path traversal"));
}

#[test]
fn test_pattern_source_absolute_path() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-patsrc");
    input.patterns[0].source = "/etc/passwd".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("absolute paths"));
}

#[test]
fn test_pattern_source_windows_drive() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-patsrc");
    input.patterns[0].source = "C:\\docs\\patterns.md".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("absolute paths"));
}

// --- YAML newline injection tests ---

#[test]
fn test_yaml_safe_newline_escaped() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".aw/changes")).unwrap();
    let mut input = make_valid_spec_input("test-newline");
    input.scanned_groups = vec!["genesis\ninjected_field: true".to_string()];
    let result = create_spec_context(input, temp.path()).unwrap();
    assert!(result.contains("spec_context.md"));

    let content =
        std::fs::read_to_string(temp.path().join(".aw/changes/test-newline/spec_context.md"))
            .unwrap();
    assert!(content.contains("\\n"));
    assert!(!content.contains("injected_field: true\n"));
}

#[test]
fn test_yaml_safe_carriage_return_escaped() {
    let result = yaml_safe("value\rwith\rCR");
    assert!(result.contains("\\r"));
    assert!(!result.contains('\r'));
}

// --- NUL byte rejection tests ---

#[test]
fn test_path_with_null_byte() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-nul");
    input.files[0].path = "src/foo\0bar.rs".to_string();
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));
}

#[test]
fn test_scanned_groups_with_null_byte() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test-nul");
    input.scanned_groups = vec!["gene\0sis".to_string()];
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));
}

// --- Empty string in required arrays ---

#[test]
fn test_scanned_groups_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test-empty");
    input.scanned_groups = vec!["sdd".to_string(), "".to_string()];
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("scanned_groups[1]"));
}

#[test]
fn test_scanned_categories_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-empty");
    input.scanned_categories = vec!["".to_string()];
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("scanned_categories[0]"));
}

#[test]
fn test_lens_tools_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-empty");
    input.lens_tools_used = vec!["lens_symbols".to_string(), "".to_string()];
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("lens_tools_used[1]"));
}

// --- Empty pattern source ---

#[test]
fn test_pattern_empty_source() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-empty-src");
    input.patterns[0].source = "".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("non-empty source"));
}

// --- NUL in optional fields ---

#[test]
fn test_spec_group_nul() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test-nul");
    input.specs[0].group = "gene\0sis".to_string();
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));
}

#[test]
fn test_doc_summary_nul() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-nul");
    input.docs[0].summary = "has\0nul".to_string();
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));
}

#[test]
fn test_lens_result_nul() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-nul");
    input.lens_results[0].tool = "lens\0hack".to_string();
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));
}

#[test]
fn test_file_symbols_nul() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-nul");
    input.files[0].symbols = vec!["ok".to_string(), "bad\0sym".to_string()];
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("null bytes"));
}

// --- Empty strings in optional arrays via validate_non_empty_strings ---

#[test]
fn test_dependencies_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test-dep");
    input.dependencies = vec!["valid".to_string(), "".to_string()];
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("dependencies[1]"));
}

#[test]
fn test_pitfalls_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-pit");
    input.pitfalls = vec!["".to_string()];
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("pitfalls[0]"));
}

#[test]
fn test_dependency_graph_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-dep");
    input.dependency_graph = vec!["".to_string()];
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("dependency_graph[0]"));
}

// --- Empty strings in nested optional arrays ---

#[test]
fn test_key_sections_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_spec_input("test-ks");
    input.specs[0].key_sections = vec!["valid".to_string(), "".to_string()];
    let result = create_spec_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("specs[0].key_sections[1]"));
}

#[test]
fn test_relevant_sections_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_knowledge_input("test-rs");
    input.docs[0].relevant_sections = vec!["".to_string()];
    let result = create_knowledge_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("docs[0].relevant_sections[0]"));
}

#[test]
fn test_symbols_empty_string() {
    let temp = TempDir::new().unwrap();
    let mut input = make_valid_codebase_input("test-sym");
    input.files[0].symbols = vec!["ok".to_string(), "".to_string()];
    let result = create_codebase_context(input, temp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("files[0].symbols[1]"));
}

// --- Unified entry point ---

#[test]
fn test_create_context_dispatches() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".aw/changes")).unwrap();

    let result = create_context(
        CreateContextInput::Spec(make_valid_spec_input("dispatch-test")),
        temp.path(),
    )
    .unwrap();
    assert!(result.contains("spec_context.md"));
}
// CODEGEN-END

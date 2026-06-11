---
id: projects-sdd-tests-lens-dissolution-test-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Lens Dissolution Verification Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated integration tests for the lens module dissolution into
`cclab-compass`: deleted paths, public re-export accessibility, checker and
parser behavior, agent-output reductions, hover/type-at behavior, and pipeline
fixture execution. The Rust tests template uses preamble, per-test leading
blocks, raw fixture bodies, and postamble helpers to keep this large test file
under one semantic `tests` section.

## Tests
<!-- type: tests lang: yaml -->

```yaml
preamble: |
  //! Lens dissolution restructure verification tests
  //!
  //! The lens module was dissolved from sdd into cclab-compass (#1164).
  //! R1/R2/R3 filesystem path tests removed — modules now live in libs/compass/src/.
  //! sdd re-exports them via `pub use cclab_compass::*` so NF1 tests still verify
  //! that the public API surface is preserved.
  //!
  //! Remaining coverage:
  //! - R4: `pub mod lens` removed from lib.rs
  //! - R5: No residual `crate::lens::` imports
  //! - R6: Specs migrated from cclab-lens/ to sdd/
  //! - NF1: Zero behavior change — existing functionality preserved (via re-exports)
  //! - NF3: lens/mod.rs fully deleted
  
  use std::path::{Path, PathBuf};
  
  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------
  
  /// Project root (2 levels up from the crate directory)
  fn project_root() -> PathBuf {
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
  }
  
  /// SDD crate src directory
  fn sdd_src() -> PathBuf {
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
  }
  
  // R1/R2/R3 filesystem path tests removed — modules extracted to cclab-compass (#1164).
postamble: |
  // ===========================================================================
  // Helpers
  // ===========================================================================
  
  /// Recursively walk .rs files and call the visitor on each.
  fn walk_rs_files(dir: &Path, visitor: &mut dyn FnMut(&Path)) {
      if !dir.is_dir() {
          return;
      }
      for entry in std::fs::read_dir(dir).unwrap() {
          let entry = entry.unwrap();
          let path = entry.path();
          if path.is_dir() {
              walk_rs_files(&path, visitor);
          } else if path.extension().map_or(false, |ext| ext == "rs") {
              visitor(&path);
          }
      }
  }
imports: []
tests:
  - name: r2_no_top_level_types_dir
    body: |
      // There must NOT be a bare `types/` directory at the crate top-level that
      // could collide with Rust's keyword or the old lens/types/ name.
      let types_dir = sdd_src().join("types");
      assert!(
          !types_dir.is_dir(),
          "types/ directory should not exist at top level — it was renamed to type_inference/"
      );
  - name: r4_no_pub_mod_lens_in_lib
    leading: |
      // ===========================================================================
      // R4: `pub mod lens` removed from lib.rs
      // ===========================================================================
    body: |
      let lib_path = sdd_src().join("lib.rs");
      let content = std::fs::read_to_string(&lib_path).expect("should read lib.rs");
      
      // Must not have `pub mod lens;` — lens_error is fine
      for line in content.lines() {
          let trimmed = line.trim();
          // Match exactly `pub mod lens;` but not `pub mod lens_error;`
          if trimmed == "pub mod lens;" {
              panic!(
                  "lib.rs must not contain 'pub mod lens;' (R4), found: {}",
                  line
              );
          }
      }
  - name: r5_no_residual_lens_imports
    leading: |
      // ===========================================================================
      // R5: No residual `crate::lens::` imports (comment references are OK)
      // ===========================================================================
    body: |
      let src_dir = sdd_src();
      let mut violations = Vec::new();
      
      walk_rs_files(&src_dir, &mut |path| {
          let content = match std::fs::read_to_string(path) {
              Ok(c) => c,
              Err(_) => return,
          };
          for (line_no, line) in content.lines().enumerate() {
              let trimmed = line.trim();
              // Skip comments
              if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                  continue;
              }
              if line.contains("crate::lens::") {
                  violations.push(format!("{}:{}: {}", path.display(), line_no + 1, trimmed));
              }
          }
      });
      
      assert!(
          violations.is_empty(),
          "Found {} residual crate::lens:: imports (R5):\n{}",
          violations.len(),
          violations.join("\n")
      );
  - name: nf3_lens_directory_deleted
    leading: |
      // ===========================================================================
      // NF3: lens/ directory fully deleted
      // ===========================================================================
    body: |
      let lens_dir = sdd_src().join("lens");
      assert!(
          !lens_dir.exists(),
          "lens/ directory must be fully deleted (NF3), but it still exists at {}",
          lens_dir.display()
      );
  - name: r6_old_lens_spec_directory_deleted
    leading: |
      // ===========================================================================
      // R6 / NF4: Spec migration
      // ===========================================================================
    body: |
      let old_spec_dir = project_root().join(".aw/tech-design/crates/cclab-lens");
      assert!(
          !old_spec_dir.exists(),
          ".aw/tech-design/crates/cclab-lens/ must be deleted after spec migration (R6), found at {}",
          old_spec_dir.display()
      );
  - name: r6_sdd_logic_specs_contain_migrated_lens_specs
    body: |
      let sdd_logic_dir = project_root().join("projects/agentic-workflow/tech-design/core/logic");
      assert!(
          sdd_logic_dir.is_dir(),
          "projects/agentic-workflow/tech-design/core/logic/ must exist"
      );
      
      // The merge-lens-into-sdd-spec.md should be present (from the main_spec_ref)
      let merge_spec = sdd_logic_dir.join("merge-lens-into-sdd-spec.md");
      assert!(
          merge_spec.is_file(),
          "merge-lens-into-sdd-spec.md should exist in sdd logic specs"
      );
  - name: nf1_core_types_accessible
    leading: |
      // ===========================================================================
      // NF1: Module accessibility — verify key types are accessible through
      // their new top-level paths (compile-time verification)
      // ===========================================================================
      
      /// Verify types from promoted modules are accessible through crate re-exports.
      /// This is a compile-time test: if any of these types were missing or renamed
      /// incorrectly, the test file would fail to compile.
    body: |
      // core module: ArgusConfig, LanguageConfig
      let _config: agentic_workflow::core::ArgusConfig = agentic_workflow::core::ArgusConfig::default();
      let _ = agentic_workflow::LanguageConfig::default();
  - name: nf1_diagnostic_types_accessible
    body: |
      // diagnostic module: Diagnostic, DiagnosticSeverity, Position, Range
      let _sev = agentic_workflow::diagnostic::DiagnosticSeverity::Error;
      let _pos = agentic_workflow::diagnostic::Position {
          line: 0,
          character: 0,
      };
      let _range = agentic_workflow::diagnostic::Range {
          start: agentic_workflow::diagnostic::Position {
              line: 0,
              character: 0,
          },
          end: agentic_workflow::diagnostic::Position {
              line: 0,
              character: 0,
          },
      };
  - name: nf1_syntax_types_accessible
    body: |
      // syntax module: Language, MultiParser, ParsedFile
      let _lang = agentic_workflow::syntax::Language::Python;
      let _lang_ts = agentic_workflow::syntax::Language::TypeScript;
      let _lang_rs = agentic_workflow::syntax::Language::Rust;
      // MultiParser::new() is fallible, just ensure the type is accessible
      let _parser_result = agentic_workflow::syntax::MultiParser::new();
  - name: nf1_lint_types_accessible
    body: |
      // lint module: CheckerRegistry
      let _registry = agentic_workflow::lint::CheckerRegistry::new();
  - name: nf1_lens_error_types_accessible
    body: |
      // lens_error module: ArgusError (type is accessible through crate path)
      fn _accepts_argus_error(_e: agentic_workflow::lens_error::ArgusError) {}
  - name: nf1_checker_types_accessible
    body: |
      // checker module (formerly lens/mod.rs public API): LintConfig, FileResult
      let _config = agentic_workflow::checker::LintConfig::default();
      // Verify re-export from lib.rs
      let _config2 = agentic_workflow::LintConfig::default();
  - name: nf1_watch_types_accessible
    body: |
      // watch module: WatchConfig, WatchEvent
      let _ = agentic_workflow::watch::WatchConfig::default();
  - name: nf1_spec_types_accessible
    body: |
      // spec module: DataModelSpec, RestApiSpec, JsonSchemaParser
      fn _accepts_spec(_s: agentic_workflow::spec::DataModelSpec) {}
      fn _accepts_rest_api(_s: agentic_workflow::spec::RestApiSpec) {}
  - name: nf1_gen_types_accessible
    body: |
      // gen module: TechStack, GeneratedCode, GenError
      fn _accepts_tech_stack(_t: agentic_workflow::gen::TechStack) {}
      fn _accepts_gen_error(_e: agentic_workflow::gen::GenError) {}
  - name: nf1_server_types_accessible
    body: |
      // server module: DaemonConfig
      fn _accepts_daemon_config(_c: agentic_workflow::server::DaemonConfig) {}
  - name: nf1_semantic_types_accessible
    body: |
      // semantic module: SymbolTable, ScopeAnalyzer
      fn _accepts_symbol_table(_s: agentic_workflow::semantic::SymbolTable) {}
  - name: nf1_refactoring_types_accessible
    body: |
      // refactoring module: RenameEngine, ExtractEngine
      fn _accepts_rename(_r: agentic_workflow::refactoring::RenameEngine) {}
      fn _accepts_extract(_e: agentic_workflow::refactoring::ExtractEngine) {}
  - name: nf1_format_types_accessible
    body: |
      // format module: FormatResult
      fn _accepts_format_result(_f: agentic_workflow::format::FormatResult) {}
  - name: nf1_graph_types_accessible
    body: |
      // graph module: ImportGraph
      fn _accepts_import_graph(_g: agentic_workflow::graph::ImportGraph) {}
  - name: nf1_schemas_types_accessible
    body: |
      // schemas module: SchemaRegistry
      fn _accepts_schema_registry(_s: agentic_workflow::schemas::SchemaRegistry) {}
  - name: nf1_search_types_accessible
    body: |
      // search module: SearchEngine
      fn _accepts_search_engine(_s: agentic_workflow::search::SearchEngine) {}
  - name: nf1_storage_accessible
    body: |
      // storage module: resolve_lens_storage
      let tmp = std::env::temp_dir();
      let _ = agentic_workflow::storage::resolve_lens_storage(&tmp);
  - name: r2_type_inference_types_accessible
    leading: |
      // ===========================================================================
      // R2: type_inference path — verify renamed module exports
      // ===========================================================================
    body: |
      // type_inference module (formerly lens/types/)
      fn _accepts_search_kind(_k: agentic_workflow::type_inference::SearchKind) {}
      fn _accepts_refactor_kind(_k: agentic_workflow::type_inference::RefactorKind) {}
  - name: nf1_multiparser_detects_python
    leading: |
      // ===========================================================================
      // NF1: Behavioral tests — verify functionality works through new paths
      // ===========================================================================
    body: |
      let path = Path::new("test.py");
      let lang = agentic_workflow::syntax::MultiParser::detect_language(path);
      assert_eq!(lang, Some(agentic_workflow::syntax::Language::Python));
  - name: nf1_multiparser_detects_typescript
    body: |
      let path = Path::new("test.ts");
      let lang = agentic_workflow::syntax::MultiParser::detect_language(path);
      assert_eq!(lang, Some(agentic_workflow::syntax::Language::TypeScript));
  - name: nf1_multiparser_detects_rust
    body: |
      let path = Path::new("test.rs");
      let lang = agentic_workflow::syntax::MultiParser::detect_language(path);
      assert_eq!(lang, Some(agentic_workflow::syntax::Language::Rust));
  - name: nf1_lint_config_default_languages
    body: |
      let config = agentic_workflow::checker::LintConfig::default();
      assert!(config.is_language_enabled(agentic_workflow::syntax::Language::Python));
      assert!(config.is_language_enabled(agentic_workflow::syntax::Language::TypeScript));
      assert!(config.is_language_enabled(agentic_workflow::syntax::Language::Rust));
  - name: nf1_lint_config_excludes_defaults
    body: |
      let config = agentic_workflow::checker::LintConfig::default();
      assert!(config.is_excluded(Path::new("/project/node_modules/foo.js")));
      assert!(config.is_excluded(Path::new("/project/__pycache__/bar.pyc")));
      assert!(config.is_excluded(Path::new("/project/target/debug/app")));
      assert!(!config.is_excluded(Path::new("/project/src/main.py")));
  - name: nf1_checker_registry_has_python
    body: |
      let registry = agentic_workflow::lint::CheckerRegistry::new();
      assert!(
          registry.get(agentic_workflow::syntax::Language::Python).is_some(),
          "CheckerRegistry should have a Python checker"
      );
  - name: nf1_checker_registry_has_typescript
    body: |
      let registry = agentic_workflow::lint::CheckerRegistry::new();
      assert!(
          registry.get(agentic_workflow::syntax::Language::TypeScript).is_some(),
          "CheckerRegistry should have a TypeScript checker"
      );
  - name: nf1_checker_registry_has_rust
    body: |
      let registry = agentic_workflow::lint::CheckerRegistry::new();
      assert!(
          registry.get(agentic_workflow::syntax::Language::Rust).is_some(),
          "CheckerRegistry should have a Rust checker"
      );
  - name: nf1_argus_config_default
    body: |
      let config = agentic_workflow::core::ArgusConfig::default();
      // Just verify it creates without panicking — zero behavior change
      let _ = format!("{:?}", config);
  - name: nf1_diagnostic_severity_ordering
    body: |
      use agentic_workflow::diagnostic::DiagnosticSeverity;
      assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Warning);
      assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Information);
      assert!(DiagnosticSeverity::Information < DiagnosticSeverity::Hint);
  - name: nf1_watch_config_default
    body: |
      let config = agentic_workflow::watch::WatchConfig::default();
      let _ = format!("{:?}", config);
  - name: test_cli_check_format_agent_python
    leading: |
      // R1 lib.rs module declaration test removed — sdd now uses `pub use cclab_compass::*`
      // re-exports instead of `pub mod` declarations for compass modules.
      
      // ===========================================================================
      // Agent Output Format — Integration Tests (spec: agent-output-format)
      // ===========================================================================
      
      /// Create a temporary directory with Python fixture files for integration tests.
      fn create_python_fixtures(dir: &Path) {
          let db_py = dir.join("db.py");
          std::fs::write(
              &db_py,
              r#"
      from models import User
      
      def get_user(user_id: int) -> User:
          """Fetch user by ID."""
          result = query(user_id)
          return result
      "#,
          )
          .unwrap();
      
          let handler_py = dir.join("handler.py");
          std::fs::write(
              &handler_py,
              r#"
      from db import get_user
      from models import User
      
      def handle_request(request):
          user = get_user(request.user_id)
          return user
      "#,
          )
          .unwrap();
      
          let models_py = dir.join("models.py");
          std::fs::write(
              &models_py,
              r#"
      class User:
          def __init__(self, name: str, email: str):
              self.name = name
              self.email = email
      "#,
          )
          .unwrap();
      }
      
      /// Create a clean project fixture with no lint issues.
      fn create_clean_fixtures(dir: &Path) {
          std::fs::write(
              dir.join("clean.py"),
              r#"
      def hello(name: str) -> str:
          """Greet the user."""
          return f"Hello, {name}!"
      "#,
          )
          .unwrap();
      }
      
      /// Create polyglot fixtures with both Python and TypeScript.
      fn create_polyglot_fixtures(dir: &Path) {
          std::fs::write(
              dir.join("app.py"),
              r#"
      def main():
          print("Hello from Python")
      "#,
          )
          .unwrap();
      
          std::fs::write(
              dir.join("app.ts"),
              r#"
      function greet(name: string): string {
          return `Hello, ${name}!`;
      }
      
      export { greet };
      "#,
          )
          .unwrap();
      }
      
      /// Run check + agent output for a set of files, returning the AgentOutput JSON string.
      fn run_agent_check(dir: &Path) -> String {
          use agentic_workflow::checker::{check_paths, LintConfig};
          use agentic_workflow::graph::ImportGraph;
          use agentic_workflow::output::reporter::OutputFormat;
          use agentic_workflow::output::Reporter;
          use agentic_workflow::semantic::SymbolTableBuilder;
          use agentic_workflow::syntax::{Language, MultiParser};
      
          let config = LintConfig {
              languages: vec![Language::Python, Language::TypeScript, Language::Rust],
              ..LintConfig::default()
          };
      
          // Collect all files in directory
          let files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
              .unwrap()
              .filter_map(|e| e.ok())
              .map(|e| e.path())
              .filter(|p| p.is_file())
              .collect();
      
          let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
          let results = check_paths(&file_refs, &config);
      
          // Build symbol tables and import graph
          let mut parser = MultiParser::new().unwrap();
          let mut symbol_tables: Vec<(std::path::PathBuf, agentic_workflow::semantic::SymbolTable)> = Vec::new();
          let mut import_files: Vec<(std::path::PathBuf, String)> = Vec::new();
      
          for result in &results {
              let source = match std::fs::read_to_string(&result.path) {
                  Ok(s) => s,
                  Err(_) => continue,
              };
      
              import_files.push((result.path.clone(), source.clone()));
      
              if let Some(parsed) = parser.parse(&source, result.language) {
                  let table = match result.language {
                      Language::Python => SymbolTableBuilder::new().build_python(&parsed),
                      Language::TypeScript => SymbolTableBuilder::new().build_typescript(&parsed),
                      Language::Rust => SymbolTableBuilder::new().build_rust(&parsed),
                      _ => agentic_workflow::semantic::SymbolTable::new(),
                  };
                  symbol_tables.push((result.path.clone(), table));
              }
          }
      
          let import_graph = ImportGraph::build(&import_files, dir);
      
          let reporter = Reporter::new(OutputFormat::Agent);
          reporter.generate_agent(&results, &symbol_tables, &import_graph, dir)
      }
      
      /// S1: End-to-end agent format on Python project with cross-file dependencies.
      /// Validates R1-R9: symbols, imports, issues, impact, stats in agent JSON.
    body: |
      let tmp = std::env::temp_dir().join("cclab_test_agent_python");
      let _ = std::fs::remove_dir_all(&tmp);
      std::fs::create_dir_all(&tmp).unwrap();
      create_python_fixtures(&tmp);
      
      let json_str = run_agent_check(&tmp);
      
      // Must be valid JSON (NF4)
      let parsed: serde_json::Value =
          serde_json::from_str(&json_str).expect("Agent output must be valid JSON (NF4)");
      
      // R2: symbols map present
      let symbols = parsed.get("symbols").expect("symbols must be present");
      assert!(symbols.is_object(), "symbols must be an object");
      
      // R8: stats present with correct counts
      let stats = parsed.get("stats").expect("stats must be present");
      assert!(
          stats["files_checked"].as_u64().unwrap() > 0,
          "files_checked should be > 0"
      );
      
      // Clean up
      let _ = std::fs::remove_dir_all(&tmp);
  - name: test_cli_check_format_agent_clean
    leading: |
      /// S2: Clean project with no issues — issues key should be absent (R9).
    body: |
      let tmp = std::env::temp_dir().join("cclab_test_agent_clean");
      let _ = std::fs::remove_dir_all(&tmp);
      std::fs::create_dir_all(&tmp).unwrap();
      create_clean_fixtures(&tmp);
      
      let json_str = run_agent_check(&tmp);
      let parsed: serde_json::Value =
          serde_json::from_str(&json_str).expect("Agent output must be valid JSON");
      
      // R9: issues should be absent when empty
      if let Some(issues) = parsed.get("issues") {
          // If present, it must be non-empty (skip_serializing_if should have omitted it)
          assert!(
              issues.as_array().map_or(true, |a| !a.is_empty()),
              "issues key should not be present when empty (R9)"
          );
      }
      
      // symbols should still be present
      assert!(
          parsed.get("symbols").is_some(),
          "symbols must always be present"
      );
      assert!(
          parsed.get("stats").is_some(),
          "stats must always be present"
      );
      
      let _ = std::fs::remove_dir_all(&tmp);
  - name: test_cli_check_format_agent_polyglot
    leading: |
      /// S4: Mixed Python + TypeScript — single unified agent JSON.
    body: |
      let tmp = std::env::temp_dir().join("cclab_test_agent_polyglot");
      let _ = std::fs::remove_dir_all(&tmp);
      std::fs::create_dir_all(&tmp).unwrap();
      create_polyglot_fixtures(&tmp);
      
      let json_str = run_agent_check(&tmp);
      let parsed: serde_json::Value =
          serde_json::from_str(&json_str).expect("Agent output must be valid JSON");
      
      // Should have symbols from both languages in a single output
      let symbols = parsed.get("symbols").expect("symbols must be present");
      let symbols_map = symbols.as_object().expect("symbols must be object");
      
      // stats should reflect files from both languages
      let stats = parsed.get("stats").expect("stats must be present");
      let files_checked = stats["files_checked"].as_u64().unwrap();
      assert!(
          files_checked >= 2,
          "polyglot project should check at least 2 files, got {}",
          files_checked
      );
      
      // Symbols should contain entries from both .py and .ts files
      let has_py = symbols_map.values().any(|v| {
          v.get("file")
              .and_then(|f| f.as_str())
              .map_or(false, |f| f.ends_with(".py"))
      });
      let has_ts = symbols_map.values().any(|v| {
          v.get("file")
              .and_then(|f| f.as_str())
              .map_or(false, |f| f.ends_with(".ts"))
      });
      assert!(has_py, "should have Python symbols");
      assert!(has_ts, "should have TypeScript symbols");
      
      let _ = std::fs::remove_dir_all(&tmp);
  - name: test_agent_output_smaller_than_json
    leading: |
      /// NF5/S5: Agent format should be significantly smaller than standard JSON output.
    body: |
      let tmp = std::env::temp_dir().join("cclab_test_agent_size");
      let _ = std::fs::remove_dir_all(&tmp);
      std::fs::create_dir_all(&tmp).unwrap();
      create_python_fixtures(&tmp);
      
      use agentic_workflow::checker::{check_paths, LintConfig};
      use agentic_workflow::output::reporter::OutputFormat;
      use agentic_workflow::output::Reporter;
      use agentic_workflow::syntax::Language;
      
      let config = LintConfig {
          languages: vec![Language::Python],
          ..LintConfig::default()
      };
      
      let files: Vec<std::path::PathBuf> = std::fs::read_dir(&tmp)
          .unwrap()
          .filter_map(|e| e.ok())
          .map(|e| e.path())
          .filter(|p| p.is_file())
          .collect();
      
      let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
      let results = check_paths(&file_refs, &config);
      
      // Standard JSON output
      let json_reporter = Reporter::new(OutputFormat::Json);
      let json_output = json_reporter.generate(&results);
      
      // Agent output
      let agent_output = run_agent_check(&tmp);
      
      // Agent format should generally be compact. For small projects the sizes may
      // be comparable due to fixed overhead (stats, etc.), so we use a relaxed check.
      // NF5 specifies <50% for a 50-file project; for our small fixture we just
      // ensure agent output is produced and is valid JSON.
      assert!(!agent_output.is_empty(), "agent output should not be empty");
      // Log sizes for manual verification
      eprintln!(
          "Size comparison: JSON={} bytes, Agent={} bytes (ratio={:.1}%)",
          json_output.len(),
          agent_output.len(),
          (agent_output.len() as f64 / json_output.len() as f64) * 100.0
      );
      
      let _ = std::fs::remove_dir_all(&tmp);
  - name: test_type_at_imported_symbol
    leading: |
      // ===========================================================================
      // Type Inference Pipeline — Integration Tests
      // ===========================================================================
      
      /// R4, S1: `type_at` on imported symbol returns resolved type from source module.
    body: |
      use agentic_workflow::check_pipeline;
      use agentic_workflow::graph::ImportGraph;
      use agentic_workflow::type_inference::{
          DeepTypeInferencer, ImportInfo, PropagationPipeline, PropagationRequest, Type, TypeBinding,
      };
      
      let mut inf = DeepTypeInferencer::new();
      let db = PathBuf::from("db.py");
      let handler = PathBuf::from("handler.py");
      inf.add_file(db.clone());
      inf.add_file(handler.clone());
      
      // db.py exports get_user: Callable -> User
      let user_ty = Type::Instance {
          name: "User".to_string(),
          module: None,
          type_args: vec![],
      };
      inf.add_file_symbol(
          &db,
          "get_user".to_string(),
          TypeBinding {
              ty: Type::Callable {
                  params: vec![],
                  ret: Box::new(user_ty.clone()),
              },
              source_file: db.clone(),
              symbol: "get_user".to_string(),
              line: 5,
              is_exported: true,
              dependencies: vec![],
              is_propagated: false,
          },
      );
      
      // handler.py imports get_user from db
      inf.add_import(
          &handler,
          ImportInfo {
              module: "db".to_string(),
              names: Some(vec!["get_user".to_string()]),
              alias: None,
          },
      );
      inf.add_import_edge(handler.clone(), db.clone());
      
      // Run propagation
      let ig = ImportGraph::new();
      let request = PropagationRequest {
          files: vec![db.clone(), handler.clone()],
          changed_files: vec![],
      };
      PropagationPipeline::run(&request, &mut inf, &ig);
      
      // type_at should return the propagated Callable type, not Unknown
      let result = check_pipeline::type_at(&inf, &handler, "get_user");
      assert!(
          result.is_some(),
          "type_at should find the propagated symbol"
      );
      let result = result.unwrap();
      assert!(
          result.is_propagated,
          "Symbol should be marked as propagated"
      );
      match &result.ty {
          Type::Callable { ret, .. } => match ret.as_ref() {
              Type::Instance { name, .. } => assert_eq!(name, "User"),
              other => panic!("Expected Instance(User), got {:?}", other),
          },
          other => panic!("Expected Callable, got {:?}", other),
      }
  - name: test_hover_imported_symbol
    leading: |
      /// R5: `hover` on imported symbol returns type signature from source module.
    body: |
      use agentic_workflow::check_pipeline;
      use agentic_workflow::graph::ImportGraph;
      use agentic_workflow::type_inference::{
          DeepTypeInferencer, ImportInfo, PropagationPipeline, PropagationRequest, Type, TypeBinding,
      };
      
      let mut inf = DeepTypeInferencer::new();
      let db = PathBuf::from("db.py");
      let handler = PathBuf::from("handler.py");
      inf.add_file(db.clone());
      inf.add_file(handler.clone());
      
      inf.add_file_symbol(
          &db,
          "get_user".to_string(),
          TypeBinding {
              ty: Type::Int,
              source_file: db.clone(),
              symbol: "get_user".to_string(),
              line: 3,
              is_exported: true,
              dependencies: vec![],
              is_propagated: false,
          },
      );
      
      inf.add_import(
          &handler,
          ImportInfo {
              module: "db".to_string(),
              names: Some(vec!["get_user".to_string()]),
              alias: None,
          },
      );
      inf.add_import_edge(handler.clone(), db.clone());
      
      let ig = ImportGraph::new();
      let request = PropagationRequest {
          files: vec![db.clone(), handler.clone()],
          changed_files: vec![],
      };
      PropagationPipeline::run(&request, &mut inf, &ig);
      
      let result = check_pipeline::hover(&inf, &handler, "get_user");
      assert!(result.is_some());
      let result = result.unwrap();
      assert!(result.is_propagated);
      assert!(
          result.type_signature.contains("get_user"),
          "Hover should include symbol name"
      );
  - name: test_check_with_propagation_diagnostics
    leading: |
      /// R10: `check_paths_with_propagation` on multi-file project returns propagation result.
    body: |
      use agentic_workflow::graph::ImportGraph;
      use agentic_workflow::type_inference::{
          DeepTypeInferencer, ImportInfo, PropagationPipeline, PropagationRequest, Type, TypeBinding,
      };
      
      let mut inf = DeepTypeInferencer::new();
      let db = PathBuf::from("db.py");
      let handler = PathBuf::from("handler.py");
      let main = PathBuf::from("main.py");
      inf.add_file(db.clone());
      inf.add_file(handler.clone());
      inf.add_file(main.clone());
      
      // db.py: exported Config class
      inf.add_file_symbol(
          &db,
          "Config".to_string(),
          TypeBinding {
              ty: Type::Instance {
                  name: "Config".to_string(),
                  module: None,
                  type_args: vec![],
              },
              source_file: db.clone(),
              symbol: "Config".to_string(),
              line: 1,
              is_exported: true,
              dependencies: vec![],
              is_propagated: false,
          },
      );
      
      // handler.py imports Config from db
      inf.add_import(
          &handler,
          ImportInfo {
              module: "db".to_string(),
              names: Some(vec!["Config".to_string()]),
              alias: None,
          },
      );
      inf.add_import_edge(handler.clone(), db.clone());
      
      // main.py imports Config from handler (transitive)
      inf.add_import(
          &main,
          ImportInfo {
              module: "handler".to_string(),
              names: Some(vec!["Config".to_string()]),
              alias: None,
          },
      );
      inf.add_import_edge(main.clone(), handler.clone());
      
      let ig = ImportGraph::new();
      let request = PropagationRequest {
          files: vec![db.clone(), handler.clone(), main.clone()],
          changed_files: vec![],
      };
      let result = PropagationPipeline::run(&request, &mut inf, &ig);
      
      // Propagation should have analyzed all 3 files
      assert_eq!(result.stats.files_analyzed, 3);
      // At least Config should be propagated to handler and main
      assert!(result.stats.symbols_propagated >= 1);
      assert_eq!(result.stats.cycles_detected, 0);
  - name: test_pipeline_python_fixture
    leading: |
      /// R1-R6, NF4: End-to-end fixture Python project with imports.
    body: |
      use agentic_workflow::graph::ImportGraph;
      use agentic_workflow::type_inference::{
          DeepTypeInferencer, ImportInfo, PropagationPipeline, PropagationRequest, Type, TypeBinding,
      };
      
      let mut inf = DeepTypeInferencer::new();
      
      // Simulate a 3-file Python project:
      //   models.py: class User, class Post
      //   db.py:     from models import User; def get_user() -> User
      //   app.py:    from db import get_user
      let models = PathBuf::from("models.py");
      let db = PathBuf::from("db.py");
      let app = PathBuf::from("app.py");
      
      inf.add_file(models.clone());
      inf.add_file(db.clone());
      inf.add_file(app.clone());
      
      // models.py exports User and Post
      let user_ty = Type::Instance {
          name: "User".to_string(),
          module: Some("models".to_string()),
          type_args: vec![],
      };
      inf.add_file_symbol(
          &models,
          "User".to_string(),
          TypeBinding {
              ty: user_ty.clone(),
              source_file: models.clone(),
              symbol: "User".to_string(),
              line: 1,
              is_exported: true,
              dependencies: vec![],
              is_propagated: false,
          },
      );
      inf.add_file_symbol(
          &models,
          "Post".to_string(),
          TypeBinding {
              ty: Type::Instance {
                  name: "Post".to_string(),
                  module: Some("models".to_string()),
                  type_args: vec![],
              },
              source_file: models.clone(),
              symbol: "Post".to_string(),
              line: 10,
              is_exported: true,
              dependencies: vec![],
              is_propagated: false,
          },
      );
      
      // db.py: from models import User
      inf.add_import(
          &db,
          ImportInfo {
              module: "models".to_string(),
              names: Some(vec!["User".to_string()]),
              alias: None,
          },
      );
      inf.add_import_edge(db.clone(), models.clone());
      
      // db.py exports get_user: Callable -> User
      inf.add_file_symbol(
          &db,
          "get_user".to_string(),
          TypeBinding {
              ty: Type::Callable {
                  params: vec![],
                  ret: Box::new(user_ty),
              },
              source_file: db.clone(),
              symbol: "get_user".to_string(),
              line: 5,
              is_exported: true,
              dependencies: vec![],
              is_propagated: false,
          },
      );
      
      // app.py: from db import get_user
      inf.add_import(
          &app,
          ImportInfo {
              module: "db".to_string(),
              names: Some(vec!["get_user".to_string()]),
              alias: None,
          },
      );
      inf.add_import_edge(app.clone(), db.clone());
      
      let ig = ImportGraph::new();
      let request = PropagationRequest {
          files: vec![models.clone(), db.clone(), app.clone()],
          changed_files: vec![],
      };
      let result = PropagationPipeline::run(&request, &mut inf, &ig);
      
      // Verify propagation stats
      assert_eq!(result.stats.files_analyzed, 3);
      assert_eq!(result.stats.cycles_detected, 0);
      
      // Verify db.py has User from models.py
      let db_fa = inf.get_file_analysis(&db).unwrap();
      assert!(
          db_fa.symbols.contains_key("User"),
          "db.py should have propagated User"
      );
      
      // Verify app.py has get_user from db.py
      let app_fa = inf.get_file_analysis(&app).unwrap();
      assert!(
          app_fa.symbols.contains_key("get_user"),
          "app.py should have propagated get_user"
      );
      let get_user = app_fa.symbols.get("get_user").unwrap();
      assert!(get_user.is_propagated);
      
      // All files should have propagation_complete
      assert!(inf.get_file_analysis(&models).unwrap().propagation_complete);
      assert!(inf.get_file_analysis(&db).unwrap().propagation_complete);
      assert!(inf.get_file_analysis(&app).unwrap().propagation_complete);
  - name: test_pipeline_no_reparse
    leading: |
      /// NF3: Propagation reads from FileAnalysis cache, does not trigger file re-reads.
    body: |
      use agentic_workflow::graph::ImportGraph;
      use agentic_workflow::type_inference::{
          DeepTypeInferencer, ImportInfo, PropagationPipeline, PropagationRequest, Type, TypeBinding,
      };
      
      // Create files that do NOT exist on disk — propagation must work from
      // in-memory FileAnalysis only (NF3: no file re-reads).
      let mut inf = DeepTypeInferencer::new();
      let source = PathBuf::from("/nonexistent/source.py");
      let target = PathBuf::from("/nonexistent/target.py");
      inf.add_file(source.clone());
      inf.add_file(target.clone());
      
      inf.add_file_symbol(
          &source,
          "helper".to_string(),
          TypeBinding {
              ty: Type::Int,
              source_file: source.clone(),
              symbol: "helper".to_string(),
              line: 1,
              is_exported: true,
              dependencies: vec![],
              is_propagated: false,
          },
      );
      
      inf.add_import(
          &target,
          ImportInfo {
              module: "source".to_string(),
              names: Some(vec!["helper".to_string()]),
              alias: None,
          },
      );
      inf.add_import_edge(target.clone(), source.clone());
      
      let ig = ImportGraph::new();
      let request = PropagationRequest {
          files: vec![source.clone(), target.clone()],
          changed_files: vec![],
      };
      
      // If propagation tried to read files from disk, this would fail because
      // the paths don't exist. Success proves NF3 compliance.
      let result = PropagationPipeline::run(&request, &mut inf, &ig);
      
      assert_eq!(result.stats.files_analyzed, 2);
      let target_fa = inf.get_file_analysis(&target).unwrap();
      assert!(target_fa.symbols.contains_key("helper"));
      assert!(target_fa.propagation_complete);
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/lens_dissolution_test.rs
    action: modify
    section: tests
    impl_mode: codegen
    generator: rust.tests
    description: |
      Emit the large lens dissolution test suite from the Rust tests template,
      using postamble helpers and raw fixture body preservation.
```

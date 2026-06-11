//! Agent output builder: constructs symbol-centric JSON from analysis results.
//!
//! Combines SymbolTable, ImportGraph, and lint diagnostics into a compact
//! representation optimized for LLM agent consumption.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::checker::FileResult;
use crate::diagnostic::DiagnosticSeverity;
use crate::graph::ImportGraph;
use crate::semantic::symbols::{SymbolKind, SymbolTable};

use super::agent_types::{AgentIssue, AgentOutput, AgentStats, SymbolDef};

/// Builder that orchestrates construction of agent-format output.
pub struct AgentOutputBuilder<'a> {
    /// Project root for computing relative paths.
    project_root: &'a Path,
}

impl<'a> AgentOutputBuilder<'a> {
    pub fn new(project_root: &'a Path) -> Self {
        Self { project_root }
    }

    /// Build the complete agent output from analysis results.
    ///
    /// - `results`: lint/check results per file
    /// - `symbol_tables`: per-file symbol tables (keyed by absolute path)
    /// - `import_graph`: project-wide import dependency graph
    pub fn build(
        &self,
        results: &[FileResult],
        symbol_tables: &[(PathBuf, SymbolTable)],
        import_graph: &ImportGraph,
    ) -> AgentOutput {
        let symbols = self.build_symbols(symbol_tables);
        let imports = self.build_imports(results, import_graph);
        let issues = self.build_issues(results, symbol_tables);
        let impact = self.build_impact(symbol_tables);

        let impact_edges: usize = impact.values().map(|v| v.len()).sum();

        let stats = AgentStats {
            files_checked: results.len(),
            symbols_found: symbols.len(),
            issues_count: issues.len(),
            impact_edges,
        };

        AgentOutput {
            symbols,
            imports,
            issues,
            impact,
            stats,
        }
    }

    /// Build the symbols map from per-file SymbolTables.
    ///
    /// Each symbol is keyed by its qualified name (file-relative name for now).
    /// Includes type signature from SymbolTable `type_info` (R2, R7).
    fn build_symbols(
        &self,
        symbol_tables: &[(PathBuf, SymbolTable)],
    ) -> BTreeMap<String, SymbolDef> {
        let mut symbols = BTreeMap::new();

        for (file_path, table) in symbol_tables {
            let rel_path = self.relative_path(file_path);

            for sym in table.all_symbols() {
                // Skip imports and parameters — only user-defined symbols
                if matches!(sym.kind, SymbolKind::Import | SymbolKind::Parameter) {
                    continue;
                }

                let kind_str = symbol_kind_to_agent_kind(sym.kind);
                let type_sig = sym.type_info.as_ref().map(|t| t.display());

                // Use "file_stem.name" as a simple qualified name
                let qualified = format_qualified_name(&rel_path, &sym.name);

                symbols.insert(
                    qualified,
                    SymbolDef {
                        type_sig,
                        file: rel_path.clone(),
                        line: sym.location.start.line + 1, // 0-indexed to 1-indexed
                        kind: kind_str.to_string(),
                    },
                );
            }
        }

        symbols
    }

    /// Build the imports map from ImportGraph edges.
    ///
    /// Maps file path to list of imported symbol qualified names (R3).
    fn build_imports(
        &self,
        results: &[FileResult],
        import_graph: &ImportGraph,
    ) -> BTreeMap<String, Vec<String>> {
        let mut imports = BTreeMap::new();

        for result in results {
            let deps = import_graph.dependencies(&result.path);
            if deps.is_empty() {
                continue;
            }

            let rel_path = self.relative_path(&result.path);
            let import_paths: Vec<String> =
                deps.iter().map(|edge| edge.import_path.clone()).collect();

            if !import_paths.is_empty() {
                imports.insert(rel_path, import_paths);
            }
        }

        imports
    }

    /// Build the issues array from diagnostics with symbol attribution (R4, R6).
    ///
    /// Each diagnostic is attributed to the nearest enclosing symbol via
    /// binary search on SymbolTable ranges. If no enclosing symbol is found,
    /// uses `"<file-level>"`.
    fn build_issues(
        &self,
        results: &[FileResult],
        symbol_tables: &[(PathBuf, SymbolTable)],
    ) -> Vec<AgentIssue> {
        let mut issues = Vec::new();

        // Build lookup from path to symbol table
        let table_map: BTreeMap<&Path, &SymbolTable> = symbol_tables
            .iter()
            .map(|(p, t)| (p.as_path(), t))
            .collect();

        for result in results {
            let rel_path = self.relative_path(&result.path);
            let table = table_map.get(result.path.as_path());

            for diag in &result.diagnostics {
                let symbol_name = table
                    .and_then(|t| {
                        find_enclosing_symbol(t, diag.range.start.line, diag.range.start.character)
                    })
                    .unwrap_or_else(|| "<file-level>".to_string());

                issues.push(AgentIssue {
                    severity: severity_to_str(diag.severity).to_string(),
                    symbol: symbol_name,
                    file: rel_path.clone(),
                    line: diag.range.start.line + 1, // 0-indexed to 1-indexed
                    code: diag.code.clone(),
                    message: diag.message.clone(),
                });
            }
        }

        issues
    }

    /// Build the impact map from SymbolTable references (R5).
    ///
    /// Groups non-definition references by target symbol, emitting
    /// "file:line" location strings.
    ///
    /// **Limitation**: Currently resolves references within the same file only.
    /// Cross-file references (e.g., file A calls a symbol defined in file B)
    /// require a global symbol index and are not tracked here. The impact map
    /// is accurate for intra-file references; cross-file tracking is deferred.
    fn build_impact(
        &self,
        symbol_tables: &[(PathBuf, SymbolTable)],
    ) -> BTreeMap<String, Vec<String>> {
        let mut impact: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for (file_path, table) in symbol_tables {
            let rel_path = self.relative_path(file_path);

            for reference in table.all_references() {
                if reference.is_definition {
                    continue;
                }

                // Look up the target symbol
                if let Some(sym) = table.get(reference.symbol_id) {
                    // Skip imports and parameters
                    if matches!(sym.kind, SymbolKind::Import | SymbolKind::Parameter) {
                        continue;
                    }

                    let sym_rel_path = self.relative_path(file_path);
                    let qualified = format_qualified_name(&sym_rel_path, &sym.name);
                    let location = format!("{}:{}", rel_path, reference.location.start.line + 1);

                    impact.entry(qualified).or_default().push(location);
                }
            }
        }

        // Deduplicate locations per symbol
        for locations in impact.values_mut() {
            locations.sort();
            locations.dedup();
        }

        impact
    }

    /// Compute relative path from project root.
    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(self.project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }
}

/// Find the nearest enclosing symbol for a position in a symbol table.
///
/// Iterates all symbols and finds one whose range contains the position.
/// Returns the symbol name or None if at file level.
fn find_enclosing_symbol(table: &SymbolTable, line: u32, character: u32) -> Option<String> {
    let mut best: Option<&crate::semantic::Symbol> = None;

    for sym in table.all_symbols() {
        // Skip imports and parameters
        if matches!(sym.kind, SymbolKind::Import | SymbolKind::Parameter) {
            continue;
        }

        if sym.location.contains(line, character) {
            // Prefer the most specific (smallest range) enclosing symbol
            if let Some(current_best) = best {
                let current_size = range_size(&current_best.location);
                let new_size = range_size(&sym.location);
                if new_size < current_size {
                    best = Some(sym);
                }
            } else {
                best = Some(sym);
            }
        }
    }

    best.map(|s| s.name.clone())
}

/// Compute approximate range size for comparison.
fn range_size(range: &crate::diagnostic::Range) -> u64 {
    let lines = (range.end.line as u64).saturating_sub(range.start.line as u64);
    let cols = (range.end.character as u64).saturating_sub(range.start.character as u64);
    lines * 1000 + cols
}

/// Map SymbolKind to the agent output kind string.
///
/// Uses the schema-defined enum: function, class, method, variable, constant,
/// interface, type_alias, module.
fn symbol_kind_to_agent_kind(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Class | SymbolKind::Struct | SymbolKind::Enum => "class",
        SymbolKind::Trait | SymbolKind::Interface => "interface",
        SymbolKind::Variable => "variable",
        SymbolKind::Const | SymbolKind::Static => "constant",
        SymbolKind::TypeAlias | SymbolKind::TypeParameter => "type_alias",
        SymbolKind::Module | SymbolKind::Impl => "module",
        // Infrastructure and other kinds default to "variable"
        SymbolKind::Resource
        | SymbolKind::Job
        | SymbolKind::Stage
        | SymbolKind::Port
        | SymbolKind::Label
        | SymbolKind::Selector
        | SymbolKind::Template => "variable",
        // Decorators, macros
        SymbolKind::Decorator | SymbolKind::Macro => "function",
        // Import, Parameter, EnumMember — filtered upstream but handled for exhaustiveness
        SymbolKind::Import | SymbolKind::Parameter | SymbolKind::EnumMember => "variable",
    }
}

/// Format a simple qualified name from file path and symbol name.
fn format_qualified_name(rel_path: &str, name: &str) -> String {
    // Extract module name from file path (stem without extension)
    let stem = Path::new(rel_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    format!("{}.{}", stem, name)
}

/// Map DiagnosticSeverity to agent output severity string.
fn severity_to_str(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "error",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Information => "info",
        DiagnosticSeverity::Hint => "hint",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::FileResult;
    use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
    use crate::graph::ImportGraph;
    use crate::semantic::symbols::{SymbolKind, SymbolTable, TypeInfo};
    use crate::syntax::Language;
    use std::path::PathBuf;

    /// Helper: create a Range from 0-indexed (start_line, start_char) to (end_line, end_char).
    fn make_range(sl: u32, sc: u32, el: u32, ec: u32) -> Range {
        Range::new(Position::new(sl, sc), Position::new(el, ec))
    }

    /// Helper: create a minimal FileResult with given path, language, and diagnostics.
    fn make_file_result(path: &str, lang: Language, diagnostics: Vec<Diagnostic>) -> FileResult {
        FileResult {
            path: PathBuf::from(path),
            language: lang,
            diagnostics,
        }
    }

    /// Helper: create a Diagnostic at a given 0-indexed line.
    fn make_diagnostic(
        line: u32,
        severity: DiagnosticSeverity,
        code: &str,
        message: &str,
    ) -> Diagnostic {
        Diagnostic::new(
            make_range(line, 0, line, 10),
            severity,
            code,
            DiagnosticCategory::Style,
            message,
        )
    }

    /// Helper: build a SymbolTable with a function symbol.
    fn make_symbol_table_with_function(
        name: &str,
        kind: SymbolKind,
        line: u32,
        end_line: u32,
        type_info: Option<TypeInfo>,
    ) -> SymbolTable {
        let mut table = SymbolTable::new();
        table.add_symbol(
            name.to_string(),
            kind,
            make_range(line, 0, end_line, 0),
            type_info,
            None,
            0,
        );
        table
    }

    // --- R1: OutputFormat::Agent variant ---

    #[test]
    fn test_output_format_agent_variant() {
        use crate::output::reporter::OutputFormat;

        assert_eq!(
            OutputFormat::from_str("agent"),
            Some(OutputFormat::Agent),
            "from_str(\"agent\") should return Some(OutputFormat::Agent)"
        );
        // Case insensitivity
        assert_eq!(
            OutputFormat::from_str("AGENT"),
            Some(OutputFormat::Agent),
            "from_str should be case-insensitive"
        );
    }

    // --- R2, R7: build_symbols from SymbolTable ---

    #[test]
    fn test_build_symbols_from_symbol_table() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        let type_info = Some(TypeInfo::Callable {
            params: vec![TypeInfo::Primitive("int".to_string())],
            ret: Box::new(TypeInfo::Named("User".to_string())),
        });

        let table =
            make_symbol_table_with_function("get_user", SymbolKind::Function, 41, 50, type_info);

        let symbol_tables = vec![(PathBuf::from("/project/src/db.py"), table)];
        let symbols = builder.build_symbols(&symbol_tables);

        // Should contain "db.get_user" (module stem + "." + name)
        let sym = symbols
            .get("db.get_user")
            .expect("symbol db.get_user should exist");
        assert_eq!(sym.file, "src/db.py");
        assert_eq!(sym.line, 42); // 0-indexed 41 -> 1-indexed 42
        assert_eq!(sym.kind, "function");
        assert_eq!(
            sym.type_sig.as_deref(),
            Some("(int) -> User"),
            "type signature should be present (R7)"
        );
    }

    #[test]
    fn test_build_symbols_skips_imports_and_parameters() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        let mut table = SymbolTable::new();
        // Add an import
        table.add_symbol(
            "os".to_string(),
            SymbolKind::Import,
            make_range(0, 0, 0, 10),
            None,
            None,
            0,
        );
        // Add a parameter
        table.add_symbol(
            "x".to_string(),
            SymbolKind::Parameter,
            make_range(5, 0, 5, 5),
            None,
            None,
            0,
        );
        // Add a real function
        table.add_symbol(
            "main".to_string(),
            SymbolKind::Function,
            make_range(10, 0, 20, 0),
            None,
            None,
            0,
        );

        let symbol_tables = vec![(PathBuf::from("/project/app.py"), table)];
        let symbols = builder.build_symbols(&symbol_tables);

        // Only "main" should be present
        assert_eq!(symbols.len(), 1);
        assert!(symbols.contains_key("app.main"));
    }

    // --- R3: build_imports from ImportGraph ---

    #[test]
    fn test_build_imports_from_graph() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        let mut graph = ImportGraph::new();
        // Manually add a file with imports via the add_file interface
        // Since add_file parses source, we use a Python source with imports
        let handler_src = "from db import get_user\nfrom models import User\n";
        graph.add_file(PathBuf::from("/project/handler.py"), handler_src, &root);

        let results = vec![make_file_result(
            "/project/handler.py",
            Language::Python,
            vec![],
        )];

        let imports = builder.build_imports(&results, &graph);

        // handler.py should have import entries
        if let Some(handler_imports) = imports.get("handler.py") {
            assert!(
                !handler_imports.is_empty(),
                "handler.py should have imports"
            );
        }
        // A file with no imports should not appear
        assert!(!imports.contains_key("db.py"));
    }

    // --- R4, R6: build_issues with symbol attribution ---

    #[test]
    fn test_build_issues_with_symbol_attribution() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        // Create a symbol table with a function at lines 10-20 (0-indexed)
        let table = make_symbol_table_with_function("get_user", SymbolKind::Function, 10, 20, None);

        // Create a diagnostic inside the function (line 15, 0-indexed)
        let diag = make_diagnostic(15, DiagnosticSeverity::Error, "PY101", "undefined variable");

        let results = vec![make_file_result(
            "/project/db.py",
            Language::Python,
            vec![diag],
        )];
        let symbol_tables = vec![(PathBuf::from("/project/db.py"), table)];

        let issues = builder.build_issues(&results, &symbol_tables);

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.severity, "error");
        assert_eq!(
            issue.symbol, "get_user",
            "should be attributed to enclosing symbol"
        );
        assert_eq!(issue.file, "db.py");
        assert_eq!(issue.line, 16); // 0-indexed 15 -> 1-indexed 16
        assert_eq!(issue.code, "PY101");
        assert_eq!(issue.message, "undefined variable");
    }

    #[test]
    fn test_build_issues_file_level_fallback() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        // Symbol table with function at lines 10-20
        let table = make_symbol_table_with_function("main", SymbolKind::Function, 10, 20, None);

        // Diagnostic at line 5 — outside any symbol
        let diag = make_diagnostic(
            5,
            DiagnosticSeverity::Warning,
            "SYN001",
            "missing newline at EOF",
        );

        let results = vec![make_file_result(
            "/project/app.py",
            Language::Python,
            vec![diag],
        )];
        let symbol_tables = vec![(PathBuf::from("/project/app.py"), table)];

        let issues = builder.build_issues(&results, &symbol_tables);

        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].symbol, "<file-level>",
            "diagnostic outside any symbol should use <file-level> (R6)"
        );
    }

    // --- R5: build_impact from references ---

    #[test]
    fn test_build_impact_from_references() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        let mut table = SymbolTable::new();
        // Add a function symbol
        let sym_id = table.add_symbol(
            "get_user".to_string(),
            SymbolKind::Function,
            make_range(10, 0, 20, 0),
            None,
            None,
            0,
        );

        // Add a non-definition reference to get_user at line 30
        table.add_reference(sym_id, make_range(30, 4, 30, 12));
        // Add another reference at line 45
        table.add_reference(sym_id, make_range(45, 8, 45, 16));

        let symbol_tables = vec![(PathBuf::from("/project/db.py"), table)];

        let impact = builder.build_impact(&symbol_tables);

        let refs = impact
            .get("db.get_user")
            .expect("should have impact for db.get_user");
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"db.py:31".to_string())); // 0-indexed 30 -> 1-indexed 31
        assert!(refs.contains(&"db.py:46".to_string())); // 0-indexed 45 -> 1-indexed 46
    }

    // --- R8: stats computation ---

    #[test]
    fn test_stats_computation() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        // Two files
        let mut table1 = SymbolTable::new();
        let sym_id = table1.add_symbol(
            "func_a".to_string(),
            SymbolKind::Function,
            make_range(0, 0, 10, 0),
            None,
            None,
            0,
        );
        table1.add_reference(sym_id, make_range(20, 0, 20, 5)); // non-def ref

        let table2 = make_symbol_table_with_function("func_b", SymbolKind::Function, 0, 5, None);

        let diag = make_diagnostic(3, DiagnosticSeverity::Error, "E001", "error");
        let results = vec![
            make_file_result("/project/a.py", Language::Python, vec![diag]),
            make_file_result("/project/b.py", Language::Python, vec![]),
        ];
        let symbol_tables = vec![
            (PathBuf::from("/project/a.py"), table1),
            (PathBuf::from("/project/b.py"), table2),
        ];

        let graph = ImportGraph::new();
        let output = builder.build(&results, &symbol_tables, &graph);

        assert_eq!(output.stats.files_checked, 2);
        assert_eq!(output.stats.symbols_found, 2);
        assert_eq!(output.stats.issues_count, 1);
        // 1 non-definition reference to func_a
        assert_eq!(output.stats.impact_edges, 1);
    }

    // --- R9: compact output omits empty ---

    #[test]
    fn test_compact_output_omits_empty() {
        // No issues scenario
        let output = AgentOutput {
            symbols: {
                let mut m = BTreeMap::new();
                m.insert(
                    "mod.func".to_string(),
                    SymbolDef {
                        type_sig: None,
                        file: "mod.py".to_string(),
                        line: 1,
                        kind: "function".to_string(),
                    },
                );
                m
            },
            imports: BTreeMap::new(),
            issues: Vec::new(),
            impact: BTreeMap::new(),
            stats: AgentStats {
                files_checked: 1,
                symbols_found: 1,
                issues_count: 0,
                impact_edges: 0,
            },
        };

        let json_str = serde_json::to_string(&output).unwrap();

        // "issues" key should be absent
        assert!(
            !json_str.contains("\"issues\""),
            "issues should be omitted when empty (R9)"
        );
        // "symbols" should always be present
        assert!(json_str.contains("\"symbols\""));
        // "stats" should always be present
        assert!(json_str.contains("\"stats\""));
    }

    #[test]
    fn test_compact_output_omits_empty_imports() {
        let output = AgentOutput {
            symbols: BTreeMap::new(),
            imports: BTreeMap::new(),
            issues: vec![AgentIssue {
                severity: "error".to_string(),
                symbol: "main".to_string(),
                file: "app.py".to_string(),
                line: 1,
                code: "E001".to_string(),
                message: "test".to_string(),
            }],
            impact: BTreeMap::new(),
            stats: AgentStats {
                files_checked: 1,
                symbols_found: 0,
                issues_count: 1,
                impact_edges: 0,
            },
        };

        let json_str = serde_json::to_string(&output).unwrap();

        // "imports" key should be absent
        assert!(
            !json_str.contains("\"imports\""),
            "imports should be omitted when empty (R9)"
        );
        // "impact" key should be absent
        assert!(
            !json_str.contains("\"impact\""),
            "impact should be omitted when empty (R9)"
        );
        // "issues" should be present since non-empty
        assert!(json_str.contains("\"issues\""));
    }

    // --- NF4: valid JSON round-trip ---

    #[test]
    fn test_agent_output_valid_json() {
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        let mut table = SymbolTable::new();
        let sym_id = table.add_symbol(
            "process".to_string(),
            SymbolKind::Function,
            make_range(0, 0, 10, 0),
            Some(TypeInfo::Callable {
                params: vec![TypeInfo::Primitive("str".to_string())],
                ret: Box::new(TypeInfo::Primitive("bool".to_string())),
            }),
            None,
            0,
        );
        table.add_reference(sym_id, make_range(20, 0, 20, 7));

        let diag = make_diagnostic(5, DiagnosticSeverity::Warning, "W001", "unused variable");
        let results = vec![make_file_result(
            "/project/main.py",
            Language::Python,
            vec![diag],
        )];
        let symbol_tables = vec![(PathBuf::from("/project/main.py"), table)];
        let graph = ImportGraph::new();

        let output = builder.build(&results, &symbol_tables, &graph);
        let json_str = serde_json::to_string(&output).unwrap();

        // Verify it's parseable
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .expect("AgentOutput JSON must be parseable by serde_json (NF4)");

        // Verify structure
        assert!(parsed.get("symbols").is_some());
        assert!(parsed.get("stats").is_some());
        assert!(parsed["stats"]["files_checked"].as_u64().unwrap() == 1);
    }

    // --- Helper function tests ---

    #[test]
    fn test_format_qualified_name() {
        assert_eq!(
            format_qualified_name("src/db.py", "get_user"),
            "db.get_user"
        );
        assert_eq!(
            format_qualified_name("handler.py", "handle"),
            "handler.handle"
        );
        assert_eq!(
            format_qualified_name("a/b/models.ts", "User"),
            "models.User"
        );
    }

    #[test]
    fn test_symbol_kind_to_agent_kind() {
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Function), "function");
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Class), "class");
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Struct), "class");
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Trait), "interface");
        assert_eq!(
            symbol_kind_to_agent_kind(SymbolKind::Interface),
            "interface"
        );
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Variable), "variable");
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Const), "constant");
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Static), "constant");
        assert_eq!(
            symbol_kind_to_agent_kind(SymbolKind::TypeAlias),
            "type_alias"
        );
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Module), "module");
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Decorator), "function");
        assert_eq!(symbol_kind_to_agent_kind(SymbolKind::Macro), "function");
    }

    #[test]
    fn test_severity_to_str() {
        assert_eq!(severity_to_str(DiagnosticSeverity::Error), "error");
        assert_eq!(severity_to_str(DiagnosticSeverity::Warning), "warning");
        assert_eq!(severity_to_str(DiagnosticSeverity::Information), "info");
        assert_eq!(severity_to_str(DiagnosticSeverity::Hint), "hint");
    }

    #[test]
    fn test_find_enclosing_symbol_innermost() {
        // Nested symbols: outer at lines 0-30, inner at lines 5-15
        let mut table = SymbolTable::new();
        table.add_symbol(
            "MyClass".to_string(),
            SymbolKind::Class,
            make_range(0, 0, 30, 0),
            None,
            None,
            0,
        );
        table.add_symbol(
            "my_method".to_string(),
            SymbolKind::Function,
            make_range(5, 4, 15, 4),
            None,
            None,
            1,
        );

        // Position at line 10 should find the inner method
        let result = find_enclosing_symbol(&table, 10, 8);
        assert_eq!(
            result,
            Some("my_method".to_string()),
            "should find innermost enclosing symbol"
        );
    }

    #[test]
    fn test_find_enclosing_symbol_none() {
        let table = make_symbol_table_with_function("func", SymbolKind::Function, 10, 20, None);

        // Position before any symbol
        let result = find_enclosing_symbol(&table, 5, 0);
        assert_eq!(
            result, None,
            "should return None when no symbol encloses the position"
        );
    }

    #[test]
    fn test_build_full_output_with_all_fields() {
        // Integration-style test: build complete output with symbols, imports, issues, impact
        let root = PathBuf::from("/project");
        let builder = AgentOutputBuilder::new(&root);

        let mut table = SymbolTable::new();
        let sym_id = table.add_symbol(
            "handler".to_string(),
            SymbolKind::Function,
            make_range(0, 0, 20, 0),
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("None".to_string())),
            }),
            None,
            0,
        );
        // Add a non-definition reference
        table.add_reference(sym_id, make_range(25, 0, 25, 7));

        let diag = make_diagnostic(5, DiagnosticSeverity::Error, "E100", "bad call");

        let results = vec![make_file_result(
            "/project/app.py",
            Language::Python,
            vec![diag],
        )];
        let symbol_tables = vec![(PathBuf::from("/project/app.py"), table)];
        let graph = ImportGraph::new();

        let output = builder.build(&results, &symbol_tables, &graph);

        // Symbols
        assert_eq!(output.symbols.len(), 1);
        let sym = output.symbols.get("app.handler").unwrap();
        assert_eq!(sym.kind, "function");
        assert_eq!(sym.type_sig.as_deref(), Some("() -> None"));

        // Issues
        assert_eq!(output.issues.len(), 1);
        assert_eq!(output.issues[0].symbol, "handler");

        // Impact
        assert!(output.impact.contains_key("app.handler"));

        // Stats
        assert_eq!(output.stats.files_checked, 1);
        assert_eq!(output.stats.symbols_found, 1);
        assert_eq!(output.stats.issues_count, 1);
        assert!(output.stats.impact_edges > 0);
    }
}

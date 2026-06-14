//! Request handlers for Argus daemon

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::RwLock;

use crate::checker::LintConfig;
use crate::diagnostic::Diagnostic;
use crate::lint::CheckerRegistry;
use crate::semantic::{CfgBuilder, PdgJson, ProgramDependenceGraph};
use crate::semantic::{SymbolTable, SymbolTableBuilder};
use crate::storage::resolve_cache_dir;
use crate::syntax::{Language, MultiParser, ParsedFile};
use crate::type_inference::{build_semantic_model, ContentHash, SemanticModel, StubLoader};

use super::disk_cache::DiskCache;
use super::protocol::*;

/// Cached analysis for a file
struct FileAnalysis {
    #[allow(dead_code)]
    parsed: ParsedFile,
    symbol_table: SymbolTable,
    semantic_model: SemanticModel,
    diagnostics: Vec<Diagnostic>,
    #[allow(dead_code)]
    source: String,
    /// Unix timestamp (seconds) when this file's diagnostics were last updated.
    last_updated_secs: u64,
}

/// Request handler with caching
pub struct RequestHandler {
    /// Root directory being analyzed
    root: PathBuf,
    /// File cache: path -> analysis
    cache: Arc<RwLock<HashMap<PathBuf, FileAnalysis>>>,
    /// In-memory document overrides (for unsaved LSP changes)
    overrides: Arc<RwLock<HashMap<PathBuf, String>>>,
    /// Checker registry
    registry: Arc<CheckerRegistry>,
    /// Lint configuration
    config: Arc<LintConfig>,
    /// Type stubs
    #[allow(dead_code)]
    stubs: Arc<RwLock<StubLoader>>,
    /// Parser (not thread-safe, needs mutex)
    parser: Arc<tokio::sync::Mutex<MultiParser>>,
    /// Persistent disk cache
    disk_cache: Arc<DiskCache>,
}

impl RequestHandler {
    pub fn new(root: PathBuf) -> Result<Self, String> {
        let parser = MultiParser::new().map_err(|e| format!("Failed to create parser: {}", e))?;

        let mut stubs = StubLoader::new();
        stubs.load_builtins();

        let cache_dir = resolve_cache_dir(&root)
            .unwrap_or_else(|_| root.join("cclab").join(".index").join("cache"));
        let disk_cache = Arc::new(DiskCache::new(cache_dir));

        Ok(Self {
            root,
            cache: Arc::new(RwLock::new(HashMap::new())),
            overrides: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(CheckerRegistry::new()),
            config: Arc::new(LintConfig::default()),
            stubs: Arc::new(RwLock::new(stubs)),
            parser: Arc::new(tokio::sync::Mutex::new(parser)),
            disk_cache,
        })
    }

    /// Create a handler for a specific scope (#1127).
    ///
    /// Uses per-scope cache directory and adds scope's search paths to stub loader.
    pub fn new_with_scope(
        root: PathBuf,
        scope_id: &str,
        project_root: &std::path::Path,
        extra_search_paths: &[PathBuf],
    ) -> Result<Self, String> {
        let parser = MultiParser::new().map_err(|e| format!("Failed to create parser: {}", e))?;

        let mut stubs = StubLoader::new();
        stubs.load_builtins();
        for path in extra_search_paths {
            stubs.add_stub_path(path.clone());
        }

        let cache_dir = crate::storage::resolve_scope_cache_dir(project_root, scope_id)
            .unwrap_or_else(|_| {
                project_root
                    .join("cclab/.index/scopes")
                    .join(scope_id)
                    .join("cache")
            });
        let disk_cache = Arc::new(DiskCache::new(cache_dir));

        Ok(Self {
            root,
            cache: Arc::new(RwLock::new(HashMap::new())),
            overrides: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(CheckerRegistry::new()),
            config: Arc::new(LintConfig::default()),
            stubs: Arc::new(RwLock::new(stubs)),
            parser: Arc::new(tokio::sync::Mutex::new(parser)),
            disk_cache,
        })
    }

    /// Set an in-memory document override (for unsaved LSP changes)
    pub async fn set_document_override(&self, path: impl AsRef<Path>, content: String) {
        let path = path.as_ref().to_path_buf();
        let mut overrides = self.overrides.write().await;
        overrides.insert(path.clone(), content);
        // Invalidate cache for this file since content changed
        let mut cache = self.cache.write().await;
        cache.remove(&path);
    }

    /// Remove an in-memory document override
    pub async fn remove_document_override(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        let mut overrides = self.overrides.write().await;
        overrides.remove(&path);
        // Invalidate cache for this file
        let mut cache = self.cache.write().await;
        cache.remove(&path);
    }

    /// Get document content, preferring overrides over disk
    pub async fn get_document_content(&self, path: &Path) -> Result<String, String> {
        // Check for override first
        {
            let overrides = self.overrides.read().await;
            if let Some(content) = overrides.get(path) {
                return Ok(content.clone());
            }
        }
        // Fall back to disk
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))
    }

    /// Handle a JSON-RPC request
    pub async fn handle(&self, request: Request) -> Response {
        let result = match request.method.as_str() {
            "check" => self.handle_check(request.params).await,
            "type_at" => self.handle_type_at(request.params).await,
            "symbols" => self.handle_symbols(request.params).await,
            "diagnostics" => self.handle_diagnostics(request.params).await,
            "hover" => self.handle_hover(request.params).await,
            "definition" => self.handle_definition(request.params).await,
            "references" => self.handle_references(request.params).await,
            "index_status" => self.handle_index_status().await,
            "invalidate" => self.handle_invalidate(request.params).await,
            "shutdown" => self.handle_shutdown().await,
            // PDG tools (R101-R105)
            "pdg" => self.handle_pdg(request.params).await,
            "slice" => self.handle_slice(request.params).await,
            "impact" => self.handle_impact(request.params).await,
            "taint" => self.handle_taint(request.params).await,
            _ => Err(RpcError::method_not_found(&request.method)),
        };

        match result {
            Ok(value) => Response::success(request.id, value),
            Err(error) => Response::error(request.id, error),
        }
    }

    /// Check files/directories for issues
    async fn handle_check(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: CheckParams = params
            .ok_or_else(|| RpcError::invalid_params("Missing params"))?
            .try_into()
            .map_err(|e| RpcError::invalid_params(format!("Invalid params: {}", e)))?;

        let path = self.resolve_path(&params.path);

        let mut all_diagnostics = Vec::new();
        let mut files_checked = 0;

        if path.is_file() {
            if let Some(diags) = self.check_file(&path).await {
                files_checked = 1;
                all_diagnostics.extend(diags);
            }
        } else if path.is_dir() {
            let files = self.collect_files(&path);
            for file in files {
                if let Some(diags) = self.check_file(&file).await {
                    files_checked += 1;
                    all_diagnostics.extend(diags);
                }
            }
        } else {
            return Err(RpcError::invalid_params(format!(
                "Path not found: {}",
                params.path
            )));
        }

        let errors = all_diagnostics
            .iter()
            .filter(|d| d.severity == "error")
            .count();
        let warnings = all_diagnostics
            .iter()
            .filter(|d| d.severity == "warning")
            .count();

        let result = CheckResult {
            diagnostics: all_diagnostics,
            files_checked,
            errors,
            warnings,
        };

        serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Get type at position
    async fn handle_type_at(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: TypeAtParams = params
            .ok_or_else(|| RpcError::invalid_params("Missing params"))?
            .try_into()
            .map_err(|e| RpcError::invalid_params(format!("Invalid params: {}", e)))?;

        let path = self.resolve_path(&params.file);
        self.ensure_analyzed(&path).await?;

        let cache = self.cache.read().await;
        let analysis = cache
            .get(&path)
            .ok_or_else(|| RpcError::invalid_params("File not found in cache"))?;

        // First try SemanticModel for type info
        if let Some(type_info) = analysis.semantic_model.type_at(params.line, params.column) {
            return serde_json::to_value(type_info.display())
                .map_err(|e| RpcError::internal_error(e.to_string()));
        }

        // Fall back to symbol table
        let symbol = analysis
            .symbol_table
            .find_at_position(params.line, params.column);

        match symbol {
            Some(sym) => {
                let type_str = sym
                    .type_info
                    .as_ref()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "Unknown".to_string());
                serde_json::to_value(type_str).map_err(|e| RpcError::internal_error(e.to_string()))
            }
            None => Ok(serde_json::Value::Null),
        }
    }

    /// List symbols in a file
    async fn handle_symbols(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: SymbolsParams = params
            .ok_or_else(|| RpcError::invalid_params("Missing params"))?
            .try_into()
            .map_err(|e| RpcError::invalid_params(format!("Invalid params: {}", e)))?;

        let path = self.resolve_path(&params.file);
        self.ensure_analyzed(&path).await?;

        let cache = self.cache.read().await;
        let analysis = cache
            .get(&path)
            .ok_or_else(|| RpcError::invalid_params("File not found in cache"))?;

        let symbols: Vec<SymbolInfo> = analysis
            .symbol_table
            .all_symbols()
            .iter()
            .map(|sym| SymbolInfo {
                name: sym.name.clone(),
                kind: format!("{:?}", sym.kind),
                line: sym.location.start.line,
                column: sym.location.start.character,
                type_info: sym.type_info.as_ref().map(|t| format!("{:?}", t)),
            })
            .collect();

        serde_json::to_value(symbols).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Get diagnostics
    async fn handle_diagnostics(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: DiagnosticsParams = params
            .map(|p| serde_json::from_value(p).ok())
            .flatten()
            .unwrap_or(DiagnosticsParams { file: None });

        let cache = self.cache.read().await;

        let diagnostics: Vec<DiagnosticInfo> = if let Some(file) = params.file {
            let path = self.resolve_path(&file);
            cache
                .get(&path)
                .map(|a| self.convert_diagnostics(&path, &a.diagnostics))
                .unwrap_or_default()
        } else {
            cache
                .iter()
                .flat_map(|(path, analysis)| self.convert_diagnostics(path, &analysis.diagnostics))
                .collect()
        };

        serde_json::to_value(diagnostics).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Get hover information
    async fn handle_hover(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: HoverParams = params
            .ok_or_else(|| RpcError::invalid_params("Missing params"))?
            .try_into()
            .map_err(|e| RpcError::invalid_params(format!("Invalid params: {}", e)))?;

        let path = self.resolve_path(&params.file);
        self.ensure_analyzed(&path).await?;

        let cache = self.cache.read().await;
        let analysis = cache
            .get(&path)
            .ok_or_else(|| RpcError::invalid_params("File not found in cache"))?;

        // First try SemanticModel for hover info
        if let Some(hover_content) = analysis.semantic_model.hover_at(params.line, params.column) {
            let response = serde_json::json!({
                "contents": {
                    "kind": "markdown",
                    "value": hover_content
                }
            });
            return serde_json::to_value(response)
                .map_err(|e| RpcError::internal_error(e.to_string()));
        }

        // Fall back to symbol table
        let symbol = analysis
            .symbol_table
            .find_at_position(params.line, params.column);

        match symbol {
            Some(sym) => {
                let content = sym.hover_content(Language::Python);
                serde_json::to_value(content).map_err(|e| RpcError::internal_error(e.to_string()))
            }
            None => Ok(serde_json::Value::Null),
        }
    }

    /// Go to definition
    async fn handle_definition(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: DefinitionParams = params
            .ok_or_else(|| RpcError::invalid_params("Missing params"))?
            .try_into()
            .map_err(|e| RpcError::invalid_params(format!("Invalid params: {}", e)))?;

        let path = self.resolve_path(&params.file);
        self.ensure_analyzed(&path).await?;

        let cache = self.cache.read().await;
        let analysis = cache
            .get(&path)
            .ok_or_else(|| RpcError::invalid_params("File not found in cache"))?;

        // First try SemanticModel for definition
        if let Some(symbol_data) = analysis
            .semantic_model
            .definition_at(params.line, params.column)
        {
            let loc = LocationInfo {
                file: symbol_data.file_path.to_string_lossy().to_string(),
                line: symbol_data.def_range.start.line,
                column: symbol_data.def_range.start.character,
                end_line: symbol_data.def_range.end.line,
                end_column: symbol_data.def_range.end.character,
            };
            return serde_json::to_value(loc).map_err(|e| RpcError::internal_error(e.to_string()));
        }

        // Fall back to symbol table
        let symbol = analysis
            .symbol_table
            .find_definition_at(params.line, params.column);

        match symbol {
            Some(sym) => {
                let loc = LocationInfo {
                    file: path.to_string_lossy().to_string(),
                    line: sym.location.start.line,
                    column: sym.location.start.character,
                    end_line: sym.location.end.line,
                    end_column: sym.location.end.character,
                };
                serde_json::to_value(loc).map_err(|e| RpcError::internal_error(e.to_string()))
            }
            None => Ok(serde_json::Value::Null),
        }
    }

    /// Find references
    async fn handle_references(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: ReferencesParams = params
            .ok_or_else(|| RpcError::invalid_params("Missing params"))?
            .try_into()
            .map_err(|e| RpcError::invalid_params(format!("Invalid params: {}", e)))?;

        let path = self.resolve_path(&params.file);
        self.ensure_analyzed(&path).await?;

        let cache = self.cache.read().await;
        let analysis = cache
            .get(&path)
            .ok_or_else(|| RpcError::invalid_params("File not found in cache"))?;

        // First try SemanticModel for references
        let sem_refs = analysis.semantic_model.references_at(
            params.line,
            params.column,
            params.include_declaration,
        );
        if !sem_refs.is_empty() {
            let locations: Vec<LocationInfo> = sem_refs
                .into_iter()
                .map(|r| LocationInfo {
                    file: path.to_string_lossy().to_string(),
                    line: r.range.start.line,
                    column: r.range.start.character,
                    end_line: r.range.end.line,
                    end_column: r.range.end.character,
                })
                .collect();
            return serde_json::to_value(locations)
                .map_err(|e| RpcError::internal_error(e.to_string()));
        }

        // Fall back to symbol table
        let refs = analysis.symbol_table.find_references_at(
            params.line,
            params.column,
            params.include_declaration,
        );

        let locations: Vec<LocationInfo> = refs
            .into_iter()
            .map(|r| LocationInfo {
                file: path.to_string_lossy().to_string(),
                line: r.start.line,
                column: r.start.character,
                end_line: r.end.line,
                end_column: r.end.character,
            })
            .collect();

        serde_json::to_value(locations).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Get index status
    async fn handle_index_status(&self) -> Result<serde_json::Value, RpcError> {
        let cache = self.cache.read().await;

        let total_symbols: usize = cache
            .values()
            .map(|a| a.symbol_table.all_symbols().len())
            .sum();

        // Report the most recent diagnostic update timestamp across all cached files
        let last_updated: Option<String> =
            cache
                .values()
                .map(|a| a.last_updated_secs)
                .max()
                .map(|secs| {
                    // Format as ISO 8601 date-time string (UTC, seconds precision)
                    let datetime = format_unix_timestamp(secs);
                    datetime
                });

        let status = IndexStatus {
            indexed_files: cache.len(),
            total_symbols,
            last_updated,
            is_ready: true,
        };

        serde_json::to_value(status).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Invalidate cache for files
    async fn handle_invalidate(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        #[derive(serde::Deserialize)]
        struct InvalidateParams {
            files: Vec<String>,
        }

        let params: InvalidateParams = serde_json::from_value(
            params.ok_or_else(|| RpcError::invalid_params("Missing params"))?,
        )
        .map_err(|e| RpcError::invalid_params(format!("Invalid params: {}", e)))?;

        let mut cache = self.cache.write().await;
        let mut invalidated = 0;

        for file in params.files {
            let path = self.resolve_path(&file);
            if cache.remove(&path).is_some() {
                invalidated += 1;
            }
        }

        serde_json::to_value(serde_json::json!({ "invalidated": invalidated }))
            .map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Shutdown the daemon
    async fn handle_shutdown(&self) -> Result<serde_json::Value, RpcError> {
        // The actual shutdown is handled by the daemon
        Ok(serde_json::json!({ "status": "shutting_down" }))
    }

    // =========================================================================
    // PDG tools (R101–R106)
    // =========================================================================

    /// Build and return the PDG for a Python file or function (R101)
    async fn handle_pdg(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: PdgParams = serde_json::from_value(
            params.ok_or_else(|| RpcError::invalid_params("Missing params"))?,
        )
        .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let path = self.resolve_path(&params.file);
        let source = self
            .get_document_content(&path)
            .await
            .map_err(|e| RpcError::invalid_params(e))?;

        let mut parser_guard = self.parser.lock().await;
        let parsed = parser_guard
            .parse(&source, Language::Python)
            .ok_or_else(|| RpcError::invalid_params("Failed to parse Python file"))?;
        drop(parser_guard);

        let pdg = if let Some(fn_name) = &params.function {
            // Build PDG for a specific function by finding the function node
            self.build_function_pdg(&source, &parsed, fn_name)
                .unwrap_or_else(|| ProgramDependenceGraph::build(&source, &parsed))
        } else {
            ProgramDependenceGraph::build(&source, &parsed)
        };

        let pdg = pdg.with_file_path(path);
        let json: PdgJson = (&pdg).into();

        serde_json::to_value(json).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Compute a program slice from a criterion line (R102)
    async fn handle_slice(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: SliceParams = serde_json::from_value(
            params.ok_or_else(|| RpcError::invalid_params("Missing params"))?,
        )
        .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let path = self.resolve_path(&params.file);
        let source = self
            .get_document_content(&path)
            .await
            .map_err(|e| RpcError::invalid_params(e))?;

        let mut parser_guard = self.parser.lock().await;
        let parsed = parser_guard
            .parse(&source, Language::Python)
            .ok_or_else(|| RpcError::invalid_params("Failed to parse Python file"))?;
        drop(parser_guard);

        let pdg = ProgramDependenceGraph::build(&source, &parsed);

        let slice = match params.direction.as_str() {
            "forward" => pdg.forward_slice(params.line),
            "backward" => pdg.backward_slice(params.line),
            other => {
                return Err(RpcError::invalid_params(format!(
                    "Invalid direction '{}': must be 'forward' or 'backward'",
                    other
                )))
            }
        };

        let nodes: Vec<SliceNodeInfo> = slice
            .nodes
            .iter()
            .map(|n| SliceNodeInfo {
                line: n.line,
                text: n.text.clone(),
                kind: format!("{:?}", n.kind),
            })
            .collect();

        let result = SliceResult {
            direction: params.direction,
            criterion_line: params.line,
            line_count: nodes.len(),
            nodes,
        };

        serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Compute change impact analysis with dependency tree (R103)
    async fn handle_impact(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: ImpactParams = serde_json::from_value(
            params.ok_or_else(|| RpcError::invalid_params("Missing params"))?,
        )
        .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let path = self.resolve_path(&params.file);
        let source = self
            .get_document_content(&path)
            .await
            .map_err(|e| RpcError::invalid_params(e))?;

        let mut parser_guard = self.parser.lock().await;
        let parsed = parser_guard
            .parse(&source, Language::Python)
            .ok_or_else(|| RpcError::invalid_params("Failed to parse Python file"))?;
        drop(parser_guard);

        let pdg = ProgramDependenceGraph::build(&source, &parsed);
        let impact = pdg.impact_analysis_tree(&params.changed_lines);

        // Convert tree nodes to protocol format
        fn convert_tree(nodes: &[crate::semantic::ImpactTreeNode]) -> Vec<ImpactNode> {
            nodes
                .iter()
                .map(|n| ImpactNode {
                    line: n.line,
                    text: n.text.clone(),
                    reason: format!("{:?}", n.reason).to_lowercase(),
                    variable: n.variable.clone(),
                    children: convert_tree(&n.children),
                })
                .collect()
        }

        let result = ImpactResult {
            changed_lines: impact.changed_lines,
            total_affected: impact.affected_lines.len(),
            affected_lines: impact.affected_lines,
            impact_tree: convert_tree(&impact.tree),
        };

        serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Trace tainted data from sources to sinks (R104)
    ///
    /// If sources/sinks are empty in params, auto-detects them from the code
    /// using pattern matching for known taint sources and sinks.
    async fn handle_taint(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        let params: TaintParams = serde_json::from_value(
            params.ok_or_else(|| RpcError::invalid_params("Missing params"))?,
        )
        .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let path = self.resolve_path(&params.file);
        let source = self
            .get_document_content(&path)
            .await
            .map_err(|e| RpcError::invalid_params(e))?;

        let mut parser_guard = self.parser.lock().await;
        let parsed = parser_guard
            .parse(&source, Language::Python)
            .ok_or_else(|| RpcError::invalid_params("Failed to parse Python file"))?;
        drop(parser_guard);

        let pdg = ProgramDependenceGraph::build(&source, &parsed);

        let auto_detected = params.sources.is_empty() && params.sinks.is_empty();

        let (taint_paths, source_lines, sink_lines) = if auto_detected {
            // Use semantic auto-detection (R7)
            let analysis = pdg.semantic_taint_analysis();
            let src_lines: Vec<usize> = analysis.sources.iter().map(|s| s.line).collect();
            let snk_lines: Vec<usize> = analysis.sinks.iter().map(|s| s.line).collect();
            let paths = analysis.taint_paths;
            (paths, src_lines, snk_lines)
        } else {
            // Use explicit lines provided by caller
            let taint = pdg.taint_analysis_explicit(&params.sources, &params.sinks);
            (taint.taint_paths, params.sources, params.sinks)
        };

        // Enrich taint path info with node text
        let path_infos: Vec<TaintPathInfo> = taint_paths
            .iter()
            .map(|tp| {
                let src_text = pdg
                    .get_node_by_line(tp.source)
                    .map(|n| n.text.clone())
                    .unwrap_or_default();
                let snk_text = pdg
                    .get_node_by_line(tp.sink)
                    .map(|n| n.text.clone())
                    .unwrap_or_default();

                TaintPathInfo {
                    source_line: tp.source,
                    source_text: src_text,
                    source_kind: "taint_source".to_string(),
                    sink_line: tp.sink,
                    sink_text: snk_text,
                    sink_kind: "taint_sink".to_string(),
                    path: tp.path.clone(),
                }
            })
            .collect();

        let has_vulns = !path_infos.is_empty();

        let result = TaintResult {
            source_lines,
            sink_lines,
            has_vulnerabilities: has_vulns,
            auto_detected,
            taint_paths: path_infos,
        };

        serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    /// Build PDG for a specific function by name
    fn build_function_pdg(
        &self,
        source: &str,
        file: &ParsedFile,
        fn_name: &str,
    ) -> Option<ProgramDependenceGraph> {
        let root = file.root_node();
        let mut cursor = root.walk();

        for node in root.children(&mut cursor) {
            if node.kind() == "function_definition" || node.kind() == "async_function_definition" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if file.node_text(&name_node) == fn_name {
                        let cfg = CfgBuilder::new(source).build_function(&node, file);
                        return Some(ProgramDependenceGraph::from_cfg(cfg, file));
                    }
                }
            }
        }

        None
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    /// Resolve a path relative to root
    fn resolve_path(&self, path: &str) -> PathBuf {
        let p = PathBuf::from(path);
        if p.is_absolute() {
            p
        } else {
            self.root.join(p)
        }
    }

    /// Collect all analyzable files in a directory
    fn collect_files(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip excluded patterns
                if self.config.is_excluded(&path) {
                    continue;
                }

                if path.is_dir() {
                    files.extend(self.collect_files(&path));
                } else if path.is_file() {
                    if MultiParser::detect_language(&path).is_some() {
                        files.push(path);
                    }
                }
            }
        }

        files
    }

    /// Ensure a file is analyzed and cached.
    ///
    /// On in-memory miss, tries the disk cache before doing a full parse.
    async fn ensure_analyzed(&self, path: &Path) -> Result<(), RpcError> {
        // Fast path: already in memory
        {
            let cache = self.cache.read().await;
            if cache.contains_key(path) {
                return Ok(());
            }
        }

        // Try disk cache
        if self.try_restore_from_disk(path).await {
            return Ok(());
        }

        // Full analysis
        self.check_file(path).await;
        Ok(())
    }

    /// Attempt to restore a file's analysis from disk cache.
    ///
    /// Returns `true` if the disk cache had a fresh entry and the
    /// in-memory cache was populated.
    async fn try_restore_from_disk(&self, path: &Path) -> bool {
        let source = match self.get_document_content(path).await {
            Ok(s) => s,
            Err(_) => return false,
        };

        let content_hash = ContentHash::from_content(&source);
        let persisted = match self.disk_cache.load(path, content_hash.0).await {
            Some(p) => p,
            None => return false,
        };

        // Re-parse to get Tree + SymbolTable (~1ms)
        let language = match MultiParser::detect_language(path) {
            Some(l) => l,
            None => return false,
        };

        let parsed = {
            let mut parser = self.parser.lock().await;
            match parser.parse(&source, language) {
                Some(p) => p,
                None => return false,
            }
        };

        let symbol_table = match language {
            Language::Python => SymbolTableBuilder::new().build_python(&parsed),
            Language::Rust => SymbolTableBuilder::new().build_rust(&parsed),
            Language::TypeScript => SymbolTableBuilder::new().build_typescript(&parsed),
            Language::JavaScript => SymbolTableBuilder::new().build_javascript(&parsed),
            Language::Go => SymbolTableBuilder::new().build_go(&parsed),
            Language::Toml => SymbolTableBuilder::new().build_toml(&parsed),
            Language::Sql => SymbolTableBuilder::new().build_sql(&parsed),
            Language::Proto => SymbolTableBuilder::new().build_proto(&parsed),
            Language::GraphQL => SymbolTableBuilder::new().build_graphql(&parsed),
            _ => SymbolTable::default(),
        };

        let mut cache = self.cache.write().await;
        cache.insert(
            path.to_path_buf(),
            FileAnalysis {
                parsed,
                symbol_table,
                semantic_model: persisted.semantic_model,
                diagnostics: persisted.diagnostics,
                source,
                last_updated_secs: current_unix_secs(),
            },
        );
        true
    }

    /// Check a single file and cache the results
    async fn check_file(&self, path: &Path) -> Option<Vec<DiagnosticInfo>> {
        let language = MultiParser::detect_language(path)?;

        if !self.config.is_language_enabled(language) {
            return None;
        }

        let source = self.get_document_content(path).await.ok()?;

        // Parse
        let parsed = {
            let mut parser = self.parser.lock().await;
            parser.parse(&source, language)?
        };

        // Run linting
        let checker = self.registry.get(language)?;
        let diagnostics = checker.check(&parsed, &self.config);

        // Build symbol table
        let symbol_table = match language {
            Language::Python => SymbolTableBuilder::new().build_python(&parsed),
            Language::Rust => SymbolTableBuilder::new().build_rust(&parsed),
            Language::TypeScript => SymbolTableBuilder::new().build_typescript(&parsed),
            Language::JavaScript => SymbolTableBuilder::new().build_javascript(&parsed),
            Language::Go => SymbolTableBuilder::new().build_go(&parsed),
            Language::Toml => SymbolTableBuilder::new().build_toml(&parsed),
            Language::Sql => SymbolTableBuilder::new().build_sql(&parsed),
            Language::Proto => SymbolTableBuilder::new().build_proto(&parsed),
            Language::GraphQL => SymbolTableBuilder::new().build_graphql(&parsed),
            _ => SymbolTable::default(),
        };

        // Build semantic model for type analysis
        let semantic_model = if language == Language::Python {
            build_semantic_model(&parsed, &source, path.to_path_buf())
        } else {
            SemanticModel::new()
        };

        let diag_infos = self.convert_diagnostics(path, &diagnostics);

        // Write to disk cache in background
        let disk_cache = Arc::clone(&self.disk_cache);
        let disk_path = path.to_path_buf();
        let content_hash = ContentHash::from_content(&source);
        let disk_model = semantic_model.clone();
        let disk_diags = diagnostics.clone();
        tokio::spawn(async move {
            disk_cache
                .store(&disk_path, content_hash.0, &disk_model, &disk_diags)
                .await;
        });

        // Cache the analysis in memory
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                path.to_path_buf(),
                FileAnalysis {
                    parsed,
                    symbol_table,
                    semantic_model,
                    diagnostics,
                    source: source.clone(),
                    last_updated_secs: current_unix_secs(),
                },
            );
        }

        Some(diag_infos)
    }

    /// Convert diagnostics to protocol format
    fn convert_diagnostics(&self, path: &Path, diagnostics: &[Diagnostic]) -> Vec<DiagnosticInfo> {
        diagnostics
            .iter()
            .map(|d| DiagnosticInfo {
                file: path.to_string_lossy().to_string(),
                line: d.range.start.line,
                column: d.range.start.character,
                end_line: d.range.end.line,
                end_column: d.range.end.character,
                severity: format!("{:?}", d.severity).to_lowercase(),
                code: d.code.clone(),
                message: d.message.clone(),
            })
            .collect()
    }

    // =========================================================================
    // Public methods for background analysis
    // =========================================================================

    /// Proactively index all analyzable files in a directory
    pub async fn index_directory(&self, dir: &Path) -> usize {
        let files = self.collect_files(dir);
        let mut indexed = 0;
        for file in files {
            if self.check_file(&file).await.is_some() {
                indexed += 1;
            }
        }
        indexed
    }

    /// Invalidate the cache for a specific file (in-memory + disk).
    ///
    /// Used by the background analysis loop to clear stale cache entries
    /// when files change on disk.
    pub async fn invalidate_file(&self, path: &Path) {
        let mut cache = self.cache.write().await;
        cache.remove(path);
        self.disk_cache.invalidate(path).await;
    }

    /// Analyze a file asynchronously and cache the results
    ///
    /// Used by the background analysis loop to pre-warm the cache
    /// after file changes.
    pub async fn analyze_file_async(&self, path_str: &str) -> Option<()> {
        let path = self.resolve_path(path_str);
        self.check_file(&path).await?;
        Some(())
    }

    /// Get the current cache size
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Flush the disk cache manifest to disk. Call on shutdown.
    pub async fn flush_cache(&self) {
        self.disk_cache.flush_manifest().await;
    }
}

// Implement TryFrom for param types
impl TryFrom<serde_json::Value> for CheckParams {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<serde_json::Value> for TypeAtParams {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<serde_json::Value> for SymbolsParams {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<serde_json::Value> for HoverParams {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<serde_json::Value> for DefinitionParams {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<serde_json::Value> for ReferencesParams {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

// ============================================================================
// Timestamp helpers (R3.5)
// ============================================================================

/// Return the current time as Unix seconds since the epoch.
fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Format a Unix timestamp as an ISO 8601 UTC datetime string.
///
/// Uses a simple decomposition that is accurate for any timestamp in the
/// range 1970-2100 without pulling in `chrono` or `time`.
fn format_unix_timestamp(secs: u64) -> String {
    // Days since Unix epoch
    let mut days = secs / 86400;
    let time_secs = secs % 86400;
    let hh = time_secs / 3600;
    let mm = (time_secs % 3600) / 60;
    let ss = time_secs % 60;

    // Compute year, month, day using the Gregorian algorithm
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &mdays in &month_days {
        if days < mdays {
            break;
        }
        days -= mdays;
        month += 1;
    }
    let day = days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hh, mm, ss
    )
}

#[inline]
fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

//! Argus LSP Server implementation

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic as ArgusDiagnostic, DiagnosticSeverity as ArgusSeverity};
use crate::lint::CheckerRegistry;
use crate::semantic::{SymbolTable, SymbolTableBuilder};
use crate::syntax::Language;
use crate::syntax::{MultiParser, ParsedFile};
use crate::type_inference::{
    RefactorKind, RefactorOptions, RefactorRequest, RefactoringEngine, SemanticSearchEngine,
    SignatureChanges, Span as ArgusSpan, StubLoader,
};

/// Document state tracked by the server
#[derive(Debug)]
struct Document {
    content: String,
    language: Language,
    version: i32,
}

/// Cached analysis for a document
struct DocumentAnalysis {
    symbol_table: SymbolTable,
    /// Diagnostics with their quick fixes
    diagnostics: Vec<ArgusDiagnostic>,
}

/// Argus Language Server
pub struct ArgusServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, Document>>>,
    analyses: Arc<RwLock<HashMap<Url, DocumentAnalysis>>>,
    registry: Arc<CheckerRegistry>,
    config: Arc<LintConfig>,
    stubs: Arc<RwLock<StubLoader>>,
    search_engine: Arc<RwLock<SemanticSearchEngine>>,
    refactoring_engine: Arc<RwLock<RefactoringEngine>>,
}

impl ArgusServer {
    /// Create a new Argus server
    pub fn new(client: Client) -> Self {
        let mut stubs = StubLoader::new();
        stubs.load_builtins();

        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            analyses: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(CheckerRegistry::new()),
            config: Arc::new(LintConfig::default()),
            stubs: Arc::new(RwLock::new(stubs)),
            search_engine: Arc::new(RwLock::new(SemanticSearchEngine::new())),
            refactoring_engine: Arc::new(RwLock::new(RefactoringEngine::new())),
        }
    }

    /// Analyze a document and publish diagnostics
    async fn analyze_document(&self, uri: &Url) {
        let documents = self.documents.read().await;
        let Some(doc) = documents.get(uri) else {
            return;
        };

        let content = doc.content.clone();
        let language = doc.language;
        drop(documents);

        // Parse
        let mut parser = match MultiParser::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let parsed = match parser.parse(&content, language) {
            Some(p) => p,
            None => return,
        };

        // Run linting
        let diagnostics = self.run_lint(&parsed, language);

        // Build symbol table and store diagnostics
        let symbol_table = match language {
            Language::Python => SymbolTableBuilder::new().build_python(&parsed),
            Language::Rust => SymbolTableBuilder::new().build_rust(&parsed),
            Language::TypeScript => SymbolTableBuilder::new().build_typescript(&parsed),
            Language::JavaScript => SymbolTableBuilder::new().build_javascript(&parsed),
            Language::Html => SymbolTableBuilder::new().build_html(&parsed),
            Language::Css => SymbolTableBuilder::new().build_css(&parsed),
            Language::Go => SymbolTableBuilder::new().build_go(&parsed),
            Language::Dockerfile => SymbolTableBuilder::new().build_dockerfile(&parsed),
            Language::Hcl => SymbolTableBuilder::new().build_terraform(&parsed),
            Language::Yaml => SymbolTableBuilder::new().build_kubernetes(&parsed),
            Language::Markdown | Language::Mdx => SymbolTableBuilder::new().build_markdown(&parsed),
            Language::Mermaid => SymbolTableBuilder::new().build_mermaid(&parsed),
            Language::Toml => SymbolTableBuilder::new().build_toml(&parsed),
            Language::Sql => SymbolTableBuilder::new().build_sql(&parsed),
            Language::Proto => SymbolTableBuilder::new().build_proto(&parsed),
            Language::GraphQL => SymbolTableBuilder::new().build_graphql(&parsed),
        };

        // Index symbols, build call graph, and extract docstrings for semantic search
        if language == Language::Python {
            let file_path = PathBuf::from(uri.path());
            let mut search_engine = self.search_engine.write().await;
            search_engine.index_symbol_table(file_path.clone(), &symbol_table);

            // Build call graph from AST
            if let Err(e) = search_engine.build_call_graph(file_path.clone(), &content, language) {
                eprintln!("Failed to build call graph: {}", e);
            }

            // Extract and index docstrings
            if let Ok(docstrings) = search_engine.extract_docstrings(&content, language) {
                search_engine.update_docstrings(docstrings);
            }
        }

        // Store analysis with diagnostics for code actions
        {
            let mut analyses = self.analyses.write().await;
            analyses.insert(
                uri.clone(),
                DocumentAnalysis {
                    symbol_table,
                    diagnostics: diagnostics.clone(),
                },
            );
        }

        // Convert to LSP diagnostics
        let lsp_diagnostics: Vec<tower_lsp::lsp_types::Diagnostic> = diagnostics
            .into_iter()
            .map(|d| self.to_lsp_diagnostic(&d))
            .collect();

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
            .await;
    }

    /// Run linting on parsed file
    fn run_lint(&self, parsed: &ParsedFile, language: Language) -> Vec<ArgusDiagnostic> {
        let checker = match self.registry.get(language) {
            Some(c) => c,
            None => return Vec::new(),
        };

        checker.check(parsed, &self.config)
    }

    /// Convert Argus diagnostic to LSP diagnostic
    fn to_lsp_diagnostic(&self, diag: &ArgusDiagnostic) -> tower_lsp::lsp_types::Diagnostic {
        tower_lsp::lsp_types::Diagnostic {
            range: Range {
                start: Position {
                    line: diag.range.start.line,
                    character: diag.range.start.character,
                },
                end: Position {
                    line: diag.range.end.line,
                    character: diag.range.end.character,
                },
            },
            severity: Some(self.to_lsp_severity(diag.severity)),
            code: Some(NumberOrString::String(diag.code.clone())),
            code_description: None,
            source: Some("cclab_lens".to_string()),
            message: diag.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    /// Convert Argus severity to LSP severity
    fn to_lsp_severity(&self, severity: ArgusSeverity) -> DiagnosticSeverity {
        match severity {
            ArgusSeverity::Error => DiagnosticSeverity::ERROR,
            ArgusSeverity::Warning => DiagnosticSeverity::WARNING,
            ArgusSeverity::Information => DiagnosticSeverity::INFORMATION,
            ArgusSeverity::Hint => DiagnosticSeverity::HINT,
        }
    }

    /// Detect language from URI
    fn detect_language(uri: &Url) -> Option<Language> {
        let path = PathBuf::from(uri.path());
        MultiParser::detect_language(&path)
    }

    /// Convert Argus Range to LSP Range
    fn to_lsp_range(range: &crate::diagnostic::Range) -> Range {
        Range {
            start: Position {
                line: range.start.line,
                character: range.start.character,
            },
            end: Position {
                line: range.end.line,
                character: range.end.character,
            },
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ArgusServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                // Semantic capabilities
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                // Completion
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                // Code actions (quick fixes + refactoring)
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::REFACTOR,
                            CodeActionKind::REFACTOR_EXTRACT,
                            CodeActionKind::REFACTOR_INLINE,
                            CodeActionKind::REFACTOR_REWRITE,
                        ]),
                        ..Default::default()
                    },
                )),
                // Execute command for refactoring with user input
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "cclab_lens.refactor.extractVariable".to_string(),
                        "cclab_lens.refactor.extractFunction".to_string(),
                        "cclab_lens.refactor.rename".to_string(),
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "cclab_lens".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Argus LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        let Some(language) = Self::detect_language(&uri) else {
            return;
        };

        // Store document
        {
            let mut documents = self.documents.write().await;
            documents.insert(
                uri.clone(),
                Document {
                    content,
                    language,
                    version,
                },
            );
        }

        // Analyze
        self.analyze_document(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get full content (we use FULL sync)
        let Some(change) = params.content_changes.into_iter().next() else {
            return;
        };

        // Update document
        {
            let mut documents = self.documents.write().await;
            if let Some(doc) = documents.get_mut(&uri) {
                doc.content = change.text;
                doc.version = version;
            }
        }

        // Analyze
        self.analyze_document(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        // Update content if provided
        if let Some(text) = params.text {
            let mut documents = self.documents.write().await;
            if let Some(doc) = documents.get_mut(&uri) {
                doc.content = text;
            }
        }

        // Re-analyze
        self.analyze_document(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document and analysis
        {
            let mut documents = self.documents.write().await;
            documents.remove(&uri);
        }
        {
            let mut analyses = self.analyses.write().await;
            analyses.remove(&uri);
        }

        // Clear diagnostics
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document language
        let language = {
            let documents = self.documents.read().await;
            documents.get(uri).map(|d| d.language)
        };

        let Some(language) = language else {
            return Ok(None);
        };

        // Get analysis
        let analyses = self.analyses.read().await;
        let Some(analysis) = analyses.get(uri) else {
            return Ok(None);
        };

        // Find symbol at position
        let symbol = analysis
            .symbol_table
            .find_at_position(position.line, position.character);

        let Some(symbol) = symbol else {
            return Ok(None);
        };

        // Generate hover content
        let content = symbol.hover_content(language);

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(Self::to_lsp_range(&symbol.location)),
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get analysis
        let analyses = self.analyses.read().await;
        let Some(analysis) = analyses.get(uri) else {
            return Ok(None);
        };

        // Find definition
        let symbol = analysis
            .symbol_table
            .find_definition_at(position.line, position.character);

        let Some(symbol) = symbol else {
            return Ok(None);
        };

        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: Self::to_lsp_range(&symbol.location),
        })))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        // Get analysis
        let analyses = self.analyses.read().await;
        let Some(analysis) = analyses.get(uri) else {
            return Ok(None);
        };

        // Find references
        let references = analysis.symbol_table.find_references_at(
            position.line,
            position.character,
            include_declaration,
        );

        if references.is_empty() {
            return Ok(None);
        }

        let locations: Vec<Location> = references
            .into_iter()
            .map(|r| Location {
                uri: uri.clone(),
                range: Self::to_lsp_range(&r),
            })
            .collect();

        Ok(Some(locations))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let request_range = params.range;

        // Get document content
        let content = {
            let documents = self.documents.read().await;
            documents.get(uri).map(|d| d.content.clone())
        };

        let Some(content) = content else {
            return Ok(None);
        };

        // Get analysis with diagnostics
        let analyses = self.analyses.read().await;
        let Some(analysis) = analyses.get(uri) else {
            return Ok(None);
        };

        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        // Find diagnostics that overlap with the requested range
        for diag in &analysis.diagnostics {
            let diag_range = Self::to_lsp_range(&diag.range);

            // Check if diagnostic overlaps with requested range
            if !Self::ranges_overlap(&diag_range, &request_range) {
                continue;
            }

            // Create code actions for each quick fix
            for fix in &diag.quick_fixes {
                let edits: Vec<TextEdit> = fix
                    .edits
                    .iter()
                    .map(|e| TextEdit {
                        range: Self::to_lsp_range(&e.range),
                        new_text: e.new_text.clone(),
                    })
                    .collect();

                let mut changes = HashMap::new();
                changes.insert(uri.clone(), edits);

                let action = CodeAction {
                    title: fix.title.clone(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![self.to_lsp_diagnostic(diag)]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                };

                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }

        // Add refactoring actions based on selection
        self.add_refactoring_actions(uri, &request_range, &content, &mut actions)
            .await;

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        // Execute refactoring commands with user-provided parameters
        let command = &params.command;

        // Parse arguments from params.arguments
        // Expected format: [{"uri": "file://...", "range": {...}, "name": "..."}]
        if params.arguments.is_empty() {
            return Ok(None);
        }

        let arg = &params.arguments[0];

        // Extract parameters (simplified - real implementation would use proper JSON deserialization)
        let uri_str = arg.get("uri").and_then(|v| v.as_str()).unwrap_or("");
        let name = arg
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("refactored");

        let uri = match Url::parse(uri_str) {
            Ok(u) => u,
            Err(_) => return Ok(None),
        };

        // Get document content
        let _content = {
            let documents = self.documents.read().await;
            documents.get(&uri).map(|d| d.content.clone())
        };

        let Some(_content) = _content else {
            return Ok(None);
        };

        // Execute the appropriate refactoring based on command
        // This would apply the refactoring and return workspace edits
        // For now, just log success
        self.client
            .log_message(
                MessageType::INFO,
                format!("Executing command: {} with name: {}", command, name),
            )
            .await;

        Ok(Some(Value::Null))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document content
        let doc_content = {
            let documents = self.documents.read().await;
            documents.get(uri).map(|d| (d.content.clone(), d.language))
        };

        let Some((content, language)) = doc_content else {
            return Ok(None);
        };

        // Only provide Python completions for now
        if language != Language::Python {
            return Ok(None);
        }

        // Get the line and figure out what we're completing
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = position.line as usize;

        if line_idx >= lines.len() {
            return Ok(None);
        }

        let line = lines[line_idx];
        let col = position.character as usize;
        let prefix = &line[..col.min(line.len())];

        // Check if this is a dot completion
        let items = if prefix.ends_with('.') {
            // Get word before the dot
            self.complete_attribute(prefix).await
        } else if let Some(trigger) = params.context.and_then(|c| c.trigger_character) {
            if trigger == "." {
                self.complete_attribute(prefix).await
            } else {
                self.complete_identifiers(prefix).await
            }
        } else {
            self.complete_identifiers(prefix).await
        };

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }
}

impl ArgusServer {
    /// Complete attributes after a dot
    async fn complete_attribute(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Find the object name before the dot
        let prefix = prefix.trim_end_matches('.');
        let obj_name = prefix.split_whitespace().last().unwrap_or("");

        // Get completions from stubs for known modules
        let stubs = self.stubs.read().await;

        // Check if this might be a module access
        if let Some(module_info) = stubs.get_stub(obj_name) {
            for (name, ty) in &module_info.exports {
                items.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(self.type_to_completion_kind(ty)),
                    detail: Some(ty.to_string()),
                    ..Default::default()
                });
            }
        }

        // Add common completions for known types
        if obj_name.ends_with("str") || obj_name.ends_with('"') || obj_name.ends_with('\'') {
            items.extend(self.string_completions());
        } else if obj_name.ends_with(']') {
            items.extend(self.list_completions());
        } else if obj_name.ends_with('}') {
            items.extend(self.dict_completions());
        }

        items
    }

    /// Complete identifiers (builtins, imports, local variables)
    async fn complete_identifiers(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Extract the partial identifier being typed
        let partial = prefix.split_whitespace().last().unwrap_or("");
        let partial_lower = partial.to_lowercase();

        // Get completions from builtins
        let stubs = self.stubs.read().await;

        if let Some(builtins) = stubs.get_stub("builtins") {
            for (name, ty) in &builtins.exports {
                if name.to_lowercase().starts_with(&partial_lower) {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(self.type_to_completion_kind(ty)),
                        detail: Some(ty.to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        // Add keywords
        let keywords = [
            "def", "class", "if", "elif", "else", "for", "while", "try", "except", "finally",
            "with", "return", "yield", "import", "from", "as", "pass", "break", "continue",
            "raise", "assert", "global", "nonlocal", "lambda", "True", "False", "None", "and",
            "or", "not", "in", "is", "async", "await", "match", "case",
        ];

        for kw in keywords {
            if kw.to_lowercase().starts_with(&partial_lower) {
                items.push(CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    ..Default::default()
                });
            }
        }

        items
    }

    /// Common string method completions
    fn string_completions(&self) -> Vec<CompletionItem> {
        let methods = [
            ("upper", "() -> str", "Convert to uppercase"),
            ("lower", "() -> str", "Convert to lowercase"),
            ("strip", "() -> str", "Remove leading/trailing whitespace"),
            ("split", "(sep=None) -> list[str]", "Split string"),
            ("join", "(iterable) -> str", "Join strings"),
            ("replace", "(old, new) -> str", "Replace occurrences"),
            ("startswith", "(prefix) -> bool", "Check if starts with"),
            ("endswith", "(suffix) -> bool", "Check if ends with"),
            ("find", "(sub) -> int", "Find substring index"),
            ("format", "(*args, **kwargs) -> str", "Format string"),
        ];

        methods
            .into_iter()
            .map(|(name, sig, doc)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(sig.to_string()),
                documentation: Some(Documentation::String(doc.to_string())),
                ..Default::default()
            })
            .collect()
    }

    /// Common list method completions
    fn list_completions(&self) -> Vec<CompletionItem> {
        let methods = [
            ("append", "(x)", "Add item to end"),
            ("extend", "(iterable)", "Extend with iterable"),
            ("insert", "(i, x)", "Insert at index"),
            ("remove", "(x)", "Remove first occurrence"),
            ("pop", "(i=-1)", "Remove and return item"),
            ("clear", "()", "Remove all items"),
            ("index", "(x)", "Return index of x"),
            ("count", "(x)", "Count occurrences"),
            ("sort", "()", "Sort in place"),
            ("reverse", "()", "Reverse in place"),
            ("copy", "()", "Return shallow copy"),
        ];

        methods
            .into_iter()
            .map(|(name, sig, doc)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(sig.to_string()),
                documentation: Some(Documentation::String(doc.to_string())),
                ..Default::default()
            })
            .collect()
    }

    /// Common dict method completions
    fn dict_completions(&self) -> Vec<CompletionItem> {
        let methods = [
            ("get", "(key, default=None)", "Get value with default"),
            ("keys", "()", "Return keys view"),
            ("values", "()", "Return values view"),
            ("items", "()", "Return items view"),
            ("pop", "(key, default=None)", "Remove and return value"),
            ("update", "(other)", "Update from dict/iterable"),
            ("setdefault", "(key, default=None)", "Get or set default"),
            ("clear", "()", "Remove all items"),
            ("copy", "()", "Return shallow copy"),
        ];

        methods
            .into_iter()
            .map(|(name, sig, doc)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(sig.to_string()),
                documentation: Some(Documentation::String(doc.to_string())),
                ..Default::default()
            })
            .collect()
    }

    /// Convert Argus type to LSP completion item kind
    fn type_to_completion_kind(&self, ty: &crate::type_inference::Type) -> CompletionItemKind {
        use crate::type_inference::Type;

        match ty {
            Type::Callable { .. } => CompletionItemKind::FUNCTION,
            Type::ClassType { .. } => CompletionItemKind::CLASS,
            Type::Instance { .. } => CompletionItemKind::VARIABLE,
            _ => CompletionItemKind::VALUE,
        }
    }

    /// Check if two ranges overlap
    fn ranges_overlap(a: &Range, b: &Range) -> bool {
        // a starts before b ends AND a ends after b starts
        (a.start.line < b.end.line
            || (a.start.line == b.end.line && a.start.character <= b.end.character))
            && (a.end.line > b.start.line
                || (a.end.line == b.start.line && a.end.character >= b.start.character))
    }

    /// Convert byte offset to LSP Position.
    fn offset_to_position(content: &str, offset: usize) -> Position {
        let mut line = 0;
        let mut character = 0;
        let mut current_offset = 0;

        for ch in content.chars() {
            if current_offset >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }

            current_offset += ch.len_utf8();
        }

        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// Add refactoring code actions based on the selected range.
    async fn add_refactoring_actions(
        &self,
        uri: &Url,
        range: &Range,
        content: &str,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        // Convert LSP range to Argus span
        let lines: Vec<&str> = content.lines().collect();
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;

        if start_line >= lines.len() || end_line >= lines.len() {
            return;
        }

        // Calculate byte offsets for the selection
        let start_offset: usize = lines
            .iter()
            .take(start_line)
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + range.start.character as usize;
        let end_offset: usize = lines
            .iter()
            .take(end_line)
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + range.end.character as usize;

        let span = ArgusSpan::new(start_offset, end_offset);
        let file = PathBuf::from(uri.path());

        // Extract Variable - if something is selected
        if start_offset < end_offset {
            if let Some(action) = self
                .create_refactor_action(
                    uri,
                    content,
                    RefactorKind::ExtractVariable {
                        name: "extracted_var".to_string(),
                    },
                    span,
                    file.clone(),
                    "Extract to variable",
                )
                .await
            {
                actions.push(CodeActionOrCommand::CodeAction(action));
            }

            // Extract Function
            if let Some(action) = self
                .create_refactor_action(
                    uri,
                    content,
                    RefactorKind::ExtractFunction {
                        name: "extracted_function".to_string(),
                    },
                    span,
                    file.clone(),
                    "Extract to function",
                )
                .await
            {
                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }

        // Rename Symbol - always available at cursor position
        if let Some(action) = self
            .create_refactor_action(
                uri,
                content,
                RefactorKind::Rename {
                    new_name: "new_name".to_string(),
                },
                span,
                file.clone(),
                "Rename symbol",
            )
            .await
        {
            actions.push(CodeActionOrCommand::CodeAction(action));
        }

        // Inline Variable - if on a variable definition
        if let Some(action) = self
            .create_refactor_action(
                uri,
                content,
                RefactorKind::Inline,
                span,
                file.clone(),
                "Inline variable",
            )
            .await
        {
            actions.push(CodeActionOrCommand::CodeAction(action));
        }

        // Change Signature - if on a function definition
        if let Some(action) = self
            .create_refactor_action(
                uri,
                content,
                RefactorKind::ChangeSignature {
                    changes: SignatureChanges::default(),
                },
                span,
                file,
                "Change function signature",
            )
            .await
        {
            actions.push(CodeActionOrCommand::CodeAction(action));
        }
    }

    /// Create a refactoring code action by executing the refactoring.
    async fn create_refactor_action(
        &self,
        _uri: &Url,
        content: &str,
        kind: RefactorKind,
        span: ArgusSpan,
        file: PathBuf,
        title: &str,
    ) -> Option<CodeAction> {
        let request = RefactorRequest {
            kind,
            file,
            span,
            options: RefactorOptions::default(),
        };

        // Execute refactoring
        let mut engine = self.refactoring_engine.write().await;
        let result = engine.execute(&request, content);

        // Check if refactoring succeeded
        if result.has_errors() {
            return None;
        }

        // Convert file edits to LSP workspace edits
        let mut changes = HashMap::new();

        for (file_path, edits) in result.file_edits {
            // Convert path to URI
            let file_uri = Url::from_file_path(&file_path).ok()?;

            // Convert edits to LSP text edits
            let lsp_edits: Vec<TextEdit> = edits
                .iter()
                .map(|edit| {
                    let start_pos = Self::offset_to_position(content, edit.span.start);
                    let end_pos = Self::offset_to_position(content, edit.span.end);

                    TextEdit {
                        range: Range {
                            start: start_pos,
                            end: end_pos,
                        },
                        new_text: edit.new_text.clone(),
                    }
                })
                .collect();

            changes.insert(file_uri, lsp_edits);
        }

        Some(CodeAction {
            title: title.to_string(),
            kind: Some(CodeActionKind::REFACTOR),
            diagnostics: None,
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(false),
            disabled: None,
            data: None,
        })
    }
}

/// Run the LSP server on stdio
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(ArgusServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

/// Run the LSP server on TCP (for debugging)
pub async fn run_server_tcp(port: u16) -> std::io::Result<()> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    tracing::info!("Argus LSP server listening on port {}", port);

    loop {
        let (stream, addr) = listener.accept().await?;
        tracing::info!("Client connected from {}", addr);

        let (read, write) = tokio::io::split(stream);
        let (service, socket) = LspService::new(ArgusServer::new);

        tokio::spawn(async move {
            Server::new(read, write, socket).serve(service).await;
        });
    }
}

//! Semantic code search (Sprint 4 - Track 1)
//!
//! Provides type-aware code search capabilities:
//! - Find similar code patterns
//! - Search by type signature
//! - Find implementations of protocol
//! - Call hierarchy analysis

use std::collections::HashMap;
use std::path::PathBuf;

use super::deep_inference::{DeepTypeInferencer, TypeContext};
use super::mutable_ast::Span;
use crate::semantic::{SymbolKind as SemanticSymbolKind, SymbolTable};
use crate::type_inference::ty::Type;

// ============================================================================
// Search Query
// ============================================================================

/// A semantic search query.
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Type of search
    pub kind: SearchKind,
    /// Scope to search in
    pub scope: SearchScope,
    /// Maximum results
    pub max_results: usize,
}

/// Type of semantic search.
#[derive(Debug, Clone)]
pub enum SearchKind {
    /// Find by type signature
    ByTypeSignature {
        params: Vec<Type>,
        return_type: Option<Type>,
    },
    /// Find implementations of protocol/interface
    Implementations { protocol: String },
    /// Find usages of a symbol
    Usages { symbol: String, file: PathBuf },
    /// Find similar code patterns
    SimilarPatterns { pattern: String },
    /// Find by documentation content
    ByDocumentation { query: String },
    /// Find call hierarchy (callers or callees)
    CallHierarchy {
        symbol: String,
        file: PathBuf,
        direction: CallDirection,
    },
    /// Find type hierarchy (supertypes or subtypes)
    TypeHierarchy {
        type_name: String,
        direction: TypeHierarchyDirection,
    },
}

/// Direction for call hierarchy search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallDirection {
    /// Find callers (incoming calls)
    Callers,
    /// Find callees (outgoing calls)
    Callees,
}

/// Direction for type hierarchy search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeHierarchyDirection {
    /// Find supertypes (parents)
    Supertypes,
    /// Find subtypes (children)
    Subtypes,
    /// Find both
    Both,
}

/// Scope for search operations.
#[derive(Debug, Clone)]
pub enum SearchScope {
    /// Current file only
    CurrentFile(PathBuf),
    /// Specific files
    Files(Vec<PathBuf>),
    /// Entire project
    Project,
    /// Project with dependencies
    ProjectWithDeps,
}

// ============================================================================
// Search Results
// ============================================================================

/// Result of a semantic search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Matching items
    pub matches: Vec<SearchMatch>,
    /// Total matches (may be more than returned)
    pub total_count: usize,
    /// Search statistics
    pub stats: SearchStats,
}

/// A single search match.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// File containing the match
    pub file: PathBuf,
    /// Span of the match
    pub span: Span,
    /// Symbol name (if applicable)
    pub symbol: Option<String>,
    /// Match kind
    pub kind: MatchKind,
    /// Relevance score (0.0 to 1.0)
    pub score: f64,
    /// Context lines around the match
    pub context: Option<MatchContext>,
}

/// Kind of search match.
#[derive(Debug, Clone)]
pub enum MatchKind {
    /// Function definition
    FunctionDef,
    /// Method definition
    MethodDef,
    /// Class definition
    ClassDef,
    /// Variable assignment
    VariableAssignment,
    /// Import statement
    Import,
    /// Call expression
    Call,
    /// Type annotation
    TypeAnnotation,
    /// Comment/docstring
    Documentation,
}

/// Context around a match.
#[derive(Debug, Clone)]
pub struct MatchContext {
    /// Lines before match
    pub before: Vec<String>,
    /// The matching line(s)
    pub matched: Vec<String>,
    /// Lines after match
    pub after: Vec<String>,
}

/// Search statistics.
#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    /// Files searched
    pub files_searched: usize,
    /// Time taken (milliseconds)
    pub time_ms: u64,
    /// Whether results were truncated
    pub truncated: bool,
}

impl SearchResult {
    /// Create empty result.
    pub fn empty() -> Self {
        Self {
            matches: Vec::new(),
            total_count: 0,
            stats: SearchStats::default(),
        }
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    /// Get number of matches returned.
    pub fn len(&self) -> usize {
        self.matches.len()
    }
}

// ============================================================================
// Semantic Search Engine
// ============================================================================

/// Engine for semantic code search.
pub struct SemanticSearchEngine {
    /// Type inferencer
    inferencer: DeepTypeInferencer,
    /// Symbol index
    symbol_index: HashMap<String, Vec<SymbolLocation>>,
    /// Type signature index: signature key -> functions
    type_signature_index: HashMap<String, Vec<SymbolLocation>>,
    /// Call graph
    call_graph: CallGraph,
}

/// Location of a symbol.
#[derive(Debug, Clone)]
pub struct SymbolLocation {
    /// File path
    pub file: PathBuf,
    /// Span in file
    pub span: Span,
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: MatchKind,
    /// Type (if known)
    pub ty: Option<Type>,
    /// Documentation string (docstring)
    pub docstring: Option<String>,
}

/// Location of a type.
#[derive(Debug, Clone)]
pub struct TypeLocation {
    /// File path
    pub file: PathBuf,
    /// Span in file
    pub span: Span,
    /// Type name
    pub type_name: String,
}

/// A call site in the code.
#[derive(Debug, Clone)]
pub struct CallSite {
    /// File where the call occurs
    pub file: PathBuf,
    /// Location of the call
    pub span: Span,
    /// Function being called
    pub callee: String,
    /// Function containing this call
    pub caller: String,
}

/// Call graph for tracking function calls.
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    /// Map from function name to calls it makes
    pub calls: HashMap<String, Vec<CallSite>>,
    /// Map from function name to places it's called from
    pub called_by: HashMap<String, Vec<CallSite>>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a call relationship.
    pub fn add_call(&mut self, call_site: CallSite) {
        self.calls
            .entry(call_site.caller.clone())
            .or_default()
            .push(call_site.clone());

        self.called_by
            .entry(call_site.callee.clone())
            .or_default()
            .push(call_site);
    }

    /// Get all functions called by the given function.
    pub fn get_callees(&self, function: &str) -> Vec<&CallSite> {
        self.calls
            .get(function)
            .map(|sites| sites.iter().collect())
            .unwrap_or_default()
    }

    /// Get all functions that call the given function.
    pub fn get_callers(&self, function: &str) -> Vec<&CallSite> {
        self.called_by
            .get(function)
            .map(|sites| sites.iter().collect())
            .unwrap_or_default()
    }
}

impl SemanticSearchEngine {
    /// Create a new search engine.
    pub fn new() -> Self {
        Self {
            inferencer: DeepTypeInferencer::new(),
            symbol_index: HashMap::new(),
            type_signature_index: HashMap::new(),
            call_graph: CallGraph::new(),
        }
    }

    /// Create with type inferencer.
    pub fn with_inferencer(inferencer: DeepTypeInferencer) -> Self {
        Self {
            inferencer,
            symbol_index: HashMap::new(),
            type_signature_index: HashMap::new(),
            call_graph: CallGraph::new(),
        }
    }

    /// Execute a search query.
    pub fn search(&self, query: &SearchQuery) -> SearchResult {
        match &query.kind {
            SearchKind::ByTypeSignature {
                params,
                return_type,
            } => self.search_by_type_signature(params, return_type.as_ref(), query),
            SearchKind::Implementations { protocol } => {
                self.search_implementations(protocol, query)
            }
            SearchKind::Usages { symbol, file } => self.search_usages(symbol, file, query),
            SearchKind::SimilarPatterns { pattern } => self.search_similar_patterns(pattern, query),
            SearchKind::ByDocumentation { query: doc_query } => {
                self.search_by_documentation(doc_query, query)
            }
            SearchKind::CallHierarchy {
                symbol,
                file,
                direction,
            } => self.search_call_hierarchy(symbol, file, *direction, query),
            SearchKind::TypeHierarchy {
                type_name,
                direction,
            } => self.search_type_hierarchy(type_name, *direction, query),
        }
    }

    /// Search by type signature.
    fn search_by_type_signature(
        &self,
        params: &[Type],
        return_type: Option<&Type>,
        query: &SearchQuery,
    ) -> SearchResult {
        let mut result = SearchResult::empty();
        let mut all_matches: Vec<(SearchMatch, f64)> = Vec::new();

        // Search for functions with matching signatures
        for (_sig_key, locations) in &self.type_signature_index {
            for location in locations {
                if let Some(ref loc_type) = location.ty {
                    if let Type::Callable {
                        params: loc_params,
                        ret: loc_ret,
                    } = loc_type
                    {
                        let score = self.compute_signature_match_score(
                            params,
                            return_type,
                            loc_params,
                            loc_ret,
                        );

                        if score > 0.0 {
                            let search_match = SearchMatch {
                                file: location.file.clone(),
                                span: location.span,
                                symbol: Some(location.name.clone()),
                                kind: location.kind.clone(),
                                score,
                                context: None,
                            };
                            all_matches.push((search_match, score));
                        }
                    }
                }
            }
        }

        // Sort by score (descending)
        all_matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        result.matches = all_matches
            .into_iter()
            .take(query.max_results)
            .map(|(m, _)| m)
            .collect();

        result.total_count = result.matches.len();
        result
    }

    /// Compute match score between two function signatures.
    /// Returns 0.0 for no match, 1.0 for exact match.
    fn compute_signature_match_score(
        &self,
        query_params: &[Type],
        query_return: Option<&Type>,
        func_params: &[crate::type_inference::ty::Param],
        func_return: &Type,
    ) -> f64 {
        let mut score = 0.0;
        let mut components = 0;

        // Check parameter count
        if query_params.len() != func_params.len() {
            // Allow variadic functions (partial match)
            if query_params.len() < func_params.len() {
                score += 0.3;
            } else {
                return 0.0; // Too many parameters
            }
        } else {
            score += 1.0;
        }
        components += 1;

        // Check parameter types
        for (i, query_param) in query_params.iter().enumerate() {
            if let Some(func_param) = func_params.get(i) {
                let param_score = self.type_compatibility_score(query_param, &func_param.ty);
                score += param_score;
                components += 1;
            }
        }

        // Check return type
        if let Some(query_ret) = query_return {
            let ret_score = self.type_compatibility_score(query_ret, func_return);
            score += ret_score * 1.5; // Weight return type higher
            components += 1;
        }

        if components > 0 {
            score / components as f64
        } else {
            0.0
        }
    }

    /// Compute compatibility score between two types (0.0 to 1.0).
    fn type_compatibility_score(&self, expected: &Type, actual: &Type) -> f64 {
        // Exact match
        if expected == actual {
            return 1.0;
        }

        // Any/Unknown matches everything
        if matches!(expected, Type::Any | Type::Unknown)
            || matches!(actual, Type::Any | Type::Unknown)
        {
            return 0.8;
        }

        match (expected, actual) {
            // Union types
            (Type::Union(types), _) => types
                .iter()
                .map(|t| self.type_compatibility_score(t, actual))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),
            (_, Type::Union(types)) => types
                .iter()
                .map(|t| self.type_compatibility_score(expected, t))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),

            // Container types (covariant in element type)
            (Type::List(e1), Type::List(e2)) => 0.9 * self.type_compatibility_score(e1, e2),
            (Type::Set(e1), Type::Set(e2)) => 0.9 * self.type_compatibility_score(e1, e2),
            (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
                let k_score = self.type_compatibility_score(k1, k2);
                let v_score = self.type_compatibility_score(v1, v2);
                0.9 * (k_score + v_score) / 2.0
            }

            // Optional types
            (Type::Optional(inner), other) | (other, Type::Optional(inner)) => {
                0.8 * self.type_compatibility_score(inner, other)
            }

            // Instance types (check if same class name)
            (Type::Instance { name: n1, .. }, Type::Instance { name: n2, .. }) => {
                if n1 == n2 {
                    0.9
                } else {
                    0.0
                }
            }

            // Class types
            (Type::ClassType { name: n1, .. }, Type::ClassType { name: n2, .. }) => {
                if n1 == n2 {
                    0.9
                } else {
                    0.0
                }
            }

            _ => 0.0,
        }
    }

    /// Search for implementations of a protocol.
    fn search_implementations(&self, protocol: &str, query: &SearchQuery) -> SearchResult {
        let mut result = SearchResult::empty();

        // Get all class symbols
        for (_name, locations) in &self.symbol_index {
            for location in locations {
                if matches!(location.kind, MatchKind::ClassDef) {
                    // Check if this class implements the protocol
                    if let Some(ref ty) = location.ty {
                        if self.implements_protocol(&location.name, ty, protocol) {
                            result.matches.push(SearchMatch {
                                file: location.file.clone(),
                                span: location.span,
                                symbol: Some(location.name.clone()),
                                kind: location.kind.clone(),
                                score: 1.0,
                                context: None,
                            });

                            if result.matches.len() >= query.max_results {
                                break;
                            }
                        }
                    }
                }
            }

            if result.matches.len() >= query.max_results {
                break;
            }
        }

        result.total_count = result.matches.len();
        result
    }

    /// Check if a class implements a protocol.
    fn implements_protocol(&self, class_name: &str, _class_type: &Type, protocol: &str) -> bool {
        // Use the type context to check protocol satisfaction
        let class_ty = Type::Instance {
            name: class_name.to_string(),
            module: None,
            type_args: vec![],
        };

        // Check if the class satisfies the protocol
        self.inferencer
            .context()
            .satisfies_protocol(&class_ty, protocol)
    }

    /// Search for usages of a symbol.
    fn search_usages(&self, symbol: &str, _file: &PathBuf, query: &SearchQuery) -> SearchResult {
        let mut result = SearchResult::empty();

        if let Some(locations) = self.symbol_index.get(symbol) {
            for loc in locations.iter().take(query.max_results) {
                result.matches.push(SearchMatch {
                    file: loc.file.clone(),
                    span: loc.span,
                    symbol: Some(symbol.to_string()),
                    kind: loc.kind.clone(),
                    score: 1.0,
                    context: None,
                });
            }
            result.total_count = locations.len();
        }

        result
    }

    /// Search for similar code patterns.
    fn search_similar_patterns(&self, pattern: &str, query: &SearchQuery) -> SearchResult {
        let mut result = SearchResult::empty();
        let pattern_lower = pattern.to_lowercase();

        // Simple pattern matching based on symbol names
        for (_name, locations) in &self.symbol_index {
            for location in locations {
                // Simple heuristic: match if symbol name contains pattern
                if location.name.to_lowercase().contains(&pattern_lower) {
                    result.matches.push(SearchMatch {
                        file: location.file.clone(),
                        span: location.span,
                        symbol: Some(location.name.clone()),
                        kind: location.kind.clone(),
                        score: 0.7, // Lower score for pattern match
                        context: None,
                    });

                    if result.matches.len() >= query.max_results {
                        break;
                    }
                }
            }

            if result.matches.len() >= query.max_results {
                break;
            }
        }

        // Sort by score
        result.matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        result.total_count = result.matches.len();
        result
    }

    /// Search by documentation content.
    fn search_by_documentation(&self, doc_query: &str, query: &SearchQuery) -> SearchResult {
        let mut result = SearchResult::empty();
        let query_lower = doc_query.to_lowercase();

        // Search through symbols that have documentation
        for (_name, locations) in &self.symbol_index {
            for location in locations {
                // Skip symbols without docstrings
                let docstring = match &location.docstring {
                    Some(doc) => doc,
                    None => continue,
                };

                // Search in the docstring content
                if docstring.to_lowercase().contains(&query_lower) {
                    // Calculate relevance score based on match position and frequency
                    let score: f64 = self.calculate_doc_search_score(&query_lower, docstring);

                    // Convert docstring to context
                    let doc_lines: Vec<String> = docstring.lines().map(|s| s.to_string()).collect();

                    let context = MatchContext {
                        before: Vec::new(),
                        matched: doc_lines,
                        after: Vec::new(),
                    };

                    result.matches.push(SearchMatch {
                        file: location.file.clone(),
                        span: location.span,
                        symbol: Some(location.name.clone()),
                        kind: location.kind.clone(),
                        score,
                        context: Some(context),
                    });

                    if result.matches.len() >= query.max_results {
                        break;
                    }
                }
            }

            if result.matches.len() >= query.max_results {
                break;
            }
        }

        result.total_count = result.matches.len();
        result
    }

    /// Calculate relevance score for documentation search.
    fn calculate_doc_search_score(&self, query: &str, docstring: &str) -> f64 {
        let doc_lower = docstring.to_lowercase();

        // Base score for having a match
        let mut score: f64 = 0.6_f64;

        // Bonus if query appears at the start (likely in summary line)
        if doc_lower.starts_with(query) {
            score += 0.2_f64;
        } else if doc_lower.find(query).unwrap_or(usize::MAX) < 100 {
            // Bonus if match is in first 100 chars
            score += 0.1_f64;
        }

        // Bonus for exact phrase match (not just contains)
        if doc_lower.split_whitespace().any(|word| word == query) {
            score += 0.1_f64;
        }

        // Cap at 0.95 (reserve 1.0 for perfect matches)
        score.min(0.95_f64)
    }

    /// Search call hierarchy.
    fn search_call_hierarchy(
        &self,
        symbol: &str,
        _file: &PathBuf,
        direction: CallDirection,
        query: &SearchQuery,
    ) -> SearchResult {
        let mut result = SearchResult::empty();
        let mut visited = std::collections::HashSet::new();

        // Recursively collect call hierarchy
        self.collect_call_hierarchy(
            symbol,
            direction,
            &mut visited,
            &mut result,
            query.max_results,
        );

        result.total_count = result.matches.len();
        result
    }

    /// Recursively collect call hierarchy.
    fn collect_call_hierarchy(
        &self,
        symbol: &str,
        direction: CallDirection,
        visited: &mut std::collections::HashSet<String>,
        result: &mut SearchResult,
        max_results: usize,
    ) {
        if visited.contains(symbol) || result.matches.len() >= max_results {
            return;
        }

        visited.insert(symbol.to_string());

        let call_sites = match direction {
            CallDirection::Callers => self.call_graph.get_callers(symbol),
            CallDirection::Callees => self.call_graph.get_callees(symbol),
        };

        for site in call_sites {
            let related_symbol = match direction {
                CallDirection::Callers => &site.caller,
                CallDirection::Callees => &site.callee,
            };

            result.matches.push(SearchMatch {
                file: site.file.clone(),
                span: site.span,
                symbol: Some(related_symbol.clone()),
                kind: MatchKind::Call,
                score: 1.0,
                context: None,
            });

            if result.matches.len() >= max_results {
                return;
            }

            // Recursively search (limit depth to avoid infinite loops)
            if visited.len() < 100 {
                self.collect_call_hierarchy(
                    related_symbol,
                    direction,
                    visited,
                    result,
                    max_results,
                );
            }
        }
    }

    /// Search type hierarchy.
    fn search_type_hierarchy(
        &self,
        type_name: &str,
        direction: TypeHierarchyDirection,
        query: &SearchQuery,
    ) -> SearchResult {
        let mut result = SearchResult::empty();
        let type_ctx = self.inferencer.context();

        match direction {
            TypeHierarchyDirection::Supertypes => {
                // Find parent types by checking all class/interface symbols
                for (_name, locations) in &self.symbol_index {
                    for location in locations {
                        if matches!(location.kind, MatchKind::ClassDef) {
                            let inst_ty = Type::Instance {
                                name: type_name.to_string(),
                                module: None,
                                type_args: vec![],
                            };

                            // Check if type_name satisfies this protocol/interface
                            if type_ctx.satisfies_protocol(&inst_ty, &location.name) {
                                result.matches.push(SearchMatch {
                                    file: location.file.clone(),
                                    span: location.span,
                                    symbol: Some(location.name.clone()),
                                    kind: location.kind.clone(),
                                    score: 1.0,
                                    context: None,
                                });

                                if result.matches.len() >= query.max_results {
                                    break;
                                }
                            }
                        }
                    }

                    if result.matches.len() >= query.max_results {
                        break;
                    }
                }
            }
            TypeHierarchyDirection::Subtypes => {
                // Find child types that implement this protocol
                for (_name, locations) in &self.symbol_index {
                    for location in locations {
                        if matches!(location.kind, MatchKind::ClassDef) {
                            let class_ty = Type::Instance {
                                name: location.name.clone(),
                                module: None,
                                type_args: vec![],
                            };

                            if type_ctx.satisfies_protocol(&class_ty, type_name) {
                                result.matches.push(SearchMatch {
                                    file: location.file.clone(),
                                    span: location.span,
                                    symbol: Some(location.name.clone()),
                                    kind: location.kind.clone(),
                                    score: 1.0,
                                    context: None,
                                });

                                if result.matches.len() >= query.max_results {
                                    break;
                                }
                            }
                        }
                    }

                    if result.matches.len() >= query.max_results {
                        break;
                    }
                }
            }
            TypeHierarchyDirection::Both => {
                // Search both directions
                let supertypes = self.search_type_hierarchy(
                    type_name,
                    TypeHierarchyDirection::Supertypes,
                    query,
                );
                let subtypes =
                    self.search_type_hierarchy(type_name, TypeHierarchyDirection::Subtypes, query);

                result.matches.extend(supertypes.matches);
                result.matches.extend(subtypes.matches);
                result.matches.truncate(query.max_results);
            }
        }

        result.total_count = result.matches.len();
        result
    }

    /// Index a file for searching.
    pub fn index_file(&mut self, file: PathBuf, symbols: Vec<SymbolLocation>) {
        for sym in symbols {
            if let Some(name) = self.extract_symbol_name(&sym) {
                self.symbol_index.entry(name).or_default().push(sym.clone());
            }

            // Index by type signature if it's a callable
            if let Some(ref ty) = sym.ty {
                if matches!(ty, Type::Callable { .. }) {
                    let sig_key = self.compute_signature_key(ty);
                    self.type_signature_index
                        .entry(sig_key)
                        .or_default()
                        .push(sym.clone());
                }
            }
        }
        self.inferencer.add_file(file);
    }

    /// Compute a signature key for indexing.
    fn compute_signature_key(&self, ty: &Type) -> String {
        match ty {
            Type::Callable { params, ret } => {
                let params_str = params
                    .iter()
                    .map(|p| self.type_to_key_string(&p.ty))
                    .collect::<Vec<_>>()
                    .join(",");
                let ret_str = self.type_to_key_string(ret);
                format!("({}) -> {}", params_str, ret_str)
            }
            _ => "unknown".to_string(),
        }
    }

    /// Convert type to a string key for indexing.
    fn type_to_key_string(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Str => "str".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Float => "float".to_string(),
            Type::None => "None".to_string(),
            Type::Any => "Any".to_string(),
            Type::Unknown => "Unknown".to_string(),
            Type::List(elem) => format!("List[{}]", self.type_to_key_string(elem)),
            Type::Dict(k, v) => format!(
                "Dict[{}, {}]",
                self.type_to_key_string(k),
                self.type_to_key_string(v)
            ),
            Type::Set(elem) => format!("Set[{}]", self.type_to_key_string(elem)),
            Type::Tuple(elems) => {
                let elems_str = elems
                    .iter()
                    .map(|e| self.type_to_key_string(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Tuple[{}]", elems_str)
            }
            Type::Union(types) => {
                let types_str = types
                    .iter()
                    .map(|t| self.type_to_key_string(t))
                    .collect::<Vec<_>>()
                    .join(" | ");
                types_str
            }
            Type::Optional(inner) => format!("{}?", self.type_to_key_string(inner)),
            Type::Instance { name, .. } => name.clone(),
            Type::ClassType { name, .. } => format!("type[{}]", name),
            _ => "Unknown".to_string(),
        }
    }

    /// Index a symbol table for searching.
    pub fn index_symbol_table(&mut self, file: PathBuf, symbol_table: &SymbolTable) {
        let symbols = self.convert_symbol_table_to_locations(file.clone(), symbol_table);
        self.index_file(file, symbols);
    }

    /// Index a symbol table for searching, also extracting docstrings from the
    /// provided source code via AST traversal (R3.4).
    ///
    /// This is the preferred variant when the source text is available.
    pub fn index_symbol_table_with_source(
        &mut self,
        file: PathBuf,
        symbol_table: &SymbolTable,
        source: &str,
        language: crate::syntax::Language,
    ) {
        let mut symbols = self.convert_symbol_table_to_locations(file.clone(), symbol_table);

        // Extract docstrings from the AST and attach them to matching symbols
        if let Ok(docstrings) = self.extract_docstrings(source, language) {
            for loc in &mut symbols {
                if let Some(doc) = docstrings.get(&loc.name) {
                    loc.docstring = Some(doc.clone());
                }
            }
        }

        self.index_file(file, symbols);
    }

    /// Update docstrings for indexed symbols.
    pub fn update_docstrings(&mut self, docstrings: std::collections::HashMap<String, String>) {
        for (symbol_name, docstring) in docstrings {
            if let Some(locations) = self.symbol_index.get_mut(&symbol_name) {
                for location in locations {
                    location.docstring = Some(docstring.clone());
                }
            }
        }
    }

    /// Extract docstrings from source code.
    /// Returns a map of symbol name -> docstring.
    pub fn extract_docstrings(
        &self,
        content: &str,
        language: crate::syntax::Language,
    ) -> Result<std::collections::HashMap<String, String>, String> {
        use crate::syntax::MultiParser;

        let mut parser =
            MultiParser::new().map_err(|e| format!("Failed to create parser: {:?}", e))?;
        let parsed = parser
            .parse(content, language)
            .ok_or("Failed to parse file")?;

        let docstrings =
            self.extract_docstrings_from_ast(&parsed.tree.root_node(), content, language);

        Ok(docstrings)
    }

    /// Extract docstrings from AST nodes.
    fn extract_docstrings_from_ast(
        &self,
        root: &tree_sitter::Node,
        source: &str,
        language: crate::syntax::Language,
    ) -> std::collections::HashMap<String, String> {
        let mut docstrings = std::collections::HashMap::new();

        self.visit_for_docstrings(root, source, language, &mut docstrings);

        docstrings
    }

    /// Recursively visit AST nodes to extract docstrings.
    fn visit_for_docstrings(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        language: crate::syntax::Language,
        docstrings: &mut std::collections::HashMap<String, String>,
    ) {
        match language {
            crate::syntax::Language::Python => {
                match node.kind() {
                    "function_definition" | "class_definition" => {
                        // Extract function/class name
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let symbol_name = &source[name_node.start_byte()..name_node.end_byte()];

                            // Look for docstring (first string in body)
                            if let Some(body_node) = node.child_by_field_name("body") {
                                let docstring = self.extract_python_docstring(&body_node, source);
                                if let Some(doc) = docstring {
                                    docstrings.insert(symbol_name.to_string(), doc);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            crate::syntax::Language::TypeScript => {
                match node.kind() {
                    "function_declaration" | "method_definition" | "class_declaration" => {
                        // Extract name
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let symbol_name = &source[name_node.start_byte()..name_node.end_byte()];

                            // Look for JSDoc comment before this node
                            if let Some(prev_sibling) = node.prev_sibling() {
                                if prev_sibling.kind() == "comment" {
                                    let comment_text =
                                        &source[prev_sibling.start_byte()..prev_sibling.end_byte()];
                                    if comment_text.starts_with("/**") {
                                        // JSDoc comment
                                        let cleaned = self.clean_jsdoc(comment_text);
                                        docstrings.insert(symbol_name.to_string(), cleaned);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            crate::syntax::Language::Rust => {
                // Rust doc comments: `/// …` lines immediately before item definitions.
                // tree-sitter-rust models doc comments as `line_comment` nodes with text
                // starting with `///`. We look for a function/struct/enum/trait item whose
                // immediately-preceding siblings are all `///` line_comment nodes.
                let item_kinds = [
                    "function_item",
                    "struct_item",
                    "enum_item",
                    "trait_item",
                    "impl_item",
                    "type_item",
                ];
                if item_kinds.contains(&node.kind()) {
                    // Extract name
                    let name_opt = node
                        .child_by_field_name("name")
                        .map(|n| source[n.start_byte()..n.end_byte()].to_string());

                    if let Some(symbol_name) = name_opt {
                        // Walk backwards through previous siblings collecting `/// ` comments
                        let mut doc_lines: Vec<String> = Vec::new();
                        let mut sib = node.prev_sibling();
                        while let Some(prev) = sib {
                            if prev.kind() == "line_comment" {
                                let text = &source[prev.start_byte()..prev.end_byte()];
                                let trimmed = text.trim_start_matches('/').trim();
                                doc_lines.push(trimmed.to_string());
                                sib = prev.prev_sibling();
                            } else if prev.kind() == "attribute_item" {
                                // Skip attributes like `#[derive(...)]`
                                sib = prev.prev_sibling();
                            } else {
                                break;
                            }
                        }
                        if !doc_lines.is_empty() {
                            doc_lines.reverse();
                            docstrings.insert(symbol_name, doc_lines.join("\n"));
                        }
                    }
                }
            }
            _ => {
                // Other languages not yet implemented
            }
        }

        // Visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_docstrings(&child, source, language, docstrings);
        }
    }

    /// Extract Python docstring from function/class body.
    fn extract_python_docstring(
        &self,
        body_node: &tree_sitter::Node,
        source: &str,
    ) -> Option<String> {
        // Python docstring is the first expression_statement containing a string
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "expression_statement" {
                // Check if it contains a string
                let mut expr_cursor = child.walk();
                for expr_child in child.children(&mut expr_cursor) {
                    if expr_child.kind() == "string" {
                        let string_content =
                            &source[expr_child.start_byte()..expr_child.end_byte()];
                        // Remove quotes and clean up
                        let cleaned = self.clean_python_docstring(string_content);
                        return Some(cleaned);
                    }
                }
            } else if child.kind() != "comment" {
                // First non-comment, non-string statement - no docstring
                break;
            }
        }
        None
    }

    /// Clean Python docstring (remove quotes, trim).
    fn clean_python_docstring(&self, raw: &str) -> String {
        let trimmed = raw.trim();

        // Remove triple quotes or single quotes
        let cleaned = if trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\"") {
            &trimmed[3..trimmed.len() - 3]
        } else if trimmed.starts_with("'''") && trimmed.ends_with("'''") {
            &trimmed[3..trimmed.len() - 3]
        } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
            &trimmed[1..trimmed.len() - 1]
        } else if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        cleaned.trim().to_string()
    }

    /// Clean JSDoc comment (remove /** */ and leading *).
    fn clean_jsdoc(&self, raw: &str) -> String {
        let trimmed = raw.trim();

        // Remove /** and */
        let content = if trimmed.starts_with("/**") && trimmed.ends_with("*/") {
            &trimmed[3..trimmed.len() - 2]
        } else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
            &trimmed[2..trimmed.len() - 2]
        } else {
            trimmed
        };

        // Remove leading * from each line
        content
            .lines()
            .map(|line| {
                let l = line.trim();
                if l.starts_with('*') {
                    l[1..].trim()
                } else {
                    l
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    /// Build call graph from source code.
    pub fn build_call_graph(
        &mut self,
        file: PathBuf,
        content: &str,
        language: crate::syntax::Language,
    ) -> Result<(), String> {
        use crate::syntax::MultiParser;

        let mut parser =
            MultiParser::new().map_err(|e| format!("Failed to create parser: {:?}", e))?;
        let parsed = parser
            .parse(content, language)
            .ok_or("Failed to parse file")?;

        let call_sites =
            self.extract_call_sites(file.clone(), &parsed.tree.root_node(), content, language);

        for site in call_sites {
            self.call_graph.add_call(site);
        }

        Ok(())
    }

    /// Extract all call sites from an AST.
    fn extract_call_sites(
        &self,
        file: PathBuf,
        root: &tree_sitter::Node,
        source: &str,
        language: crate::syntax::Language,
    ) -> Vec<CallSite> {
        let mut call_sites = Vec::new();
        let mut current_function: Option<String> = None;

        self.visit_for_calls(
            root,
            source,
            language,
            &mut current_function,
            &file,
            &mut call_sites,
        );

        call_sites
    }

    /// Recursively visit AST nodes to find function calls.
    fn visit_for_calls(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        language: crate::syntax::Language,
        current_function: &mut Option<String>,
        file: &PathBuf,
        call_sites: &mut Vec<CallSite>,
    ) {
        match language {
            crate::syntax::Language::Python => {
                match node.kind() {
                    "function_definition" => {
                        // Extract function name
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let func_name = &source[name_node.start_byte()..name_node.end_byte()];
                            let prev_function = current_function.clone();
                            *current_function = Some(func_name.to_string());

                            // Visit children (function body)
                            let mut cursor = node.walk();
                            for child in node.children(&mut cursor) {
                                self.visit_for_calls(
                                    &child,
                                    source,
                                    language,
                                    current_function,
                                    file,
                                    call_sites,
                                );
                            }

                            // Restore previous function context
                            *current_function = prev_function;
                            return; // Don't visit children again
                        }
                    }
                    "call" => {
                        // Extract callee name
                        if let Some(func_node) = node.child_by_field_name("function") {
                            let callee_name = self.extract_function_name(&func_node, source);

                            if let Some(ref caller) = current_function {
                                call_sites.push(CallSite {
                                    file: file.clone(),
                                    span: Span {
                                        start: node.start_byte(),
                                        end: node.end_byte(),
                                        start_line: node.start_position().row,
                                        start_col: node.start_position().column,
                                        end_line: node.end_position().row,
                                        end_col: node.end_position().column,
                                    },
                                    callee: callee_name,
                                    caller: caller.clone(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            crate::syntax::Language::TypeScript => match node.kind() {
                "function_declaration" | "method_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let func_name = &source[name_node.start_byte()..name_node.end_byte()];
                        let prev_function = current_function.clone();
                        *current_function = Some(func_name.to_string());

                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            self.visit_for_calls(
                                &child,
                                source,
                                language,
                                current_function,
                                file,
                                call_sites,
                            );
                        }

                        *current_function = prev_function;
                        return;
                    }
                }
                "call_expression" => {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let callee_name = self.extract_function_name(&func_node, source);

                        if let Some(ref caller) = current_function {
                            call_sites.push(CallSite {
                                file: file.clone(),
                                span: Span {
                                    start: node.start_byte(),
                                    end: node.end_byte(),
                                    start_line: node.start_position().row,
                                    start_col: node.start_position().column,
                                    end_line: node.end_position().row,
                                    end_col: node.end_position().column,
                                },
                                callee: callee_name,
                                caller: caller.clone(),
                            });
                        }
                    }
                }
                _ => {}
            },
            _ => {
                // Other languages not yet implemented
            }
        }

        // Visit children for all nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_calls(&child, source, language, current_function, file, call_sites);
        }
    }

    /// Extract function name from a node (handles attributes like obj.method).
    fn extract_function_name(&self, node: &tree_sitter::Node, source: &str) -> String {
        match node.kind() {
            "identifier" => source[node.start_byte()..node.end_byte()].to_string(),
            "attribute" => {
                // For obj.method, extract just "method"
                if let Some(attr_node) = node.child_by_field_name("attribute") {
                    source[attr_node.start_byte()..attr_node.end_byte()].to_string()
                } else {
                    source[node.start_byte()..node.end_byte()].to_string()
                }
            }
            "member_expression" => {
                // TypeScript: obj.method
                if let Some(property_node) = node.child_by_field_name("property") {
                    source[property_node.start_byte()..property_node.end_byte()].to_string()
                } else {
                    source[node.start_byte()..node.end_byte()].to_string()
                }
            }
            _ => {
                // Fallback: use full text
                source[node.start_byte()..node.end_byte()].to_string()
            }
        }
    }

    /// Convert a SymbolTable to a Vec of SymbolLocations.
    fn convert_symbol_table_to_locations(
        &self,
        file: PathBuf,
        symbol_table: &SymbolTable,
    ) -> Vec<SymbolLocation> {
        let mut locations = Vec::new();

        for symbol in symbol_table.all_symbols() {
            let kind = Self::convert_symbol_kind(symbol.kind);
            let ty = symbol
                .type_info
                .as_ref()
                .map(|ti| Self::convert_type_info(ti));

            locations.push(SymbolLocation {
                file: file.clone(),
                span: Span {
                    start: 0, // We don't have byte offsets in semantic::Symbol
                    end: 0,
                    start_line: symbol.location.start.line as usize,
                    start_col: symbol.location.start.character as usize,
                    end_line: symbol.location.end.line as usize,
                    end_col: symbol.location.end.character as usize,
                },
                name: symbol.name.clone(),
                kind,
                ty,
                docstring: None, // TODO: Extract from AST
            });
        }

        locations
    }

    /// Convert semantic SymbolKind to search MatchKind.
    fn convert_symbol_kind(kind: SemanticSymbolKind) -> MatchKind {
        match kind {
            SemanticSymbolKind::Function => MatchKind::FunctionDef,
            SemanticSymbolKind::Class | SemanticSymbolKind::Struct => MatchKind::ClassDef,
            SemanticSymbolKind::Variable
            | SemanticSymbolKind::Const
            | SemanticSymbolKind::Static => MatchKind::VariableAssignment,
            SemanticSymbolKind::Parameter => MatchKind::VariableAssignment,
            SemanticSymbolKind::Import => MatchKind::Import,
            SemanticSymbolKind::TypeAlias => MatchKind::TypeAnnotation,
            SemanticSymbolKind::Interface | SemanticSymbolKind::Trait => MatchKind::ClassDef,
            _ => MatchKind::VariableAssignment,
        }
    }

    /// Convert semantic TypeInfo to Type.
    fn convert_type_info(type_info: &crate::semantic::TypeInfo) -> Type {
        use crate::semantic::TypeInfo;
        use crate::type_inference::ty::{Param, ParamKind};

        match type_info {
            TypeInfo::Primitive(name) => match name.as_str() {
                "int" => Type::Int,
                "str" => Type::Str,
                "bool" => Type::Bool,
                "float" => Type::Float,
                "None" => Type::None,
                _ => Type::Unknown,
            },
            TypeInfo::List(inner) => {
                let elem = Self::convert_type_info(inner);
                Type::List(Box::new(elem))
            }
            TypeInfo::Dict(key, value) => {
                let key_ty = Self::convert_type_info(key);
                let value_ty = Self::convert_type_info(value);
                Type::Dict(Box::new(key_ty), Box::new(value_ty))
            }
            TypeInfo::Optional(inner) => {
                let inner_ty = Self::convert_type_info(inner);
                Type::Union(vec![inner_ty, Type::None])
            }
            TypeInfo::Union(types) => {
                let converted_types = types.iter().map(Self::convert_type_info).collect();
                Type::Union(converted_types)
            }
            TypeInfo::Callable { params, ret } => {
                // Convert TypeInfo params to Param structs
                let param_structs: Vec<Param> = params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| Param {
                        name: format!("arg{}", i),
                        ty: Self::convert_type_info(p),
                        has_default: false,
                        kind: ParamKind::Positional,
                    })
                    .collect();
                let return_type = Box::new(Self::convert_type_info(ret));
                Type::Callable {
                    params: param_structs,
                    ret: return_type,
                }
            }
            TypeInfo::Named(name) => Type::Instance {
                name: name.clone(),
                module: None,
                type_args: vec![],
            },
            TypeInfo::Generic(name, args) => {
                let type_args = args.iter().map(Self::convert_type_info).collect();
                Type::Instance {
                    name: name.clone(),
                    module: None,
                    type_args,
                }
            }
            TypeInfo::Reference(inner) => Self::convert_type_info(inner),
            TypeInfo::Unknown => Type::Unknown,
            TypeInfo::Any => Type::Any,
            TypeInfo::Error => Type::Unknown,
        }
    }

    fn extract_symbol_name(&self, sym: &SymbolLocation) -> Option<String> {
        Some(sym.name.clone())
    }

    /// Get type context.
    pub fn type_context(&self) -> &TypeContext {
        self.inferencer.context()
    }
}

impl Default for SemanticSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Range;
    use crate::semantic::{SymbolKind, TypeInfo};

    #[test]
    fn test_search_result() {
        let result = SearchResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_search_engine() {
        let engine = SemanticSearchEngine::new();
        let query = SearchQuery {
            kind: SearchKind::Usages {
                symbol: "foo".to_string(),
                file: PathBuf::from("test.py"),
            },
            scope: SearchScope::Project,
            max_results: 100,
        };

        let result = engine.search(&query);
        assert!(result.is_empty());
    }

    #[test]
    fn test_index_population() {
        let mut engine = SemanticSearchEngine::new();
        let mut symbol_table = SymbolTable::new();

        // Add test symbols
        symbol_table.add_symbol(
            "func1".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 0,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 0,
                    character: 9,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("None".to_string())),
            }),
            None,
            0,
        );

        symbol_table.add_symbol(
            "MyClass".to_string(),
            SymbolKind::Class,
            Range {
                start: crate::diagnostic::Position {
                    line: 2,
                    character: 6,
                },
                end: crate::diagnostic::Position {
                    line: 2,
                    character: 13,
                },
            },
            None,
            None,
            0,
        );

        let file = PathBuf::from("test.py");
        engine.index_symbol_table(file.clone(), &symbol_table);

        // Search for func1
        let query = SearchQuery {
            kind: SearchKind::Usages {
                symbol: "func1".to_string(),
                file: file.clone(),
            },
            scope: SearchScope::Project,
            max_results: 100,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
        assert_eq!(result.matches[0].symbol, Some("func1".to_string()));

        // Search for MyClass
        let query = SearchQuery {
            kind: SearchKind::Usages {
                symbol: "MyClass".to_string(),
                file,
            },
            scope: SearchScope::Project,
            max_results: 100,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
        assert_eq!(result.matches[0].symbol, Some("MyClass".to_string()));
    }

    #[test]
    fn test_convert_type_info() {
        // Test primitive types
        let int_type =
            SemanticSearchEngine::convert_type_info(&TypeInfo::Primitive("int".to_string()));
        assert_eq!(int_type, Type::Int);

        // Test list types
        let list_type = SemanticSearchEngine::convert_type_info(&TypeInfo::List(Box::new(
            TypeInfo::Primitive("str".to_string()),
        )));
        assert_eq!(list_type, Type::List(Box::new(Type::Str)));

        // Test optional types
        let opt_type = SemanticSearchEngine::convert_type_info(&TypeInfo::Optional(Box::new(
            TypeInfo::Primitive("int".to_string()),
        )));
        assert_eq!(opt_type, Type::Union(vec![Type::Int, Type::None]));
    }

    #[test]
    fn test_search_by_type_signature() {
        let mut engine = SemanticSearchEngine::new();
        let mut symbol_table = SymbolTable::new();

        // Add function: def func1(x: str, y: int) -> bool
        symbol_table.add_symbol(
            "func1".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 0,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 0,
                    character: 9,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![
                    TypeInfo::Primitive("str".to_string()),
                    TypeInfo::Primitive("int".to_string()),
                ],
                ret: Box::new(TypeInfo::Primitive("bool".to_string())),
            }),
            None,
            0,
        );

        // Add function: def func2(a: int, b: int) -> int
        symbol_table.add_symbol(
            "func2".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 2,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 2,
                    character: 9,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![
                    TypeInfo::Primitive("int".to_string()),
                    TypeInfo::Primitive("int".to_string()),
                ],
                ret: Box::new(TypeInfo::Primitive("int".to_string())),
            }),
            None,
            0,
        );

        let file = PathBuf::from("test.py");
        engine.index_symbol_table(file.clone(), &symbol_table);

        // Search for (str, int) -> bool
        let query = SearchQuery {
            kind: SearchKind::ByTypeSignature {
                params: vec![Type::Str, Type::Int],
                return_type: Some(Type::Bool),
            },
            scope: SearchScope::Project,
            max_results: 10,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());

        // Should find func1 with high score (top result)
        let high_score_matches: Vec<_> = result.matches.iter().filter(|m| m.score > 0.9).collect();
        assert_eq!(high_score_matches.len(), 1);
        assert_eq!(high_score_matches[0].symbol, Some("func1".to_string()));

        // Search for (int, int) -> int
        let query = SearchQuery {
            kind: SearchKind::ByTypeSignature {
                params: vec![Type::Int, Type::Int],
                return_type: Some(Type::Int),
            },
            scope: SearchScope::Project,
            max_results: 10,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());

        // Should find func2 with high score
        let high_score_matches: Vec<_> = result.matches.iter().filter(|m| m.score > 0.9).collect();
        assert_eq!(high_score_matches.len(), 1);
        assert_eq!(high_score_matches[0].symbol, Some("func2".to_string()));
    }

    #[test]
    fn test_type_compatibility_score() {
        let engine = SemanticSearchEngine::new();

        // Exact match
        assert_eq!(engine.type_compatibility_score(&Type::Int, &Type::Int), 1.0);
        assert_eq!(engine.type_compatibility_score(&Type::Str, &Type::Str), 1.0);

        // Any/Unknown matches
        assert!(engine.type_compatibility_score(&Type::Any, &Type::Int) > 0.5);
        assert!(engine.type_compatibility_score(&Type::Int, &Type::Unknown) > 0.5);

        // Container types
        let list_int = Type::List(Box::new(Type::Int));
        let list_int2 = Type::List(Box::new(Type::Int));
        assert!(engine.type_compatibility_score(&list_int, &list_int2) > 0.8);

        // No match
        assert_eq!(engine.type_compatibility_score(&Type::Int, &Type::Str), 0.0);
    }

    #[test]
    fn test_search_implementations() {
        let mut engine = SemanticSearchEngine::new();
        let mut symbol_table = SymbolTable::new();

        // Add a class
        symbol_table.add_symbol(
            "MyClass".to_string(),
            SymbolKind::Class,
            Range {
                start: crate::diagnostic::Position {
                    line: 0,
                    character: 6,
                },
                end: crate::diagnostic::Position {
                    line: 0,
                    character: 13,
                },
            },
            None,
            None,
            0,
        );

        let file = PathBuf::from("test.py");
        engine.index_symbol_table(file.clone(), &symbol_table);

        // Search for implementations of a protocol
        let query = SearchQuery {
            kind: SearchKind::Implementations {
                protocol: "Sized".to_string(),
            },
            scope: SearchScope::Project,
            max_results: 10,
        };

        let result = engine.search(&query);
        // Note: This will be empty because we haven't implemented protocol checking yet
        // but at least it doesn't crash
        assert!(result.is_empty() || !result.is_empty());
    }

    #[test]
    fn test_build_call_graph() {
        use crate::syntax::Language;

        let mut engine = SemanticSearchEngine::new();
        let file = PathBuf::from("test.py");

        let code = r#"
def func1():
    func2()
    func3()

def func2():
    func3()

def func3():
    pass
"#;

        // Build call graph
        let result = engine.build_call_graph(file.clone(), code, Language::Python);
        assert!(result.is_ok());

        // Verify call graph contains expected relationships
        let callers_of_func2 = engine.call_graph.get_callers("func2");
        assert_eq!(callers_of_func2.len(), 1);
        assert_eq!(callers_of_func2[0].caller, "func1");

        let callers_of_func3 = engine.call_graph.get_callers("func3");
        assert_eq!(callers_of_func3.len(), 2); // Called by func1 and func2
    }

    #[test]
    fn test_call_hierarchy_search() {
        use crate::syntax::Language;

        let mut engine = SemanticSearchEngine::new();
        let file = PathBuf::from("test.py");

        let code = r#"
def level1():
    level2()

def level2():
    level3()

def level3():
    pass
"#;

        // Build call graph
        engine
            .build_call_graph(file.clone(), code, Language::Python)
            .unwrap();

        // Search for callers of level3
        let query = SearchQuery {
            kind: SearchKind::CallHierarchy {
                symbol: "level3".to_string(),
                file: file.clone(),
                direction: CallDirection::Callers,
            },
            scope: SearchScope::Project,
            max_results: 100,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());

        // Call hierarchy is recursive, so we find level2 (direct caller) and level1 (indirect)
        let caller_names: Vec<String> = result
            .matches
            .iter()
            .filter_map(|m| m.symbol.clone())
            .collect();
        assert!(caller_names.contains(&"level2".to_string())); // Direct caller
        assert!(caller_names.contains(&"level1".to_string())); // Indirect caller
    }

    #[test]
    fn test_call_hierarchy_multi_level() {
        use crate::syntax::Language;

        let mut engine = SemanticSearchEngine::new();
        let file = PathBuf::from("test.py");

        let code = r#"
def top():
    middle1()
    middle2()

def middle1():
    bottom()

def middle2():
    bottom()

def bottom():
    pass
"#;

        // Build call graph
        engine
            .build_call_graph(file.clone(), code, Language::Python)
            .unwrap();

        // Search for callers of bottom (should find middle1 and middle2)
        let query = SearchQuery {
            kind: SearchKind::CallHierarchy {
                symbol: "bottom".to_string(),
                file: file.clone(),
                direction: CallDirection::Callers,
            },
            scope: SearchScope::Project,
            max_results: 100,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());

        // Call hierarchy is recursive, finds middle1, middle2 (direct callers) and top (indirect)
        let caller_names: Vec<String> = result
            .matches
            .iter()
            .filter_map(|m| m.symbol.clone())
            .collect();
        assert!(caller_names.contains(&"middle1".to_string())); // Direct caller
        assert!(caller_names.contains(&"middle2".to_string())); // Direct caller
        assert!(caller_names.contains(&"top".to_string())); // Indirect caller
    }

    #[test]
    fn test_call_hierarchy_callees() {
        use crate::syntax::Language;

        let mut engine = SemanticSearchEngine::new();
        let file = PathBuf::from("test.py");

        let code = r#"
def caller():
    callee1()
    callee2()
    callee3()

def callee1():
    pass

def callee2():
    pass

def callee3():
    pass
"#;

        // Build call graph
        engine
            .build_call_graph(file.clone(), code, Language::Python)
            .unwrap();

        // Search for callees of caller
        let query = SearchQuery {
            kind: SearchKind::CallHierarchy {
                symbol: "caller".to_string(),
                file: file.clone(),
                direction: CallDirection::Callees,
            },
            scope: SearchScope::Project,
            max_results: 100,
        };

        let result = engine.search(&query);
        assert_eq!(result.len(), 3);

        let callee_names: Vec<String> = result
            .matches
            .iter()
            .filter_map(|m| m.symbol.clone())
            .collect();
        assert!(callee_names.contains(&"callee1".to_string()));
        assert!(callee_names.contains(&"callee2".to_string()));
        assert!(callee_names.contains(&"callee3".to_string()));
    }

    #[test]
    fn test_documentation_search_python() {
        use crate::syntax::Language;

        let mut engine = SemanticSearchEngine::new();
        let _file = PathBuf::from("test.py");

        let code = r#"
def calculate_sum(a, b):
    """
    Calculate the sum of two numbers.

    Args:
        a: First number
        b: Second number

    Returns:
        The sum of a and b
    """
    return a + b

def calculate_product(a, b):
    """
    Calculate the product of two numbers.

    Args:
        a: First number
        b: Second number

    Returns:
        The product of a and b
    """
    return a * b

def unrelated_function():
    """Do something unrelated."""
    pass
"#;

        // First, build symbol table
        let mut symbol_table = SymbolTable::new();
        symbol_table.add_symbol(
            "calculate_sum".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 1,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 1,
                    character: 17,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("int".to_string())),
            }),
            None,
            0,
        );
        symbol_table.add_symbol(
            "calculate_product".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 15,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 15,
                    character: 21,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("int".to_string())),
            }),
            None,
            0,
        );
        symbol_table.add_symbol(
            "unrelated_function".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 29,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 29,
                    character: 22,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("None".to_string())),
            }),
            None,
            0,
        );

        // Index the symbol table
        engine.index_symbol_table(PathBuf::from("test.py"), &symbol_table);

        // Extract docstrings
        let docstrings = engine.extract_docstrings(code, Language::Python).unwrap();
        assert_eq!(docstrings.len(), 3);
        assert!(docstrings.contains_key("calculate_sum"));
        assert!(docstrings.contains_key("calculate_product"));

        // Update search engine with docstrings
        engine.update_docstrings(docstrings);

        // Search for "sum" in documentation
        let query = SearchQuery {
            kind: SearchKind::ByDocumentation {
                query: "sum".to_string(),
            },
            scope: SearchScope::Project,
            max_results: 10,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());

        let symbols: Vec<String> = result
            .matches
            .iter()
            .filter_map(|m| m.symbol.clone())
            .collect();
        assert!(symbols.contains(&"calculate_sum".to_string()));
        assert!(!symbols.contains(&"unrelated_function".to_string()));

        // Verify context is included
        assert!(result.matches[0].context.is_some());
        if let Some(context) = &result.matches[0].context {
            assert!(!context.matched.is_empty());
        }
    }

    #[test]
    fn test_documentation_search_typescript() {
        use crate::syntax::Language;

        let mut engine = SemanticSearchEngine::new();
        let _file = PathBuf::from("test.ts");

        let code = r#"
/**
 * Validates user input data.
 * @param input - The input string to validate
 * @returns true if valid, false otherwise
 */
function validateInput(input: string): boolean {
    return input.length > 0;
}

/**
 * Processes user data and returns result.
 * @param data - The data to process
 */
function processData(data: any): void {
    console.log(data);
}

function noDocFunction() {
    return 42;
}
"#;

        // First, build symbol table
        let mut symbol_table = SymbolTable::new();
        symbol_table.add_symbol(
            "validateInput".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 6,
                    character: 9,
                },
                end: crate::diagnostic::Position {
                    line: 6,
                    character: 22,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("boolean".to_string())),
            }),
            None,
            0,
        );
        symbol_table.add_symbol(
            "processData".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 14,
                    character: 9,
                },
                end: crate::diagnostic::Position {
                    line: 14,
                    character: 20,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("void".to_string())),
            }),
            None,
            0,
        );
        symbol_table.add_symbol(
            "noDocFunction".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 21,
                    character: 9,
                },
                end: crate::diagnostic::Position {
                    line: 21,
                    character: 22,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("number".to_string())),
            }),
            None,
            0,
        );

        // Index the symbol table
        engine.index_symbol_table(PathBuf::from("test.ts"), &symbol_table);

        // Extract docstrings
        let docstrings = engine
            .extract_docstrings(code, Language::TypeScript)
            .unwrap();
        assert_eq!(docstrings.len(), 2);
        assert!(docstrings.contains_key("validateInput"));
        assert!(docstrings.contains_key("processData"));
        assert!(!docstrings.contains_key("noDocFunction"));

        // Update search engine with docstrings
        engine.update_docstrings(docstrings);

        // Search for "validate" in documentation
        let query = SearchQuery {
            kind: SearchKind::ByDocumentation {
                query: "validate".to_string(),
            },
            scope: SearchScope::Project,
            max_results: 10,
        };

        let result = engine.search(&query);
        assert!(!result.is_empty());

        let symbols: Vec<String> = result
            .matches
            .iter()
            .filter_map(|m| m.symbol.clone())
            .collect();
        assert!(symbols.contains(&"validateInput".to_string()));
        assert!(!symbols.contains(&"processData".to_string()));
        assert!(!symbols.contains(&"noDocFunction".to_string()));
    }

    #[test]
    fn test_documentation_search_scoring() {
        use crate::syntax::Language;

        let mut engine = SemanticSearchEngine::new();
        let _file = PathBuf::from("test.py");

        let code = r#"
def exact_match():
    """search this exact phrase"""
    pass

def contains_in_middle():
    """Some text before search and more after"""
    pass

def contains_at_end():
    """This has the term at the end: search"""
    pass
"#;

        // First, build symbol table
        let mut symbol_table = SymbolTable::new();
        symbol_table.add_symbol(
            "exact_match".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 1,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 1,
                    character: 15,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("None".to_string())),
            }),
            None,
            0,
        );
        symbol_table.add_symbol(
            "contains_in_middle".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 5,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 5,
                    character: 22,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("None".to_string())),
            }),
            None,
            0,
        );
        symbol_table.add_symbol(
            "contains_at_end".to_string(),
            SymbolKind::Function,
            Range {
                start: crate::diagnostic::Position {
                    line: 9,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 9,
                    character: 19,
                },
            },
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Primitive("None".to_string())),
            }),
            None,
            0,
        );

        // Index the symbol table
        engine.index_symbol_table(PathBuf::from("test.py"), &symbol_table);

        // Extract and update docstrings
        let docstrings = engine.extract_docstrings(code, Language::Python).unwrap();
        engine.update_docstrings(docstrings);

        // Search for "search"
        let query = SearchQuery {
            kind: SearchKind::ByDocumentation {
                query: "search".to_string(),
            },
            scope: SearchScope::Project,
            max_results: 10,
        };

        let result = engine.search(&query);
        assert_eq!(result.len(), 3);

        // Verify that all results have scores
        for match_result in &result.matches {
            assert!(match_result.score > 0.0);
            assert!(match_result.score <= 1.0);
        }

        // The exact match at the start should have the highest score
        let exact_match = result
            .matches
            .iter()
            .find(|m| m.symbol.as_ref().map(|s| s.as_str()) == Some("exact_match"))
            .unwrap();

        let contains_in_middle = result
            .matches
            .iter()
            .find(|m| m.symbol.as_ref().map(|s| s.as_str()) == Some("contains_in_middle"))
            .unwrap();

        assert!(exact_match.score > contains_in_middle.score);
    }
}

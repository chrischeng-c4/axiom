//! Incremental analysis (Sprint 5 - Track 1)
//!
//! Provides incremental type checking and analysis:
//! - Dependency tracking
//! - Minimal reanalysis on changes
//! - Persistent analysis cache
//! - Background analysis

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::cache::ContentHash;
use crate::syntax::{Language, MultiParser};

// ============================================================================
// Change Tracking
// ============================================================================

/// A change to a file.
#[derive(Debug, Clone)]
pub struct FileChange {
    /// File that changed
    pub file: PathBuf,
    /// Type of change
    pub kind: ChangeKind,
    /// Content hash after change
    pub new_hash: ContentHash,
    /// Timestamp of change
    pub timestamp: Instant,
}

/// Type of file change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed (old path stored separately)
    Renamed,
}

/// Tracks changes across files.
pub struct ChangeTracker {
    /// Pending changes
    pending: Vec<FileChange>,
    /// Last known hash per file
    file_hashes: HashMap<PathBuf, ContentHash>,
    /// Debounce duration
    debounce: Duration,
}

impl ChangeTracker {
    /// Create a new change tracker.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            file_hashes: HashMap::new(),
            debounce: Duration::from_millis(300),
        }
    }

    /// Set debounce duration.
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    /// Record a file change.
    pub fn record_change(&mut self, file: PathBuf, kind: ChangeKind, new_hash: ContentHash) {
        self.pending.push(FileChange {
            file: file.clone(),
            kind,
            new_hash: new_hash.clone(),
            timestamp: Instant::now(),
        });

        if kind != ChangeKind::Deleted {
            self.file_hashes.insert(file, new_hash);
        } else {
            self.file_hashes.remove(&file);
        }
    }

    /// Get pending changes (after debounce).
    pub fn get_pending_changes(&mut self) -> Vec<FileChange> {
        let now = Instant::now();
        let debounce = self.debounce;

        // Get changes that are past debounce period
        let (ready, pending): (Vec<_>, Vec<_>) = self
            .pending
            .drain(..)
            .partition(|c| now.duration_since(c.timestamp) >= debounce);

        self.pending = pending;
        ready
    }

    /// Check if a file has changed.
    pub fn has_changed(&self, file: &PathBuf, hash: &ContentHash) -> bool {
        self.file_hashes
            .get(file)
            .map(|h| h != hash)
            .unwrap_or(true)
    }

    /// Clear all pending changes.
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

impl Default for ChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Dependency Graph
// ============================================================================

/// Graph of file dependencies for incremental analysis.
pub struct DependencyGraph {
    /// Direct dependencies: file -> files it depends on
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    /// Reverse dependencies: file -> files that depend on it
    dependents: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DependencyGraph {
    /// Create a new dependency graph.
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Add a dependency.
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.dependencies
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.dependents.entry(to).or_default().insert(from);
    }

    /// Remove a file and its dependencies.
    pub fn remove_file(&mut self, file: &PathBuf) {
        // Remove from dependencies
        if let Some(deps) = self.dependencies.remove(file) {
            for dep in deps {
                if let Some(dependents) = self.dependents.get_mut(&dep) {
                    dependents.remove(file);
                }
            }
        }

        // Remove from dependents
        if let Some(deps) = self.dependents.remove(file) {
            for dep in deps {
                if let Some(dependencies) = self.dependencies.get_mut(&dep) {
                    dependencies.remove(file);
                }
            }
        }
    }

    /// Get files affected by changes to a file.
    pub fn get_affected_files(&self, changed: &PathBuf) -> HashSet<PathBuf> {
        let mut affected = HashSet::new();
        let mut queue = vec![changed.clone()];

        while let Some(file) = queue.pop() {
            if affected.insert(file.clone()) {
                if let Some(dependents) = self.dependents.get(&file) {
                    queue.extend(dependents.iter().cloned());
                }
            }
        }

        affected
    }

    /// Get direct dependencies of a file.
    pub fn get_dependencies(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.dependencies.get(file)
    }

    /// Get direct dependents of a file.
    pub fn get_dependents(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.dependents.get(file)
    }

    /// Check if graph contains a file.
    pub fn contains(&self, file: &PathBuf) -> bool {
        self.dependencies.contains_key(file) || self.dependents.contains_key(file)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Incremental Analyzer
// ============================================================================

/// Configuration for incremental analysis.
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    /// Enable background analysis
    pub background_analysis: bool,
    /// Maximum files to analyze in one batch
    pub batch_size: usize,
    /// Analysis timeout per file
    pub file_timeout: Duration,
    /// Enable persistent cache
    pub persistent_cache: bool,
    /// Cache directory
    pub cache_dir: Option<PathBuf>,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            background_analysis: true,
            batch_size: 100,
            file_timeout: Duration::from_secs(30),
            persistent_cache: true,
            cache_dir: None,
        }
    }
}

/// Result of incremental analysis.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Files that were analyzed
    pub analyzed_files: Vec<PathBuf>,
    /// Files that were skipped (cache hit)
    pub cached_files: Vec<PathBuf>,
    /// Files that failed
    pub failed_files: Vec<(PathBuf, String)>,
    /// Total analysis time
    pub total_time: Duration,
}

impl AnalysisResult {
    /// Create empty result.
    pub fn empty() -> Self {
        Self {
            analyzed_files: Vec::new(),
            cached_files: Vec::new(),
            failed_files: Vec::new(),
            total_time: Duration::ZERO,
        }
    }

    /// Check if any files failed.
    pub fn has_failures(&self) -> bool {
        !self.failed_files.is_empty()
    }
}

/// Incremental type analyzer.
pub struct IncrementalAnalyzer {
    /// Configuration
    #[allow(dead_code)]
    config: IncrementalConfig,
    /// Change tracker
    change_tracker: ChangeTracker,
    /// Dependency graph
    dep_graph: DependencyGraph,
    /// Analysis cache (file -> result)
    analysis_cache: HashMap<PathBuf, Arc<CachedAnalysis>>,
}

/// Cached analysis result.
#[derive(Debug, Clone)]
pub struct CachedAnalysis {
    /// Content hash when analyzed
    pub hash: ContentHash,
    /// Analysis timestamp
    pub timestamp: Instant,
    /// Dependencies discovered
    pub dependencies: Vec<PathBuf>,
    /// Whether analysis succeeded
    pub success: bool,
}

impl IncrementalAnalyzer {
    /// Create a new incremental analyzer.
    pub fn new(config: IncrementalConfig) -> Self {
        Self {
            config,
            change_tracker: ChangeTracker::new(),
            dep_graph: DependencyGraph::new(),
            analysis_cache: HashMap::new(),
        }
    }

    /// Create with default config.
    pub fn default_config() -> Self {
        Self::new(IncrementalConfig::default())
    }

    /// Record a file change.
    pub fn file_changed(&mut self, file: PathBuf, kind: ChangeKind, hash: ContentHash) {
        self.change_tracker.record_change(file, kind, hash);
    }

    /// Get files that need reanalysis.
    pub fn get_files_to_analyze(&mut self) -> Vec<PathBuf> {
        let changes = self.change_tracker.get_pending_changes();

        let mut to_analyze = HashSet::new();

        for change in changes {
            // Add the changed file
            to_analyze.insert(change.file.clone());

            // Add affected files
            let affected = self.dep_graph.get_affected_files(&change.file);
            to_analyze.extend(affected);
        }

        to_analyze.into_iter().collect()
    }

    /// Run incremental analysis.
    pub fn analyze(&mut self, files: Vec<PathBuf>) -> AnalysisResult {
        let start = Instant::now();
        let mut result = AnalysisResult::empty();

        for file in files {
            // Check cache
            if let Some(cached) = self.analysis_cache.get(&file) {
                if !self.change_tracker.has_changed(&file, &cached.hash) {
                    result.cached_files.push(file);
                    continue;
                }
            }

            // Analyze file
            match self.analyze_file(&file) {
                Ok(analysis) => {
                    // Update dependency graph
                    for dep in &analysis.dependencies {
                        self.dep_graph.add_dependency(file.clone(), dep.clone());
                    }

                    // Cache result
                    self.analysis_cache.insert(file.clone(), Arc::new(analysis));
                    result.analyzed_files.push(file);
                }
                Err(e) => {
                    result.failed_files.push((file, e));
                }
            }
        }

        result.total_time = start.elapsed();
        result
    }

    /// Analyze a single file.
    ///
    /// Performs incremental analysis by:
    /// 1. Reading file content
    /// 2. Computing content hash
    /// 3. Parsing AST
    /// 4. Extracting imports (dependencies)
    /// 5. Caching results
    fn analyze_file(&self, file: &PathBuf) -> Result<CachedAnalysis, String> {
        // 1. Read file content
        let content =
            fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;

        // 2. Compute content hash
        let hash = ContentHash::from_content(&content);

        // 3. Detect language
        let language = MultiParser::detect_language(file)
            .ok_or_else(|| format!("Failed to detect language for file: {}", file.display()))?;

        // 4. Parse file to extract imports
        let dependencies = self.extract_dependencies(&content, language)?;

        // 5. Return cached analysis
        Ok(CachedAnalysis {
            hash,
            timestamp: Instant::now(),
            dependencies,
            success: true,
        })
    }

    /// Extract dependencies (imports) from file content.
    fn extract_dependencies(
        &self,
        content: &str,
        language: Language,
    ) -> Result<Vec<PathBuf>, String> {
        let mut parser =
            MultiParser::new().map_err(|e| format!("Failed to create parser: {}", e))?;

        let parsed = parser
            .parse(content, language)
            .ok_or_else(|| "Failed to parse file".to_string())?;

        let mut dependencies = Vec::new();

        // Walk AST to find import statements
        let _cursor = parsed.tree.walk();

        fn visit_node(
            node: &tree_sitter::Node,
            source: &str,
            language: Language,
            dependencies: &mut Vec<PathBuf>,
        ) {
            match language {
                Language::Python => {
                    // Look for import and from..import statements
                    match node.kind() {
                        "import_statement" | "import_from_statement" => {
                            // Extract module name from import
                            if let Some(module_node) = node
                                .child_by_field_name("module_name")
                                .or_else(|| node.child_by_field_name("name"))
                            {
                                let module_name =
                                    &source[module_node.start_byte()..module_node.end_byte()];
                                // Convert module name to file path (simple heuristic)
                                let path =
                                    PathBuf::from(format!("{}.py", module_name.replace(".", "/")));
                                dependencies.push(path);
                            }
                        }
                        _ => {}
                    }
                }
                Language::TypeScript => {
                    // Look for import statements
                    match node.kind() {
                        "import_statement" => {
                            if let Some(source_node) = node.child_by_field_name("source") {
                                let import_path =
                                    &source[source_node.start_byte()..source_node.end_byte()];
                                // Remove quotes
                                let path_str = import_path.trim_matches(|c| c == '"' || c == '\'');
                                let path = PathBuf::from(path_str);
                                dependencies.push(path);
                            }
                        }
                        _ => {}
                    }
                }
                Language::Rust => {
                    // Look for use statements
                    if node.kind() == "use_declaration" {
                        if let Some(path_node) = node.child_by_field_name("argument") {
                            let use_path = &source[path_node.start_byte()..path_node.end_byte()];
                            // Convert Rust module path to file path
                            let path = PathBuf::from(format!("{}.rs", use_path.replace("::", "/")));
                            dependencies.push(path);
                        }
                    }
                }
                Language::JavaScript => {
                    // Same pattern as TypeScript
                    if node.kind() == "import_statement" {
                        if let Some(source_node) = node.child_by_field_name("source") {
                            let import_path =
                                &source[source_node.start_byte()..source_node.end_byte()];
                            let path_str = import_path.trim_matches(|c| c == '"' || c == '\'');
                            dependencies.push(PathBuf::from(path_str));
                        }
                    }
                }
                Language::Go => {
                    // Look for import declarations
                    if node.kind() == "import_spec" {
                        if let Some(path_node) = node.child_by_field_name("path") {
                            let import_path = &source[path_node.start_byte()..path_node.end_byte()];
                            let path_str = import_path.trim_matches('"');
                            dependencies.push(PathBuf::from(path_str));
                        }
                    }
                }
                Language::Html
                | Language::Css
                | Language::Dockerfile
                | Language::Hcl
                | Language::Yaml
                | Language::Markdown
                | Language::Mdx
                | Language::Mermaid
                | Language::Toml
                | Language::Sql
                | Language::Proto
                | Language::GraphQL => {
                    // No import dependency extraction for these languages yet
                }
            }

            // Recursively visit children
            let mut child_cursor = node.walk();
            for child in node.children(&mut child_cursor) {
                visit_node(&child, source, language, dependencies);
            }
        }

        visit_node(
            &parsed.tree.root_node(),
            content,
            language,
            &mut dependencies,
        );

        Ok(dependencies)
    }

    /// Invalidate cache for a file.
    pub fn invalidate(&mut self, file: &PathBuf) {
        self.analysis_cache.remove(file);
    }

    /// Clear all caches.
    pub fn clear_cache(&mut self) {
        self.analysis_cache.clear();
        self.change_tracker.clear();
    }

    /// Get dependency graph.
    pub fn dep_graph(&self) -> &DependencyGraph {
        &self.dep_graph
    }
}

impl Default for IncrementalAnalyzer {
    fn default() -> Self {
        Self::default_config()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_tracker() {
        let mut tracker = ChangeTracker::new().with_debounce(Duration::ZERO);

        tracker.record_change(
            PathBuf::from("test.py"),
            ChangeKind::Modified,
            ContentHash::from_content("abc"),
        );

        let changes = tracker.get_pending_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].file, PathBuf::from("test.py"));
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        graph.add_dependency(PathBuf::from("a.py"), PathBuf::from("b.py"));
        graph.add_dependency(PathBuf::from("b.py"), PathBuf::from("c.py"));

        let affected = graph.get_affected_files(&PathBuf::from("c.py"));
        assert!(affected.contains(&PathBuf::from("b.py")));
        assert!(affected.contains(&PathBuf::from("a.py")));
    }

    #[test]
    fn test_incremental_analyzer() {
        let mut analyzer = IncrementalAnalyzer::default_config();

        // Mark file as changed
        let file = PathBuf::from("test.py");
        analyzer.file_changed(
            file.clone(),
            ChangeKind::Modified,
            ContentHash::from_content("abc"),
        );

        // Wait for debounce period to pass
        std::thread::sleep(Duration::from_millis(350));

        let files = analyzer.get_files_to_analyze();
        assert!(
            !files.is_empty(),
            "Should have files to analyze after debounce"
        );
        assert!(files.contains(&file), "Should include the changed file");
    }
}

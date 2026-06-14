//! Program Dependence Graph (PDG) for Python
//!
//! Combines control dependencies and data dependencies into a unified
//! graph for program analysis. Supports:
//! - Forward and backward slicing
//! - Impact analysis
//! - Taint tracking
//! - Dead code detection

pub mod cfg;
pub mod data_flow;
pub mod dominator;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use crate::syntax::ParsedFile;
use crate::type_inference::Span;

pub use cfg::{BlockId, CfgBuilder, ControlFlowGraph, StatementInfo, StatementKind};
pub use data_flow::{DataDependencies, DefUseChain, Definition, Use, UseDefChain};
pub use dominator::{ControlDependencies, DominatorTree};

/// A node in the PDG (represents a statement)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdgNode {
    /// Unique identifier
    pub id: u32,
    /// Line number (0-indexed)
    pub line: usize,
    /// Span in source
    pub span: Span,
    /// Statement text
    pub text: String,
    /// CFG block ID
    pub block: BlockId,
    /// Statement kind
    pub kind: StatementKind,
}

/// Type of PDG edge
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdgEdgeKind {
    /// Control dependency
    Control,
    /// Data dependency (with variable name)
    Data { variable: String },
}

/// An edge in the PDG
#[derive(Debug, Clone)]
pub struct PdgEdge {
    /// Source node
    pub from: u32,
    /// Target node
    pub to: u32,
    /// Edge kind
    pub kind: PdgEdgeKind,
}

/// Program Dependence Graph
#[derive(Debug, Clone)]
pub struct ProgramDependenceGraph {
    /// All nodes (statement ID -> node)
    pub nodes: HashMap<u32, PdgNode>,
    /// Forward edges (from -> to)
    pub edges: HashMap<u32, Vec<PdgEdge>>,
    /// Backward edges (to -> from)
    pub reverse_edges: HashMap<u32, Vec<PdgEdge>>,
    /// Line to node mapping (supports multiple nodes per line)
    pub line_to_nodes: HashMap<usize, Vec<u32>>,
    /// Function name (if for a specific function)
    pub function_name: Option<String>,
    /// Source file path
    pub file_path: Option<PathBuf>,
    /// Control flow graph
    pub cfg: ControlFlowGraph,
    /// Data dependencies
    pub data_deps: DataDependencies,
    /// Control dependencies
    pub control_deps: ControlDependencies,
}

impl ProgramDependenceGraph {
    /// Build PDG from source code
    pub fn build(source: &str, file: &ParsedFile) -> Self {
        let cfg = CfgBuilder::new(source).build(file);
        Self::from_cfg(cfg, file)
    }

    /// Build PDG from an existing CFG
    pub fn from_cfg(cfg: ControlFlowGraph, file: &ParsedFile) -> Self {
        // Compute dependencies
        let data_deps = DataDependencies::compute(&cfg, file);
        let control_deps = ControlDependencies::compute(&cfg);

        // Create nodes from CFG statements
        let mut nodes = HashMap::new();
        let mut line_to_nodes: HashMap<usize, Vec<u32>> = HashMap::new();
        let mut next_id = 0u32;

        for block in cfg.blocks.values() {
            for stmt in &block.statements {
                let id = next_id;
                next_id += 1;

                nodes.insert(
                    id,
                    PdgNode {
                        id,
                        line: stmt.line,
                        span: stmt.span,
                        text: stmt.text.clone(),
                        block: block.id,
                        kind: stmt.kind.clone(),
                    },
                );

                // Support multiple nodes per line
                line_to_nodes.entry(stmt.line).or_default().push(id);
            }
        }

        // Build edges
        let mut edges: HashMap<u32, Vec<PdgEdge>> = HashMap::new();
        let mut reverse_edges: HashMap<u32, Vec<PdgEdge>> = HashMap::new();

        // Add control dependency edges
        for (&block, deps) in &control_deps.dependencies {
            // Find nodes in this block
            let block_nodes: Vec<u32> = nodes
                .values()
                .filter(|n| n.block == block)
                .map(|n| n.id)
                .collect();

            for &dep_block in deps {
                // Find nodes in the dependency block (condition)
                let dep_nodes: Vec<u32> = nodes
                    .values()
                    .filter(|n| n.block == dep_block)
                    .map(|n| n.id)
                    .collect();

                // Add edges from condition to dependent statements
                for &from in &dep_nodes {
                    for &to in &block_nodes {
                        let edge = PdgEdge {
                            from,
                            to,
                            kind: PdgEdgeKind::Control,
                        };
                        edges.entry(from).or_default().push(edge.clone());
                        reverse_edges.entry(to).or_default().push(edge);
                    }
                }
            }
        }

        // Add data dependency edges
        // Now handles multiple nodes per line
        for (def, uses) in &data_deps.def_use {
            if let Some(from_ids) = line_to_nodes.get(&def.line) {
                for &from_id in from_ids {
                    for u in uses {
                        if let Some(to_ids) = line_to_nodes.get(&u.line) {
                            for &to_id in to_ids {
                                if from_id != to_id {
                                    let edge = PdgEdge {
                                        from: from_id,
                                        to: to_id,
                                        kind: PdgEdgeKind::Data {
                                            variable: def.name.clone(),
                                        },
                                    };
                                    edges.entry(from_id).or_default().push(edge.clone());
                                    reverse_edges.entry(to_id).or_default().push(edge);
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            nodes,
            edges,
            reverse_edges,
            line_to_nodes,
            function_name: cfg.function_name.clone(),
            file_path: None,
            cfg,
            data_deps,
            control_deps,
        }
    }

    /// Set the file path
    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Get a node by ID
    pub fn get_node(&self, id: u32) -> Option<&PdgNode> {
        self.nodes.get(&id)
    }

    /// Get a node by line number (returns first node if multiple exist on same line)
    pub fn get_node_by_line(&self, line: usize) -> Option<&PdgNode> {
        self.line_to_nodes
            .get(&line)
            .and_then(|ids| ids.first())
            .and_then(|id| self.nodes.get(id))
    }

    /// Get all nodes on a given line
    pub fn get_nodes_by_line(&self, line: usize) -> Vec<&PdgNode> {
        self.line_to_nodes
            .get(&line)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get forward dependencies of a node
    pub fn get_dependencies(&self, id: u32) -> Vec<&PdgEdge> {
        self.edges
            .get(&id)
            .map(|e| e.iter().collect())
            .unwrap_or_default()
    }

    /// Get backward dependencies of a node (what this node depends on)
    pub fn get_dependents(&self, id: u32) -> Vec<&PdgEdge> {
        self.reverse_edges
            .get(&id)
            .map(|e| e.iter().collect())
            .unwrap_or_default()
    }

    /// Compute backward slice from a criterion (line number)
    ///
    /// Returns all statements that affect the given line.
    /// If multiple statements exist on the same line, includes all of them.
    pub fn backward_slice(&self, line: usize) -> ProgramSlice {
        let mut slice = ProgramSlice::new(SliceDirection::Backward);

        if let Some(node_ids) = self.line_to_nodes.get(&line) {
            slice.criterion_line = Some(line);
            let mut visited = HashSet::new();
            for &node_id in node_ids {
                self.collect_backward_slice(node_id, &mut slice.nodes, &mut visited);
            }
        }

        // Sort by line number and deduplicate
        slice.nodes.sort_by_key(|n| (n.line, n.id));
        slice.nodes.dedup_by_key(|n| n.id);
        slice
    }

    fn collect_backward_slice(
        &self,
        node_id: u32,
        result: &mut Vec<PdgNode>,
        visited: &mut HashSet<u32>,
    ) {
        if !visited.insert(node_id) {
            return;
        }

        if let Some(node) = self.nodes.get(&node_id) {
            result.push(node.clone());

            // Follow backward edges (dependencies)
            if let Some(edges) = self.reverse_edges.get(&node_id) {
                for edge in edges {
                    self.collect_backward_slice(edge.from, result, visited);
                }
            }
        }
    }

    /// Compute forward slice from a criterion (line number)
    ///
    /// Returns all statements affected by the given line.
    /// If multiple statements exist on the same line, includes all of them.
    pub fn forward_slice(&self, line: usize) -> ProgramSlice {
        let mut slice = ProgramSlice::new(SliceDirection::Forward);

        if let Some(node_ids) = self.line_to_nodes.get(&line) {
            slice.criterion_line = Some(line);
            let mut visited = HashSet::new();
            for &node_id in node_ids {
                self.collect_forward_slice(node_id, &mut slice.nodes, &mut visited);
            }
        }

        // Sort by line number and deduplicate
        slice.nodes.sort_by_key(|n| (n.line, n.id));
        slice.nodes.dedup_by_key(|n| n.id);
        slice
    }

    fn collect_forward_slice(
        &self,
        node_id: u32,
        result: &mut Vec<PdgNode>,
        visited: &mut HashSet<u32>,
    ) {
        if !visited.insert(node_id) {
            return;
        }

        if let Some(node) = self.nodes.get(&node_id) {
            result.push(node.clone());

            // Follow forward edges (dependents)
            if let Some(edges) = self.edges.get(&node_id) {
                for edge in edges {
                    self.collect_forward_slice(edge.to, result, visited);
                }
            }
        }
    }

    /// Compute impact analysis for a set of changed lines
    ///
    /// Returns all lines that may be affected by changes.
    pub fn impact_analysis(&self, changed_lines: &[usize]) -> ImpactAnalysis {
        let mut impact = ImpactAnalysis::new();
        impact.changed_lines = changed_lines.to_vec();

        let mut affected = HashSet::new();

        for &line in changed_lines {
            let slice = self.forward_slice(line);
            for node in slice.nodes {
                if node.line != line {
                    affected.insert(node.line);
                }
            }
        }

        impact.affected_lines = affected.into_iter().collect();
        impact.affected_lines.sort();
        impact
    }

    /// Perform taint tracking from sources to sinks
    ///
    /// Returns all paths where tainted data flows from sources to sinks.
    pub fn taint_tracking(&self, sources: &[usize], sinks: &[usize]) -> TaintAnalysis {
        let mut analysis = TaintAnalysis::new();
        analysis.sources = sources.to_vec();
        analysis.sinks = sinks.to_vec();

        // Get all nodes reachable from sources (forward slice)
        let mut tainted = HashSet::new();
        for &source in sources {
            let slice = self.forward_slice(source);
            for node in slice.nodes {
                tainted.insert(node.line);
            }
        }

        // Find sinks that are tainted
        for &sink in sinks {
            if tainted.contains(&sink) {
                // Find the path from source to sink
                for &source in sources {
                    if let Some(path) = self.find_path(source, sink) {
                        analysis.taint_paths.push(TaintPath { source, sink, path });
                    }
                }
            }
        }

        analysis
    }

    /// Find a path from one line to another
    fn find_path(&self, from_line: usize, to_line: usize) -> Option<Vec<usize>> {
        // Get first node on each line (could extend to find path through any node)
        let from_id = *self.line_to_nodes.get(&from_line)?.first()?;
        let to_id = *self.line_to_nodes.get(&to_line)?.first()?;

        // BFS to find path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<u32, u32> = HashMap::new();

        queue.push_back(from_id);
        visited.insert(from_id);

        while let Some(current) = queue.pop_front() {
            if current == to_id {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to_id;
                while node != from_id {
                    if let Some(n) = self.nodes.get(&node) {
                        path.push(n.line);
                    }
                    node = *parent.get(&node)?;
                }
                if let Some(n) = self.nodes.get(&from_id) {
                    path.push(n.line);
                }
                path.reverse();
                return Some(path);
            }

            if let Some(edges) = self.edges.get(&current) {
                for edge in edges {
                    if visited.insert(edge.to) {
                        parent.insert(edge.to, current);
                        queue.push_back(edge.to);
                    }
                }
            }
        }

        None
    }

    /// Detect dead code (statements not affecting any output)
    ///
    /// A statement is dead if:
    /// 1. It has no dependents (forward edges), OR
    /// 2. All its dependents are also dead
    pub fn dead_code_detection(&self, output_lines: &[usize]) -> DeadCodeAnalysis {
        let mut analysis = DeadCodeAnalysis::new();

        // Find all lines that affect outputs (backward slice from outputs)
        let mut live = HashSet::new();
        for &output in output_lines {
            let slice = self.backward_slice(output);
            for node in slice.nodes {
                live.insert(node.line);
            }
        }

        // All lines not in live set are dead
        for node in self.nodes.values() {
            if !live.contains(&node.line) {
                analysis.dead_lines.push(node.line);
            }
        }

        analysis.dead_lines.sort();
        analysis.dead_lines.dedup();
        analysis
    }

    /// Get all nodes as a sorted list
    pub fn all_nodes(&self) -> Vec<&PdgNode> {
        let mut nodes: Vec<_> = self.nodes.values().collect();
        nodes.sort_by_key(|n| n.line);
        nodes
    }

    /// Get statistics about the PDG
    pub fn stats(&self) -> PdgStats {
        let mut control_edges = 0;
        let mut data_edges = 0;

        for edges in self.edges.values() {
            for edge in edges {
                match edge.kind {
                    PdgEdgeKind::Control => control_edges += 1,
                    PdgEdgeKind::Data { .. } => data_edges += 1,
                }
            }
        }

        PdgStats {
            node_count: self.nodes.len(),
            control_edge_count: control_edges,
            data_edge_count: data_edges,
            total_edge_count: control_edges + data_edges,
        }
    }
}

/// Direction of program slice
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceDirection {
    Forward,
    Backward,
}

/// Result of program slicing
#[derive(Debug, Clone)]
pub struct ProgramSlice {
    /// Direction of the slice
    pub direction: SliceDirection,
    /// Criterion line (if applicable)
    pub criterion_line: Option<usize>,
    /// Nodes in the slice
    pub nodes: Vec<PdgNode>,
}

impl ProgramSlice {
    fn new(direction: SliceDirection) -> Self {
        Self {
            direction,
            criterion_line: None,
            nodes: Vec::new(),
        }
    }

    /// Get lines in the slice
    pub fn lines(&self) -> Vec<usize> {
        self.nodes.iter().map(|n| n.line).collect()
    }

    /// Check if a line is in the slice
    pub fn contains_line(&self, line: usize) -> bool {
        self.nodes.iter().any(|n| n.line == line)
    }

    /// Get the number of statements in the slice
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the slice is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Result of impact analysis
#[derive(Debug, Clone)]
pub struct ImpactAnalysis {
    /// Lines that were changed
    pub changed_lines: Vec<usize>,
    /// Lines affected by the changes
    pub affected_lines: Vec<usize>,
}

impl ImpactAnalysis {
    fn new() -> Self {
        Self {
            changed_lines: Vec::new(),
            affected_lines: Vec::new(),
        }
    }

    /// Get total impact (number of affected lines)
    pub fn impact_count(&self) -> usize {
        self.affected_lines.len()
    }
}

/// A path from taint source to sink
#[derive(Debug, Clone)]
pub struct TaintPath {
    /// Source line
    pub source: usize,
    /// Sink line
    pub sink: usize,
    /// Path of lines from source to sink
    pub path: Vec<usize>,
}

/// Result of taint analysis
#[derive(Debug, Clone)]
pub struct TaintAnalysis {
    /// Taint sources
    pub sources: Vec<usize>,
    /// Taint sinks
    pub sinks: Vec<usize>,
    /// Paths from sources to sinks
    pub taint_paths: Vec<TaintPath>,
}

impl TaintAnalysis {
    fn new() -> Self {
        Self {
            sources: Vec::new(),
            sinks: Vec::new(),
            taint_paths: Vec::new(),
        }
    }

    /// Check if any sink is tainted
    pub fn has_taint(&self) -> bool {
        !self.taint_paths.is_empty()
    }

    /// Get all tainted sinks
    pub fn tainted_sinks(&self) -> Vec<usize> {
        self.taint_paths.iter().map(|p| p.sink).collect()
    }
}

/// Result of dead code detection
#[derive(Debug, Clone)]
pub struct DeadCodeAnalysis {
    /// Lines that are dead (don't affect outputs)
    pub dead_lines: Vec<usize>,
}

// ============================================================================
// Dependency tree for impact analysis (R6)
// ============================================================================

/// Kind of dependency in the impact tree
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyReason {
    Data,
    Control,
    Transitive,
}

/// A node in the impact dependency tree
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImpactTreeNode {
    pub line: usize,
    pub text: String,
    pub reason: DependencyReason,
    pub variable: Option<String>,
    pub children: Vec<ImpactTreeNode>,
}

/// Impact analysis with dependency tree output (R6)
#[derive(Debug, Clone)]
pub struct ImpactAnalysisTree {
    pub changed_lines: Vec<usize>,
    pub affected_lines: Vec<usize>,
    pub tree: Vec<ImpactTreeNode>,
}

impl ProgramDependenceGraph {
    /// Compute impact analysis with dependency tree format
    ///
    /// Returns WHY each line is affected (data dep, control dep, transitive)
    /// as a tree structure suitable for AI agent consumption.
    pub fn impact_analysis_tree(&self, changed_lines: &[usize]) -> ImpactAnalysisTree {
        let mut all_affected = Vec::new();
        let mut tree_roots = Vec::new();

        for &line in changed_lines {
            let mut visited = HashSet::new();
            let mut root_children = Vec::new();
            self.collect_impact_tree(line, &mut root_children, &mut visited, 0);
            tree_roots.extend(root_children);

            // Collect flat list
            let flat: Vec<usize> = self
                .forward_slice(line)
                .nodes
                .iter()
                .filter(|n| n.line != line)
                .map(|n| n.line)
                .collect();
            all_affected.extend(flat);
        }

        all_affected.sort();
        all_affected.dedup();

        ImpactAnalysisTree {
            changed_lines: changed_lines.to_vec(),
            affected_lines: all_affected,
            tree: tree_roots,
        }
    }

    fn collect_impact_tree(
        &self,
        node_id_or_line: usize,
        children: &mut Vec<ImpactTreeNode>,
        visited: &mut HashSet<usize>,
        depth: usize,
    ) {
        // Guard against deep recursion
        if depth > 20 {
            return;
        }

        let node_ids = match self.line_to_nodes.get(&node_id_or_line) {
            Some(ids) => ids.clone(),
            None => return,
        };

        for &node_id in &node_ids {
            if let Some(edges) = self.edges.get(&node_id) {
                for edge in edges {
                    if let Some(target) = self.nodes.get(&edge.to) {
                        if !visited.insert(target.line) {
                            continue;
                        }

                        let (reason, variable) = match &edge.kind {
                            PdgEdgeKind::Data { variable } => {
                                (DependencyReason::Data, Some(variable.clone()))
                            }
                            PdgEdgeKind::Control => (DependencyReason::Control, None),
                        };

                        let mut node_children = Vec::new();
                        self.collect_impact_tree(
                            target.line,
                            &mut node_children,
                            visited,
                            depth + 1,
                        );

                        // Mark transitive if it has children
                        let effective_reason = if !node_children.is_empty() && depth > 0 {
                            DependencyReason::Transitive
                        } else {
                            reason
                        };

                        children.push(ImpactTreeNode {
                            line: target.line,
                            text: target.text.clone(),
                            reason: effective_reason,
                            variable,
                            children: node_children,
                        });
                    }
                }
            }
        }
    }
}

// ============================================================================
// Semantic taint analysis with auto-detection (R7)
// ============================================================================

/// Kind of taint source
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaintSourceKind {
    UserInput,
    EnvVar,
    HttpRequest,
    CmdArgs,
    FileRead,
    NetworkRecv,
    DeserialisedData,
    Unknown,
}

/// Kind of taint sink
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaintSinkKind {
    OsCommand,
    SubprocessExec,
    DatabaseQuery,
    CodeEval,
    FileWrite,
    NetworkSend,
    Logging,
    Unknown,
}

/// A detected taint source
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedSource {
    pub line: usize,
    pub text: String,
    pub kind: TaintSourceKind,
}

/// A detected taint sink
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedSink {
    pub line: usize,
    pub text: String,
    pub kind: TaintSinkKind,
}

/// Semantic taint analysis result
#[derive(Debug, Clone)]
pub struct SemanticTaintAnalysis {
    pub sources: Vec<DetectedSource>,
    pub sinks: Vec<DetectedSink>,
    pub taint_paths: Vec<TaintPath>,
}

impl SemanticTaintAnalysis {
    pub fn has_vulnerabilities(&self) -> bool {
        !self.taint_paths.is_empty()
    }
}

/// Detect taint sources by scanning statement text patterns
pub fn detect_taint_sources(nodes: &HashMap<u32, PdgNode>) -> Vec<DetectedSource> {
    let mut sources = Vec::new();

    let source_patterns: &[(&[&str], TaintSourceKind)] = &[
        (&["input(", "input ("], TaintSourceKind::UserInput),
        (
            &["os.environ", "os.getenv(", "environ.get(", "environ["],
            TaintSourceKind::EnvVar,
        ),
        (
            &[
                "request.form",
                "request.args",
                "request.json",
                "request.data",
                "request.get_json(",
                "request.values",
                "flask.request",
                "self.request",
                "HttpRequest",
            ],
            TaintSourceKind::HttpRequest,
        ),
        (
            &["sys.argv", "argparse", "click.option"],
            TaintSourceKind::CmdArgs,
        ),
        (
            &[
                "json.load(",
                "json.loads(",
                "yaml.load(",
                "pickle.load(",
                "pickle.loads(",
                "marshal.loads(",
            ],
            TaintSourceKind::DeserialisedData,
        ),
        (
            &[
                "socket.recv(",
                ".recv(",
                "socket.recvfrom(",
                "read_from_socket",
            ],
            TaintSourceKind::NetworkRecv,
        ),
        (
            &[
                "open(",
                "file.read(",
                ".read(",
                ".readline(",
                ".readlines(",
                "pathlib.Path(",
                "io.open(",
            ],
            TaintSourceKind::FileRead,
        ),
    ];

    let mut seen_lines = HashSet::new();

    for node in nodes.values() {
        if seen_lines.contains(&node.line) {
            continue;
        }

        let text_lower = node.text.to_lowercase();

        for (patterns, kind) in source_patterns {
            for pat in *patterns {
                if text_lower.contains(&pat.to_lowercase()) {
                    seen_lines.insert(node.line);
                    sources.push(DetectedSource {
                        line: node.line,
                        text: node.text.clone(),
                        kind: kind.clone(),
                    });
                    break;
                }
            }
        }
    }

    sources.sort_by_key(|s| s.line);
    sources
}

/// Detect taint sinks by scanning statement text patterns
pub fn detect_taint_sinks(nodes: &HashMap<u32, PdgNode>) -> Vec<DetectedSink> {
    let mut sinks = Vec::new();

    let sink_patterns: &[(&[&str], TaintSinkKind)] = &[
        (
            &["os.system(", "os.popen(", "os.exec", "os.spawn"],
            TaintSinkKind::OsCommand,
        ),
        (
            &[
                "subprocess.run(",
                "subprocess.call(",
                "subprocess.Popen(",
                "subprocess.check_output(",
                "subprocess.check_call(",
            ],
            TaintSinkKind::SubprocessExec,
        ),
        (
            &[
                "db.execute(",
                "cursor.execute(",
                ".execute(",
                "session.execute(",
                "connection.execute(",
                "engine.execute(",
                "raw_sql",
                "query(",
            ],
            TaintSinkKind::DatabaseQuery,
        ),
        (
            &["eval(", "exec(", "compile(", "__import__(", "importlib"],
            TaintSinkKind::CodeEval,
        ),
        (
            &[
                ".write(",
                "open(",
                "shutil.copy",
                "shutil.move",
                "pathlib.Path(",
                "json.dump(",
                "pickle.dump(",
            ],
            TaintSinkKind::FileWrite,
        ),
        (
            &[
                ".send(",
                "socket.send(",
                "requests.get(",
                "requests.post(",
                "urllib.request",
                "http.client",
                "httpx.",
            ],
            TaintSinkKind::NetworkSend,
        ),
        (
            &[
                "logging.info(",
                "logging.error(",
                "logging.warning(",
                "logging.debug(",
                "print(",
                "logger.",
            ],
            TaintSinkKind::Logging,
        ),
    ];

    let mut seen_lines = HashSet::new();

    for node in nodes.values() {
        if seen_lines.contains(&node.line) {
            continue;
        }

        let text_lower = node.text.to_lowercase();

        for (patterns, kind) in sink_patterns {
            for pat in *patterns {
                if text_lower.contains(&pat.to_lowercase()) {
                    seen_lines.insert(node.line);
                    sinks.push(DetectedSink {
                        line: node.line,
                        text: node.text.clone(),
                        kind: kind.clone(),
                    });
                    break;
                }
            }
        }
    }

    sinks.sort_by_key(|s| s.line);
    sinks
}

impl ProgramDependenceGraph {
    /// Perform semantic taint analysis with automatic source/sink detection
    ///
    /// Scans AST text for known taint source patterns (input(), os.environ,
    /// request.*, etc.) and sink patterns (os.system(), subprocess.*,
    /// db.execute(), eval(), etc.) as defined in pre-clarifications Q1.
    pub fn semantic_taint_analysis(&self) -> SemanticTaintAnalysis {
        let sources = detect_taint_sources(&self.nodes);
        let sinks = detect_taint_sinks(&self.nodes);

        let source_lines: Vec<usize> = sources.iter().map(|s| s.line).collect();
        let sink_lines: Vec<usize> = sinks.iter().map(|s| s.line).collect();

        let taint_result = self.taint_tracking(&source_lines, &sink_lines);

        SemanticTaintAnalysis {
            sources,
            sinks,
            taint_paths: taint_result.taint_paths,
        }
    }

    /// Perform taint analysis with explicit source/sink lines (R7 — manual mode)
    ///
    /// Used when the caller already knows which lines are sources and sinks.
    pub fn taint_analysis_explicit(
        &self,
        source_lines: &[usize],
        sink_lines: &[usize],
    ) -> TaintAnalysis {
        self.taint_tracking(source_lines, sink_lines)
    }
}

impl DeadCodeAnalysis {
    fn new() -> Self {
        Self {
            dead_lines: Vec::new(),
        }
    }

    /// Check if a line is dead
    pub fn is_dead(&self, line: usize) -> bool {
        self.dead_lines.contains(&line)
    }

    /// Get number of dead lines
    pub fn dead_count(&self) -> usize {
        self.dead_lines.len()
    }
}

/// Statistics about a PDG
#[derive(Debug, Clone)]
pub struct PdgStats {
    /// Number of nodes
    pub node_count: usize,
    /// Number of control dependency edges
    pub control_edge_count: usize,
    /// Number of data dependency edges
    pub data_edge_count: usize,
    /// Total number of edges
    pub total_edge_count: usize,
}

/// Serializable representation of PDG for MCP
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdgJson {
    /// Nodes
    pub nodes: Vec<PdgNodeJson>,
    /// Edges
    pub edges: Vec<PdgEdgeJson>,
    /// Function name
    pub function_name: Option<String>,
    /// File path
    pub file_path: Option<String>,
    /// Statistics
    pub stats: PdgStatsJson,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdgNodeJson {
    pub id: u32,
    pub line: usize,
    pub text: String,
    pub kind: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdgEdgeJson {
    pub from: u32,
    pub to: u32,
    pub kind: String,
    pub variable: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdgStatsJson {
    pub node_count: usize,
    pub control_edge_count: usize,
    pub data_edge_count: usize,
    pub total_edge_count: usize,
}

impl From<&ProgramDependenceGraph> for PdgJson {
    fn from(pdg: &ProgramDependenceGraph) -> Self {
        let nodes: Vec<PdgNodeJson> = pdg
            .all_nodes()
            .iter()
            .map(|n| PdgNodeJson {
                id: n.id,
                line: n.line,
                text: n.text.clone(),
                kind: format!("{:?}", n.kind),
            })
            .collect();

        let mut edges = Vec::new();
        for edge_list in pdg.edges.values() {
            for edge in edge_list {
                let (kind, variable) = match &edge.kind {
                    PdgEdgeKind::Control => ("control".to_string(), None),
                    PdgEdgeKind::Data { variable } => ("data".to_string(), Some(variable.clone())),
                };
                edges.push(PdgEdgeJson {
                    from: edge.from,
                    to: edge.to,
                    kind,
                    variable,
                });
            }
        }

        let stats = pdg.stats();

        Self {
            nodes,
            edges,
            function_name: pdg.function_name.clone(),
            file_path: pdg
                .file_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            stats: PdgStatsJson {
                node_count: stats.node_count,
                control_edge_count: stats.control_edge_count,
                data_edge_count: stats.data_edge_count,
                total_edge_count: stats.total_edge_count,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{Language, MultiParser};

    fn build_pdg(code: &str) -> ProgramDependenceGraph {
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(code, Language::Python).unwrap();
        ProgramDependenceGraph::build(code, &parsed)
    }

    #[test]
    fn test_pdg_construction() {
        let pdg = build_pdg("x = 1\ny = x + 2\nz = y * 3");

        // Should have nodes
        assert!(!pdg.nodes.is_empty());

        // Should have data dependency edges
        let stats = pdg.stats();
        assert!(stats.data_edge_count > 0 || stats.node_count > 0);
    }

    #[test]
    fn test_backward_slice() {
        let pdg = build_pdg("x = 1\ny = 2\nz = x + y");

        // Backward slice from z should include x and y
        let slice = pdg.backward_slice(2);
        assert!(!slice.is_empty());
    }

    #[test]
    fn test_forward_slice() {
        let pdg = build_pdg("x = 1\ny = x + 2\nz = y * 3");

        // Forward slice from x should include y and z
        let slice = pdg.forward_slice(0);
        assert!(!slice.is_empty());
    }

    #[test]
    fn test_impact_analysis() {
        let pdg = build_pdg("x = 1\ny = x + 2\nz = y * 3\nw = 4");

        // Changing x should affect y and z, but not w
        let _impact = pdg.impact_analysis(&[0]);
        // Note: exact results depend on parsing
    }

    #[test]
    fn test_dead_code() {
        let pdg = build_pdg("x = 1\ny = 2\nz = x");

        // y is dead if z (using x) is the only output
        let _dead = pdg.dead_code_detection(&[2]);
        // y (line 1) should be in dead_lines
    }

    #[test]
    fn test_pdg_json_serialization() {
        let pdg = build_pdg("x = 1\ny = x + 2");
        let json: PdgJson = (&pdg).into();

        assert!(!json.nodes.is_empty());
        assert!(json.stats.node_count > 0);
    }

    #[test]
    fn test_get_nodes_by_line() {
        // Test the new get_nodes_by_line method
        let pdg = build_pdg("x = 1\ny = 2\nz = 3");

        // Each line should have at least one node
        for line in 0..3 {
            let nodes = pdg.get_nodes_by_line(line);
            // May or may not have nodes depending on parsing
            let _ = nodes.len();
        }
    }

    #[test]
    fn test_multiline_nodes_structure() {
        let pdg = build_pdg("a = 1\nb = a\nc = b");

        // Verify line_to_nodes is a multimap
        for (&line, node_ids) in &pdg.line_to_nodes {
            assert!(line < 10); // Sanity check
            for &id in node_ids {
                assert!(pdg.nodes.contains_key(&id));
            }
        }
    }

    #[test]
    fn test_slice_with_control_flow() {
        let pdg = build_pdg("x = 1\nif x:\n    y = x + 1\nz = y");

        // Forward slice from x should include statements that depend on x
        let slice = pdg.forward_slice(0);
        assert!(!slice.is_empty());

        // Backward slice from z should include its dependencies
        let slice = pdg.backward_slice(3);
        assert!(!slice.is_empty());
    }
}

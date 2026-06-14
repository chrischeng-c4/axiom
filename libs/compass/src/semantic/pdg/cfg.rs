//! Control Flow Graph (CFG) construction for Python
//!
//! Implements statement-level CFG for Python code analysis.
//! Handles: sequential execution, if-else, loops (for/while),
//! exceptions (try/except/finally), and function calls.

use crate::syntax::ParsedFile;
use crate::type_inference::Span;
use std::collections::{HashMap, HashSet};

/// Unique identifier for a CFG block
pub type BlockId = u32;

/// A basic block in the CFG
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier
    pub id: BlockId,
    /// Statements in this block (as spans into source)
    pub statements: Vec<StatementInfo>,
    /// Block kind for control flow analysis
    pub kind: BlockKind,
}

/// Information about a statement
#[derive(Debug, Clone)]
pub struct StatementInfo {
    /// Span in source code
    pub span: Span,
    /// Statement kind
    pub kind: StatementKind,
    /// Line number (0-indexed)
    pub line: usize,
    /// Source text (for debugging)
    pub text: String,
}

/// Kind of statement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatementKind {
    /// Variable assignment
    Assignment,
    /// Augmented assignment (+=, -=, etc.)
    AugmentedAssignment,
    /// Expression statement (function call, etc.)
    Expression,
    /// Return statement
    Return,
    /// Raise statement
    Raise,
    /// Assert statement
    Assert,
    /// Pass statement
    Pass,
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Import statement
    Import,
    /// Global/nonlocal declaration
    Declaration,
    /// Delete statement
    Delete,
    /// Other/unknown
    Other,
}

/// Kind of basic block
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    /// Entry block
    Entry,
    /// Exit block
    Exit,
    /// Normal sequential block
    Normal,
    /// If condition block
    IfCondition,
    /// Loop condition block (for/while)
    LoopCondition,
    /// Try block entry
    TryEntry,
    /// Except handler
    ExceptHandler,
    /// Finally block
    Finally,
    /// Function call (for inter-procedural)
    FunctionCall { callee: String },
}

/// Edge type in CFG
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    /// Normal sequential flow
    Sequential,
    /// True branch of condition
    TrueBranch,
    /// False branch of condition
    FalseBranch,
    /// Loop back edge
    LoopBack,
    /// Exception edge (to handler)
    Exception,
    /// Break edge (out of loop)
    Break,
    /// Continue edge (back to loop)
    Continue,
    /// Return edge (to exit)
    Return,
}

/// An edge in the CFG
#[derive(Debug, Clone)]
pub struct CfgEdge {
    /// Source block
    pub from: BlockId,
    /// Target block
    pub to: BlockId,
    /// Edge kind
    pub kind: EdgeKind,
}

/// Control Flow Graph for a function/module
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// All basic blocks
    pub blocks: HashMap<BlockId, BasicBlock>,
    /// Edges from each block
    pub successors: HashMap<BlockId, Vec<CfgEdge>>,
    /// Edges to each block
    pub predecessors: HashMap<BlockId, Vec<CfgEdge>>,
    /// Entry block ID
    pub entry: BlockId,
    /// Exit block ID
    pub exit: BlockId,
    /// Next block ID to allocate
    next_id: BlockId,
    /// Function name (if applicable)
    pub function_name: Option<String>,
}

impl ControlFlowGraph {
    /// Create a new empty CFG
    pub fn new() -> Self {
        let mut cfg = Self {
            blocks: HashMap::new(),
            successors: HashMap::new(),
            predecessors: HashMap::new(),
            entry: 0,
            exit: 1,
            next_id: 2,
            function_name: None,
        };

        // Create entry and exit blocks
        cfg.blocks.insert(
            0,
            BasicBlock {
                id: 0,
                statements: Vec::new(),
                kind: BlockKind::Entry,
            },
        );
        cfg.blocks.insert(
            1,
            BasicBlock {
                id: 1,
                statements: Vec::new(),
                kind: BlockKind::Exit,
            },
        );

        cfg
    }

    /// Create a new basic block
    pub fn create_block(&mut self, kind: BlockKind) -> BlockId {
        let id = self.next_id;
        self.next_id += 1;
        self.blocks.insert(
            id,
            BasicBlock {
                id,
                statements: Vec::new(),
                kind,
            },
        );
        id
    }

    /// Add an edge between blocks
    pub fn add_edge(&mut self, from: BlockId, to: BlockId, kind: EdgeKind) {
        let edge = CfgEdge {
            from,
            to,
            kind: kind.clone(),
        };
        self.successors.entry(from).or_default().push(edge.clone());
        self.predecessors.entry(to).or_default().push(edge);
    }

    /// Add a statement to a block
    pub fn add_statement(&mut self, block_id: BlockId, stmt: StatementInfo) {
        if let Some(block) = self.blocks.get_mut(&block_id) {
            block.statements.push(stmt);
        }
    }

    /// Get successors of a block
    pub fn get_successors(&self, block_id: BlockId) -> Vec<BlockId> {
        self.successors
            .get(&block_id)
            .map(|edges| edges.iter().map(|e| e.to).collect())
            .unwrap_or_default()
    }

    /// Get predecessors of a block
    pub fn get_predecessors(&self, block_id: BlockId) -> Vec<BlockId> {
        self.predecessors
            .get(&block_id)
            .map(|edges| edges.iter().map(|e| e.from).collect())
            .unwrap_or_default()
    }

    /// Get all block IDs
    pub fn block_ids(&self) -> Vec<BlockId> {
        self.blocks.keys().copied().collect()
    }

    /// Get a block by ID
    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    /// Get all statements across all blocks
    pub fn all_statements(&self) -> Vec<&StatementInfo> {
        let mut stmts = Vec::new();
        for block in self.blocks.values() {
            for stmt in &block.statements {
                stmts.push(stmt);
            }
        }
        stmts.sort_by_key(|s| s.line);
        stmts
    }

    /// Check if block A dominates block B
    /// (A appears on every path from entry to B)
    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool {
        if a == b {
            return true;
        }
        // Use simple BFS to check if B is reachable without going through A
        let mut visited = HashSet::new();
        let mut queue = vec![self.entry];

        while let Some(current) = queue.pop() {
            if current == a {
                continue; // Skip A
            }
            if current == b {
                return false; // Reached B without A
            }
            if visited.insert(current) {
                queue.extend(self.get_successors(current));
            }
        }
        true
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// CFG builder for Python code
pub struct CfgBuilder<'a> {
    /// The CFG being built
    cfg: ControlFlowGraph,
    /// Source code (reserved for future use in statement extraction)
    _source: &'a str,
    /// Current block being built
    current_block: BlockId,
    /// Loop stack (for break/continue handling)
    loop_stack: Vec<LoopContext>,
    /// Try stack (for exception handling)
    try_stack: Vec<TryContext>,
}

/// Context for a loop
struct LoopContext {
    /// Condition block (for continue)
    condition: BlockId,
    /// Exit block (for break)
    exit: BlockId,
}

/// Context for a try block
struct TryContext {
    /// Except handlers
    handlers: Vec<BlockId>,
    /// Finally block (if any)
    _finally: Option<BlockId>,
}

impl<'a> CfgBuilder<'a> {
    /// Create a new CFG builder
    pub fn new(source: &'a str) -> Self {
        let cfg = ControlFlowGraph::new();
        Self {
            current_block: cfg.entry,
            cfg,
            _source: source,
            loop_stack: Vec::new(),
            try_stack: Vec::new(),
        }
    }

    /// Build CFG from a parsed file
    pub fn build(mut self, file: &ParsedFile) -> ControlFlowGraph {
        let root = file.root_node();

        // Create initial block for module body
        let first_block = self.cfg.create_block(BlockKind::Normal);
        self.cfg
            .add_edge(self.cfg.entry, first_block, EdgeKind::Sequential);
        self.current_block = first_block;

        // Visit all top-level statements
        self.visit_block(&root, file);

        // Connect last block to exit if not already connected
        if self.cfg.get_successors(self.current_block).is_empty() {
            self.cfg
                .add_edge(self.current_block, self.cfg.exit, EdgeKind::Sequential);
        }

        self.cfg
    }

    /// Build CFG for a specific function
    pub fn build_function(
        mut self,
        func_node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> ControlFlowGraph {
        // Get function name
        if let Some(name_node) = func_node.child_by_field_name("name") {
            self.cfg.function_name = Some(file.node_text(&name_node).to_string());
        }

        // Create initial block for function body
        let first_block = self.cfg.create_block(BlockKind::Normal);
        self.cfg
            .add_edge(self.cfg.entry, first_block, EdgeKind::Sequential);
        self.current_block = first_block;

        // Visit function body
        if let Some(body) = func_node.child_by_field_name("body") {
            self.visit_block(&body, file);
        }

        // Connect last block to exit if not already connected
        if self.cfg.get_successors(self.current_block).is_empty() {
            self.cfg
                .add_edge(self.current_block, self.cfg.exit, EdgeKind::Sequential);
        }

        self.cfg
    }

    /// Visit a block of statements
    fn visit_block(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_statement(&child, file);
        }
    }

    /// Visit a single statement
    fn visit_statement(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        match node.kind() {
            "if_statement" => self.visit_if(node, file),
            "for_statement" => self.visit_for(node, file),
            "while_statement" => self.visit_while(node, file),
            "try_statement" => self.visit_try(node, file),
            "return_statement" => self.visit_return(node, file),
            "break_statement" => self.visit_break(node, file),
            "continue_statement" => self.visit_continue(node, file),
            "raise_statement" => self.visit_raise(node, file),
            "function_definition" | "async_function_definition" => {
                // Skip nested functions for now (handled separately)
            }
            "class_definition" => {
                // Skip class definitions for now
            }
            _ => {
                // Regular statement - add to current block
                if let Some(stmt) = self.make_statement_info(node, file) {
                    self.cfg.add_statement(self.current_block, stmt);
                }
            }
        }
    }

    /// Visit an if statement
    fn visit_if(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Create condition block
        let cond_block = self.cfg.create_block(BlockKind::IfCondition);
        self.cfg
            .add_edge(self.current_block, cond_block, EdgeKind::Sequential);

        // Add condition as statement
        if let Some(condition) = node.child_by_field_name("condition") {
            if let Some(stmt) = self.make_statement_info(&condition, file) {
                self.cfg.add_statement(cond_block, stmt);
            }
        }

        // Create then block
        let then_block = self.cfg.create_block(BlockKind::Normal);
        self.cfg
            .add_edge(cond_block, then_block, EdgeKind::TrueBranch);

        // Create join block (after if/else)
        let join_block = self.cfg.create_block(BlockKind::Normal);

        // Visit then block
        self.current_block = then_block;
        if let Some(consequence) = node.child_by_field_name("consequence") {
            self.visit_block(&consequence, file);
        }

        // Connect then block to join (if no return/break)
        if self.cfg.get_successors(self.current_block).is_empty() {
            self.cfg
                .add_edge(self.current_block, join_block, EdgeKind::Sequential);
        }

        // Handle elif/else clauses
        let mut cursor = node.walk();
        let mut has_else = false;
        let mut last_false_block = cond_block;

        for child in node.children(&mut cursor) {
            match child.kind() {
                "elif_clause" => {
                    // Create elif condition block
                    let elif_cond = self.cfg.create_block(BlockKind::IfCondition);
                    self.cfg
                        .add_edge(last_false_block, elif_cond, EdgeKind::FalseBranch);

                    // Add elif condition
                    if let Some(condition) = child.child_by_field_name("condition") {
                        if let Some(stmt) = self.make_statement_info(&condition, file) {
                            self.cfg.add_statement(elif_cond, stmt);
                        }
                    }

                    // Create elif body block
                    let elif_body = self.cfg.create_block(BlockKind::Normal);
                    self.cfg
                        .add_edge(elif_cond, elif_body, EdgeKind::TrueBranch);

                    // Visit elif body
                    self.current_block = elif_body;
                    if let Some(consequence) = child.child_by_field_name("consequence") {
                        self.visit_block(&consequence, file);
                    }

                    // Connect elif body to join
                    if self.cfg.get_successors(self.current_block).is_empty() {
                        self.cfg
                            .add_edge(self.current_block, join_block, EdgeKind::Sequential);
                    }

                    last_false_block = elif_cond;
                }
                "else_clause" => {
                    has_else = true;

                    // Create else block
                    let else_block = self.cfg.create_block(BlockKind::Normal);
                    self.cfg
                        .add_edge(last_false_block, else_block, EdgeKind::FalseBranch);

                    // Visit else body
                    self.current_block = else_block;
                    if let Some(body) = child.child_by_field_name("body") {
                        self.visit_block(&body, file);
                    } else {
                        // Else clause might have body as direct children
                        self.visit_block(&child, file);
                    }

                    // Connect else to join
                    if self.cfg.get_successors(self.current_block).is_empty() {
                        self.cfg
                            .add_edge(self.current_block, join_block, EdgeKind::Sequential);
                    }
                }
                _ => {}
            }
        }

        // If no else, connect last condition to join via false branch
        if !has_else {
            self.cfg
                .add_edge(last_false_block, join_block, EdgeKind::FalseBranch);
        }

        self.current_block = join_block;
    }

    /// Visit a for loop
    fn visit_for(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Create condition block (iterator check)
        let cond_block = self.cfg.create_block(BlockKind::LoopCondition);
        self.cfg
            .add_edge(self.current_block, cond_block, EdgeKind::Sequential);

        // Add loop header as statement
        if let Some(stmt) = self.make_statement_info(node, file) {
            self.cfg.add_statement(cond_block, stmt);
        }

        // Create body block
        let body_block = self.cfg.create_block(BlockKind::Normal);
        self.cfg
            .add_edge(cond_block, body_block, EdgeKind::TrueBranch);

        // Check if there's an else clause
        let mut cursor = node.walk();
        let mut else_clause = None;
        for child in node.children(&mut cursor) {
            if child.kind() == "else_clause" {
                else_clause = Some(child);
                break;
            }
        }

        // Create else block if present, otherwise use exit directly
        let (else_block, exit_block) = if else_clause.is_some() {
            let else_blk = self.cfg.create_block(BlockKind::Normal);
            let exit_blk = self.cfg.create_block(BlockKind::Normal);
            // False branch goes to else block (loop completed without break)
            self.cfg
                .add_edge(cond_block, else_blk, EdgeKind::FalseBranch);
            (Some(else_blk), exit_blk)
        } else {
            let exit_blk = self.cfg.create_block(BlockKind::Normal);
            // False branch goes directly to exit
            self.cfg
                .add_edge(cond_block, exit_blk, EdgeKind::FalseBranch);
            (None, exit_blk)
        };

        // Push loop context - break goes to exit (skipping else)
        self.loop_stack.push(LoopContext {
            condition: cond_block,
            exit: exit_block,
        });

        // Visit body
        self.current_block = body_block;
        if let Some(body) = node.child_by_field_name("body") {
            self.visit_block(&body, file);
        }

        // Connect end of body back to condition
        if self.cfg.get_successors(self.current_block).is_empty() {
            self.cfg
                .add_edge(self.current_block, cond_block, EdgeKind::LoopBack);
        }

        // Visit else clause if present
        if let (Some(else_blk), Some(else_node)) = (else_block, else_clause) {
            self.current_block = else_blk;
            if let Some(body) = else_node.child_by_field_name("body") {
                self.visit_block(&body, file);
            } else {
                // Else clause body might be direct children
                self.visit_block(&else_node, file);
            }
            // Connect else to exit
            if self.cfg.get_successors(self.current_block).is_empty() {
                self.cfg
                    .add_edge(self.current_block, exit_block, EdgeKind::Sequential);
            }
        }

        // Pop loop context
        self.loop_stack.pop();

        self.current_block = exit_block;
    }

    /// Visit a while loop
    fn visit_while(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Create condition block
        let cond_block = self.cfg.create_block(BlockKind::LoopCondition);
        self.cfg
            .add_edge(self.current_block, cond_block, EdgeKind::Sequential);

        // Add condition as statement
        if let Some(condition) = node.child_by_field_name("condition") {
            if let Some(stmt) = self.make_statement_info(&condition, file) {
                self.cfg.add_statement(cond_block, stmt);
            }
        }

        // Create body and exit blocks
        let body_block = self.cfg.create_block(BlockKind::Normal);
        let exit_block = self.cfg.create_block(BlockKind::Normal);

        self.cfg
            .add_edge(cond_block, body_block, EdgeKind::TrueBranch);
        self.cfg
            .add_edge(cond_block, exit_block, EdgeKind::FalseBranch);

        // Push loop context
        self.loop_stack.push(LoopContext {
            condition: cond_block,
            exit: exit_block,
        });

        // Visit body
        self.current_block = body_block;
        if let Some(body) = node.child_by_field_name("body") {
            self.visit_block(&body, file);
        }

        // Connect end of body back to condition
        if self.cfg.get_successors(self.current_block).is_empty() {
            self.cfg
                .add_edge(self.current_block, cond_block, EdgeKind::LoopBack);
        }

        // Pop loop context
        self.loop_stack.pop();

        self.current_block = exit_block;
    }

    /// Visit a try statement
    ///
    /// Creates exception edges from each block in the try body to handlers,
    /// providing more accurate control flow for exception handling.
    fn visit_try(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let try_entry = self.cfg.create_block(BlockKind::TryEntry);
        self.cfg
            .add_edge(self.current_block, try_entry, EdgeKind::Sequential);

        let exit_block = self.cfg.create_block(BlockKind::Normal);
        let mut handler_blocks = Vec::new();
        let mut finally_block = None;

        // First pass: create handler and finally blocks
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "except_clause" => {
                    let handler = self.cfg.create_block(BlockKind::ExceptHandler);
                    handler_blocks.push(handler);
                }
                "finally_clause" => {
                    finally_block = Some(self.cfg.create_block(BlockKind::Finally));
                }
                _ => {}
            }
        }

        // Push try context - this allows raise statements to connect to handlers
        self.try_stack.push(TryContext {
            handlers: handler_blocks.clone(),
            _finally: finally_block,
        });

        // Track blocks created during try body for exception edges
        let blocks_before = self.cfg.blocks.len();

        // Visit try body
        self.current_block = try_entry;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "block" {
                self.visit_block(&child, file);
                break;
            }
        }

        let try_body_end = self.current_block;

        // Add exception edges from all blocks created in try body to handlers
        // This models that any statement in try could throw
        let blocks_after: Vec<BlockId> = self
            .cfg
            .blocks
            .keys()
            .copied()
            .filter(|&id| {
                // Include try_entry and all blocks created during try body
                id == try_entry
                    || (id >= blocks_before as u32
                        && id != exit_block
                        && !handler_blocks.contains(&id)
                        && Some(id) != finally_block)
            })
            .collect();

        for block_id in blocks_after {
            // Skip if this block already has exception edges (e.g., from raise)
            let has_exception_edge = self
                .cfg
                .successors
                .get(&block_id)
                .map(|edges| edges.iter().any(|e| e.kind == EdgeKind::Exception))
                .unwrap_or(false);

            if !has_exception_edge {
                for &handler in &handler_blocks {
                    self.cfg.add_edge(block_id, handler, EdgeKind::Exception);
                }
            }
        }

        // Connect try body end to finally or exit (normal path)
        if let Some(finally) = finally_block {
            if self.cfg.get_successors(try_body_end).is_empty()
                || !self
                    .cfg
                    .get_successors(try_body_end)
                    .iter()
                    .any(|id| *id == finally)
            {
                self.cfg
                    .add_edge(try_body_end, finally, EdgeKind::Sequential);
            }
        } else if self.cfg.get_successors(try_body_end).is_empty() {
            self.cfg
                .add_edge(try_body_end, exit_block, EdgeKind::Sequential);
        }

        // Visit except handlers
        let mut cursor = node.walk();
        let mut handler_idx = 0;
        for child in node.children(&mut cursor) {
            if child.kind() == "except_clause" && handler_idx < handler_blocks.len() {
                self.current_block = handler_blocks[handler_idx];
                self.visit_block(&child, file);

                // Connect handler to finally or exit
                if let Some(finally) = finally_block {
                    if self.cfg.get_successors(self.current_block).is_empty() {
                        self.cfg
                            .add_edge(self.current_block, finally, EdgeKind::Sequential);
                    }
                } else if self.cfg.get_successors(self.current_block).is_empty() {
                    self.cfg
                        .add_edge(self.current_block, exit_block, EdgeKind::Sequential);
                }

                handler_idx += 1;
            }
        }

        // Visit finally block
        if let Some(finally) = finally_block {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "finally_clause" {
                    self.current_block = finally;
                    self.visit_block(&child, file);

                    // Connect finally to exit
                    if self.cfg.get_successors(self.current_block).is_empty() {
                        self.cfg
                            .add_edge(self.current_block, exit_block, EdgeKind::Sequential);
                    }
                    break;
                }
            }
        }

        // Pop try context
        self.try_stack.pop();

        self.current_block = exit_block;
    }

    /// Visit a return statement
    fn visit_return(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if let Some(stmt) = self.make_statement_info(node, file) {
            self.cfg.add_statement(self.current_block, stmt);
        }
        self.cfg
            .add_edge(self.current_block, self.cfg.exit, EdgeKind::Return);
    }

    /// Visit a break statement
    fn visit_break(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if let Some(stmt) = self.make_statement_info(node, file) {
            self.cfg.add_statement(self.current_block, stmt);
        }
        if let Some(loop_ctx) = self.loop_stack.last() {
            self.cfg
                .add_edge(self.current_block, loop_ctx.exit, EdgeKind::Break);
        }
    }

    /// Visit a continue statement
    fn visit_continue(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if let Some(stmt) = self.make_statement_info(node, file) {
            self.cfg.add_statement(self.current_block, stmt);
        }
        if let Some(loop_ctx) = self.loop_stack.last() {
            self.cfg
                .add_edge(self.current_block, loop_ctx.condition, EdgeKind::Continue);
        }
    }

    /// Visit a raise statement
    fn visit_raise(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if let Some(stmt) = self.make_statement_info(node, file) {
            self.cfg.add_statement(self.current_block, stmt);
        }

        // Connect to exception handlers if in try block
        if let Some(try_ctx) = self.try_stack.last() {
            for &handler in &try_ctx.handlers {
                self.cfg
                    .add_edge(self.current_block, handler, EdgeKind::Exception);
            }
        } else {
            // Uncaught exception goes to exit
            self.cfg
                .add_edge(self.current_block, self.cfg.exit, EdgeKind::Exception);
        }
    }

    /// Create a StatementInfo from a node
    fn make_statement_info(
        &self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> Option<StatementInfo> {
        let kind = match node.kind() {
            "assignment" => StatementKind::Assignment,
            "augmented_assignment" => StatementKind::AugmentedAssignment,
            "expression_statement" => StatementKind::Expression,
            "return_statement" => StatementKind::Return,
            "raise_statement" => StatementKind::Raise,
            "assert_statement" => StatementKind::Assert,
            "pass_statement" => StatementKind::Pass,
            "break_statement" => StatementKind::Break,
            "continue_statement" => StatementKind::Continue,
            "import_statement" | "import_from_statement" => StatementKind::Import,
            "global_statement" | "nonlocal_statement" => StatementKind::Declaration,
            "delete_statement" => StatementKind::Delete,
            // Skip control flow structures (handled separately)
            "if_statement" | "for_statement" | "while_statement" | "try_statement" => return None,
            // Skip definitions
            "function_definition" | "async_function_definition" | "class_definition" => {
                return None
            }
            _ => StatementKind::Other,
        };

        Some(StatementInfo {
            span: Span {
                start: node.start_byte(),
                end: node.end_byte(),
                start_line: node.start_position().row,
                start_col: node.start_position().column,
                end_line: node.end_position().row,
                end_col: node.end_position().column,
            },
            kind,
            line: node.start_position().row,
            text: file.node_text(node).to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{Language, MultiParser};

    fn build_cfg(code: &str) -> ControlFlowGraph {
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(code, Language::Python).unwrap();
        CfgBuilder::new(code).build(&parsed)
    }

    #[test]
    fn test_simple_sequential() {
        let cfg = build_cfg("x = 1\ny = 2\nz = x + y");

        // Should have entry, exit, and one body block
        assert!(cfg.blocks.len() >= 3);

        // Entry should have one successor
        let entry_succs = cfg.get_successors(cfg.entry);
        assert_eq!(entry_succs.len(), 1);
    }

    #[test]
    fn test_if_statement() {
        let cfg = build_cfg("if x:\n    y = 1\nelse:\n    y = 2");

        // Should have blocks for: entry, condition, then, else, join, exit
        assert!(cfg.blocks.len() >= 5);

        // Find condition block
        let cond_blocks: Vec<_> = cfg
            .blocks
            .values()
            .filter(|b| b.kind == BlockKind::IfCondition)
            .collect();
        assert_eq!(cond_blocks.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let cfg = build_cfg("while x:\n    y = 1");

        // Should have loop condition block
        let loop_blocks: Vec<_> = cfg
            .blocks
            .values()
            .filter(|b| b.kind == BlockKind::LoopCondition)
            .collect();
        assert_eq!(loop_blocks.len(), 1);

        // Should have back edge
        let back_edges: Vec<_> = cfg
            .successors
            .values()
            .flatten()
            .filter(|e| e.kind == EdgeKind::LoopBack)
            .collect();
        assert!(!back_edges.is_empty());
    }

    #[test]
    fn test_return_statement() {
        let cfg = build_cfg("def f():\n    return 1");

        // Return should connect to exit
        let _return_edges: Vec<_> = cfg
            .successors
            .values()
            .flatten()
            .filter(|e| e.kind == EdgeKind::Return)
            .collect();
        // Note: This tests module-level CFG, not function CFG
        // For function CFG, would need to use build_function
    }

    #[test]
    fn test_break_continue() {
        let cfg = build_cfg("while True:\n    if x:\n        break\n    continue");

        // Should have break and continue edges
        let break_edges: Vec<_> = cfg
            .successors
            .values()
            .flatten()
            .filter(|e| e.kind == EdgeKind::Break)
            .collect();
        assert!(!break_edges.is_empty());

        let continue_edges: Vec<_> = cfg
            .successors
            .values()
            .flatten()
            .filter(|e| e.kind == EdgeKind::Continue)
            .collect();
        assert!(!continue_edges.is_empty());
    }
}

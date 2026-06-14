//! Mutable AST with persistent data structures (Sprint 2 - Track 2)
//!
//! Provides a mutable AST implementation for efficient code transformations:
//! - Copy-on-write node modifications
//! - Persistent immutable snapshots
//! - Efficient tree diffing
//! - Undo/redo support

use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Node ID and References
// ============================================================================

/// Unique identifier for AST nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// A reference to a node that may be borrowed or owned.
#[derive(Debug, Clone)]
pub enum NodeRef {
    /// Direct reference by ID
    Direct(NodeId),
    /// Path from root
    Path(Vec<usize>),
}

// ============================================================================
// Mutable AST Node
// ============================================================================

/// A mutable AST node with copy-on-write semantics.
#[derive(Debug, Clone)]
pub struct MutableNode {
    /// Unique node ID
    pub id: NodeId,
    /// Node kind (e.g., "function_definition", "assignment")
    pub kind: String,
    /// Text span in source
    pub span: Span,
    /// Node value/text (for leaf nodes)
    pub value: Option<String>,
    /// Child nodes (copy-on-write)
    pub children: Arc<Vec<MutableNode>>,
    /// Node metadata
    pub metadata: NodeMetadata,
}

/// Span in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Start line (0-indexed)
    pub start_line: usize,
    /// Start column (0-indexed)
    pub start_col: usize,
    /// End line
    pub end_line: usize,
    /// End column
    pub end_col: usize,
}

impl Span {
    /// Create a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        }
    }

    /// Create with full position info.
    pub fn with_lines(
        start: usize,
        end: usize,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Self {
            start,
            end,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Check if spans overlap.
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Check if this span contains another.
    pub fn contains(&self, other: &Span) -> bool {
        self.start <= other.start && self.end >= other.end
    }
}

/// Metadata attached to nodes.
#[derive(Debug, Clone, Default)]
pub struct NodeMetadata {
    /// Type annotation (if any)
    pub type_annotation: Option<String>,
    /// Documentation comment
    pub docstring: Option<String>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

impl MutableNode {
    /// Create a new mutable node.
    pub fn new(id: NodeId, kind: impl Into<String>, span: Span) -> Self {
        Self {
            id,
            kind: kind.into(),
            span,
            value: None,
            children: Arc::new(Vec::new()),
            metadata: NodeMetadata::default(),
        }
    }

    /// Create a leaf node with value.
    pub fn leaf(id: NodeId, kind: impl Into<String>, span: Span, value: impl Into<String>) -> Self {
        Self {
            id,
            kind: kind.into(),
            span,
            value: Some(value.into()),
            children: Arc::new(Vec::new()),
            metadata: NodeMetadata::default(),
        }
    }

    /// Add a child node (copy-on-write).
    pub fn add_child(&mut self, child: MutableNode) {
        Arc::make_mut(&mut self.children).push(child);
    }

    /// Get child at index.
    pub fn child(&self, index: usize) -> Option<&MutableNode> {
        self.children.get(index)
    }

    /// Get mutable child at index (copy-on-write).
    pub fn child_mut(&mut self, index: usize) -> Option<&mut MutableNode> {
        Arc::make_mut(&mut self.children).get_mut(index)
    }

    /// Find child by kind.
    pub fn find_child(&self, kind: &str) -> Option<&MutableNode> {
        self.children.iter().find(|c| c.kind == kind)
    }

    /// Replace a child at index.
    pub fn replace_child(&mut self, index: usize, new_child: MutableNode) -> Option<MutableNode> {
        let children = Arc::make_mut(&mut self.children);
        if index < children.len() {
            Some(std::mem::replace(&mut children[index], new_child))
        } else {
            None
        }
    }

    /// Remove a child at index.
    pub fn remove_child(&mut self, index: usize) -> Option<MutableNode> {
        let children = Arc::make_mut(&mut self.children);
        if index < children.len() {
            Some(children.remove(index))
        } else {
            None
        }
    }

    /// Insert a child at index.
    pub fn insert_child(&mut self, index: usize, child: MutableNode) {
        let children = Arc::make_mut(&mut self.children);
        if index <= children.len() {
            children.insert(index, child);
        }
    }

    /// Traverse the tree depth-first.
    pub fn traverse<F>(&self, mut f: F)
    where
        F: FnMut(&MutableNode),
    {
        self.traverse_impl(&mut f);
    }

    fn traverse_impl<F>(&self, f: &mut F)
    where
        F: FnMut(&MutableNode),
    {
        f(self);
        for child in self.children.iter() {
            child.traverse_impl(f);
        }
    }

    /// Find a node by ID.
    pub fn find_by_id(&self, target: NodeId) -> Option<&MutableNode> {
        if self.id == target {
            return Some(self);
        }
        for child in self.children.iter() {
            if let Some(found) = child.find_by_id(target) {
                return Some(found);
            }
        }
        None
    }

    /// Find a node at a specific span.
    pub fn find_at_span(&self, span: &Span) -> Option<&MutableNode> {
        if self.span.start == span.start && self.span.end == span.end {
            return Some(self);
        }
        for child in self.children.iter() {
            if let Some(found) = child.find_at_span(span) {
                return Some(found);
            }
        }
        None
    }

    /// Collect all nodes of a specific kind.
    pub fn collect_by_kind<'a>(&'a self, kind: &str, results: &mut Vec<&'a MutableNode>) {
        if self.kind == kind {
            results.push(self);
        }
        for child in self.children.iter() {
            child.collect_by_kind(kind, results);
        }
    }

    /// Find the innermost node containing a position.
    pub fn find_at_position(&self, line: usize, col: usize) -> Option<&MutableNode> {
        // Check if position is within this node's span
        if line < self.span.start_line || line > self.span.end_line {
            return None;
        }
        if line == self.span.start_line && col < self.span.start_col {
            return None;
        }
        if line == self.span.end_line && col > self.span.end_col {
            return None;
        }

        // Check children for a more specific match
        for child in self.children.iter() {
            if let Some(found) = child.find_at_position(line, col) {
                return Some(found);
            }
        }

        // This is the innermost node containing the position
        Some(self)
    }
}

// ============================================================================
// Mutable AST Tree
// ============================================================================

/// A mutable AST with snapshot support.
pub struct MutableAst {
    /// Root node
    root: MutableNode,
    /// Next node ID
    next_id: usize,
    /// Snapshots for undo
    snapshots: Vec<MutableNode>,
    /// Current snapshot index
    snapshot_index: usize,
    /// Maximum snapshots to keep
    max_snapshots: usize,
}

impl MutableAst {
    /// Create a new mutable AST.
    pub fn new(root: MutableNode) -> Self {
        Self {
            root,
            next_id: 1,
            snapshots: Vec::new(),
            snapshot_index: 0,
            max_snapshots: 100,
        }
    }

    /// Get the root node.
    pub fn root(&self) -> &MutableNode {
        &self.root
    }

    /// Get mutable root node.
    pub fn root_mut(&mut self) -> &mut MutableNode {
        &mut self.root
    }

    /// Generate a new node ID.
    pub fn new_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Take a snapshot for undo support.
    pub fn snapshot(&mut self) {
        // Truncate any redo history
        self.snapshots.truncate(self.snapshot_index);

        // Add new snapshot
        self.snapshots.push(self.root.clone());
        self.snapshot_index = self.snapshots.len();

        // Limit history size
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
            self.snapshot_index = self.snapshots.len();
        }
    }

    /// Undo to previous snapshot.
    pub fn undo(&mut self) -> bool {
        if self.snapshot_index > 0 {
            // Save current state for redo
            if self.snapshot_index == self.snapshots.len() {
                self.snapshots.push(self.root.clone());
            }
            self.snapshot_index -= 1;
            self.root = self.snapshots[self.snapshot_index].clone();
            true
        } else {
            false
        }
    }

    /// Redo to next snapshot.
    pub fn redo(&mut self) -> bool {
        if self.snapshot_index < self.snapshots.len() - 1 {
            self.snapshot_index += 1;
            self.root = self.snapshots[self.snapshot_index].clone();
            true
        } else {
            false
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        self.snapshot_index > 0
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        self.snapshot_index < self.snapshots.len().saturating_sub(1)
    }

    /// Find a node by ID.
    pub fn find_node(&self, id: NodeId) -> Option<&MutableNode> {
        self.root.find_by_id(id)
    }

    /// Apply an edit operation.
    pub fn apply_edit(&mut self, edit: AstEdit) -> bool {
        match edit {
            AstEdit::Replace { target, new_node } => self.replace_node(target, new_node),
            AstEdit::Insert {
                parent,
                index,
                node,
            } => self.insert_node(parent, index, node),
            AstEdit::Remove { target } => self.remove_node(target),
            AstEdit::UpdateValue { target, value } => self.update_value(target, value),
        }
    }

    fn replace_node(&mut self, target: NodeId, new_node: MutableNode) -> bool {
        self.replace_in_subtree(&mut self.root.clone(), target, new_node)
            .map(|new_root| self.root = new_root)
            .is_some()
    }

    fn replace_in_subtree(
        &self,
        node: &mut MutableNode,
        target: NodeId,
        new_node: MutableNode,
    ) -> Option<MutableNode> {
        if node.id == target {
            return Some(new_node);
        }

        let children = Arc::make_mut(&mut node.children);
        for i in 0..children.len() {
            if children[i].id == target {
                children[i] = new_node;
                return Some(node.clone());
            }
            if let Some(updated) =
                self.replace_in_subtree(&mut children[i].clone(), target, new_node.clone())
            {
                children[i] = updated;
                return Some(node.clone());
            }
        }
        None
    }

    fn insert_node(&mut self, parent: NodeId, index: usize, node: MutableNode) -> bool {
        if let Some(parent_node) = self.find_node_mut(parent) {
            parent_node.insert_child(index, node);
            true
        } else {
            false
        }
    }

    fn remove_node(&mut self, target: NodeId) -> bool {
        self.remove_from_subtree(&mut self.root.clone(), target)
            .map(|new_root| self.root = new_root)
            .is_some()
    }

    fn remove_from_subtree(&self, node: &mut MutableNode, target: NodeId) -> Option<MutableNode> {
        let children = Arc::make_mut(&mut node.children);
        for i in 0..children.len() {
            if children[i].id == target {
                children.remove(i);
                return Some(node.clone());
            }
            if let Some(updated) = self.remove_from_subtree(&mut children[i].clone(), target) {
                children[i] = updated;
                return Some(node.clone());
            }
        }
        None
    }

    fn update_value(&mut self, target: NodeId, value: String) -> bool {
        if let Some(node) = self.find_node_mut(target) {
            node.value = Some(value);
            true
        } else {
            false
        }
    }

    fn find_node_mut(&mut self, id: NodeId) -> Option<&mut MutableNode> {
        Self::find_in_subtree_mut(&mut self.root, id)
    }

    fn find_in_subtree_mut(node: &mut MutableNode, target: NodeId) -> Option<&mut MutableNode> {
        if node.id == target {
            return Some(node);
        }
        let children = Arc::make_mut(&mut node.children);
        for child in children.iter_mut() {
            if let Some(found) = Self::find_in_subtree_mut(child, target) {
                return Some(found);
            }
        }
        None
    }

    /// Find a node at a specific span.
    pub fn find_at_span(&self, span: &Span) -> Option<&MutableNode> {
        self.root.find_at_span(span)
    }

    /// Find all nodes of a specific kind.
    pub fn find_by_kind(&self, kind: &str) -> Vec<&MutableNode> {
        let mut results = Vec::new();
        self.root.collect_by_kind(kind, &mut results);
        results
    }

    /// Find the innermost node containing a position.
    pub fn find_at_position(&self, line: usize, col: usize) -> Option<&MutableNode> {
        self.root.find_at_position(line, col)
    }
}

/// An edit operation on the AST.
#[derive(Debug, Clone)]
pub enum AstEdit {
    /// Replace a node with a new one
    Replace {
        target: NodeId,
        new_node: MutableNode,
    },
    /// Insert a new child node
    Insert {
        parent: NodeId,
        index: usize,
        node: MutableNode,
    },
    /// Remove a node
    Remove { target: NodeId },
    /// Update a leaf node's value
    UpdateValue { target: NodeId, value: String },
}

// ============================================================================
// Tree Diff
// ============================================================================

/// A diff between two AST trees.
#[derive(Debug, Clone)]
pub struct TreeDiff {
    /// Edit operations to transform old tree to new tree
    pub edits: Vec<AstEdit>,
}

impl TreeDiff {
    /// Compute diff between two trees.
    pub fn compute(old: &MutableNode, new: &MutableNode) -> Self {
        let mut edits = Vec::new();
        Self::compute_diff(old, new, &mut edits);
        Self { edits }
    }

    fn compute_diff(old: &MutableNode, new: &MutableNode, edits: &mut Vec<AstEdit>) {
        // If nodes are different types or values, replace
        if old.kind != new.kind || old.value != new.value {
            edits.push(AstEdit::Replace {
                target: old.id,
                new_node: new.clone(),
            });
            return;
        }

        // Compare children
        let old_len = old.children.len();
        let new_len = new.children.len();

        // Compare existing children
        let min_len = old_len.min(new_len);
        for i in 0..min_len {
            Self::compute_diff(&old.children[i], &new.children[i], edits);
        }

        // Handle removed children
        for i in (new_len..old_len).rev() {
            edits.push(AstEdit::Remove {
                target: old.children[i].id,
            });
        }

        // Handle added children
        for i in old_len..new_len {
            edits.push(AstEdit::Insert {
                parent: old.id,
                index: i,
                node: new.children[i].clone(),
            });
        }
    }

    /// Check if diff is empty.
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    /// Get number of edits.
    pub fn len(&self) -> usize {
        self.edits.len()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutable_node() {
        let mut root = MutableNode::new(NodeId(0), "module", Span::new(0, 100));
        let child = MutableNode::leaf(NodeId(1), "identifier", Span::new(0, 5), "foo");

        root.add_child(child);

        assert_eq!(root.children.len(), 1);
        assert_eq!(root.child(0).unwrap().kind, "identifier");
    }

    #[test]
    fn test_copy_on_write() {
        let root = MutableNode::new(NodeId(0), "module", Span::new(0, 100));
        let mut root2 = root.clone();

        // Before modification, they share the same Arc
        assert!(Arc::ptr_eq(&root.children, &root2.children));

        // After modification, they don't
        root2.add_child(MutableNode::leaf(
            NodeId(1),
            "identifier",
            Span::new(0, 5),
            "foo",
        ));
        assert!(!Arc::ptr_eq(&root.children, &root2.children));
    }

    #[test]
    fn test_undo_redo() {
        let root = MutableNode::new(NodeId(0), "module", Span::new(0, 100));
        let mut ast = MutableAst::new(root);

        // Take snapshot
        ast.snapshot();

        // Make change
        ast.root_mut().add_child(MutableNode::leaf(
            NodeId(1),
            "identifier",
            Span::new(0, 5),
            "foo",
        ));
        assert_eq!(ast.root().children.len(), 1);

        // Undo
        assert!(ast.undo());
        assert_eq!(ast.root().children.len(), 0);

        // Redo
        assert!(ast.redo());
        assert_eq!(ast.root().children.len(), 1);
    }

    #[test]
    fn test_tree_diff() {
        let old = MutableNode::new(NodeId(0), "module", Span::new(0, 100));
        let mut new = MutableNode::new(NodeId(0), "module", Span::new(0, 100));
        new.add_child(MutableNode::leaf(
            NodeId(1),
            "identifier",
            Span::new(0, 5),
            "foo",
        ));

        let diff = TreeDiff::compute(&old, &new);
        assert_eq!(diff.len(), 1);
    }
}

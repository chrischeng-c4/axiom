//! DOM node types.

use std::collections::HashMap;

/// Node ID for internal reference.
pub type NodeId = usize;

/// DOM node types.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    /// Document root.
    Document,
    /// Element node (tag).
    Element,
    /// Text content.
    Text,
    /// Comment node.
    Comment,
    /// CDATA section (XML).
    CData,
    /// Processing instruction.
    ProcessingInstruction,
    /// DOCTYPE declaration.
    DocType,
}

/// A DOM node.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique node ID.
    pub id: NodeId,
    /// Node type.
    pub node_type: NodeType,
    /// Tag name (for elements).
    pub tag_name: Option<String>,
    /// Namespace URI (for XML).
    pub namespace: Option<String>,
    /// Namespace prefix (for XML).
    pub prefix: Option<String>,
    /// Attributes (for elements).
    pub attributes: HashMap<String, String>,
    /// Text content (for text/comment/cdata nodes).
    pub text: Option<String>,
    /// Parent node ID.
    pub parent: Option<NodeId>,
    /// Child node IDs.
    pub children: Vec<NodeId>,
}

impl Node {
    /// Create a new document node.
    pub fn document(id: NodeId) -> Self {
        Self {
            id,
            node_type: NodeType::Document,
            tag_name: None,
            namespace: None,
            prefix: None,
            attributes: HashMap::new(),
            text: None,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Create a new element node.
    pub fn element(id: NodeId, tag_name: impl Into<String>) -> Self {
        Self {
            id,
            node_type: NodeType::Element,
            tag_name: Some(tag_name.into()),
            namespace: None,
            prefix: None,
            attributes: HashMap::new(),
            text: None,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Create a new text node.
    pub fn text(id: NodeId, content: impl Into<String>) -> Self {
        Self {
            id,
            node_type: NodeType::Text,
            tag_name: None,
            namespace: None,
            prefix: None,
            attributes: HashMap::new(),
            text: Some(content.into()),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Create a new comment node.
    pub fn comment(id: NodeId, content: impl Into<String>) -> Self {
        Self {
            id,
            node_type: NodeType::Comment,
            tag_name: None,
            namespace: None,
            prefix: None,
            attributes: HashMap::new(),
            text: Some(content.into()),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Check if this is an element.
    pub fn is_element(&self) -> bool {
        self.node_type == NodeType::Element
    }

    /// Check if this is a text node.
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::Text
    }

    /// Get the tag name (lowercase).
    pub fn tag(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }

    /// Get an attribute value.
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    /// Set an attribute.
    pub fn set_attr(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(name.into(), value.into());
    }

    /// Remove an attribute.
    pub fn remove_attr(&mut self, name: &str) -> Option<String> {
        self.attributes.remove(name)
    }

    /// Check if element has a class.
    pub fn has_class(&self, class: &str) -> bool {
        self.get_attr("class")
            .map(|c| c.split_whitespace().any(|s| s == class))
            .unwrap_or(false)
    }

    /// Get element ID.
    pub fn get_id(&self) -> Option<&str> {
        self.get_attr("id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_node() {
        let mut node = Node::element(1, "div");
        node.set_attr("class", "container active");
        node.set_attr("id", "main");

        assert!(node.is_element());
        assert_eq!(node.tag(), Some("div"));
        assert!(node.has_class("container"));
        assert!(node.has_class("active"));
        assert!(!node.has_class("foo"));
        assert_eq!(node.get_id(), Some("main"));
    }

    #[test]
    fn test_text_node() {
        let node = Node::text(2, "Hello, World!");
        assert!(node.is_text());
        assert_eq!(node.text.as_deref(), Some("Hello, World!"));
    }

    #[test]
    fn test_remove_attr() {
        let mut node = Node::element(1, "div");
        node.set_attr("class", "container");
        node.set_attr("id", "main");

        assert_eq!(node.get_attr("class"), Some("container"));
        let removed = node.remove_attr("class");
        assert_eq!(removed, Some("container".to_string()));
        assert_eq!(node.get_attr("class"), None);
        assert_eq!(node.get_attr("id"), Some("main"));

        // Remove non-existent
        let not_found = node.remove_attr("nonexistent");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_comment_node() {
        let node = Node::comment(3, "This is a comment");
        assert!(!node.is_element());
        assert!(!node.is_text());
        assert_eq!(node.text.as_deref(), Some("This is a comment"));
    }

    #[test]
    fn test_document_node() {
        let node = Node::document(0);
        assert!(!node.is_element());
        assert!(!node.is_text());
        assert_eq!(node.tag(), None);
    }
}

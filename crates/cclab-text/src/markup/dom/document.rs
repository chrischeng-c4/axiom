//! DOM Document - manages the node tree.

use super::node::{Node, NodeId, NodeType};
use crate::markup::error::{MarkupError, Result};

/// A DOM document containing a tree of nodes.
#[derive(Debug)]
pub struct Document {
    /// All nodes in the document.
    nodes: Vec<Node>,
    /// Root node ID (always 0).
    root: NodeId,
}

impl Document {
    /// Create a new empty document.
    pub fn new() -> Self {
        let root = Node::document(0);
        Self {
            nodes: vec![root],
            root: 0,
        }
    }

    /// Get the root node ID.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Get a node by ID.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Create a new element and return its ID.
    pub fn create_element(&mut self, tag_name: impl Into<String>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node::element(id, tag_name));
        id
    }

    /// Create a new text node and return its ID.
    pub fn create_text(&mut self, content: impl Into<String>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node::text(id, content));
        id
    }

    /// Create a new comment node and return its ID.
    pub fn create_comment(&mut self, content: impl Into<String>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node::comment(id, content));
        id
    }

    /// Append a child node to a parent.
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<()> {
        // Set parent reference
        if let Some(child) = self.nodes.get_mut(child_id) {
            child.parent = Some(parent_id);
        } else {
            return Err(MarkupError::NodeNotFound);
        }

        // Add to parent's children
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(child_id);
            Ok(())
        } else {
            Err(MarkupError::NodeNotFound)
        }
    }

    /// Insert a child before another node.
    pub fn insert_before(
        &mut self,
        parent_id: NodeId,
        new_id: NodeId,
        ref_id: NodeId,
    ) -> Result<()> {
        // Set parent reference
        if let Some(child) = self.nodes.get_mut(new_id) {
            child.parent = Some(parent_id);
        } else {
            return Err(MarkupError::NodeNotFound);
        }

        // Find position and insert
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            if let Some(pos) = parent.children.iter().position(|&id| id == ref_id) {
                parent.children.insert(pos, new_id);
                Ok(())
            } else {
                Err(MarkupError::NodeNotFound)
            }
        } else {
            Err(MarkupError::NodeNotFound)
        }
    }

    /// Remove a node from its parent.
    pub fn remove(&mut self, node_id: NodeId) -> Result<()> {
        let parent_id = self
            .nodes
            .get(node_id)
            .and_then(|n| n.parent)
            .ok_or(MarkupError::NodeNotFound)?;

        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.retain(|&id| id != node_id);
        }

        if let Some(node) = self.nodes.get_mut(node_id) {
            node.parent = None;
        }

        Ok(())
    }

    /// Get children of a node.
    pub fn children(&self, node_id: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(node_id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    /// Get parent of a node.
    pub fn parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes.get(node_id).and_then(|n| n.parent)
    }

    /// Get siblings of a node.
    pub fn siblings(&self, node_id: NodeId) -> Vec<NodeId> {
        self.parent(node_id)
            .map(|pid| {
                self.children(pid)
                    .into_iter()
                    .filter(|&id| id != node_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all descendants of a node (depth-first).
    pub fn descendants(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut stack = self.children(node_id);
        stack.reverse();

        while let Some(id) = stack.pop() {
            result.push(id);
            let children = self.children(id);
            for child in children.into_iter().rev() {
                stack.push(child);
            }
        }

        result
    }

    /// Get all ancestors of a node.
    pub fn ancestors(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut current = self.parent(node_id);

        while let Some(id) = current {
            result.push(id);
            current = self.parent(id);
        }

        result
    }

    /// Collect namespace declarations from ancestors.
    ///
    /// Returns a map of xmlns attributes from all ancestors, with closest
    /// ancestor taking precedence (innermost scope wins).
    fn collect_ancestor_namespaces(
        &self,
        node_id: NodeId,
    ) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;
        let mut ns_map: HashMap<String, String> = HashMap::new();
        let ancestors = self.ancestors(node_id);

        // Process from root to node (reverse order) so inner scopes override outer
        for ancestor_id in ancestors.into_iter().rev() {
            if let Some(ancestor) = self.get(ancestor_id) {
                for (key, value) in &ancestor.attributes {
                    if key == "xmlns" || key.starts_with("xmlns:") {
                        ns_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        ns_map
    }

    /// Find all elements with a given tag name.
    pub fn find_by_tag(&self, tag: &str) -> Vec<NodeId> {
        let tag_lower = tag.to_lowercase();
        self.nodes
            .iter()
            .filter(|n| {
                n.is_element()
                    && n.tag()
                        .map(|t| t.to_lowercase() == tag_lower)
                        .unwrap_or(false)
            })
            .map(|n| n.id)
            .collect()
    }

    /// Find element by ID attribute.
    pub fn find_by_id(&self, id: &str) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|n| n.is_element() && n.get_id() == Some(id))
            .map(|n| n.id)
    }

    /// Find all elements with a given class.
    pub fn find_by_class(&self, class: &str) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.is_element() && n.has_class(class))
            .map(|n| n.id)
            .collect()
    }

    /// Find all elements with a given namespace URI and local name.
    ///
    /// This is namespace-aware lookup that matches elements regardless of prefix.
    ///
    /// # Example
    ///
    /// ```
    /// use cclab_text::markup::xml::parse_xml;
    ///
    /// let xml = r#"<a:root xmlns:a="http://example.com"><a:item>A</a:item></a:root>"#;
    /// let doc = parse_xml(xml);
    ///
    /// // Find by namespace URI, not prefix
    /// let items = doc.find_by_tag_ns("http://example.com", "item");
    /// assert_eq!(items.len(), 1);
    /// ```
    pub fn find_by_tag_ns(&self, namespace_uri: &str, local_name: &str) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| {
                n.is_element()
                    && n.namespace.as_deref() == Some(namespace_uri)
                    && n.tag().map(|t| t == local_name).unwrap_or(false)
            })
            .map(|n| n.id)
            .collect()
    }

    /// Find all elements with a given namespace URI (any local name).
    pub fn find_by_namespace(&self, namespace_uri: &str) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.is_element() && n.namespace.as_deref() == Some(namespace_uri))
            .map(|n| n.id)
            .collect()
    }

    /// Get text content of a node and its descendants.
    pub fn text_content(&self, node_id: NodeId) -> String {
        let mut result = String::new();

        if let Some(node) = self.get(node_id) {
            if node.is_text() {
                if let Some(text) = &node.text {
                    result.push_str(text);
                }
            } else {
                for child_id in &node.children {
                    result.push_str(&self.text_content(*child_id));
                }
            }
        }

        result
    }

    /// Get inner HTML of a node.
    pub fn inner_html(&self, node_id: NodeId) -> String {
        let mut result = String::new();

        if let Some(node) = self.get(node_id) {
            for &child_id in &node.children {
                result.push_str(&self.outer_html(child_id));
            }
        }

        result
    }

    /// Get outer HTML of a node.
    pub fn outer_html(&self, node_id: NodeId) -> String {
        let Some(node) = self.get(node_id) else {
            return String::new();
        };

        match node.node_type {
            NodeType::Text => node.text.clone().unwrap_or_default(),
            NodeType::Comment => format!("<!--{}-->", node.text.as_deref().unwrap_or("")),
            NodeType::Element => {
                let tag = node.tag().unwrap_or("unknown");
                let mut attrs = String::new();

                for (key, value) in &node.attributes {
                    attrs.push_str(&format!(r#" {}="{}""#, key, escape_attr(value)));
                }

                let inner = self.inner_html(node_id);

                if is_void_element(tag) && inner.is_empty() {
                    format!("<{}{} />", tag, attrs)
                } else {
                    format!("<{}{}>{}</{}>", tag, attrs, inner, tag)
                }
            }
            NodeType::Document => self.inner_html(node_id),
            _ => String::new(),
        }
    }

    /// Get outer XML of a node, preserving namespaces.
    ///
    /// This produces well-formed XML with namespace prefixes.
    /// When serializing a non-root node, ancestor namespace declarations
    /// are included to ensure the output is valid standalone XML.
    pub fn outer_xml(&self, node_id: NodeId) -> String {
        self.outer_xml_with_inherited_ns(node_id, true)
    }

    /// Internal method to serialize XML with optional namespace inheritance.
    fn outer_xml_with_inherited_ns(&self, node_id: NodeId, include_ancestor_ns: bool) -> String {
        let Some(node) = self.get(node_id) else {
            return String::new();
        };

        match node.node_type {
            NodeType::Text => escape_xml_text(node.text.as_deref().unwrap_or("")),
            NodeType::Comment => format!("<!--{}-->", node.text.as_deref().unwrap_or("")),
            NodeType::Element => {
                let local_name = node.tag().unwrap_or("unknown");

                // Build qualified name with prefix
                let qname = match &node.prefix {
                    Some(prefix) => format!("{}:{}", prefix, local_name),
                    None => local_name.to_string(),
                };

                let mut attrs = String::new();

                // Include inherited xmlns declarations if this is the root of serialization
                if include_ancestor_ns {
                    let inherited_ns = self.collect_ancestor_namespaces(node_id);
                    for (key, value) in &inherited_ns {
                        // Only include if not already on this node
                        if !node.attributes.contains_key(key) {
                            attrs.push_str(&format!(r#" {}="{}""#, key, escape_attr(value)));
                        }
                    }
                }

                // Include this node's attributes
                for (key, value) in &node.attributes {
                    attrs.push_str(&format!(r#" {}="{}""#, key, escape_attr(value)));
                }

                // Serialize children without including ancestor namespaces (already included)
                let mut inner = String::new();
                for &child_id in &node.children {
                    inner.push_str(&self.outer_xml_with_inherited_ns(child_id, false));
                }

                if inner.is_empty() {
                    format!("<{}{}/>", qname, attrs)
                } else {
                    format!("<{}{}>{}</{}>", qname, attrs, inner, qname)
                }
            }
            NodeType::Document => self.inner_xml(node_id),
            _ => String::new(),
        }
    }

    /// Get inner XML of a node.
    pub fn inner_xml(&self, node_id: NodeId) -> String {
        let mut result = String::new();

        if let Some(node) = self.get(node_id) {
            for &child_id in &node.children {
                result.push_str(&self.outer_xml(child_id));
            }
        }

        result
    }

    /// Serialize document to XML string with optional indentation.
    pub fn to_xml(&self, pretty: bool) -> String {
        if pretty {
            self.to_xml_pretty(self.root, 0)
        } else {
            self.inner_xml(self.root)
        }
    }

    fn to_xml_pretty(&self, node_id: NodeId, depth: usize) -> String {
        let Some(node) = self.get(node_id) else {
            return String::new();
        };

        let indent = "  ".repeat(depth);

        match node.node_type {
            NodeType::Text => {
                let text = node.text.as_deref().unwrap_or("");
                if text.trim().is_empty() {
                    String::new()
                } else {
                    escape_xml_text(text)
                }
            }
            NodeType::Comment => {
                format!("{}<!--{}-->\n", indent, node.text.as_deref().unwrap_or(""))
            }
            NodeType::Element => {
                let local_name = node.tag().unwrap_or("unknown");
                let qname = match &node.prefix {
                    Some(prefix) => format!("{}:{}", prefix, local_name),
                    None => local_name.to_string(),
                };

                let mut attrs = String::new();
                for (key, value) in &node.attributes {
                    attrs.push_str(&format!(r#" {}="{}""#, key, escape_attr(value)));
                }

                // Check if has only text content
                let has_only_text = node.children.len() == 1
                    && self
                        .get(node.children[0])
                        .map(|n| n.is_text())
                        .unwrap_or(false);

                if node.children.is_empty() {
                    format!("{}<{}{}/>\n", indent, qname, attrs)
                } else if has_only_text {
                    let text = self.text_content(node_id);
                    format!(
                        "{}<{}{}>{}</{}>\n",
                        indent,
                        qname,
                        attrs,
                        escape_xml_text(&text),
                        qname
                    )
                } else {
                    let mut result = format!("{}<{}{}>\n", indent, qname, attrs);
                    for &child_id in &node.children {
                        result.push_str(&self.to_xml_pretty(child_id, depth + 1));
                    }
                    result.push_str(&format!("{}</{}>\n", indent, qname));
                    result
                }
            }
            NodeType::Document => {
                let mut result = String::new();
                for &child_id in &node.children {
                    result.push_str(&self.to_xml_pretty(child_id, depth));
                }
                result
            }
            _ => String::new(),
        }
    }

    /// Get number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if document is empty (only root).
    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 1
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape HTML attribute value.
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape XML text content.
fn escape_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Check if an HTML tag is void (self-closing).
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert_eq!(doc.len(), 1);
        assert!(doc.get(0).is_some());
    }

    #[test]
    fn test_append_child() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let text = doc.create_text("Hello");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, text).unwrap();

        assert_eq!(doc.children(doc.root()), vec![div]);
        assert_eq!(doc.children(div), vec![text]);
        assert_eq!(doc.parent(text), Some(div));
    }

    #[test]
    fn test_text_content() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        let t1 = doc.create_text("Hello, ");
        let t2 = doc.create_text("World!");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, t1).unwrap();
        doc.append_child(div, span).unwrap();
        doc.append_child(span, t2).unwrap();

        assert_eq!(doc.text_content(div), "Hello, World!");
    }

    #[test]
    fn test_find_by_tag() {
        let mut doc = Document::new();
        let div1 = doc.create_element("div");
        let div2 = doc.create_element("div");
        let span = doc.create_element("span");

        doc.append_child(doc.root(), div1).unwrap();
        doc.append_child(div1, div2).unwrap();
        doc.append_child(div1, span).unwrap();

        let divs = doc.find_by_tag("div");
        assert_eq!(divs.len(), 2);
    }

    #[test]
    fn test_outer_html() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.get_mut(div).unwrap().set_attr("class", "container");
        let text = doc.create_text("Hello");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, text).unwrap();

        let html = doc.outer_html(div);
        assert!(html.contains("<div"));
        assert!(html.contains("class=\"container\""));
        assert!(html.contains("Hello"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_find_by_tag_ns() {
        use crate::markup::xml::parse_xml;

        let xml = r#"<root xmlns:a="http://a.com" xmlns:b="http://b.com">
            <a:item>A</a:item>
            <b:item>B</b:item>
        </root>"#;
        let doc = parse_xml(xml);

        let a_items = doc.find_by_tag_ns("http://a.com", "item");
        let b_items = doc.find_by_tag_ns("http://b.com", "item");

        assert_eq!(a_items.len(), 1);
        assert_eq!(b_items.len(), 1);

        // Verify content
        assert_eq!(doc.text_content(a_items[0]), "A");
        assert_eq!(doc.text_content(b_items[0]), "B");
    }

    #[test]
    fn test_find_by_namespace() {
        use crate::markup::xml::parse_xml;

        let xml = r#"<root xmlns:ns="http://example.com">
            <ns:one>1</ns:one>
            <ns:two>2</ns:two>
            <other>3</other>
        </root>"#;
        let doc = parse_xml(xml);

        let ns_elements = doc.find_by_namespace("http://example.com");
        assert_eq!(ns_elements.len(), 2);
    }

    #[test]
    fn test_outer_xml_preserves_ancestor_namespaces() {
        use crate::markup::xml::parse_xml;

        let xml = r#"<root xmlns="http://default.com" xmlns:a="http://a.com">
            <a:item>content</a:item>
        </root>"#;
        let doc = parse_xml(xml);

        // Get the inner item element
        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 1);

        // Serialize just the item - should include ancestor namespaces
        let item_xml = doc.outer_xml(items[0]);

        // The output should contain the namespace declaration for a:
        assert!(item_xml.contains("a:item"), "Should have prefixed element");
        assert!(
            item_xml.contains("xmlns:a"),
            "Should include ancestor xmlns:a"
        );
    }

    #[test]
    fn test_is_empty() {
        let doc = Document::new();
        assert!(doc.is_empty()); // Only root node

        let mut doc2 = Document::new();
        let div = doc2.create_element("div");
        doc2.append_child(doc2.root(), div).unwrap();
        assert!(!doc2.is_empty());
    }

    #[test]
    fn test_create_comment() {
        let mut doc = Document::new();
        let comment = doc.create_comment("This is a comment");
        doc.append_child(doc.root(), comment).unwrap();

        let html = doc.outer_html(doc.root());
        assert!(html.contains("<!--This is a comment-->"));
    }

    #[test]
    fn test_descendants() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        let p = doc.create_element("p");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, span).unwrap();
        doc.append_child(span, p).unwrap();

        let descs = doc.descendants(div);
        assert_eq!(descs.len(), 2); // span and p
    }

    #[test]
    fn test_ancestors() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        let p = doc.create_element("p");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, span).unwrap();
        doc.append_child(span, p).unwrap();

        let ancs = doc.ancestors(p);
        assert_eq!(ancs.len(), 3); // span, div, root
    }

    #[test]
    fn test_siblings() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span1 = doc.create_element("span");
        let span2 = doc.create_element("span");
        let span3 = doc.create_element("span");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, span1).unwrap();
        doc.append_child(div, span2).unwrap();
        doc.append_child(div, span3).unwrap();

        let sibs = doc.siblings(span2);
        assert_eq!(sibs.len(), 2); // span1 and span3
    }

    #[test]
    fn test_remove() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, span).unwrap();

        assert_eq!(doc.children(div).len(), 1);
        let _ = doc.remove(span);
        assert_eq!(doc.children(div).len(), 0);
    }

    #[test]
    fn test_insert_before() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span1 = doc.create_element("span");
        let span2 = doc.create_element("span");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, span2).unwrap();
        // insert_before(parent, new_node, ref_node)
        doc.insert_before(div, span1, span2).unwrap();

        let children = doc.children(div);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], span1);
        assert_eq!(children[1], span2);
    }

    #[test]
    fn test_inner_html() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        let text = doc.create_text("Hello");

        doc.append_child(doc.root(), div).unwrap();
        doc.append_child(div, span).unwrap();
        doc.append_child(span, text).unwrap();

        let inner = doc.inner_html(div);
        assert!(inner.contains("<span>"));
        assert!(inner.contains("Hello"));
    }

    #[test]
    fn test_find_by_class() {
        let mut doc = Document::new();
        let div1 = doc.create_element("div");
        let div2 = doc.create_element("div");
        let div3 = doc.create_element("div");

        doc.get_mut(div1).unwrap().set_attr("class", "foo bar");
        doc.get_mut(div2).unwrap().set_attr("class", "foo");
        doc.get_mut(div3).unwrap().set_attr("class", "baz");

        doc.append_child(doc.root(), div1).unwrap();
        doc.append_child(doc.root(), div2).unwrap();
        doc.append_child(doc.root(), div3).unwrap();

        let foos = doc.find_by_class("foo");
        assert_eq!(foos.len(), 2);
    }
}

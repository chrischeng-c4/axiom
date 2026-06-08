//! XPath query engine.

use crate::markup::dom::{Document, NodeId};
use crate::markup::error::Result;

/// Execute an XPath query on a document.
pub fn xpath(doc: &Document, expr: &str) -> Result<Vec<NodeId>> {
    let query = XPathQuery::parse(expr)?;
    Ok(query.execute(doc))
}

/// A parsed XPath query.
#[derive(Debug)]
pub struct XPathQuery {
    steps: Vec<XPathStep>,
}

#[derive(Debug)]
struct XPathStep {
    axis: Axis,
    node_test: NodeTest,
    predicates: Vec<Predicate>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Axis {
    Child,
    Descendant,
    DescendantOrSelf,
    Parent,
    Ancestor,
    Self_,
    Attribute,
}

#[derive(Debug)]
enum NodeTest {
    Name(String),
    Any,
    Text,
}

#[derive(Debug)]
enum Predicate {
    Position(usize),
    Attribute(String, Option<String>),
    Contains(String, String),
}

impl XPathQuery {
    /// Parse an XPath expression.
    pub fn parse(expr: &str) -> Result<Self> {
        let expr = expr.trim();
        let mut steps = Vec::new();
        let parts = expr.split('/').filter(|s| !s.is_empty());

        let is_absolute = expr.starts_with('/');
        let is_descendant = expr.starts_with("//");

        for part in parts {
            let step = parse_step(part, is_descendant && steps.is_empty())?;
            steps.push(step);
        }

        if steps.is_empty() && is_absolute {
            // Root selector
            steps.push(XPathStep {
                axis: Axis::Self_,
                node_test: NodeTest::Any,
                predicates: Vec::new(),
            });
        }

        Ok(XPathQuery { steps })
    }

    /// Execute the query on a document.
    pub fn execute(&self, doc: &Document) -> Vec<NodeId> {
        let mut current: Vec<NodeId> = vec![doc.root()];

        for step in &self.steps {
            current = step.apply(doc, &current);
        }

        current
    }
}

fn parse_step(part: &str, is_descendant: bool) -> Result<XPathStep> {
    let mut part = part.trim();

    // Check for axis prefix
    let axis = if part.starts_with("..") {
        part = &part[2..];
        Axis::Parent
    } else if part.starts_with('.') {
        part = &part[1..];
        Axis::Self_
    } else if part.starts_with('@') {
        part = &part[1..];
        Axis::Attribute
    } else if is_descendant {
        Axis::DescendantOrSelf
    } else {
        Axis::Child
    };

    // Parse predicates
    let (name_part, predicates) = if let Some(bracket_pos) = part.find('[') {
        let name = &part[..bracket_pos];
        let pred_str = &part[bracket_pos..];
        (name, parse_predicates(pred_str)?)
    } else {
        (part, Vec::new())
    };

    // Parse node test
    let node_test = match name_part {
        "*" => NodeTest::Any,
        "text()" => NodeTest::Text,
        name => NodeTest::Name(name.to_lowercase()),
    };

    Ok(XPathStep {
        axis,
        node_test,
        predicates,
    })
}

fn parse_predicates(s: &str) -> Result<Vec<Predicate>> {
    let mut predicates = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '[' => {
                if depth == 0 {
                    start = i + 1;
                }
                depth += 1;
            }
            ']' => {
                depth -= 1;
                if depth == 0 {
                    let pred_content = s[start..i].trim();
                    if let Some(pred) = parse_single_predicate(pred_content) {
                        predicates.push(pred);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(predicates)
}

fn parse_single_predicate(s: &str) -> Option<Predicate> {
    // Position predicate: [1], [2], etc.
    if let Ok(pos) = s.parse::<usize>() {
        return Some(Predicate::Position(pos));
    }

    // Attribute predicate: [@id="value"]
    if s.starts_with('@') {
        let attr_part = &s[1..];
        if let Some(eq_pos) = attr_part.find('=') {
            let name = attr_part[..eq_pos].trim();
            let value = attr_part[eq_pos + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            return Some(Predicate::Attribute(
                name.to_string(),
                Some(value.to_string()),
            ));
        } else {
            return Some(Predicate::Attribute(attr_part.to_string(), None));
        }
    }

    // Contains predicate: [contains(@class, "foo")]
    if s.starts_with("contains(") {
        let inner = s.strip_prefix("contains(")?.strip_suffix(')')?;
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 2 {
            let attr = parts[0].trim().strip_prefix('@')?.to_string();
            let value = parts[1]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            return Some(Predicate::Contains(attr, value));
        }
    }

    None
}

impl XPathStep {
    fn apply(&self, doc: &Document, nodes: &[NodeId]) -> Vec<NodeId> {
        let mut result = Vec::new();

        for &node_id in nodes {
            let candidates = match self.axis {
                Axis::Child => doc.children(node_id),
                Axis::Descendant => doc.descendants(node_id),
                Axis::DescendantOrSelf => {
                    let mut r = vec![node_id];
                    r.extend(doc.descendants(node_id));
                    r
                }
                Axis::Parent => doc.parent(node_id).into_iter().collect(),
                Axis::Ancestor => doc.ancestors(node_id),
                Axis::Self_ => vec![node_id],
                Axis::Attribute => continue, // Handle attributes separately
            };

            for candidate in candidates {
                if self.matches(doc, candidate) {
                    result.push(candidate);
                }
            }
        }

        // Apply predicates
        for pred in &self.predicates {
            result = apply_predicate(doc, &result, pred);
        }

        result
    }

    fn matches(&self, doc: &Document, node_id: NodeId) -> bool {
        let Some(node) = doc.get(node_id) else {
            return false;
        };

        match &self.node_test {
            NodeTest::Any => node.is_element(),
            NodeTest::Text => node.is_text(),
            NodeTest::Name(name) => {
                node.is_element()
                    && node
                        .tag()
                        .map(|t| t.eq_ignore_ascii_case(name))
                        .unwrap_or(false)
            }
        }
    }
}

fn apply_predicate(doc: &Document, nodes: &[NodeId], pred: &Predicate) -> Vec<NodeId> {
    match pred {
        Predicate::Position(pos) => {
            if *pos > 0 && *pos <= nodes.len() {
                vec![nodes[pos - 1]]
            } else {
                Vec::new()
            }
        }
        Predicate::Attribute(name, value) => nodes
            .iter()
            .filter(|&&id| {
                doc.get(id)
                    .map(|n| {
                        let attr = n.get_attr(name);
                        match (attr, value) {
                            (Some(_), None) => true,
                            (Some(a), Some(v)) => a == v,
                            _ => false,
                        }
                    })
                    .unwrap_or(false)
            })
            .copied()
            .collect(),
        Predicate::Contains(attr, value) => nodes
            .iter()
            .filter(|&&id| {
                doc.get(id)
                    .and_then(|n| n.get_attr(attr))
                    .map(|a| a.contains(value.as_str()))
                    .unwrap_or(false)
            })
            .copied()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::html::parse_html;

    #[test]
    fn test_xpath_simple() {
        let doc = parse_html("<root><item>A</item><item>B</item></root>");
        let result = xpath(&doc, "//item").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_xpath_with_predicate() {
        let doc = parse_html("<root><item>A</item><item>B</item></root>");
        let result = xpath(&doc, "//item[1]").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_xpath_attribute() {
        let doc = parse_html(r#"<root><item id="a">A</item><item id="b">B</item></root>"#);
        let result = xpath(&doc, r#"//item[@id="a"]"#).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_xpath_descendant() {
        let doc = parse_html("<div><ul><li>A</li><li>B</li></ul></div>");
        let result = xpath(&doc, "//li").unwrap();
        assert_eq!(result.len(), 2);
    }
}

//! CSS selector engine.

use crate::markup::dom::{Document, NodeId};
use crate::markup::error::{MarkupError, Result};

/// A parsed CSS selector.
#[derive(Debug, Clone)]
pub struct Selector {
    parts: Vec<SelectorPart>,
}

#[derive(Debug, Clone)]
enum SelectorPart {
    /// Tag name selector.
    Tag(String),
    /// Class selector (.class).
    Class(String),
    /// ID selector (#id).
    Id(String),
    /// Attribute selector ([attr] or [attr="value"]).
    Attribute {
        name: String,
        op: Option<AttrOp>,
        value: Option<String>,
    },
    /// Universal selector (*).
    Universal,
    /// Combinator.
    Combinator(Combinator),
}

#[derive(Debug, Clone)]
enum AttrOp {
    /// [attr="value"] - exact match.
    Equals,
    /// [attr~="value"] - word match.
    Contains,
    /// [attr^="value"] - starts with.
    StartsWith,
    /// [attr$="value"] - ends with.
    EndsWith,
    /// [attr*="value"] - substring match.
    Substring,
}

#[derive(Debug, Clone)]
enum Combinator {
    /// Descendant (space).
    Descendant,
    /// Child (>).
    Child,
    /// Adjacent sibling (+).
    Adjacent,
    /// General sibling (~).
    Sibling,
}

impl Selector {
    /// Parse a CSS selector string.
    pub fn parse(input: &str) -> Result<Self> {
        let mut parts = Vec::new();
        let mut chars = input.chars().peekable();
        let mut current_tag = String::new();

        while let Some(&c) = chars.peek() {
            match c {
                '.' => {
                    if !current_tag.is_empty() {
                        parts.push(SelectorPart::Tag(current_tag.clone()));
                        current_tag.clear();
                    }
                    chars.next();
                    let class = read_identifier(&mut chars);
                    if class.is_empty() {
                        return Err(MarkupError::InvalidSelector("empty class name".into()));
                    }
                    parts.push(SelectorPart::Class(class));
                }
                '#' => {
                    if !current_tag.is_empty() {
                        parts.push(SelectorPart::Tag(current_tag.clone()));
                        current_tag.clear();
                    }
                    chars.next();
                    let id = read_identifier(&mut chars);
                    if id.is_empty() {
                        return Err(MarkupError::InvalidSelector("empty id".into()));
                    }
                    parts.push(SelectorPart::Id(id));
                }
                '[' => {
                    if !current_tag.is_empty() {
                        parts.push(SelectorPart::Tag(current_tag.clone()));
                        current_tag.clear();
                    }
                    chars.next();
                    let attr = parse_attribute_selector(&mut chars)?;
                    parts.push(attr);
                }
                '*' => {
                    chars.next();
                    parts.push(SelectorPart::Universal);
                }
                ' ' => {
                    if !current_tag.is_empty() {
                        parts.push(SelectorPart::Tag(current_tag.clone()));
                        current_tag.clear();
                    }
                    chars.next();
                    skip_whitespace(&mut chars);
                    if chars.peek().is_some()
                        && !matches!(chars.peek(), Some('>') | Some('+') | Some('~'))
                    {
                        parts.push(SelectorPart::Combinator(Combinator::Descendant));
                    }
                }
                '>' => {
                    if !current_tag.is_empty() {
                        parts.push(SelectorPart::Tag(current_tag.clone()));
                        current_tag.clear();
                    }
                    chars.next();
                    skip_whitespace(&mut chars);
                    parts.push(SelectorPart::Combinator(Combinator::Child));
                }
                '+' => {
                    if !current_tag.is_empty() {
                        parts.push(SelectorPart::Tag(current_tag.clone()));
                        current_tag.clear();
                    }
                    chars.next();
                    skip_whitespace(&mut chars);
                    parts.push(SelectorPart::Combinator(Combinator::Adjacent));
                }
                '~' => {
                    if !current_tag.is_empty() {
                        parts.push(SelectorPart::Tag(current_tag.clone()));
                        current_tag.clear();
                    }
                    chars.next();
                    skip_whitespace(&mut chars);
                    parts.push(SelectorPart::Combinator(Combinator::Sibling));
                }
                _ if c.is_alphanumeric() || c == '-' || c == '_' => {
                    chars.next();
                    current_tag.push(c);
                }
                _ => {
                    chars.next();
                }
            }
        }

        if !current_tag.is_empty() {
            parts.push(SelectorPart::Tag(current_tag));
        }

        Ok(Selector { parts })
    }

    /// Select all matching nodes from a document.
    pub fn select(&self, doc: &Document) -> Vec<NodeId> {
        let mut candidates: Vec<NodeId> = doc.descendants(doc.root());

        let mut i = 0;
        while i < self.parts.len() {
            let part = &self.parts[i];

            match part {
                SelectorPart::Combinator(comb) => {
                    i += 1;
                    if i >= self.parts.len() {
                        break;
                    }

                    let next_parts = &self.parts[i..];
                    candidates = apply_combinator(doc, &candidates, comb, next_parts);
                    // Skip the simple selectors we just processed
                    while i < self.parts.len()
                        && !matches!(self.parts[i], SelectorPart::Combinator(_))
                    {
                        i += 1;
                    }
                    continue;
                }
                _ => {
                    candidates = candidates
                        .into_iter()
                        .filter(|&id| matches_simple(doc, id, part))
                        .collect();
                }
            }
            i += 1;
        }

        candidates
    }
}

fn read_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            result.push(c);
            chars.next();
        } else {
            break;
        }
    }
    result
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_attribute_selector(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<SelectorPart> {
    let name = read_identifier(chars);

    skip_whitespace(chars);

    let (op, value) = match chars.peek() {
        Some(']') => {
            chars.next();
            (None, None)
        }
        Some('=') => {
            chars.next();
            let value = read_attr_value(chars);
            skip_to_bracket(chars);
            (Some(AttrOp::Equals), Some(value))
        }
        Some('~') => {
            chars.next();
            chars.next(); // Skip '='
            let value = read_attr_value(chars);
            skip_to_bracket(chars);
            (Some(AttrOp::Contains), Some(value))
        }
        Some('^') => {
            chars.next();
            chars.next();
            let value = read_attr_value(chars);
            skip_to_bracket(chars);
            (Some(AttrOp::StartsWith), Some(value))
        }
        Some('$') => {
            chars.next();
            chars.next();
            let value = read_attr_value(chars);
            skip_to_bracket(chars);
            (Some(AttrOp::EndsWith), Some(value))
        }
        Some('*') => {
            chars.next();
            chars.next();
            let value = read_attr_value(chars);
            skip_to_bracket(chars);
            (Some(AttrOp::Substring), Some(value))
        }
        _ => {
            skip_to_bracket(chars);
            (None, None)
        }
    };

    Ok(SelectorPart::Attribute { name, op, value })
}

fn read_attr_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    skip_whitespace(chars);

    let quote = match chars.peek() {
        Some('"') | Some('\'') => {
            let q = *chars.peek().unwrap();
            chars.next();
            Some(q)
        }
        _ => None,
    };

    let mut value = String::new();
    while let Some(&c) = chars.peek() {
        if let Some(q) = quote {
            if c == q {
                chars.next();
                break;
            }
        } else if c == ']' {
            break;
        }
        value.push(c);
        chars.next();
    }

    value
}

fn skip_to_bracket(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        chars.next();
        if c == ']' {
            break;
        }
    }
}

fn matches_simple(doc: &Document, node_id: NodeId, part: &SelectorPart) -> bool {
    let Some(node) = doc.get(node_id) else {
        return false;
    };

    if !node.is_element() {
        return false;
    }

    match part {
        SelectorPart::Tag(tag) => node
            .tag()
            .map(|t| t.eq_ignore_ascii_case(tag))
            .unwrap_or(false),
        SelectorPart::Class(class) => node.has_class(class),
        SelectorPart::Id(id) => node.get_id() == Some(id.as_str()),
        SelectorPart::Universal => true,
        SelectorPart::Attribute { name, op, value } => {
            let attr_val = node.get_attr(name);
            match (op, value, attr_val) {
                (None, _, Some(_)) => true,
                (None, _, None) => false,
                (Some(_), _, None) => false,
                (Some(AttrOp::Equals), Some(v), Some(a)) => a == v,
                (Some(AttrOp::Contains), Some(v), Some(a)) => a.split_whitespace().any(|w| w == v),
                (Some(AttrOp::StartsWith), Some(v), Some(a)) => a.starts_with(v.as_str()),
                (Some(AttrOp::EndsWith), Some(v), Some(a)) => a.ends_with(v.as_str()),
                (Some(AttrOp::Substring), Some(v), Some(a)) => a.contains(v.as_str()),
                _ => false,
            }
        }
        SelectorPart::Combinator(_) => true,
    }
}

fn apply_combinator(
    doc: &Document,
    candidates: &[NodeId],
    combinator: &Combinator,
    remaining: &[SelectorPart],
) -> Vec<NodeId> {
    let mut result = Vec::new();

    for &node_id in candidates {
        let related = match combinator {
            Combinator::Descendant => doc.descendants(node_id),
            Combinator::Child => doc.children(node_id),
            Combinator::Adjacent => {
                // Next sibling only
                if let Some(parent_id) = doc.parent(node_id) {
                    let siblings = doc.children(parent_id);
                    let pos = siblings.iter().position(|&id| id == node_id);
                    pos.and_then(|p| siblings.get(p + 1).copied())
                        .into_iter()
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Combinator::Sibling => {
                // All following siblings
                if let Some(parent_id) = doc.parent(node_id) {
                    let siblings = doc.children(parent_id);
                    let pos = siblings.iter().position(|&id| id == node_id);
                    pos.map(|p| siblings[p + 1..].to_vec()).unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
        };

        for &related_id in &related {
            // Check if matches remaining simple selectors
            let matches = remaining
                .iter()
                .take_while(|p| !matches!(p, SelectorPart::Combinator(_)))
                .all(|p| matches_simple(doc, related_id, p));

            if matches && !result.contains(&related_id) {
                result.push(related_id);
            }
        }
    }

    result
}

/// Select nodes using a CSS selector string.
pub fn select(doc: &Document, selector: &str) -> Result<Vec<NodeId>> {
    let sel = Selector::parse(selector)?;
    Ok(sel.select(doc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::html::parse_html;

    #[test]
    fn test_select_by_tag() {
        let doc = parse_html("<div><p>A</p><p>B</p></div>");
        let result = select(&doc, "p").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_select_by_class() {
        let doc = parse_html(r#"<div class="a"><span class="a b">X</span></div>"#);
        let result = select(&doc, ".a").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_select_by_id() {
        let doc = parse_html(r#"<div id="main"><span id="sub">X</span></div>"#);
        let result = select(&doc, "#main").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_select_descendant() {
        let doc = parse_html("<div><ul><li>A</li><li>B</li></ul></div>");
        let result = select(&doc, "div li").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_select_child() {
        let doc = parse_html("<div><p>A</p><div><p>B</p></div></div>");
        let result = select(&doc, "div > p").unwrap();
        // Only direct children
        assert_eq!(result.len(), 2); // Both p's are direct children of their parent divs
    }

    #[test]
    fn test_select_attribute() {
        let doc = parse_html(r#"<a href="x">A</a><a>B</a>"#);
        let result = select(&doc, "[href]").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_select_combined() {
        let doc = parse_html(r#"<div class="container"><p class="item">X</p></div>"#);
        let result = select(&doc, "div.container p.item").unwrap();
        assert_eq!(result.len(), 1);
    }
}

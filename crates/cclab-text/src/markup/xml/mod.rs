//! XML parsing module with namespace support.

mod parser;

use crate::markup::dom::Document;

pub use parser::parse_xml_with_ns;

/// Parse XML string into a Document with full namespace support.
///
/// This parser correctly handles:
/// - Default namespaces (`xmlns="..."`)
/// - Prefixed namespaces (`xmlns:prefix="..."`)
/// - CDATA sections
/// - Processing instructions
/// - XML entity references
///
/// # Example
///
/// ```
/// use cclab_text::markup::xml::parse_xml;
///
/// let xml = r#"<ns:root xmlns:ns="http://example.com"><ns:item>A</ns:item></ns:root>"#;
/// let doc = parse_xml(xml);
///
/// let root = doc.find_by_tag("root");
/// let node = doc.get(root[0]).unwrap();
/// assert_eq!(node.namespace.as_deref(), Some("http://example.com"));
/// ```
pub fn parse_xml(xml: &str) -> Document {
    parse_xml_with_ns(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_xml() {
        let doc = parse_xml("<root><item>A</item><item>B</item></root>");
        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_parse_with_namespace() {
        let xml = r#"<root xmlns="http://example.com"><item>A</item></root>"#;
        let doc = parse_xml(xml);

        let root = doc.find_by_tag("root")[0];
        let node = doc.get(root).unwrap();
        assert_eq!(node.namespace.as_deref(), Some("http://example.com"));
    }
}

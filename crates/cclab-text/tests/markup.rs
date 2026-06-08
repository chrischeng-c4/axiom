//! Integration tests for the markup module (HTML/XML parsing, CSS selectors, XPath, XSLT).
//!
//! These tests were extracted from the inline `#[cfg(test)]` modules in the markup source files.

#![cfg(feature = "markup")]

use cclab_text::markup::{parse_html, parse_xml, select, transform, xpath, Document};

// ============================================================================
// Document tests (from dom/document.rs)
// ============================================================================

mod document_tests {
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

// ============================================================================
// HTML parser tests (from html/parser.rs)
// ============================================================================

mod html_tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let doc = parse_html("<div>Hello</div>");
        let divs = doc.find_by_tag("div");
        assert_eq!(divs.len(), 1);
        assert_eq!(doc.text_content(divs[0]), "Hello");
    }

    #[test]
    fn test_parse_nested() {
        let doc = parse_html("<div><span>A</span><span>B</span></div>");
        let spans = doc.find_by_tag("span");
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_parse_attributes() {
        let doc = parse_html(r#"<div id="main" class="container">Content</div>"#);
        let div_id = doc.find_by_id("main").unwrap();
        let div = doc.get(div_id).unwrap();
        assert!(div.has_class("container"));
    }

    #[test]
    fn test_parse_unclosed_tags() {
        // Lenient parsing - unclosed tags should be handled
        let doc = parse_html("<div><p>First<p>Second</div>");
        let paragraphs = doc.find_by_tag("p");
        assert_eq!(paragraphs.len(), 2);
    }

    #[test]
    fn test_parse_void_elements() {
        let doc = parse_html("<div><br><hr><img src='test.png'></div>");
        assert_eq!(doc.find_by_tag("br").len(), 1);
        assert_eq!(doc.find_by_tag("hr").len(), 1);
        assert_eq!(doc.find_by_tag("img").len(), 1);
    }

    #[test]
    fn test_parse_malformed() {
        // Missing closing tags
        let doc = parse_html("<div><span>text");
        assert_eq!(doc.find_by_tag("div").len(), 1);
        assert_eq!(doc.find_by_tag("span").len(), 1);
    }
}

// ============================================================================
// CSS selector tests (from css/selector.rs)
// ============================================================================

mod css_tests {
    use super::*;

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

// ============================================================================
// XPath tests (from xpath/mod.rs)
// ============================================================================

mod xpath_tests {
    use super::*;

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

// ============================================================================
// XSLT tests (from xslt/mod.rs)
// ============================================================================

mod xslt_tests {
    use super::*;

    #[test]
    fn test_simple_transform() {
        let source = parse_html("<root><item>Hello</item></root>");
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <output><xsl:value-of select="//item"/></output>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_apply_templates() {
        let source = parse_html("<root><item>A</item><item>B</item></root>");
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result><xsl:apply-templates select="//item"/></result>
                </xsl:template>
                <xsl:template match="item">
                    <entry><xsl:value-of select="."/></entry>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<entry>A</entry>"));
        assert!(result.contains("<entry>B</entry>"));
    }

    #[test]
    fn test_choose_when_otherwise() {
        let source = parse_html(r#"<root><item type="a">A</item><item type="b">B</item></root>"#);
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result>
                        <xsl:for-each select="//item">
                            <xsl:choose>
                                <xsl:when test="@type='a'"><alpha><xsl:value-of select="."/></alpha></xsl:when>
                                <xsl:otherwise><other><xsl:value-of select="."/></other></xsl:otherwise>
                            </xsl:choose>
                        </xsl:for-each>
                    </result>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<alpha>A</alpha>"));
        assert!(result.contains("<other>B</other>"));
    }

    #[test]
    fn test_copy_of() {
        let source = parse_html("<root><item>Content</item></root>");
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result><xsl:copy-of select="//item"/></result>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<item>Content</item>"));
    }

    #[test]
    fn test_copy() {
        let source = parse_html(r#"<root><item id="x">Content</item></root>"#);
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result><xsl:apply-templates select="//item"/></result>
                </xsl:template>
                <xsl:template match="item">
                    <xsl:copy>COPIED</xsl:copy>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<item"));
        assert!(result.contains("COPIED"));
    }

    #[test]
    fn test_if_condition() {
        let source = parse_html(r#"<root><item active="true">A</item><item>B</item></root>"#);
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result>
                        <xsl:for-each select="//item">
                            <xsl:if test="@active">
                                <active><xsl:value-of select="."/></active>
                            </xsl:if>
                        </xsl:for-each>
                    </result>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<active>A</active>"));
        assert!(!result.contains("<active>B</active>"));
    }

    #[test]
    fn test_xsl_text() {
        let source = parse_html("<root><item>A</item></root>");
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result><xsl:text>Hello, World!</xsl:text></result>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("Hello, World!"));
    }

    #[test]
    fn test_xsl_element() {
        let source = parse_html(r#"<root><item name="div">Content</item></root>"#);
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result>
                        <xsl:element name="dynamic-tag">
                            <xsl:value-of select="//item"/>
                        </xsl:element>
                    </result>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<dynamic-tag>"));
        assert!(result.contains("Content"));
        assert!(result.contains("</dynamic-tag>"));
    }

    #[test]
    fn test_xsl_element_with_attribute() {
        let source = parse_html("<root><item>Value</item></root>");
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result>
                        <xsl:element name="span">
                            <xsl:attribute name="class">highlight</xsl:attribute>
                            <xsl:value-of select="//item"/>
                        </xsl:element>
                    </result>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<span"));
        assert!(result.contains("class=\"highlight\""));
        assert!(result.contains("Value"));
    }

    #[test]
    fn test_xsl_comment() {
        let source = parse_html("<root><item>A</item></root>");
        let xslt = parse_html(
            r#"
            <xsl:stylesheet version="1.0">
                <xsl:template match="/">
                    <result>
                        <xsl:comment>This is a comment</xsl:comment>
                        <item><xsl:value-of select="//item"/></item>
                    </result>
                </xsl:template>
            </xsl:stylesheet>
        "#,
        );

        let result = transform(&source, &xslt).unwrap();
        assert!(result.contains("<!--This is a comment-->"));
        assert!(result.contains("<item>A</item>"));
    }
}

// ============================================================================
// XML parser tests (from xml/parser.rs)
// ============================================================================

mod xml_tests {
    use super::*;

    #[test]
    fn test_parse_simple_xml() {
        let doc = parse_xml("<root><item>A</item><item>B</item></root>");
        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_parse_with_default_namespace() {
        let xml = r#"<root xmlns="http://example.com"><item>A</item></root>"#;
        let doc = parse_xml(xml);

        let root = doc.find_by_tag("root");
        assert_eq!(root.len(), 1);

        let node = doc.get(root[0]).unwrap();
        assert_eq!(node.namespace.as_deref(), Some("http://example.com"));
    }

    #[test]
    fn test_parse_with_prefixed_namespace() {
        let xml = r#"<ns:root xmlns:ns="http://example.com"><ns:item>A</ns:item></ns:root>"#;
        let doc = parse_xml(xml);

        let root = doc.find_by_tag("root");
        assert_eq!(root.len(), 1);

        let node = doc.get(root[0]).unwrap();
        assert_eq!(node.namespace.as_deref(), Some("http://example.com"));
        assert_eq!(node.prefix.as_deref(), Some("ns"));
    }

    #[test]
    fn test_parse_mixed_namespaces() {
        let xml = r#"
            <root xmlns="http://default.com" xmlns:other="http://other.com">
                <item>Default NS</item>
                <other:item>Other NS</other:item>
            </root>
        "#;
        let doc = parse_xml(xml);

        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);

        // First item has default namespace
        let item1 = doc.get(items[0]).unwrap();
        assert_eq!(item1.namespace.as_deref(), Some("http://default.com"));
        assert!(item1.prefix.is_none());

        // Second item has prefixed namespace
        let item2 = doc.get(items[1]).unwrap();
        assert_eq!(item2.namespace.as_deref(), Some("http://other.com"));
        assert_eq!(item2.prefix.as_deref(), Some("other"));
    }

    #[test]
    fn test_parse_cdata() {
        let xml = r#"<root><![CDATA[<not-a-tag>]]></root>"#;
        let doc = parse_xml(xml);

        let root = doc.find_by_tag("root")[0];
        assert_eq!(doc.text_content(root), "<not-a-tag>");
    }

    #[test]
    fn test_parse_processing_instruction() {
        let xml = r#"<?xml version="1.0"?><root>content</root>"#;
        let doc = parse_xml(xml);

        let root = doc.find_by_tag("root");
        assert_eq!(root.len(), 1);
    }

    #[test]
    fn test_parse_self_closing() {
        let xml = r#"<root><empty/><empty2 /></root>"#;
        let doc = parse_xml(xml);

        assert_eq!(doc.find_by_tag("empty").len(), 1);
        assert_eq!(doc.find_by_tag("empty2").len(), 1);
    }

    #[test]
    fn test_prefix_sensitive_end_tags() {
        // Test that end tags correctly match by prefix, not just local name
        let xml = r#"<root xmlns:a="http://a.com" xmlns:b="http://b.com">
            <a:item>A content</a:item>
            <b:item>B content</b:item>
        </root>"#;
        let doc = parse_xml(xml);

        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);

        // Verify correct namespace assignment
        let item1 = doc.get(items[0]).unwrap();
        let item2 = doc.get(items[1]).unwrap();

        assert_eq!(item1.namespace.as_deref(), Some("http://a.com"));
        assert_eq!(item1.prefix.as_deref(), Some("a"));
        assert_eq!(item2.namespace.as_deref(), Some("http://b.com"));
        assert_eq!(item2.prefix.as_deref(), Some("b"));
    }

    #[test]
    fn test_nested_same_local_name_different_prefix() {
        // Nested elements with same local name but different prefixes
        let xml = r#"<a:root xmlns:a="http://a.com" xmlns:b="http://b.com">
            <a:item>
                <b:item>nested</b:item>
            </a:item>
        </a:root>"#;
        let doc = parse_xml(xml);

        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);

        // Find by namespace
        let a_items = doc.find_by_tag_ns("http://a.com", "item");
        let b_items = doc.find_by_tag_ns("http://b.com", "item");

        assert_eq!(a_items.len(), 1);
        assert_eq!(b_items.len(), 1);
    }
}

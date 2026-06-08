//! XSLT transformation module.
//!
//! Supports XSLT 1.0 core instructions:
//! - `xsl:template` with match and mode attributes
//! - `xsl:apply-templates` with select and mode
//! - `xsl:value-of` with select
//! - `xsl:for-each` with select
//! - `xsl:if` with test
//! - `xsl:choose`, `xsl:when`, `xsl:otherwise`
//! - `xsl:copy`, `xsl:copy-of`
//! - `xsl:text` with disable-output-escaping
//! - `xsl:attribute` with name and namespace
//! - `xsl:element` with name and namespace
//! - `xsl:comment` for XML comments

use crate::markup::dom::Document;
use crate::markup::error::{MarkupError, Result};

/// Transform an XML document using an XSLT stylesheet.
pub fn transform(source: &Document, stylesheet: &Document) -> Result<String> {
    let transformer = Transformer::new(stylesheet)?;
    transformer.transform(source)
}

/// XSLT transformer.
pub struct Transformer<'a> {
    stylesheet: &'a Document,
}

impl<'a> Transformer<'a> {
    /// Create a new transformer from a stylesheet document.
    pub fn new(stylesheet: &'a Document) -> Result<Self> {
        // Validate it's an XSLT stylesheet
        let stylesheets = stylesheet.find_by_tag("xsl:stylesheet");
        let transforms = stylesheet.find_by_tag("xsl:transform");

        if stylesheets.is_empty() && transforms.is_empty() {
            return Err(MarkupError::InvalidXslt(
                "document is not an XSLT stylesheet".into(),
            ));
        }

        Ok(Self { stylesheet })
    }

    /// Transform a source document.
    pub fn transform(&self, source: &Document) -> Result<String> {
        // Find root template (match="/")
        let templates = self.stylesheet.find_by_tag("xsl:template");
        let root_template = templates.iter().find(|&&id| {
            self.stylesheet
                .get(id)
                .and_then(|n| n.get_attr("match"))
                .map(|m| m == "/")
                .unwrap_or(false)
        });

        if let Some(&template_id) = root_template {
            self.apply_template(source, template_id, source.root(), None)
        } else {
            // Default: output text content
            Ok(source.text_content(source.root()))
        }
    }

    fn apply_template(
        &self,
        source: &Document,
        template_id: usize,
        context: usize,
        _mode: Option<&str>,
    ) -> Result<String> {
        let mut output = String::new();

        let Some(template) = self.stylesheet.get(template_id) else {
            return Ok(output);
        };

        for &child_id in &template.children {
            output.push_str(&self.process_node(source, child_id, context)?);
        }

        Ok(output)
    }

    fn process_node(&self, source: &Document, node_id: usize, context: usize) -> Result<String> {
        let Some(node) = self.stylesheet.get(node_id) else {
            return Ok(String::new());
        };

        if node.is_text() {
            return Ok(node.text.clone().unwrap_or_default());
        }

        let tag = node.tag().unwrap_or("");

        match tag {
            "xsl:value-of" => {
                let select = node.get_attr("select").unwrap_or(".");
                self.evaluate_value_of(source, context, select)
            }
            "xsl:for-each" => {
                let select = node.get_attr("select").unwrap_or("*");
                self.evaluate_for_each(source, context, select, node_id)
            }
            "xsl:if" => {
                let test = node.get_attr("test").unwrap_or("false()");
                self.evaluate_if(source, context, test, node_id)
            }
            "xsl:apply-templates" => self.evaluate_apply_templates(source, context, node_id),
            "xsl:choose" => self.evaluate_choose(source, context, node_id),
            "xsl:copy" => self.evaluate_copy(source, context, node_id),
            "xsl:copy-of" => {
                let select = node.get_attr("select").unwrap_or(".");
                self.evaluate_copy_of(source, context, select)
            }
            "xsl:text" => self.evaluate_text(node_id),
            "xsl:attribute" => {
                // xsl:attribute should be handled by parent element
                // When encountered standalone, just return empty
                Ok(String::new())
            }
            "xsl:element" => self.evaluate_element(source, context, node_id),
            "xsl:comment" => self.evaluate_comment(source, context, node_id),
            _ if tag.starts_with("xsl:") => {
                // Unknown XSLT element, skip
                Ok(String::new())
            }
            _ => {
                // Literal result element
                let mut result = format!("<{}", tag);

                for (name, value) in &node.attributes {
                    result.push_str(&format!(r#" {}="{}""#, name, value));
                }

                result.push('>');

                for &child_id in &node.children {
                    result.push_str(&self.process_node(source, child_id, context)?);
                }

                result.push_str(&format!("</{}>", tag));
                Ok(result)
            }
        }
    }

    /// Evaluate xsl:apply-templates - find and apply matching templates.
    fn evaluate_apply_templates(
        &self,
        source: &Document,
        context: usize,
        node_id: usize,
    ) -> Result<String> {
        let Some(node) = self.stylesheet.get(node_id) else {
            return Ok(String::new());
        };

        // Get select attribute (defaults to child nodes)
        let select = node.get_attr("select");
        let mode = node.get_attr("mode");

        // Get nodes to process
        let nodes_to_process = if let Some(sel) = select {
            if sel == "*" || sel == "node()" {
                source.children(context)
            } else {
                super::xpath::xpath(source, sel)?
            }
        } else {
            // Default: process all child nodes
            source.children(context)
        };

        let mut output = String::new();

        for target_node in nodes_to_process {
            // Find matching template
            if let Some(template_id) = self.find_matching_template(source, target_node, mode) {
                output.push_str(&self.apply_template(source, template_id, target_node, mode)?);
            } else {
                // Default XSLT template behavior per XSLT 1.0 spec:
                // - Text nodes: output the text
                // - Element nodes: recursively apply templates to children
                output.push_str(&self.apply_default_template(source, target_node, mode)?);
            }
        }

        Ok(output)
    }

    /// Apply default XSLT template behavior (XSLT 1.0 spec).
    fn apply_default_template(
        &self,
        source: &Document,
        node_id: usize,
        mode: Option<&str>,
    ) -> Result<String> {
        let Some(node) = source.get(node_id) else {
            return Ok(String::new());
        };

        if node.is_text() {
            // Text nodes: output the text content
            return Ok(node.text.clone().unwrap_or_default());
        }

        if node.is_element() {
            // Element nodes: recursively apply templates to all child nodes
            let mut output = String::new();
            for &child_id in &node.children {
                if let Some(template_id) = self.find_matching_template(source, child_id, mode) {
                    output.push_str(&self.apply_template(source, template_id, child_id, mode)?);
                } else {
                    output.push_str(&self.apply_default_template(source, child_id, mode)?);
                }
            }
            return Ok(output);
        }

        // Other node types: no output
        Ok(String::new())
    }

    /// Find a template matching the given node.
    fn find_matching_template(
        &self,
        source: &Document,
        node_id: usize,
        mode: Option<&str>,
    ) -> Option<usize> {
        let templates = self.stylesheet.find_by_tag("xsl:template");

        let node = source.get(node_id)?;
        let node_tag = node.tag();

        for &template_id in &templates {
            let template = self.stylesheet.get(template_id)?;

            // Check mode matches
            let template_mode = template.get_attr("mode");
            if mode != template_mode {
                continue;
            }

            // Check match pattern
            if let Some(match_pattern) = template.get_attr("match") {
                // Skip root template
                if match_pattern == "/" {
                    continue;
                }

                // Simple pattern matching
                if match_pattern == "*" && node.is_element() {
                    return Some(template_id);
                }

                // Match by tag name
                if let Some(tag) = node_tag {
                    if match_pattern == tag || match_pattern.ends_with(&format!("/{}", tag)) {
                        return Some(template_id);
                    }

                    // Handle //tag pattern
                    if match_pattern.starts_with("//") {
                        let pattern_tag = &match_pattern[2..];
                        if pattern_tag == tag {
                            return Some(template_id);
                        }
                    }
                }

                // Match text() nodes
                if match_pattern == "text()" && node.is_text() {
                    return Some(template_id);
                }
            }
        }

        None
    }

    /// Evaluate xsl:choose - process first matching when or otherwise.
    fn evaluate_choose(&self, source: &Document, context: usize, node_id: usize) -> Result<String> {
        let Some(node) = self.stylesheet.get(node_id) else {
            return Ok(String::new());
        };

        // Find xsl:when children
        for &child_id in &node.children {
            let Some(child) = self.stylesheet.get(child_id) else {
                continue;
            };

            match child.tag() {
                Some("xsl:when") => {
                    let test = child.get_attr("test").unwrap_or("false()");
                    if self.evaluate_condition(source, context, test)? {
                        // Execute this when block
                        let mut output = String::new();
                        for &when_child in &child.children {
                            output.push_str(&self.process_node(source, when_child, context)?);
                        }
                        return Ok(output);
                    }
                }
                Some("xsl:otherwise") => {
                    // Execute otherwise block (fallback)
                    let mut output = String::new();
                    for &otherwise_child in &child.children {
                        output.push_str(&self.process_node(source, otherwise_child, context)?);
                    }
                    return Ok(output);
                }
                _ => continue,
            }
        }

        Ok(String::new())
    }

    /// Evaluate xsl:copy - shallow copy of context node.
    fn evaluate_copy(&self, source: &Document, context: usize, node_id: usize) -> Result<String> {
        let Some(source_node) = source.get(context) else {
            return Ok(String::new());
        };

        let Some(xsl_node) = self.stylesheet.get(node_id) else {
            return Ok(String::new());
        };

        if source_node.is_text() {
            // Copy text content
            return Ok(source_node.text.clone().unwrap_or_default());
        }

        if source_node.is_element() {
            let tag = source_node.tag().unwrap_or("unknown");
            let prefix = source_node.prefix.as_ref();

            let qname = match prefix {
                Some(p) => format!("{}:{}", p, tag),
                None => tag.to_string(),
            };

            let mut result = format!("<{}", qname);

            // Copy attributes
            for (name, value) in &source_node.attributes {
                result.push_str(&format!(r#" {}="{}""#, name, escape_attr(value)));
            }

            result.push('>');

            // Process xsl:copy children (not source children)
            for &child_id in &xsl_node.children {
                result.push_str(&self.process_node(source, child_id, context)?);
            }

            result.push_str(&format!("</{}>", qname));
            return Ok(result);
        }

        Ok(String::new())
    }

    /// Evaluate xsl:copy-of - deep copy of selected nodes.
    fn evaluate_copy_of(&self, source: &Document, context: usize, select: &str) -> Result<String> {
        if select == "." {
            // Copy current node and all descendants
            return Ok(source.outer_xml(context));
        }

        if select.starts_with('@') {
            // Copy attribute value
            let attr_name = &select[1..];
            return Ok(source
                .get(context)
                .and_then(|n| n.get_attr(attr_name))
                .map(String::from)
                .unwrap_or_default());
        }

        // Use XPath to select nodes and copy them
        let nodes = super::xpath::xpath(source, select)?;
        let mut output = String::new();

        for node_id in nodes {
            output.push_str(&source.outer_xml(node_id));
        }

        Ok(output)
    }

    /// Evaluate xsl:text - output text with optional escaping control.
    fn evaluate_text(&self, node_id: usize) -> Result<String> {
        let Some(node) = self.stylesheet.get(node_id) else {
            return Ok(String::new());
        };

        // Get text content from children
        let mut text = String::new();
        for &child_id in &node.children {
            if let Some(child) = self.stylesheet.get(child_id) {
                if child.is_text() {
                    text.push_str(child.text.as_deref().unwrap_or(""));
                }
            }
        }

        // Check disable-output-escaping attribute
        let disable_escaping = node
            .get_attr("disable-output-escaping")
            .map(|v| v == "yes")
            .unwrap_or(false);

        if disable_escaping {
            Ok(text)
        } else {
            Ok(escape_text(&text))
        }
    }

    /// Evaluate xsl:element - create a dynamic element.
    fn evaluate_element(
        &self,
        source: &Document,
        context: usize,
        node_id: usize,
    ) -> Result<String> {
        let Some(node) = self.stylesheet.get(node_id) else {
            return Ok(String::new());
        };

        // Get element name (required)
        let name = match node.get_attr("name") {
            Some(n) => self.evaluate_attribute_value(source, context, n)?,
            None => return Ok(String::new()),
        };

        // Get namespace (optional)
        let namespace = node.get_attr("namespace");

        // Build opening tag
        let mut result = format!("<{}", name);

        // Add namespace if specified
        if let Some(ns) = namespace {
            result.push_str(&format!(r#" xmlns="{}""#, ns));
        }

        // Process xsl:attribute children first to collect attributes
        let mut content = String::new();
        for &child_id in &node.children {
            if let Some(child) = self.stylesheet.get(child_id) {
                if child.tag() == Some("xsl:attribute") {
                    // Process attribute
                    if let Some(attr_name) = child.get_attr("name") {
                        let attr_name =
                            self.evaluate_attribute_value(source, context, attr_name)?;
                        let mut attr_value = String::new();
                        for &attr_child in &child.children {
                            attr_value.push_str(&self.process_node(source, attr_child, context)?);
                        }
                        result.push_str(&format!(
                            r#" {}="{}""#,
                            attr_name,
                            escape_attr(&attr_value)
                        ));
                    }
                } else {
                    // Regular content
                    content.push_str(&self.process_node(source, child_id, context)?);
                }
            }
        }

        result.push('>');
        result.push_str(&content);
        result.push_str(&format!("</{}>", name));

        Ok(result)
    }

    /// Evaluate xsl:comment - create an XML comment.
    fn evaluate_comment(
        &self,
        source: &Document,
        context: usize,
        node_id: usize,
    ) -> Result<String> {
        let Some(node) = self.stylesheet.get(node_id) else {
            return Ok(String::new());
        };

        // Get comment content from children
        let mut content = String::new();
        for &child_id in &node.children {
            content.push_str(&self.process_node(source, child_id, context)?);
        }

        // XML comments cannot contain "--" or end with "-"
        let sanitized = content.replace("--", "- -");
        let sanitized = if sanitized.ends_with('-') {
            format!("{} ", sanitized)
        } else {
            sanitized
        };

        Ok(format!("<!--{}-->", sanitized))
    }

    /// Evaluate an attribute value template (AVT).
    /// Supports {xpath} expressions within attribute values.
    fn evaluate_attribute_value(
        &self,
        source: &Document,
        context: usize,
        value: &str,
    ) -> Result<String> {
        // Simple implementation: if value contains {}, evaluate the content
        if value.contains('{') && value.contains('}') {
            let mut result = String::new();
            let mut chars = value.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '{' {
                    if chars.peek() == Some(&'{') {
                        // Escaped {{ -> {
                        chars.next();
                        result.push('{');
                    } else {
                        // XPath expression
                        let mut expr = String::new();
                        for c in chars.by_ref() {
                            if c == '}' {
                                break;
                            }
                            expr.push(c);
                        }
                        result.push_str(&self.evaluate_value_of(source, context, &expr)?);
                    }
                } else if c == '}' {
                    if chars.peek() == Some(&'}') {
                        // Escaped }} -> }
                        chars.next();
                        result.push('}');
                    } else {
                        result.push(c);
                    }
                } else {
                    result.push(c);
                }
            }

            Ok(result)
        } else {
            Ok(value.to_string())
        }
    }

    /// Evaluate a condition expression.
    fn evaluate_condition(&self, source: &Document, context: usize, test: &str) -> Result<bool> {
        // Simple boolean conditions
        if test == "true()" {
            return Ok(true);
        }
        if test == "false()" {
            return Ok(false);
        }

        // String comparison: @attr='value' or @attr="value" (must check BEFORE simple existence)
        if test.contains('=') && test.contains('@') {
            let parts: Vec<&str> = test.splitn(2, '=').collect();
            if parts.len() == 2 {
                let attr_part = parts[0].trim();
                let value_part = parts[1].trim().trim_matches('"').trim_matches('\'');

                if attr_part.starts_with('@') {
                    let attr_name = &attr_part[1..];
                    let attr_value = source.get(context).and_then(|n| n.get_attr(attr_name));

                    return Ok(attr_value == Some(value_part));
                }
            }
        }

        // Attribute existence check (simple @attr without comparison)
        if test.starts_with('@') && !test.contains('=') {
            let attr_name = &test[1..];
            return Ok(source
                .get(context)
                .and_then(|n| n.get_attr(attr_name))
                .is_some());
        }

        // Numeric comparison (basic support)
        if let Some(pos) = test.find(" &gt; ").or_else(|| test.find(" > ")) {
            let (left, right) = test.split_at(pos);
            let right = right.trim_start_matches(" &gt; ").trim_start_matches(" > ");
            if let (Ok(l), Ok(r)) = (left.trim().parse::<f64>(), right.trim().parse::<f64>()) {
                return Ok(l > r);
            }
        }

        if let Some(pos) = test.find(" &lt; ").or_else(|| test.find(" < ")) {
            let (left, right) = test.split_at(pos);
            let right = right.trim_start_matches(" &lt; ").trim_start_matches(" < ");
            if let (Ok(l), Ok(r)) = (left.trim().parse::<f64>(), right.trim().parse::<f64>()) {
                return Ok(l < r);
            }
        }

        // XPath returns non-empty result
        let results = super::xpath::xpath(source, test)?;
        Ok(!results.is_empty())
    }

    fn evaluate_value_of(&self, source: &Document, context: usize, select: &str) -> Result<String> {
        // Simple XPath evaluation
        if select == "." {
            return Ok(source.text_content(context));
        }

        if select.starts_with('@') {
            let attr_name = &select[1..];
            return Ok(source
                .get(context)
                .and_then(|n| n.get_attr(attr_name))
                .map(String::from)
                .unwrap_or_default());
        }

        // Use XPath for other cases
        let results = super::xpath::xpath(source, select)?;
        if let Some(&first) = results.first() {
            Ok(source.text_content(first))
        } else {
            Ok(String::new())
        }
    }

    fn evaluate_for_each(
        &self,
        source: &Document,
        context: usize,
        select: &str,
        template_id: usize,
    ) -> Result<String> {
        let mut output = String::new();

        // Get nodes to iterate
        let nodes = if select == "*" {
            source.children(context)
        } else {
            super::xpath::xpath(source, select)?
        };

        let Some(template) = self.stylesheet.get(template_id) else {
            return Ok(output);
        };

        for node_id in nodes {
            for &child_id in &template.children {
                output.push_str(&self.process_node(source, child_id, node_id)?);
            }
        }

        Ok(output)
    }

    fn evaluate_if(
        &self,
        source: &Document,
        context: usize,
        test: &str,
        node_id: usize,
    ) -> Result<String> {
        let condition = self.evaluate_condition(source, context, test)?;

        if condition {
            let Some(node) = self.stylesheet.get(node_id) else {
                return Ok(String::new());
            };

            let mut output = String::new();
            for &child_id in &node.children {
                output.push_str(&self.process_node(source, child_id, context)?);
            }
            Ok(output)
        } else {
            Ok(String::new())
        }
    }
}

/// Escape attribute value for XML output.
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape text content for XML output.
fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::html::parse_html;

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

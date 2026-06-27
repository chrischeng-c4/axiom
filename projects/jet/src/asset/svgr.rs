// <HANDWRITE gap="missing-generator:logic:25f2eddb" tracker="standardize-gap-projects-jet-src-asset-svgr-rs" reason="New SVGR transform: parse an .svg file and emit a React component module (a component returning the SVG as JSX with props/ref forwarded); configurable named (ReactComponent) and/or default export, matching vite-plugin-svgr defaults.">
//! SVGR transform: turn an `.svg` file into a React component module.
//!
//! Mirrors `vite-plugin-svgr` (which `fe-shared` configures with
//! `{ exportType: 'named' }`): `import { ReactComponent as Icon } from
//! './icon.svg'` yields a React component that renders the SVG markup as JSX
//! and forwards incoming props — including `ref`, `className`, and `style` —
//! onto the root `<svg>` element.
//!
//! The emitted module is JSX text (`React.createElement` is NOT pre-compiled
//! here); jet's normal JS/JSX transform pipeline consumes it like any other
//! authored component module. We deliberately keep the transform a pure
//! string-in/string-out function so it can be unit-tested without a build.
//!
//! Scope (issue #203): the common case — a simple SVG with element nesting,
//! self-closing tags, and attribute conversion. Advanced SVGO optimization,
//! exotic SVG features, and non-React targets are deferred — see the
//! `// TODO(#203 follow-up)` markers below.

use anyhow::{anyhow, Result};

/// Which exports the emitted component module should provide.
///
/// Defaults to [`SvgrExportType::Named`] to match the `fe-shared`
/// `vite-plugin-svgr` config (`{ exportType: 'named' }`), where the component
/// is imported as `import { ReactComponent as Icon } from './icon.svg'`.
/// @spec .aw/tech-design/projects/jet/logic/svgr-import-svg-as-a-react-component-named-default-export.md#logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgrExportType {
    /// Export only `export const ReactComponent = ...` (vite-plugin-svgr
    /// `exportType: 'named'`, the `fe-shared` default).
    Named,
    /// Export only `export default ...` so `import Icon from './icon.svg'`
    /// resolves to the component.
    Default,
    /// Export both the named `ReactComponent` and the default — useful when a
    /// codebase mixes both import styles.
    Both,
}

/// @spec .aw/tech-design/projects/jet/logic/svgr-import-svg-as-a-react-component-named-default-export.md#logic
impl Default for SvgrExportType {
    fn default() -> Self {
        // Match fe-shared's `{ exportType: 'named' }`.
        SvgrExportType::Named
    }
}

/// Transform raw SVG source text into a React component module (JSX text).
///
/// The returned module:
/// - imports React (`import * as React from "react";`),
/// - declares a `ReactComponent` that takes `props` and forwards `ref`,
/// - returns the SVG as JSX with the root `<svg>` spreading `{...props}` and
///   `ref={ref}`,
/// - converts SVG/HTML attributes to their JSX equivalents (`class` →
///   `className`, kebab-case → camelCase, `style="..."` → object literal),
/// - exports per [`SvgrExportType`].
///
/// Errors if the source contains no `<svg>` root element.
/// @spec .aw/tech-design/projects/jet/logic/svgr-import-svg-as-a-react-component-named-default-export.md#logic
pub fn transform_svg_to_component(svg_src: &str, export_type: SvgrExportType) -> Result<String> {
    let jsx = svg_to_jsx(svg_src)?;

    // The component forwards `ref` so callers can attach a ref to the root
    // `<svg>` (parity with vite-plugin-svgr's `ref: true` default-ish
    // behavior; we always forward to keep the surface predictable).
    let component_body = format!(
        "const ReactComponent = React.forwardRef(function SvgComponent(props, ref) {{\n\
         \x20\x20return (\n{}\n  );\n}});",
        indent_block(&jsx, 4)
    );

    let mut module = String::new();
    module.push_str("import * as React from \"react\";\n");
    module.push('\n');
    module.push_str(&component_body);
    module.push('\n');
    module.push_str(&export_clause(export_type));

    Ok(module)
}

/// Build the trailing `export ...` lines for the requested [`SvgrExportType`].
fn export_clause(export_type: SvgrExportType) -> String {
    match export_type {
        SvgrExportType::Named => "export { ReactComponent };\n".to_string(),
        SvgrExportType::Default => "export default ReactComponent;\n".to_string(),
        SvgrExportType::Both => {
            "export { ReactComponent };\nexport default ReactComponent;\n".to_string()
        }
    }
}

/// Indent every non-empty line of `block` by `spaces` spaces.
fn indent_block(block: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    block
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── SVG → JSX ───────────────────────────────────────────────────────────────

/// A parsed SVG/XML element tree, intentionally minimal — enough for the
/// common icon case (nested elements, attributes, self-closing tags). We do
/// NOT aim for full XML conformance; namespaced/CDATA/DOCTYPE handling beyond
/// the common case is `// TODO(#203 follow-up)`.
#[derive(Debug)]
struct Element {
    tag: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
    self_closing: bool,
}

#[derive(Debug)]
enum Node {
    Element(Element),
    Text(String),
}

/// Convert SVG source into a JSX expression string with `{...props}` and
/// `ref={ref}` spread onto the root `<svg>`.
fn svg_to_jsx(svg_src: &str) -> Result<String> {
    let mut parser = SvgParser::new(svg_src);
    // Skip prolog: XML declaration, comments, DOCTYPE before the root.
    parser.skip_prolog();
    let root = parser
        .parse_element()?
        .ok_or_else(|| anyhow!("svgr: no <svg> root element found in source"))?;

    if root.tag.to_ascii_lowercase() != "svg" {
        return Err(anyhow!(
            "svgr: expected root <svg> element, found <{}>",
            root.tag
        ));
    }

    Ok(render_root_svg(&root))
}

/// Render the root `<svg>` element, injecting the prop spread + ref forward.
fn render_root_svg(root: &Element) -> String {
    let mut out = String::new();
    out.push_str("<svg");
    for (name, value) in &root.attrs {
        out.push(' ');
        out.push_str(&render_attr(name, value));
    }
    // Forward incoming props after the static attrs so caller overrides win,
    // then the ref (callers commonly pass ref explicitly via forwardRef).
    out.push_str(" {...props}");
    out.push_str(" ref={ref}");

    if root.children.is_empty() {
        out.push_str(" />");
        return out;
    }
    out.push('>');
    out.push('\n');
    for child in &root.children {
        let rendered = render_node(child);
        if !rendered.trim().is_empty() {
            out.push_str(&indent_block(&rendered, 2));
            out.push('\n');
        }
    }
    out.push_str("</svg>");
    out
}

fn render_node(node: &Node) -> String {
    match node {
        Node::Text(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                String::new()
            } else {
                // JSX text: escape `{`, `}`, `<`, `>` by wrapping in an
                // expression string to stay safe for arbitrary content.
                format!("{{{:?}}}", trimmed)
            }
        }
        Node::Element(el) => render_element(el),
    }
}

fn render_element(el: &Element) -> String {
    let mut out = String::new();
    out.push('<');
    out.push_str(&el.tag);
    for (name, value) in &el.attrs {
        out.push(' ');
        out.push_str(&render_attr(name, value));
    }
    if el.self_closing || el.children.is_empty() {
        out.push_str(" />");
        return out;
    }
    out.push('>');
    out.push('\n');
    for child in &el.children {
        let rendered = render_node(child);
        if !rendered.trim().is_empty() {
            out.push_str(&indent_block(&rendered, 2));
            out.push('\n');
        }
    }
    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
    out
}

/// Render a single attribute as JSX: convert the name and value form.
fn render_attr(name: &str, value: &str) -> String {
    let jsx_name = convert_attr_name(name);
    if jsx_name == "style" {
        // `style="fill:red;stroke-width:2"` → `style={{fill:"red",strokeWidth:"2"}}`
        // Each `{{` / `}}` in the format string emits one literal brace, so we
        // need four to produce the JSX double-brace object expression.
        format!("style={{{{{}}}}}", style_string_to_object(value))
    } else {
        // Use Rust's debug-string escaping for a valid double-quoted JS string.
        format!("{}={:?}", jsx_name, value)
    }
}

/// Convert an SVG/HTML attribute name to its JSX/React equivalent.
///
/// Rules (common case):
/// - `class` → `className`
/// - `for` → `htmlFor`
/// - kebab-case → camelCase (`stroke-width` → `strokeWidth`,
///   `clip-rule` → `clipRule`)
/// - Namespaced attrs (`xmlns:xlink`, `xlink:href`) → React's colon-preserving
///   forms where React expects them. React 16+ accepts `xmlns:xlink` and
///   `xlink:href` as-is in JSX strings, so we preserve the colon form rather
///   than risk an incorrect camelCase mapping. `// TODO(#203 follow-up)`:
///   full xlink/xml namespace attribute table.
fn convert_attr_name(name: &str) -> String {
    match name {
        "class" => return "className".to_string(),
        "for" => return "htmlFor".to_string(),
        _ => {}
    }

    // Preserve namespaced attributes verbatim (React accepts `xlink:href`,
    // `xmlns:xlink` as string-literal JSX attribute names). Converting these
    // to camelCase would break SVG rendering, so leave them.
    if name.contains(':') {
        return name.to_string();
    }

    // `data-*` and `aria-*` are passed through unchanged (React convention).
    if name.starts_with("data-") || name.starts_with("aria-") {
        return name.to_string();
    }

    kebab_to_camel(name)
}

/// Convert `kebab-case` to `camelCase`. A name with no `-` is returned as-is.
fn kebab_to_camel(name: &str) -> String {
    if !name.contains('-') {
        return name.to_string();
    }
    let mut out = String::with_capacity(name.len());
    let mut upper_next = false;
    for ch in name.chars() {
        if ch == '-' {
            upper_next = true;
        } else if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Convert a CSS `style="..."` string into a JS object-literal body.
///
/// `"fill:red; stroke-width: 2px"` → `fill:"red",strokeWidth:"2px"`
/// (caller wraps the result in `style={{ ... }}`).
fn style_string_to_object(style: &str) -> String {
    let mut props: Vec<String> = Vec::new();
    for decl in style.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        let Some((prop, val)) = decl.split_once(':') else {
            continue;
        };
        let prop = prop.trim();
        let val = val.trim();
        if prop.is_empty() {
            continue;
        }
        // CSS custom properties (`--foo`) must keep their literal name as a
        // quoted key. `// TODO(#203 follow-up)`: numeric unitless values could
        // be emitted as numbers, but quoting strings is always valid.
        let key = if prop.starts_with("--") {
            format!("{:?}", prop)
        } else {
            css_prop_to_camel(prop)
        };
        props.push(format!("{}:{:?}", key, val));
    }
    props.join(",")
}

/// Convert a CSS property name to its React style-object key (`camelCase`).
/// Vendor-prefixed `-webkit-*` becomes `WebkitFoo` (leading capital) per React.
fn css_prop_to_camel(prop: &str) -> String {
    if prop.starts_with('-') {
        // `-webkit-transform` → `WebkitTransform`
        let camel = kebab_to_camel(prop.trim_start_matches('-'));
        let mut chars = camel.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => camel,
        }
    } else {
        kebab_to_camel(prop)
    }
}

// ─── Minimal SVG/XML parser ────────────────────────────────────────────────

struct SvgParser<'a> {
    bytes: &'a [u8],
    src: &'a str,
    pos: usize,
}

impl<'a> SvgParser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            bytes: src.as_bytes(),
            src,
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    /// Skip `<?xml ... ?>`, `<!-- ... -->`, and `<!DOCTYPE ...>` before root.
    fn skip_prolog(&mut self) {
        loop {
            self.skip_whitespace();
            if self.starts_with("<?") {
                if let Some(end) = self.src[self.pos..].find("?>") {
                    self.pos += end + 2;
                    continue;
                }
            } else if self.starts_with("<!--") {
                if let Some(end) = self.src[self.pos..].find("-->") {
                    self.pos += end + 3;
                    continue;
                }
            } else if self.starts_with("<!") {
                // DOCTYPE or other declaration — skip to the closing `>`.
                if let Some(end) = self.src[self.pos..].find('>') {
                    self.pos += end + 1;
                    continue;
                }
            }
            break;
        }
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.src[self.pos..].starts_with(prefix)
    }

    /// Parse the next element (recursively). Returns `None` at EOF.
    fn parse_element(&mut self) -> Result<Option<Element>> {
        self.skip_whitespace();
        if self.pos >= self.bytes.len() {
            return Ok(None);
        }
        // Skip inline comments between elements.
        if self.starts_with("<!--") {
            if let Some(end) = self.src[self.pos..].find("-->") {
                self.pos += end + 3;
                return self.parse_element();
            }
            return Err(anyhow!("svgr: unterminated comment"));
        }
        if !self.starts_with("<") {
            return Err(anyhow!("svgr: expected '<' to start an element"));
        }
        self.pos += 1; // consume '<'

        let tag = self.read_name();
        if tag.is_empty() {
            return Err(anyhow!("svgr: empty tag name"));
        }

        let mut attrs = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                return Err(anyhow!("svgr: unexpected EOF inside <{}>", tag));
            }
            if self.starts_with("/>") {
                self.pos += 2;
                return Ok(Some(Element {
                    tag,
                    attrs,
                    children: Vec::new(),
                    self_closing: true,
                }));
            }
            if self.starts_with(">") {
                self.pos += 1;
                break;
            }
            let (name, value) = self.read_attr()?;
            attrs.push((name, value));
        }

        // Parse children until the matching close tag.
        let mut children = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                return Err(anyhow!("svgr: unexpected EOF, <{}> not closed", tag));
            }
            if self.starts_with("</") {
                self.pos += 2;
                let close = self.read_name();
                self.skip_whitespace();
                if self.starts_with(">") {
                    self.pos += 1;
                }
                if !close.eq_ignore_ascii_case(&tag) {
                    return Err(anyhow!(
                        "svgr: mismatched close tag </{}> for <{}>",
                        close,
                        tag
                    ));
                }
                break;
            }
            if self.starts_with("<!--") {
                if let Some(end) = self.src[self.pos..].find("-->") {
                    self.pos += end + 3;
                    continue;
                }
                return Err(anyhow!("svgr: unterminated comment in <{}>", tag));
            }
            if self.starts_with("<") {
                if let Some(child) = self.parse_element()? {
                    children.push(Node::Element(child));
                }
            } else {
                let text = self.read_text();
                if !text.trim().is_empty() {
                    children.push(Node::Text(text));
                }
            }
        }

        Ok(Some(Element {
            tag,
            attrs,
            children,
            self_closing: false,
        }))
    }

    /// Read a tag or attribute name (letters, digits, `-`, `_`, `:`, `.`).
    fn read_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let c = self.bytes[self.pos] as char;
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.src[start..self.pos].to_string()
    }

    /// Read one `name="value"` (or `name='value'`) attribute.
    fn read_attr(&mut self) -> Result<(String, String)> {
        let name = self.read_name();
        if name.is_empty() {
            return Err(anyhow!("svgr: malformed attribute near byte {}", self.pos));
        }
        self.skip_whitespace();
        if !self.starts_with("=") {
            // Boolean / valueless attribute (rare in SVG). Treat value as the
            // attribute name itself → JSX `name={true}` equivalent rendered as
            // an empty string value here; keep it simple.
            return Ok((name, String::new()));
        }
        self.pos += 1; // consume '='
        self.skip_whitespace();
        if self.pos >= self.bytes.len() {
            return Err(anyhow!("svgr: EOF after '=' for attribute {}", name));
        }
        let quote = self.bytes[self.pos] as char;
        if quote != '"' && quote != '\'' {
            return Err(anyhow!(
                "svgr: unquoted attribute value for {} (got {:?})",
                name,
                quote
            ));
        }
        self.pos += 1; // consume opening quote
        let start = self.pos;
        while self.pos < self.bytes.len() && (self.bytes[self.pos] as char) != quote {
            self.pos += 1;
        }
        let value = self.src[start..self.pos].to_string();
        if self.pos < self.bytes.len() {
            self.pos += 1; // consume closing quote
        }
        // Decode the handful of XML entities that commonly appear in attribute
        // values. `// TODO(#203 follow-up)`: full entity table / numeric refs.
        let value = decode_basic_entities(&value);
        Ok((name, value))
    }

    /// Read raw text until the next `<`.
    fn read_text(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.bytes.len() && (self.bytes[self.pos] as char) != '<' {
            self.pos += 1;
        }
        decode_basic_entities(&self.src[start..self.pos])
    }
}

/// Decode the five predefined XML entities (common case only).
fn decode_basic_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_export_type_is_named() {
        assert_eq!(SvgrExportType::default(), SvgrExportType::Named);
    }

    #[test]
    fn class_becomes_classname() {
        assert_eq!(convert_attr_name("class"), "className");
    }

    #[test]
    fn kebab_attr_becomes_camel() {
        assert_eq!(convert_attr_name("stroke-width"), "strokeWidth");
        assert_eq!(convert_attr_name("clip-rule"), "clipRule");
        assert_eq!(convert_attr_name("fill-opacity"), "fillOpacity");
    }

    #[test]
    fn data_and_aria_pass_through() {
        assert_eq!(convert_attr_name("data-foo"), "data-foo");
        assert_eq!(convert_attr_name("aria-label"), "aria-label");
    }

    #[test]
    fn namespaced_attr_preserved() {
        assert_eq!(convert_attr_name("xlink:href"), "xlink:href");
        assert_eq!(convert_attr_name("xmlns:xlink"), "xmlns:xlink");
    }

    #[test]
    fn style_string_converts_to_object_body() {
        let body = style_string_to_object("fill:red; stroke-width: 2px");
        assert!(body.contains("fill:\"red\""), "body: {body}");
        assert!(body.contains("strokeWidth:\"2px\""), "body: {body}");
    }

    #[test]
    fn simple_svg_path_emits_named_module() {
        let svg = r#"<svg viewBox="0 0 24 24"><path d="M0 0h24v24H0z"/></svg>"#;
        let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
        assert!(module.contains("import * as React from \"react\""));
        assert!(module.contains("React.forwardRef"));
        assert!(module.contains("<svg"));
        assert!(module.contains("<path"));
        assert!(module.contains("{...props}"));
        assert!(module.contains("ref={ref}"));
        assert!(module.contains("export { ReactComponent };"));
        assert!(!module.contains("export default"));
    }

    #[test]
    fn default_export_emits_default_only() {
        let svg = r#"<svg><path d="M0 0"/></svg>"#;
        let module = transform_svg_to_component(svg, SvgrExportType::Default).unwrap();
        assert!(module.contains("export default ReactComponent;"));
        assert!(!module.contains("export { ReactComponent }"));
    }

    #[test]
    fn both_export_emits_named_and_default() {
        let svg = r#"<svg><path d="M0 0"/></svg>"#;
        let module = transform_svg_to_component(svg, SvgrExportType::Both).unwrap();
        assert!(module.contains("export { ReactComponent };"));
        assert!(module.contains("export default ReactComponent;"));
    }

    #[test]
    fn class_attr_converted_in_output() {
        let svg = r#"<svg class="icon"><path class="fill" d="M0 0"/></svg>"#;
        let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
        assert!(module.contains("className=\"icon\""), "module:\n{module}");
        assert!(module.contains("className=\"fill\""), "module:\n{module}");
        assert!(
            !module.contains(" class="),
            "should not emit raw class= :\n{module}"
        );
    }

    #[test]
    fn style_attr_converted_in_output() {
        let svg = r#"<svg style="fill:red;stroke-width:2"><path d="M0 0"/></svg>"#;
        let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
        assert!(module.contains("style={{"), "module:\n{module}");
        assert!(module.contains("strokeWidth:\"2\""), "module:\n{module}");
    }

    #[test]
    fn nested_elements_are_rendered() {
        let svg = r#"<svg><g fill="none"><path d="M0 0"/><circle cx="1" cy="2" r="3"/></g></svg>"#;
        let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
        assert!(module.contains("<g"), "module:\n{module}");
        assert!(module.contains("<path"), "module:\n{module}");
        assert!(module.contains("<circle"), "module:\n{module}");
        assert!(module.contains("</g>"), "module:\n{module}");
    }

    #[test]
    fn skips_xml_prolog_and_comments() {
        let svg = "<?xml version=\"1.0\"?>\n<!-- a comment -->\n<svg><path d=\"M0 0\"/></svg>";
        let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
        assert!(module.contains("<svg"));
        assert!(module.contains("<path"));
    }

    #[test]
    fn non_svg_root_is_an_error() {
        let err = transform_svg_to_component("<div></div>", SvgrExportType::Named).unwrap_err();
        assert!(format!("{err}").contains("svg"), "err: {err}");
    }

    #[test]
    fn empty_source_is_an_error() {
        assert!(transform_svg_to_component("", SvgrExportType::Named).is_err());
    }

    #[test]
    fn css_custom_property_in_style_is_quoted_key() {
        let body = style_string_to_object("--my-var: 4px; color: red");
        assert!(body.contains("\"--my-var\":\"4px\""), "body: {body}");
        assert!(body.contains("color:\"red\""), "body: {body}");
    }

    #[test]
    fn vendor_prefixed_style_prop_capitalized() {
        let body = style_string_to_object("-webkit-transform: scale(1)");
        assert!(body.contains("WebkitTransform"), "body: {body}");
    }
}
// </HANDWRITE>

//! Rust public export scanner for Python-oriented code generation.
//!
//! Parses Rust source files using tree-sitter to extract public items:
//! - `pub struct` (data or stateful)
//! - `pub enum`
//! - `pub fn` / `pub async fn`
//!
//! Convention: public Rust items can be discovered and projected into Python-facing generators.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Node, Parser};

/// Scanned Rust exports from a crate
#[derive(Debug, Default)]
pub struct RustExports {
    /// Public structs
    pub structs: Vec<RustStruct>,
    /// Public enums
    pub enums: Vec<RustEnum>,
    /// Public functions
    pub functions: Vec<RustFunction>,
}

impl RustExports {
    /// Merge another RustExports into this one
    pub fn merge(&mut self, other: RustExports) {
        self.structs.extend(other.structs);
        self.enums.extend(other.enums);
        self.functions.extend(other.functions);
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.structs.is_empty() && self.enums.is_empty() && self.functions.is_empty()
    }
}

/// Kind of struct (determines wrapper strategy)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructKind {
    /// Data struct: all fields simple types, no methods.
    Data,
    /// Stateful struct: has methods taking `&self` or `&mut self` → `Arc<RwLock<T>>`
    Stateful,
    /// Has generic type parameters → mark as TODO
    Generic,
}

/// Public struct with its fields and methods
#[derive(Debug)]
pub struct RustStruct {
    pub name: String,
    pub kind: StructKind,
    pub fields: Vec<RustField>,
    pub methods: Vec<RustMethod>,
    pub docstring: Option<String>,
    pub derives: Vec<String>,
    /// Whether struct derives Clone
    pub has_clone: bool,
    /// Whether struct derives Default
    pub has_default: bool,
}

/// Struct field
#[derive(Debug)]
pub struct RustField {
    pub name: String,
    pub ty: String,
    pub is_public: bool,
    pub docstring: Option<String>,
}

/// Struct method (from impl block)
#[derive(Debug, Clone)]
pub struct RustMethod {
    pub name: String,
    pub params: Vec<RustParam>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub is_static: bool,
    pub takes_self: bool,
    pub takes_mut_self: bool,
    pub docstring: Option<String>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct RustParam {
    pub name: String,
    pub ty: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Public enum
#[derive(Debug)]
pub struct RustEnum {
    pub name: String,
    pub variants: Vec<RustEnumVariant>,
    pub docstring: Option<String>,
    /// Whether all variants are unit variants (no data)
    pub is_simple: bool,
}

/// Enum variant
#[derive(Debug)]
pub struct RustEnumVariant {
    pub name: String,
    /// None for unit variants, Some for tuple/struct variants
    pub data: Option<String>,
    pub docstring: Option<String>,
}

/// Public function
#[derive(Debug)]
pub struct RustFunction {
    pub name: String,
    pub params: Vec<RustParam>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub docstring: Option<String>,
}

/// Scanner for Rust public exports
pub struct RustScanner {
    parser: Parser,
}

impl RustScanner {
    /// Create a new scanner
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .context("Failed to set Rust language for parser")?;
        Ok(Self { parser })
    }

    /// Scan a Rust crate directory for public exports
    pub fn scan_crate(&mut self, crate_path: &Path) -> Result<RustExports> {
        let src_dir = if crate_path.join("src").exists() {
            crate_path.join("src")
        } else {
            crate_path.to_path_buf()
        };

        let lib_rs = src_dir.join("lib.rs");
        if !lib_rs.exists() {
            anyhow::bail!("No lib.rs found in {}", src_dir.display());
        }

        let mut exports = RustExports::default();
        let mut impl_methods: HashMap<String, Vec<RustMethod>> = HashMap::new();

        // First pass: scan lib.rs for pub items and pub mod declarations
        let content = std::fs::read_to_string(&lib_rs)
            .with_context(|| format!("Failed to read {}", lib_rs.display()))?;

        let (lib_exports, lib_impls) = self.scan_file(&content)?;
        for (name, methods) in lib_impls {
            impl_methods.entry(name).or_default().extend(methods);
        }
        exports.merge(lib_exports);

        // Find and scan pub mod files
        let pub_mods = self.find_pub_mods(&content)?;
        for mod_name in pub_mods {
            let mod_file = src_dir.join(format!("{}.rs", mod_name));
            let mod_dir = src_dir.join(&mod_name).join("mod.rs");

            let mod_path = if mod_file.exists() {
                mod_file
            } else if mod_dir.exists() {
                mod_dir
            } else {
                continue;
            };

            if let Ok(mod_content) = std::fs::read_to_string(&mod_path) {
                let (mod_exports, mod_impls) = self.scan_file(&mod_content)?;
                for (name, methods) in mod_impls {
                    impl_methods.entry(name).or_default().extend(methods);
                }
                exports.merge(mod_exports);
            }
        }

        // Attach methods to structs
        for s in &mut exports.structs {
            if let Some(methods) = impl_methods.remove(&s.name) {
                s.methods = methods;
                // If has methods that take &self or &mut self, mark as stateful
                if s.methods.iter().any(|m| m.takes_self || m.takes_mut_self) {
                    s.kind = StructKind::Stateful;
                }
            }
        }

        Ok(exports)
    }

    /// Scan a single file for public items
    pub fn scan_file(
        &mut self,
        content: &str,
    ) -> Result<(RustExports, HashMap<String, Vec<RustMethod>>)> {
        let tree = self
            .parser
            .parse(content, None)
            .context("Failed to parse Rust source")?;

        let root = tree.root_node();
        let mut exports = RustExports::default();
        let mut impl_methods: HashMap<String, Vec<RustMethod>> = HashMap::new();

        self.visit_node(root, content, &mut exports, &mut impl_methods)?;

        Ok((exports, impl_methods))
    }

    /// Find `pub mod` declarations in a file
    fn find_pub_mods(&mut self, content: &str) -> Result<Vec<String>> {
        let tree = self
            .parser
            .parse(content, None)
            .context("Failed to parse Rust source")?;

        let root = tree.root_node();
        let mut mods = Vec::new();

        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "mod_item" {
                if self.has_pub_visibility(child, content) {
                    if let Some(name) = self.get_child_text(child, "identifier", content) {
                        // Only include mods without body (external files)
                        if child.child_by_field_name("body").is_none() {
                            mods.push(name);
                        }
                    }
                }
            }
        }

        Ok(mods)
    }

    /// Visit AST node and extract public items
    fn visit_node(
        &self,
        node: Node,
        source: &str,
        exports: &mut RustExports,
        impl_methods: &mut HashMap<String, Vec<RustMethod>>,
    ) -> Result<()> {
        match node.kind() {
            "struct_item" => {
                if self.has_pub_visibility(node, source) {
                    if let Some(s) = self.extract_struct(node, source)? {
                        exports.structs.push(s);
                    }
                }
            }
            "enum_item" => {
                if self.has_pub_visibility(node, source) {
                    if let Some(e) = self.extract_enum(node, source)? {
                        exports.enums.push(e);
                    }
                }
            }
            "function_item" => {
                if self.has_pub_visibility(node, source) {
                    if let Some(f) = self.extract_function(node, source)? {
                        exports.functions.push(f);
                    }
                }
            }
            "impl_item" => {
                // Extract methods from impl blocks
                if let Some((type_name, methods)) = self.extract_impl_methods(node, source)? {
                    impl_methods.entry(type_name).or_default().extend(methods);
                }
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child, source, exports, impl_methods)?;
        }

        Ok(())
    }

    /// Check if a node has `pub` visibility
    fn has_pub_visibility(&self, node: Node, source: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let text = self.node_text(child, source);
                return text.starts_with("pub");
            }
        }
        false
    }

    /// Extract struct definition
    fn extract_struct(&self, node: Node, source: &str) -> Result<Option<RustStruct>> {
        let name = match self.get_child_text(node, "type_identifier", source) {
            Some(n) => n,
            None => return Ok(None),
        };

        // Check for generic parameters
        let has_generics = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "type_parameters");

        let kind = if has_generics {
            StructKind::Generic
        } else {
            StructKind::Data // Will be updated if methods are found
        };

        let docstring = self.extract_docstring(node, source);
        let derives = self.extract_derives(node, source);
        let has_clone = derives.iter().any(|d| d == "Clone");
        let has_default = derives.iter().any(|d| d == "Default");

        // Extract fields
        let mut fields = Vec::new();
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "field_declaration" {
                    if let Some(field) = self.extract_field(child, source)? {
                        fields.push(field);
                    }
                }
            }
        }

        Ok(Some(RustStruct {
            name,
            kind,
            fields,
            methods: Vec::new(), // Filled in later
            docstring,
            derives,
            has_clone,
            has_default,
        }))
    }

    /// Extract struct field
    fn extract_field(&self, node: Node, source: &str) -> Result<Option<RustField>> {
        let name = match self.get_child_text(node, "field_identifier", source) {
            Some(n) => n,
            None => return Ok(None),
        };

        let ty = node
            .child_by_field_name("type")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();

        let is_public = self.has_pub_visibility(node, source);
        let docstring = self.extract_docstring(node, source);

        Ok(Some(RustField {
            name,
            ty,
            is_public,
            docstring,
        }))
    }

    /// Extract enum definition
    fn extract_enum(&self, node: Node, source: &str) -> Result<Option<RustEnum>> {
        let name = match self.get_child_text(node, "type_identifier", source) {
            Some(n) => n,
            None => return Ok(None),
        };

        let docstring = self.extract_docstring(node, source);

        // Extract variants
        let mut variants = Vec::new();
        let mut is_simple = true;

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "enum_variant" {
                    if let Some(variant) = self.extract_enum_variant(child, source)? {
                        if variant.data.is_some() {
                            is_simple = false;
                        }
                        variants.push(variant);
                    }
                }
            }
        }

        Ok(Some(RustEnum {
            name,
            variants,
            docstring,
            is_simple,
        }))
    }

    /// Extract enum variant
    fn extract_enum_variant(&self, node: Node, source: &str) -> Result<Option<RustEnumVariant>> {
        let name = match self.get_child_text(node, "identifier", source) {
            Some(n) => n,
            None => return Ok(None),
        };

        let docstring = self.extract_docstring(node, source);

        // Check for tuple or struct variant data
        let data = node
            .children(&mut node.walk())
            .find(|c| {
                c.kind() == "ordered_field_declaration_list" || c.kind() == "field_declaration_list"
            })
            .map(|c| self.node_text(c, source));

        Ok(Some(RustEnumVariant {
            name,
            data,
            docstring,
        }))
    }

    /// Extract function definition
    fn extract_function(&self, node: Node, source: &str) -> Result<Option<RustFunction>> {
        let name = match self.get_child_text(node, "identifier", source) {
            Some(n) => n,
            None => return Ok(None),
        };

        let docstring = self.extract_docstring(node, source);
        let is_async = self.node_text(node, source).contains("async fn");

        let params = self.extract_parameters(node, source)?;
        let return_type = self.extract_return_type(node, source);

        Ok(Some(RustFunction {
            name,
            params,
            return_type,
            is_async,
            docstring,
        }))
    }

    /// Extract methods from impl block
    fn extract_impl_methods(
        &self,
        node: Node,
        source: &str,
    ) -> Result<Option<(String, Vec<RustMethod>)>> {
        // Get the type being implemented
        let type_name = node
            .child_by_field_name("type")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();

        if type_name.is_empty() {
            return Ok(None);
        }

        // Skip trait implementations for now
        if node.children(&mut node.walk()).any(|c| c.kind() == "trait") {
            return Ok(None);
        }

        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_item" {
                    if self.has_pub_visibility(child, source) {
                        if let Some(method) = self.extract_method(child, source)? {
                            methods.push(method);
                        }
                    }
                }
            }
        }

        if methods.is_empty() {
            return Ok(None);
        }

        Ok(Some((type_name, methods)))
    }

    /// Extract method from function item
    fn extract_method(&self, node: Node, source: &str) -> Result<Option<RustMethod>> {
        let name = match self.get_child_text(node, "identifier", source) {
            Some(n) => n,
            None => return Ok(None),
        };

        let docstring = self.extract_docstring(node, source);
        let is_async = self.node_text(node, source).contains("async fn");

        // Check for self parameter
        let (takes_self, takes_mut_self, is_static) = self.check_self_param(node, source);

        let params = self.extract_parameters(node, source)?;
        let return_type = self.extract_return_type(node, source);

        Ok(Some(RustMethod {
            name,
            params,
            return_type,
            is_async,
            is_static,
            takes_self,
            takes_mut_self,
            docstring,
        }))
    }

    /// Check self parameter type
    fn check_self_param(&self, node: Node, source: &str) -> (bool, bool, bool) {
        if let Some(params) = node.child_by_field_name("parameters") {
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                if child.kind() == "self_parameter" {
                    let text = self.node_text(child, source);
                    if text.contains("&mut self") {
                        return (false, true, false);
                    } else if text.contains("&self") || text == "self" {
                        return (true, false, false);
                    }
                }
            }
        }
        (false, false, true)
    }

    /// Extract function parameters
    fn extract_parameters(&self, node: Node, source: &str) -> Result<Vec<RustParam>> {
        let mut params = Vec::new();

        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    let name = child
                        .child_by_field_name("pattern")
                        .map(|n| self.node_text(n, source))
                        .unwrap_or_default();

                    let ty = child
                        .child_by_field_name("type")
                        .map(|n| self.node_text(n, source))
                        .unwrap_or_default();

                    // Skip Python-oriented internal params
                    if name == "py" || name == "_py" || ty.contains("Python") {
                        continue;
                    }

                    let is_optional = ty.starts_with("Option<");

                    params.push(RustParam {
                        name,
                        ty,
                        is_optional,
                        default_value: None,
                    });
                }
            }
        }

        Ok(params)
    }

    /// Extract return type
    fn extract_return_type(&self, node: Node, source: &str) -> Option<String> {
        node.child_by_field_name("return_type")
            .map(|n| self.node_text(n, source))
    }

    /// Extract #[derive(...)] attributes
    fn extract_derives(&self, node: Node, source: &str) -> Vec<String> {
        let mut derives = Vec::new();

        // Look at preceding siblings for attributes
        let mut prev = node.prev_sibling();
        while let Some(sibling) = prev {
            if sibling.kind() == "attribute_item" {
                let text = self.node_text(sibling, source);
                if text.contains("derive") {
                    // Extract derive contents: #[derive(Clone, Debug)] -> ["Clone", "Debug"]
                    if let Some(start) = text.find("derive(") {
                        let rest = &text[start + 7..];
                        if let Some(end) = rest.find(')') {
                            let inner = &rest[..end];
                            for item in inner.split(',') {
                                derives.push(item.trim().to_string());
                            }
                        }
                    }
                }
                prev = sibling.prev_sibling();
            } else {
                break;
            }
        }

        derives
    }

    /// Extract docstring from preceding comments
    fn extract_docstring(&self, node: Node, source: &str) -> Option<String> {
        let mut doc_lines = Vec::new();

        let start_byte = node.start_byte();
        let preceding = &source[..start_byte];

        for line in preceding.lines().rev() {
            let trimmed = line.trim();
            if trimmed.starts_with("///") {
                let doc = trimmed.trim_start_matches("///").trim();
                doc_lines.insert(0, doc.to_string());
            } else if trimmed.starts_with("#[doc") {
                // Handle #[doc = "..."]
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed.rfind('"') {
                        if start < end {
                            let doc = &trimmed[start + 1..end];
                            doc_lines.insert(0, doc.to_string());
                        }
                    }
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join("\n"))
        }
    }

    /// Get text from a child node by kind
    fn get_child_text(&self, node: Node, kind: &str, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == kind {
                return Some(self.node_text(child, source));
            }
        }
        None
    }

    /// Get text from a node
    fn node_text(&self, node: Node, source: &str) -> String {
        source[node.byte_range()].to_string()
    }
}

impl Default for RustScanner {
    fn default() -> Self {
        Self::new().expect("Failed to create RustScanner")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_pub_struct() {
        let mut scanner = RustScanner::new().unwrap();
        let source = r#"
/// A user in the system.
#[derive(Clone, Debug)]
pub struct User {
    pub name: String,
    pub age: u32,
    password: String,
}
"#;
        let (exports, _) = scanner.scan_file(source).unwrap();

        assert_eq!(exports.structs.len(), 1);
        let user = &exports.structs[0];
        assert_eq!(user.name, "User");
        assert!(user
            .docstring
            .as_ref()
            .unwrap()
            .contains("user in the system"));
        assert!(user.has_clone);
        assert_eq!(user.fields.len(), 3);

        // Check field visibility
        assert!(user.fields[0].is_public); // name
        assert!(user.fields[1].is_public); // age
        assert!(!user.fields[2].is_public); // password
    }

    #[test]
    fn test_scan_pub_enum() {
        let mut scanner = RustScanner::new().unwrap();
        let source = r#"
/// Loading strategy for relationships.
pub enum LoadingStrategy {
    /// Load lazily on access
    Lazy,
    /// Load eagerly with parent
    Eager,
    /// Use subquery for loading
    Subquery,
}
"#;
        let (exports, _) = scanner.scan_file(source).unwrap();

        assert_eq!(exports.enums.len(), 1);
        let e = &exports.enums[0];
        assert_eq!(e.name, "LoadingStrategy");
        assert!(e.is_simple);
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name, "Lazy");
    }

    #[test]
    fn test_scan_pub_function() {
        let mut scanner = RustScanner::new().unwrap();
        let source = r#"
/// Connect to the database.
pub async fn connect(url: &str) -> Result<Connection> {
    todo!()
}
"#;
        let (exports, _) = scanner.scan_file(source).unwrap();

        assert_eq!(exports.functions.len(), 1);
        let f = &exports.functions[0];
        assert_eq!(f.name, "connect");
        assert!(f.is_async);
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].name, "url");
    }

    #[test]
    fn test_scan_impl_methods() {
        let mut scanner = RustScanner::new().unwrap();
        let source = r#"
pub struct QueryBuilder {
    table: String,
}

impl QueryBuilder {
    /// Create a new query builder.
    pub fn new(table: &str) -> Self {
        todo!()
    }

    /// Add SELECT columns.
    pub fn select(&self, columns: Vec<String>) -> Self {
        todo!()
    }

    /// Build the query.
    pub async fn execute(&mut self) -> Result<Vec<Row>> {
        todo!()
    }
}
"#;
        let (mut exports, impl_methods) = scanner.scan_file(source).unwrap();

        // Attach methods to struct
        for s in &mut exports.structs {
            if let Some(methods) = impl_methods.get(&s.name) {
                s.methods = methods.clone();
            }
        }

        assert_eq!(exports.structs.len(), 1);
        let qb = &exports.structs[0];
        assert_eq!(qb.name, "QueryBuilder");
        assert_eq!(qb.methods.len(), 3);

        // Check new() is static
        let new_method = qb.methods.iter().find(|m| m.name == "new").unwrap();
        assert!(new_method.is_static);

        // Check select() takes &self
        let select_method = qb.methods.iter().find(|m| m.name == "select").unwrap();
        assert!(select_method.takes_self);

        // Check execute() is async and takes &mut self
        let execute_method = qb.methods.iter().find(|m| m.name == "execute").unwrap();
        assert!(execute_method.is_async);
        assert!(execute_method.takes_mut_self);
    }

    #[test]
    fn test_generic_struct_detection() {
        let mut scanner = RustScanner::new().unwrap();
        let source = r#"
pub struct Container<T> {
    data: T,
}
"#;
        let (exports, _) = scanner.scan_file(source).unwrap();

        assert_eq!(exports.structs.len(), 1);
        assert_eq!(exports.structs[0].kind, StructKind::Generic);
    }

    #[test]
    fn test_non_simple_enum() {
        let mut scanner = RustScanner::new().unwrap();
        let source = r#"
pub enum Value {
    Int(i64),
    String(String),
    List(Vec<Value>),
}
"#;
        let (exports, _) = scanner.scan_file(source).unwrap();

        assert_eq!(exports.enums.len(), 1);
        assert!(!exports.enums[0].is_simple);
    }
}

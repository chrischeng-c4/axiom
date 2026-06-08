//! Multi-language parser using tree-sitter

use crate::lens_error::{ArgusError, Result};
use std::path::Path;
use tree_sitter::{Parser, Tree};

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Python,
    TypeScript,
    Rust,
    JavaScript,
    Go,
    Html,
    Css,
    Dockerfile,
    Hcl,
    Yaml,
    Markdown,
    Mdx,
    Mermaid,
    Toml,
    Sql,
    Proto,
    GraphQL,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::Rust => "rust",
            Language::JavaScript => "javascript",
            Language::Go => "go",
            Language::Html => "html",
            Language::Css => "css",
            Language::Dockerfile => "dockerfile",
            Language::Hcl => "hcl",
            Language::Yaml => "yaml",
            Language::Markdown => "markdown",
            Language::Mdx => "mdx",
            Language::Mermaid => "mermaid",
            Language::Toml => "toml",
            Language::Sql => "sql",
            Language::Proto => "proto",
            Language::GraphQL => "graphql",
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Python => &["py", "pyi"],
            Language::TypeScript => &["ts", "tsx"],
            Language::Rust => &["rs"],
            Language::JavaScript => &["js", "jsx"],
            Language::Go => &["go"],
            Language::Html => &["html", "htm"],
            Language::Css => &["css"],
            Language::Dockerfile => &[],
            Language::Hcl => &["tf", "tfvars"],
            Language::Yaml => &["yaml", "yml"],
            Language::Markdown => &["md", "markdown"],
            Language::Mdx => &["mdx"],
            Language::Mermaid => &["mmd", "mermaid"],
            Language::Toml => &["toml"],
            Language::Sql => &["sql"],
            Language::Proto => &["proto"],
            Language::GraphQL => &["graphql", "gql"],
        }
    }
}

/// Multi-language parser
pub struct MultiParser {
    python_parser: Parser,
    typescript_parser: Parser,
    rust_parser: Parser,
    javascript_parser: Parser,
    go_parser: Parser,
    html_parser: Parser,
    css_parser: Parser,
    hcl_parser: Parser,
    yaml_parser: Parser,
    // R3: AST grammars for SQL, Protobuf, GraphQL, TOML
    sql_parser: Parser,
    proto_parser: Parser,
    graphql_parser: Parser,
    toml_parser: Parser,
}

impl MultiParser {
    pub fn new() -> Result<Self> {
        let mut python_parser = Parser::new();
        python_parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load Python grammar: {}", e)))?;

        let mut typescript_parser = Parser::new();
        typescript_parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load TypeScript grammar: {}", e)))?;

        let mut rust_parser = Parser::new();
        rust_parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load Rust grammar: {}", e)))?;

        let mut javascript_parser = Parser::new();
        javascript_parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load JavaScript grammar: {}", e)))?;

        let mut go_parser = Parser::new();
        go_parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load Go grammar: {}", e)))?;

        let mut html_parser = Parser::new();
        html_parser
            .set_language(&tree_sitter_html::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load HTML grammar: {}", e)))?;

        let mut css_parser = Parser::new();
        css_parser
            .set_language(&tree_sitter_css::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load CSS grammar: {}", e)))?;

        let mut hcl_parser = Parser::new();
        hcl_parser
            .set_language(&tree_sitter_hcl::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load HCL grammar: {}", e)))?;

        let mut yaml_parser = Parser::new();
        yaml_parser
            .set_language(&tree_sitter_yaml::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load YAML grammar: {}", e)))?;

        // R3: AST grammars for SQL, Protobuf, GraphQL, TOML
        // Crate names: tree-sitter-sequel, tree-sitter-proto,
        //              tree-sitter-graphql, tree-sitter-toml-ng
        let mut sql_parser = Parser::new();
        sql_parser
            .set_language(&tree_sitter_sequel::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load SQL grammar: {}", e)))?;

        let mut proto_parser = Parser::new();
        proto_parser
            .set_language(&tree_sitter_proto::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load Protobuf grammar: {}", e)))?;

        let mut graphql_parser = Parser::new();
        graphql_parser
            .set_language(&tree_sitter_graphql::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load GraphQL grammar: {}", e)))?;

        let mut toml_parser = Parser::new();
        toml_parser
            .set_language(&tree_sitter_toml_ng::LANGUAGE.into())
            .map_err(|e| ArgusError::parser(format!("Failed to load TOML grammar: {}", e)))?;

        Ok(Self {
            python_parser,
            typescript_parser,
            rust_parser,
            javascript_parser,
            go_parser,
            html_parser,
            css_parser,
            hcl_parser,
            yaml_parser,
            sql_parser,
            proto_parser,
            graphql_parser,
            toml_parser,
        })
    }

    /// Detect language from file path (extension + filename)
    pub fn detect_language(path: &Path) -> Option<Language> {
        // Check filename first (for Dockerfile)
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "Dockerfile"
                || name.starts_with("Dockerfile.")
                || name.ends_with(".dockerfile")
            {
                return Some(Language::Dockerfile);
            }
        }

        let ext = path.extension()?.to_str()?;
        match ext {
            "py" | "pyi" => Some(Language::Python),
            "ts" | "tsx" => Some(Language::TypeScript),
            "rs" => Some(Language::Rust),
            "js" | "jsx" => Some(Language::JavaScript),
            "go" => Some(Language::Go),
            "html" | "htm" => Some(Language::Html),
            "css" => Some(Language::Css),
            "tf" | "tfvars" => Some(Language::Hcl),
            "yaml" | "yml" => Some(Language::Yaml),
            "md" | "markdown" => Some(Language::Markdown),
            "mdx" => Some(Language::Mdx),
            "mmd" | "mermaid" => Some(Language::Mermaid),
            "toml" => Some(Language::Toml),
            "sql" => Some(Language::Sql),
            "proto" => Some(Language::Proto),
            "graphql" | "gql" => Some(Language::GraphQL),
            _ => None,
        }
    }

    /// Parse source code
    pub fn parse(&mut self, source: &str, language: Language) -> Option<ParsedFile> {
        let parser = match language {
            Language::Python => &mut self.python_parser,
            Language::TypeScript => &mut self.typescript_parser,
            Language::Rust => &mut self.rust_parser,
            Language::JavaScript => &mut self.javascript_parser,
            Language::Go => &mut self.go_parser,
            Language::Html => &mut self.html_parser,
            Language::Css => &mut self.css_parser,
            Language::Dockerfile => return None, // line-based parsing, no tree-sitter
            Language::Hcl => &mut self.hcl_parser,
            Language::Yaml => &mut self.yaml_parser,
            Language::Markdown | Language::Mdx | Language::Mermaid => return None, // line-based
            // R3: AST grammars for SQL, Protobuf, GraphQL, TOML
            Language::Sql => &mut self.sql_parser,
            Language::Proto => &mut self.proto_parser,
            Language::GraphQL => &mut self.graphql_parser,
            Language::Toml => &mut self.toml_parser,
        };

        let tree = parser.parse(source, None)?;
        let has_errors = tree.root_node().has_error();

        Some(ParsedFile {
            source: source.to_string(),
            tree,
            language,
            has_errors,
            is_line_based: false,
        })
    }
}

/// Information about a parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Start byte offset of the error
    pub start_byte: usize,
    /// End byte offset of the error
    pub end_byte: usize,
    /// Start position (line, column)
    pub start_position: (usize, usize),
    /// End position (line, column)
    pub end_position: (usize, usize),
    /// The error node kind (usually "ERROR")
    pub kind: String,
}

/// Parsed file with AST
pub struct ParsedFile {
    pub source: String,
    pub tree: Tree,
    pub language: Language,
    pub has_errors: bool,
    /// True when the file was created via `line_based()` (no real tree-sitter parse).
    /// Checkers must fall back to line-based analysis in this case even though
    /// `has_errors` is false.
    pub is_line_based: bool,
}

impl ParsedFile {
    /// Create a ParsedFile for line-based analysis (no tree-sitter)
    /// Used for languages like Dockerfile that don't have compatible grammars
    pub fn line_based(source: String, language: Language) -> Self {
        // Parse with a dummy parser that produces a minimal tree
        let mut parser = Parser::new();
        // Use a simple grammar to get a valid tree structure
        let _ = parser.set_language(&tree_sitter_html::LANGUAGE.into());
        let tree = parser.parse("", None).expect("empty parse should succeed");
        Self {
            has_errors: false,
            source,
            tree,
            language,
            is_line_based: true,
        }
    }

    /// Get the root node
    pub fn root_node(&self) -> tree_sitter::Node<'_> {
        self.tree.root_node()
    }

    /// Get source text for a node
    pub fn node_text(&self, node: &tree_sitter::Node<'_>) -> &str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }

    /// Walk the AST with a visitor function
    /// Returns true to continue traversal, false to stop
    pub fn walk<F>(&self, mut visitor: F)
    where
        F: FnMut(&tree_sitter::Node<'_>, usize) -> bool,
    {
        Self::walk_recursive(&self.root_node(), 0, &mut visitor);
    }

    fn walk_recursive<F>(node: &tree_sitter::Node<'_>, depth: usize, visitor: &mut F)
    where
        F: FnMut(&tree_sitter::Node<'_>, usize) -> bool,
    {
        if !visitor(node, depth) {
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_recursive(&child, depth + 1, visitor);
        }
    }

    /// Collect all parse errors from the tree
    pub fn collect_errors(&self) -> Vec<ParseError> {
        let mut errors = Vec::new();
        self.walk(|node, _depth| {
            if node.is_error() || node.is_missing() {
                errors.push(ParseError {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_position: (
                        node.start_position().row + 1,
                        node.start_position().column + 1,
                    ),
                    end_position: (node.end_position().row + 1, node.end_position().column + 1),
                    kind: node.kind().to_string(),
                });
            }
            true
        });
        errors
    }

    /// Walk the AST with error recovery, skipping ERROR nodes
    pub fn walk_with_recovery<F>(&self, mut visitor: F)
    where
        F: FnMut(&tree_sitter::Node<'_>, usize) -> bool,
    {
        Self::walk_with_recovery_recursive(&self.root_node(), 0, &mut visitor);
    }

    fn walk_with_recovery_recursive<F>(node: &tree_sitter::Node<'_>, depth: usize, visitor: &mut F)
    where
        F: FnMut(&tree_sitter::Node<'_>, usize) -> bool,
    {
        if node.is_error() || node.is_missing() {
            return;
        }

        if !visitor(node, depth) {
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_with_recovery_recursive(&child, depth + 1, visitor);
        }
    }

    /// Get valid (non-error) top-level statements
    pub fn valid_statements(&self) -> Vec<tree_sitter::Node<'_>> {
        let root = self.root_node();
        let mut cursor = root.walk();
        let mut valid = Vec::new();

        for child in root.children(&mut cursor) {
            if !child.is_error() && !child.is_missing() {
                valid.push(child);
            }
        }

        valid
    }

    /// Check if a node is inside an error region
    pub fn is_inside_error(&self, node: &tree_sitter::Node<'_>) -> bool {
        let mut current = *node;
        while let Some(parent) = current.parent() {
            if parent.is_error() {
                return true;
            }
            current = parent;
        }
        false
    }

    /// Get the next valid sibling after an error node
    pub fn synchronize_after<'a>(node: &tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
        let mut current = node.next_sibling();
        while let Some(sibling) = current {
            if !sibling.is_error() && !sibling.is_missing() {
                return Some(sibling);
            }
            current = sibling.next_sibling();
        }
        None
    }
}

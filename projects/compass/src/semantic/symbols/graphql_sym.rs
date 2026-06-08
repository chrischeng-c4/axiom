//! GraphQL symbol extraction (line-based)
//!
//! Extracts symbols from GraphQL schema/query files:
//! - type -> Class
//! - field -> Resource (field)
//! - query/mutation/subscription -> Function
//! - fragment -> Variable
//! - enum -> Enum

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::{Position, Range};

impl SymbolTableBuilder {
    /// Visit GraphQL source and extract symbols.
    pub(crate) fn visit_graphql_lines(&mut self, source: &str) {
        let mut brace_depth: i32 = 0;
        let mut context_stack: Vec<String> = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Type definition: type Name { ... }
            if trimmed.starts_with("type ") {
                let name = trimmed
                    .strip_prefix("type ")
                    .unwrap_or("")
                    .split(|c: char| c.is_whitespace() || c == '{' || c == '@')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    let col = line.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Class,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("type".to_string()),
                        self.current_scope,
                    );
                    context_stack.push("type".to_string());
                }
            }

            // Input type: input Name { ... }
            if trimmed.starts_with("input ") {
                let name = trimmed
                    .strip_prefix("input ")
                    .unwrap_or("")
                    .split(|c: char| c.is_whitespace() || c == '{')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    let col = line.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Class,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("input type".to_string()),
                        self.current_scope,
                    );
                    context_stack.push("type".to_string());
                }
            }

            // Interface: interface Name { ... }
            if trimmed.starts_with("interface ") {
                let name = trimmed
                    .strip_prefix("interface ")
                    .unwrap_or("")
                    .split(|c: char| c.is_whitespace() || c == '{')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    let col = line.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Class,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("interface".to_string()),
                        self.current_scope,
                    );
                    context_stack.push("type".to_string());
                }
            }

            // Enum: enum Name { ... }
            if trimmed.starts_with("enum ") {
                let name = trimmed
                    .strip_prefix("enum ")
                    .unwrap_or("")
                    .split(|c: char| c.is_whitespace() || c == '{')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    let col = line.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Enum,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("enum".to_string()),
                        self.current_scope,
                    );
                    context_stack.push("enum".to_string());
                }
            }

            // Query/Mutation/Subscription operations
            for keyword in &["query ", "mutation ", "subscription "] {
                if trimmed.starts_with(keyword) {
                    let name = trimmed
                        .strip_prefix(keyword)
                        .unwrap_or("")
                        .split(|c: char| c.is_whitespace() || c == '{' || c == '(')
                        .next()
                        .unwrap_or("");
                    if !name.is_empty() {
                        let col = line.find(name).unwrap_or(0) as u32;
                        self.table.add_symbol(
                            name.to_string(),
                            SymbolKind::Function,
                            mk_range(line_num, col, name.len()),
                            None,
                            Some(keyword.trim().to_string()),
                            self.current_scope,
                        );
                    }
                    context_stack.push("operation".to_string());
                }
            }

            // Fragment: fragment Name on Type { ... }
            if trimmed.starts_with("fragment ") {
                let name = trimmed
                    .strip_prefix("fragment ")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    let col = line.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Variable,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("fragment".to_string()),
                        self.current_scope,
                    );
                    context_stack.push("fragment".to_string());
                }
            }

            // Field definition inside type/input/interface
            let in_type = context_stack.last().map_or(false, |c| c == "type");
            if in_type && brace_depth >= 1 && trimmed.contains(':') {
                let field_name = trimmed
                    .split(|c: char| c == ':' || c == '(' || c.is_whitespace())
                    .next()
                    .unwrap_or("");
                if !field_name.is_empty()
                    && field_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_alphabetic())
                    && !field_name.starts_with("type")
                    && !field_name.starts_with("input")
                {
                    let col = line.find(field_name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        field_name.to_string(),
                        SymbolKind::Resource,
                        mk_range(line_num, col, field_name.len()),
                        None,
                        Some("field".to_string()),
                        self.current_scope,
                    );
                }
            }

            // Track brace depth and context
            for ch in trimmed.chars() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        context_stack.pop();
                    }
                }
            }
        }
    }

    /// Build symbol table for a GraphQL file (line-based).
    pub fn build_graphql(mut self, file: &crate::syntax::ParsedFile) -> super::SymbolTable {
        self.visit_graphql_lines(&file.source);
        self.table
    }

    /// Build symbol table for GraphQL from raw source (test helper).
    #[cfg(test)]
    pub fn build_graphql_from_source(mut self, source: &str) -> super::SymbolTable {
        self.visit_graphql_lines(source);
        self.table
    }
}

fn mk_range(line: u32, col: u32, len: usize) -> Range {
    Range::new(
        Position::new(line, col),
        Position::new(line, col + len as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};

    fn build(source: &str) -> super::super::SymbolTable {
        SymbolTableBuilder::new().build_graphql_from_source(source)
    }

    #[test]
    fn test_type_and_fields() {
        let src = "type User {\n  id: ID!\n  name: String\n  email: String\n}\n";
        let table = build(src);

        let classes: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            classes.contains(&"User"),
            "missing type 'User', got {:?}",
            classes
        );

        let fields: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Resource)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            fields.contains(&"id"),
            "missing field 'id', got {:?}",
            fields
        );
        assert!(
            fields.contains(&"name"),
            "missing field 'name', got {:?}",
            fields
        );
    }

    #[test]
    fn test_query_and_fragment() {
        let src = "query GetUser {\n  user {\n    id\n  }\n}\n\nfragment UserFields on User {\n  id\n  name\n}\n";
        let table = build(src);

        let funcs: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            funcs.contains(&"GetUser"),
            "missing query 'GetUser', got {:?}",
            funcs
        );

        let vars: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            vars.contains(&"UserFields"),
            "missing fragment 'UserFields', got {:?}",
            vars
        );
    }

    #[test]
    fn test_enum() {
        let src = "enum Role {\n  ADMIN\n  USER\n  GUEST\n}\n";
        let table = build(src);

        let enums: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Enum)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            enums.contains(&"Role"),
            "missing enum 'Role', got {:?}",
            enums
        );
    }
}

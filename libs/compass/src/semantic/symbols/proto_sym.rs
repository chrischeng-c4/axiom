//! Protocol Buffer (proto3) symbol extraction (line-based)
//!
//! Extracts symbols from proto files:
//! - message -> Class
//! - field -> Resource (field)
//! - service -> Interface
//! - rpc -> Function
//! - enum -> Enum

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::{Position, Range};

impl SymbolTableBuilder {
    /// Visit proto source and extract symbols.
    pub(crate) fn visit_proto_lines(&mut self, source: &str) {
        let mut brace_depth: i32 = 0;
        let mut context_stack: Vec<String> = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Message definition
            if trimmed.starts_with("message ") {
                let name = trimmed
                    .strip_prefix("message ")
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
                        Some("message".to_string()),
                        self.current_scope,
                    );
                    context_stack.push("message".to_string());
                }
            }

            // Service definition
            if trimmed.starts_with("service ") {
                let name = trimmed
                    .strip_prefix("service ")
                    .unwrap_or("")
                    .split(|c: char| c.is_whitespace() || c == '{')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    let col = line.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Interface,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("service".to_string()),
                        self.current_scope,
                    );
                    context_stack.push("service".to_string());
                }
            }

            // Enum definition
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

            // RPC method (inside service)
            if trimmed.starts_with("rpc ") {
                let name = trimmed
                    .strip_prefix("rpc ")
                    .unwrap_or("")
                    .split(|c: char| c.is_whitespace() || c == '(')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    let col = line.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Function,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("rpc method".to_string()),
                        self.current_scope,
                    );
                }
            }

            // Field definition (inside message): type name = number;
            let in_message = context_stack.last().map_or(false, |c| c == "message");
            if in_message
                && brace_depth >= 1
                && trimmed.contains('=')
                && trimmed.ends_with(';')
                && !trimmed.starts_with("reserved")
                && !trimmed.starts_with("option")
            {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    let field_name = parts[1];
                    if !field_name.contains('=') {
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

    /// Build symbol table for a proto file (line-based).
    pub fn build_proto(mut self, file: &crate::syntax::ParsedFile) -> super::SymbolTable {
        self.visit_proto_lines(&file.source);
        self.table
    }

    /// Build symbol table for proto from raw source (test helper).
    #[cfg(test)]
    pub fn build_proto_from_source(mut self, source: &str) -> super::SymbolTable {
        self.visit_proto_lines(source);
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
        SymbolTableBuilder::new().build_proto_from_source(source)
    }

    #[test]
    fn test_message_and_fields() {
        let src = "syntax = \"proto3\";\npackage test;\n\nmessage User {\n  string name = 1;\n  int32 age = 2;\n}\n";
        let table = build(src);

        let classes: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            classes.contains(&"User"),
            "missing message 'User', got {:?}",
            classes
        );

        let fields: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Resource)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            fields.contains(&"name"),
            "missing field 'name', got {:?}",
            fields
        );
        assert!(
            fields.contains(&"age"),
            "missing field 'age', got {:?}",
            fields
        );
    }

    #[test]
    fn test_service_and_rpc() {
        let src = "service UserService {\n  rpc GetUser (GetUserRequest) returns (User);\n  rpc ListUsers (ListUsersRequest) returns (ListUsersResponse);\n}\n";
        let table = build(src);

        let interfaces: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Interface)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            interfaces.contains(&"UserService"),
            "missing service, got {:?}",
            interfaces
        );

        let methods: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            methods.contains(&"GetUser"),
            "missing rpc 'GetUser', got {:?}",
            methods
        );
        assert!(
            methods.contains(&"ListUsers"),
            "missing rpc 'ListUsers', got {:?}",
            methods
        );
    }

    #[test]
    fn test_enum() {
        let src = "enum Status {\n  UNKNOWN = 0;\n  ACTIVE = 1;\n  INACTIVE = 2;\n}\n";
        let table = build(src);

        let enums: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Enum)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            enums.contains(&"Status"),
            "missing enum 'Status', got {:?}",
            enums
        );
    }
}

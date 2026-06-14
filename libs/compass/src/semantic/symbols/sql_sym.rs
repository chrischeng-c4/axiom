//! SQL symbol extraction (line-based)
//!
//! Extracts symbols from SQL files:
//! - CREATE TABLE -> Class
//! - Column definitions -> Field
//! - CREATE FUNCTION/PROCEDURE -> Function
//! - CREATE INDEX -> Variable

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::{Position, Range};

impl SymbolTableBuilder {
    /// Visit SQL source and extract symbols.
    pub(crate) fn visit_sql_lines(&mut self, source: &str) {
        let mut in_create_table = false;
        let mut paren_depth: i32 = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            let upper = trimmed.to_uppercase();

            // CREATE TABLE
            if upper.starts_with("CREATE TABLE") || upper.starts_with("CREATE TEMPORARY TABLE") {
                let name = extract_create_name(trimmed, "TABLE");
                if let Some(name) = name {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.clone(),
                        SymbolKind::Class,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("table".to_string()),
                        self.current_scope,
                    );
                    in_create_table = true;
                    paren_depth = 0;
                }
            }

            // CREATE FUNCTION / CREATE PROCEDURE
            if upper.starts_with("CREATE FUNCTION")
                || upper.starts_with("CREATE OR REPLACE FUNCTION")
            {
                let name = extract_create_name(trimmed, "FUNCTION");
                if let Some(name) = name {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.clone(),
                        SymbolKind::Function,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("function".to_string()),
                        self.current_scope,
                    );
                }
            }

            if upper.starts_with("CREATE PROCEDURE")
                || upper.starts_with("CREATE OR REPLACE PROCEDURE")
            {
                let name = extract_create_name(trimmed, "PROCEDURE");
                if let Some(name) = name {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.clone(),
                        SymbolKind::Function,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("procedure".to_string()),
                        self.current_scope,
                    );
                }
            }

            // CREATE INDEX
            if upper.starts_with("CREATE INDEX") || upper.starts_with("CREATE UNIQUE INDEX") {
                let name = extract_create_name(trimmed, "INDEX");
                if let Some(name) = name {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.clone(),
                        SymbolKind::Variable,
                        mk_range(line_num, col, name.len()),
                        None,
                        Some("index".to_string()),
                        self.current_scope,
                    );
                }
            }

            // Track parens for column definitions inside CREATE TABLE
            if in_create_table {
                for ch in trimmed.chars() {
                    if ch == '(' {
                        paren_depth += 1;
                    } else if ch == ')' {
                        paren_depth -= 1;
                    }
                }

                // Column definition: lines inside the first paren level
                if paren_depth == 1 && !upper.starts_with("CREATE") {
                    if let Some(col_name) = extract_column_name(trimmed) {
                        let col = line.find(col_name).unwrap_or(0) as u32;
                        self.table.add_symbol(
                            col_name.to_string(),
                            SymbolKind::Resource,
                            mk_range(line_num, col, col_name.len()),
                            None,
                            Some("column".to_string()),
                            self.current_scope,
                        );
                    }
                }

                if paren_depth <= 0 && trimmed.contains(')') {
                    in_create_table = false;
                }
            }
        }
    }

    /// Build symbol table for a SQL file (line-based).
    pub fn build_sql(mut self, file: &crate::syntax::ParsedFile) -> super::SymbolTable {
        self.visit_sql_lines(&file.source);
        self.table
    }

    /// Build symbol table for SQL from raw source (test helper).
    #[cfg(test)]
    pub fn build_sql_from_source(mut self, source: &str) -> super::SymbolTable {
        self.visit_sql_lines(source);
        self.table
    }
}

/// Extract the object name after a CREATE ... keyword.
/// Handles patterns like: CREATE TABLE name, CREATE TABLE IF NOT EXISTS name,
/// CREATE TABLE schema.name, etc.
fn extract_create_name(line: &str, object_type: &str) -> Option<String> {
    let upper = line.to_uppercase();
    let type_pos = upper.find(object_type)?;
    let after = &line[type_pos + object_type.len()..].trim_start();

    // Skip "IF NOT EXISTS"
    let after = if after.to_uppercase().starts_with("IF NOT EXISTS") {
        after["IF NOT EXISTS".len()..].trim_start()
    } else {
        after
    };

    // Get the name (possibly schema.name)
    let name = after
        .split(|c: char| c == '(' || c.is_whitespace() || c == ';')
        .next()?
        .trim();

    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Extract column name from a column definition line.
/// Skips constraint keywords like PRIMARY, UNIQUE, FOREIGN, CHECK, CONSTRAINT, INDEX.
fn extract_column_name(line: &str) -> Option<&str> {
    let trimmed = line.trim().trim_end_matches(',');
    let upper = trimmed.to_uppercase();

    // Skip constraint lines
    let skip_prefixes = [
        "PRIMARY",
        "UNIQUE",
        "FOREIGN",
        "CHECK",
        "CONSTRAINT",
        "INDEX",
        "KEY",
    ];
    for prefix in &skip_prefixes {
        if upper.starts_with(prefix) {
            return None;
        }
    }

    // First token is the column name
    let name = trimmed.split_whitespace().next()?;
    // Must start with a letter or underscore
    if name
        .chars()
        .next()
        .map_or(false, |c| c.is_alphabetic() || c == '_')
    {
        Some(name)
    } else {
        None
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
        SymbolTableBuilder::new().build_sql_from_source(source)
    }

    #[test]
    fn test_create_table_and_columns() {
        let src = "CREATE TABLE users (\n  id INT PRIMARY KEY,\n  name VARCHAR(100),\n  email VARCHAR(255)\n);\n";
        let table = build(src);
        let classes: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            classes.contains(&"users"),
            "missing table 'users', got {:?}",
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
            "missing column 'id', got {:?}",
            fields
        );
        assert!(
            fields.contains(&"name"),
            "missing column 'name', got {:?}",
            fields
        );
    }

    #[test]
    fn test_create_function() {
        let src = "CREATE FUNCTION get_user(user_id INT) RETURNS TEXT;\n";
        let table = build(src);
        let funcs: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            funcs.iter().any(|f| f.starts_with("get_user")),
            "missing function 'get_user', got {:?}",
            funcs
        );
    }

    #[test]
    fn test_create_index() {
        let src = "CREATE INDEX idx_users_email ON users (email);\n";
        let table = build(src);
        let vars: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            vars.contains(&"idx_users_email"),
            "missing index, got {:?}",
            vars
        );
    }
}

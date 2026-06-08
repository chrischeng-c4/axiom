//! Query builder helper functions.

use crate::{DataBridgeError, Result};
use unicode_normalization::UnicodeNormalization;
use super::types::AggregateFunction;
use super::window::WindowExpression;

/// Quotes a SQL identifier.
///
/// Handles schema-qualified names by quoting each part separately.
pub fn quote_identifier(name: &str) -> String {
    if name.contains('.') {
        name.split('.')
            .map(|part| format!("\"{}\"", part))
            .collect::<Vec<_>>()
            .join(".")
    } else {
        format!("\"{}\"", name)
    }
}

/// Validates a SQL identifier (table/column name).
///
/// Supports both simple identifiers and schema-qualified names (e.g., "public.users").
pub fn validate_identifier(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(DataBridgeError::Query("Identifier cannot be empty".to_string()));
    }

    // Check if this is a schema-qualified name (e.g., "public.users")
    if name.contains('.') {
        let parts: Vec<&str> = name.split('.').collect();

        // Only allow schema.table format (two parts)
        if parts.len() != 2 {
            return Err(DataBridgeError::Query(
                format!("Invalid schema-qualified identifier '{}': must be in format 'schema.table'", name)
            ));
        }

        // Validate each part separately
        for part in parts {
            validate_identifier_part(part)?;
        }

        return Ok(());
    }

    // Simple identifier - validate as a single part
    validate_identifier_part(name)
}

/// Validates a single part of an identifier (no dots allowed).
pub fn validate_identifier_part(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(DataBridgeError::Query("Identifier part cannot be empty".to_string()));
    }

    // Normalize to NFKC to prevent Unicode confusables
    let name = name.nfkc().collect::<String>();

    // Check length (PostgreSQL limit is 63 bytes per part)
    if name.len() > 63 {
        return Err(DataBridgeError::Query(
            format!("Identifier '{}' exceeds maximum length of 63", name)
        ));
    }

    // Must start with letter or underscore
    let first_char = name.chars().next()
        .ok_or_else(|| DataBridgeError::Query(
            format!("Identifier '{}' is empty or invalid", name)
        ))?;
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return Err(DataBridgeError::Query(
            format!("Identifier '{}' must start with a letter or underscore", name)
        ));
    }

    // Rest must be alphanumeric or underscore
    for ch in name.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(DataBridgeError::Query(
                format!("Identifier '{}' contains invalid character '{}'", name, ch)
            ));
        }
    }

    // Prevent system schema access
    let name_lower = name.to_lowercase();
    if name_lower.starts_with("pg_") {
        return Err(DataBridgeError::Query(
            format!("Access to PostgreSQL system catalog '{}' is not allowed", name)
        ));
    }

    if name_lower == "information_schema" {
        return Err(DataBridgeError::Query(
            "Access to information_schema is not allowed".to_string()
        ));
    }

    // Prevent SQL keywords
    const SQL_KEYWORDS: &[&str] = &[
        "select", "insert", "update", "delete", "drop", "create", "alter",
        "truncate", "grant", "revoke", "exec", "execute", "union", "declare",
        "table", "index", "view", "schema", "database", "user", "role",
        "from", "where", "join", "inner", "outer", "left", "right",
        "on", "using", "and", "or", "not", "in", "exists", "between",
        "like", "ilike", "is", "null", "true", "false", "case", "when",
        "then", "else", "end", "as", "order", "by", "group", "having",
        "limit", "offset", "distinct", "all", "any", "some",
    ];

    if SQL_KEYWORDS.contains(&name_lower.as_str()) {
        return Err(DataBridgeError::Query(
            format!("Identifier '{}' is a reserved SQL keyword", name)
        ));
    }

    Ok(())
}

/// Builds the SQL for an aggregate function.
pub fn build_aggregate_sql(func: &AggregateFunction) -> String {
    match func {
        AggregateFunction::Count => "COUNT(*)".to_string(),
        AggregateFunction::CountColumn(col) => format!("COUNT({})", quote_identifier(col)),
        AggregateFunction::CountDistinct(col) => format!("COUNT(DISTINCT {})", quote_identifier(col)),
        AggregateFunction::Sum(col) => format!("SUM({})", quote_identifier(col)),
        AggregateFunction::Avg(col) => format!("AVG({})", quote_identifier(col)),
        AggregateFunction::Min(col) => format!("MIN({})", quote_identifier(col)),
        AggregateFunction::Max(col) => format!("MAX({})", quote_identifier(col)),
    }
}

/// Builds the SQL for a window function expression.
pub fn build_window_sql(expr: &WindowExpression) -> String {
    use super::window::WindowFunction;

    let func_sql = match &expr.function {
        WindowFunction::RowNumber => "ROW_NUMBER()".to_string(),
        WindowFunction::Rank => "RANK()".to_string(),
        WindowFunction::DenseRank => "DENSE_RANK()".to_string(),
        WindowFunction::Ntile(n) => format!("NTILE({})", n),
        WindowFunction::Lag(col, offset, _) => {
            let off = offset.unwrap_or(1);
            format!("LAG({}, {})", quote_identifier(col), off)
        }
        WindowFunction::Lead(col, offset, _) => {
            let off = offset.unwrap_or(1);
            format!("LEAD({}, {})", quote_identifier(col), off)
        }
        WindowFunction::FirstValue(col) => {
            format!("FIRST_VALUE({})", quote_identifier(col))
        }
        WindowFunction::LastValue(col) => {
            format!("LAST_VALUE({})", quote_identifier(col))
        }
        WindowFunction::Sum(col) => format!("SUM({})", quote_identifier(col)),
        WindowFunction::Avg(col) => format!("AVG({})", quote_identifier(col)),
        WindowFunction::Count => "COUNT(*)".to_string(),
        WindowFunction::CountColumn(col) => {
            format!("COUNT({})", quote_identifier(col))
        }
        WindowFunction::Min(col) => format!("MIN({})", quote_identifier(col)),
        WindowFunction::Max(col) => format!("MAX({})", quote_identifier(col)),
    };

    let mut over_parts = Vec::new();

    if !expr.spec.partition_by.is_empty() {
        let cols: Vec<String> = expr
            .spec
            .partition_by
            .iter()
            .map(|c| quote_identifier(c))
            .collect();
        over_parts.push(format!("PARTITION BY {}", cols.join(", ")));
    }

    if !expr.spec.order_by.is_empty() {
        let cols: Vec<String> = expr
            .spec
            .order_by
            .iter()
            .map(|(c, d)| format!("{} {}", quote_identifier(c), d.to_sql()))
            .collect();
        over_parts.push(format!("ORDER BY {}", cols.join(", ")));
    }

    format!(
        "{} OVER ({}) AS {}",
        func_sql,
        over_parts.join(" "),
        quote_identifier(&expr.alias)
    )
}

/// Adjusts parameter indices in SQL by adding an offset.
///
/// This is used when combining CTE parameters with main query parameters.
pub fn adjust_param_indices(sql: &str, offset: usize) -> String {
    if offset == 0 {
        return sql.to_string();
    }

    let mut result = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Found a parameter marker, extract the number
            let mut num_str = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() {
                    // Safe: peek() returned Some, so next() will also return Some
                    if let Some(digit) = chars.next() {
                        num_str.push(digit);
                    }
                } else {
                    break;
                }
            }

            if !num_str.is_empty() {
                if let Ok(num) = num_str.parse::<usize>() {
                    // Adjust the parameter index
                    result.push('$');
                    result.push_str(&(num + offset).to_string());
                } else {
                    // Failed to parse, keep as-is
                    result.push('$');
                    result.push_str(&num_str);
                }
            } else {
                // No digits after $, keep as-is
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

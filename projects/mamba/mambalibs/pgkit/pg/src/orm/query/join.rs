//! JOIN clause types and utilities.

use crate::Result;
use super::builder::QueryBuilder;
use super::types::JoinType;

/// Structured JOIN condition to prevent SQL injection
/// Only allows safe table.column = table.column patterns
#[derive(Debug, Clone)]
pub struct JoinCondition {
    /// Column from the left (main) table
    pub left_column: String,
    /// Table/alias on the right side of the join
    pub right_table: String,
    /// Column from the right table
    pub right_column: String,
}

impl JoinCondition {
    /// Create a new JOIN condition with validated identifiers
    pub fn new(left_column: &str, right_table: &str, right_column: &str) -> Result<Self> {
        QueryBuilder::validate_identifier(left_column)?;
        QueryBuilder::validate_identifier(right_table)?;
        QueryBuilder::validate_identifier(right_column)?;
        Ok(Self {
            left_column: left_column.to_string(),
            right_table: right_table.to_string(),
            right_column: right_column.to_string(),
        })
    }

    /// Generate SQL for the ON clause
    pub fn to_sql(&self, main_table: &str) -> String {
        format!(
            "\"{}\".\"{}\" = \"{}\".\"{}\"",
            main_table,
            self.left_column,
            self.right_table,
            self.right_column
        )
    }
}

/// Represents a JOIN clause
#[derive(Debug, Clone)]
pub struct JoinClause {
    /// Type of JOIN (INNER, LEFT, RIGHT, FULL)
    pub join_type: JoinType,
    /// Table to join
    pub table: String,
    /// Optional alias for the joined table
    pub alias: Option<String>,
    /// ON condition for the join
    pub on_condition: JoinCondition,
}

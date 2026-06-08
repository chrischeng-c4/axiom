//! QueryBuilder struct and core implementation.

use super::helpers::{validate_identifier, validate_identifier_part};
use super::join::JoinClause;
use super::types::{
    AggregateFunction, CommonTableExpression, HavingCondition, Operator, OrderDirection, SetQuery,
    Subquery,
};
use super::window::WindowExpression;
use crate::{ExtractedValue, Result};

/// Represents a WHERE condition.
#[derive(Debug, Clone)]
pub(crate) struct WhereCondition {
    pub(crate) field: String,
    pub(crate) operator: Operator,
    pub(crate) value: Option<ExtractedValue>, // None for IS NULL / IS NOT NULL
    pub(crate) subquery: Option<Subquery>,    // Some for subquery conditions
}

/// Type-safe SQL query builder.
///
/// Provides a fluent API for constructing SELECT, INSERT, UPDATE, and DELETE queries
/// with parameter binding and security validation.
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    pub(crate) table: String,
    /// SELECT columns (empty means SELECT *)
    pub(crate) select_columns: Vec<String>,
    /// JOIN clauses
    pub(crate) joins: Vec<JoinClause>,
    /// WHERE conditions (field, operator, value)
    pub(crate) where_conditions: Vec<WhereCondition>,
    /// ORDER BY clauses (field, direction)
    pub(crate) order_by_clauses: Vec<(String, OrderDirection)>,
    /// LIMIT clause
    pub(crate) limit_value: Option<i64>,
    /// OFFSET clause
    pub(crate) offset_value: Option<i64>,
    /// Aggregate functions with optional aliases
    pub(crate) aggregates: Vec<(AggregateFunction, Option<String>)>,
    /// GROUP BY columns
    pub(crate) group_by_columns: Vec<String>,
    /// HAVING conditions for filtering aggregate results
    pub(crate) having_conditions: Vec<HavingCondition>,
    /// Whether to use DISTINCT
    pub(crate) distinct: bool,
    /// Columns for DISTINCT ON (PostgreSQL-specific)
    pub(crate) distinct_on_columns: Vec<String>,
    /// Common Table Expressions (WITH clause)
    pub(crate) ctes: Vec<CommonTableExpression>,
    /// Window function expressions
    pub(crate) windows: Vec<WindowExpression>,
    /// Set operations (UNION, INTERSECT, EXCEPT)
    pub(crate) set_operations: Vec<SetQuery>,
    /// Columns to return from UPDATE/DELETE (RETURNING clause)
    pub(crate) returning: Vec<String>,
    /// Columns to defer (exclude from initial SELECT) - for lazy loading optimization
    pub(crate) deferred_columns: Vec<String>,
    /// If set, only these columns are selected (overrides select_columns)
    pub(crate) only_columns: Option<Vec<String>>,
    /// If true, select_columns are raw SQL expressions (no quoting applied)
    pub(crate) raw_select: bool,
}

impl QueryBuilder {
    /// Creates a new query builder for a table.
    ///
    /// # Arguments
    ///
    /// * `table` - Table name (validated for SQL injection)
    ///
    /// # Errors
    ///
    /// Returns error if table name is invalid.
    pub fn new(table: &str) -> Result<Self> {
        Self::validate_identifier(table)?;
        Ok(Self {
            table: table.to_string(),
            select_columns: Vec::new(),
            joins: Vec::new(),
            where_conditions: Vec::new(),
            order_by_clauses: Vec::new(),
            limit_value: None,
            offset_value: None,
            aggregates: Vec::new(),
            group_by_columns: Vec::new(),
            having_conditions: Vec::new(),
            distinct: false,
            distinct_on_columns: Vec::new(),
            ctes: Vec::new(),
            windows: Vec::new(),
            set_operations: Vec::new(),
            returning: Vec::new(),
            deferred_columns: Vec::new(),
            only_columns: None,
            raw_select: false,
        })
    }

    /// Quotes a SQL identifier.
    ///
    /// Handles schema-qualified names by quoting each part separately.
    pub fn quote_identifier(name: &str) -> String {
        super::helpers::quote_identifier(name)
    }

    /// Validates a SQL identifier (table/column name).
    ///
    /// Supports both simple identifiers and schema-qualified names (e.g., "public.users").
    pub fn validate_identifier(name: &str) -> Result<()> {
        validate_identifier(name)
    }

    /// Validates a single part of an identifier (no dots allowed).
    pub fn validate_identifier_part(name: &str) -> Result<()> {
        validate_identifier_part(name)
    }

    /// Get the table name
    pub fn table(&self) -> &str {
        &self.table
    }

    /// Builds a query and returns (SQL, parameters) tuple.
    ///
    /// This is a convenience method for SELECT queries.
    pub fn build(&self) -> (String, Vec<ExtractedValue>) {
        self.build_select()
    }
}

//! Window function types and utilities.

use crate::ExtractedValue;
use super::types::OrderDirection;

/// Window function types
#[derive(Debug, Clone, PartialEq)]
pub enum WindowFunction {
    /// ROW_NUMBER() - assigns sequential numbers
    RowNumber,
    /// RANK() - assigns rank with gaps
    Rank,
    /// DENSE_RANK() - assigns rank without gaps
    DenseRank,
    /// NTILE(n) - divides rows into n groups
    Ntile(i32),
    /// LAG(column, offset, default) - access previous row
    Lag(String, Option<i32>, Option<ExtractedValue>),
    /// LEAD(column, offset, default) - access next row
    Lead(String, Option<i32>, Option<ExtractedValue>),
    /// FIRST_VALUE(column) - first value in window
    FirstValue(String),
    /// LAST_VALUE(column) - last value in window
    LastValue(String),
    /// SUM(column) as window function
    Sum(String),
    /// AVG(column) as window function
    Avg(String),
    /// COUNT(*) as window function
    Count,
    /// COUNT(column) as window function
    CountColumn(String),
    /// MIN(column) as window function
    Min(String),
    /// MAX(column) as window function
    Max(String),
}

/// Window specification (PARTITION BY, ORDER BY)
#[derive(Debug, Clone, Default)]
pub struct WindowSpec {
    /// PARTITION BY columns
    pub partition_by: Vec<String>,
    /// ORDER BY columns with direction
    pub order_by: Vec<(String, OrderDirection)>,
}

impl WindowSpec {
    /// Creates a new empty window specification.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the PARTITION BY columns for the window.
    ///
    /// This divides rows into groups that share common values in the specified columns.
    pub fn partition_by(mut self, columns: &[&str]) -> Self {
        self.partition_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Adds an ORDER BY clause to the window specification.
    ///
    /// This defines the ordering of rows within each partition.
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order_by.push((column.to_string(), direction));
        self
    }
}

/// A window function expression with alias
#[derive(Debug, Clone)]
pub struct WindowExpression {
    /// The window function
    pub function: WindowFunction,
    /// The window specification
    pub spec: WindowSpec,
    /// Alias for the result column
    pub alias: String,
}

//! Query builder types and enums.

use crate::ExtractedValue;

/// Represents a Common Table Expression (CTE) for WITH clause
#[derive(Debug, Clone)]
pub struct CommonTableExpression {
    /// The name of the CTE (used to reference it in the main query)
    pub name: String,
    /// The SQL query for this CTE (will be built from a QueryBuilder)
    pub sql: String,
    /// Parameters for this CTE's query
    pub params: Vec<ExtractedValue>,
}

/// Represents a subquery for use in WHERE clauses
#[derive(Debug, Clone)]
pub struct Subquery {
    /// The SQL of the subquery
    pub sql: String,
    /// Parameters for the subquery
    pub params: Vec<ExtractedValue>,
}

/// Query comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Equal (=)
    Eq,
    /// Not equal (!=)
    Ne,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
    /// IN clause
    In,
    /// NOT IN clause
    NotIn,
    /// BETWEEN lower AND upper
    Between,
    /// NOT BETWEEN lower AND upper
    NotBetween,
    /// LIKE pattern matching
    Like,
    /// ILIKE case-insensitive pattern matching
    ILike,
    /// IS NULL
    IsNull,
    /// IS NOT NULL
    IsNotNull,
    /// Column value is in subquery results
    InSubquery,
    /// Column value is not in subquery results
    NotInSubquery,
    /// Subquery returns at least one row
    Exists,
    /// Subquery returns no rows
    NotExists,
    /// JSONB contains @>
    JsonContains,
    /// JSONB contained by <@
    JsonContainedBy,
    /// JSONB key exists ?
    JsonKeyExists,
    /// JSONB any key exists ?|
    JsonAnyKeyExists,
    /// JSONB all keys exist ?&
    JsonAllKeysExist,
    // Array operators
    /// Array contains value (value = ANY(array_column))
    Any,
    /// Array contains all values (array_column @> ARRAY[values])
    ArrayContains,
    /// Array is contained by (array_column <@ ARRAY[values])
    ArrayContainedBy,
    /// Array overlaps with values (array_column && ARRAY[values])
    ArrayOverlaps,
    /// Array has element (value = ANY(array_column)) - alias for Any
    Has,
    /// Array has all elements (array_column @> ARRAY[values]) - alias for ArrayContains
    HasAll,
    /// Array has any element (array_column && ARRAY[values]) - alias for ArrayOverlaps
    HasAny,
}

impl Operator {
    /// Returns the SQL operator string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            Operator::Eq => "=",
            Operator::Ne => "!=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
            Operator::Between => "BETWEEN",
            Operator::NotBetween => "NOT BETWEEN",
            Operator::Like => "LIKE",
            Operator::ILike => "ILIKE",
            Operator::IsNull => "IS NULL",
            Operator::IsNotNull => "IS NOT NULL",
            Operator::InSubquery => "IN",
            Operator::NotInSubquery => "NOT IN",
            Operator::Exists => "EXISTS",
            Operator::NotExists => "NOT EXISTS",
            Operator::JsonContains => "@>",
            Operator::JsonContainedBy => "<@",
            Operator::JsonKeyExists => "?",
            Operator::JsonAnyKeyExists => "?|",
            Operator::JsonAllKeysExist => "?&",
            // Array operators
            Operator::Any | Operator::Has => "= ANY",
            Operator::ArrayContains | Operator::HasAll => "@>",
            Operator::ArrayContainedBy => "<@",
            Operator::ArrayOverlaps | Operator::HasAny => "&&",
        }
    }

    /// Returns true if this operator requires special array handling in WHERE clause
    pub fn is_array_operator(&self) -> bool {
        matches!(
            self,
            Operator::Any
                | Operator::Has
                | Operator::ArrayContains
                | Operator::HasAll
                | Operator::ArrayContainedBy
                | Operator::ArrayOverlaps
                | Operator::HasAny
        )
    }
}

/// SQL aggregate functions.
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    /// COUNT(*) - count all rows
    Count,
    /// COUNT(column) - count non-null values in column
    CountColumn(String),
    /// COUNT(DISTINCT column) - count distinct values
    CountDistinct(String),
    /// SUM(column) - sum of values
    Sum(String),
    /// AVG(column) - average of values
    Avg(String),
    /// MIN(column) - minimum value
    Min(String),
    /// MAX(column) - maximum value
    Max(String),
}

/// Represents a HAVING clause condition for aggregate queries
#[derive(Debug, Clone)]
pub struct HavingCondition {
    /// The aggregate expression (e.g., "COUNT(*)", "SUM(amount)")
    pub aggregate: AggregateFunction,
    /// The comparison operator
    pub operator: Operator,
    /// The value to compare against
    pub value: ExtractedValue,
}

/// Sort order direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

impl OrderDirection {
    /// Returns the SQL order direction string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

/// Type of SQL JOIN
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    /// INNER JOIN
    Inner,
    /// LEFT JOIN
    Left,
    /// RIGHT JOIN
    Right,
    /// FULL OUTER JOIN
    Full,
}

impl JoinType {
    /// Returns the SQL JOIN type string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL OUTER JOIN",
        }
    }
}

/// Set operation types for combining query results
#[derive(Debug, Clone, PartialEq)]
pub enum SetOperation {
    /// UNION - combines results, removes duplicates
    Union,
    /// UNION ALL - combines results, keeps duplicates
    UnionAll,
    /// INTERSECT - returns only rows in both queries
    Intersect,
    /// INTERSECT ALL - keeps duplicates
    IntersectAll,
    /// EXCEPT - returns rows in first query but not second
    Except,
    /// EXCEPT ALL - keeps duplicates
    ExceptAll,
}

impl SetOperation {
    /// Returns the SQL set operation string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            SetOperation::Union => " UNION ",
            SetOperation::UnionAll => " UNION ALL ",
            SetOperation::Intersect => " INTERSECT ",
            SetOperation::IntersectAll => " INTERSECT ALL ",
            SetOperation::Except => " EXCEPT ",
            SetOperation::ExceptAll => " EXCEPT ALL ",
        }
    }
}

/// A combined query with set operation
#[derive(Debug, Clone)]
pub struct SetQuery {
    /// The operation to perform
    pub operation: SetOperation,
    /// The SQL of the other query
    pub sql: String,
    /// Parameters for the other query
    pub params: Vec<ExtractedValue>,
}

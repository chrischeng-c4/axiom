//! Query builder and expression types for MongoDB operations.
//!
//! This module provides type-safe query construction with:
//! - `QueryExpr`: Represents MongoDB query conditions (eq, ne, gt, etc.)
//! - `QueryBuilder`: Clone-based chainable API for building queries
//!
//! # Example
//!
//! ```rust,ignore
//! use mongokit::query::{QueryBuilder, QueryExpr};
//!
//! let query = QueryBuilder::new("users")
//!     .filter(QueryExpr::eq("status", "active"))
//!     .filter(QueryExpr::gte("age", 18))
//!     .sort(vec![("name".to_string(), 1)])
//!     .limit(10)
//!     .build_filter();
//! ```

use bson::{doc, Bson, Document as BsonDocument};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};

/// Represents a MongoDB query expression.
///
/// Each variant corresponds to a MongoDB comparison or logical operator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryExpr {
    /// Equality: `{ field: { $eq: value } }`
    Eq { field: String, value: Bson },
    /// Not equal: `{ field: { $ne: value } }`
    Ne { field: String, value: Bson },
    /// Greater than: `{ field: { $gt: value } }`
    Gt { field: String, value: Bson },
    /// Greater than or equal: `{ field: { $gte: value } }`
    Gte { field: String, value: Bson },
    /// Less than: `{ field: { $lt: value } }`
    Lt { field: String, value: Bson },
    /// Less than or equal: `{ field: { $lte: value } }`
    Lte { field: String, value: Bson },
    /// In array: `{ field: { $in: [values] } }`
    In { field: String, values: Vec<Bson> },
    /// Not in array: `{ field: { $nin: [values] } }`
    Nin { field: String, values: Vec<Bson> },
    /// Field exists: `{ field: { $exists: true/false } }`
    Exists { field: String, exists: bool },
    /// Regex match: `{ field: { $regex: pattern } }`
    Regex { field: String, pattern: String },
    /// Logical AND: `{ $and: [exprs] }`
    And(Vec<QueryExpr>),
    /// Logical OR: `{ $or: [exprs] }`
    Or(Vec<QueryExpr>),
    /// Raw BSON document (for advanced queries)
    Raw(BsonDocument),
}

impl QueryExpr {
    /// Create an equality expression.
    pub fn eq(field: impl Into<String>, value: impl Into<Bson>) -> Self {
        QueryExpr::Eq {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a not-equal expression.
    pub fn ne(field: impl Into<String>, value: impl Into<Bson>) -> Self {
        QueryExpr::Ne {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a greater-than expression.
    pub fn gt(field: impl Into<String>, value: impl Into<Bson>) -> Self {
        QueryExpr::Gt {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a greater-than-or-equal expression.
    pub fn gte(field: impl Into<String>, value: impl Into<Bson>) -> Self {
        QueryExpr::Gte {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a less-than expression.
    pub fn lt(field: impl Into<String>, value: impl Into<Bson>) -> Self {
        QueryExpr::Lt {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a less-than-or-equal expression.
    pub fn lte(field: impl Into<String>, value: impl Into<Bson>) -> Self {
        QueryExpr::Lte {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create an $in expression.
    pub fn in_<I, V>(field: impl Into<String>, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Bson>,
    {
        QueryExpr::In {
            field: field.into(),
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }

    /// Create a $nin expression.
    pub fn nin<I, V>(field: impl Into<String>, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Bson>,
    {
        QueryExpr::Nin {
            field: field.into(),
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }

    /// Create an $exists expression.
    pub fn exists(field: impl Into<String>, exists: bool) -> Self {
        QueryExpr::Exists {
            field: field.into(),
            exists,
        }
    }

    /// Create a $regex expression.
    pub fn regex(field: impl Into<String>, pattern: impl Into<String>) -> Self {
        QueryExpr::Regex {
            field: field.into(),
            pattern: pattern.into(),
        }
    }

    /// Create a logical AND expression.
    pub fn and(exprs: Vec<QueryExpr>) -> Self {
        QueryExpr::And(exprs)
    }

    /// Create a logical OR expression.
    pub fn or(exprs: Vec<QueryExpr>) -> Self {
        QueryExpr::Or(exprs)
    }

    /// Create a raw BSON expression.
    pub fn raw(doc: BsonDocument) -> Self {
        QueryExpr::Raw(doc)
    }

    /// Convert the expression to a BSON document.
    pub fn to_bson(&self) -> BsonDocument {
        match self {
            QueryExpr::Eq { field, value } => {
                doc! { field: { "$eq": value.clone() } }
            }
            QueryExpr::Ne { field, value } => {
                doc! { field: { "$ne": value.clone() } }
            }
            QueryExpr::Gt { field, value } => {
                doc! { field: { "$gt": value.clone() } }
            }
            QueryExpr::Gte { field, value } => {
                doc! { field: { "$gte": value.clone() } }
            }
            QueryExpr::Lt { field, value } => {
                doc! { field: { "$lt": value.clone() } }
            }
            QueryExpr::Lte { field, value } => {
                doc! { field: { "$lte": value.clone() } }
            }
            QueryExpr::In { field, values } => {
                doc! { field: { "$in": values.clone() } }
            }
            QueryExpr::Nin { field, values } => {
                doc! { field: { "$nin": values.clone() } }
            }
            QueryExpr::Exists { field, exists } => {
                doc! { field: { "$exists": *exists } }
            }
            QueryExpr::Regex { field, pattern } => {
                doc! { field: { "$regex": pattern.clone() } }
            }
            QueryExpr::And(exprs) => {
                let bson_exprs: Vec<Bson> =
                    exprs.iter().map(|e| Bson::Document(e.to_bson())).collect();
                doc! { "$and": bson_exprs }
            }
            QueryExpr::Or(exprs) => {
                let bson_exprs: Vec<Bson> =
                    exprs.iter().map(|e| Bson::Document(e.to_bson())).collect();
                doc! { "$or": bson_exprs }
            }
            QueryExpr::Raw(doc) => doc.clone(),
        }
    }
}

/// Clone-based chainable query builder for MongoDB find operations.
///
/// Each method returns a new `QueryBuilder` instance with the updated state,
/// leaving the original instance unchanged (immutable pattern).
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    collection_name: String,
    filters: Vec<QueryExpr>,
    sort: Option<Vec<(String, i32)>>,
    skip: Option<u64>,
    limit: Option<i64>,
    projection: Option<Vec<String>>,
}

impl QueryBuilder {
    /// Create a new query builder for the given collection.
    pub fn new(collection_name: impl Into<String>) -> Self {
        Self {
            collection_name: collection_name.into(),
            filters: Vec::new(),
            sort: None,
            skip: None,
            limit: None,
            projection: None,
        }
    }

    /// Get the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Add a filter expression. Returns a new QueryBuilder (clone-based).
    pub fn filter(mut self, expr: QueryExpr) -> Self {
        self.filters.push(expr);
        self
    }

    /// Add multiple filter expressions. Returns a new QueryBuilder.
    pub fn filters(mut self, exprs: Vec<QueryExpr>) -> Self {
        self.filters.extend(exprs);
        self
    }

    /// Set sort order. Returns a new QueryBuilder.
    ///
    /// # Arguments
    /// * `fields` - Vec of (field_name, direction) where direction is 1 (asc) or -1 (desc)
    pub fn sort(mut self, fields: Vec<(String, i32)>) -> Self {
        self.sort = Some(fields);
        self
    }

    /// Set skip count. Returns a new QueryBuilder.
    pub fn skip(mut self, n: u64) -> Self {
        self.skip = Some(n);
        self
    }

    /// Set limit count. Returns a new QueryBuilder.
    pub fn limit(mut self, n: i64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Set projection fields. Returns a new QueryBuilder.
    pub fn projection(mut self, fields: Vec<String>) -> Self {
        self.projection = Some(fields);
        self
    }

    /// Build the filter as a BSON document.
    pub fn build_filter(&self) -> BsonDocument {
        if self.filters.is_empty() {
            return BsonDocument::new();
        }

        if self.filters.len() == 1 {
            return self.filters[0].to_bson();
        }

        // Multiple filters: combine with $and
        let bson_exprs: Vec<Bson> = self
            .filters
            .iter()
            .map(|e| Bson::Document(e.to_bson()))
            .collect();
        doc! { "$and": bson_exprs }
    }

    /// Build the sort document.
    pub fn build_sort(&self) -> Option<BsonDocument> {
        self.sort.as_ref().map(|fields| {
            let mut doc = BsonDocument::new();
            for (field, direction) in fields {
                doc.insert(field, *direction);
            }
            doc
        })
    }

    /// Build the projection document.
    pub fn build_projection(&self) -> Option<BsonDocument> {
        self.projection.as_ref().map(|fields| {
            let mut doc = BsonDocument::new();
            for field in fields {
                doc.insert(field, 1);
            }
            doc
        })
    }

    /// Build FindOptions for MongoDB query.
    pub fn build_options(&self) -> FindOptions {
        let mut options = FindOptions::default();
        options.sort = self.build_sort();
        options.skip = self.skip;
        options.limit = self.limit;
        options.projection = self.build_projection();

        // Optimization: set batch_size to match limit
        if let Some(limit) = self.limit {
            options.batch_size = Some(limit as u32);
        }

        options
    }

    /// Get the number of filters.
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    /// Get skip value.
    pub fn get_skip(&self) -> Option<u64> {
        self.skip
    }

    /// Get limit value.
    pub fn get_limit(&self) -> Option<i64> {
        self.limit
    }

    /// Check if sort is set.
    pub fn has_sort(&self) -> bool {
        self.sort.is_some()
    }

    /// Check if projection is set.
    pub fn has_projection(&self) -> bool {
        self.projection.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_expr_eq() {
        let expr = QueryExpr::eq("name", "Alice");
        let bson = expr.to_bson();
        assert_eq!(bson, doc! { "name": { "$eq": "Alice" } });
    }

    #[test]
    fn test_query_expr_gt() {
        let expr = QueryExpr::gt("age", 18);
        let bson = expr.to_bson();
        assert_eq!(bson, doc! { "age": { "$gt": 18 } });
    }

    #[test]
    fn test_query_expr_in() {
        let expr = QueryExpr::in_("status", vec!["active", "pending"]);
        let bson = expr.to_bson();
        assert_eq!(bson, doc! { "status": { "$in": ["active", "pending"] } });
    }

    #[test]
    fn test_query_expr_exists() {
        let expr = QueryExpr::exists("email", true);
        let bson = expr.to_bson();
        assert_eq!(bson, doc! { "email": { "$exists": true } });
    }

    #[test]
    fn test_query_expr_regex() {
        let expr = QueryExpr::regex("email", "@example\\.com$");
        let bson = expr.to_bson();
        assert_eq!(bson, doc! { "email": { "$regex": "@example\\.com$" } });
    }

    #[test]
    fn test_query_expr_and() {
        let expr = QueryExpr::and(vec![
            QueryExpr::eq("status", "active"),
            QueryExpr::gte("age", 18),
        ]);
        let bson = expr.to_bson();

        let expected = doc! {
            "$and": [
                { "status": { "$eq": "active" } },
                { "age": { "$gte": 18 } }
            ]
        };
        assert_eq!(bson, expected);
    }

    #[test]
    fn test_query_expr_or() {
        let expr = QueryExpr::or(vec![
            QueryExpr::eq("role", "admin"),
            QueryExpr::eq("role", "superuser"),
        ]);
        let bson = expr.to_bson();

        let expected = doc! {
            "$or": [
                { "role": { "$eq": "admin" } },
                { "role": { "$eq": "superuser" } }
            ]
        };
        assert_eq!(bson, expected);
    }

    #[test]
    fn test_query_builder_new() {
        let qb = QueryBuilder::new("users");
        assert_eq!(qb.collection_name(), "users");
        assert_eq!(qb.filter_count(), 0);
        assert!(qb.get_skip().is_none());
        assert!(qb.get_limit().is_none());
    }

    #[test]
    fn test_query_builder_filter() {
        let qb = QueryBuilder::new("users").filter(QueryExpr::eq("status", "active"));

        assert_eq!(qb.filter_count(), 1);
        let filter = qb.build_filter();
        assert_eq!(filter, doc! { "status": { "$eq": "active" } });
    }

    #[test]
    fn test_query_builder_multiple_filters() {
        let qb = QueryBuilder::new("users")
            .filter(QueryExpr::eq("status", "active"))
            .filter(QueryExpr::gte("age", 18));

        assert_eq!(qb.filter_count(), 2);
        let filter = qb.build_filter();

        let expected = doc! {
            "$and": [
                { "status": { "$eq": "active" } },
                { "age": { "$gte": 18 } }
            ]
        };
        assert_eq!(filter, expected);
    }

    #[test]
    fn test_query_builder_sort() {
        let qb =
            QueryBuilder::new("users").sort(vec![("name".to_string(), 1), ("age".to_string(), -1)]);

        assert!(qb.has_sort());
        let sort = qb.build_sort().unwrap();
        assert_eq!(sort, doc! { "name": 1, "age": -1 });
    }

    #[test]
    fn test_query_builder_skip_limit() {
        let qb = QueryBuilder::new("users").skip(10).limit(20);

        assert_eq!(qb.get_skip(), Some(10));
        assert_eq!(qb.get_limit(), Some(20));
    }

    #[test]
    fn test_query_builder_projection() {
        let qb =
            QueryBuilder::new("users").projection(vec!["name".to_string(), "email".to_string()]);

        assert!(qb.has_projection());
        let proj = qb.build_projection().unwrap();
        assert_eq!(proj, doc! { "name": 1, "email": 1 });
    }

    #[test]
    fn test_query_builder_chaining() {
        let qb = QueryBuilder::new("users")
            .filter(QueryExpr::eq("active", true))
            .sort(vec![("created_at".to_string(), -1)])
            .skip(5)
            .limit(10)
            .projection(vec!["name".to_string(), "email".to_string()]);

        assert_eq!(qb.collection_name(), "users");
        assert_eq!(qb.filter_count(), 1);
        assert!(qb.has_sort());
        assert_eq!(qb.get_skip(), Some(5));
        assert_eq!(qb.get_limit(), Some(10));
        assert!(qb.has_projection());
    }

    #[test]
    fn test_query_builder_build_options() {
        let qb = QueryBuilder::new("users")
            .sort(vec![("name".to_string(), 1)])
            .skip(10)
            .limit(20)
            .projection(vec!["name".to_string()]);

        let options = qb.build_options();
        assert!(options.sort.is_some());
        assert_eq!(options.skip, Some(10));
        assert_eq!(options.limit, Some(20));
        assert!(options.projection.is_some());
        assert_eq!(options.batch_size, Some(20));
    }

    #[test]
    fn test_query_builder_empty_filter() {
        let qb = QueryBuilder::new("users");
        let filter = qb.build_filter();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_query_builder_clone_immutability() {
        let qb1 = QueryBuilder::new("users");
        let qb2 = qb1.clone().filter(QueryExpr::eq("x", 1));

        // qb1 should be unchanged
        assert_eq!(qb1.filter_count(), 0);
        // qb2 should have the filter
        assert_eq!(qb2.filter_count(), 1);
    }
}

//! Aggregation pipeline builder for MongoDB.
//!
//! This module provides a fluent API for building MongoDB aggregation pipelines
//! with type-safe stages and accumulators.
//!
//! # Example
//!
//! ```rust,ignore
//! use mongokit::aggregation::{AggregationBuilder, Accumulator};
//!
//! let pipeline = AggregationBuilder::new("users")
//!     .match_stage(doc! { "active": true })
//!     .group(None, vec![Accumulator::avg("age", "avg_age")])
//!     .build();
//! ```

use bson::{doc, Bson, Document as BsonDocument};
use serde::{Deserialize, Serialize};

/// Represents a group-by key for $group stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupId {
    /// No grouping - aggregate all documents into single result.
    Null,
    /// Group by a single field.
    Field(String),
    /// Group by multiple fields.
    Fields(Vec<String>),
    /// Group by a custom expression.
    Expression(BsonDocument),
}

impl GroupId {
    /// Convert to BSON value for $group._id.
    pub fn to_bson(&self) -> Bson {
        match self {
            GroupId::Null => Bson::Null,
            GroupId::Field(f) => Bson::String(format!("${}", f)),
            GroupId::Fields(fields) => {
                let mut doc = BsonDocument::new();
                for f in fields {
                    doc.insert(f.clone(), format!("${}", f));
                }
                Bson::Document(doc)
            }
            GroupId::Expression(expr) => Bson::Document(expr.clone()),
        }
    }
}

/// Represents an accumulator expression for $group stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Accumulator {
    /// Calculate average: `{ $avg: "$field" }`
    Avg { field: String, alias: String },
    /// Calculate sum: `{ $sum: "$field" }` or `{ $sum: 1 }` for count
    Sum {
        field: Option<String>,
        alias: String,
    },
    /// Find minimum: `{ $min: "$field" }`
    Min { field: String, alias: String },
    /// Find maximum: `{ $max: "$field" }`
    Max { field: String, alias: String },
    /// Count documents: `{ $sum: 1 }`
    Count { alias: String },
}

impl Accumulator {
    /// Create an $avg accumulator.
    pub fn avg(field: impl Into<String>, alias: impl Into<String>) -> Self {
        Accumulator::Avg {
            field: field.into(),
            alias: alias.into(),
        }
    }

    /// Create a $sum accumulator for a field.
    pub fn sum(field: impl Into<String>, alias: impl Into<String>) -> Self {
        Accumulator::Sum {
            field: Some(field.into()),
            alias: alias.into(),
        }
    }

    /// Create a $min accumulator.
    pub fn min(field: impl Into<String>, alias: impl Into<String>) -> Self {
        Accumulator::Min {
            field: field.into(),
            alias: alias.into(),
        }
    }

    /// Create a $max accumulator.
    pub fn max(field: impl Into<String>, alias: impl Into<String>) -> Self {
        Accumulator::Max {
            field: field.into(),
            alias: alias.into(),
        }
    }

    /// Create a count accumulator (using $sum: 1).
    pub fn count(alias: impl Into<String>) -> Self {
        Accumulator::Count {
            alias: alias.into(),
        }
    }

    /// Get the alias (output field name) for this accumulator.
    pub fn alias(&self) -> &str {
        match self {
            Accumulator::Avg { alias, .. } => alias,
            Accumulator::Sum { alias, .. } => alias,
            Accumulator::Min { alias, .. } => alias,
            Accumulator::Max { alias, .. } => alias,
            Accumulator::Count { alias } => alias,
        }
    }

    /// Convert to BSON expression.
    pub fn to_bson(&self) -> (String, Bson) {
        let (alias, expr) = match self {
            Accumulator::Avg { field, alias } => {
                (alias.clone(), doc! { "$avg": format!("${}", field) })
            }
            Accumulator::Sum {
                field: Some(f),
                alias,
            } => (alias.clone(), doc! { "$sum": format!("${}", f) }),
            Accumulator::Sum { field: None, alias } => (alias.clone(), doc! { "$sum": 1 }),
            Accumulator::Min { field, alias } => {
                (alias.clone(), doc! { "$min": format!("${}", field) })
            }
            Accumulator::Max { field, alias } => {
                (alias.clone(), doc! { "$max": format!("${}", field) })
            }
            Accumulator::Count { alias } => (alias.clone(), doc! { "$sum": 1 }),
        };
        (alias, Bson::Document(expr))
    }
}

/// Represents a stage in an aggregation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AggregationStage {
    /// $match stage for filtering documents.
    Match(BsonDocument),
    /// $group stage for grouping and aggregating.
    Group {
        id: GroupId,
        accumulators: Vec<Accumulator>,
    },
}

impl AggregationStage {
    /// Convert to BSON document.
    pub fn to_bson(&self) -> BsonDocument {
        match self {
            AggregationStage::Match(filter) => {
                doc! { "$match": filter.clone() }
            }
            AggregationStage::Group { id, accumulators } => {
                let mut group_doc = BsonDocument::new();
                group_doc.insert("_id", id.to_bson());
                for acc in accumulators {
                    let (alias, expr) = acc.to_bson();
                    group_doc.insert(alias, expr);
                }
                doc! { "$group": group_doc }
            }
        }
    }
}

/// Fluent builder for MongoDB aggregation pipelines.
///
/// Supports $match and $group stages with common accumulator helpers.
#[derive(Debug, Clone)]
pub struct AggregationBuilder {
    collection: String,
    stages: Vec<AggregationStage>,
}

impl AggregationBuilder {
    /// Create a new builder for the given collection.
    pub fn new(collection: impl Into<String>) -> Self {
        AggregationBuilder {
            collection: collection.into(),
            stages: Vec::new(),
        }
    }

    /// Get the collection name.
    pub fn collection(&self) -> &str {
        &self.collection
    }

    /// Add a $match stage to filter documents.
    pub fn match_stage(mut self, filter: BsonDocument) -> Self {
        self.stages.push(AggregationStage::Match(filter));
        self
    }

    /// Add a $group stage with the given group key and accumulators.
    pub fn group(mut self, id: GroupId, accumulators: Vec<Accumulator>) -> Self {
        self.stages
            .push(AggregationStage::Group { id, accumulators });
        self
    }

    /// Add a $group stage with null _id (aggregate all into single result).
    pub fn group_all(self, accumulators: Vec<Accumulator>) -> Self {
        self.group(GroupId::Null, accumulators)
    }

    /// Build the pipeline as a vector of BSON documents.
    pub fn build(self) -> Vec<BsonDocument> {
        self.stages.into_iter().map(|s| s.to_bson()).collect()
    }

    /// Get the number of stages in the pipeline.
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    /// Check if the pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }
}

// Convenience functions for creating common aggregation patterns
impl AggregationBuilder {
    /// Create a simple avg aggregation with optional filter.
    pub fn simple_avg(
        collection: impl Into<String>,
        field: impl Into<String>,
        filter: Option<BsonDocument>,
    ) -> Vec<BsonDocument> {
        let mut builder = Self::new(collection);
        if let Some(f) = filter {
            builder = builder.match_stage(f);
        }
        builder
            .group_all(vec![Accumulator::avg(field, "result")])
            .build()
    }

    /// Create a simple sum aggregation with optional filter.
    pub fn simple_sum(
        collection: impl Into<String>,
        field: impl Into<String>,
        filter: Option<BsonDocument>,
    ) -> Vec<BsonDocument> {
        let mut builder = Self::new(collection);
        if let Some(f) = filter {
            builder = builder.match_stage(f);
        }
        builder
            .group_all(vec![Accumulator::sum(field, "result")])
            .build()
    }

    /// Create a simple min aggregation with optional filter.
    pub fn simple_min(
        collection: impl Into<String>,
        field: impl Into<String>,
        filter: Option<BsonDocument>,
    ) -> Vec<BsonDocument> {
        let mut builder = Self::new(collection);
        if let Some(f) = filter {
            builder = builder.match_stage(f);
        }
        builder
            .group_all(vec![Accumulator::min(field, "result")])
            .build()
    }

    /// Create a simple max aggregation with optional filter.
    pub fn simple_max(
        collection: impl Into<String>,
        field: impl Into<String>,
        filter: Option<BsonDocument>,
    ) -> Vec<BsonDocument> {
        let mut builder = Self::new(collection);
        if let Some(f) = filter {
            builder = builder.match_stage(f);
        }
        builder
            .group_all(vec![Accumulator::max(field, "result")])
            .build()
    }

    /// Create a simple count aggregation with optional filter.
    pub fn simple_count(
        collection: impl Into<String>,
        filter: Option<BsonDocument>,
    ) -> Vec<BsonDocument> {
        let mut builder = Self::new(collection);
        if let Some(f) = filter {
            builder = builder.match_stage(f);
        }
        builder.group_all(vec![Accumulator::count("count")]).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_avg_with_filter() {
        let filter = doc! { "status": "active" };
        let pipeline = AggregationBuilder::simple_avg("users", "age", Some(filter.clone()));

        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline[0], doc! { "$match": filter });
        assert_eq!(
            pipeline[1],
            doc! { "$group": { "_id": null, "result": { "$avg": "$age" } } }
        );
    }

    #[test]
    fn test_simple_avg_without_filter() {
        let pipeline = AggregationBuilder::simple_avg("users", "age", None);

        assert_eq!(pipeline.len(), 1);
        assert_eq!(
            pipeline[0],
            doc! { "$group": { "_id": null, "result": { "$avg": "$age" } } }
        );
    }

    #[test]
    fn test_simple_count() {
        let filter = doc! { "active": true };
        let pipeline = AggregationBuilder::simple_count("users", Some(filter.clone()));

        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline[0], doc! { "$match": filter });
        assert_eq!(
            pipeline[1],
            doc! { "$group": { "_id": null, "count": { "$sum": 1 } } }
        );
    }

    #[test]
    fn test_simple_sum() {
        let pipeline = AggregationBuilder::simple_sum("orders", "amount", None);

        assert_eq!(pipeline.len(), 1);
        assert_eq!(
            pipeline[0],
            doc! { "$group": { "_id": null, "result": { "$sum": "$amount" } } }
        );
    }

    #[test]
    fn test_simple_min_max() {
        let pipeline_min = AggregationBuilder::simple_min("products", "price", None);
        let pipeline_max = AggregationBuilder::simple_max("products", "price", None);

        assert_eq!(
            pipeline_min[0],
            doc! { "$group": { "_id": null, "result": { "$min": "$price" } } }
        );
        assert_eq!(
            pipeline_max[0],
            doc! { "$group": { "_id": null, "result": { "$max": "$price" } } }
        );
    }

    #[test]
    fn test_fluent_builder() {
        let pipeline = AggregationBuilder::new("orders")
            .match_stage(doc! { "status": "completed" })
            .group(
                GroupId::Field("customer_id".to_string()),
                vec![
                    Accumulator::sum("amount", "total"),
                    Accumulator::count("order_count"),
                ],
            )
            .build();

        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline[0], doc! { "$match": { "status": "completed" } });

        // Check $group stage structure
        let group_stage = &pipeline[1];
        assert!(group_stage.contains_key("$group"));
    }

    #[test]
    fn test_group_by_multiple_fields() {
        let pipeline = AggregationBuilder::new("sales")
            .group(
                GroupId::Fields(vec!["year".to_string(), "month".to_string()]),
                vec![Accumulator::sum("revenue", "total_revenue")],
            )
            .build();

        assert_eq!(pipeline.len(), 1);
        let group_doc = pipeline[0].get_document("$group").unwrap();
        let id_doc = group_doc.get_document("_id").unwrap();
        assert_eq!(id_doc.get_str("year").unwrap(), "$year");
        assert_eq!(id_doc.get_str("month").unwrap(), "$month");
    }

    #[test]
    fn test_accumulator_alias() {
        let acc = Accumulator::avg("price", "average_price");
        assert_eq!(acc.alias(), "average_price");

        let count_acc = Accumulator::count("total");
        assert_eq!(count_acc.alias(), "total");
    }

    #[test]
    fn test_builder_is_empty() {
        let builder = AggregationBuilder::new("test");
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);

        let builder = builder.match_stage(doc! {});
        assert!(!builder.is_empty());
        assert_eq!(builder.len(), 1);
    }
}

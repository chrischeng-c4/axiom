//! Bulk write operation types for MongoDB.
//!
//! This module defines the core `BulkOperation` enum that represents all
//! MongoDB bulk write operations. These types are designed to be:
//! - Type-safe with Rust enum pattern matching
//! - Serializable via serde for JSON/BSON
//! - Convertible from Python dictionaries by the active language binding layer
//!
//! # Example
//!
//! ```rust,ignore
//! use mongokit::bulk::{BulkOperation, BulkWriteResult};
//!
//! let ops = vec![
//!     BulkOperation::UpdateOne {
//!         filter: doc! { "status": "pending" },
//!         update: doc! { "$set": { "status": "active" } },
//!         upsert: false,
//!         array_filters: None,
//!     },
//!     BulkOperation::InsertOne {
//!         document: doc! { "name": "Alice" },
//!     },
//!     BulkOperation::DeleteMany {
//!         filter: doc! { "expired": true },
//!     },
//! ];
//! ```

use bson::Document as BsonDocument;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single bulk write operation.
///
/// Each variant corresponds to a MongoDB bulk write operation type.
/// The enum provides exhaustive pattern matching for type safety.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum BulkOperation {
    /// Update a single document matching the filter.
    UpdateOne {
        /// Query filter to match documents.
        filter: BsonDocument,
        /// Update operations (e.g., `{"$set": {...}}`).
        update: BsonDocument,
        /// If true, insert a new document if no match is found.
        #[serde(default)]
        upsert: bool,
        /// Optional array filters for updating nested arrays.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        array_filters: Option<Vec<BsonDocument>>,
    },

    /// Update multiple documents matching the filter.
    UpdateMany {
        /// Query filter to match documents.
        filter: BsonDocument,
        /// Update operations (e.g., `{"$set": {...}}`).
        update: BsonDocument,
        /// If true, insert a new document if no match is found.
        #[serde(default)]
        upsert: bool,
        /// Optional array filters for updating nested arrays.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        array_filters: Option<Vec<BsonDocument>>,
    },

    /// Insert a single document.
    InsertOne {
        /// The document to insert.
        document: BsonDocument,
    },

    /// Delete a single document matching the filter.
    DeleteOne {
        /// Query filter to match documents.
        filter: BsonDocument,
    },

    /// Delete multiple documents matching the filter.
    DeleteMany {
        /// Query filter to match documents.
        filter: BsonDocument,
    },

    /// Replace a single document matching the filter.
    ReplaceOne {
        /// Query filter to match documents.
        filter: BsonDocument,
        /// The replacement document.
        replacement: BsonDocument,
        /// If true, insert a new document if no match is found.
        #[serde(default)]
        upsert: bool,
    },
}

impl BulkOperation {
    /// Get the operation type as a string.
    pub fn op_type(&self) -> &'static str {
        match self {
            BulkOperation::UpdateOne { .. } => "update_one",
            BulkOperation::UpdateMany { .. } => "update_many",
            BulkOperation::InsertOne { .. } => "insert_one",
            BulkOperation::DeleteOne { .. } => "delete_one",
            BulkOperation::DeleteMany { .. } => "delete_many",
            BulkOperation::ReplaceOne { .. } => "replace_one",
        }
    }

    /// Check if this operation has upsert enabled.
    pub fn is_upsert(&self) -> bool {
        match self {
            BulkOperation::UpdateOne { upsert, .. } => *upsert,
            BulkOperation::UpdateMany { upsert, .. } => *upsert,
            BulkOperation::ReplaceOne { upsert, .. } => *upsert,
            _ => false,
        }
    }

    /// Get the filter document (if applicable).
    pub fn filter(&self) -> Option<&BsonDocument> {
        match self {
            BulkOperation::UpdateOne { filter, .. } => Some(filter),
            BulkOperation::UpdateMany { filter, .. } => Some(filter),
            BulkOperation::DeleteOne { filter, .. } => Some(filter),
            BulkOperation::DeleteMany { filter, .. } => Some(filter),
            BulkOperation::ReplaceOne { filter, .. } => Some(filter),
            BulkOperation::InsertOne { .. } => None,
        }
    }
}

/// Error details for a failed bulk write operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulkWriteError {
    /// Index of the operation that failed.
    pub index: usize,
    /// Error code (if available from MongoDB).
    pub code: Option<i32>,
    /// Error message.
    pub message: String,
}

/// Result of a bulk write operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BulkWriteResult {
    /// Number of documents inserted.
    pub inserted_count: u64,
    /// Number of documents matched by update/replace filters.
    pub matched_count: u64,
    /// Number of documents modified.
    pub modified_count: u64,
    /// Number of documents deleted.
    pub deleted_count: u64,
    /// Number of documents upserted (inserted via upsert).
    pub upserted_count: u64,
    /// Map of operation index to upserted document ObjectId.
    #[serde(default)]
    pub upserted_ids: HashMap<usize, String>,
    /// List of errors from failed operations (in unordered mode).
    #[serde(default)]
    pub write_errors: Vec<BulkWriteError>,
}

impl BulkWriteResult {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any documents were affected.
    pub fn has_changes(&self) -> bool {
        self.inserted_count > 0
            || self.modified_count > 0
            || self.deleted_count > 0
            || self.upserted_count > 0
    }

    /// Get total number of write operations that succeeded.
    pub fn total_affected(&self) -> u64 {
        self.inserted_count + self.modified_count + self.deleted_count + self.upserted_count
    }

    /// Check if there were any write errors.
    pub fn has_errors(&self) -> bool {
        !self.write_errors.is_empty()
    }

    /// Add an error for a failed operation.
    pub fn add_error(&mut self, index: usize, code: Option<i32>, message: String) {
        self.write_errors.push(BulkWriteError {
            index,
            code,
            message,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::doc;

    #[test]
    fn test_bulk_operation_serde() {
        let op = BulkOperation::UpdateOne {
            filter: doc! { "status": "pending" },
            update: doc! { "$set": { "status": "active" } },
            upsert: false,
            array_filters: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("update_one"));
        assert!(json.contains("pending"));

        // Deserialize back
        let deserialized: BulkOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_insert_one_serde() {
        let op = BulkOperation::InsertOne {
            document: doc! { "name": "Alice", "age": 30 },
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("insert_one"));
        assert!(json.contains("Alice"));

        let deserialized: BulkOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_delete_many_serde() {
        let op = BulkOperation::DeleteMany {
            filter: doc! { "expired": true },
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("delete_many"));

        let deserialized: BulkOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_replace_one_with_upsert() {
        let op = BulkOperation::ReplaceOne {
            filter: doc! { "_id": "123" },
            replacement: doc! { "name": "Bob" },
            upsert: true,
        };

        assert!(op.is_upsert());
        assert_eq!(op.op_type(), "replace_one");
        assert!(op.filter().is_some());
    }

    #[test]
    fn test_bulk_write_result() {
        let mut result = BulkWriteResult::new();
        assert!(!result.has_changes());
        assert_eq!(result.total_affected(), 0);

        result.inserted_count = 5;
        result.modified_count = 3;
        result.deleted_count = 2;

        assert!(result.has_changes());
        assert_eq!(result.total_affected(), 10);
    }

    #[test]
    fn test_op_type() {
        assert_eq!(
            BulkOperation::UpdateOne {
                filter: doc! {},
                update: doc! {},
                upsert: false,
                array_filters: None,
            }
            .op_type(),
            "update_one"
        );

        assert_eq!(
            BulkOperation::InsertOne { document: doc! {} }.op_type(),
            "insert_one"
        );

        assert_eq!(
            BulkOperation::DeleteMany { filter: doc! {} }.op_type(),
            "delete_many"
        );
    }

    #[test]
    fn test_deserialize_from_dict_format() {
        // This tests the format that Python will send
        let json = r#"{"op": "update_one", "filter": {"status": "pending"}, "update": {"$set": {"status": "active"}}, "upsert": false}"#;
        let op: BulkOperation = serde_json::from_str(json).unwrap();

        match op {
            BulkOperation::UpdateOne {
                filter,
                update,
                upsert,
                ..
            } => {
                assert_eq!(filter.get_str("status").unwrap(), "pending");
                assert!(update.contains_key("$set"));
                assert!(!upsert);
            }
            _ => panic!("Expected UpdateOne"),
        }
    }
}

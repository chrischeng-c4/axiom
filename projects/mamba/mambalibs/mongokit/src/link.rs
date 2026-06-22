//! Link types and batched fetching for document relationships.
//!
//! This module provides types and utilities for handling document links
//! (references between documents) with efficient batch fetching.
//!
//! # Example
//!
//! ```rust,ignore
//! use mongokit::link::{LinkField, LinkType, LinkRef};
//!
//! let field = LinkField::new("author", LinkType::Link, "users", false);
//! let refs = vec![
//!     LinkRef::new(doc_id, "author", target_id),
//! ];
//! ```

use bson::{oid::ObjectId, Document as BsonDocument};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of link relationship between documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// Forward reference: this document points to another.
    Link,
    /// Reverse reference: another document points to this one.
    BackLink,
}

/// Describes a link field in a document model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkField {
    /// Name of the field in the document.
    pub field_name: String,
    /// Type of link (forward or reverse).
    pub link_type: LinkType,
    /// Target collection name.
    pub target_collection: String,
    /// Whether this field holds a list of references.
    pub is_list: bool,
}

impl LinkField {
    /// Create a new LinkField.
    pub fn new(
        field_name: impl Into<String>,
        link_type: LinkType,
        target_collection: impl Into<String>,
        is_list: bool,
    ) -> Self {
        Self {
            field_name: field_name.into(),
            link_type,
            target_collection: target_collection.into(),
            is_list,
        }
    }

    /// Create a single forward link field.
    pub fn link(field_name: impl Into<String>, target_collection: impl Into<String>) -> Self {
        Self::new(field_name, LinkType::Link, target_collection, false)
    }

    /// Create a list forward link field.
    pub fn link_list(field_name: impl Into<String>, target_collection: impl Into<String>) -> Self {
        Self::new(field_name, LinkType::Link, target_collection, true)
    }

    /// Create a back link field.
    pub fn back_link(field_name: impl Into<String>, target_collection: impl Into<String>) -> Self {
        Self::new(field_name, LinkType::BackLink, target_collection, true)
    }
}

/// A reference from one document to another.
#[derive(Debug, Clone)]
pub struct LinkRef {
    /// ObjectId of the source document.
    pub source_doc_id: ObjectId,
    /// Field name containing the reference.
    pub field_name: String,
    /// ObjectId of the target document.
    pub target_id: ObjectId,
    /// Index in the list (if field is a list), None for single refs.
    pub list_index: Option<usize>,
}

impl LinkRef {
    /// Create a new LinkRef for a single reference.
    pub fn new(
        source_doc_id: ObjectId,
        field_name: impl Into<String>,
        target_id: ObjectId,
    ) -> Self {
        Self {
            source_doc_id,
            field_name: field_name.into(),
            target_id,
            list_index: None,
        }
    }

    /// Create a new LinkRef for a list reference at given index.
    pub fn new_at_index(
        source_doc_id: ObjectId,
        field_name: impl Into<String>,
        target_id: ObjectId,
        index: usize,
    ) -> Self {
        Self {
            source_doc_id,
            field_name: field_name.into(),
            target_id,
            list_index: Some(index),
        }
    }
}

/// Result of a batch fetch operation.
#[derive(Debug, Clone, Default)]
pub struct BatchFetchResult {
    /// Fetched documents keyed by ObjectId.
    pub fetched_docs: HashMap<ObjectId, BsonDocument>,
    /// Any errors encountered during fetch (non-fatal).
    /// Note: Fatal errors are propagated immediately via Result.
    /// This field is for collecting non-fatal issues like missing documents.
    pub errors: Vec<String>,
}

impl BatchFetchResult {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if fetch was successful (no errors).
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get number of documents fetched.
    pub fn count(&self) -> usize {
        self.fetched_docs.len()
    }

    /// Merge another result into this one.
    pub fn merge(&mut self, other: BatchFetchResult) {
        self.fetched_docs.extend(other.fetched_docs);
        self.errors.extend(other.errors);
    }
}

/// Collected references grouped by target collection.
#[derive(Debug, Clone, Default)]
pub struct CollectedRefs {
    /// References grouped by target collection name.
    pub by_collection: HashMap<String, Vec<LinkRef>>,
}

impl CollectedRefs {
    /// Create a new empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a reference.
    pub fn add(&mut self, collection: impl Into<String>, link_ref: LinkRef) {
        self.by_collection
            .entry(collection.into())
            .or_default()
            .push(link_ref);
    }

    /// Get unique target IDs for a collection.
    pub fn unique_ids(&self, collection: &str) -> Vec<ObjectId> {
        self.by_collection
            .get(collection)
            .map(|refs| {
                let mut ids: Vec<ObjectId> = refs.iter().map(|r| r.target_id).collect();
                ids.sort();
                ids.dedup();
                ids
            })
            .unwrap_or_default()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.by_collection.is_empty()
    }

    /// Get all collection names.
    pub fn collections(&self) -> Vec<&str> {
        self.by_collection.keys().map(|s| s.as_str()).collect()
    }
}

/// Extract forward link references from a document based on link fields.
///
/// Note: BackLink fields are skipped by this function. BackLinks require
/// reverse queries from target collections and must be handled separately.
pub fn extract_refs_from_doc(doc: &BsonDocument, link_fields: &[LinkField]) -> CollectedRefs {
    let mut refs = CollectedRefs::new();

    // Get source document ID
    let source_id = match doc.get_object_id("_id") {
        Ok(id) => id,
        Err(_) => return refs, // No _id, skip
    };

    for field in link_fields {
        if field.link_type == LinkType::BackLink {
            // BackLinks are handled differently (query from target)
            continue;
        }

        if let Some(value) = doc.get(&field.field_name) {
            if field.is_list {
                // Handle list of references
                if let Some(arr) = value.as_array() {
                    for (idx, item) in arr.iter().enumerate() {
                        if let Some(target_id) = extract_object_id(item) {
                            refs.add(
                                &field.target_collection,
                                LinkRef::new_at_index(source_id, &field.field_name, target_id, idx),
                            );
                        }
                    }
                }
            } else {
                // Handle single reference
                if let Some(target_id) = extract_object_id(value) {
                    refs.add(
                        &field.target_collection,
                        LinkRef::new(source_id, &field.field_name, target_id),
                    );
                }
            }
        }
    }

    refs
}

/// Extract ObjectId from a BSON value.
/// Supports both raw ObjectId and DBRef format.
fn extract_object_id(value: &bson::Bson) -> Option<ObjectId> {
    match value {
        bson::Bson::ObjectId(id) => Some(*id),
        bson::Bson::Document(doc) => {
            // Handle DBRef format: {"$ref": "collection", "$id": ObjectId}
            doc.get_object_id("$id")
                .ok()
                .or_else(|| doc.get_object_id("_id").ok())
        }
        bson::Bson::String(s) => ObjectId::parse_str(s).ok(),
        _ => None,
    }
}

/// Distribute fetched documents back to source documents.
pub fn distribute_fetched_docs(
    docs: &mut [BsonDocument],
    refs: &CollectedRefs,
    fetched: &BatchFetchResult,
) {
    // Build a map from source_id to doc index
    let mut doc_index_map: HashMap<ObjectId, usize> = HashMap::new();
    for (idx, doc) in docs.iter().enumerate() {
        if let Ok(id) = doc.get_object_id("_id") {
            doc_index_map.insert(id, idx);
        }
    }

    // Distribute fetched docs
    for refs_list in refs.by_collection.values() {
        for link_ref in refs_list {
            if let Some(&doc_idx) = doc_index_map.get(&link_ref.source_doc_id) {
                if let Some(fetched_doc) = fetched.fetched_docs.get(&link_ref.target_id) {
                    let doc = &mut docs[doc_idx];

                    if let Some(list_idx) = link_ref.list_index {
                        // Update list item
                        if let Some(arr) = doc.get_array_mut(&link_ref.field_name).ok() {
                            if list_idx < arr.len() {
                                arr[list_idx] = bson::Bson::Document(fetched_doc.clone());
                            }
                        }
                    } else {
                        // Update single field
                        doc.insert(&link_ref.field_name, fetched_doc.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::doc;

    #[test]
    fn test_link_field_new() {
        let field = LinkField::new("author", LinkType::Link, "users", false);
        assert_eq!(field.field_name, "author");
        assert_eq!(field.link_type, LinkType::Link);
        assert_eq!(field.target_collection, "users");
        assert!(!field.is_list);
    }

    #[test]
    fn test_link_field_helpers() {
        let single = LinkField::link("author", "users");
        assert_eq!(single.link_type, LinkType::Link);
        assert!(!single.is_list);

        let list = LinkField::link_list("tags", "tags");
        assert_eq!(list.link_type, LinkType::Link);
        assert!(list.is_list);

        let back = LinkField::back_link("posts", "posts");
        assert_eq!(back.link_type, LinkType::BackLink);
        assert!(back.is_list);
    }

    #[test]
    fn test_link_ref_new() {
        let source_id = ObjectId::new();
        let target_id = ObjectId::new();

        let single = LinkRef::new(source_id, "author", target_id);
        assert_eq!(single.source_doc_id, source_id);
        assert_eq!(single.field_name, "author");
        assert_eq!(single.target_id, target_id);
        assert!(single.list_index.is_none());

        let indexed = LinkRef::new_at_index(source_id, "tags", target_id, 2);
        assert_eq!(indexed.list_index, Some(2));
    }

    #[test]
    fn test_batch_fetch_result() {
        let mut result = BatchFetchResult::new();
        assert!(result.is_success());
        assert_eq!(result.count(), 0);

        let id = ObjectId::new();
        result
            .fetched_docs
            .insert(id, doc! { "_id": id, "name": "test" });
        assert_eq!(result.count(), 1);

        result.errors.push("error".to_string());
        assert!(!result.is_success());
    }

    #[test]
    fn test_collected_refs() {
        let mut refs = CollectedRefs::new();
        assert!(refs.is_empty());

        let source_id = ObjectId::new();
        let target_id1 = ObjectId::new();
        let target_id2 = ObjectId::new();

        refs.add("users", LinkRef::new(source_id, "author", target_id1));
        refs.add("users", LinkRef::new(source_id, "reviewer", target_id2));
        refs.add("users", LinkRef::new(source_id, "author", target_id1)); // duplicate

        assert!(!refs.is_empty());
        assert_eq!(refs.collections().len(), 1);

        let unique_ids = refs.unique_ids("users");
        assert_eq!(unique_ids.len(), 2); // deduplicated
    }

    #[test]
    fn test_extract_refs_from_doc() {
        let source_id = ObjectId::new();
        let author_id = ObjectId::new();
        let tag_id1 = ObjectId::new();
        let tag_id2 = ObjectId::new();

        let doc = doc! {
            "_id": source_id,
            "title": "Post",
            "author": author_id,
            "tags": [tag_id1, tag_id2],
        };

        let link_fields = vec![
            LinkField::link("author", "users"),
            LinkField::link_list("tags", "tags"),
        ];

        let refs = extract_refs_from_doc(&doc, &link_fields);

        let user_ids = refs.unique_ids("users");
        assert_eq!(user_ids.len(), 1);
        assert_eq!(user_ids[0], author_id);

        let tag_ids = refs.unique_ids("tags");
        assert_eq!(tag_ids.len(), 2);
    }

    #[test]
    fn test_extract_object_id_formats() {
        let oid = ObjectId::new();

        // Direct ObjectId
        let val1 = bson::Bson::ObjectId(oid);
        assert_eq!(extract_object_id(&val1), Some(oid));

        // DBRef format
        let val2 = bson::Bson::Document(doc! { "$ref": "users", "$id": oid });
        assert_eq!(extract_object_id(&val2), Some(oid));

        // String format
        let val3 = bson::Bson::String(oid.to_hex());
        assert_eq!(extract_object_id(&val3), Some(oid));

        // Invalid
        let val4 = bson::Bson::Int32(123);
        assert_eq!(extract_object_id(&val4), None);
    }
}

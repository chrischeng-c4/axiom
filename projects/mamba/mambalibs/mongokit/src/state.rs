//! State tracking for document changes (Copy-On-Write).
//!
//! This module provides a StateTracker that records field changes
//! and supports rollback to original values.
//!
//! # Example
//!
//! ```rust,ignore
//! use mongokit::state::StateTracker;
//! use bson::{doc, Bson};
//!
//! let mut tracker = StateTracker::new();
//! tracker.track_change("name", Bson::String("Alice".to_string()));
//! // ... modify document ...
//! assert!(tracker.is_modified());
//! assert!(tracker.has_changed("name"));
//! ```

use bson::{Bson, Document as BsonDocument};
use std::collections::HashSet;

/// Tracks changes to document fields using Copy-On-Write semantics.
///
/// On first write to a field, the original value is stored.
/// Subsequent writes to the same field do not update the stored original.
///
/// # Limitations
///
/// - **Missing vs Null**: The tracker cannot distinguish between a field
///   that was missing and one that was explicitly null. Rollback will
///   restore the original value but cannot remove fields that were added.
/// - **Field-level granularity**: Nested changes track only the top-level
///   field name. Callers should pass the top-level field and its full
///   original value when tracking nested changes.
#[derive(Debug, Clone, Default)]
pub struct StateTracker {
    /// Original values for changed fields (stored on first write).
    original_values: BsonDocument,
    /// Set of field names that have been modified.
    changed_fields: HashSet<String>,
}

impl StateTracker {
    /// Create a new empty StateTracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a field change by storing the original value (COW).
    ///
    /// Only the first call for a given field stores the original value.
    /// Subsequent calls for the same field are ignored (COW semantics).
    ///
    /// For nested changes, track the top-level field name.
    pub fn track_change(&mut self, field: impl Into<String>, original_value: Bson) {
        let field = field.into();
        // Extract top-level field name for nested paths
        let top_level_field = field.split('.').next().unwrap_or(&field).to_string();

        // COW: only store on first write
        if !self.changed_fields.contains(&top_level_field) {
            self.original_values.insert(&top_level_field, original_value);
            self.changed_fields.insert(top_level_field);
        }
    }

    /// Check if any field has been modified.
    pub fn is_modified(&self) -> bool {
        !self.changed_fields.is_empty()
    }

    /// Check if a specific field has been modified.
    pub fn has_changed(&self, field: &str) -> bool {
        // Check top-level field for nested paths
        let top_level_field = field.split('.').next().unwrap_or(field);
        self.changed_fields.contains(top_level_field)
    }

    /// Get the names of all changed fields (sorted for deterministic ordering).
    pub fn changed_field_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.changed_fields.iter().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get changes as a document containing only the modified fields
    /// with their current values from the provided document.
    pub fn get_changes(&self, current_data: &BsonDocument) -> BsonDocument {
        let mut changes = BsonDocument::new();
        for field in &self.changed_fields {
            if let Some(value) = current_data.get(field) {
                changes.insert(field, value.clone());
            }
        }
        changes
    }

    /// Rollback all changes by restoring original values to the document.
    ///
    /// After rollback, the tracker is cleared (no more changes tracked).
    pub fn rollback(&mut self, document: &mut BsonDocument) {
        for (field, original_value) in self.original_values.iter() {
            document.insert(field, original_value.clone());
        }
        self.clear();
    }

    /// Reset the tracker, clearing all change tracking state.
    ///
    /// The current document state becomes the new baseline.
    pub fn reset(&mut self) {
        self.clear();
    }

    /// Clear internal state (helper for rollback and reset).
    fn clear(&mut self) {
        self.original_values.clear();
        self.changed_fields.clear();
    }

    /// Get the original value for a specific field (if tracked).
    pub fn get_original(&self, field: &str) -> Option<&Bson> {
        self.original_values.get(field)
    }

    /// Reconstruct the full original document state from current data
    /// and tracked changes.
    ///
    /// Returns a document with original values for changed fields
    /// and current values for unchanged fields.
    pub fn get_all_original_data(&self, current_data: &BsonDocument) -> BsonDocument {
        let mut original = current_data.clone();
        // Overwrite changed fields with original values
        for (field, original_value) in self.original_values.iter() {
            original.insert(field, original_value.clone());
        }
        original
    }

    /// Get the number of changed fields.
    pub fn change_count(&self) -> usize {
        self.changed_fields.len()
    }

    /// Compare a field's current value to its original value.
    ///
    /// Returns true if the field is tracked AND its current value differs from original.
    pub fn compare_field(&self, field: &str, current_data: &BsonDocument) -> bool {
        let top_level_field = field.split('.').next().unwrap_or(field);
        if !self.changed_fields.contains(top_level_field) {
            return false;
        }
        let original = self.original_values.get(top_level_field);
        let current = current_data.get(top_level_field);
        match (original, current) {
            (Some(o), Some(c)) => o != c,
            (Some(_), None) => true, // Field was deleted
            (None, Some(_)) => true, // Should not happen with COW semantics
            (None, None) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::doc;

    #[test]
    fn test_new_tracker_is_not_modified() {
        let tracker = StateTracker::new();
        assert!(!tracker.is_modified());
        assert_eq!(tracker.change_count(), 0);
    }

    #[test]
    fn test_track_single_change() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));

        assert!(tracker.is_modified());
        assert!(tracker.has_changed("name"));
        assert!(!tracker.has_changed("age"));
        assert_eq!(tracker.change_count(), 1);
    }

    #[test]
    fn test_cow_behavior() {
        let mut tracker = StateTracker::new();

        // First write stores "Alice"
        tracker.track_change("name", Bson::String("Alice".to_string()));
        assert_eq!(
            tracker.get_original("name"),
            Some(&Bson::String("Alice".to_string()))
        );

        // Second write is ignored (COW)
        tracker.track_change("name", Bson::String("Bob".to_string()));
        assert_eq!(
            tracker.get_original("name"),
            Some(&Bson::String("Alice".to_string()))
        );

        assert_eq!(tracker.change_count(), 1);
    }

    #[test]
    fn test_get_changes() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));
        tracker.track_change("age", Bson::Int32(30));

        let current = doc! {
            "name": "Bob",
            "age": 31,
            "city": "Taipei"
        };

        let changes = tracker.get_changes(&current);
        assert_eq!(changes.get_str("name"), Ok("Bob"));
        assert_eq!(changes.get_i32("age"), Ok(31));
        assert!(changes.get("city").is_none()); // Not changed
    }

    #[test]
    fn test_rollback() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));
        tracker.track_change("age", Bson::Int32(30));

        let mut document = doc! {
            "name": "Bob",
            "age": 31
        };

        tracker.rollback(&mut document);

        assert_eq!(document.get_str("name"), Ok("Alice"));
        assert_eq!(document.get_i32("age"), Ok(30));
        assert!(!tracker.is_modified());
    }

    #[test]
    fn test_reset() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));
        assert!(tracker.is_modified());

        tracker.reset();

        assert!(!tracker.is_modified());
        assert_eq!(tracker.change_count(), 0);
    }

    #[test]
    fn test_get_all_original_data() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));

        let current = doc! {
            "name": "Bob",
            "age": 31,
            "city": "Taipei"
        };

        let original = tracker.get_all_original_data(&current);

        assert_eq!(original.get_str("name"), Ok("Alice")); // Restored
        assert_eq!(original.get_i32("age"), Ok(31)); // Unchanged
        assert_eq!(original.get_str("city"), Ok("Taipei")); // Unchanged
    }

    #[test]
    fn test_nested_field_tracking() {
        let mut tracker = StateTracker::new();

        // Track nested field - should mark parent as dirty
        let original_user = doc! {
            "name": "Alice",
            "address": {
                "city": "Taipei"
            }
        };
        tracker.track_change("user", Bson::Document(original_user.clone()));

        // Checking nested path should check parent
        assert!(tracker.has_changed("user"));
        assert!(tracker.has_changed("user.address"));
        assert!(tracker.has_changed("user.address.city"));
    }

    #[test]
    fn test_changed_field_names() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));
        tracker.track_change("age", Bson::Int32(30));

        let names = tracker.changed_field_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"name"));
        assert!(names.contains(&"age"));
    }

    #[test]
    fn test_track_multiple_fields() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));
        tracker.track_change("age", Bson::Int32(30));
        tracker.track_change("email", Bson::String("alice@example.com".to_string()));

        assert_eq!(tracker.change_count(), 3);
        assert!(tracker.has_changed("name"));
        assert!(tracker.has_changed("age"));
        assert!(tracker.has_changed("email"));
        assert!(!tracker.has_changed("phone"));
    }

    #[test]
    fn test_compare_field_unchanged() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));

        // Current data matches original - no actual change
        let current = doc! { "name": "Alice" };
        assert!(!tracker.compare_field("name", &current));
    }

    #[test]
    fn test_compare_field_changed() {
        let mut tracker = StateTracker::new();
        tracker.track_change("name", Bson::String("Alice".to_string()));

        // Current data differs from original
        let current = doc! { "name": "Bob" };
        assert!(tracker.compare_field("name", &current));
    }

    #[test]
    fn test_compare_field_untracked() {
        let tracker = StateTracker::new();

        // Field not tracked - should return false
        let current = doc! { "name": "Alice" };
        assert!(!tracker.compare_field("name", &current));
    }

    #[test]
    fn test_compare_field_nested() {
        let mut tracker = StateTracker::new();
        tracker.track_change("user", Bson::Document(doc! { "name": "Alice" }));

        // Check nested path - should check top-level field
        let current = doc! { "user": { "name": "Bob" } };
        assert!(tracker.compare_field("user.name", &current));
        assert!(tracker.compare_field("user", &current));
    }
}

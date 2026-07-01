//! Row attribute payloads + a composable filter — the metadata half of a vector
//! database (what separates an ANN *library* from a vector *DB*).
//!
//! Every stored vector row can carry a small typed [`Payload`] (a bag of named
//! scalar [`AttrValue`]s). A [`Filter`] is an AND of [`Clause`]s over those
//! payloads; [`Filter::matches`] decides whether a row survives. Filtered k-NN
//! then returns the top-k among ONLY the matching rows (see
//! [`crate::index::VectorIndex::search_knn_filtered`]).
//!
//! The types are deliberately minimal and typed — one integer and one
//! string/enum attribute is enough for the table-stakes filtered-search feature
//! (`category == 3`, `12 <= bucket <= 40`), and keeps the payload store a plain
//! row-aligned `Vec<Payload>` next to the vectors.

use std::collections::HashMap;

/// A single typed scalar attribute value: a 64-bit integer or a short string
/// (used as an enum-like "tag"). Comparison for [`Clause::Eq`] is exact and
/// type-aware — an `Int(3)` never equals a `Str("3")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
    /// A 64-bit signed integer attribute (e.g. `bucket`, `year`, `count`).
    Int(i64),
    /// A short string / enum tag attribute (e.g. `category`, `lang`).
    Str(String),
}

impl AttrValue {
    /// Convenience constructor for an integer attribute.
    pub fn int(v: i64) -> Self {
        AttrValue::Int(v)
    }

    /// Convenience constructor for a string attribute.
    pub fn str(v: impl Into<String>) -> Self {
        AttrValue::Str(v.into())
    }

    /// The integer payload, or `None` if this value is a string.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            AttrValue::Int(i) => Some(*i),
            AttrValue::Str(_) => None,
        }
    }

    /// The string payload, or `None` if this value is an integer.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AttrValue::Str(s) => Some(s),
            AttrValue::Int(_) => None,
        }
    }
}

impl From<i64> for AttrValue {
    fn from(v: i64) -> Self {
        AttrValue::Int(v)
    }
}

impl From<&str> for AttrValue {
    fn from(v: &str) -> Self {
        AttrValue::Str(v.to_string())
    }
}

impl From<String> for AttrValue {
    fn from(v: String) -> Self {
        AttrValue::Str(v)
    }
}

/// The scalar attributes attached to one vector row. An empty payload (the
/// default given to rows added without one) matches only the empty filter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Payload {
    /// Named attributes for this row. Missing keys make any clause referencing
    /// them fail (a row without a `category` never matches `category == 3`).
    pub tags: HashMap<String, AttrValue>,
}

impl Payload {
    /// An empty payload (no attributes).
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style insert: attach `key = val`, returning `self`. Handy for the
    /// deterministic per-row payloads in tests and the bench
    /// (`Payload::new().with("category", 3i64).with("bucket", 40i64)`).
    pub fn with(mut self, key: impl Into<String>, val: impl Into<AttrValue>) -> Self {
        self.tags.insert(key.into(), val.into());
        self
    }

    /// Insert (or overwrite) `key = val` in place.
    pub fn insert(&mut self, key: impl Into<String>, val: impl Into<AttrValue>) {
        self.tags.insert(key.into(), val.into());
    }

    /// The value stored under `key`, if any.
    pub fn get(&self, key: &str) -> Option<&AttrValue> {
        self.tags.get(key)
    }

    /// Whether this payload has zero attributes.
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}

/// One filter predicate over a [`Payload`]. The building block of a [`Filter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clause {
    /// `payload[key] == val` (exact, type-aware). Fails if `key` is absent.
    Eq(String, AttrValue),
    /// `lo <= payload[key] <= hi`, inclusive, over an [`AttrValue::Int`]. Fails
    /// if `key` is absent or is a string attribute.
    IntRange(String, i64, i64),
}

impl Clause {
    /// Whether `payload` satisfies this single clause.
    pub fn matches(&self, payload: &Payload) -> bool {
        match self {
            Clause::Eq(key, val) => payload.get(key) == Some(val),
            Clause::IntRange(key, lo, hi) => {
                matches!(payload.get(key), Some(AttrValue::Int(i)) if lo <= i && i <= hi)
            }
        }
    }
}

/// A conjunction (AND) of [`Clause`]s. A row matches iff it satisfies EVERY
/// clause. The empty filter (no clauses) matches every row — the natural
/// identity, so an unset filter is a no-op.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// The clauses AND-ed together.
    pub clauses: Vec<Clause>,
}

impl Filter {
    /// An empty filter (matches everything).
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style: AND an equality clause (`key == val`).
    pub fn eq(mut self, key: impl Into<String>, val: impl Into<AttrValue>) -> Self {
        self.clauses.push(Clause::Eq(key.into(), val.into()));
        self
    }

    /// Builder-style: AND an inclusive integer-range clause (`lo <= key <= hi`).
    pub fn int_range(mut self, key: impl Into<String>, lo: i64, hi: i64) -> Self {
        self.clauses.push(Clause::IntRange(key.into(), lo, hi));
        self
    }

    /// Builder-style: AND an arbitrary [`Clause`].
    pub fn and(mut self, clause: Clause) -> Self {
        self.clauses.push(clause);
        self
    }

    /// Whether `payload` satisfies ALL clauses (vacuously true when empty).
    pub fn matches(&self, payload: &Payload) -> bool {
        self.clauses.iter().all(|c| c.matches(payload))
    }

    /// Whether this filter has zero clauses (matches everything).
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(category: i64, bucket: i64) -> Payload {
        Payload::new()
            .with("category", category)
            .with("bucket", bucket)
            .with("lang", "en")
    }

    #[test]
    fn eq_is_type_aware() {
        let p = payload(3, 40);
        assert!(Clause::Eq("category".into(), AttrValue::int(3)).matches(&p));
        assert!(!Clause::Eq("category".into(), AttrValue::int(4)).matches(&p));
        // Int(3) is not Str("3").
        assert!(!Clause::Eq("category".into(), AttrValue::str("3")).matches(&p));
        // String tag equality.
        assert!(Clause::Eq("lang".into(), AttrValue::str("en")).matches(&p));
        // Absent key never matches.
        assert!(!Clause::Eq("missing".into(), AttrValue::int(0)).matches(&p));
    }

    #[test]
    fn int_range_is_inclusive() {
        let p = payload(3, 40);
        assert!(Clause::IntRange("bucket".into(), 40, 40).matches(&p));
        assert!(Clause::IntRange("bucket".into(), 10, 100).matches(&p));
        assert!(!Clause::IntRange("bucket".into(), 41, 100).matches(&p));
        assert!(!Clause::IntRange("bucket".into(), 0, 39).matches(&p));
        // Range over a string attribute or a missing key never matches.
        assert!(!Clause::IntRange("lang".into(), 0, 100).matches(&p));
        assert!(!Clause::IntRange("missing".into(), 0, 100).matches(&p));
    }

    #[test]
    fn filter_is_and_of_clauses() {
        let p = payload(3, 40);
        // Both clauses hold.
        assert!(Filter::new()
            .eq("category", 3i64)
            .int_range("bucket", 0, 50)
            .matches(&p));
        // Second clause fails ⇒ whole filter fails.
        assert!(!Filter::new()
            .eq("category", 3i64)
            .int_range("bucket", 41, 50)
            .matches(&p));
        // Empty filter matches everything.
        assert!(Filter::new().matches(&p));
        assert!(Filter::new().matches(&Payload::new()));
    }
}

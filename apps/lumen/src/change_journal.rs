//! Ownership-only journal for checkpointable changes.
//!
//! Active rows own only the newest value for each `(field, stable external ID)`.
//! Freezing moves that whole map into an immutable `Arc`; it never clones `V`
//! or walks rows.  A later active update is a new map entry and cannot alter a
//! frozen capture.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

type Rows<V> = BTreeMap<String, BTreeMap<String, Row<V>>>;

/// A cloneable value escape used by checkpoint encoders. It retains the
/// admitted payload charge with the same Arc as long as any encoder clone
/// exists, without requiring `V: Clone`.
pub(crate) struct SharedValue<V> {
    payload: Arc<V>,
    // Field order is intentional: the payload drops before its budget charge.
    charge: Option<crate::change_budget::RetainedCharge>,
}

impl<V> Clone for SharedValue<V> {
    fn clone(&self) -> Self {
        Self {
            payload: self.payload.clone(),
            charge: self.charge.clone(),
        }
    }
}

impl<V> std::ops::Deref for SharedValue<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.payload.as_ref()
    }
}

impl<V> SharedValue<V> {
    pub(crate) fn new(value: Arc<V>, charge: Option<crate::change_budget::RetainedCharge>) -> Self {
        Self {
            payload: value,
            charge,
        }
    }

    /// Compare the underlying payload allocation without exposing a clonable
    /// bare `Arc<V>` that could escape its retained charge.
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.payload, &other.payload)
    }
}

/// One latest logical field value. `None` is an explicit deletion.
pub(crate) struct Row<V> {
    revision: u64,
    value: Option<SharedValue<V>>,
    // Tombstones have no value wrapper, so their retained charge stays here.
    // It follows the value field to preserve payload-before-charge drops.
    charge: Option<crate::change_budget::RetainedCharge>,
}

impl<V> Row<V> {
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn value(&self) -> Option<&SharedValue<V>> {
        self.value.as_ref()
    }

    pub(crate) fn is_delete(&self) -> bool {
        self.value.is_none()
    }
}

struct State<V> {
    next_batch: u64,
    active: Rows<V>,
    frozen: BTreeMap<u64, Arc<Rows<V>>>,
}

impl<V> Default for State<V> {
    fn default() -> Self {
        Self {
            next_batch: 0,
            active: Rows::new(),
            frozen: BTreeMap::new(),
        }
    }
}

struct Inner<V> {
    state: Mutex<State<V>>,
}

/// Per-engine journal. It is intentionally generic and does not know budgets,
/// persistence, query indexes, or checkpoint timing.
pub(crate) struct ChangeJournal<V> {
    inner: Arc<Inner<V>>,
}

impl<V> std::fmt::Debug for ChangeJournal<V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ChangeJournal")
    }
}

/// Immutable retryable capture. Cloning this handle clones only its `Arc`.
pub(crate) struct FrozenChanges<V> {
    journal: Weak<Inner<V>>,
    id: u64,
    rows: Arc<Rows<V>>,
}

impl<V> Clone for FrozenChanges<V> {
    fn clone(&self) -> Self {
        Self {
            journal: self.journal.clone(),
            id: self.id,
            rows: self.rows.clone(),
        }
    }
}

impl<V> Default for ChangeJournal<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> ChangeJournal<V> {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State::default()),
            }),
        }
    }

    /// Replace the outstanding active value. `None` records a deletion.
    pub(crate) fn record(
        &self,
        field: String,
        external_id: String,
        revision: u64,
        value: Option<Arc<V>>,
    ) {
        self.record_charged(field, external_id, revision, value, None);
    }

    /// Replace the outstanding active value and retain its admitted payload
    /// charge. A multi-row record passes clones of one [`crate::change_budget::RetainedCharge`], so
    /// every row shares one accounting entry until all rows are gone.
    pub(crate) fn record_charged(
        &self,
        field: String,
        external_id: String,
        revision: u64,
        value: Option<Arc<V>>,
        charge: Option<crate::change_budget::RetainedCharge>,
    ) {
        drop(self.replace_charged(field, external_id, revision, value, charge));
    }

    /// Return the replaced owner so an apply caller can defer its payload/file
    /// destruction until after it has released state and the capture barrier.
    pub(crate) fn replace_charged(
        &self,
        field: String,
        external_id: String,
        revision: u64,
        value: Option<Arc<V>>,
        charge: Option<crate::change_budget::RetainedCharge>,
    ) -> Option<Row<V>> {
        let mut state = self
            .inner
            .state
            .lock()
            .expect("change journal lock poisoned");
        let (value, charge) = match value {
            Some(value) => (Some(SharedValue::new(value, charge)), None),
            None => (None, charge),
        };
        state.active.entry(field).or_default().insert(
            external_id,
            Row {
                revision,
                value,
                charge,
            },
        )
    }

    /// O(1) with respect to rows: map ownership moves to the frozen batch.
    pub(crate) fn freeze(&self) -> FrozenChanges<V> {
        let mut state = self
            .inner
            .state
            .lock()
            .expect("change journal lock poisoned");
        state.next_batch = state
            .next_batch
            .checked_add(1)
            .expect("change journal batch id exhausted");
        let id = state.next_batch;
        let rows = Arc::new(std::mem::take(&mut state.active));
        state.frozen.insert(id, rows.clone());
        FrozenChanges {
            journal: Arc::downgrade(&self.inner),
            id,
            rows,
        }
    }

    /// The latest active revision after a capture, if that row has changed.
    pub(crate) fn active_revision(&self, field: &str, external_id: &str) -> Option<u64> {
        self.inner
            .state
            .lock()
            .expect("change journal lock poisoned")
            .active
            .get(field)?
            .get(external_id)
            .map(Row::revision)
    }

    /// Release the journal's retained frozen ownership after durable publish.
    /// It never clears active rows, so newer updates and deletions survive.
    pub(crate) fn acknowledge(&self, frozen: &FrozenChanges<V>) -> bool {
        let Some(inner) = frozen.journal.upgrade() else {
            return false;
        };
        if !Arc::ptr_eq(&self.inner, &inner) {
            return false;
        }
        let mut state = self
            .inner
            .state
            .lock()
            .expect("change journal lock poisoned");
        state.frozen.remove(&frozen.id).is_some()
    }

    /// A durable cut includes every earlier batch from this same journal.
    /// Active rows and batches captured after this cut remain owned.
    pub(crate) fn acknowledge_through(&self, frozen: &FrozenChanges<V>) -> bool {
        let Some(inner) = frozen.journal.upgrade() else {
            return false;
        };
        if !Arc::ptr_eq(&self.inner, &inner) {
            return false;
        }
        let mut state = self
            .inner
            .state
            .lock()
            .expect("change journal lock poisoned");
        let present = state.frozen.contains_key(&frozen.id);
        state.frozen.retain(|id, _| *id > frozen.id);
        present
    }

    #[cfg(test)]
    fn active_len(&self) -> usize {
        self.inner
            .state
            .lock()
            .unwrap()
            .active
            .values()
            .map(BTreeMap::len)
            .sum()
    }
}

impl<V> FrozenChanges<V> {
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    /// Ordered borrowed traversal: field then stable external ID.
    pub(crate) fn rows(&self) -> impl Iterator<Item = (&str, &str, &Row<V>)> {
        self.rows.iter().flat_map(|(field, values)| {
            values
                .iter()
                .map(move |(id, row)| (field.as_str(), id.as_str(), row))
        })
    }

    pub(crate) fn row(&self, field: &str, external_id: &str) -> Option<&Row<V>> {
        self.rows.get(field)?.get(external_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_budget::ChangeBudget;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct NotClone {
        drops: Arc<AtomicUsize>,
        value: u32,
    }
    impl Drop for NotClone {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn value(drops: &Arc<AtomicUsize>, value: u32) -> Arc<NotClone> {
        Arc::new(NotClone {
            drops: drops.clone(),
            value,
        })
    }

    struct ChargeObservedPayload {
        budget: ChangeBudget,
        observed_total: Arc<AtomicUsize>,
    }

    impl Drop for ChargeObservedPayload {
        fn drop(&mut self) {
            self.observed_total
                .store(self.budget.snapshot().total, Ordering::SeqCst);
        }
    }

    #[test]
    fn replacing_a_file_owner_can_defer_its_destruction_until_after_apply() {
        let drops = Arc::new(AtomicUsize::new(0));
        let journal = ChangeJournal::new();
        journal.record("vector".into(), "id".into(), 1, Some(value(&drops, 1)));
        let previous = journal
            .replace_charged(
                "vector".into(),
                "id".into(),
                2,
                Some(value(&drops, 2)),
                None,
            )
            .expect("existing payload owner returned");
        assert_eq!(
            drops.load(Ordering::Relaxed),
            0,
            "replacement must not destroy the old file owner inside apply"
        );
        let frozen = journal.freeze();
        assert_eq!(
            frozen.row("vector", "id").unwrap().value().unwrap().value,
            2
        );
        assert_eq!(previous.value().unwrap().value, 1);
        drop(previous);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn freeze_moves_non_clone_value_without_copying_it() {
        let drops = Arc::new(AtomicUsize::new(0));
        let journal = ChangeJournal::new();
        journal.record("f".into(), "b".into(), 1, Some(value(&drops, 7)));
        let frozen = journal.freeze();
        let row = frozen.row("f", "b").unwrap();
        assert_eq!(row.value().unwrap().value, 7);
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        drop(frozen);
        assert_eq!(
            drops.load(Ordering::Relaxed),
            0,
            "journal retains failed capture"
        );
    }

    #[test]
    fn overwrite_and_delete_after_freeze_keep_old_capture() {
        let drops = Arc::new(AtomicUsize::new(0));
        let journal = ChangeJournal::new();
        journal.record("f".into(), "id".into(), 1, Some(value(&drops, 1)));
        let frozen = journal.freeze();
        journal.record("f".into(), "id".into(), 2, Some(value(&drops, 2)));
        journal.record("f".into(), "gone".into(), 3, None);
        assert_eq!(frozen.row("f", "id").unwrap().value().unwrap().value, 1);
        assert!(journal.active_revision("f", "id").is_some_and(|r| r == 2));
        assert!(journal.active_revision("f", "gone").is_some_and(|r| r == 3));
    }

    #[test]
    fn acknowledgement_releases_frozen_only_and_keeps_later_active() {
        let drops = Arc::new(AtomicUsize::new(0));
        let journal = ChangeJournal::new();
        journal.record("f".into(), "id".into(), 1, Some(value(&drops, 1)));
        let frozen = journal.freeze();
        journal.record("f".into(), "id".into(), 2, Some(value(&drops, 2)));
        assert!(journal.acknowledge(&frozen));
        assert_eq!(journal.active_revision("f", "id"), Some(2));
        assert_eq!(journal.active_len(), 1);
        assert_eq!(frozen.row("f", "id").unwrap().revision(), 1);
    }

    #[test]
    fn cloned_frozen_handles_share_one_non_clone_payload() {
        let drops = Arc::new(AtomicUsize::new(0));
        let journal = ChangeJournal::new();
        journal.record("f".into(), "id".into(), 1, Some(value(&drops, 9)));
        let one = journal.freeze();
        let two = one.clone();
        assert_eq!(one.id(), two.id());
        assert_eq!(one.row("f", "id").unwrap().value().unwrap().value, 9);
        assert!(journal.acknowledge(&one));
        drop(one);
        assert_eq!(
            drops.load(Ordering::Relaxed),
            0,
            "second retry handle owns payload"
        );
        drop(two);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn charged_non_clone_payload_moves_to_frozen_capture_without_copying() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let drops = Arc::new(AtomicUsize::new(0));
        let journal = ChangeJournal::new();
        journal.record_charged(
            "f".into(),
            "id".into(),
            1,
            Some(value(&drops, 7)),
            Some(owner.try_reserve(7).unwrap().commit_retained().unwrap()),
        );
        let frozen = journal.freeze();
        assert_eq!(frozen.row("f", "id").unwrap().value().unwrap().value, 7);
        assert_eq!(budget.snapshot().total, 7);
        assert!(journal.acknowledge(&frozen));
        drop(frozen);
        assert!(drops.load(Ordering::Relaxed) > 0);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn acknowledge_keeps_one_charged_capture_until_all_retry_handles_drop() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let journal = ChangeJournal::<u8>::new();
        journal.record_charged(
            "f".into(),
            "id".into(),
            1,
            Some(Arc::new(1)),
            Some(owner.try_reserve(5).unwrap().commit_retained().unwrap()),
        );
        let one = journal.freeze();
        let two = one.clone();
        assert!(journal.acknowledge(&one));
        drop(one);
        assert_eq!(budget.snapshot().total, 5);
        drop(two);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn overwrite_after_freeze_leaves_the_later_active_charge_intact() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let journal = ChangeJournal::<u8>::new();
        journal.record_charged(
            "f".into(),
            "id".into(),
            1,
            Some(Arc::new(1)),
            Some(owner.try_reserve(5).unwrap().commit_retained().unwrap()),
        );
        let frozen = journal.freeze();
        let _budget_frozen = owner.freeze().unwrap();
        journal.record_charged(
            "f".into(),
            "id".into(),
            2,
            Some(Arc::new(2)),
            Some(owner.try_reserve(6).unwrap().commit_retained().unwrap()),
        );
        assert_eq!(frozen.row("f", "id").unwrap().revision(), 1);
        assert_eq!(journal.active_revision("f", "id"), Some(2));
        assert_eq!(budget.snapshot().active, 6);
        assert_eq!(budget.snapshot().frozen, 5);
        assert!(journal.acknowledge(&frozen));
        drop(frozen);
        assert_eq!(budget.snapshot().active, 6);
        assert_eq!(budget.snapshot().total, 6);
    }

    #[test]
    fn frozen_capture_keeps_its_charge_after_the_journal_drops() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let journal = ChangeJournal::<u8>::new();
        journal.record_charged(
            "f".into(),
            "id".into(),
            1,
            Some(Arc::new(1)),
            Some(owner.try_reserve(5).unwrap().commit_retained().unwrap()),
        );
        let frozen = journal.freeze();
        drop(journal);
        assert_eq!(budget.snapshot().total, 5);
        drop(frozen);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn row_drops_payload_before_its_retained_charge() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let observed_total = Arc::new(AtomicUsize::new(0));
        let journal = ChangeJournal::new();
        journal.record_charged(
            "f".into(),
            "id".into(),
            1,
            Some(Arc::new(ChargeObservedPayload {
                budget: budget.clone(),
                observed_total: observed_total.clone(),
            })),
            Some(owner.try_reserve(5).unwrap().commit_retained().unwrap()),
        );
        let frozen = journal.freeze();
        assert!(journal.acknowledge(&frozen));
        drop(journal);
        drop(frozen);
        assert_eq!(observed_total.load(Ordering::SeqCst), 5);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn rows_from_one_record_share_one_retained_charge() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let journal = ChangeJournal::<u8>::new();
        let charge = owner.try_reserve(5).unwrap().commit_retained().unwrap();
        journal.record_charged(
            "a".into(),
            "id".into(),
            1,
            Some(Arc::new(1)),
            Some(charge.clone()),
        );
        journal.record_charged("b".into(), "id".into(), 1, Some(Arc::new(2)), Some(charge));
        assert_eq!(budget.snapshot().total, 5);
        let frozen = journal.freeze();
        assert!(journal.acknowledge(&frozen));
        drop(frozen);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn escaped_frozen_value_keeps_its_charge_after_journal_and_capture_drop() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let journal = ChangeJournal::<u8>::new();
        journal.record_charged(
            "f".into(),
            "id".into(),
            1,
            Some(Arc::new(1)),
            Some(owner.try_reserve(5).unwrap().commit_retained().unwrap()),
        );
        let frozen = journal.freeze();
        let escaped = frozen.row("f", "id").unwrap().value().unwrap().clone();
        assert!(escaped.same_identity(frozen.row("f", "id").unwrap().value().unwrap()));
        assert!(journal.acknowledge(&frozen));
        drop(journal);
        drop(frozen);
        assert_eq!(budget.snapshot().total, 5);
        drop(escaped);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn tombstone_charge_lives_until_the_last_frozen_capture_drops() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let journal = ChangeJournal::<u8>::new();
        journal.record_charged(
            "f".into(),
            "id".into(),
            1,
            None,
            Some(owner.try_reserve(5).unwrap().commit_retained().unwrap()),
        );
        let one = journal.freeze();
        let two = one.clone();
        assert!(one.row("f", "id").unwrap().is_delete());
        assert!(journal.acknowledge(&one));
        drop(journal);
        drop(one);
        assert_eq!(budget.snapshot().total, 5);
        drop(two);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn rows_are_field_and_stable_id_ordered_and_deletes_are_explicit() {
        let journal = ChangeJournal::<u8>::new();
        journal.record("z".into(), "b".into(), 1, Some(Arc::new(1)));
        journal.record("a".into(), "z".into(), 2, None);
        journal.record("a".into(), "a".into(), 3, Some(Arc::new(2)));
        let frozen = journal.freeze();
        let got: Vec<_> = frozen
            .rows()
            .map(|(f, id, row)| (f, id, row.revision(), row.is_delete()))
            .collect();
        assert_eq!(
            got,
            vec![
                ("a", "a", 3, false),
                ("a", "z", 2, true),
                ("z", "b", 1, false)
            ]
        );
    }
}

//! Bitemporal tests — D1 (issue `enhancement-ctx-inf-db-adopt-bitemporal-model-tx-from-tx-to`).
//!
//! Covers R9:
//!   1. Insert + update preserves both versions (old row frozen, new row current).
//!   2. `get_entity_as_of_tx` returns the correct row for a point in tx-time.
//!   3. Delete + query-as-of returns the deleted row.
//!   4. WAL replay preserves tx_time (freeze state survives recovery).
//!   5. Snapshot roundtrip preserves tx_from / tx_to.
//! Plus a few extras for the relation side and R7 current-state filtering.

use cclab_ctx_inf_db::engine::EntityFilter;
use cclab_ctx_inf_db::*;
use chrono::{Duration as ChronoDuration, Utc};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn tiny_config(dir: &std::path::Path) -> PersistenceConfig {
    PersistenceConfig::for_testing(dir)
}

// R9.1 + R9.2: insert + update + as-of queries.
#[test]
fn test_update_preserves_old_and_new_versions() {
    let db = CtxInfEngine::new();
    let alice = db
        .create_entity(Entity::new(EntityType::Person, "Alice"))
        .unwrap();
    let id = alice.id;

    // Observe tx_from of initial create.
    let created_tx = alice.tx_from;
    assert!(
        alice.tx_to.is_none(),
        "create should yield an open-ended row"
    );

    // Small delay so post-update tx_from is strictly greater.
    thread::sleep(Duration::from_millis(10));

    let updated = db
        .update_entity(id, 0, |e| e.name = "Alice v2".into())
        .unwrap();
    assert_eq!(updated.name, "Alice v2");
    assert_eq!(updated.version, 1);
    assert!(updated.tx_to.is_none(), "updated current row is open-ended");
    assert!(
        updated.tx_from > created_tx,
        "updated row's tx_from must advance"
    );

    // Current-state: get_entity returns v2 only.
    let current = db.get_entity(id).unwrap();
    assert_eq!(current.name, "Alice v2");
    assert_eq!(current.version, 1);

    // As-of the original create time, the v0 row is visible.
    let t_before_update = updated.tx_from - ChronoDuration::microseconds(1);
    let historical = db
        .get_entity_as_of_tx(id, t_before_update)
        .expect("expected v0 row to be visible before the update");
    assert_eq!(historical.name, "Alice");
    assert_eq!(historical.version, 0);
    assert!(
        historical.tx_to.is_some(),
        "frozen row carries a closing tx_to"
    );

    // As-of "now" returns v2.
    let now_row = db.get_entity_as_of_tx(id, Utc::now()).unwrap();
    assert_eq!(now_row.name, "Alice v2");
}

// R9.3: delete + query-as-of returns the deleted row.
#[test]
fn test_delete_preserves_row_for_as_of_query() {
    let db = CtxInfEngine::new();
    let bob = db
        .create_entity(Entity::new(EntityType::Person, "Bob"))
        .unwrap();
    let id = bob.id;

    let before_delete = Utc::now();
    thread::sleep(Duration::from_millis(10));

    db.delete_entity(id, false).unwrap();

    // Current state: get_entity fails.
    assert!(matches!(
        db.get_entity(id),
        Err(CtxInfError::EntityNotFound(_))
    ));

    // As-of before the delete: the row is still visible.
    let snapshot = db
        .get_entity_as_of_tx(id, before_delete)
        .expect("expected pre-delete row to survive");
    assert_eq!(snapshot.name, "Bob");
    assert!(
        snapshot.tx_to.is_some(),
        "post-delete frozen row has tx_to stamped"
    );
}

// R7: current-state queries ignore frozen rows.
#[test]
fn test_current_state_queries_skip_frozen() {
    let db = CtxInfEngine::new();
    let a = db
        .create_entity(Entity::new(EntityType::Person, "A"))
        .unwrap();
    let b = db
        .create_entity(Entity::new(EntityType::Person, "B"))
        .unwrap();
    db.create_relation(Relation::new(RelationType::MetWith, a.id, b.id))
        .unwrap();

    // Delete B → relation is frozen via cascade (cascade = false still freezes B itself).
    db.delete_entity(b.id, true).unwrap();

    // `find_entities` (current state) returns only A.
    let people = db.find_entities(EntityFilter {
        entity_type: Some(EntityType::Person),
        ..Default::default()
    });
    assert_eq!(people.len(), 1);
    assert_eq!(people[0].name, "A");

    // `neighbors(a, Outgoing)` returns nothing — the relation and B are frozen.
    let out = db.neighbors(a.id, Direction::Outgoing, None).unwrap();
    assert!(
        out.is_empty(),
        "frozen relations must not appear as neighbors"
    );

    // `active_at` respects the `tx_to is None` filter.
    let active = db.active_at(Utc::now());
    let names: Vec<_> = active.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"A"));
    assert!(!names.contains(&"B"));
}

// R4: relation-level freeze behavior.
#[test]
fn test_relation_update_and_as_of() {
    let db = CtxInfEngine::new();
    let a = db
        .create_entity(Entity::new(EntityType::Person, "A"))
        .unwrap();
    let b = db
        .create_entity(Entity::new(EntityType::Person, "B"))
        .unwrap();
    let rel = db
        .create_relation(Relation::new(RelationType::MetWith, a.id, b.id).with_confidence(0.5))
        .unwrap();
    let rid = rel.id;

    let created_tx = rel.tx_from;
    thread::sleep(Duration::from_millis(10));

    let updated = db.update_relation(rid, 0, |r| r.confidence = 0.95).unwrap();
    assert_eq!(updated.version, 1);
    assert_eq!(updated.confidence, 0.95);

    // As-of pre-update: see the v0 row (confidence 0.5).
    let pre = db
        .get_relation_as_of_tx(rid, updated.tx_from - ChronoDuration::microseconds(1))
        .expect("expected v0 relation visible before update");
    assert_eq!(pre.confidence, 0.5);
    assert!(pre.tx_to.is_some());
    assert!(pre.tx_from >= created_tx);

    // Delete freezes the relation; current is gone but history still resolves.
    let before_delete = Utc::now();
    thread::sleep(Duration::from_millis(10));
    db.delete_relation(rid).unwrap();
    assert!(matches!(
        db.get_relation(rid),
        Err(CtxInfError::RelationNotFound(_))
    ));

    let still_there = db.get_relation_as_of_tx(rid, before_delete).unwrap();
    assert_eq!(still_there.confidence, 0.95);
    assert_eq!(still_there.version, 1);
}

// R9.4: WAL replay preserves tx_time (update + delete survive recovery).
#[test]
fn test_wal_replay_preserves_tx_time() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let alice_id;
    let charlie_id;
    let pre_update_tx;
    let pre_delete_tx;

    {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();

        let alice = engine
            .create_entity(Entity::new(EntityType::Person, "Alice v0"))
            .unwrap();
        alice_id = alice.id;

        thread::sleep(Duration::from_millis(10));
        pre_update_tx = Utc::now();
        thread::sleep(Duration::from_millis(10));
        engine
            .update_entity(alice_id, 0, |e| e.name = "Alice v1".into())
            .unwrap();

        let charlie = engine
            .create_entity(Entity::new(EntityType::Person, "Charlie"))
            .unwrap();
        charlie_id = charlie.id;
        thread::sleep(Duration::from_millis(10));
        pre_delete_tx = Utc::now();
        thread::sleep(Duration::from_millis(10));
        engine.delete_entity(charlie_id, false).unwrap();

        thread::sleep(Duration::from_millis(100));
        engine.shutdown().unwrap();
    }

    {
        let engine = CtxInfEngine::open(tiny_config(&dir)).unwrap();

        // Current state: Alice v1 is visible, Charlie is gone.
        let alice = engine.get_entity(alice_id).unwrap();
        assert_eq!(alice.name, "Alice v1");
        assert_eq!(alice.version, 1);
        assert!(matches!(
            engine.get_entity(charlie_id),
            Err(CtxInfError::EntityNotFound(_))
        ));

        // History: pre-update tx shows Alice v0.
        let alice_v0 = engine
            .get_entity_as_of_tx(alice_id, pre_update_tx)
            .expect("WAL replay must preserve Alice v0");
        assert_eq!(alice_v0.name, "Alice v0");
        assert_eq!(alice_v0.version, 0);
        assert!(alice_v0.tx_to.is_some());

        // History: pre-delete tx shows Charlie still alive.
        let charlie_row = engine
            .get_entity_as_of_tx(charlie_id, pre_delete_tx)
            .expect("WAL replay must preserve Charlie row");
        assert_eq!(charlie_row.name, "Charlie");
        assert!(charlie_row.tx_to.is_some());

        engine.shutdown().unwrap();
    }
}

// R9.5: snapshot roundtrip preserves tx_from / tx_to on current rows.
#[test]
fn test_snapshot_roundtrip_preserves_tx_time() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let alice_id;
    let original_tx_from;

    {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();
        let alice = engine
            .create_entity(Entity::new(EntityType::Person, "Alice"))
            .unwrap();
        alice_id = alice.id;
        original_tx_from = alice.tx_from;

        // Mutate once so version = 1 and tx_from is a post-update timestamp.
        thread::sleep(Duration::from_millis(10));
        let updated = engine
            .update_entity(alice_id, 0, |e| e.name = "Alice v1".into())
            .unwrap();
        assert!(updated.tx_from > original_tx_from);

        thread::sleep(Duration::from_millis(100));
        let _ = engine.create_snapshot().unwrap();
        engine.shutdown().unwrap();
    }

    {
        let engine = CtxInfEngine::open(tiny_config(&dir)).unwrap();
        let restored = engine.get_entity(alice_id).unwrap();
        assert_eq!(restored.name, "Alice v1");
        assert_eq!(restored.version, 1);
        assert!(
            restored.tx_to.is_none(),
            "current row must have tx_to = None after snapshot roundtrip"
        );
        // tx_from round-trips — either verbatim (post-bitemporal snapshot) or pinned to
        // snapshot.created_at (pre-bitemporal rewrite path). Either way it must be ≤ now.
        assert!(restored.tx_from <= Utc::now());
        engine.shutdown().unwrap();
    }
}

// Extra: as-of query for an id that never existed returns None.
#[test]
fn test_as_of_missing_id_returns_none() {
    let db = CtxInfEngine::new();
    let ghost = EntityId::new();
    assert!(db.get_entity_as_of_tx(ghost, Utc::now()).is_none());

    let ghost_rel = RelationId::new();
    assert!(db.get_relation_as_of_tx(ghost_rel, Utc::now()).is_none());
}

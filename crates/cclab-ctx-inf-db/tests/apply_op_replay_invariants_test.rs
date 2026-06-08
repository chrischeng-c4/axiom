//! Recovery `apply_op` invariant gap-documentation tests.
//!
//! Pins CURRENT silent-admit behavior of `RecoveryManager::apply_op`
//! for adversarial corrupt-WAL inputs (form b per issue R2a/R2b):
//! `CreateRelation` (recovery.rs:214-232) and `UpdateEntity`
//! (recovery.rs:207-210) bypass the existence checks that public
//! `create_relation` / `update_entity` enforce. `DeleteEntity`
//! (recovery.rs:211-213) goes via `engine.delete_entity` with `let _ =`
//! and is correctly idempotent — pinned as a contrast test. R5: zero
//! prod-code touches. R6: no new deps; frame format mirrored from
//! `cclab_wal::entry::encode_entry` (entry.rs:117-136) — `[u32 BE
//! length][JSON WalEntry][u32 BE CRC32]`. The WAL header is written by
//! the engine on first open, so crafted entries simply append.

use cclab_ctx_inf_db::{
    CtxInfEngine, Entity, EntityId, EntityType, GraphOp, PersistenceConfig, Relation, RelationType,
};
use cclab_wal::WalEntry;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────

/// Frame a `GraphOp` per `cclab_wal::entry::encode_entry` and append to
/// `wal-current.log`. Uses serde_json + crc32fast directly so we never
/// broaden cclab-wal visibility (R6). Caller must ensure the WAL header
/// is already on disk (the engine writes it on first open).
fn append_crafted_wal_entry(wal_path: &Path, op: &GraphOp) {
    let entry = WalEntry::new(op.clone());
    let data = serde_json::to_vec(&entry).expect("serialize WalEntry<GraphOp>");
    let checksum = crc32fast::hash(&data);

    let mut buf = Vec::with_capacity(4 + data.len() + 4);
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(&data);
    buf.extend_from_slice(&checksum.to_be_bytes());

    let mut file = OpenOptions::new()
        .append(true)
        .open(wal_path)
        .expect("open wal-current.log for append");
    file.write_all(&buf).expect("write framed entry");
    file.sync_all().expect("fsync crafted entry");
}

/// Spawn engine on `dir`, insert one Person `name`, flush + drain,
/// shutdown. Returns inserted EntityId.
fn seed_one_entity(dir: &Path, name: &str) -> EntityId {
    let engine = CtxInfEngine::with_persistence(PersistenceConfig::for_testing(dir)).unwrap();
    let e = engine
        .create_entity(Entity::new(EntityType::Person, name))
        .unwrap();
    let id = e.id;
    engine.flush();
    thread::sleep(Duration::from_millis(100));
    engine.shutdown().unwrap();
    id
}

// ── R2a: orphan relation silently admitted ───────────────────────────

/// Invariant: `engine.create_relation` returns `DanglingReference` when
/// source/target don't exist (engine.rs:222-227, mirrored by
/// engine_test.rs::test_relation_dangling_reference).
/// Ignore-state: NOT ignored (form b — pins current behavior).
/// Recovery line exercised: recovery.rs:214-232 (`CreateRelation` arm).
#[test]
fn test_replay_admits_orphan_relation_from_corrupt_wal() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let alice_id = seed_one_entity(&dir, "Alice");

    let phantom_id = EntityId::new();
    let orphan_rel = Relation::new(RelationType::MetWith, alice_id, phantom_id);
    let orphan_rel_id = orphan_rel.id;

    append_crafted_wal_entry(
        &dir.join("wal-current.log"),
        &GraphOp::CreateRelation {
            relation: orphan_rel,
        },
    );

    // INVARIANT GAP (recovery.rs:214): apply_op::CreateRelation does not
    // check that source/target exist before insert + adjacency push.
    // Fix would gate on engine.entities.contains_key for both endpoints.
    // OUT OF SCOPE per R5.
    let (engine, stats) =
        CtxInfEngine::open_with_stats(PersistenceConfig::for_testing(&dir)).unwrap();

    assert_eq!(stats.wal_entries_replayed, 2, "Alice + orphan must replay");
    assert_eq!(stats.corrupted_entries, 0, "valid CRC; not corrupted");
    let s = engine.stats();
    assert_eq!(
        s.relation_count, 1,
        "INVARIANT GAP — orphan relation silently admitted"
    );
    assert_eq!(s.entity_count, 1, "phantom never inserted as entity");
    assert!(
        engine.get_relation(orphan_rel_id).is_ok(),
        "INVARIANT GAP — orphan relation retrievable"
    );

    engine.shutdown().unwrap();
}

// ── R2b: update on nonexistent entity silently admitted ──────────────

/// Invariant: `engine.update_entity` returns `EntityNotFound` when id
/// doesn't exist (engine.rs:131-140).
/// Ignore-state: NOT ignored (form b — pins current behavior).
/// Recovery line exercised: recovery.rs:207-210 (`UpdateEntity` arm).
#[test]
fn test_replay_admits_update_on_nonexistent_entity_from_corrupt_wal() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let _alice_id = seed_one_entity(&dir, "Alice");

    let phantom = Entity::new(EntityType::Person, "PhantomBob");
    let phantom_id = phantom.id;

    append_crafted_wal_entry(
        &dir.join("wal-current.log"),
        &GraphOp::UpdateEntity {
            id: phantom_id,
            entity: phantom,
            frozen_tx_to: chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap(),
        },
    );

    // INVARIANT GAP (recovery.rs:207): apply_op::UpdateEntity inserts
    // unconditionally, materializing a brand-new entity if id never
    // existed. Fix would gate on engine.entities.contains_key(&id).
    // OUT OF SCOPE per R5.
    let (engine, stats) =
        CtxInfEngine::open_with_stats(PersistenceConfig::for_testing(&dir)).unwrap();

    assert_eq!(stats.wal_entries_replayed, 2, "Alice + phantom must replay");
    assert_eq!(stats.corrupted_entries, 0, "valid CRC; not corrupted");
    let s = engine.stats();
    assert_eq!(
        s.entity_count, 2,
        "INVARIANT GAP — phantom materialized by replay"
    );
    let back = engine
        .get_entity(phantom_id)
        .expect("INVARIANT GAP — phantom retrievable post-replay");
    assert_eq!(back.name, "PhantomBob");

    engine.shutdown().unwrap();
}

// ── R2c: delete on nonexistent entity is a noop (CORRECT) ────────────

/// Invariant: replay of `DeleteEntity` for a never-created id MUST be
/// a noop — no panic, no spurious mutations.
/// Ignore-state: NOT ignored (contrast test — pins CORRECT behavior).
/// Recovery line exercised: recovery.rs:211-213 (`DeleteEntity` arm,
/// `let _ = engine.delete_entity(...)` swallows EntityNotFound).
#[test]
fn test_replay_delete_on_nonexistent_entity_is_noop() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let alice_id = seed_one_entity(&dir, "Alice");

    let phantom_id = EntityId::new();
    append_crafted_wal_entry(
        &dir.join("wal-current.log"),
        &GraphOp::DeleteEntity {
            id: phantom_id,
            cascade: false,
            tx_to: chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap(),
        },
    );

    let (engine, stats) =
        CtxInfEngine::open_with_stats(PersistenceConfig::for_testing(&dir)).unwrap();

    assert_eq!(stats.wal_entries_replayed, 2, "Alice + delete must replay");
    assert_eq!(stats.corrupted_entries, 0, "valid CRC; not corrupted");
    let s = engine.stats();
    assert_eq!(s.entity_count, 1, "delete-of-missing is a noop");
    assert!(engine.get_entity(alice_id).is_ok(), "Alice intact");

    engine.shutdown().unwrap();
}

// ── R2d: happy-path create_relation regression guard ─────────────────

/// Invariant: replay of `CreateRelation` whose endpoints DO exist must
/// succeed end-to-end (entities + relation + adjacency rebuilt).
/// Ignore-state: NOT ignored (happy-path regression guard).
/// Recovery line exercised: recovery.rs:214-232 with valid endpoints —
/// guards that any future invariant tightening (per R2a fix) does not
/// regress legitimate replay.
#[test]
fn test_replay_create_relation_with_valid_endpoints_still_works() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let alice_id;
    let bob_id;
    let rel_id;
    {
        let engine = CtxInfEngine::with_persistence(PersistenceConfig::for_testing(&dir)).unwrap();
        let alice = engine
            .create_entity(Entity::new(EntityType::Person, "Alice"))
            .unwrap();
        alice_id = alice.id;
        let bob = engine
            .create_entity(Entity::new(EntityType::Person, "Bob"))
            .unwrap();
        bob_id = bob.id;
        let rel = engine
            .create_relation(Relation::new(RelationType::MetWith, alice_id, bob_id))
            .unwrap();
        rel_id = rel.id;
        engine.flush();
        thread::sleep(Duration::from_millis(100));
        engine.shutdown().unwrap();
    }

    let (engine, stats) =
        CtxInfEngine::open_with_stats(PersistenceConfig::for_testing(&dir)).unwrap();

    assert_eq!(stats.wal_entries_replayed, 3, "2 entities + 1 relation");
    assert_eq!(stats.corrupted_entries, 0);

    assert_eq!(engine.get_entity(alice_id).unwrap().name, "Alice");
    assert_eq!(engine.get_entity(bob_id).unwrap().name, "Bob");
    let rel = engine.get_relation(rel_id).expect("relation replayed");
    assert_eq!(rel.source, alice_id);
    assert_eq!(rel.target, bob_id);

    let s = engine.stats();
    assert_eq!(s.entity_count, 2);
    assert_eq!(s.relation_count, 1);

    engine.shutdown().unwrap();
}

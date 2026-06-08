//! Torn-page / partial-WAL-append / bad-snapshot recovery invariants.
//!
//! Pins `src/storage/recovery.rs` rustdoc contract: "Skip corrupted entries
//! (log warning); stop on first unrecoverable read error". 5 scenarios:
//! R2a truncated-tail WAL, R2b CRC flip mid-entry, R2c leftover `.tmp` snapshot,
//! R2d zero-byte WAL, R2e bad-SHA-256 snapshot payload.
//!
//! R5: no production code changes. If a test reveals a real recovery bug,
//! mark `#[ignore = "blocked on bug-<slug>"]` and file a separate bug.

use cclab_ctx_inf_db::*;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// ── setup + R3 corruption primitives ─────────────────────────────────

fn setup() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();
    (temp, dir)
}

fn tiny_config(dir: &Path) -> PersistenceConfig {
    PersistenceConfig::for_testing(dir)
}

fn truncate_file(path: &Path, new_len: u64) {
    let f = OpenOptions::new().write(true).open(path).unwrap();
    f.set_len(new_len).unwrap();
    f.sync_all().unwrap();
}

fn flip_byte_at(path: &Path, offset: u64) {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    f.seek(SeekFrom::Start(offset)).unwrap();
    let mut buf = [0u8; 1];
    f.read_exact(&mut buf).unwrap();
    buf[0] ^= 0xFF;
    f.seek(SeekFrom::Start(offset)).unwrap();
    f.write_all(&buf).unwrap();
    f.sync_all().unwrap();
}

fn write_garbage_file(path: &Path, bytes: &[u8]) {
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();
    f.write_all(bytes).unwrap();
    f.sync_all().unwrap();
}

// ── WAL layout helpers (inspect-only, no production access) ─────────

/// Walk WAL length-prefixes to find entry N (1-indexed). Returns
/// `(payload_start, payload_len)`. WAL file header is 32 bytes
/// (cclab_wal::WalHeader::SIZE); each entry is `[u32 BE len][data][u32 BE crc]`.
fn find_nth_entry(wal: &Path, n: usize) -> Option<(u64, u32)> {
    assert!(n >= 1);
    let bytes = fs::read(wal).ok()?;
    let mut pos: usize = 32;
    for idx in 1..=n {
        if pos + 4 > bytes.len() {
            return None;
        }
        let len = u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);
        let total = 4 + len as usize + 4;
        if pos + total > bytes.len() {
            return None;
        }
        if idx == n {
            return Some(((pos + 4) as u64, len));
        }
        pos += total;
    }
    None
}

fn write_entities_and_shutdown(dir: &Path, count: usize) -> Vec<EntityId> {
    let engine = CtxInfEngine::with_persistence(tiny_config(dir)).unwrap();
    let mut ids = Vec::with_capacity(count);
    for i in 0..count {
        let e = engine
            .create_entity(Entity::new(EntityType::Person, format!("Person-{:02}", i)))
            .unwrap();
        ids.push(e.id);
    }
    engine.flush();
    thread::sleep(Duration::from_millis(150));
    engine.shutdown().unwrap();
    ids
}

fn current_wal(dir: &Path) -> PathBuf {
    dir.join("wal-current.log")
}

fn latest_snap(dir: &Path) -> PathBuf {
    let mut snaps: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with("snapshot-") && s.ends_with(".snap"))
                .unwrap_or(false)
        })
        .collect();
    snaps.sort();
    snaps.pop().expect(".snap file must exist")
}

// ── R2a: truncated last WAL entry ───────────────────────────────────

#[test]
fn test_recovery_survives_truncated_last_wal_entry() {
    let (_temp, dir) = setup();
    let ids = write_entities_and_shutdown(&dir, 10);

    let wal = current_wal(&dir);
    let (payload_start, payload_len) = find_nth_entry(&wal, 10).expect("10th entry");
    assert!(payload_len > 2);
    truncate_file(&wal, payload_start + u64::from(payload_len) / 2);

    let (engine, _stats) = CtxInfEngine::open_with_stats(tiny_config(&dir)).unwrap();
    let people = engine.entities_by_type(&EntityType::Person);
    let count = people.len();
    assert!(
        (9..=10).contains(&count),
        "expected 9 or 10 entities, got {}",
        count
    );
    for e in &people {
        assert!(!e.name.is_empty());
        assert!(matches!(e.entity_type, EntityType::Person));
    }
    for id in ids.iter().take(9) {
        assert!(engine.get_entity(*id).is_ok(), "early entity must survive");
    }
    engine.shutdown().unwrap();
}

// ── R2b: CRC flip mid-entry — invariant under skip OR stop (R7) ─────

#[test]
fn test_recovery_skips_crc_mismatch_entry() {
    let (_temp, dir) = setup();
    let _ids = write_entities_and_shutdown(&dir, 10);

    let wal = current_wal(&dir);
    let (payload_start, payload_len) = find_nth_entry(&wal, 5).expect("5th entry");
    assert!(payload_len > 2);
    flip_byte_at(&wal, payload_start + u64::from(payload_len) / 2);

    let (engine, _stats) = CtxInfEngine::open_with_stats(tiny_config(&dir)).unwrap();
    let people = engine.entities_by_type(&EntityType::Person);
    let count = people.len();

    // R7: range-only. Skip yields 9; stop yields 4; either acceptable.
    assert!(
        count > 0 && count < 10,
        "expected 0 < count < 10 (skip→9 / stop→4), got {}",
        count
    );

    // Structural invariant: no garbled entity ever surfaces.
    for e in &people {
        assert!(!e.name.is_empty());
        assert!(matches!(e.entity_type, EntityType::Person));
        assert!(
            e.name.starts_with("Person-"),
            "surfaced entity must match write pattern, got {:?}",
            e.name
        );
    }
    engine.shutdown().unwrap();
}

// ── R2c: leftover .tmp snapshot must be ignored ─────────────────────

#[test]
fn test_recovery_ignores_leftover_tmp_snapshot() {
    let (_temp, dir) = setup();

    let ids = {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();
        let mut ids = Vec::new();
        for i in 0..5 {
            let e = engine
                .create_entity(Entity::new(EntityType::Person, format!("Person-{:02}", i)))
                .unwrap();
            ids.push(e.id);
        }
        engine.flush();
        thread::sleep(Duration::from_millis(100));
        let _ = engine.create_snapshot().unwrap();
        engine.shutdown().unwrap();
        ids
    };

    // Plausible header (CISN magic + version=1) to tempt a naive loader.
    let stray = dir.join("snapshot-99999999999.tmp");
    let mut hdr = Vec::with_capacity(80);
    hdr.extend_from_slice(b"CISN");
    hdr.extend_from_slice(&1u32.to_be_bytes());
    hdr.extend_from_slice(&0i64.to_be_bytes());
    hdr.extend_from_slice(&99u64.to_be_bytes()); // lie about entity count
    hdr.extend_from_slice(&0u64.to_be_bytes());
    hdr.extend_from_slice(&0u64.to_be_bytes());
    hdr.extend_from_slice(&[0u8; 32]);
    hdr.extend_from_slice(b"garbage payload must never be loaded");
    write_garbage_file(&stray, &hdr);

    let (engine, stats) = CtxInfEngine::open_with_stats(tiny_config(&dir)).unwrap();
    assert!(stats.snapshot_loaded, ".snap must load");
    assert_eq!(stats.snapshot_entities, 5);
    let people = engine.entities_by_type(&EntityType::Person);
    assert_eq!(people.len(), 5);
    for id in &ids {
        assert!(engine.get_entity(*id).is_ok());
    }
    assert!(stray.exists(), "recovery must not delete .tmp debris");
    engine.shutdown().unwrap();
}

// ── R2d: zero-byte WAL file — no panic ──────────────────────────────

#[test]
fn test_recovery_handles_empty_wal_file() {
    let (_temp, dir) = setup();
    fs::create_dir_all(&dir).unwrap();

    // Dash prefix so cclab_wal::find_wal_files picks it up.
    let wal = dir.join("wal-00000001.log");
    write_garbage_file(&wal, &[]);
    assert_eq!(fs::metadata(&wal).unwrap().len(), 0);

    let (engine, stats) = RecoveryManager::recover(&dir).expect("zero-byte WAL must not abort");
    assert_eq!(stats.wal_entries_replayed, 0);
    assert!(!stats.snapshot_loaded);
    assert_eq!(engine.stats().entity_count, 0);
    assert_eq!(engine.stats().relation_count, 0);
}

// ── R2e: bad-SHA-256 snapshot must be refused ───────────────────────

#[test]
fn test_recovery_rejects_snapshot_with_bad_sha256() {
    let (_temp, dir) = setup();

    {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();
        for i in 0..5 {
            engine
                .create_entity(Entity::new(EntityType::Person, format!("Person-{:02}", i)))
                .unwrap();
        }
        engine.flush();
        thread::sleep(Duration::from_millis(100));
        let _ = engine.create_snapshot().unwrap();
        engine.shutdown().unwrap();
    }

    // Snapshot header is 72 bytes; flip at offset 80 to clearly land in payload.
    let snap = latest_snap(&dir);
    assert!(fs::metadata(&snap).unwrap().len() > 100);
    flip_byte_at(&snap, 80);

    // Acceptable: Err OR Ok with snapshot_loaded=false (fallback). Forbidden:
    // panic, or snapshot_loaded=true (silently loaded corrupt state).
    match CtxInfEngine::open_with_stats(tiny_config(&dir)) {
        Ok((engine, stats)) => {
            assert!(
                !stats.snapshot_loaded,
                "corrupt snapshot must not be silently loaded"
            );
            let people = engine.entities_by_type(&EntityType::Person);
            for e in &people {
                assert!(!e.name.is_empty());
                assert!(e.name.starts_with("Person-"));
                assert!(matches!(e.entity_type, EntityType::Person));
            }
            engine.shutdown().unwrap();
        }
        Err(_) => {} // explicit refusal ok
    }
}

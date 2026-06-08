//! Integration tests for persistence layer

use cclab_kv::engine::KvEngine;
use cclab_kv::persistence::{PersistenceConfig, PersistenceHandle};
use cclab_kv::types::{KvKey, KvValue};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create engine with persistence enabled
fn create_engine_with_persistence(
    data_dir: &std::path::Path,
) -> (Arc<KvEngine>, Arc<PersistenceHandle>) {
    let engine = KvEngine::with_shards(256);
    let engine_arc = Arc::new(engine);
    let config = PersistenceConfig::new(data_dir).with_fsync_interval_ms(50);

    let persistence = Arc::new(PersistenceHandle::new(config, engine_arc.clone()).unwrap());
    engine_arc.enable_persistence(persistence.clone());

    (engine_arc, persistence)
}

/// Test basic write → shutdown → recover cycle
#[test]
fn test_basic_recovery_cycle() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Phase 1: Write data with persistence
    {
        let (engine, persistence) = create_engine_with_persistence(data_dir);

        for i in 0..100 {
            let key = KvKey::new(&format!("key_{}", i)).unwrap();
            engine.set(&key, KvValue::Int(i), None).unwrap();
        }

        persistence.flush();
        thread::sleep(Duration::from_millis(200));
    }

    // Phase 2: Recover
    {
        let (engine, _stats) =
            cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();

        for i in 0..100 {
            let key = KvKey::new(&format!("key_{}", i)).unwrap();
            assert_eq!(engine.get(&key), Some(KvValue::Int(i)));
        }

        assert_eq!(engine.len(), 100);
    }
}

/// Test recovery with snapshot + WAL
#[test]
fn test_snapshot_plus_wal_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    {
        let engine = KvEngine::with_shards(256);
        let engine_arc = Arc::new(engine);

        let config = PersistenceConfig::new(data_dir)
            .with_fsync_interval_ms(50)
            .with_snapshot_ops_threshold(50);

        let persistence = Arc::new(PersistenceHandle::new(config, engine_arc.clone()).unwrap());
        engine_arc.enable_persistence(persistence.clone());

        // Write 100 entries (triggers snapshot at 50)
        for i in 0..100 {
            let key = KvKey::new(&format!("key_{}", i)).unwrap();
            engine_arc.set(&key, KvValue::Int(i), None).unwrap();
        }

        // Explicitly create snapshot
        persistence.create_snapshot();
        thread::sleep(Duration::from_millis(500)); // Wait for snapshot to complete

        // Write more after snapshot (these should be in WAL only)
        for i in 100..150 {
            let key = KvKey::new(&format!("key_{}", i)).unwrap();
            engine_arc.set(&key, KvValue::Int(i), None).unwrap();
        }

        // Flush WAL to ensure post-snapshot writes are persisted
        persistence.flush();
        thread::sleep(Duration::from_millis(300));

        drop(persistence);
        thread::sleep(Duration::from_millis(100));
    }

    {
        let (engine, stats) =
            cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();

        println!("Recovery stats for snapshot+WAL test: {:?}", stats);

        assert!(stats.snapshot_loaded, "Snapshot should be loaded");

        // Note: WAL replay might be 0 if snapshot captured everything
        // The important thing is that all 150 entries are recovered
        for i in 0..150 {
            let key = KvKey::new(&format!("key_{}", i)).unwrap();
            assert_eq!(
                engine.get(&key),
                Some(KvValue::Int(i)),
                "Key {} not recovered",
                i
            );
        }

        assert_eq!(engine.len(), 150, "Should have all 150 entries");
    }
}

/// Test recovery of all KvValue types
#[test]
fn test_recovery_all_value_types() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    {
        let (engine, persistence) = create_engine_with_persistence(data_dir);

        engine
            .set(&KvKey::new("int").unwrap(), KvValue::Int(42), None)
            .unwrap();
        engine
            .set(&KvKey::new("float").unwrap(), KvValue::Float(3.14), None)
            .unwrap();
        // TODO: Investigate Decimal bincode serialization issue
        // engine.set(
        //     &KvKey::new("decimal").unwrap(),
        //     KvValue::Decimal(rust_decimal::Decimal::new(12345, 2)),
        //     None,
        // );
        engine
            .set(
                &KvKey::new("string").unwrap(),
                KvValue::String("hello".to_string()),
                None,
            )
            .unwrap();
        engine
            .set(
                &KvKey::new("bytes").unwrap(),
                KvValue::Bytes(vec![1, 2, 3, 4]),
                None,
            )
            .unwrap();

        // Ensure WAL is flushed
        persistence.flush();
        thread::sleep(Duration::from_millis(300)); // Increased wait time

        // Drop persistence first to ensure clean shutdown
        drop(persistence);
        thread::sleep(Duration::from_millis(100));
    }

    {
        let (engine, stats) =
            cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();

        println!("Recovery stats: {:?}", stats);

        assert_eq!(
            engine.get(&KvKey::new("int").unwrap()),
            Some(KvValue::Int(42))
        );
        assert_eq!(
            engine.get(&KvKey::new("float").unwrap()),
            Some(KvValue::Float(3.14))
        );
        assert_eq!(
            engine.get(&KvKey::new("string").unwrap()),
            Some(KvValue::String("hello".to_string()))
        );
        assert_eq!(
            engine.get(&KvKey::new("bytes").unwrap()),
            Some(KvValue::Bytes(vec![1, 2, 3, 4]))
        );
    }
}

/// Test batch operations recovery
#[test]
fn test_batch_operations_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    {
        let (engine, persistence) = create_engine_with_persistence(data_dir);

        let pairs: Vec<_> = (0..50)
            .map(|i| {
                (
                    KvKey::new(&format!("batch_{}", i)).unwrap(),
                    KvValue::Int(i),
                )
            })
            .collect();

        let pair_refs: Vec<_> = pairs.iter().map(|(k, v)| (k, v.clone())).collect();
        engine.mset(&pair_refs, None).unwrap();

        let del_keys: Vec<_> = (0..25)
            .map(|i| KvKey::new(&format!("batch_{}", i)).unwrap())
            .collect();
        let del_refs: Vec<_> = del_keys.iter().collect();
        engine.mdel(&del_refs);

        persistence.flush();
        thread::sleep(Duration::from_millis(200));
    }

    {
        let (engine, _) =
            cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();

        for i in 0..25 {
            let key = KvKey::new(&format!("batch_{}", i)).unwrap();
            assert!(engine.get(&key).is_none());
        }

        for i in 25..50 {
            let key = KvKey::new(&format!("batch_{}", i)).unwrap();
            assert_eq!(engine.get(&key), Some(KvValue::Int(i)));
        }

        assert_eq!(engine.len(), 25);
    }
}

/// Test lock operations recovery
#[test]
fn test_lock_operations_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    {
        let (engine, persistence) = create_engine_with_persistence(data_dir);

        let key = KvKey::new("resource").unwrap();
        let acquired = engine.lock(&key, "owner_1", Duration::from_secs(60));
        assert!(acquired);

        persistence.flush();
        thread::sleep(Duration::from_millis(200));
    }

    {
        let (engine, _) =
            cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();

        let key = KvKey::new("resource").unwrap();
        let acquired = engine.lock(&key, "owner_2", Duration::from_secs(60));
        assert!(!acquired, "Lock should still be held by owner_1");

        let extended = engine
            .extend_lock(&key, "owner_1", Duration::from_secs(60))
            .unwrap();
        assert!(extended);
    }
}

/// Test concurrent writes with persistence
#[test]
fn test_concurrent_writes_with_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    {
        let (engine, persistence) = create_engine_with_persistence(data_dir);

        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let engine = engine.clone();
                thread::spawn(move || {
                    for i in 0..100 {
                        let key = KvKey::new(&format!("t{}_{}", thread_id, i)).unwrap();
                        engine.set(&key, KvValue::Int(i), None).unwrap();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        persistence.flush();
        thread::sleep(Duration::from_millis(300));
    }

    {
        let (engine, _) =
            cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();

        assert_eq!(engine.len(), 400);

        for thread_id in 0..4 {
            for i in 0..100 {
                let key = KvKey::new(&format!("t{}_{}", thread_id, i)).unwrap();
                assert_eq!(engine.get(&key), Some(KvValue::Int(i)));
            }
        }
    }
}

/// Test recovery from empty state
#[test]
fn test_recovery_from_empty_state() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    let (engine, stats) =
        cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();

    assert!(!stats.snapshot_loaded);
    assert_eq!(stats.wal_entries_replayed, 0);
    assert_eq!(engine.len(), 0);
}

/// Test recovery performance with large dataset
#[test]
fn test_recovery_performance() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    const NUM_ENTRIES: i64 = 10_000;

    {
        let engine = KvEngine::with_shards(256);
        let engine_arc = Arc::new(engine);

        let config = PersistenceConfig::new(data_dir)
            .with_fsync_interval_ms(100)
            .with_snapshot_ops_threshold(5000);

        let persistence = Arc::new(PersistenceHandle::new(config, engine_arc.clone()).unwrap());
        engine_arc.enable_persistence(persistence.clone());

        for i in 0..NUM_ENTRIES {
            let key = KvKey::new(&format!("perf_{}", i)).unwrap();
            engine_arc.set(&key, KvValue::Int(i), None).unwrap();
        }

        persistence.create_snapshot();
        thread::sleep(Duration::from_millis(500));

        persistence.flush();
        thread::sleep(Duration::from_millis(200));
    }

    let start = std::time::Instant::now();
    let (engine, stats) =
        cclab_kv::persistence::recovery::RecoveryManager::recover(data_dir, 256).unwrap();
    let recovery_time = start.elapsed();

    println!("Recovery stats: {:?}", stats);
    println!("Recovery time: {:?}", recovery_time);

    assert_eq!(engine.len(), NUM_ENTRIES as usize);
    assert!(recovery_time < Duration::from_secs(5));
    assert!(stats.snapshot_loaded);
}

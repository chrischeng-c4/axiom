// HANDWRITE-BEGIN gap="missing-generator:unit-test:42f5646b" tracker="pending-tracker" reason="R1/R2: snapshot_bytes round-trips the engine's key/value set, and run_backup_once writes then prunes an artifact at a file:// destination."
//! keep backup adoption tests (#776).
//!
//! - R1: `snapshot_bytes` serializes the engine's live key/value set into a
//!   versioned `KeepSnapshot` that round-trips.
//! - R2: a `file://` destination resolves to a `LocalFsSink`; `run_backup_once`
//!   writes the payload as an artifact and `RetentionPolicy` pruning removes
//!   aged objects. `run_backup` exercises the full recover → serialize → sink
//!   path the `keep backup` verb / CronJob runner uses.
//!
//! @spec projects/keep/tech-design/logic/adopt-libs-service-backup-snapshot-sink-keep-backup-verb.md

use std::time::SystemTime;

use keep::backup::{
    run_backup, run_backup_once, sink_from_destination, snapshot_bytes, BackupDestination,
    KeepSnapshot, RetentionPolicy, KEEP_SNAPSHOT_VERSION,
};
use keep::{KvEngine, KvKey, KvValue};

/// R1 — `snapshot_bytes` captures the engine's live key/value set into a
/// versioned payload that deserializes back to the same values.
#[test]
fn snapshot_bytes_round_trips_engine_state() {
    let engine = KvEngine::new();
    engine
        .set(
            &KvKey::new("alpha").unwrap(),
            KvValue::String("one".into()),
            None,
        )
        .unwrap();
    engine
        .set(&KvKey::new("beta").unwrap(), KvValue::Int(2), None)
        .unwrap();

    let bytes = snapshot_bytes(&engine).unwrap();
    let snap: KeepSnapshot = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(snap.version, KEEP_SNAPSHOT_VERSION);
    assert_eq!(snap.values.len(), 2);
    assert!(snap
        .values
        .iter()
        .any(|(k, v)| k == "alpha" && *v == KvValue::String("one".into())));
    assert!(snap
        .values
        .iter()
        .any(|(k, v)| k == "beta" && *v == KvValue::Int(2)));
}

/// R2 — a `file://` destination writes an artifact via `run_backup_once`, and
/// `RetentionPolicy` age pruning (`BackupSink::prune`) removes the aged object.
#[test]
fn local_backup_writes_then_prunes() {
    let dir = tempfile::tempdir().unwrap();
    let uri = format!("file://{}", dir.path().display());
    let dest = BackupDestination::from_uri(&uri).unwrap();
    let sink = sink_from_destination(&dest).unwrap();

    let payload = br#"{"version":1,"values":[]}"#;
    let result = run_backup_once(
        sink.as_ref(),
        SystemTime::now(),
        payload,
        &RetentionPolicy::default(),
    )
    .unwrap();
    assert_eq!(result.object.bytes, payload.len());
    assert_eq!(result.pruned, 0, "default retention prunes nothing");
    let artifact = dir.path().join(&result.object.key);
    assert!(artifact.exists(), "backup artifact should be written");

    // Age pruning with max_age_seconds=0 removes every existing object.
    std::thread::sleep(std::time::Duration::from_millis(10));
    let removed = sink.prune(0).unwrap();
    assert_eq!(removed, 1, "retention prunes the aged artifact");
    assert!(!artifact.exists(), "pruned artifact is gone");
}

/// R1+R2 end-to-end — `run_backup` recovers a consistent snapshot from a data
/// dir (empty here) and writes it to a local destination, exactly as the
/// `keep backup` verb does.
#[test]
fn run_backup_end_to_end_local() {
    let data_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();
    let dest =
        BackupDestination::from_uri(&format!("file://{}", dest_dir.path().display())).unwrap();

    let result = run_backup(data_dir.path(), 4, &dest, &RetentionPolicy::default()).unwrap();

    let artifact = dest_dir.path().join(&result.object.key);
    assert!(artifact.exists(), "run_backup writes an artifact");
    let snap: KeepSnapshot = serde_json::from_slice(&std::fs::read(&artifact).unwrap()).unwrap();
    assert_eq!(snap.version, KEEP_SNAPSHOT_VERSION);
    assert!(
        snap.values.is_empty(),
        "an empty data dir yields an empty snapshot"
    );
}
// HANDWRITE-END

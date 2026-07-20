// HANDWRITE-BEGIN gap="missing-generator:unit-test:eaf1d20e" tracker="#1585" reason="Exercise exact file backup seeding, canonical marker/snapshot recovery, empty-directory enforcement, and malformed-seed rejection without a live restore endpoint. generator gap: missing-generator:bootstrap-integration-test (#1585)."
//! Cold backup seed conformance (#1585). These tests use the exact snapshot
//! bytes `/admin/backup` emits and the same marker/snapshot paths a real
//! `TapeRaft` startup restores; no live restore endpoint exists or is needed.

use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use raft_runtime::Membership;
use tape::raft::{prepare_bootstrap_seed, snapshot_bytes, TapeRaft};
use tape::TapeJournal;

#[tokio::test]
async fn file_snapshot_seeds_fresh_pvc_before_raft_catch_up() {
    let source = Arc::new(Mutex::new(TapeJournal::default()));
    source
        .lock()
        .unwrap()
        .append("orders", None, serde_json::json!({ "n": 7 }), Some(100));
    let bytes = snapshot_bytes(&source, 42).unwrap();

    let dir = tempfile::tempdir().unwrap();
    prepare_bootstrap_seed(dir.path(), 3, &bytes).unwrap();

    let restored = Arc::new(Mutex::new(TapeJournal::default()));
    let raft = TapeRaft::spawn(
        Arc::clone(&restored),
        &dir.path().join("raft"),
        3,
        Membership {
            voters: vec![3],
            learners: Vec::new(),
        },
        HashMap::new(),
        TapeRaft::host_config(8),
    )
    .unwrap();
    assert_eq!(raft.applied_index(), 42);
    let events = restored.lock().unwrap().replay("orders", None, None, None);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].payload["n"], 7);
}

#[test]
fn dirty_pvc_is_rejected_without_overwriting_local_state() {
    let source = Arc::new(Mutex::new(TapeJournal::default()));
    let bytes = snapshot_bytes(&source, 0).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let sentinel = dir.path().join("existing-state");
    std::fs::write(&sentinel, b"keep").unwrap();

    let err = prepare_bootstrap_seed(dir.path(), 0, &bytes).unwrap_err();
    assert!(err.to_string().contains("empty data directory"), "{err:#}");
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"keep");
    assert!(!dir.path().join("raft").exists());
}

#[test]
fn malformed_seed_is_rejected_before_creating_raft_files() {
    let dir = tempfile::tempdir().unwrap();
    let err = prepare_bootstrap_seed(dir.path(), 1, b"not-json").unwrap_err();
    assert!(
        err.to_string().contains("decode bootstrap JournalSnapshot"),
        "{err:#}"
    );
    assert!(
        std::fs::read_dir(dir.path()).unwrap().next().is_none(),
        "invalid bytes must not create partial raft state"
    );
}
// HANDWRITE-END

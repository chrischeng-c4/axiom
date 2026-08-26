// HANDWRITE-BEGIN gap="missing-generator:unit-test:eaf1d20e" tracker="#1585" reason="Exercise exact file backup seeding, canonical marker/snapshot recovery, empty-directory enforcement, and malformed-seed rejection without a live restore endpoint. generator gap: missing-generator:bootstrap-integration-test (#1585)."
//! Cold backup seed conformance (#1585). These tests use the exact snapshot
//! bytes `/admin/backup` emits and the same marker/snapshot paths a real
//! `TapeRaft` startup restores; no live restore endpoint exists or is needed.

use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use raft_runtime::Membership;
use tape::raft::{data_dir_has_existing_state, prepare_bootstrap_seed, snapshot_bytes, TapeRaft};
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

/// #2443: a freshly provisioned cloud PV mounts its ext4 filesystem root at
/// the data dir, so `lost+found` exists on every real PVC. It is mkfs output,
/// not raft state, and must not block a cold seed.
#[test]
fn ext4_lost_and_found_alone_is_not_raft_state() {
    let source = Arc::new(Mutex::new(TapeJournal::default()));
    source
        .lock()
        .unwrap()
        .append("orders", None, serde_json::json!({ "n": 9 }), Some(100));
    let bytes = snapshot_bytes(&source, 7).unwrap();

    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("lost+found")).unwrap();
    prepare_bootstrap_seed(dir.path(), 2, &bytes).unwrap();
    assert!(dir.path().join("raft").exists());
}

/// #2443: `lost+found` is the ONLY tolerated entry — anything else alongside
/// it is still treated as existing raft state and refused.
#[test]
fn lost_and_found_plus_any_other_entry_is_still_rejected() {
    let source = Arc::new(Mutex::new(TapeJournal::default()));
    let bytes = snapshot_bytes(&source, 0).unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("lost+found")).unwrap();
    let sentinel = dir.path().join("existing-state");
    std::fs::write(&sentinel, b"keep").unwrap();

    let err = prepare_bootstrap_seed(dir.path(), 0, &bytes).unwrap_err();
    assert!(err.to_string().contains("empty data directory"), "{err:#}");
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"keep");
    assert!(!dir.path().join("raft").exists());
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

/// #2468: `data_dir_has_existing_state` is the exact probe a restarting pod
/// must consult before deciding whether to seed at all — it has to agree
/// with `prepare_bootstrap_seed`'s own refusal on every shape: absent dir,
/// `lost+found`-only dir (#2443, still "empty"), and a genuinely populated
/// dir.
#[test]
fn existing_state_probe_agrees_with_prepare_bootstrap_seed_on_every_shape() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist-yet");
    assert!(!data_dir_has_existing_state(&missing).unwrap());

    let fresh = dir.path().join("fresh");
    std::fs::create_dir(&fresh).unwrap();
    std::fs::create_dir(fresh.join("lost+found")).unwrap();
    assert!(!data_dir_has_existing_state(&fresh).unwrap());

    let populated = dir.path().join("populated");
    std::fs::create_dir(&populated).unwrap();
    std::fs::create_dir(populated.join("lost+found")).unwrap();
    std::fs::create_dir(populated.join("raft")).unwrap();
    assert!(data_dir_has_existing_state(&populated).unwrap());
}

/// #2468: `bootstrapSeedUri` lives on the CR and is injected into every
/// pod's env, so a routine restart onto this pod's OWN already-seeded PVC
/// must not crash-loop on `prepare_bootstrap_seed`'s refusal. The caller
/// (`tape serve`) consults `data_dir_has_existing_state` first and skips the
/// reseed entirely when it is true; this proves the probe correctly detects
/// the post-seed dir, and that leaving the seed alone (never re-invoking
/// `prepare_bootstrap_seed`) still boots the durable journal intact.
#[tokio::test]
async fn restart_onto_already_seeded_dir_is_detected_so_the_caller_can_skip_reseeding() {
    let source = Arc::new(Mutex::new(TapeJournal::default()));
    source
        .lock()
        .unwrap()
        .append("orders", None, serde_json::json!({ "n": 5 }), Some(100));
    let bytes = snapshot_bytes(&source, 11).unwrap();

    let dir = tempfile::tempdir().unwrap();

    // First boot: fresh PVC, bootstrapSeedUri set -> the caller's probe sees
    // no existing state and proceeds to seed exactly as today.
    assert!(!data_dir_has_existing_state(dir.path()).unwrap());
    prepare_bootstrap_seed(dir.path(), 5, &bytes).unwrap();

    // Second boot (pod replacement onto the SAME PVC, CR still carries the
    // field): the probe must now report existing state so the caller skips
    // the fetch+seed instead of crash-looping.
    assert!(data_dir_has_existing_state(dir.path()).unwrap());

    // Skipping the reseed (the caller never calls prepare_bootstrap_seed
    // again) must leave the durable journal fully intact and bootable.
    let restored = Arc::new(Mutex::new(TapeJournal::default()));
    let raft = TapeRaft::spawn(
        Arc::clone(&restored),
        &dir.path().join("raft"),
        5,
        Membership {
            voters: vec![5],
            learners: Vec::new(),
        },
        HashMap::new(),
        TapeRaft::host_config(8),
    )
    .unwrap();
    assert_eq!(raft.applied_index(), 11);
    let events = restored.lock().unwrap().replay("orders", None, None, None);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].payload["n"], 5);

    // prepare_bootstrap_seed itself still refuses on this now-populated dir
    // -- the last line of defense stays intact even though the restart path
    // no longer calls it.
    let err = prepare_bootstrap_seed(dir.path(), 5, &bytes).unwrap_err();
    assert!(err.to_string().contains("empty data directory"), "{err:#}");
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

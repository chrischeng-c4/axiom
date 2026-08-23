use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use tempfile::TempDir;

use raft_core::{EntryKind, PersistedState, RaftEntry};
use raft_runtime::{FsyncPolicy, RaftStore};

fn find_artifact(dir_path: &std::path::Path) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".artifact") {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn count_artifacts(dir_path: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".artifact") {
                    count += 1;
                }
            }
        }
    }
    count
}

#[test]
fn measurement_1_hard_state_size_bounded_and_independent_of_snapshot() {
    let dir1 = TempDir::new().unwrap();
    let store1 = RaftStore::open(dir1.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let snap_1m = vec![0x11; 1024 * 1024];
    let state1 = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![RaftEntry {
            term: 1,
            index: 2,
            command: vec![1, 2, 3, 4],
            kind: EntryKind::Command,
        }],
        commit_index: 2,
        snapshot_index: 1,
        snapshot_term: 1,
        snapshot: snap_1m,
        conf: None,
    };
    store1.save(&state1).unwrap();
    let len1 = std::fs::metadata(store1.path()).unwrap().len();

    let dir2 = TempDir::new().unwrap();
    let store2 = RaftStore::open(dir2.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let snap_8m = vec![0x22; 8 * 1024 * 1024];
    let state2 = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![RaftEntry {
            term: 1,
            index: 2,
            command: vec![1, 2, 3, 4],
            kind: EntryKind::Command,
        }],
        commit_index: 2,
        snapshot_index: 1,
        snapshot_term: 1,
        snapshot: snap_8m,
        conf: None,
    };
    store2.save(&state2).unwrap();
    let len2 = std::fs::metadata(store2.path()).unwrap().len();

    assert!(
        len1 <= 4096,
        "store1 hard-state file length {len1} exceeds 4096"
    );
    assert!(
        len2 <= 4096,
        "store2 hard-state file length {len2} exceeds 4096"
    );
    let diff = if len1 > len2 {
        len1 - len2
    } else {
        len2 - len1
    };
    assert!(
        diff < 512,
        "hard-state lengths differ by {diff} bytes (expected < 512)"
    );
}

#[test]
fn measurement_2_log_append_does_not_rewrite_snapshot_artifact() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let snap = vec![0xAA; 1024 * 1024];
    let state = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![],
        commit_index: 1,
        snapshot_index: 1,
        snapshot_term: 1,
        snapshot: snap.clone(),
        conf: None,
    };
    store.save(&state).unwrap();

    let art_path = find_artifact(dir.path()).expect("artifact file must exist");
    let art_meta_before = std::fs::metadata(&art_path).unwrap();
    let art_ino_before = art_meta_before.ino();
    let art_len_before = art_meta_before.len();
    let hard_state_len_before = std::fs::metadata(store.path()).unwrap().len();

    // Save one more 64-byte log entry with unchanged snapshot
    let state2 = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![RaftEntry {
            term: 1,
            index: 2,
            command: vec![0x55; 64],
            kind: EntryKind::Command,
        }],
        commit_index: 2,
        snapshot_index: 1,
        snapshot_term: 1,
        snapshot: snap,
        conf: None,
    };
    store.save(&state2).unwrap();

    let art_meta_after = std::fs::metadata(&art_path).unwrap();
    let art_ino_after = art_meta_after.ino();
    let art_len_after = art_meta_after.len();
    let hard_state_len_after = std::fs::metadata(store.path()).unwrap().len();

    assert_eq!(
        art_ino_before, art_ino_after,
        "artifact inode must be unchanged"
    );
    assert_eq!(
        art_len_before, art_len_after,
        "artifact length must be unchanged"
    );
    let growth = hard_state_len_after.saturating_sub(hard_state_len_before);
    assert!(
        growth < 512,
        "hard-state file grew by {growth} bytes (expected < 512)"
    );
}

#[test]
fn measurement_3_fault_before_publish_recovers_last_complete_generation() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let gen1_bytes = vec![0x11; 1024];
    let state_gen1 = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![],
        commit_index: 1,
        snapshot_index: 1,
        snapshot_term: 1,
        snapshot: gen1_bytes.clone(),
        conf: None,
    };
    store.save(&state_gen1).unwrap();

    // Arm fault after artifact is written, before reference is published
    store.inject_next_after_artifact_failure_with_kind(ErrorKind::StorageFull);

    let gen2_bytes = vec![0x22; 1024];
    let state_gen2 = PersistedState {
        term: 2,
        voted_for: Some(1),
        log: vec![],
        commit_index: 2,
        snapshot_index: 2,
        snapshot_term: 2,
        snapshot: gen2_bytes,
        conf: None,
    };
    let res = store.save(&state_gen2);
    assert!(res.is_err(), "save must fail when fault is armed");

    // Open a fresh RaftStore on the same directory
    let fresh_store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();
    let loaded = fresh_store
        .load()
        .unwrap()
        .expect("fresh store must load state");

    assert_eq!(loaded.snapshot_index, 1);
    assert_eq!(loaded.snapshot_term, 1);
    assert_eq!(loaded.snapshot, gen1_bytes);
}

#[test]
fn measurement_4_fault_after_publish_retains_durable_generation_and_collects_later() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let gen1_bytes = vec![0x11; 512];
    store
        .save(&PersistedState {
            term: 1,
            voted_for: Some(1),
            log: vec![],
            commit_index: 1,
            snapshot_index: 1,
            snapshot_term: 1,
            snapshot: gen1_bytes,
            conf: None,
        })
        .unwrap();

    // Arm fault after reference published, before collection
    store.inject_next_after_publish_failure_with_kind(ErrorKind::StorageFull);

    let gen2_bytes = vec![0x22; 512];
    let res = store.save(&PersistedState {
        term: 2,
        voted_for: Some(1),
        log: vec![],
        commit_index: 2,
        snapshot_index: 2,
        snapshot_term: 2,
        snapshot: gen2_bytes.clone(),
        conf: None,
    });
    assert!(res.is_err(), "save must return Err when fault is armed");

    // Collection is the last step, so at this seam both generations' artifacts
    // are still on disk. Without this the row cannot tell "collection has not
    // run yet" from "collection already ran before the reference was durable" —
    // the second is an ordering defect that leaves the durable reference naming
    // an artifact that has been deleted.
    assert_eq!(
        count_artifacts(dir.path()),
        2,
        "collection must not have run yet at the after-publish seam"
    );

    // A fresh store loads generation 2 because the hard-state reference was already durable
    let fresh_store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();
    let loaded_gen2 = fresh_store
        .load()
        .unwrap()
        .expect("fresh store must load state");
    assert_eq!(loaded_gen2.snapshot_index, 2);
    assert_eq!(loaded_gen2.snapshot_term, 2);
    assert_eq!(loaded_gen2.snapshot, gen2_bytes);

    // Now publish generation 3 cleanly
    let gen3_bytes = vec![0x33; 512];
    fresh_store
        .save(&PersistedState {
            term: 3,
            voted_for: Some(1),
            log: vec![],
            commit_index: 3,
            snapshot_index: 3,
            snapshot_term: 3,
            snapshot: gen3_bytes.clone(),
            conf: None,
        })
        .unwrap();

    // Exactly one artifact remains in the directory
    assert_eq!(
        count_artifacts(dir.path()),
        1,
        "dir must hold exactly one artifact after generation 3"
    );
    let art_path = find_artifact(dir.path()).unwrap();
    let art_bytes = std::fs::read(art_path).unwrap();
    assert_eq!(art_bytes, gen3_bytes);
}

#[test]
fn measurement_5_missing_artifact_fails_identically_regardless_of_log_size() {
    // Case A: 2-entry log
    let dir_a = TempDir::new().unwrap();
    let store_a = RaftStore::open(dir_a.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let snap_a = vec![0xAA; 512];
    store_a
        .save(&PersistedState {
            term: 3,
            voted_for: Some(1),
            log: vec![
                RaftEntry {
                    term: 3,
                    index: 6,
                    command: vec![1, 2, 3],
                    kind: EntryKind::Command,
                },
                RaftEntry {
                    term: 3,
                    index: 7,
                    command: vec![4, 5, 6],
                    kind: EntryKind::Command,
                },
            ],
            commit_index: 7,
            snapshot_index: 5,
            snapshot_term: 3,
            snapshot: snap_a,
            conf: None,
        })
        .unwrap();

    let art_a = find_artifact(dir_a.path()).expect("artifact must exist in dir A");
    std::fs::remove_file(art_a).unwrap();

    let fresh_a = RaftStore::open(dir_a.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();
    let err_a = fresh_a.load().unwrap_err();
    assert_eq!(err_a.kind(), ErrorKind::InvalidData);
    let msg_a = err_a.to_string();
    assert!(
        msg_a.contains("missing snapshot artifact for index 5 term 3"),
        "error message {msg_a} must name missing artifact"
    );

    // Case B: 1 MiB log
    let dir_b = TempDir::new().unwrap();
    let store_b = RaftStore::open(dir_b.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let snap_b = vec![0xAA; 512];
    store_b
        .save(&PersistedState {
            term: 3,
            voted_for: Some(1),
            log: vec![RaftEntry {
                term: 3,
                index: 6,
                command: vec![0x99; 1024 * 1024],
                kind: EntryKind::Command,
            }],
            commit_index: 6,
            snapshot_index: 5,
            snapshot_term: 3,
            snapshot: snap_b,
            conf: None,
        })
        .unwrap();

    let art_b = find_artifact(dir_b.path()).expect("artifact must exist in dir B");
    std::fs::remove_file(art_b).unwrap();

    let fresh_b = RaftStore::open(dir_b.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();
    let err_b = fresh_b.load().unwrap_err();
    assert_eq!(err_b.kind(), ErrorKind::InvalidData);
    let msg_b = err_b.to_string();
    assert!(
        msg_b.contains("missing snapshot artifact for index 5 term 3"),
        "error message {msg_b} must name missing artifact"
    );

    assert_eq!(
        err_a.kind(),
        err_b.kind(),
        "error kinds must be identical across both log sizes"
    );
    assert_eq!(
        msg_a, msg_b,
        "error messages must be identical across both log sizes"
    );
}

#[test]
fn measurement_6_corrupted_content_same_length_fails_load() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let snap = vec![0x55; 1024];
    store
        .save(&PersistedState {
            term: 1,
            voted_for: Some(1),
            log: vec![],
            commit_index: 1,
            snapshot_index: 1,
            snapshot_term: 1,
            snapshot: snap,
            conf: None,
        })
        .unwrap();

    let art_path = find_artifact(dir.path()).expect("artifact must exist");
    // Overwrite the artifact with same length (1024 bytes) but different content
    let corrupted_content = vec![0x66; 1024];
    std::fs::write(&art_path, corrupted_content).unwrap();

    let fresh_store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();
    let res = fresh_store.load();
    assert!(
        res.is_err(),
        "load must fail with Err when artifact content is corrupted"
    );
    assert_eq!(
        res.unwrap_err().kind(),
        ErrorKind::InvalidData,
        "error kind must be InvalidData"
    );
}

#[test]
fn measurement_7_superseded_artifacts_collected_leaving_only_latest() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let gen1_bytes = vec![0x10; 256];
    store
        .save(&PersistedState {
            term: 1,
            voted_for: Some(1),
            log: vec![],
            commit_index: 1,
            snapshot_index: 1,
            snapshot_term: 1,
            snapshot: gen1_bytes,
            conf: None,
        })
        .unwrap();

    let gen2_bytes = vec![0x20; 256];
    store
        .save(&PersistedState {
            term: 2,
            voted_for: Some(1),
            log: vec![],
            commit_index: 2,
            snapshot_index: 2,
            snapshot_term: 2,
            snapshot: gen2_bytes,
            conf: None,
        })
        .unwrap();

    let gen3_bytes = vec![0x30; 256];
    store
        .save(&PersistedState {
            term: 3,
            voted_for: Some(1),
            log: vec![],
            commit_index: 3,
            snapshot_index: 3,
            snapshot_term: 3,
            snapshot: gen3_bytes.clone(),
            conf: None,
        })
        .unwrap();

    assert_eq!(
        count_artifacts(dir.path()),
        1,
        "exactly one artifact must remain in the directory"
    );

    let art_path = find_artifact(dir.path()).unwrap();
    let art_bytes = std::fs::read(art_path).unwrap();
    assert_eq!(
        art_bytes, gen3_bytes,
        "surviving artifact bytes must equal generation 3 snapshot byte-for-byte"
    );
}

#[test]
fn measurement_8_legacy_inline_snapshot_loads_without_artifact() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Os).unwrap();

    let snap_bytes = vec![0x42; 128];
    // Manually construct a RAFTST01 hard-state file with inline snapshot bytes (Round B format)
    let mut buf = Vec::new();
    buf.extend_from_slice(b"RAFTST01");
    buf.extend_from_slice(&2u64.to_le_bytes()); // term = 2
    buf.push(1); // voted_for = Some(1)
    buf.extend_from_slice(&1u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes()); // commit_index = 1
    buf.extend_from_slice(&1u64.to_le_bytes()); // snapshot_index = 1
    buf.extend_from_slice(&2u64.to_le_bytes()); // snapshot_term = 2
    buf.extend_from_slice(&(snap_bytes.len() as u64).to_le_bytes()); // snapshot_len = 128
    buf.extend_from_slice(&snap_bytes); // inline snapshot bytes!
    buf.extend_from_slice(&0u64.to_le_bytes()); // log_len = 0

    std::fs::write(store.path(), buf).unwrap();

    assert_eq!(
        count_artifacts(dir.path()),
        0,
        "no artifact file must exist in directory"
    );

    let loaded = store
        .load()
        .unwrap()
        .expect("legacy inline state must load successfully");

    assert_eq!(loaded.snapshot_index, 1);
    assert_eq!(loaded.snapshot_term, 2);
    assert_eq!(loaded.snapshot, snap_bytes);
}

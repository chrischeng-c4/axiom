use std::io::ErrorKind;
use tempfile::TempDir;

use raft_core::{EntryKind, PersistedState, RaftEntry};
use raft_runtime::{FsyncPolicy, RaftStore};

fn pseudo_random_bytes(len: usize) -> Vec<u8> {
    let mut state: u64 = 0xdeadbeefcafebabe;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push((state >> 33) as u8);
    }
    out
}

#[test]
fn measurement_1_command_bytes_bounded_size() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();

    let command = pseudo_random_bytes(1024 * 1024);
    let state = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![RaftEntry {
            term: 1,
            index: 1,
            command,
            kind: EntryKind::Command,
        }],
        commit_index: 1,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: None,
    };

    store.save(&state).unwrap();

    let meta = std::fs::metadata(store.path()).unwrap();
    let file_len = meta.len();
    assert!(
        file_len <= 1024 * 1024 + 4096,
        "file length {file_len} exceeds 1 MiB + 4096"
    );
    assert!(file_len >= 1024 * 1024);
}

#[test]
fn measurement_2_snapshot_bytes_bounded_size() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();

    let snapshot = pseudo_random_bytes(1024 * 1024);
    let state = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![],
        commit_index: 1,
        snapshot_index: 1,
        snapshot_term: 1,
        snapshot,
        conf: None,
    };

    store.save(&state).unwrap();

    let total_dir_len: u64 = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|entry| entry.metadata().map(|m| m.len()).unwrap_or(0))
        .sum();

    assert!(
        total_dir_len <= 1024 * 1024 + 4096,
        "total dir length {total_dir_len} exceeds 1 MiB + 4096"
    );
    assert!(total_dir_len >= 1024 * 1024);
}

#[test]
fn measurement_3_payload_byte_exactness() {
    let payloads = vec![
        vec![0x00; 1024 * 1024],
        vec![0xFF; 1024 * 1024],
        b"\"\\\n".to_vec(),
        vec![0xC3, 0x28],
        vec![],
        vec![0x42],
    ];

    for payload in payloads {
        let dir = TempDir::new().unwrap();
        let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();

        let state = PersistedState {
            term: 2,
            voted_for: Some(3),
            log: vec![RaftEntry {
                term: 2,
                index: 1,
                command: payload.clone(),
                kind: EntryKind::Command,
            }],
            commit_index: 1,
            snapshot_index: 0,
            snapshot_term: 0,
            snapshot: payload.clone(),
            conf: None,
        };

        store.save(&state).unwrap();

        let loaded = store.load().unwrap().expect("state must load");
        assert_eq!(loaded.term, state.term);
        assert_eq!(loaded.voted_for, state.voted_for);
        assert_eq!(loaded.commit_index, state.commit_index);
        assert_eq!(loaded.snapshot_index, state.snapshot_index);
        assert_eq!(loaded.snapshot_term, state.snapshot_term);
        assert_eq!(loaded.snapshot, payload);
        assert_eq!(loaded.log.len(), 1);
        assert_eq!(loaded.log[0].command, payload);
        assert_eq!(loaded, state);
    }
}

#[test]
fn measurement_4_cache_footprint_bounded_and_invariant() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();

    assert_eq!(store.cache_footprint(), 0);

    let state_1k = PersistedState {
        term: 1,
        voted_for: None,
        log: vec![],
        commit_index: 0,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![0xAA; 1024],
        conf: None,
    };
    store.save(&state_1k).unwrap();

    let footprint_1k = store.cache_footprint();
    assert!(
        footprint_1k > 0 && footprint_1k <= 128,
        "footprint_1k is {footprint_1k}"
    );

    let state_1m = PersistedState {
        term: 1,
        voted_for: None,
        log: vec![],
        commit_index: 0,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![0xBB; 1024 * 1024],
        conf: None,
    };
    store.save(&state_1m).unwrap();

    let footprint_1m = store.cache_footprint();
    assert_eq!(
        footprint_1k, footprint_1m,
        "cache footprint must be identical for 1 KiB and 1 MiB saves"
    );
}

#[test]
fn measurement_5_dedup_and_fault_injection_interaction() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();

    let state_a = PersistedState {
        term: 1,
        voted_for: Some(1),
        log: vec![RaftEntry {
            term: 1,
            index: 1,
            command: vec![1, 2, 3],
            kind: EntryKind::Command,
        }],
        commit_index: 1,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: None,
    };

    store.save(&state_a).unwrap();

    store.inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    // Identical save must short-circuit through dedup without consuming the armed fault injection.
    let identical_save = store.save(&state_a);
    assert!(
        identical_save.is_ok(),
        "identical save must return Ok via dedup"
    );

    let state_b = PersistedState {
        term: 2,
        voted_for: Some(1),
        log: vec![RaftEntry {
            term: 2,
            index: 1,
            command: vec![1, 2, 3],
            kind: EntryKind::Command,
        }],
        commit_index: 1,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: None,
    };

    // Different save must not short-circuit, consuming the fault injection and returning StorageFull.
    let diff_save = store.save(&state_b);
    assert!(diff_save.is_err(), "different save must trigger injection");
    assert_eq!(diff_save.unwrap_err().kind(), ErrorKind::StorageFull);
}

#[test]
fn measurement_6_legacy_json_backward_compatibility() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();

    let legacy_json = br#"{"term":3,"voted_for":2,"log":[{"term":3,"index":1,"command":[10,20,30]}],"commit_index":1,"snapshot_index":1,"snapshot_term":2,"snapshot":[40,50,60]}"#;
    std::fs::write(store.path(), legacy_json).unwrap();

    let loaded = store.load().unwrap().expect("legacy state must load");
    let expected = PersistedState {
        term: 3,
        voted_for: Some(2),
        log: vec![RaftEntry {
            term: 3,
            index: 1,
            command: vec![10, 20, 30],
            kind: EntryKind::Command,
        }],
        commit_index: 1,
        snapshot_index: 1,
        snapshot_term: 2,
        snapshot: vec![40, 50, 60],
        conf: None,
    };

    assert_eq!(loaded, expected);
}

#[test]
fn measurement_7_unrecognised_format_marker_refusal() {
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();

    let future_format_bytes = b"RAFTST99\x01\x02\x03\x04\x05\x06\x07\x08";
    std::fs::write(store.path(), future_format_bytes).unwrap();

    let res = store.load();
    assert!(
        res.is_err(),
        "unrecognised format marker must return an Err"
    );
    assert_eq!(
        res.unwrap_err().kind(),
        ErrorKind::InvalidData,
        "error kind must be InvalidData"
    );
}

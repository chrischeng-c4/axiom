use std::io;
use tempfile::TempDir;

use raft_core::{PersistedState, RaftEntry};
use raft_runtime::{group::GroupId, FsyncPolicy, RaftStore};

#[test]
fn measurement_1_migrate_legacy_json_to_named_group() {
    let dir = TempDir::new().unwrap();
    let legacy_file = dir.path().join("raft-7.state");

    let legacy_json = serde_json::json!({
        "term": 3,
        "voted_for": 2,
        "log": [
            {
                "term": 3,
                "index": 1,
                "command": [10, 20, 30]
            }
        ],
        "commit_index": 1,
        "snapshot_index": 0,
        "snapshot_term": 0,
        "snapshot": []
    });
    std::fs::write(&legacy_file, serde_json::to_vec(&legacy_json).unwrap()).unwrap();

    let store = RaftStore::migrate_legacy_to_group(
        dir.path().to_str().unwrap(),
        7,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    assert!(store.path().ends_with("raft-7-616c706861.state"));
    assert!(!legacy_file.exists());
    assert!(dir.path().join("raft-7-616c706861.state").exists());

    let state = store.load().unwrap().expect("migrated state should load");
    assert_eq!(state.term, 3);
    assert_eq!(state.voted_for, Some(2));
    assert_eq!(state.commit_index, 1);
    assert_eq!(state.snapshot_index, 0);
    assert_eq!(state.snapshot_term, 0);
    assert_eq!(state.snapshot, Vec::<u8>::new());
    assert_eq!(
        state.log,
        vec![RaftEntry {
            term: 3,
            index: 1,
            command: vec![10, 20, 30],
        }]
    );
}

#[test]
fn measurement_2_migrate_legacy_store_with_snapshot_artifact() {
    let dir = TempDir::new().unwrap();
    let snap_payload = vec![100, 101, 102, 103, 104, 105, 106, 107];

    {
        let store = RaftStore::open(dir.path().to_str().unwrap(), 7, FsyncPolicy::Os).unwrap();
        store
            .save(&PersistedState {
                term: 2,
                voted_for: Some(1),
                log: vec![],
                commit_index: 3,
                snapshot_index: 3,
                snapshot_term: 2,
                snapshot: snap_payload.clone(),
            })
            .unwrap();
    }

    assert!(dir.path().join("raft-7.state").exists());
    assert!(dir.path().join("raft-7-snap-3-2.artifact").exists());

    let store = RaftStore::migrate_legacy_to_group(
        dir.path().to_str().unwrap(),
        7,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    assert!(store.path().ends_with("raft-7-616c706861.state"));
    assert!(!dir.path().join("raft-7.state").exists());
    assert!(!dir.path().join("raft-7-snap-3-2.artifact").exists());
    assert!(dir.path().join("raft-7-616c706861.state").exists());
    assert!(dir
        .path()
        .join("raft-7-616c706861-snap-3-2.artifact")
        .exists());

    let state = store
        .load()
        .unwrap()
        .expect("migrated store should load snapshot state");
    assert_eq!(state.term, 2);
    assert_eq!(state.voted_for, Some(1));
    assert_eq!(state.commit_index, 3);
    assert_eq!(state.snapshot_index, 3);
    assert_eq!(state.snapshot_term, 2);
    assert_eq!(state.snapshot, snap_payload);
}

#[test]
fn measurement_3_open_named_group_refuses_when_legacy_state_file_exists() {
    let dir = TempDir::new().unwrap();
    let legacy_file = dir.path().join("raft-7.state");

    let legacy_json = serde_json::json!({
        "term": 3,
        "voted_for": 2,
        "log": [],
        "commit_index": 1,
        "snapshot_index": 0,
        "snapshot_term": 0,
        "snapshot": []
    });
    let original_bytes = serde_json::to_vec(&legacy_json).unwrap();
    std::fs::write(&legacy_file, &original_bytes).unwrap();

    let res = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        7,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    );

    let err = res
        .err()
        .expect("open_group must fail when legacy state exists");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(
        err.to_string().contains("raft-7.state"),
        "error message '{}' must contain literal 'raft-7.state'",
        err
    );

    assert!(legacy_file.exists());
    let current_bytes = std::fs::read(&legacy_file).unwrap();
    assert_eq!(current_bytes, original_bytes);
}

#[test]
fn measurement_4_migrate_refuses_when_target_already_exists() {
    let dir = TempDir::new().unwrap();
    let legacy_file = dir.path().join("raft-7.state");
    let target_file = dir.path().join("raft-7-616c706861.state");

    let legacy_json = serde_json::json!({
        "term": 3,
        "voted_for": 2,
        "log": [],
        "commit_index": 1,
        "snapshot_index": 0,
        "snapshot_term": 0,
        "snapshot": []
    });
    let target_json = serde_json::json!({
        "term": 4,
        "voted_for": 3,
        "log": [],
        "commit_index": 2,
        "snapshot_index": 0,
        "snapshot_term": 0,
        "snapshot": []
    });

    let orig_legacy_bytes = serde_json::to_vec(&legacy_json).unwrap();
    let orig_target_bytes = serde_json::to_vec(&target_json).unwrap();

    std::fs::write(&legacy_file, &orig_legacy_bytes).unwrap();
    std::fs::write(&target_file, &orig_target_bytes).unwrap();

    let res = RaftStore::migrate_legacy_to_group(
        dir.path().to_str().unwrap(),
        7,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    );

    assert!(res.is_err());
    assert_eq!(std::fs::read(&legacy_file).unwrap(), orig_legacy_bytes);
    assert_eq!(std::fs::read(&target_file).unwrap(), orig_target_bytes);
}

#[test]
fn measurement_5_migrate_refuses_when_legacy_file_corrupt() {
    let dir = TempDir::new().unwrap();
    let legacy_file = dir.path().join("raft-7.state");
    let target_file = dir.path().join("raft-7-616c706861.state");

    let mut corrupt_bytes = b"RAFTSTXX".to_vec();
    corrupt_bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    std::fs::write(&legacy_file, &corrupt_bytes).unwrap();

    let res = RaftStore::migrate_legacy_to_group(
        dir.path().to_str().unwrap(),
        7,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    );

    assert!(res.is_err());
    assert_eq!(std::fs::read(&legacy_file).unwrap(), corrupt_bytes);
    assert!(!target_file.exists());
}

#[test]
fn measurement_6_legacy_open_and_other_node_named_open_succeed() {
    let dir = TempDir::new().unwrap();
    let legacy_file = dir.path().join("raft-7.state");

    let legacy_json = serde_json::json!({
        "term": 3,
        "voted_for": 2,
        "log": [
            {
                "term": 3,
                "index": 1,
                "command": [5, 6, 7]
            }
        ],
        "commit_index": 1,
        "snapshot_index": 0,
        "snapshot_term": 0,
        "snapshot": []
    });
    std::fs::write(&legacy_file, serde_json::to_vec(&legacy_json).unwrap()).unwrap();

    let store7 = RaftStore::open(dir.path().to_str().unwrap(), 7, FsyncPolicy::Os).unwrap();
    let state7 = store7
        .load()
        .unwrap()
        .expect("node 7 legacy load should succeed");
    assert_eq!(state7.term, 3);
    assert_eq!(state7.voted_for, Some(2));
    assert_eq!(state7.commit_index, 1);

    let store8 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        8,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    assert_eq!(store8.load().unwrap(), None);
}

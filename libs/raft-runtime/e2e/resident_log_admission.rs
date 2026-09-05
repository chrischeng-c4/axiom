use std::{collections::HashMap, sync::Arc};

use raft_core::{EntryKind, PersistedState, RaftEntry};
use raft_runtime::{FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStore};

#[path = "support/cluster.rs"]
mod cluster;
use cluster::TestSm;

#[tokio::test]
async fn resident_log_limit_backpressures_then_reopens_after_compaction() {
    std::env::set_var("RAFT_RUNTIME_MAX_RESIDENT_LOG_BYTES", "64");
    let data = tempfile::tempdir().unwrap();
    let state_machine = TestSm::new();
    let host = RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(data.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap(),
        state_machine as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    );

    assert_eq!(host.propose(vec![1; 40]).await.unwrap(), 1);
    let error = host.propose(vec![2; 40]).await.unwrap_err();
    assert!(error.to_string().contains("resident log memory limit"));
    host.snapshot_and_compact_through(1).await.unwrap();
    assert_eq!(host.propose(vec![3; 40]).await.unwrap(), 2);
    std::env::remove_var("RAFT_RUNTIME_MAX_RESIDENT_LOG_BYTES");
}

#[tokio::test]
async fn corrupt_referenced_v4_log_refuses_host_startup() {
    let data = tempfile::tempdir().unwrap();
    let store = RaftStore::open(data.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap();
    store
        .save(&PersistedState {
            term: 1,
            voted_for: Some(0),
            log: vec![RaftEntry {
                term: 1,
                index: 1,
                command: vec![7; 128],
                kind: EntryKind::Command,
            }],
            commit_index: 1,
            snapshot_index: 0,
            snapshot_term: 0,
            snapshot: Vec::new(),
            conf: None,
        })
        .unwrap();
    let artifact = std::fs::read_dir(data.path())
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("-log-") && name.ends_with(".artifact"))
        })
        .expect("V4 log artifact");
    let length = std::fs::metadata(&artifact).unwrap().len();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&artifact)
        .unwrap()
        .set_len(length / 2)
        .unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        RaftHost::spawn(
            0,
            Membership {
                voters: vec![0],
                learners: Vec::new(),
            },
            HashMap::new(),
            store,
            TestSm::new() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        )
    }));
    assert!(result.is_err(), "corrupt V4 log must refuse host startup");
}

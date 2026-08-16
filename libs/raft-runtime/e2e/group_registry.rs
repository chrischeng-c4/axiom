use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

use raft_core::VoteReq;
use raft_runtime::{
    group::GroupId, FsyncPolicy, HostConfig, Index, Membership, RaftHost, RaftRegistry,
    RaftStateMachine, RaftStatus, RaftStore, RegistryError,
};

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::bind;

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

struct SequenceSm {
    commands: Mutex<Vec<u64>>,
    applied: AtomicU64,
}

impl SequenceSm {
    fn new() -> Arc<Self> {
        Arc::new(SequenceSm {
            commands: Mutex::new(Vec::new()),
            applied: AtomicU64::new(0),
        })
    }

    fn recorded(&self) -> Vec<u64> {
        self.commands.lock().unwrap().clone()
    }
}

impl RaftStateMachine for SequenceSm {
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()> {
        let val = if command.len() == 8 {
            u64::from_le_bytes(command.try_into().unwrap())
        } else if command.len() == 1 {
            command[0] as u64
        } else {
            0
        };
        self.commands.lock().unwrap().push(val);
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        let cmds = self.commands.lock().unwrap().clone();
        let bytes = serde_json::to_vec(&cmds)?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn std::io::Read) -> anyhow::Result<()> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        if bytes.is_empty() {
            return Ok(());
        }
        let cmds: Vec<u64> = serde_json::from_slice(&bytes)?;
        let last = cmds.len() as u64;
        *self.commands.lock().unwrap() = cmds;
        self.applied.store(last, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

async fn wait_leader(host: &RaftHost) {
    for _ in 0..200 {
        if host.is_leader().await {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("leader was not elected within timeout");
}

#[tokio::test]
async fn row1_multi_group_multiplexing() {
    let dir = TempDir::new().unwrap();
    let sm1 = SequenceSm::new();
    let sm2 = SequenceSm::new();

    let store1 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Always,
    )
    .unwrap();

    let store2 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("beta".to_string()),
        FsyncPolicy::Always,
    )
    .unwrap();

    let host1 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store1,
        sm1.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let host2 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("beta".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store2,
        sm2.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let registry = RaftRegistry::new();
    registry.register(host1.clone()).unwrap();
    registry.register(host2.clone()).unwrap();

    wait_leader(&host1).await;
    wait_leader(&host2).await;

    let (l, url) = bind().await;
    let serve = tokio::spawn({
        let r = registry.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    // Propose 11 and 12 to group alpha via shared listener
    let resp1 = client
        .post(&format!("{}/raft/publish", url))
        .json(&serde_json::json!({
            "group_id": "alpha",
            "command": 11u64.to_le_bytes().to_vec(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp1.status(), reqwest::StatusCode::OK);

    let resp2 = client
        .post(&format!("{}/raft/publish", url))
        .json(&serde_json::json!({
            "group_id": "alpha",
            "command": 12u64.to_le_bytes().to_vec(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), reqwest::StatusCode::OK);

    // Propose 21 to group beta via shared listener
    let resp3 = client
        .post(&format!("{}/raft/publish", url))
        .json(&serde_json::json!({
            "group_id": "beta",
            "command": 21u64.to_le_bytes().to_vec(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp3.status(), reqwest::StatusCode::OK);

    // Row 1 assertions: Exact sequences recorded
    assert_eq!(sm1.recorded(), vec![11, 12]);
    assert_eq!(sm2.recorded(), vec![21]);

    serve.abort();
    drop(host1);
    drop(host2);
}

#[tokio::test]
async fn row2_reopen_from_same_dir_replay_isolation() {
    let dir = TempDir::new().unwrap();

    // First session: propose into both groups
    {
        let sm1 = SequenceSm::new();
        let sm2 = SequenceSm::new();

        let store1 = RaftStore::open_group(
            dir.path().to_str().unwrap(),
            0,
            GroupId("alpha".to_string()),
            FsyncPolicy::Always,
        )
        .unwrap();

        let store2 = RaftStore::open_group(
            dir.path().to_str().unwrap(),
            0,
            GroupId("beta".to_string()),
            FsyncPolicy::Always,
        )
        .unwrap();

        let host1 = Arc::new(RaftHost::spawn_group(
            0,
            GroupId("alpha".to_string()),
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            store1,
            sm1.clone() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        ));

        let host2 = Arc::new(RaftHost::spawn_group(
            0,
            GroupId("beta".to_string()),
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            store2,
            sm2.clone() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        ));

        let registry = RaftRegistry::new();
        registry.register(host1.clone()).unwrap();
        registry.register(host2.clone()).unwrap();

        wait_leader(&host1).await;
        wait_leader(&host2).await;

        let (l, url) = bind().await;
        let serve = tokio::spawn({
            let r = registry.router();
            async move {
                loop {
                    if let Ok((stream, _)) = l.accept().await {
                        let r = r.clone();
                        tokio::spawn(async move {
                            let _ = transport_h2c::server::serve_connection(stream, r).await;
                        });
                    }
                }
            }
        });

        let client = h2c_client();

        let resp1 = client
            .post(&format!("{}/raft/publish", url))
            .json(&serde_json::json!({
                "group_id": "alpha",
                "command": 11u64.to_le_bytes().to_vec(),
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp1.status(), reqwest::StatusCode::OK);

        let resp2 = client
            .post(&format!("{}/raft/publish", url))
            .json(&serde_json::json!({
                "group_id": "alpha",
                "command": 12u64.to_le_bytes().to_vec(),
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp2.status(), reqwest::StatusCode::OK);

        let resp3 = client
            .post(&format!("{}/raft/publish", url))
            .json(&serde_json::json!({
                "group_id": "beta",
                "command": 21u64.to_le_bytes().to_vec(),
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp3.status(), reqwest::StatusCode::OK);

        serve.abort();
    }

    // Row 2: Reopen both groups from the same data dir with fresh state machines
    let sm1_fresh = SequenceSm::new();
    let sm2_fresh = SequenceSm::new();

    let store1_fresh = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Always,
    )
    .unwrap();

    let store2_fresh = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("beta".to_string()),
        FsyncPolicy::Always,
    )
    .unwrap();

    let host1_fresh = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store1_fresh,
        sm1_fresh.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let host2_fresh = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("beta".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store2_fresh,
        sm2_fresh.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    // Assert replayed sequences
    assert_eq!(sm1_fresh.recorded(), vec![11, 12]);
    assert_eq!(sm2_fresh.recorded(), vec![21]);

    drop(host1_fresh);
    drop(host2_fresh);
}

#[tokio::test]
async fn row3_unknown_group_refusal_negative_control() {
    let dir = TempDir::new().unwrap();
    let sm1 = SequenceSm::new();
    let sm2 = SequenceSm::new();

    let store1 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let store2 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("beta".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let host1 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store1,
        sm1.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let host2 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("beta".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store2,
        sm2.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let registry = RaftRegistry::new();
    registry.register(host1.clone()).unwrap();
    registry.register(host2.clone()).unwrap();

    let (l, url) = bind().await;
    let serve = tokio::spawn({
        let r = registry.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let z_before: BTreeMap<String, RaftStatus> = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Unknown group request-vote RPC
    let resp = client
        .post(&format!("{}/raft/request-vote", url))
        .json(&serde_json::json!({
            "group_id": "unregistered_group",
            "from": 99,
            "req": VoteReq {
                term: 999,
                candidate: 99,
                last_log_index: 999,
                last_log_term: 999,
            }
        }))
        .send()
        .await
        .unwrap();

    // Must be 404 NOT_FOUND
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    let z_after: BTreeMap<String, RaftStatus> = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let alpha_before = z_before.get("alpha").unwrap();
    let alpha_after = z_after.get("alpha").unwrap();
    assert_eq!(alpha_before.term, alpha_after.term);
    assert_eq!(alpha_before.last_index, alpha_after.last_index);
    assert_eq!(alpha_before.applied_index, alpha_after.applied_index);

    let beta_before = z_before.get("beta").unwrap();
    let beta_after = z_after.get("beta").unwrap();
    assert_eq!(beta_before.term, beta_after.term);
    assert_eq!(beta_before.last_index, beta_after.last_index);
    assert_eq!(beta_before.applied_index, beta_after.applied_index);

    serve.abort();
}

#[tokio::test]
async fn row4_duplicate_registration_error() {
    let dir = TempDir::new().unwrap();
    let sm1 = SequenceSm::new();
    let store1 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let host1 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store1,
        sm1.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let registry = RaftRegistry::new();
    registry.register(host1.clone()).unwrap();

    let sm_dup = SequenceSm::new();
    let store_dup = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        1,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let host_dup = Arc::new(RaftHost::spawn_group(
        1,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![1],
            learners: vec![],
        },
        HashMap::new(),
        store_dup,
        sm_dup.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let reg_err = registry.register(host_dup);
    assert_eq!(
        reg_err,
        Err(RegistryError::AlreadyRegistered(GroupId(
            "alpha".to_string()
        )))
    );

    wait_leader(&host1).await;

    let (l, url) = bind().await;
    let serve = tokio::spawn({
        let r = registry.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let resp = client
        .post(&format!("{}/raft/publish", url))
        .json(&serde_json::json!({
            "group_id": "alpha",
            "command": 42u64.to_le_bytes().to_vec(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    assert_eq!(sm1.recorded(), vec![42]);
    assert_eq!(sm_dup.recorded(), Vec::<u64>::new());

    serve.abort();
}

#[tokio::test]
async fn row5_registry_status_surface() {
    let dir = TempDir::new().unwrap();
    let sm1 = SequenceSm::new();
    let sm2 = SequenceSm::new();

    let store1 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let store2 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("beta".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let host1 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store1,
        sm1.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let host2 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("beta".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store2,
        sm2.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let registry = RaftRegistry::new();
    registry.register(host1.clone()).unwrap();
    registry.register(host2.clone()).unwrap();

    let (l, url) = bind().await;
    let serve = tokio::spawn({
        let r = registry.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    // Propose 2 commands to alpha, 1 command to beta
    host1.propose(100u64.to_le_bytes().to_vec()).await.unwrap();
    host1.propose(101u64.to_le_bytes().to_vec()).await.unwrap();
    host2.propose(200u64.to_le_bytes().to_vec()).await.unwrap();

    let statuses: BTreeMap<String, RaftStatus> = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses.get("alpha").unwrap().applied_index, 2);
    assert_eq!(statuses.get("beta").unwrap().applied_index, 1);

    serve.abort();
}

#[tokio::test]
async fn row6_failure_isolation() {
    let dir = TempDir::new().unwrap();
    let sm1 = SequenceSm::new();
    let sm2 = SequenceSm::new();

    let store1 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let store2 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("beta".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let host1 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store1,
        sm1.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let host2 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("beta".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store2,
        sm2.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let registry = RaftRegistry::new();
    registry.register(host1.clone()).unwrap();
    registry.register(host2.clone()).unwrap();

    wait_leader(&host1).await;
    wait_leader(&host2).await;

    let (l, url) = bind().await;
    let serve = tokio::spawn({
        let r = registry.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    // Arm save failure on group alpha only
    host1
        .store()
        .inject_next_save_failure_with_kind(std::io::ErrorKind::StorageFull);

    // Proposal into alpha fails
    let resp_alpha = client
        .post(&format!("{}/raft/publish", url))
        .json(&serde_json::json!({
            "group_id": "alpha",
            "command": 10u64.to_le_bytes().to_vec(),
        }))
        .send()
        .await
        .unwrap();
    // 503 is the host's own latched-durability refusal. Naming the literal
    // keeps this row able to tell it apart from the registry's 404 for an
    // unknown group and the single-host router's 400 for a foreign one;
    // `assert_ne!(.., OK)` accepts all three and so separates none of them.
    assert_eq!(resp_alpha.status().as_u16(), 503);

    // Proposal into beta succeeds
    let resp_beta = client
        .post(&format!("{}/raft/publish", url))
        .json(&serde_json::json!({
            "group_id": "beta",
            "command": 20u64.to_le_bytes().to_vec(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp_beta.status(), reqwest::StatusCode::OK);

    // Status check
    let statuses: BTreeMap<String, RaftStatus> = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(statuses.get("alpha").unwrap().durability_error.is_some());
    assert!(statuses.get("beta").unwrap().durability_error.is_none());

    serve.abort();
}

#[tokio::test]
async fn row7_snapshot_compaction_isolation() {
    let dir = TempDir::new().unwrap();
    let sm1 = SequenceSm::new();
    let sm2 = SequenceSm::new();

    let store1 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let store2 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("beta".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let host1 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store1,
        sm1.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let host2 = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("beta".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store2,
        sm2.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let registry = RaftRegistry::new();
    registry.register(host1.clone()).unwrap();
    registry.register(host2.clone()).unwrap();

    let (l, url) = bind().await;
    let serve = tokio::spawn({
        let r = registry.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    // Propose commands
    host1.propose(1u64.to_le_bytes().to_vec()).await.unwrap();
    host1.propose(2u64.to_le_bytes().to_vec()).await.unwrap();
    host2.propose(50u64.to_le_bytes().to_vec()).await.unwrap();

    let beta_path = host2.store().path();
    let meta_before = std::fs::metadata(beta_path).unwrap();
    #[cfg(unix)]
    let inode_before = {
        use std::os::unix::fs::MetadataExt;
        meta_before.ino()
    };
    let len_before = meta_before.len();
    let applied_before = host2.applied_watch().borrow().clone();

    // Snapshot and compact group alpha
    host1.snapshot_and_compact().await.unwrap();

    // Group beta inode and length unchanged
    let meta_after = std::fs::metadata(beta_path).unwrap();
    #[cfg(unix)]
    let inode_after = {
        use std::os::unix::fs::MetadataExt;
        meta_after.ino()
    };
    let len_after = meta_after.len();
    let applied_after = host2.applied_watch().borrow().clone();

    #[cfg(unix)]
    assert_eq!(inode_before, inode_after);
    assert_eq!(len_before, len_after);
    assert_eq!(applied_before, applied_after);

    // Proposal into beta through shared listener still succeeds
    let resp_beta = client
        .post(&format!("{}/raft/publish", url))
        .json(&serde_json::json!({
            "group_id": "beta",
            "command": 51u64.to_le_bytes().to_vec(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp_beta.status(), reqwest::StatusCode::OK);
    assert_eq!(sm2.recorded(), vec![50, 51]);

    serve.abort();
}

#[tokio::test]
async fn row8_foreign_group_single_host_returns_400() {
    let dir = TempDir::new().unwrap();
    let sm = SequenceSm::new();
    let store = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let host = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));

    let (l, url) = bind().await;
    let serve = tokio::spawn({
        let r = host.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let resp = client
        .post(&format!("{}/raft/request-vote", url))
        .json(&serde_json::json!({
            "group_id": "beta",
            "from": 1,
            "req": VoteReq {
                term: 2,
                candidate: 1,
                last_log_index: 0,
                last_log_term: 0,
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(resp.status().as_u16(), 400);

    serve.abort();
}

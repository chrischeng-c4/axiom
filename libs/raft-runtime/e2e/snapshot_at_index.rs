use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use raft_runtime::{
    FsyncPolicy, HostConfig, Index, Membership, RaftHost, RaftStateMachine, RaftStore,
};

struct IndexedSnapshotStateMachine {
    applied: AtomicU64,
    requested: Mutex<Vec<Index>>,
}

impl IndexedSnapshotStateMachine {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            applied: AtomicU64::new(0),
            requested: Mutex::new(Vec::new()),
        })
    }
}

impl RaftStateMachine for IndexedSnapshotStateMachine {
    fn apply(&self, index: Index, _command: &[u8]) -> anyhow::Result<()> {
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        writer.write_all(&self.applied.load(Ordering::Acquire).to_le_bytes())?;
        Ok(())
    }

    fn snapshot_at(&self, index: Index, writer: &mut dyn Write) -> anyhow::Result<()> {
        self.requested.lock().unwrap().push(index);
        writer.write_all(&index.to_le_bytes())?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        self.applied
            .store(u64::from_le_bytes(bytes), Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

#[tokio::test]
async fn external_compaction_uses_the_requested_applied_prefix() {
    let data = tempfile::tempdir().unwrap();
    let state_machine = IndexedSnapshotStateMachine::new();
    let host = RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(data.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap(),
        state_machine.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    );

    for value in 1_u8..=10 {
        host.propose(vec![value]).await.unwrap();
    }
    assert_eq!(host.snapshot_and_compact_through(6).await.unwrap(), 6);
    assert_eq!(*state_machine.requested.lock().unwrap(), vec![6]);

    host.shutdown().await.unwrap();
    drop(host);
    let persisted = RaftStore::open(data.path().to_str().unwrap(), 0, FsyncPolicy::Always)
        .unwrap()
        .load()
        .unwrap()
        .unwrap();
    assert_eq!(persisted.snapshot_index, 6);
    assert_eq!(persisted.log.len(), 4);
    assert_eq!(persisted.snapshot, 6_u64.to_le_bytes());
}

#[tokio::test]
async fn external_compaction_rejects_an_unapplied_target() {
    let data = tempfile::tempdir().unwrap();
    let state_machine = IndexedSnapshotStateMachine::new();
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
    host.propose(vec![1]).await.unwrap();
    let error = host
        .snapshot_and_compact_through(2)
        .await
        .expect_err("unapplied snapshot target must fail closed");
    assert!(error.to_string().contains("applied"));
    host.shutdown().await.unwrap();
}

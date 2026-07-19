// HANDWRITE-BEGIN gap="sift-ha-backup-contract-tests" tracker="1605" reason="Verify durable recovery, Raft single-node state-machine ordering, snapshot restore, and shared backup output."
use std::{collections::HashMap, sync::Arc};

use raft_host::{FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStore};
use sift::{
    backup::backup_journal, durability::SiftStateMachine, DurableJournal, EventEnvelope,
    EventQuery, SignalKind,
};

fn event(id: &str) -> EventEnvelope {
    let mut event = EventEnvelope::new(id, SignalKind::Log, serde_json::json!({"message":"ha"}));
    event
        .resource
        .insert("service.name".to_string(), "sift-ha-test".to_string());
    event
}

#[tokio::test]
async fn raft_state_machine_recovers_and_backup_restores_a_durable_snapshot() {
    let data_dir = tempfile::tempdir().expect("journal directory");
    let journal = Arc::new(DurableJournal::open(data_dir.path()).expect("open framed journal"));
    let state_machine = Arc::new(SiftStateMachine::new(journal.clone()));
    let host = RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        RaftStore::open(data_dir.path().to_str().unwrap(), 0, FsyncPolicy::Always)
            .expect("open raft store"),
        state_machine.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    );

    let accepted = event("raft-event");
    let index = host
        .propose(serde_json::to_vec(&accepted).expect("encode replicated event"))
        .await
        .expect("single-node raft proposal");
    assert_eq!(index, 1);
    assert_eq!(state_machine.applied_index(), 1);
    assert_eq!(journal.query(EventQuery::default()).unwrap().len(), 1);

    let backup_dir = tempfile::tempdir().expect("backup destination");
    let backup = backup_journal(
        &journal,
        &format!("file://{}", backup_dir.path().display()),
        None,
    )
    .expect("write shared backup");
    let backup_path = backup_dir.path().join(backup.object.key);
    assert!(backup_path.is_file());

    let restored_dir = tempfile::tempdir().expect("restore directory");
    let restored = DurableJournal::open(restored_dir.path()).expect("open restore journal");
    restored
        .restore_snapshot_bytes(&std::fs::read(backup_path).expect("read backup bytes"))
        .expect("restore snapshot");
    assert_eq!(
        restored.query(EventQuery::default()).unwrap()[0]
            .event
            .event_id,
        "raft-event"
    );
}

// HANDWRITE-END

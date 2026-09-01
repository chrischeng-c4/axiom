// HANDWRITE-BEGIN gap="sift-ha-backup-contract-tests" tracker="1605" reason="Verify durable recovery, Raft single-node state-machine ordering, snapshot restore, and shared backup output."
use std::{collections::HashMap, sync::Arc};

use raft_runtime::{FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStore};
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
        .propose(
            serde_json::to_vec(&serde_json::json!({
                "kind": "append_events",
                "events": [accepted]
            }))
            .expect("encode replicated event batch"),
        )
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
    let snapshot = std::fs::read(&backup_path).expect("read backup bytes");
    assert!(snapshot.starts_with(b"SIFTSNP2"));
    assert_ne!(snapshot.first(), Some(&b'{'));

    let restored_dir = tempfile::tempdir().expect("restore directory");
    let restored = DurableJournal::open(restored_dir.path()).expect("open restore journal");
    restored
        .restore_snapshot_bytes(&snapshot)
        .expect("restore snapshot");
    assert_eq!(
        restored.query(EventQuery::default()).unwrap()[0]
            .event
            .event_id,
        "raft-event"
    );

    let mut raft_snapshot = Vec::new();
    state_machine
        .snapshot(&mut raft_snapshot)
        .expect("stream Raft snapshot");
    let raft_restore_dir = tempfile::tempdir().expect("Raft restore directory");
    let raft_restore_journal =
        Arc::new(DurableJournal::open(raft_restore_dir.path()).expect("open Raft restore journal"));
    let raft_restore = SiftStateMachine::new(raft_restore_journal.clone());
    raft_restore
        .restore(&mut raft_snapshot.as_slice())
        .expect("restore streamed Raft snapshot");
    assert_eq!(raft_restore.applied_index(), 1);
    assert_eq!(raft_restore_journal.total_event_count(), 1);
}

#[test]
fn snapshot_restore_fails_closed_before_writing_corrupt_input() {
    let source_dir = tempfile::tempdir().expect("source directory");
    let source = DurableJournal::open(source_dir.path()).expect("open source journal");
    for index in 0..3 {
        source
            .append(event(&format!("snapshot-{index}")))
            .expect("append source event");
    }
    let snapshot = source.snapshot_bytes().expect("stream snapshot");

    let mut corrupt = snapshot.clone();
    let last = corrupt.last_mut().expect("snapshot payload byte");
    *last ^= 0xff;
    let mut trailing = snapshot.clone();
    trailing.push(0xff);
    let truncated = &snapshot[..snapshot.len() - 1];

    for (name, bytes) in [
        ("corrupt", corrupt.as_slice()),
        ("trailing", trailing.as_slice()),
        ("truncated", truncated),
    ] {
        let restore_dir = tempfile::tempdir().expect("restore directory");
        let restored = DurableJournal::open(restore_dir.path()).expect("open restore journal");
        let error = restored
            .restore_snapshot_bytes(bytes)
            .expect_err("invalid snapshot must fail closed");
        assert!(
            error.to_string().contains("snapshot"),
            "{name} error did not identify the snapshot: {error:#}"
        );
        assert_eq!(
            restored.total_event_count(),
            0,
            "{name} input mutated the empty journal before validation completed"
        );
    }
}

// HANDWRITE-END

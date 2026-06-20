// SPEC-MANAGED: projects/relay/tech-design/logic/raft-hard-state-persistence-fsyncpolicy-crash-safe-single-voter.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:8ae02b04" tracker="pending-tracker" reason="Tests: PersistedState save/load round-trip; a sole-voter node persisted then restored via from_persisted keeps its committed log; votedFor is remembered across restore (no double-vote); loading an empty dir returns None."
//! Raft hard-state persistence (#137): save/load round-trips, a restored node
//! keeps its committed log and remembers its vote, and an empty dir is a clean
//! fresh start.

use relay::raft::{auto_membership, PersistedState, RaftEntry, RaftMsg, RaftNode, VoteReq};
use relay::{FsyncPolicy, RaftStore};

fn entry(term: u64, index: u64, c: u8) -> RaftEntry {
    RaftEntry {
        term,
        index,
        command: vec![c],
    }
}

#[test]
fn persisted_state_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let state = PersistedState {
        term: 5,
        voted_for: Some(2),
        log: vec![entry(1, 1, 10), entry(3, 2, 20), entry(5, 3, 30)],
    };
    RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Always)
        .unwrap()
        .save(&state)
        .unwrap();
    // A fresh store over the same dir loads the identical state.
    let loaded = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Always)
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(loaded, Some(state));
}

#[test]
fn empty_dir_loads_none() {
    let dir = tempfile::tempdir().unwrap();
    let got = RaftStore::open(dir.path().to_str().unwrap(), 7, FsyncPolicy::Always)
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(got, None);
}

#[test]
fn sole_voter_restore_keeps_log_and_resumes() {
    let dir = tempfile::tempdir().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap();
    let m = auto_membership(1); // single voter

    let saved = {
        let mut node = RaftNode::new(0, &m);
        for _ in 0..12 {
            node.tick();
        }
        assert!(node.is_leader(), "sole voter elected itself");
        for c in 0..3u8 {
            node.propose(vec![c]).unwrap();
        }
        assert_eq!(node.commit_index(), 3, "sole voter commits immediately");
        let state = node.persisted();
        store.save(&state).unwrap();
        state
    };

    // "Restart": load + from_persisted.
    let loaded = store.load().unwrap().unwrap();
    assert_eq!(loaded, saved);
    let mut restored = RaftNode::from_persisted(0, &m, loaded);
    assert_eq!(restored.last_index(), 3, "log intact after restart");

    // It resumes: re-elects and can commit fresh entries (which also commits the
    // recovered prefix).
    for _ in 0..12 {
        restored.tick();
    }
    assert!(restored.is_leader());
    restored.propose(vec![99]).unwrap();
    assert_eq!(restored.last_index(), 4);
    assert_eq!(restored.commit_index(), 4, "recovered + new entries commit");
}

#[test]
fn votedfor_remembered_prevents_double_vote() {
    let dir = tempfile::tempdir().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 1, FsyncPolicy::Always).unwrap();
    let m = auto_membership(3);

    // Node 1 grants its vote to candidate 0 in term 1.
    let mut node = RaftNode::new(1, &m);
    node.handle(
        0,
        RaftMsg::Vote(VoteReq {
            term: 1,
            candidate: 0,
            last_log_index: 0,
            last_log_term: 0,
        }),
    );
    let _ = node.take_outgoing();
    assert_eq!(node.persisted().voted_for, Some(0));
    store.save(&node.persisted()).unwrap();

    // Restart, then a different candidate asks for a vote in the SAME term.
    let mut restored = RaftNode::from_persisted(1, &m, store.load().unwrap().unwrap());
    restored.handle(
        2,
        RaftMsg::Vote(VoteReq {
            term: 1,
            candidate: 2,
            last_log_index: 0,
            last_log_term: 0,
        }),
    );
    let granted = restored
        .take_outgoing()
        .into_iter()
        .any(|o| matches!(o.msg, RaftMsg::VoteResp(r) if r.granted));
    assert!(
        !granted,
        "remembered votedFor blocks a second vote in the term"
    );
}
// HANDWRITE-END

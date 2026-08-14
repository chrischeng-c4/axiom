//! Membership as a committed, persisted, generation-bound value (#3568).
//!
//! The group's configuration is a value the log commits, the durable state
//! carries, and a restart recovers — so a node comes back as the member its
//! group last agreed it was, not as the member its caller happened to name.

use raft_core::{
    auto_membership, ConfState, EntryKind, Membership, PersistedState, RaftNode,
};

fn sole_voter() -> Membership {
    Membership {
        voters: vec![0],
        learners: vec![],
    }
}

/// Drive a sole voter to leadership. Its election timeout is `ELECTION_MIN + id`
/// ticks and it is its own majority, so it elects itself without a bus.
fn elected_sole_voter() -> RaftNode {
    let mut node = RaftNode::new(0, &sole_voter());
    for _ in 0..60 {
        node.tick();
    }
    assert!(
        node.is_leader(),
        "a sole voter must reach leadership on its own ticks"
    );
    node
}

/// The defect this whole split rests on: `from_persisted` took its membership
/// from the caller's argument and never from the recovered state.
#[test]
fn recovered_configuration_beats_the_callers_bootstrap_argument() {
    let agreed = Membership {
        voters: vec![1, 2, 3],
        learners: vec![0],
    };
    let bootstrap = sole_voter();

    let mut node = RaftNode::new(0, &bootstrap);
    assert!(
        node.is_voter(),
        "the bootstrap membership makes node 0 a voter, so a recovered learner cannot be an artefact of the starting state"
    );
    assert!(node.adopt_conf(ConfState {
        membership: agreed.clone(),
        generation: 4,
    }));
    let state = node.persisted();
    drop(node);

    let recovered = RaftNode::from_persisted(0, &bootstrap, state);

    assert_eq!(
        recovered.conf_state().membership,
        agreed,
        "recovered node reports {:?}, but the store recorded {:?} and the caller passed {:?}",
        recovered.conf_state().membership,
        agreed,
        bootstrap
    );
    assert_eq!(
        recovered.conf_state().generation,
        4,
        "the recovered generation is {} but the store recorded 4",
        recovered.conf_state().generation
    );
    assert!(
        !recovered.is_voter(),
        "node 0 is a learner in the recovered configuration {:?}, yet it came back a voter — the caller's bootstrap {:?} won",
        agreed,
        bootstrap
    );
}

/// The other half of the same rule: with nothing recorded, the caller's value
/// is the bootstrap, and it starts low enough that any agreed configuration
/// supersedes it.
#[test]
fn the_bootstrap_argument_is_used_only_when_the_store_carries_no_configuration() {
    let bootstrap = auto_membership(3);
    let state = PersistedState {
        term: 2,
        voted_for: Some(1),
        log: vec![],
        commit_index: 0,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: None,
    };

    let node = RaftNode::from_persisted(0, &bootstrap, state);

    assert_eq!(node.conf_state().membership, bootstrap);
    assert_eq!(
        node.conf_state().generation,
        0,
        "a bootstrap configuration must start at the lowest generation, or the first agreed one cannot supersede it"
    );
    assert_eq!(node.current_term(), 2, "the rest of the hard state still recovers");
}

/// A generation that is stored but never compared is a field, not a guard.
#[test]
fn a_configuration_that_does_not_supersede_is_refused() {
    let mut node = RaftNode::new(
        0,
        &Membership {
            voters: vec![0, 1, 2],
            learners: vec![],
        },
    );
    let in_force = ConfState {
        membership: Membership {
            voters: vec![0, 1, 2],
            learners: vec![3],
        },
        generation: 7,
    };
    assert!(node.adopt_conf(in_force.clone()));

    let same_generation = ConfState {
        membership: sole_voter(),
        generation: 7,
    };
    assert!(
        !node.adopt_conf(same_generation),
        "generation 7 does not supersede generation 7"
    );
    assert_eq!(
        node.conf_state(),
        &in_force,
        "a refused configuration must leave the one in force untouched"
    );

    let older = ConfState {
        membership: sole_voter(),
        generation: 6,
    };
    assert!(
        !node.adopt_conf(older),
        "generation 6 does not supersede generation 7"
    );
    assert_eq!(node.conf_state(), &in_force);
}

/// Adoption is not bookkeeping: the adopted voter set is the one that decides
/// a commit.
#[test]
fn a_superseding_configuration_moves_the_voter_set_that_decides_a_commit() {
    let mut node = elected_sole_voter();
    assert_eq!(node.propose(b"a".to_vec()), Some(1));
    assert_eq!(
        node.commit_index(),
        1,
        "a sole voter is its own majority and commits its own proposal"
    );

    let widened = ConfState {
        membership: Membership {
            voters: vec![0, 1, 2],
            learners: vec![],
        },
        generation: 1,
    };
    assert!(node.adopt_conf(widened.clone()));
    assert_eq!(node.conf_state(), &widened);

    assert_eq!(node.propose(b"b".to_vec()), Some(2));
    assert_eq!(
        node.commit_index(),
        1,
        "after adopting {:?} a lone leader is no longer a majority, yet index 2 committed",
        widened.membership
    );
}

/// A configuration entry is not a client command: the consumer never sees it,
/// and committing it is what puts it in force. Both halves are asserted
/// together — a node that drops the entry and ignores it satisfies either one
/// alone.
#[test]
fn a_committed_configuration_entry_is_adopted_and_withheld_from_the_consumer() {
    let mut node = elected_sole_voter();

    assert_eq!(node.propose(b"before".to_vec()), Some(1));
    let next = ConfState {
        membership: Membership {
            voters: vec![0],
            learners: vec![9],
        },
        generation: 3,
    };
    assert_eq!(node.propose_config(next.clone()), Some(2));
    assert_eq!(node.propose(b"after".to_vec()), Some(3));
    assert_eq!(node.commit_index(), 3);

    let applied = node.take_committed();

    assert_eq!(
        applied
            .iter()
            .map(|e| e.command.clone())
            .collect::<Vec<Vec<u8>>>(),
        vec![b"before".to_vec(), b"after".to_vec()],
        "the configuration entry at index 2 reached the state machine as a client command"
    );
    assert!(
        applied.iter().all(|e| e.kind == EntryKind::Command),
        "take_committed surfaced an entry that is not a client command"
    );
    assert_eq!(
        node.conf_state(),
        &next,
        "committing a configuration entry must put it in force"
    );
    assert!(
        node.take_committed().is_empty(),
        "the withheld entry must still have advanced last_applied"
    );
}

/// The durable state is the carrier, and a record written before this change
/// still reads through the serde path the store falls back to.
#[test]
fn the_durable_state_carries_the_configuration_and_the_entry_kinds() {
    let mut node = elected_sole_voter();
    node.propose(b"cmd".to_vec()).unwrap();
    let next = ConfState {
        membership: Membership {
            voters: vec![0],
            learners: vec![4],
        },
        generation: 2,
    };
    node.propose_config(next.clone()).unwrap();
    node.take_committed();

    let state = node.persisted();
    assert_eq!(state.conf, Some(next));
    assert_eq!(state.log[0].kind, EntryKind::Command);
    assert_eq!(
        state.log[1].kind,
        EntryKind::Config,
        "the log must keep the entry's kind, or a restart replays a configuration change as a command"
    );

    let round_tripped: PersistedState =
        serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert_eq!(round_tripped, state);

    let written_before_this_change = serde_json::json!({
        "term": 3,
        "voted_for": 2,
        "log": [{ "term": 3, "index": 1, "command": [10, 20, 30] }],
        "commit_index": 1,
        "snapshot_index": 0,
        "snapshot_term": 0,
        "snapshot": []
    });
    let legacy: PersistedState = serde_json::from_value(written_before_this_change).unwrap();
    assert_eq!(
        legacy.conf, None,
        "a record with no configuration must load with none rather than be rejected"
    );
    assert_eq!(legacy.log[0].kind, EntryKind::Command);
}

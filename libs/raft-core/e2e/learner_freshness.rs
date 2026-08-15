//! How far behind a learner is *right now*, as opposed to whether it was ever
//! admitted (#3583).
//!
//! `learner_read_eligible` answers the admission question and #3569's six rows
//! pin that meaning. It is not the question a bounded read needs. For lumen's
//! entire learner population it is also constant: `cluster.rs:280` puts every
//! learner in the membership the node is *constructed* with, `RaftNode::new`
//! seeds each of those targets with `0`, and `adopt_conf` records targets with
//! `.or_insert(...)`, so the seeded `0` is never replaced. Eligibility reduces to
//! `matched >= 0`, which is true of a learner that has replicated nothing.
//!
//! These rows are about the live figure instead: the number of committed entries
//! the learner has not got, measured against the leader's commit index, asked of
//! the leader, at the moment it is asked. Every row builds its group the way
//! `cluster.rs:280` does — learners present from construction — except the last,
//! which exists to show the admission answer is untouched.

use std::collections::{HashMap, HashSet};

use raft_core::{Membership, NodeId, RaftNode, Role};

/// Voters 0,1,2 with learner 3 present from construction: the shape
/// `libs/raft-runtime/src/cluster.rs:280` builds, and the population in which
/// the admission predicate is constant.
fn three_voters_and_a_learner() -> Membership {
    Membership {
        voters: vec![0, 1, 2],
        learners: vec![3],
    }
}

/// Voters 0,1,2 and no learners: the group a learner is added *to* at runtime.
fn three_voters() -> Membership {
    Membership {
        voters: vec![0, 1, 2],
        learners: vec![],
    }
}

struct Bus {
    nodes: HashMap<NodeId, RaftNode>,
    applied: HashMap<NodeId, Vec<Vec<u8>>>,
    /// Nodes no message is delivered to, in either direction.
    dropped: HashSet<NodeId>,
}

impl Bus {
    fn new(ids: &[NodeId], start: &Membership) -> Self {
        Bus {
            nodes: ids
                .iter()
                .map(|id| (*id, RaftNode::new(*id, start)))
                .collect(),
            applied: ids.iter().map(|id| (*id, Vec::new())).collect(),
            dropped: HashSet::new(),
        }
    }

    fn round(&mut self) -> usize {
        let mut msgs = Vec::new();
        for (id, node) in self.nodes.iter_mut() {
            for out in node.take_outgoing() {
                msgs.push((*id, out));
            }
        }
        let mut delivered = 0;
        for (from, out) in msgs {
            if self.dropped.contains(&out.to) || self.dropped.contains(&from) {
                continue;
            }
            if let Some(target) = self.nodes.get_mut(&out.to) {
                target.handle(from, out.msg);
                delivered += 1;
            }
        }
        for (id, node) in self.nodes.iter_mut() {
            for entry in node.take_committed() {
                self.applied.get_mut(id).unwrap().push(entry.command);
            }
        }
        delivered
    }

    fn pump(&mut self) {
        for _ in 0..200 {
            if self.round() == 0 {
                return;
            }
        }
        panic!("the bus never reached quiescence");
    }

    /// Tick the leader and deliver, repeatedly. A peer that has been unreachable
    /// is brought forward by heartbeats, not by draining an outbox that is
    /// already empty.
    fn settle(&mut self) {
        for _ in 0..40 {
            if let Some(leader) = self.leader() {
                self.nodes.get_mut(&leader).unwrap().tick();
            }
            self.pump();
        }
    }

    fn tick_all(&mut self) {
        for node in self.nodes.values_mut() {
            node.tick();
        }
    }

    fn leader(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|(_, n)| n.is_leader())
            .map(|(id, _)| *id)
    }

    fn run_until_leader(&mut self) -> NodeId {
        for _ in 0..200 {
            self.tick_all();
            self.pump();
            if let Some(id) = self.leader() {
                return id;
            }
        }
        panic!("no leader was elected");
    }

    fn commit(&mut self, leader: NodeId, command: Vec<u8>) {
        self.nodes
            .get_mut(&leader)
            .unwrap()
            .propose(command)
            .expect("the leader appends a proposal");
        self.pump();
    }
}

/// The row that fails today. A learner that has been present since construction
/// and has replicated nothing is reported by `learner_read_eligible` as fit to
/// serve reads, because its seeded target is `0`. The live figure must say how
/// far behind it actually is.
#[test]
fn a_learner_present_from_construction_reports_the_entries_it_is_missing() {
    let mut bus = Bus::new(&[0, 1, 2, 3], &three_voters_and_a_learner());
    let leader = bus.run_until_leader();

    // Partition the learner, then commit. Two voters are a majority on their
    // own, so the group makes progress the learner does not see.
    bus.dropped.insert(3);
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }

    let node = &bus.nodes[&leader];
    let commit_index = node.commit_index();
    let matched = node
        .learner_matched(3)
        .expect("a leader knows how far a learner in its own configuration has replicated");

    // Premises, asserted rather than assumed: if either stops holding, this row
    // says so instead of quietly measuring nothing.
    assert_eq!(
        commit_index, 5,
        "this row commits five commands and nothing else, so the leader's commit index must be 5, not {commit_index}",
    );
    assert_eq!(
        matched, 0,
        "the learner was unreachable for every one of those commits, so the leader must still record it at 0, not {matched}",
    );
    assert!(
        bus.applied[&3].is_empty() && bus.applied[&leader].len() == 5,
        "the learner must be missing all five commands for this row to have a gap to report: leader applied {}, learner applied {}",
        bus.applied[&leader].len(),
        bus.applied[&3].len(),
    );
    assert_eq!(
        node.learner_read_eligible(3),
        Some(true),
        "this row exists because the admission predicate is constant for this population; if it now reports {:?}, the defect moved and this row is measuring the wrong thing",
        node.learner_read_eligible(3),
    );

    assert_eq!(
        node.learner_replication_gap(3),
        Some(5),
        "learner 3 has replicated to index {matched} and the leader's commit index is {commit_index}, so it is missing 5 committed entries, but the leader reports a gap of {:?}",
        node.learner_replication_gap(3),
    );
}

/// The figure is measured against what the group has agreed to keep, not against
/// what the leader happens to have written down. An entry appended but not
/// committed may still be lost, so counting it makes the learner look further
/// behind than it is against a bar that can move backwards.
#[test]
fn the_gap_is_measured_against_the_commit_index_not_the_last_appended_index() {
    let mut bus = Bus::new(&[0, 1, 2, 3], &three_voters_and_a_learner());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    bus.settle();

    assert_eq!(
        bus.nodes[&leader].learner_matched(3),
        Some(bus.nodes[&leader].commit_index()),
        "the learner is left to replicate to quiescence first, so that the only distance left is the uncommitted entry this row appends",
    );

    // Everyone but the leader goes away, so the next proposal is appended and can
    // never commit. The learner is unreachable too: otherwise it would replicate
    // the uncommitted entry and the two candidate reference points would agree.
    bus.dropped.insert(1);
    bus.dropped.insert(2);
    bus.dropped.insert(3);
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .propose(vec![99])
        .expect("the leader appends the proposal even with no one to acknowledge it");
    bus.pump();

    let node = &bus.nodes[&leader];
    let commit_index = node.commit_index();
    let last_index = node.last_index();
    let matched = node.learner_matched(3).expect("still a learner");

    assert!(
        node.is_leader(),
        "the node stepped down while the row was setting up, so nothing below is being asked of a leader",
    );
    assert_eq!(
        last_index,
        commit_index + 1,
        "this row needs exactly one appended-but-uncommitted entry: commit index {commit_index}, last index {last_index}",
    );
    assert_eq!(
        matched, commit_index,
        "the learner must be exactly at the commit index, so that a gap of 0 and a gap of 1 tell the two reference points apart: matched {matched}, commit index {commit_index}",
    );

    assert_eq!(
        node.learner_replication_gap(3),
        Some(0),
        "learner 3 has every committed entry (matched {matched}, commit index {commit_index}); the leader also holds an uncommitted entry at {last_index}, and reporting a gap of {:?} counts an entry the group has not agreed to keep",
        node.learner_replication_gap(3),
    );
}

/// Only a leader answers, and only about a learner. A follower's view of a
/// peer's progress is stale by construction, and a number nobody can date is
/// worse than no number: the caller cannot tell which one it got.
#[test]
fn only_a_leader_reports_a_gap_and_only_about_a_learner() {
    let mut bus = Bus::new(&[0, 1, 2, 3], &three_voters_and_a_learner());
    let leader = bus.run_until_leader();
    for i in 0..3u8 {
        bus.commit(leader, vec![i]);
    }
    let follower = (0..3u64).find(|id| *id != leader).unwrap();

    assert!(
        bus.nodes[&leader].learner_replication_gap(3).is_some(),
        "the leader replicates to the learner and must be able to report the distance; without this the row below is satisfied by a function that never answers at all",
    );
    assert_eq!(
        bus.nodes[&follower].learner_replication_gap(3),
        None,
        "node {follower} is not the leader and does not replicate to the learner, so any figure it produced would come from its own stale view",
    );
    assert_eq!(
        bus.nodes[&leader].learner_replication_gap(follower),
        None,
        "node {follower} is a voter, not a learner, and this figure is about learners",
    );
    assert_eq!(
        bus.nodes[&leader].learner_replication_gap(9),
        None,
        "node 9 is in nobody's configuration, so there is no progress to report about it",
    );
}

/// The live figure is a second answer, not a redefinition of the first. #3569's
/// admission question keeps its meaning for the population it was written for —
/// a learner added to a running group — and this row fails if the new figure was
/// bought by rerouting `learner_read_eligible` through it.
#[test]
fn the_admission_predicate_is_unchanged_for_a_learner_admitted_at_runtime() {
    let mut bus = Bus::new(&[0, 1, 2, 3], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }

    bus.dropped.insert(3);
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .add_learner(3)
        .expect("a leader admits a learner by appending a configuration entry");
    bus.pump();

    let node = &bus.nodes[&leader];
    let target = node
        .learner_read_target(3)
        .expect("an admitted learner has a recorded read target");
    assert_eq!(
        node.learner_read_eligible(3),
        Some(false),
        "learner 3 was admitted at target {target} and has replicated {:?}; the admission predicate must still withhold it",
        node.learner_matched(3),
    );
    assert_eq!(
        node.learner_replication_gap(3),
        Some(node.commit_index()),
        "a learner that has replicated nothing is missing every committed entry: commit index {}, gap {:?}",
        node.commit_index(),
        node.learner_replication_gap(3),
    );

    bus.dropped.remove(&3);
    bus.settle();

    let node = &bus.nodes[&leader];
    assert_eq!(
        node.learner_read_eligible(3),
        Some(true),
        "the learner replicated to quiescence (matched {:?} against target {:?}) and the admission predicate must admit it exactly as it did before this item",
        node.learner_matched(3),
        node.learner_read_target(3),
    );
    assert_eq!(
        node.learner_replication_gap(3),
        Some(0),
        "a caught-up learner is missing nothing, but the leader reports {:?}",
        node.learner_replication_gap(3),
    );
    assert_eq!(
        bus.nodes[&3].role(),
        Role::Follower,
        "the learner must not have campaigned while this row was running",
    );
}

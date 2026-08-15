//! Adding a learner to a running group, and withholding its reads until it has
//! caught up (#3569).
//!
//! Eligibility is asked of the **leader**, about a named learner, and that is a
//! measured decision rather than a stylistic one. Two properties of this engine
//! close the window on the learner's own side:
//!
//! * `send_append_to` ships `log.iter().filter(|e| e.index >= next)` — the whole
//!   suffix in one message, with no batch cap;
//! * `take_committed` runs `while last_applied < commit_index`, and adopts the
//!   configuration entry *inside* that loop.
//!
//! So a learner reaches `last_applied == commit_index` in the same call in which
//! it learns it is a learner. Every target it could derive locally — the
//! configuration entry's index, its commit index at adoption, the `leader_commit`
//! it was last told — is already satisfied at the instant it exists. A row
//! asserting "ineligible, then eligible" against the learner's own view is
//! therefore green before any implementation, which is the false green this item
//! exists to remove.
//!
//! The lag is real on the leader, which holds `match_index[learner]` far below
//! its own commit index for as long as the learner has not replicated. That is
//! the fact these rows measure, and it is the fact `apps/lumen/src/api.rs:1041`
//! needs in order to stop admitting a seconds-old learner for a bounded read.

use std::collections::{HashMap, HashSet};

use raft_core::{Membership, NodeId, RaftNode, Role};

/// Voters 0,1,2 with no learners: the group a learner is added *to*.
fn three_voters() -> Membership {
    Membership {
        voters: vec![0, 1, 2],
        learners: vec![],
    }
}

struct Bus {
    nodes: HashMap<NodeId, RaftNode>,
    /// Commands handed to each node's consumer, in order.
    applied: HashMap<NodeId, Vec<Vec<u8>>>,
    /// Snapshots each node's consumer was asked to load.
    installed: HashMap<NodeId, usize>,
    /// Nodes no message is delivered to. A learner added while it is in here
    /// gets the leader-side window this file measures, without depending on how
    /// many rounds the leader's next-index backoff happens to take.
    dropped: HashSet<NodeId>,
}

impl Bus {
    fn new(ids: &[NodeId], start: &Membership) -> Self {
        Bus {
            nodes: ids.iter().map(|id| (*id, RaftNode::new(*id, start))).collect(),
            applied: ids.iter().map(|id| (*id, Vec::new())).collect(),
            installed: ids.iter().map(|id| (*id, 0usize)).collect(),
            dropped: HashSet::new(),
        }
    }

    /// One delivery pass, then one drain pass. Returns the messages delivered.
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
            if node.take_installed_snapshot().is_some() {
                *self.installed.get_mut(id).unwrap() += 1;
            }
            for entry in node.take_committed() {
                self.applied.get_mut(id).unwrap().push(entry.command);
            }
        }
        delivered
    }

    /// Deliver until nothing more moves.
    fn pump(&mut self) {
        for _ in 0..200 {
            if self.round() == 0 {
                return;
            }
        }
        panic!("the bus never reached quiescence");
    }

    /// Tick the leader and deliver, repeatedly. A peer that joins a running
    /// group is first contacted by a heartbeat: `adopt_conf` only seeds its
    /// replication bookkeeping, and `propose_config`'s broadcast went out before
    /// it was a peer. A bus that merely drains outboxes therefore never reaches
    /// a newcomer at all, which is a property of the driver and not of the
    /// engine — every real driver ticks.
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

/// A running group with five committed commands and a leader. Node 3 exists as
/// a process but is in nobody's configuration, which is the state a node is in
/// immediately before it is added.
fn running_group() -> (Bus, NodeId) {
    let mut bus = Bus::new(&[0, 1, 2, 3], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    (bus, leader)
}

/// R3. The whole item: a learner added to a running group must not be reported
/// as fit to serve a read until it has actually replicated the data.
#[test]
fn a_learner_added_at_runtime_is_withheld_from_reads_until_it_reaches_the_recorded_target() {
    let (mut bus, leader) = running_group();

    // The learner is unreachable at the moment it is admitted. The voters are a
    // majority on their own, so the configuration entry still commits and the
    // leader still adopts it — this is the window, and it is the leader that can
    // see it.
    bus.dropped.insert(3);
    let at = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .add_learner(3)
        .expect("a leader admits a learner by appending a configuration entry");
    bus.pump();

    let node = &bus.nodes[&leader];
    assert!(
        node.conf_state().membership.learners.contains(&3),
        "the configuration entry at index {at} committed, but the leader's configuration is {:?}",
        node.conf_state().membership
    );
    let target = node
        .learner_read_target(3)
        .expect("an admitted learner has a recorded read target");
    let matched = node
        .learner_matched(3)
        .expect("a leader knows how far an admitted learner has replicated");
    assert!(
        matched < target,
        "this row cannot measure anything unless the learner starts behind: matched {matched}, target {target}"
    );
    assert_eq!(
        node.learner_read_eligible(3),
        Some(false),
        "learner 3 has replicated to {matched} and its recorded target is {target}, yet the leader reports it fit to serve a read",
    );

    // Now let it catch up.
    bus.dropped.remove(&3);
    bus.settle();

    let node = &bus.nodes[&leader];
    let matched = node.learner_matched(3).expect("still an admitted learner");
    let target = node.learner_read_target(3).expect("still an admitted learner");
    assert!(
        matched >= target,
        "the learner was left to replicate to quiescence but only reached {matched} against a target of {target}",
    );
    assert_eq!(
        node.learner_read_eligible(3),
        Some(true),
        "learner 3 has replicated to {matched}, at or past its recorded target of {target}, yet the leader still withholds it",
    );
    assert_eq!(
        bus.applied[&3],
        bus.applied[&leader],
        "a caught-up learner must hold the same commands as the leader",
    );
}

/// The frozen decision that separates a reachable target from an unreachable
/// one. On a busy group the leader's commit index keeps moving, so a target read
/// from it at question time is never reached and the learner is
/// indistinguishable from one that was never added.
#[test]
fn the_read_target_is_fixed_at_admission_and_does_not_follow_the_commit_index() {
    let (mut bus, leader) = running_group();

    bus.nodes.get_mut(&leader).unwrap().add_learner(3).unwrap();
    bus.settle();
    let target = bus.nodes[&leader].learner_read_target(3).unwrap();
    assert_eq!(
        bus.nodes[&leader].learner_read_eligible(3),
        Some(true),
        "the learner replicated to quiescence and must be eligible before this row can test anything",
    );

    // The group keeps committing while the learner is unreachable. Its target
    // was fixed when it was admitted, so it stays eligible: it is caught up in
    // the sense the group agreed on, not in the sense of a receding horizon.
    bus.dropped.insert(3);
    for i in 0..4u8 {
        bus.commit(leader, vec![100 + i]);
    }

    let node = &bus.nodes[&leader];
    assert_eq!(
        node.learner_read_target(3),
        Some(target),
        "the recorded target moved from {target} to {:?} because the group committed more",
        node.learner_read_target(3),
    );
    assert_eq!(
        node.learner_read_eligible(3),
        Some(true),
        "the learner met its recorded target of {target} and then the group moved on without it; a target that follows the commit index is never reached on a busy group",
    );
}

/// R3's other half: the machinery that brings a far-behind member forward
/// already exists, and a learner admitted after the leader has compacted must
/// use it rather than stalling on entries that are gone.
#[test]
fn a_learner_admitted_after_compaction_catches_up_through_the_snapshot_path() {
    let (mut bus, leader) = running_group();

    // Drop the log out from under any newcomer.
    let applied_at_compaction = bus.applied[&leader].clone();
    let up_to = bus.nodes[&leader].last_index();
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .compact(up_to, b"state-machine".to_vec());
    assert_eq!(
        bus.nodes[&leader].snapshot_index(),
        up_to,
        "the leader must have compacted before a newcomer is admitted",
    );

    bus.dropped.insert(3);
    bus.nodes.get_mut(&leader).unwrap().add_learner(3).unwrap();
    bus.pump();
    assert_eq!(
        bus.nodes[&leader].learner_read_eligible(3),
        Some(false),
        "the learner has replicated nothing and must not be eligible before it catches up",
    );

    bus.dropped.remove(&3);
    bus.settle();

    assert_eq!(
        bus.installed[&3], 1,
        "the learner's starting point was compacted away, so it can only come forward through the snapshot path, but it was asked to load {} snapshots",
        bus.installed[&3],
    );
    let node = &bus.nodes[&leader];
    assert_eq!(
        node.learner_read_eligible(3),
        Some(true),
        "the learner caught up through the snapshot but the leader reports matched {:?} against target {:?}",
        node.learner_matched(3),
        node.learner_read_target(3),
    );
    assert!(
        !applied_at_compaction.is_empty(),
        "the commands that existed before compaction are what the snapshot stands for",
    );
}

/// Eligibility is about serving reads and nothing else. Promotion is item 03.
#[test]
fn a_caught_up_learner_still_never_votes_and_never_counts_toward_a_majority() {
    let (mut bus, leader) = running_group();

    bus.nodes.get_mut(&leader).unwrap().add_learner(3).unwrap();
    bus.settle();
    assert_eq!(
        bus.nodes[&leader].learner_read_eligible(3),
        Some(true),
        "this row is about a learner that has caught up",
    );
    assert!(
        !bus.nodes[&3].is_voter(),
        "node 3 was admitted as a learner but reports itself a voter",
    );
    assert!(
        !bus.nodes[&leader].conf_state().membership.voters.contains(&3),
        "the committed configuration made the learner a voter: {:?}",
        bus.nodes[&leader].conf_state().membership,
    );

    // The two other voters go away. Leader plus caught-up learner is two nodes
    // out of a four-node group but only one voter out of three, so nothing may
    // commit.
    let before = bus.applied[&leader].len();
    bus.dropped.insert(1);
    bus.dropped.insert(2);
    bus.commit(leader, vec![200]);
    assert_eq!(
        bus.applied[&leader].len(),
        before,
        "the leader committed with only a learner acknowledging, so the learner was counted toward the majority",
    );

    // And it never campaigns, however long it is left alone.
    for _ in 0..200 {
        bus.tick_all();
        bus.pump();
    }
    assert_eq!(
        bus.nodes[&3].role(),
        Role::Follower,
        "the learner reached {:?} after being left to time out",
        bus.nodes[&3].role(),
    );
}

/// The accessors answer about learners, and only where the answer is knowable.
/// Without this a `Some(true)` returned for every node would satisfy the rows
/// above.
#[test]
fn only_an_admitted_learner_has_a_read_target_and_only_a_leader_reports_eligibility() {
    let (mut bus, leader) = running_group();
    let follower = (0..3u64).find(|id| *id != leader).unwrap();

    assert_eq!(
        bus.nodes[&leader].learner_read_target(3),
        None,
        "node 3 is in nobody's configuration yet, so there is no target to report",
    );
    assert_eq!(
        bus.nodes[&leader].learner_read_eligible(follower),
        None,
        "node {follower} is a voter, and voter reads are not gated by this predicate",
    );

    bus.nodes.get_mut(&leader).unwrap().add_learner(3).unwrap();
    bus.settle();

    assert_eq!(
        bus.nodes[&follower].learner_matched(3),
        None,
        "node {follower} is not the leader and does not replicate to the learner, so it cannot report progress it does not have",
    );
    assert!(
        bus.nodes[&leader].learner_matched(3).is_some(),
        "the leader replicates to the learner and must be able to report its progress",
    );
}

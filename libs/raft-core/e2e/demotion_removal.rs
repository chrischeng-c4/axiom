//! Demoting a voter to a learner and removing a member from a group (#3572).
//!
//! # Why the rows do not share one group size
//!
//! A group of `n` voters commits with `maj(n) = n / 2 + 1` acknowledgements —
//! `libs/raft-core/src/lib.rs:1190` computes exactly that — so it survives
//! `n - maj(n)` failures:
//!
//! ```text
//! n        2  3  4  5  6  7
//! maj(n)   2  2  3  3  4  4
//! tolerate 0  1  1  2  2  3
//! ```
//!
//! Dropping one voter therefore holds the tolerance when `n` is even and loses
//! one when `n` is odd, and the arithmetic hides it: `3 -> 2` and `5 -> 4` both
//! leave the majority unchanged while halving the group's ability to survive a
//! failure. So the successful row must shrink an even set (`4 -> 3`, `1 -> 1`)
//! and the refusals must be measured at two odd sizes rather than one.
//!
//! Three is the size everyone reaches for, and a guard that simply refuses to
//! go below three voters passes it. That guard is not the one this item asks
//! for: it lets `5 -> 4` through, which is the same loss one size up and looks
//! perfectly healthy while it happens. `removing_a_voter_from_a_five_voter_
//! group_is_refused_although_four_voters_remain` is the row that separates a
//! tolerance argument from a minimum-size check, and it is the reason the two
//! refusal rows cannot be collapsed onto one size.
//!
//! # Why demotion carries the same guard
//!
//! Demotion moves a voter into the learner set, so it shrinks the voter set by
//! exactly one, exactly as removal does. A demotion guarded only by "the target
//! is a voter" walks a three-voter group down to two by the same path a refused
//! removal is stopped from taking, and the group is then one failure from
//! losing quorum with nothing having been refused.
//!
//! # Why a removed member is observed through the leader's outbox
//!
//! `libs/raft-core/src/lib.rs:580` rebuilds `peers` from voters ∪ learners ∪
//! outgoing on every adopted configuration, and `broadcast_append` at
//! `libs/raft-core/src/lib.rs:864` iterates it, so replication to a removed
//! member should stop on its own once the final configuration commits. A row
//! that reads the membership list back cannot tell that apart from an
//! implementation that edited the list and kept replicating, which is why
//! `leader_targets` watches what the leader actually addresses.
//!
//! # Why the demotion row makes an election happen
//!
//! `libs/raft-core/src/lib.rs:754` gates campaigning on `is_voter`, recomputed
//! from the adopted configuration at `libs/raft-core/src/lib.rs:582`. Asserting
//! only that the demoted node did not become a candidate is satisfied by a
//! wedged group, so the row also requires one of the remaining voters to win an
//! election in the same window. Both halves are needed: the first alone passes
//! when nothing runs, the second alone passes when everything runs.

use std::collections::{HashMap, HashSet};

use raft_core::{DemotionRefused, Membership, NodeId, RaftNode, RemovalRefused, Role};

/// `election_timeout` is `ELECTION_MIN + id` and `ELECTION_MIN` is 50, so a
/// silent window this long exceeds every node's timeout in these groups. The
/// constant is private to the crate, so the rows carry the bound rather than
/// importing it.
const SILENT_TICKS: usize = 120;

fn voters(ids: &[NodeId]) -> Membership {
    Membership {
        voters: ids.to_vec(),
        learners: vec![],
    }
}

struct Bus {
    nodes: HashMap<NodeId, RaftNode>,
    applied: HashMap<NodeId, Vec<Vec<u8>>>,
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
            let _ = node.take_installed_snapshot();
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

    /// Tick the leader and deliver, repeatedly. A peer that has just entered the
    /// configuration is first contacted by a heartbeat, so a bus that only
    /// drains outboxes never reaches it.
    fn settle(&mut self) {
        for _ in 0..40 {
            if let Some(leader) = self.leader() {
                self.nodes.get_mut(&leader).unwrap().tick();
            }
            self.pump();
        }
    }

    fn tick_all(&mut self) {
        for (id, node) in self.nodes.iter_mut() {
            if self.dropped.contains(id) {
                continue;
            }
            node.tick();
        }
    }

    /// The reachable leader. A dropped node keeps `Leader` until it hears a
    /// higher term, which it cannot while unreachable, so a scan that ignores
    /// `dropped` reports the deposed leader and masks the one the survivors
    /// just elected.
    fn leader(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|(id, n)| n.is_leader() && !self.dropped.contains(*id))
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
        self.settle();
    }

    /// Tick `leader` `n` times and record every node it addressed, delivering
    /// the messages afterwards so the observation does not stall the group.
    fn leader_targets(&mut self, leader: NodeId, n: usize) -> HashSet<NodeId> {
        let mut seen = HashSet::new();
        for _ in 0..n {
            self.nodes.get_mut(&leader).unwrap().tick();
            let outs = self.nodes.get_mut(&leader).unwrap().take_outgoing();
            for out in &outs {
                seen.insert(out.to);
            }
            for out in outs {
                if let Some(target) = self.nodes.get_mut(&out.to) {
                    target.handle(leader, out.msg);
                }
            }
            self.pump();
        }
        seen
    }

    /// A voter of the current configuration that is not `leader`.
    fn some_other_voter(&self, leader: NodeId) -> NodeId {
        *self.nodes[&leader]
            .conf_state()
            .membership
            .voters
            .iter()
            .find(|v| **v != leader)
            .expect("the group has a voter other than its leader")
    }
}

#[test]
fn removing_a_voter_from_a_four_voter_group_commits_and_stops_replication_to_it() {
    let ids = [0, 1, 2, 3];
    let mut bus = Bus::new(&ids, &voters(&ids));
    let leader = bus.run_until_leader();
    bus.commit(leader, b"before".to_vec());
    let victim = bus.some_other_voter(leader);

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .remove_member(victim)
        .expect("four voters tolerate one failure and three still do, so this removal is safe");
    bus.settle();

    let conf = bus.nodes[&leader].conf_state().clone();
    assert!(
        !bus.nodes[&leader].is_joint(),
        "the transition must reach its final configuration, not stop at the joint one"
    );
    assert!(
        !conf.membership.voters.contains(&victim),
        "the removed member must be gone from the voter set, found {:?}",
        conf.membership.voters
    );
    assert!(
        !conf.membership.learners.contains(&victim),
        "removal is not demotion: the member must not reappear as a learner"
    );
    assert_eq!(
        conf.membership.voters.len(),
        3,
        "exactly one voter leaves, so three remain"
    );

    bus.commit(leader, b"after".to_vec());
    for id in ids.iter().filter(|id| **id != victim) {
        assert!(
            bus.applied[id].contains(&b"after".to_vec()),
            "node {id} must still apply commands once the group is three voters"
        );
    }

    let addressed = bus.leader_targets(leader, 20);
    assert!(
        !addressed.contains(&victim),
        "the leader must stop replicating to a removed member; it addressed {addressed:?}"
    );
    assert!(
        addressed.len() >= 2,
        "the leader must still be replicating to the members it kept, addressed {addressed:?}"
    );
}

#[test]
fn removing_a_voter_from_a_three_voter_group_is_refused_because_tolerance_would_reach_zero() {
    let ids = [0, 1, 2];
    let mut bus = Bus::new(&ids, &voters(&ids));
    let leader = bus.run_until_leader();
    bus.commit(leader, b"before".to_vec());
    let victim = bus.some_other_voter(leader);

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .remove_member(victim)
        .expect_err("three voters tolerate one failure and two tolerate none");
    assert_eq!(
        refusal,
        RemovalRefused::ToleranceWouldDrop {
            before: 1,
            after: 0
        },
        "the refusal must name both tolerances so an operator can see what it would have cost"
    );

    assert_eq!(
        bus.nodes[&leader].conf_state().membership.voters,
        vec![0, 1, 2],
        "a refused removal must not have proposed anything"
    );
    assert!(
        !bus.nodes[&leader].is_joint(),
        "a refused removal must not leave the group mid-transition"
    );
}

#[test]
fn removing_a_voter_from_a_five_voter_group_is_refused_although_four_voters_remain() {
    let ids = [0, 1, 2, 3, 4];
    let mut bus = Bus::new(&ids, &voters(&ids));
    let leader = bus.run_until_leader();
    bus.commit(leader, b"before".to_vec());
    let victim = bus.some_other_voter(leader);

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .remove_member(victim)
        .expect_err("five voters tolerate two failures and four tolerate one");
    assert_eq!(
        refusal,
        RemovalRefused::ToleranceWouldDrop {
            before: 2,
            after: 1
        },
        "the guard is about tolerance, not about staying above three voters"
    );

    assert_eq!(
        bus.nodes[&leader].conf_state().membership.voters.len(),
        5,
        "a refused removal must not have proposed anything"
    );
}

#[test]
fn removing_the_leader_is_refused_and_directs_the_caller_to_transfer_first() {
    let ids = [0, 1, 2, 3];
    let mut bus = Bus::new(&ids, &voters(&ids));
    let leader = bus.run_until_leader();
    bus.commit(leader, b"before".to_vec());

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .remove_member(leader)
        .expect_err("a leader cannot commit the entry that ends its own authority");
    assert_eq!(
        refusal,
        RemovalRefused::IsTheLeader { target: leader },
        "the refusal must name the node, so the caller knows which one to transfer away from"
    );

    assert_eq!(
        bus.nodes[&leader].conf_state().membership.voters.len(),
        4,
        "a refused removal must not have proposed anything"
    );

    // The refusal is only useful if the route it implies works: transfer, then
    // remove. Four voters going to three holds the tolerance at one.
    let successor = bus.some_other_voter(leader);
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .transfer_leadership(successor)
        .expect("every voter is caught up, so the transfer is accepted");
    bus.settle();
    let new_leader = bus.leader().expect("the transfer elects the named voter");
    assert_eq!(
        new_leader, successor,
        "leadership must arrive at the node that was named"
    );
    bus.nodes
        .get_mut(&new_leader)
        .unwrap()
        .remove_member(leader)
        .expect("the former leader is an ordinary voter now");
    bus.settle();
    assert!(
        !bus.nodes[&new_leader]
            .conf_state()
            .membership
            .voters
            .contains(&leader),
        "the route the refusal named must actually remove the node"
    );
}

#[test]
fn a_demoted_voter_stops_campaigning_while_the_group_still_elects() {
    let ids = [0, 1, 2, 3];
    let mut bus = Bus::new(&ids, &voters(&ids));
    let leader = bus.run_until_leader();
    bus.commit(leader, b"before".to_vec());
    let victim = bus.some_other_voter(leader);

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .demote_voter(victim)
        .expect("four voters tolerate one failure and three still do, so this demotion is safe");
    bus.settle();

    let conf = bus.nodes[&leader].conf_state().clone();
    assert!(
        !bus.nodes[&leader].is_joint(),
        "the transition must reach its final configuration"
    );
    assert!(
        !conf.membership.voters.contains(&victim),
        "the demoted node must leave the voter set, found {:?}",
        conf.membership.voters
    );
    assert!(
        conf.membership.learners.contains(&victim),
        "demotion is not removal: the node must remain a member, as a learner"
    );
    assert!(
        !bus.nodes[&victim].is_voter(),
        "the demoted node must know it is no longer a voter"
    );

    // Silence the leader. The demoted node must sit out the election that
    // follows, and the remaining voters must hold one — asserting only the
    // first half is satisfied by a group that has stopped working entirely.
    bus.dropped.insert(leader);
    let mut elected = None;
    for _ in 0..SILENT_TICKS {
        bus.tick_all();
        bus.pump();
        assert_ne!(
            bus.nodes[&victim].role(),
            Role::Candidate,
            "a demoted node must not campaign"
        );
        if elected.is_none() {
            elected = bus.leader();
        }
    }
    let elected =
        elected.expect("the three remaining voters must elect a leader without the demoted node");
    assert_ne!(
        elected, victim,
        "the demoted node must not win an election it was not allowed to start"
    );
    assert!(
        !bus.nodes[&victim].is_leader(),
        "the demoted node must not be leader at the end of the silent window"
    );
}

#[test]
fn demoting_a_voter_from_a_three_voter_group_is_refused_on_the_same_arithmetic_as_removal() {
    let ids = [0, 1, 2];
    let mut bus = Bus::new(&ids, &voters(&ids));
    let leader = bus.run_until_leader();
    bus.commit(leader, b"before".to_vec());
    let victim = bus.some_other_voter(leader);

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .demote_voter(victim)
        .expect_err("demotion shrinks the voter set by one exactly as removal does");
    assert_eq!(
        refusal,
        DemotionRefused::ToleranceWouldDrop {
            before: 1,
            after: 0
        },
        "a demotion guarded only by `the target is a voter` reaches two voters by the path a \
         refused removal is stopped from taking"
    );

    assert_eq!(
        bus.nodes[&leader].conf_state().membership.voters,
        vec![0, 1, 2],
        "a refused demotion must not have proposed anything"
    );
    assert!(
        bus.nodes[&leader]
            .conf_state()
            .membership
            .learners
            .is_empty(),
        "a refused demotion must not have moved the target into the learner set"
    );
}

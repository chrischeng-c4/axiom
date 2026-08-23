//! Non-mutating selection of a caught-up voter to receive leadership (#3664).

use std::collections::{HashMap, HashSet};

use raft_core::{Membership, NodeId, RaftNode};

/// Voters 0,1,2 — at least three, so "the named node became leader" is not the
/// same statement as "the only other node became leader".
fn three_voters() -> Membership {
    Membership {
        voters: vec![0, 1, 2],
        learners: vec![],
    }
}

fn single_voter() -> Membership {
    Membership {
        voters: vec![0],
        learners: vec![],
    }
}

struct Bus {
    nodes: HashMap<NodeId, RaftNode>,
    applied: HashMap<NodeId, Vec<Vec<u8>>>,
    dropped: HashSet<NodeId>,
    ticks: u64,
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
            ticks: 0,
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

    fn tick_live(&mut self) {
        let live: Vec<NodeId> = self
            .nodes
            .keys()
            .copied()
            .filter(|id| !self.dropped.contains(id))
            .collect();
        for id in live {
            self.nodes.get_mut(&id).unwrap().tick();
            self.ticks += 1;
        }
    }

    fn settle(&mut self) {
        for _ in 0..40 {
            if let Some(leader) = self.leader() {
                self.nodes.get_mut(&leader).unwrap().tick();
                self.ticks += 1;
            }
            self.pump();
        }
    }

    fn leader(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|(id, n)| n.is_leader() && !self.dropped.contains(*id))
            .map(|(id, _)| *id)
    }

    fn run_until_leader(&mut self) -> NodeId {
        for _ in 0..200 {
            self.tick_live();
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

/// A node that is not the leader has no handoff candidate.
#[test]
fn follower_has_no_handoff_candidate() {
    let mut bus = Bus::new(&[0, 1, 2], &three_voters());
    let leader = bus.run_until_leader();
    let follower = *[0, 1, 2].iter().find(|id| **id != leader).unwrap();
    assert_eq!(
        bus.nodes[&follower].handoff_candidate(),
        None,
        "a follower must answer None for handoff_candidate"
    );
}

/// A single-voter leader has no candidate because no other voter exists.
#[test]
fn sole_voter_leader_has_no_handoff_candidate() {
    let mut bus = Bus::new(&[0], &single_voter());
    let leader = bus.run_until_leader();
    assert_eq!(leader, 0);
    assert_eq!(
        bus.nodes[&0].handoff_candidate(),
        None,
        "a sole-voter leader must answer None for handoff_candidate"
    );
}

/// A settled three-voter leader selects a peer voter that is not itself.
#[test]
fn settled_three_voter_leader_selects_eligible_peer_voter() {
    let mut bus = Bus::new(&[0, 1, 2], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    bus.settle();

    let candidate = bus.nodes[&leader]
        .handoff_candidate()
        .expect("a settled three-voter leader must name an eligible handoff candidate");
    assert_ne!(
        candidate, leader,
        "the handoff candidate must not be the leader itself"
    );
    assert!(
        [0, 1, 2].contains(&candidate),
        "the handoff candidate must be an admitted voter"
    );
}

/// Every candidate returned by `handoff_candidate` is accepted by `transfer_leadership`.
#[test]
fn selected_candidate_is_accepted_by_transfer_leadership() {
    let mut bus = Bus::new(&[0, 1, 2], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    bus.settle();

    let candidate = bus.nodes[&leader]
        .handoff_candidate()
        .expect("a settled leader has a handoff candidate");
    let res = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .transfer_leadership(candidate);
    assert!(
        res.is_ok(),
        "every target returned by handoff_candidate must be accepted by transfer_leadership"
    );
}

/// A leader whose peers are unreachable and behind selects nothing; restoring
/// delivery restores candidate selection.
#[test]
fn unreachable_and_behind_peers_yield_no_candidate_until_settled() {
    let mut bus = Bus::new(&[0, 1, 2], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    bus.settle();

    let peers: Vec<NodeId> = [0, 1, 2].into_iter().filter(|id| *id != leader).collect();
    for &p in &peers {
        bus.dropped.insert(p);
    }

    // Leader appends un-replicated entries to advance its log past all peers' match indices.
    for i in 5..10u8 {
        bus.nodes.get_mut(&leader).unwrap().propose(vec![i]);
    }
    bus.pump();

    assert_eq!(
        bus.nodes[&leader].handoff_candidate(),
        None,
        "when all peer voters are unreachable and behind, handoff_candidate must return None"
    );

    // Second half: restore peers and settle. Now candidate must be available.
    for &p in &peers {
        bus.dropped.remove(&p);
    }
    bus.settle();

    let candidate = bus.nodes[&leader]
        .handoff_candidate()
        .expect("once peers are restored and settled, a candidate must be found");
    assert!(
        peers.contains(&candidate),
        "candidate must be one of the peer voters"
    );
    assert!(
        bus.nodes
            .get_mut(&leader)
            .unwrap()
            .transfer_leadership(candidate)
            .is_ok(),
        "transfer_leadership to candidate must succeed"
    );
}

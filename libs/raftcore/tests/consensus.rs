//! Deterministic in-process simulation of the raftcore consensus engine — no
//! network, no clock, commands are opaque bytes. A bus pumps every node's outbox
//! into its target's handler until quiescent.

use std::collections::{HashMap, HashSet};

use raftcore::{auto_membership, AppendResp, NodeId, RaftMsg, RaftNode, Role};

struct Cluster {
    nodes: HashMap<NodeId, RaftNode>,
    dropped: HashSet<NodeId>,
    applied: HashMap<NodeId, Vec<Vec<u8>>>,
}

impl Cluster {
    fn new(n: u64) -> Cluster {
        let m = auto_membership(n);
        Cluster {
            nodes: (0..n).map(|id| (id, RaftNode::new(id, &m))).collect(),
            dropped: HashSet::new(),
            applied: (0..n).map(|id| (id, Vec::new())).collect(),
        }
    }

    fn pump(&mut self) {
        let dropped = self.dropped.clone();
        let mut guard = 0;
        loop {
            let mut msgs = Vec::new();
            for (id, node) in self.nodes.iter_mut() {
                let out = node.take_outgoing();
                if !dropped.contains(id) {
                    for o in out {
                        msgs.push((*id, o));
                    }
                }
            }
            if msgs.is_empty() {
                break;
            }
            for (from, o) in msgs {
                if dropped.contains(&from) || dropped.contains(&o.to) {
                    continue;
                }
                if let Some(n) = self.nodes.get_mut(&o.to) {
                    n.handle(from, o.msg);
                }
            }
            guard += 1;
            assert!(guard < 10_000, "pump did not converge");
        }
        for (id, node) in self.nodes.iter_mut() {
            if dropped.contains(id) {
                continue;
            }
            for e in node.take_committed() {
                self.applied.get_mut(id).unwrap().push(e.command);
            }
        }
    }

    fn tick(&mut self) {
        let dropped = self.dropped.clone();
        for (id, n) in self.nodes.iter_mut() {
            if !dropped.contains(id) {
                n.tick();
            }
        }
        self.pump();
    }

    fn leader(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|(id, n)| !self.dropped.contains(id) && n.is_leader())
            .map(|(id, _)| *id)
    }

    fn run_until_leader(&mut self) -> NodeId {
        for _ in 0..200 {
            self.tick();
            if let Some(l) = self.leader() {
                return l;
            }
        }
        panic!("no leader elected");
    }

    fn propose(&mut self, cmd: Vec<u8>) {
        let l = self.leader().expect("a leader");
        self.nodes.get_mut(&l).unwrap().propose(cmd).unwrap();
        self.pump();
        self.tick();
        self.tick();
    }
}

#[test]
fn elects_one_leader_and_replicates_in_order() {
    let mut c = Cluster::new(3);
    c.run_until_leader();
    assert_eq!(c.nodes.values().filter(|n| n.is_leader()).count(), 1);
    for i in 0..5u8 {
        c.propose(vec![i]);
    }
    let expected: Vec<Vec<u8>> = (0..5u8).map(|i| vec![i]).collect();
    for id in 0..3 {
        assert_eq!(c.applied[&id], expected, "node {id} converged in order");
    }
}

#[test]
fn kill_leader_reelects_without_losing_committed() {
    let mut c = Cluster::new(3);
    let old = c.run_until_leader();
    for i in 0..3u8 {
        c.propose(vec![i]);
    }
    let before = c.applied[&old].clone();
    c.dropped.insert(old);
    let new = c.run_until_leader();
    assert_ne!(new, old);
    for id in (0..3).filter(|id| *id != old) {
        assert_eq!(&c.applied[&id][..3], &before[..], "survivor kept committed");
    }
    c.propose(vec![9]);
    for id in (0..3).filter(|id| *id != old) {
        assert_eq!(c.applied[&id].last(), Some(&vec![9u8]));
    }
}

#[test]
fn learner_applies_but_never_votes_or_counts() {
    let m = auto_membership(4);
    assert_eq!(m.voters, vec![0, 1, 2]);
    assert_eq!(m.learners, vec![3]);

    let mut c = Cluster::new(4);
    let leader = c.run_until_leader();
    assert!(m.voters.contains(&leader));
    for i in 0..3u8 {
        c.propose(vec![i]);
    }
    assert_eq!(
        c.applied[&3],
        vec![vec![0], vec![1], vec![2]],
        "learner applied"
    );
    assert_eq!(
        c.nodes[&3].role(),
        Role::Follower,
        "learner never campaigns"
    );

    // Drop 2 of 3 voters: the lone voter + learner cannot form a majority.
    let other_voters: Vec<NodeId> = (0..3).filter(|v| *v != leader).collect();
    c.dropped.insert(leader);
    c.dropped.insert(other_voters[0]);
    for _ in 0..50 {
        c.tick();
    }
    assert!(
        c.leader().is_none(),
        "learner cannot substitute for a voter"
    );
}

#[test]
fn stale_leader_steps_down_on_higher_term() {
    let mut c = Cluster::new(3);
    let leader = c.run_until_leader();
    let term = c.nodes[&leader].current_term();
    c.nodes.get_mut(&leader).unwrap().handle(
        99,
        RaftMsg::AppendResp(AppendResp {
            term: term + 5,
            success: false,
            match_index: 0,
        }),
    );
    assert_eq!(c.nodes[&leader].role(), Role::Follower);
    assert_eq!(c.nodes[&leader].current_term(), term + 5);
}

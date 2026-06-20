//! Snapshot / log-compaction: a compacted leader still brings a lagging follower
//! fully up to date by shipping its state-machine snapshot, and the durable hard
//! state round-trips the compaction point.

use std::collections::{HashMap, HashSet};

use raftcore::{auto_membership, NodeId, RaftNode};

/// State machine = the ordered list of applied commands. A snapshot is the
/// serialized prefix; installing one replaces the baseline, then committed
/// entries append onto it.
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
            // An installed snapshot resets the state-machine baseline...
            if let Some(snap) = node.take_installed_snapshot() {
                self.applied
                    .insert(*id, serde_json::from_slice(&snap).unwrap());
            }
            // ...then committed entries fold on top.
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

    /// Compact `node` up to `up_to`, snapshotting its applied prefix.
    fn compact(&mut self, node: NodeId, up_to: usize) {
        let snap = serde_json::to_vec(&self.applied[&node][..up_to]).unwrap();
        self.nodes
            .get_mut(&node)
            .unwrap()
            .compact(up_to as u64, snap);
    }
}

#[test]
fn compact_frees_the_log_but_keeps_committed_state() {
    let mut c = Cluster::new(1); // sole voter commits immediately
    c.run_until_leader();
    for i in 0..10u8 {
        c.propose(vec![i]);
    }
    assert_eq!(c.nodes[&0].log_len(), 10);
    c.compact(0, 8);
    assert_eq!(c.nodes[&0].snapshot_index(), 8);
    assert_eq!(
        c.nodes[&0].log_len(),
        2,
        "only indices 9,10 remain resident"
    );
    assert_eq!(c.nodes[&0].commit_index(), 10, "committed state unaffected");
    // Can still propose + commit after compaction.
    c.propose(vec![100]);
    assert_eq!(c.applied[&0].last(), Some(&vec![100u8]));
}

#[test]
fn lagging_follower_catches_up_via_snapshot() {
    let mut c = Cluster::new(3);
    let leader = c.run_until_leader();
    let follower = (0..3).find(|i| *i != leader).unwrap();

    // Isolate a follower, commit a batch with the remaining majority.
    c.dropped.insert(follower);
    for i in 0..10u8 {
        c.propose(vec![i]);
    }
    // Compact the leader past what the isolated follower has.
    c.compact(leader, 8);
    assert!(c.nodes[&leader].log_len() < 10, "leader log compacted");

    // Bring the follower back: it is behind the compaction point, so it must be
    // caught up via InstallSnapshot + the remaining tail.
    c.dropped.remove(&follower);
    for _ in 0..80 {
        c.tick();
    }
    let expected: Vec<Vec<u8>> = (0..10u8).map(|i| vec![i]).collect();
    assert_eq!(
        c.applied[&follower], expected,
        "follower fully recovered (8 from snapshot + 2 from the log tail)"
    );
    assert_eq!(c.nodes[&follower].snapshot_index(), 8);
}

#[test]
fn persisted_state_round_trips_the_snapshot() {
    let mut c = Cluster::new(1);
    c.run_until_leader();
    for i in 0..10u8 {
        c.propose(vec![i]);
    }
    c.compact(0, 8);

    let ps = c.nodes[&0].persisted();
    assert_eq!(ps.snapshot_index, 8);
    assert!(!ps.snapshot.is_empty());
    assert_eq!(ps.log.len(), 2);

    let m = auto_membership(1);
    let restored = RaftNode::from_persisted(0, &m, ps);
    assert_eq!(restored.snapshot_index(), 8);
    assert_eq!(restored.last_index(), 10, "snapshot_index 8 + 2 resident");
}

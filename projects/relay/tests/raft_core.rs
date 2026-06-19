// SPEC-MANAGED: projects/relay/tech-design/logic/single-shard-raft-consensus-core-self-contained-rsm-auto-voter-l.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:b72bd304" tracker="pending-tracker" reason="Deterministic in-process simulation: a message bus pumps node outboxes to handlers. Tests leader election, replicate+commit ordering, kill-leader -> re-elect with no committed loss, learner replicates/applies but never votes nor counts toward majority, stale higher-term step-down, and a relay-integration scenario (command=publish, apply=relay engine) that converges across a leader failover."
//! Single-shard Raft core (#136): a deterministic in-process simulation. A
//! message bus pumps every node's outbox to its target's handler until
//! quiescent, so the whole protocol runs with no real network or clock.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::Utc;

use relay::raft::{auto_membership, AppendResp, NodeId, RaftMsg, RaftNode, Role};
use relay::{Relay, RelayCoreConfig};

/// A deterministic cluster: owns the nodes, pumps messages, and records every
/// committed command per node (the "apply" side of the state machine).
struct Cluster {
    nodes: HashMap<NodeId, RaftNode>,
    dropped: HashSet<NodeId>,
    applied: HashMap<NodeId, Vec<Vec<u8>>>,
}

impl Cluster {
    fn new(n: u64) -> Cluster {
        let m = auto_membership(n);
        let nodes = (0..n).map(|id| (id, RaftNode::new(id, &m))).collect();
        let applied = (0..n).map(|id| (id, Vec::new())).collect();
        Cluster {
            nodes,
            dropped: HashSet::new(),
            applied,
        }
    }

    /// Deliver all outboxes to targets until no messages remain, then drain each
    /// live node's committed entries into `applied`.
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
        // a couple of ticks let the commit watermark reach everyone
        self.tick();
        self.tick();
    }
}

#[test]
fn elects_exactly_one_leader() {
    let mut c = Cluster::new(3);
    c.run_until_leader();
    let leaders: Vec<_> = c.nodes.values().filter(|n| n.is_leader()).collect();
    assert_eq!(leaders.len(), 1, "exactly one leader");
    let term = leaders[0].current_term();
    for n in c.nodes.values() {
        assert_eq!(n.current_term(), term, "all on the leader's term");
        if !n.is_leader() {
            assert_eq!(n.role(), Role::Follower);
        }
    }
}

#[test]
fn replicates_and_commits_in_order() {
    let mut c = Cluster::new(3);
    c.run_until_leader();
    for i in 0..5u8 {
        c.propose(vec![i]);
    }
    let expected: Vec<Vec<u8>> = (0..5u8).map(|i| vec![i]).collect();
    for id in 0..3 {
        assert_eq!(c.applied[&id], expected, "node {id} applied all in order");
        assert!(c.nodes[&id].commit_index() >= 5);
    }
}

#[test]
fn kill_leader_reelects_with_no_committed_loss() {
    let mut c = Cluster::new(3);
    let old = c.run_until_leader();
    for i in 0..3u8 {
        c.propose(vec![i]);
    }
    let committed_before = c.applied[&old].clone();
    assert_eq!(committed_before.len(), 3);

    // Kill the leader.
    c.dropped.insert(old);
    let new = c.run_until_leader();
    assert_ne!(new, old, "a different node leads");

    // Every survivor still has the pre-kill committed entries (no loss)...
    for id in (0..3).filter(|id| *id != old) {
        assert_eq!(
            &c.applied[&id][..3],
            &committed_before[..],
            "survivor {id} kept committed entries"
        );
    }
    // ...and the new leader can commit fresh entries.
    c.propose(vec![99]);
    for id in (0..3).filter(|id| *id != old) {
        assert_eq!(c.applied[&id].last(), Some(&vec![99u8]));
    }
}

#[test]
fn learner_replicates_but_never_votes_or_counts() {
    // N=4 -> voters {0,1,2}, learner {3}.
    let m = auto_membership(4);
    assert_eq!(m.voters, vec![0, 1, 2]);
    assert_eq!(m.learners, vec![3]);

    let mut c = Cluster::new(4);
    let leader = c.run_until_leader();
    assert!(m.voters.contains(&leader), "a voter leads, not the learner");
    assert!(!c.nodes[&3].is_voter());

    for i in 0..3u8 {
        c.propose(vec![i]);
    }
    // The learner replicated AND applied every committed entry...
    assert_eq!(c.applied[&3], vec![vec![0], vec![1], vec![2]]);
    // ...yet it never left Follower (never campaigned).
    assert_eq!(c.nodes[&3].role(), Role::Follower);

    // The learner does NOT count toward quorum: drop two of three voters and no
    // leader can form (1 voter + 1 learner < majority of 2).
    let voters_up: Vec<NodeId> = (0..3).filter(|v| *v != leader).collect();
    c.dropped.insert(leader);
    c.dropped.insert(voters_up[0]);
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
    // Deliver a message from a future term.
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

#[test]
fn relay_engines_converge_across_failover() {
    // command = a publish; apply = publish into that node's relay engine.
    fn cmd(i: usize) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({ "id": format!("m{i}"), "v": i })).unwrap()
    }

    let mut c = Cluster::new(3);
    let old = c.run_until_leader();
    for i in 0..4 {
        c.propose(cmd(i));
    }
    c.dropped.insert(old);
    c.run_until_leader();
    for i in 4..7 {
        c.propose(cmd(i));
    }

    // Replay each survivor's committed commands into a fresh relay engine and
    // assert they all hold the identical message set.
    let mut sets = Vec::new();
    for id in (0..3).filter(|id| *id != old) {
        let engine = Relay::new(RelayCoreConfig::in_memory());
        for raw in &c.applied[&id] {
            let v: serde_json::Value = serde_json::from_slice(raw).unwrap();
            let mid = v["id"].as_str().unwrap();
            engine
                .publish("s", mid, v.clone(), BTreeMap::new(), Utc::now())
                .unwrap();
        }
        engine.subscribe("s", "r", 0).unwrap();
        let ids: Vec<String> = engine
            .poll("s", "r")
            .unwrap()
            .into_iter()
            .map(|e| e.message_id)
            .collect();
        sets.push(ids);
    }
    assert_eq!(sets[0], sets[1], "survivors' engines converge");
    assert_eq!(sets[0].len(), 7, "all 7 committed publishes applied");
}
// HANDWRITE-END

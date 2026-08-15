//! A leader may not demote itself, and neither membership operation may leave
//! the group with no voters (#3587).
//!
//! # Two rules, not one
//!
//! `remove_member` already refuses a self-target by name. `demote_voter` never
//! compares `peer` against `self.id`, so a leader is an ordinary target to it,
//! and the tolerance guard does not stop the walk down: `before` and `after`
//! are both zero at every size at or below two, so the last shrink is always
//! permitted. The two rules below are separate because each is reachable while
//! the other holds, and a single check cannot stand for both.
//!
//! # Why the self-demotion rows use two group sizes
//!
//! At three voters *every* demotion is already refused — `before` is 1 and
//! `after` is 0 — so a row that only demotes a leader in a three-voter group
//! passes against an implementation that added nothing at all, and passes again
//! against one that returns the tolerance answer for a self-target. The
//! four-voter row is the one that separates them: there a non-self demotion
//! succeeds, so a refusal can only be about *who* was named. The three-voter
//! row then pins the ordering, requiring the self answer even where the
//! tolerance guard would otherwise have fired. Neither size can be dropped: the
//! first alone leaves the check free to sit after the tolerance guard, the
//! second alone is satisfied by the guard that is already there.
//!
//! # Why the floor rows put the leader outside the voter set
//!
//! With a self-refusal in place, `demote_voter` and `remove_member` can only
//! empty the voter set when the leader is not in it — every other route ends at
//! the leader refusing to name itself. That state is reachable through the
//! public API and is not constructed by hand here: `libs/raft-core/src/lib.rs`
//! `adopt_conf` recomputes `peers`, `is_voter` and the configuration from the
//! adopted state and does *not* step a leader down, and `raft-runtime` calls it
//! on every committed configuration entry. So the rows reach the state the way
//! production reaches it, and `adopt_conf` remains the only writer of the three
//! fields.
//!
//! # Why each floor row carries a two-voter control
//!
//! "The voter set cannot be emptied" and "a leader that is not a voter may not
//! change the membership" are different rules that agree on the refusal above.
//! Each floor row therefore also performs the same operation from the same
//! leader-outside-the-voter-set state with one more voter present, and requires
//! it to succeed. Without that half, a blanket refusal keyed on `is_voter`
//! passes both rows while forbidding a great deal the item does not ask for.

use std::collections::{HashMap, HashSet};

use raft_core::{
    ConfState, DemotionRefused, Membership, NodeId, RaftNode, RemovalRefused, Role,
};

fn voters(ids: &[NodeId]) -> Membership {
    Membership {
        voters: ids.to_vec(),
        learners: vec![],
    }
}

struct Bus {
    nodes: HashMap<NodeId, RaftNode>,
    dropped: HashSet<NodeId>,
}

impl Bus {
    fn new(ids: &[NodeId]) -> Self {
        let start = voters(ids);
        Bus {
            nodes: ids
                .iter()
                .map(|id| (*id, RaftNode::new(*id, &start)))
                .collect(),
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
        for node in self.nodes.values_mut() {
            let _ = node.take_installed_snapshot();
            let _ = node.take_committed();
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

    /// Tick the leader and deliver, repeatedly, so the entry a fresh leader
    /// appends on election commits before a row asks about the configuration.
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

    fn leader(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|(id, n)| n.is_leader() && !self.dropped.contains(*id))
            .map(|(id, _)| *id)
    }

    /// Elect a leader and let its first entry commit, so no uncommitted entry
    /// is outstanding when a row calls a membership operation.
    fn settled_leader(&mut self) -> NodeId {
        for _ in 0..200 {
            self.tick_all();
            self.pump();
            if let Some(id) = self.leader() {
                self.settle();
                return id;
            }
        }
        panic!("no leader was elected");
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

    /// Move `leader` out of the voter set and leave `remaining` as the voters,
    /// through the one API that writes those fields. `adopt_conf` refuses a
    /// configuration that does not supersede the one in force, so the row
    /// checks its return value rather than assuming the state changed.
    fn adopt_leader_out_of_the_voter_set(&mut self, leader: NodeId, remaining: &[NodeId]) {
        let generation = self.nodes[&leader].conf_state().generation + 1;
        let node = self.nodes.get_mut(&leader).unwrap();
        let adopted = node.adopt_conf(ConfState {
            membership: Membership {
                voters: remaining.to_vec(),
                learners: vec![leader],
            },
            outgoing: None,
            generation,
        });
        assert!(adopted, "the superseding configuration must be adopted");
        assert_eq!(
            node.role(),
            Role::Leader,
            "the premise of these rows is that adopting a configuration without \
             yourself does not step you down; if raft-core starts stepping down \
             here, these rows must be rewritten rather than deleted, because the \
             floor still has to hold for whatever reaches the operation",
        );
        assert!(
            !node.is_voter(),
            "the leader must be outside the voter set for this row to be about \
             the floor at all",
        );
        assert_eq!(
            node.conf_state().membership.voters,
            remaining.to_vec(),
            "the voter set must be exactly what was adopted",
        );
    }
}

#[test]
fn a_leader_demoting_itself_is_refused_by_name_where_demotion_is_otherwise_permitted() {
    // Four voters: dropping one leaves three, and both sizes tolerate one
    // failure, so the tolerance guard permits this shrink. Any refusal here is
    // therefore about the target being the leader and nothing else.
    let mut bus = Bus::new(&[0, 1, 2, 3]);
    let leader = bus.settled_leader();
    let before = bus.nodes[&leader].conf_state().clone();

    assert_eq!(
        bus.nodes.get_mut(&leader).unwrap().demote_voter(leader),
        Err(DemotionRefused::IsTheLeader { target: leader }),
        "a leader that demotes itself walks the group toward a configuration it \
         cannot lead and cannot repair; the caller's route is to transfer \
         leadership first",
    );
    assert_eq!(
        bus.nodes[&leader].conf_state(),
        &before,
        "a refused demotion must not have proposed a configuration",
    );
    assert!(
        !bus.nodes[&leader].is_joint(),
        "a refused demotion must not have entered a joint configuration",
    );

    // The control, on a fresh group of the same shape: naming somebody else
    // succeeds. Without it, an implementation that refuses every demotion at
    // this size passes the assertion above.
    let mut control = Bus::new(&[0, 1, 2, 3]);
    let control_leader = control.settled_leader();
    let other = control.some_other_voter(control_leader);
    assert!(
        control
            .nodes
            .get_mut(&control_leader)
            .unwrap()
            .demote_voter(other)
            .is_ok(),
        "four voters tolerate one failure and three still do, so demoting a \
         voter that is not the leader is permitted at this size and the row \
         above measures the target, not the size",
    );
}

#[test]
fn the_self_demotion_refusal_is_answered_before_the_tolerance_guard() {
    // Three voters: dropping one takes the tolerance from 1 to 0, so *every*
    // demotion is refused at this size. What must not happen is the leader
    // receiving the tolerance answer when it named itself -- asking the wrong
    // node and asking at the wrong time are different answers, and only the
    // first tells the caller that transferring leadership is the way out.
    let mut bus = Bus::new(&[0, 1, 2]);
    let leader = bus.settled_leader();
    let other = bus.some_other_voter(leader);

    assert_eq!(
        bus.nodes.get_mut(&leader).unwrap().demote_voter(other),
        Err(DemotionRefused::ToleranceWouldDrop {
            before: 1,
            after: 0
        }),
        "the premise of this row is that the tolerance guard fires at three \
         voters; #3572 fixed this verdict and it must not have moved",
    );
    assert_eq!(
        bus.nodes.get_mut(&leader).unwrap().demote_voter(leader),
        Err(DemotionRefused::IsTheLeader { target: leader }),
        "the self-target must be answered even where the tolerance guard would \
         also have refused, which places the check before it",
    );
}

#[test]
fn a_demotion_that_would_leave_no_voters_is_refused() {
    // One voter left and the leader is not it, so the self-refusal cannot
    // apply, and the tolerance arithmetic permits the shrink: at n = 1 both
    // `before` and `after` are zero. Only a floor stops this.
    let mut bus = Bus::new(&[0, 1]);
    let leader = bus.settled_leader();
    let last_voter = bus.some_other_voter(leader);
    bus.adopt_leader_out_of_the_voter_set(leader, &[last_voter]);

    assert_eq!(
        bus.nodes.get_mut(&leader).unwrap().demote_voter(last_voter),
        Err(DemotionRefused::WouldEmptyVoterSet {
            target: last_voter
        }),
        "demoting the only voter leaves a group that can never elect and can \
         never repair itself, and the tolerance guard permits it because zero \
         failures tolerated before is not fewer than zero after",
    );
    assert!(
        !bus.nodes[&leader].is_joint(),
        "a refused demotion must not have entered a joint configuration",
    );

    // The control: the same operation from the same leader-outside-the-voter-set
    // state, with a voter left over afterwards, succeeds. This is what stops the
    // floor from being written as "a leader that is not a voter may not change
    // the membership", which refuses far more than the item asks for.
    let mut control = Bus::new(&[0, 1, 2]);
    let control_leader = control.settled_leader();
    let remaining: Vec<NodeId> = [0, 1, 2]
        .into_iter()
        .filter(|id| *id != control_leader)
        .collect();
    control.adopt_leader_out_of_the_voter_set(control_leader, &remaining);
    assert!(
        control
            .nodes
            .get_mut(&control_leader)
            .unwrap()
            .demote_voter(remaining[0])
            .is_ok(),
        "one voter still remains afterwards, so the floor has nothing to say \
         and the demotion is permitted",
    );
}

#[test]
fn a_removal_that_would_leave_no_voters_is_refused() {
    // Removal carries the same floor as demotion: both shrink the voter set by
    // one, and #3572's tolerance guard is silent at this size for both.
    let mut bus = Bus::new(&[0, 1]);
    let leader = bus.settled_leader();
    let last_voter = bus.some_other_voter(leader);
    bus.adopt_leader_out_of_the_voter_set(leader, &[last_voter]);

    assert_eq!(
        bus.nodes.get_mut(&leader).unwrap().remove_member(last_voter),
        Err(RemovalRefused::WouldEmptyVoterSet {
            target: last_voter
        }),
        "removing the only voter empties the voter set by the same path a \
         refused demotion is stopped from taking",
    );
    assert!(
        !bus.nodes[&leader].is_joint(),
        "a refused removal must not have entered a joint configuration",
    );

    let mut control = Bus::new(&[0, 1, 2]);
    let control_leader = control.settled_leader();
    let remaining: Vec<NodeId> = [0, 1, 2]
        .into_iter()
        .filter(|id| *id != control_leader)
        .collect();
    control.adopt_leader_out_of_the_voter_set(control_leader, &remaining);
    assert!(
        control
            .nodes
            .get_mut(&control_leader)
            .unwrap()
            .remove_member(remaining[0])
            .is_ok(),
        "one voter still remains afterwards, so the floor has nothing to say \
         and the removal is permitted",
    );
}

//! Promoting a caught-up learner to voter through a joint configuration (#3570).
//!
//! # Why the rows use two different group sizes
//!
//! A promotion adds one voter, so the incoming set is the outgoing set plus one
//! member: `old ⊂ new`, `|new| = |old| + 1`. That containment makes one of the
//! two directions vacuous, and which one depends on the parity of `|old|`.
//!
//! * `|old| = 2k+1`: `maj_old = k+1`, `maj_new = k+2`. A set of `k+1` outgoing
//!   voters is an outgoing majority and is one short of an incoming majority. But
//!   an incoming majority holds `k+2` members of `new`, at most one of which is
//!   the new voter, so it always contains `k+1` outgoing voters — an outgoing
//!   majority too.
//! * `|old| = 2k`: `maj_old = k+1`, `maj_new = k+1`. Now an incoming majority
//!   that includes the new voter holds only `k` outgoing voters and is *not* an
//!   outgoing majority, while any outgoing majority is `k+1` members of `new` and
//!   so is an incoming majority as well.
//!
//! Enumerating every subset confirms it — `old-majority but not new-majority` /
//! `new-majority but not old-majority`:
//!
//! ```text
//! |old|=3  maj_old=2  |new|=4  maj_new=3    3 sets    0 sets
//! |old|=4  maj_old=3  |new|=5  maj_new=3    0 sets    6 sets
//! |old|=5  maj_old=3  |new|=6  maj_new=4   10 sets    0 sets
//! |old|=6  maj_old=4  |new|=7  maj_new=4    0 sets   20 sets
//! ```
//!
//! So the two withholding rows below cannot share a group size. Collapsing them
//! onto one is the silent way to lose half the evidence: whichever size is kept,
//! the other row's partition becomes a set that legitimately commits, and the row
//! then asserts that a commit which *should* happen did not — which no correct
//! implementation can satisfy, or worse, asserts nothing because the set it
//! chose satisfies both quorums.
//!
//! # Why a configuration takes effect on apply here
//!
//! `adopt_conf` is reached from exactly one place, `take_committed`
//! (`libs/raft-core/src/lib.rs:542`), so a node adopts a configuration when it
//! *applies* the entry, not when it appends it. Two consequences the rows depend
//! on:
//!
//! * the joint entry itself commits under the outgoing configuration, because
//!   that is still the configuration in force while it is in flight;
//! * a node is only joint after a driver has called `take_committed`, which the
//!   bus below does once per round, during the drain pass.
//!
//! Messages are delivered only at the start of a round, so an entry appended
//! during a drain pass cannot be acknowledged until the following round. That is
//! what makes "run rounds until the leader is joint, then partition" a
//! deterministic way to hold a group in the joint state rather than a race.

use std::collections::{HashMap, HashSet};

use raft_core::{Membership, NodeId, PromotionRefused, RaftNode};

/// Voters 0,1,2. Odd, so an outgoing majority can be one short of an incoming
/// one — the size at which the outgoing-alone row can measure anything.
fn three_voters() -> Membership {
    Membership {
        voters: vec![0, 1, 2],
        learners: vec![],
    }
}

/// Voters 0,1,2,3. Even, so an incoming majority that leans on the new voter is
/// not an outgoing majority — the size at which the incoming-alone row can
/// measure anything.
fn four_voters() -> Membership {
    Membership {
        voters: vec![0, 1, 2, 3],
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
    /// configuration is first contacted by a heartbeat, so a bus that only drains
    /// outboxes never reaches it. Every real driver ticks.
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

    /// Run rounds until `leader` has applied a joint configuration. Deterministic
    /// because the entry that leaves the joint state is appended during a drain
    /// pass and cannot be acknowledged before the next delivery pass, so the
    /// caller gets to partition first.
    fn round_until_joint(&mut self, leader: NodeId) {
        for _ in 0..200 {
            if self.nodes[&leader].is_joint() {
                return;
            }
            self.round();
        }
        panic!("the leader never entered a joint configuration");
    }

    /// The voters of the configuration `node` currently has in force.
    fn voters_of(&self, node: NodeId) -> Vec<NodeId> {
        self.nodes[&node].conf_state().membership.voters.clone()
    }
}

/// A group of `start`'s voters, plus `newcomer` admitted as a caught-up learner.
/// Returns the bus and the leader.
fn group_with_caught_up_learner(
    ids: &[NodeId],
    start: &Membership,
    newcomer: NodeId,
) -> (Bus, NodeId) {
    let mut bus = Bus::new(ids, start);
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .add_learner(newcomer)
        .expect("a leader admits a learner");
    bus.settle();
    assert_eq!(
        bus.nodes[&leader].learner_read_eligible(newcomer),
        Some(true),
        "this file's rows all start from a learner that has caught up; if it has \
         not, every promotion below is refused for the wrong reason"
    );
    (bus, leader)
}

/// R4, the whole item. A caught-up learner becomes a voter, and the group ends
/// on one configuration with the learner gone from the learner list.
#[test]
fn a_caught_up_learner_becomes_a_voter_and_the_group_leaves_the_joint_state() {
    let (mut bus, leader) = group_with_caught_up_learner(&[0, 1, 2, 3], &three_voters(), 3);

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(3)
        .expect("a leader promotes a caught-up learner");
    bus.settle();

    let node = &bus.nodes[&leader];
    assert!(
        !node.is_joint(),
        "every voter was reachable throughout, so the transition had nothing to \
         wait for and the group must not still be joint"
    );
    assert_eq!(
        node.conf_state().outgoing,
        None,
        "a group that is not joint must not still be carrying an outgoing set"
    );
    assert_eq!(
        bus.voters_of(leader),
        vec![0, 1, 2, 3],
        "node 3 was promoted, so it belongs in the voter set"
    );
    assert!(
        !node.conf_state().membership.learners.contains(&3),
        "node 3 is a voter now; leaving it in the learner list would let it be \
         counted twice, once for each role"
    );
}

/// R4. While joint, a majority of the *outgoing* voters is not enough on its
/// own. Three outgoing voters: `maj_old = 2`, `maj_new = 3`, so the two
/// reachable nodes are an outgoing majority and one short of an incoming one.
#[test]
fn while_joint_a_majority_of_the_outgoing_voters_alone_cannot_commit() {
    let (mut bus, leader) = group_with_caught_up_learner(&[0, 1, 2, 3], &three_voters(), 3);

    // Keep the leader and one other outgoing voter. Dropping the third outgoing
    // voter and the newcomer leaves exactly {leader, other}: 2 of the outgoing 3
    // and 2 of the incoming 4.
    let other = [0, 1, 2].into_iter().find(|v| *v != leader).unwrap();
    for id in [0, 1, 2].into_iter().filter(|v| *v != leader && *v != other) {
        bus.dropped.insert(id);
    }
    bus.dropped.insert(3);

    let joint_at = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(3)
        .expect("a leader promotes a caught-up learner");
    bus.settle();

    let applied_before = bus.applied[&leader].len();
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .propose(vec![99])
        .expect("the leader appends a proposal while joint");
    bus.settle();

    let node = &bus.nodes[&leader];
    assert!(
        node.is_joint(),
        "{{{leader}, {other}}} is a majority of the outgoing voters {{0,1,2}} but \
         only 2 of the 4 incoming voters {{0,1,2,3}}, so the entry that leaves the \
         joint state cannot have committed and the group must still be joint"
    );
    assert_eq!(
        node.commit_index(),
        joint_at,
        "the joint entry at {joint_at} committed under the outgoing configuration, \
         and nothing after it may commit while only {{{leader}, {other}}} can be \
         reached"
    );
    assert_eq!(
        bus.applied[&leader].len(),
        applied_before,
        "a command acknowledged by an outgoing majority alone was applied; the \
         incoming configuration was not consulted"
    );
}

/// R4. While joint, a majority of the *incoming* voters is not enough on its
/// own either. Four outgoing voters: `maj_old = 3`, `maj_new = 3`, so the three
/// reachable nodes are an incoming majority while holding only 2 of the 4
/// outgoing voters. This is the direction an implementation that consults only
/// the incoming set gets wrong, and it is unmeasurable at the size the row above
/// uses — see this file's header.
#[test]
fn while_joint_a_majority_of_the_incoming_voters_alone_cannot_commit() {
    let (mut bus, leader) =
        group_with_caught_up_learner(&[0, 1, 2, 3, 4], &four_voters(), 4);

    let joint_at = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(4)
        .expect("a leader promotes a caught-up learner");

    // The joint entry commits under the outgoing configuration, which needs 3 of
    // {0,1,2,3}; no partition that blocks the entry leaving the joint state can
    // also let it in. So enter the joint state with everyone reachable, then
    // partition down to {leader, other, 4} before the next delivery pass.
    bus.round_until_joint(leader);
    let other = [0, 1, 2, 3].into_iter().find(|v| *v != leader).unwrap();
    for id in [0, 1, 2, 3]
        .into_iter()
        .filter(|v| *v != leader && *v != other)
    {
        bus.dropped.insert(id);
    }
    bus.settle();

    let applied_before = bus.applied[&leader].len();
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .propose(vec![99])
        .expect("the leader appends a proposal while joint");
    bus.settle();

    let node = &bus.nodes[&leader];
    assert!(
        node.is_joint(),
        "{{{leader}, {other}, 4}} is 3 of the 5 incoming voters {{0,1,2,3,4}} but \
         only 2 of the 4 outgoing voters {{0,1,2,3}}, so the entry that leaves the \
         joint state cannot have committed and the group must still be joint"
    );
    assert_eq!(
        node.commit_index(),
        joint_at,
        "the joint entry at {joint_at} committed while all four outgoing voters \
         were reachable; nothing after it may commit now that only \
         {{{leader}, {other}, 4}} can be"
    );
    assert_eq!(
        bus.applied[&leader].len(),
        applied_before,
        "a command acknowledged by an incoming majority alone was applied; the \
         outgoing configuration was not consulted"
    );
}

/// R4. A leader lost while the transition is in flight does not strand the
/// group. It elects a new leader and converges on the incoming configuration.
#[test]
fn a_leader_lost_mid_transition_is_replaced_and_the_group_converges_on_the_incoming_configuration()
{
    let (mut bus, leader) = group_with_caught_up_learner(&[0, 1, 2, 3], &three_voters(), 3);

    let other = [0, 1, 2].into_iter().find(|v| *v != leader).unwrap();
    let third = [0, 1, 2]
        .into_iter()
        .find(|v| *v != leader && *v != other)
        .unwrap();
    bus.dropped.insert(third);
    bus.dropped.insert(3);

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(3)
        .expect("a leader promotes a caught-up learner");
    bus.settle();
    assert!(
        bus.nodes[&leader].is_joint(),
        "this row needs the group actually held in the joint state before the \
         leader is lost, otherwise it measures an ordinary election"
    );

    // Lose the leader, restore everyone else. The survivors are 2 of the
    // outgoing 3 and 3 of the incoming 4 — a majority of both, so they can both
    // elect and commit.
    bus.dropped.clear();
    bus.dropped.insert(leader);

    let mut elected = None;
    for _ in 0..200 {
        for (id, node) in bus.nodes.iter_mut() {
            if *id != leader {
                node.tick();
            }
        }
        bus.pump();
        if let Some(id) = bus.leader().filter(|id| *id != leader) {
            elected = Some(id);
            break;
        }
    }
    let elected = elected.expect(
        "the survivors are a majority of both configurations, so they must be \
         able to elect a leader while the transition is in flight",
    );
    bus.settle();

    let node = &bus.nodes[&elected];
    assert!(
        !node.is_joint(),
        "node {elected} took over mid-transition with a majority of both \
         configurations reachable, so it must finish the transition rather than \
         leave the group joint forever"
    );
    assert_eq!(
        bus.voters_of(elected),
        vec![0, 1, 2, 3],
        "the group converged, and the configuration it converged on must be the \
         incoming one — converging back onto the outgoing set would silently undo \
         a promotion the operator was told had begun"
    );
}

/// R4. A learner that has not caught up is refused, and the refusal carries the
/// two indices that justify it rather than being reported as a bare failure.
#[test]
fn promoting_a_learner_that_has_not_caught_up_is_refused_with_both_indices() {
    let mut bus = Bus::new(&[0, 1, 2, 3], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }

    // Admitted while unreachable: the configuration entry still commits on the
    // three voters, so node 3 is a learner that has replicated nothing.
    bus.dropped.insert(3);
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .add_learner(3)
        .expect("a leader admits a learner");
    bus.settle();
    assert_eq!(
        bus.nodes[&leader].learner_read_eligible(3),
        Some(false),
        "this row needs a learner that has demonstrably not caught up"
    );

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(3)
        .expect_err("a learner that has not caught up must not be promoted");

    match refusal {
        PromotionRefused::NotCaughtUp { matched, target } => {
            assert!(
                matched < target,
                "the refusal is only justified if the learner is actually behind, \
                 yet it reports matched {matched} against target {target}"
            );
            assert_eq!(
                Some(matched),
                bus.nodes[&leader].learner_matched(3),
                "the refusal must report the leader's own record of this \
                 learner's progress, not a figure derived somewhere else"
            );
        }
        other => panic!(
            "a lagging learner must be refused for lagging, with the two indices \
             that justify it; got {other:?}"
        ),
    }

    assert!(
        !bus.nodes[&leader].is_joint(),
        "a refused promotion must not have appended a joint configuration"
    );
    assert_eq!(
        bus.voters_of(leader),
        vec![0, 1, 2],
        "a refused promotion must leave the voter set untouched"
    );
}

/// R4. One transition at a time. A second promotion begun while a joint
/// configuration is in flight is refused rather than queued.
#[test]
fn a_second_promotion_while_a_transition_is_in_flight_is_refused() {
    let mut bus = Bus::new(&[0, 1, 2, 3, 4], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    for newcomer in [3, 4] {
        bus.nodes
            .get_mut(&leader)
            .unwrap()
            .add_learner(newcomer)
            .expect("a leader admits a learner");
        bus.settle();
    }

    // Hold the group joint: {leader, other} is a majority of the outgoing 3 and
    // one short of the incoming 4.
    let other = [0, 1, 2].into_iter().find(|v| *v != leader).unwrap();
    for id in [0, 1, 2].into_iter().filter(|v| *v != leader && *v != other) {
        bus.dropped.insert(id);
    }
    bus.dropped.insert(3);
    bus.dropped.insert(4);

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(3)
        .expect("a leader promotes a caught-up learner");
    bus.settle();
    assert!(
        bus.nodes[&leader].is_joint(),
        "this row needs a transition genuinely still in flight"
    );

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(4)
        .expect_err("a second promotion must be refused while one is in flight");
    assert!(
        matches!(refusal, PromotionRefused::TransitionInFlight),
        "the second promotion must be refused for the reason that actually \
         applies, so an operator is not told the learner is behind when it is \
         not; got {refusal:?}"
    );

    let node = &bus.nodes[&leader];
    assert_eq!(
        node.conf_state().membership.voters,
        vec![0, 1, 2, 3],
        "the refused second promotion must not have altered the transition \
         already in flight"
    );
    assert!(
        node.conf_state()
            .membership
            .learners
            .contains(&4),
        "node 4 was not promoted, so it is still a learner"
    );
}

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
//! # Why the same pair is repeated for elections
//!
//! Quorum is decided at three sites — `majority()`, the vote count in
//! `maybe_become_leader`, and the acknowledgement count that advances
//! `commit_index` — and the two commit rows reach only the third. Each of them
//! drives the group with `settle`, which ticks the leader alone, so no election
//! ever runs in them and a mutation at the vote-counting site is inert
//! everywhere they look. `a_leader_lost_mid_transition...` does hold an election,
//! but its survivors are a majority of *both* configurations, so it elects with
//! either set consulted and cannot separate them either.
//!
//! The election rows therefore repeat the partition argument above at the same
//! two sizes, and for the same reason: one size can only show the outgoing set
//! being ignored, the other only the incoming set. Each also has a second half
//! that restores a node and requires an election to succeed, so "refuse every
//! election while joint" is not a way to pass the first half.
//!
//! Both election rows need every node joint before the partition is installed,
//! not just the leader — a candidate that has not itself entered the joint state
//! runs the single-configuration path, where the mutation being hunted does not
//! exist.
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

    /// The reachable leader. A node that has been dropped keeps `Leader` until
    /// it hears a higher term, which it cannot while it is unreachable, so a
    /// scan that ignores `dropped` reports the deposed leader and masks the one
    /// the survivors just elected. `consensus.rs` and `snapshot.rs` filter here
    /// for the same reason.
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

    /// Run rounds until *every* node in `ids` has applied the joint
    /// configuration, which is what the election rows need: a mutation at the
    /// vote-counting site is inert on a candidate that is not itself joint, so a
    /// row partitioning before the followers have entered the joint state would
    /// pass with the mutant in place.
    ///
    /// Panics if the leader leaves the joint state first. That ordering would
    /// mean the transition completed before the partition could be installed,
    /// and the row would then be measuring an ordinary election.
    fn round_until_all_joint(&mut self, ids: &[NodeId], leader: NodeId) {
        let mut leader_was_joint = false;
        for _ in 0..200 {
            if ids.iter().all(|id| self.nodes[id].is_joint()) {
                return;
            }
            if leader_was_joint && !self.nodes[&leader].is_joint() {
                panic!(
                    "the leader left the joint state before every node had entered it, so \
                     this row cannot hold the group joint long enough to measure an election"
                );
            }
            leader_was_joint |= self.nodes[&leader].is_joint();
            self.round();
        }
        panic!("not every node entered the joint configuration");
    }

    /// Tick every node that is not partitioned off and deliver, up to `rounds`
    /// times. Returns the leader among the *reachable* nodes: a leader that has
    /// been dropped still reports itself one in its own state, and counting it
    /// would let a stranded partition look like it had elected.
    fn run_until_reachable_leader(&mut self, rounds: usize) -> Option<NodeId> {
        for _ in 0..rounds {
            let live: Vec<NodeId> = self
                .nodes
                .keys()
                .copied()
                .filter(|id| !self.dropped.contains(id))
                .collect();
            for id in live {
                self.nodes.get_mut(&id).unwrap().tick();
            }
            self.pump();
            if let Some(id) = self
                .nodes
                .iter()
                .find(|(id, n)| n.is_leader() && !self.dropped.contains(id))
                .map(|(id, _)| *id)
            {
                return Some(id);
            }
        }
        None
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
    for id in [0, 1, 2]
        .into_iter()
        .filter(|v| *v != leader && *v != other)
    {
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
    let (mut bus, leader) = group_with_caught_up_learner(&[0, 1, 2, 3, 4], &four_voters(), 4);

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

/// R4, election side. While joint, a majority of the *outgoing* voters is not
/// enough to elect. Three outgoing voters: `maj_old = 2`, `maj_new = 3`, so the
/// two surviving outgoing voters are an outgoing majority and one short of an
/// incoming one. This is the direction an implementation counting votes against
/// the outgoing set alone gets wrong.
///
/// The commit rows above cannot stand in for this one: `settle` ticks only the
/// leader, so no election runs in any of them, and a vote-counting mutation is
/// inert everywhere they look.
#[test]
fn while_joint_a_majority_of_the_outgoing_voters_alone_cannot_elect() {
    let (mut bus, leader) = group_with_caught_up_learner(&[0, 1, 2, 3], &three_voters(), 3);

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(3)
        .expect("a leader promotes a caught-up learner");
    bus.round_until_all_joint(&[0, 1, 2, 3], leader);

    // Lose the leader and the newcomer. The survivors are the other two
    // outgoing voters: 2 of the outgoing 3, and 2 of the incoming 4.
    let survivors: Vec<NodeId> = [0, 1, 2].into_iter().filter(|v| *v != leader).collect();
    bus.dropped.insert(leader);
    bus.dropped.insert(3);

    assert_eq!(
        bus.run_until_reachable_leader(200),
        None,
        "{survivors:?} is a majority of the outgoing voters {{0,1,2}} but only 2 of the \
         4 incoming voters {{0,1,2,3}}, so while the transition is in flight they must \
         not be able to elect between themselves"
    );

    // Restoring the newcomer makes the reachable set a majority of both, so a
    // leader must appear. Without this half, refusing every election while joint
    // would pass the assertion above.
    bus.dropped.remove(&3);
    let elected = bus.run_until_reachable_leader(200).expect(
        "with the newcomer reachable the survivors are 2 of the outgoing 3 and 3 of the \
         incoming 4 — a majority of both — so they must be able to elect",
    );
    assert!(
        survivors.contains(&elected),
        "the newcomer is not a member of the outgoing configuration, so it must not be \
         the node that takes over while that configuration is still in force; got \
         {elected}"
    );
}

/// R4, election side. While joint, a majority of the *incoming* voters is not
/// enough to elect either. Four outgoing voters: `maj_old = 3`, `maj_new = 3`,
/// so the three survivors are an incoming majority while holding only 2 of the 4
/// outgoing voters. This is the direction an implementation counting votes
/// against the incoming set alone gets wrong, and — exactly as for the commit
/// pair — it is unmeasurable at the size the row above uses.
#[test]
fn while_joint_a_majority_of_the_incoming_voters_alone_cannot_elect() {
    let (mut bus, leader) = group_with_caught_up_learner(&[0, 1, 2, 3, 4], &four_voters(), 4);

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(4)
        .expect("a leader promotes a caught-up learner");
    bus.round_until_all_joint(&[0, 1, 2, 3, 4], leader);

    // Lose the leader and one more outgoing voter. The survivors are two
    // outgoing voters plus the newcomer: 3 of the incoming 5, 2 of the outgoing 4.
    let mut rest = [0, 1, 2, 3].into_iter().filter(|v| *v != leader);
    let a = rest.next().unwrap();
    let b = rest.next().unwrap();
    let c = rest.next().unwrap();
    bus.dropped.insert(leader);
    bus.dropped.insert(c);

    assert_eq!(
        bus.run_until_reachable_leader(200),
        None,
        "{{{a}, {b}, 4}} is 3 of the 5 incoming voters {{0,1,2,3,4}} but only 2 of the 4 \
         outgoing voters {{0,1,2,3}}, so while the transition is in flight they must not \
         be able to elect between themselves"
    );

    // Restoring the third outgoing voter makes the reachable set a majority of
    // both. Without this half, refusing every election while joint would pass.
    bus.dropped.remove(&c);
    let elected = bus.run_until_reachable_leader(200).unwrap_or_else(|| {
        panic!(
            "with {a}, {b}, {c} and 4 reachable the survivors are 3 of the outgoing 4 \
             and 4 of the incoming 5 — a majority of both — so they must be able to \
             elect"
        )
    });
    assert!(
        [a, b, c].contains(&elected),
        "the newcomer is not a member of the outgoing configuration, so it must not be \
         the node that takes over while that configuration is still in force; got \
         {elected}"
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
    for id in [0, 1, 2]
        .into_iter()
        .filter(|v| *v != leader && *v != other)
    {
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
        node.conf_state().membership.learners.contains(&4),
        "node 4 was not promoted, so it is still a learner"
    );
}

/// #3585. A node that is not the leader is told exactly that.
///
/// The group here is quiescent: no transition is in flight and no transfer is
/// pending, so `TransitionInFlight` would be a false statement about it rather
/// than a vague one. The row therefore separates the two conditions the guard
/// currently answers alike, which reading the voter list back cannot do.
///
/// The second half is the row's teeth. A refusal is cheap to produce by
/// refusing everything, so the same promotion is then made on the leader and
/// must succeed: the refusal has to be about which node was asked, not about
/// the request.
#[test]
fn promoting_on_a_node_that_is_not_the_leader_names_that_and_not_a_transition() {
    let (mut bus, leader) = group_with_caught_up_learner(&[0, 1, 2, 3], &three_voters(), 3);

    let mut followers: Vec<NodeId> = bus
        .voters_of(leader)
        .into_iter()
        .filter(|id| *id != leader)
        .collect();
    followers.sort_unstable();
    let follower = *followers
        .first()
        .expect("a three-voter group has a voter that is not its leader");

    assert!(
        !bus.nodes[&follower].is_joint(),
        "nothing has been promoted yet, so no transition is in flight and the \
         in-flight refusal would be false here rather than merely unhelpful"
    );

    let refusal = bus
        .nodes
        .get_mut(&follower)
        .unwrap()
        .promote_learner(3)
        .expect_err("a node that is not the leader cannot promote a learner");
    assert!(
        matches!(refusal, PromotionRefused::NotLeader),
        "the refusal must name the condition that applies to this node -- it is \
         not the leader -- rather than reporting a transition that does not \
         exist; got {refusal:?}"
    );

    assert_eq!(
        bus.voters_of(follower),
        vec![0, 1, 2],
        "a refused promotion must leave the configuration alone"
    );

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .promote_learner(3)
        .expect(
            "the same promotion on the leader must succeed, or the refusal \
                 above was about the request rather than the node asked",
        );
    bus.settle();
    assert_eq!(
        bus.voters_of(leader),
        vec![0, 1, 2, 3],
        "the leader's promotion committed, so node 3 is a voter"
    );
}

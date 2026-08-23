//! Handing leadership to a named, caught-up voter on request (#3571).
//!
//! # Why the first row spends zero ticks
//!
//! The item exists so a node can be drained without the group waiting out an
//! election timeout. `libs/raft-core/src/lib.rs:45` sets `ELECTION_MIN` to 50
//! ticks and `libs/raft-core/src/lib.rs:263` gives each node
//! `ELECTION_MIN + id`, so "faster than the timeout" is anything under 50. That
//! bound is satisfiable by an implementation that merely shortens the target's
//! timeout and waits — which is the behaviour the group already has, reached by
//! a different route, and `## Must not do` names it.
//!
//! So the row spends **no ticks at all**. A timeout cannot fire in zero ticks,
//! whatever its length, so the only way leadership can move is a message the
//! leader sent saying to campaign now. The bus counts its own ticks and the row
//! asserts the counter is still zero at the moment it reads the leader, which
//! makes this a measured property of the run rather than a claim about how the
//! row is written: a later edit that inserts a tick to make a red row pass turns
//! that assertion red instead.
//!
//! It is also why the row pumps once *before* requesting the transfer and
//! asserts the leader has not moved. Without that half, "the leader is T after
//! pumping" does not distinguish a transfer from a bus that elects on delivery
//! alone.
//!
//! # Why the refusal rows read the target's own log
//!
//! `learner_matched` (`libs/raft-core/src/lib.rs:579`) answers `None` for a
//! voter, so a row has no public accessor for a leader's `match_index` of a
//! voter and cannot pin the refused index against the leader's own bookkeeping.
//! It can pin it against the target: a peer that was reachable through `settle`
//! has acknowledged everything it holds, so the leader's record of it equals
//! that node's `last_index()`. Asserting equality with the target's log — rather
//! than merely "some index below the leader's" — is what stops a refusal that
//! reports a constant `0` from passing.
//!
//! Each refusal row then has a second half that removes the reason and requires
//! the transfer to be accepted. A refusal that is always returned satisfies the
//! first half of every one of them.
//!
//! # Why the two receiving-side rows exist (#3586)
//!
//! Every row above is written from the sender's side: a leader is asked to hand
//! off and either does or refuses. None of them makes a node *receive* the
//! message under a condition it must refuse, so both guards inside
//! `handle_timeout_now` — a learner never campaigns, a stale term is ignored —
//! landed with no row at all. #3571's mutation sweep declared them survivors
//! before it ran and observed both surviving: deleting either guard left the
//! whole gate green.
//!
//! The learner row asserts the term as well as the role. The voter guard sits
//! above the branch that raises the receiver's term, so an implementation that
//! moved it below would still refuse to campaign while quietly bumping the
//! learner's term, and a row reading only the role would not see it.
//!
//! Both rows carry a second half that removes the reason and requires the same
//! message to be obeyed. Without it, a `handle_timeout_now` that returns
//! unconditionally — the message reaching the node and being discarded — passes
//! the first half of each.
//!
//! # Why the abandonment row observes `propose`
//!
//! "The leader stops accepting new proposals once a transfer begins" and "an
//! abandoned handoff must return leadership to the original leader" are two
//! halves of one observable: `propose` answers `None` while the transfer is in
//! flight and `Some` once it has been abandoned. That is the chosen surface for
//! the freeze, and it needs no accessor the crate does not already have. The row
//! does not pin the bound in ticks — it is the implementation's to choose — only
//! that one exists and that the original leader is the one holding the group
//! when it expires.

use std::collections::{HashMap, HashSet};

use raft_core::{Membership, NodeId, RaftMsg, RaftNode, Role, TimeoutNowReq, TransferRefused};

/// Voters 0,1,2 — at least three, so "the named node became leader" is not the
/// same statement as "the only other node became leader".
fn three_voters() -> Membership {
    Membership {
        voters: vec![0, 1, 2],
        learners: vec![],
    }
}

struct Bus {
    nodes: HashMap<NodeId, RaftNode>,
    applied: HashMap<NodeId, Vec<Vec<u8>>>,
    dropped: HashSet<NodeId>,
    /// Every `tick` this bus has issued, to any node. The first row asserts on
    /// it, so no-tick is a measurement rather than a reading of the source.
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

    /// Deliver until quiescent. Issues no ticks, which is what lets the first
    /// row bound the transfer without reference to any timeout.
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

    /// Tick the leader alone and deliver, repeatedly. A peer is first contacted
    /// by a heartbeat, so a bus that only drains outboxes never reaches one.
    fn settle(&mut self) {
        for _ in 0..40 {
            if let Some(leader) = self.leader() {
                self.nodes.get_mut(&leader).unwrap().tick();
                self.ticks += 1;
            }
            self.pump();
        }
    }

    /// The reachable leader. A node that has been dropped keeps `Leader` until
    /// it hears a higher term, which it cannot while it is unreachable, so a
    /// scan that ignores `dropped` reports the deposed leader and masks the one
    /// the survivors elected.
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

/// A settled three-voter group. Returns the bus, the leader, and a caught-up
/// voter that is not the leader — the node every row below names as the target.
fn settled_group() -> (Bus, NodeId, NodeId) {
    let mut bus = Bus::new(&[0, 1, 2], &three_voters());
    let leader = bus.run_until_leader();
    for i in 0..5u8 {
        bus.commit(leader, vec![i]);
    }
    bus.settle();
    let target = *[0, 1, 2]
        .iter()
        .find(|id| **id != leader)
        .expect("a three-voter group has a voter that is not the leader");
    assert_eq!(
        bus.nodes[&target].last_index(),
        bus.nodes[&leader].last_index(),
        "every row starts from a target that has caught up; if it has not, the \
         transfers below are refused for the wrong reason"
    );
    (bus, leader, target)
}

/// The whole item. Leadership moves to the node that was named, and it moves
/// without a single tick, so no election timeout of any length can be what
/// moved it.
#[test]
fn a_transfer_moves_leadership_to_the_named_voter_without_a_single_tick() {
    let (mut bus, leader, target) = settled_group();
    let ticks_before = bus.ticks;

    bus.pump();
    assert_eq!(
        bus.leader(),
        Some(leader),
        "delivering without ticking must not move leadership on its own; if it \
         does, the assertion after the transfer measures the bus and not the \
         handoff"
    );

    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .transfer_leadership(target)
        .expect("a leader hands off to a caught-up voter");
    bus.pump();

    assert_eq!(
        bus.ticks, ticks_before,
        "the transfer must complete on delivery alone; a row that ticks cannot \
         tell a handoff from an election timeout that was shortened"
    );
    assert_eq!(
        bus.leader(),
        Some(target),
        "leadership must arrive at the node that was named, not merely leave the \
         node that gave it up"
    );
    assert!(
        !bus.nodes[&leader].is_leader(),
        "the node that handed off must not still consider itself leader"
    );
}

/// The refusal names the target and the index it is stuck at, so a caller can
/// tell "wait for it" from "never".
#[test]
fn transferring_to_a_voter_that_has_not_caught_up_is_refused_with_its_match_index() {
    let (mut bus, leader, target) = settled_group();
    let stalled_at = bus.nodes[&target].last_index();

    bus.dropped.insert(target);
    for i in 5..10u8 {
        bus.commit(leader, vec![i]);
    }
    bus.settle();

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .transfer_leadership(target)
        .expect_err("a leader refuses to hand off to a voter that is behind");
    match refusal {
        TransferRefused::NotCaughtUp {
            target: named,
            matched,
            last_index,
        } => {
            assert_eq!(named, target, "the refusal must name the target it refused");
            assert_eq!(
                matched, stalled_at,
                "the refused index must be where the target actually stopped, not \
                 a constant; the target acknowledged everything it holds before it \
                 was dropped, so the leader's record of it is that node's own last \
                 index"
            );
            assert_eq!(
                last_index,
                bus.nodes[&leader].last_index(),
                "the refusal must say what the target is behind of"
            );
            assert!(
                matched < last_index,
                "a refusal for being behind must report an index that is behind"
            );
        }
        other => panic!("expected a not-caught-up refusal naming the target, got {other:?}"),
    }

    // Second half: remove the only reason and the same call must be accepted.
    // Without it, an implementation that refuses every transfer passes above.
    bus.dropped.remove(&target);
    bus.settle();
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .transfer_leadership(target)
        .expect("once the target has caught up the same transfer is accepted");
}

/// A learner cannot win an election, so asking it to campaign spends the whole
/// election timeout the transfer exists to avoid.
#[test]
fn transferring_to_a_learner_is_refused_because_a_learner_cannot_win() {
    let (mut bus, leader, _) = settled_group();
    bus.nodes.insert(3, RaftNode::new(3, &three_voters()));
    bus.applied.insert(3, Vec::new());
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .add_learner(3)
        .expect("a leader admits a learner");
    bus.settle();
    assert_eq!(
        bus.nodes[&leader].learner_read_eligible(3),
        Some(true),
        "the learner must be caught up, so the refusal below is about it not \
         being a voter and not about it being behind"
    );

    let refusal = bus
        .nodes
        .get_mut(&leader)
        .unwrap()
        .transfer_leadership(3)
        .expect_err("a leader refuses to hand off to a learner");
    match refusal {
        TransferRefused::NotAVoter { target } => {
            assert_eq!(target, 3, "the refusal must name the target it refused");
        }
        other => panic!("expected a not-a-voter refusal naming the learner, got {other:?}"),
    }
}

/// A node that is not the leader has no leadership to hand off. It is a
/// distinct answer from every other refusal, and giving it the wrong one is the
/// defect #3585 records in the sibling operation.
#[test]
fn transferring_from_a_node_that_is_not_the_leader_is_refused_as_such() {
    let (mut bus, leader, target) = settled_group();
    let other = *[0, 1, 2]
        .iter()
        .find(|id| **id != leader && **id != target)
        .expect("a three-voter group has a third node");

    let refusal = bus
        .nodes
        .get_mut(&target)
        .unwrap()
        .transfer_leadership(other)
        .expect_err("a follower has no leadership to hand off");
    match refusal {
        TransferRefused::NotLeader => {}
        other => panic!(
            "a follower must be told it is not the leader, not that its own \
             target is behind or is not a voter; got {other:?}"
        ),
    }
}

/// A handoff that cannot complete must expire. The leader stops proposing while
/// it is in flight and starts again once it is abandoned, and the group is led
/// by the original leader throughout rather than by nobody.
#[test]
fn a_transfer_that_cannot_complete_is_abandoned_and_the_original_leader_resumes() {
    let (mut bus, leader, target) = settled_group();

    bus.dropped.insert(target);
    bus.nodes
        .get_mut(&leader)
        .unwrap()
        .transfer_leadership(target)
        .expect("the leader accepts a handoff to a voter its records say is caught up");

    assert_eq!(
        bus.nodes.get_mut(&leader).unwrap().propose(vec![90]),
        None,
        "a leader handing off must stop accepting proposals; one that keeps \
         appending moves the target's catch-up target while it chases it"
    );

    let mut resumed = None;
    for _ in 0..500 {
        bus.tick_live();
        bus.pump();
        if let Some(index) = bus.nodes.get_mut(&leader).unwrap().propose(vec![91]) {
            resumed = Some(index);
            break;
        }
    }
    assert!(
        resumed.is_some(),
        "an abandoned handoff must give the leader back its ability to propose; \
         without a bound the group is frozen for as long as the target stays away"
    );

    bus.settle();
    assert_eq!(
        bus.leader(),
        Some(leader),
        "the original leader must be holding the group after the handoff was \
         abandoned, rather than the group being left leaderless"
    );
    assert!(
        bus.applied[&leader].contains(&vec![91]),
        "the resumed leader must be able to commit, not merely to append: a \
         `propose` that returns an index the group never applies is the freeze \
         still in force with its symptom hidden"
    );
}

/// Whether any of the messages a node just produced is a request for votes —
/// the only observable a campaign has before its first reply arrives.
fn campaigned(node: &mut RaftNode) -> bool {
    node.take_outgoing()
        .iter()
        .any(|o| matches!(o.msg, RaftMsg::Vote(_)))
}

/// Receiving side. A learner asked to campaign refuses, and refuses *before*
/// adopting the term it was asked at.
#[test]
fn a_learner_asked_to_campaign_neither_campaigns_nor_takes_the_senders_term() {
    let membership = Membership {
        voters: vec![0, 1, 2],
        learners: vec![3],
    };
    let mut learner = RaftNode::new(3, &membership);
    assert!(
        !learner.is_voter(),
        "the node under test must be a learner of its group, or this row is \
         about a voter and measures nothing"
    );
    let term_before = learner.current_term();
    let sender_term = term_before + 5;

    learner.handle(
        0,
        RaftMsg::TimeoutNow(TimeoutNowReq {
            term: sender_term,
            leader: 0,
        }),
    );

    assert_eq!(
        learner.role(),
        Role::Follower,
        "a learner cannot win an election, so campaigning spends the whole \
         election timeout the handoff exists to avoid"
    );
    assert_eq!(
        learner.current_term(),
        term_before,
        "the guard must sit above the branch that adopts the sender's term; one \
         that sits below it still refuses to campaign while raising the \
         learner's term, and a row reading only the role cannot tell the two \
         apart"
    );
    assert!(
        !campaigned(&mut learner),
        "a refusal that still puts a vote request on the wire has campaigned"
    );

    // Second half: the same message, at the same term, to a voter of the same
    // group. Without it a `handle_timeout_now` that returns unconditionally
    // passes everything above.
    let mut voter = RaftNode::new(0, &membership);
    voter.handle(
        1,
        RaftMsg::TimeoutNow(TimeoutNowReq {
            term: sender_term,
            leader: 1,
        }),
    );
    assert_eq!(
        voter.role(),
        Role::Candidate,
        "a voter given the identical message must campaign, or the refusal \
         above is the node ignoring every handoff rather than the learner guard"
    );
    assert!(
        voter.current_term() > sender_term,
        "a voter that campaigns must stand at a term above the one it was asked \
         at, since it votes for itself in a new term"
    );
}

/// Receiving side. A message from a term the receiver has already left is
/// ignored, so a deposed leader cannot make the group campaign on its behalf.
#[test]
fn a_voter_asked_to_campaign_at_a_stale_term_stays_where_it_is() {
    let (mut bus, leader, target) = settled_group();
    let term = bus.nodes[&target].current_term();
    assert!(
        term > 0,
        "the group has elected a leader, so the receiver's term stands above \
         the floor a stale message has to sit below; at term 0 there is no \
         stale term to send and this row would be vacuous"
    );
    assert_eq!(
        bus.nodes[&target].role(),
        Role::Follower,
        "the receiver must start as a follower, or 'it did not become a \
         candidate' is already true for another reason"
    );

    let node = bus.nodes.get_mut(&target).unwrap();
    node.handle(
        leader,
        RaftMsg::TimeoutNow(TimeoutNowReq {
            term: term - 1,
            leader,
        }),
    );

    assert_eq!(
        node.role(),
        Role::Follower,
        "a message from a term the receiver has already left must not start an \
         election"
    );
    assert_eq!(
        node.current_term(),
        term,
        "an ignored message must leave the receiver's term where it was"
    );
    assert!(
        !campaigned(node),
        "a receiver that ignored the message must not have put a vote request \
         on the wire"
    );

    // Second half: the same message at the receiver's own term is obeyed, so
    // the row above cannot pass by the message being discarded wholesale.
    node.handle(leader, RaftMsg::TimeoutNow(TimeoutNowReq { term, leader }));
    assert_eq!(
        node.role(),
        Role::Candidate,
        "the identical message at a term the receiver has not left must start \
         an election, or the refusal above is not about the term"
    );
}

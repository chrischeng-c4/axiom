# Replication and availability

A tape instance is a Raft group. This area is the README capability
`replicated-availability`: what a group promises about committed messages
when a member dies, restarts, or joins, and how membership and topology
change.

## Replicated group with peer mTLS

- Problem: a publisher that reaches a follower needs an answer it can act
  on, and an operator needs a replication proof that reruns the same way on
  a loaded host. Before tape@0.5.0 a direct publish to a follower was refused
  for its body before its routing, and the failover cases shared one 15 s
  deadline.
- Who: operators; every publisher, through the durability of the answer.
- Promise: three members elect a leader, replicate every append, forward
  writes from followers, and catch a fresh member up by install-snapshot. A
  `kill -9` of the leader loses no committed append. A restarted single
  member recovers its applied floor and does not re-apply a checkpoint on
  cold replay. Peer links are mutually authenticated, and the raft routes are
  never on the public router.
- Non-goals: adding or removing a member without a restart; more than one
  shard.
- Neighbours: none; first section of the area.
- Status rows: `raft-replication`, `raft-peer-mtls`, `leader-failover`.

## Deterministic failover (Milestone #115)

- Problem: a forwarding case refused for its body and a shared deadline
  meant the group's promise was measured by a clock, not by the group's own
  state, and the subscription outcome cannot prove that a lease table
  survives a failover it cannot observe.
- Who: operators, through a proof they can rerun; the subscription epic, as
  its prerequisite.
- Promise: every replication and failover case observes each step through
  its own readiness and leadership surfaces instead of one shared wall-clock
  deadline, and a follower answers a direct publish with 421 and the leader's
  id before the shared runtime's handler judges the body. Twenty consecutive
  single-threaded runs on a loaded host are green.
- Non-goals: any public contract change; this is harness and runtime-adapter
  work.
- Open: none; the ROADMAP boundary is complete.
- Neighbours: repairs Replicated group with peer mTLS; prerequisite for
  [subscriptions.md](subscriptions.md) § Subscription ack and competing
  subscribers.
- Outcome: `deterministic-failover`. Tracking: [Milestone #115](https://github.com/chrischeng-c4/axiom/milestone/115)

## Live replica membership (Milestone #124)

- Problem: a replica-count change on the custom resource restarts members
  instead of adding a learner live.
- Who: operators scaling a running instance.
- Promise: a replica-count change adds a learner, promotes it once caught
  up, or removes a voter, without restarting the surviving members, while a
  publisher keeps appending and no committed message is lost.
- Non-goals: shard count; the shared runtime's learner and promotion calls
  already exist and are not redesigned here.
- Open: none; the ROADMAP boundary is complete.
- Neighbours: extends Replicated group with peer mTLS and
  [operations.md](operations.md) § Kubernetes operator and direct install.
- Outcome: `live-replica-membership`. Tracking: [Milestone #124](https://github.com/chrischeng-c4/axiom/milestone/124)

## Multi-shard topology (Milestone #125)

- Problem: `shardCount` is fixed at 1, so one group carries every topic.
- Who: operators running more topics or throughput than one group holds.
- Promise: a `Tape` custom resource can declare more than one shard, topics
  are placed across shards, and ordering keys keep their promise within a
  key across a shard split. Shards never appear in the public API.
- Non-goals: cross-shard transactions; a public partition API.
- Open: the placement rule for topics across shards and whether it is stable
  under a shard-count change.
- Neighbours: extends Live replica membership; carries
  [subscriptions.md](subscriptions.md) § Ordering keys across shards.
- Outcome: `multi-shard-topology`. Tracking: [Milestone #125](https://github.com/chrischeng-c4/axiom/milestone/125)

## Non-goals in this area

- None beyond the inherited boundaries; availability makes no claim against
  another broker (`peer-broker-benchmarks`).

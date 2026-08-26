# Replication and availability

A tape instance is a Raft group. This area is the README capability
`replicated-availability`: what a group promises about committed messages
when a member dies, restarts, or joins, and how membership and topology
change.

## Replicated group with peer mTLS

- Problem: one of the three `raft_cluster` cases is red today: a
  follower-forwarded append answers 415 where 421 is expected since the
  shared runtime's publish handler moved to a JSON extractor. The failover
  cases share one 15 s deadline, so the proof is timing-sensitive on a
  loaded host.
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

## Deterministic failover

- Problem: the red forwarding case and the shared deadline above mean the
  group's promise is measured by a clock, not by the group's own state, and
  the subscription outcome cannot prove that a lease table survives a
  failover it cannot observe.
- Who: operators, through a proof they can rerun; the subscription epic, as
  its prerequisite.
- Promise: every replication and failover case observes each step through
  its own readiness and leadership surfaces instead of one shared wall-clock
  deadline, and the follower-forwarded append passes against the shared
  runtime's JSON publish handler. Twenty consecutive single-threaded runs on
  a loaded host are green.
- Non-goals: any public contract change; this is harness and runtime-adapter
  work.
- Open: none; the ROADMAP boundary is complete.
- Neighbours: repairs Replicated group with peer mTLS; prerequisite for
  [subscriptions.md](subscriptions.md) § Subscription ack and competing
  subscribers.
- Outcome: `deterministic-failover`. Tracking: not assigned.

## Live replica membership

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
- Outcome: `live-replica-membership`. Tracking: not assigned.

## Multi-shard topology

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
- Outcome: `multi-shard-topology`. Tracking: not assigned.

## Non-goals in this area

- None beyond the inherited boundaries; availability makes no claim against
  another broker (`peer-broker-benchmarks`).

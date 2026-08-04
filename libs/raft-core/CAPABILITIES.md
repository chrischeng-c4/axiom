# Raft Core Capabilities

## Brief

`raft-core` is the consensus state machine and nothing around it. It owns terms,
votes, the replicated log, the commit rule, and log compaction. It owns no
clock, no threads, no sockets, and no storage: a driver ticks it, hands it
messages, drains the messages it wants sent, and applies the entries it declares
committed.

That separation is the point. Every property this crate promises is reachable by
feeding one deterministic sequence of ticks and messages into a set of in-process
nodes, so a failure mode that would take a flaky multi-second integration test
elsewhere is a handful of exact calls here.

It does not own leader-lease timing policy, transport, retries, persistence
durability, or the state machine the committed entries are applied to. It
decides which entries are committed, in which order, and under whose leadership.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what `raft-core` fundamentally does: elect at most one
  leader per term, and replicate a single agreed log prefix to a majority.
- **Non-Core Features** keep a long-running group bounded — a log that grows
  forever is a correctness problem deferred, not avoided. Non-core does not mean
  optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Election Safety | - | implemented | verified | smoke | ready | core; a term elects at most one leader, because a voter spends its single vote once and only on a candidate whose log is at least as current as its own |
| Log Replication Consistency | - | implemented | verified | smoke | ready | core; an entry is committed only once a majority of voters holds it and it belongs to the current term, and a follower's divergent suffix is truncated rather than merged |
| Snapshot Compaction | - | implemented | verified | smoke | ready | non-core; a compacted prefix stops costing memory without changing any index, and a follower whose next entry has been compacted away is caught up by snapshot instead of by a request that can never be satisfied |

### Core Features

#### Election Safety

ID: election-safety
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
At most one node can be leader in any term. A voter holds exactly one vote per
term and never changes it, so two candidates cannot both collect a majority of
the same voter set. A vote is granted only to a candidate whose last log entry is
at least as up to date as the voter's own — a higher last term wins, and at equal
last term the longer log wins — so a node missing committed entries can campaign
but cannot win. Any node that observes a term higher than its own steps down to
follower and releases its vote for that new term. The voter set is always odd, so
a majority is unambiguous, and a node outside it is a learner: it replicates, it
never votes, and it never starts an election.
Surfaces:
- Rust API: `raft_core::RaftNode::tick` - advance one logical tick; a voter whose election timer expires campaigns, a leader heartbeats.
- Rust API: `raft_core::RaftNode::handle` - feed one inbound message, including `Vote` and `VoteResp`.
- Rust API: `raft_core::RaftNode::role` / `is_leader` / `is_voter` / `current_term` / `leader` - observe the outcome of an election.
- Rust API: `raft_core::auto_membership` - derive an odd voter set plus learners for a group of `n` nodes.
- Rust API: `raft_core::PersistedState` / `RaftNode::persisted` / `from_persisted` - the term and vote a restart must not forget.
Rust internal: the up-to-date comparison against the local last index and term, the vote tally restricted to the voter set, and the step-down that clears the vote on a term bump.
EC Dimensions:
- behavior: `cargo test -p raft-core --test consensus` - a lone voter wins immediately, a majority of grants promotes a candidate, a repeated request from the same candidate in the same term is idempotent, and a learner never campaigns.
- security: `cargo test -p raft-core --test consensus` - a second candidate in the same term is refused, a candidate with a shorter or staler log is refused even when the voter has not yet voted, and a node that sees a higher term in any message drops leadership before acting on it.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Single vote per term | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; a voter records the candidate it voted for and grants only to that same candidate for the rest of the term, so two candidates cannot both reach a majority and a retried request costs nothing |
| Up-to-date log requirement | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; a grant requires the candidate's last log term to exceed the voter's, or to equal it with a last index no smaller, so a node that is behind cannot be elected and erase a committed entry |
| Term monotonicity and step-down | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; observing a higher term anywhere — request or response — adopts that term, clears the vote, and demotes to follower before the message is acted on, so a stale leader cannot keep committing |
| Odd voter set with learners | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; membership derivation keeps the voter count odd for any group size, and the trailing even node becomes a learner that replicates without voting or campaigning, so a majority is always a strict one |

#### Log Replication Consistency

ID: log-replication-consistency
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
Two nodes that agree on the entry at a given index agree on every entry before
it. A follower accepts an append only when the entry preceding it matches in both
index and term; when it does not, the append is refused with a backoff hint and
the leader retreats until it finds the agreed prefix. A follower that holds a
conflicting entry truncates it and everything after it rather than merging. An
entry is committed only when a majority of voters holds it and it was created in
the current term, so an entry replicated by a previous leader is never committed
on its own count alone. Replication progress per peer only ever moves forward, so
responses that arrive out of order cannot walk a peer's match index backwards.
Committed entries are handed to the driver exactly once, in index order.
Surfaces:
- Rust API: `raft_core::RaftNode::propose` - append a command on the leader and return its index, or `None` when not leader.
- Rust API: `raft_core::RaftNode::take_outgoing` - drain the messages the driver must deliver.
- Rust API: `raft_core::RaftNode::take_committed` - drain newly committed entries in index order.
- Rust API: `raft_core::RaftNode::commit_index` / `last_index` / `log_len` - observe replication progress.
- Rust API: `raft_core::AppendReq` / `AppendResp` / `RaftEntry` / `RaftMsg` / `Outgoing` - the replication wire shapes.
Rust internal: the previous-entry match check, the conflicting-suffix truncation, the majority scan restricted to current-term entries, and the monotonic clamp on per-peer match and next indices.
EC Dimensions:
- behavior: `cargo test -p raft-core --test consensus` - a proposal on the leader reaches a majority and is reported committed once, in index order, on every node that holds it; a follower that has fallen behind is caught up by backoff and retry; a non-leader refuses to propose.
- security: `cargo test -p raft-core --test consensus` - an append whose preceding entry disagrees is refused rather than applied, a follower's divergent suffix is discarded before the leader's entries are accepted, an entry from an earlier term is not committed by majority count alone, and a delayed response never reduces a peer's recorded progress.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Previous-entry match check | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; an append is accepted only when the entry before it agrees in index and term, and anything at or below the compaction point counts as agreed, so a follower never grafts entries onto a prefix it does not share |
| Conflicting suffix truncation | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; a resident entry whose term differs from the leader's entry at the same index is dropped along with everything after it, so the follower's log converges on the leader's rather than interleaving two histories |
| Current-term majority commit | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; the commit index advances only to an index held by a majority of voters whose entry belongs to the current term, so an entry left behind by a previous leader is committed only once a current-term entry above it is |
| Monotonic replication progress | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; per-peer match and next indices only increase on success, and a failure response for a prefix a newer response already proved replicated is discarded, so out-of-order delivery cannot un-replicate an entry |
| Exactly-once committed delivery | change | - | implemented | verified | smoke | `cargo test -p raft-core --test consensus`; draining committed entries advances the applied cursor, so every entry is handed to the driver exactly once and in index order, and a second drain with no new commits yields nothing |

### Non-Core Features

#### Snapshot Compaction

ID: snapshot-compaction
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A log that has been applied can stop costing memory without any index changing
meaning. Compaction drops the applied prefix, records the index and term it
stopped at, and keeps the caller's snapshot bytes; the highest log index is the
same before and after, so no committed entry is renumbered. Compaction refuses to
run past what has actually been applied, and refuses to move backwards. A leader
that needs to send a follower an entry which has been compacted away ships the
snapshot instead of an append that could never be satisfied. A follower that
receives a strictly newer snapshot adopts it, drops its resident log, advances
its commit and applied cursors to the snapshot point, and surfaces the bytes to
its driver exactly once; an older or equal snapshot changes nothing.
Surfaces:
- Rust API: `raft_core::RaftNode::compact` - drop the applied prefix up to an index and record the caller's snapshot bytes.
- Rust API: `raft_core::RaftNode::snapshot_index` / `log_len` / `last_index` - observe what compaction did and did not change.
- Rust API: `raft_core::RaftNode::take_installed_snapshot` - collect a leader-supplied snapshot for the state machine to load.
- Rust API: `raft_core::InstallSnapshotReq` / `InstallSnapshotResp` - the snapshot-shipping wire shapes.
Rust internal: the index arithmetic that maps a log index to a position after compaction, and the choice between an append and a snapshot when a peer's next index falls to the compaction point.
EC Dimensions:
- behavior: `cargo test -p raft-core --test snapshot` - compaction reduces the resident log while leaving the highest index unchanged; a follower that is behind the compaction point is caught up by snapshot and then resumes normal appends; an installed snapshot is surfaced once and then no longer.
- security: `cargo test -p raft-core --test snapshot` - compaction past the applied cursor or backwards is refused outright, an older or equal snapshot does not truncate a follower that is ahead, and a snapshot from a stale term is answered without being installed.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bounded compaction | change | - | implemented | verified | smoke | `cargo test -p raft-core --test snapshot`; compaction runs only strictly above the current snapshot point and no further than the applied cursor, and it leaves the highest log index unchanged, so memory falls without any index being renumbered |
| Snapshot ship on compacted next index | change | - | implemented | verified | smoke | `cargo test -p raft-core --test snapshot`; when a peer's next index falls at or below the compaction point the leader sends the snapshot rather than an append, so a follower that fell too far behind is caught up instead of retried forever |
| Install supersedes the resident log | change | - | implemented | verified | smoke | `cargo test -p raft-core --test snapshot`; a strictly newer snapshot clears the resident log, adopts the snapshot index and term, advances the commit and applied cursors to it, and surfaces the bytes exactly once, while an older or equal snapshot leaves the node untouched |

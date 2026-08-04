# Raft Runtime Capabilities

## Brief

`raft-runtime` is the host around a Raft group, not the consensus algorithm.
`raft-core` owns elections, log replication and commitment. What a service
building on top of it then needs — and would otherwise rebuild once per
service — is everything between "a command was committed" and "a request can
be answered": the tick and pump loops, the peer router and its mutually
authenticated transport, the durable hard-state store, the applied-index
floor that survives a restart, and a `propose` call that returns only after
the caller's own write is visible locally.

This crate owns those answers. A service supplies a `RaftStateMachine` and
its command encoding; `raft-runtime` supplies the host that drives it,
routes a proposal to whichever replica is currently leader, replays a
committed log into a fresh state machine on restart, derives the cluster's
own topology from StatefulSet downward-API environment, and hands out
epoch-fenced assignments for state machines that authorise external effects.

It does not own command encoding, snapshot byte formats, or API-level
routing, and it does not execute the effects a fence authorises.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** are what a service depends on to be a replicated
  service at all: a deterministic state machine driven exactly once in
  index order, a proposal that reaches the leader, durable state that
  survives a restart, and knowing which replica this process is.
- **Non-Core Features** keep that contract honest under retry, restart,
  rotation and misconfiguration — idempotent proposal outcomes, a read
  consistency contract, hot-reloadable peer mTLS, a static membership
  guard, and bounded log growth. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Fenced Single-Writer Assignment | 3378 | implemented | verified | smoke | ready | core; an owner holds an assignment under a monotonic epoch, and every late or superseded holder is rejected by the same four-step validation in a fixed order |
| Deterministic Replicated State Machine | 3378 | implemented | verified | smoke | ready | core; commands are applied exactly once in index order, and the same command sequence produces the same replica state on every node and across a restart |
| Leader-Routed Proposal | 3378 | implemented | verified | smoke | ready | core; a proposal submitted to any replica reaches the current leader within a bounded budget, and returns only after the caller's own write is applied locally |
| StatefulSet Cluster Topology | 3378 | implemented | verified | smoke | ready | core; shard index, replica index, voter status and peer URLs are derived from the pod's own name and the downward-API environment, never from the binary's name |
| Idempotent Proposal Outcomes | 3378 | implemented | verified | smoke | ready | non-core; a retried proposal observes the first recorded outcome, and retention is bounded by capacity and by applied index so the cache cannot grow without limit |
| Read Consistency Header Contract | 3378 | implemented | verified | smoke | ready | non-core; one header selects leader, bounded-staleness or any-replica reads, and an absent or unparseable value falls back to the strongest mode rather than the cheapest |
| Hot-Reloadable Peer mTLS | 3378 | implemented | verified | smoke | ready | non-core; peer credentials rotate without a restart, a failed reload keeps the last known good transport, and an untrusted or mismatched peer fails before HTTP dispatch |
| Static Membership Guard | 3378 | implemented | verified | smoke | ready | non-core; a replica-count change that consensus cannot safely absorb is refused at configuration time instead of producing a split group |
| Snapshot Policy and Log Compaction | 3378 | implemented | verified | smoke | ready | non-core; a declarative policy decides when the host snapshots and compacts, and a host that has applied nothing compacts nothing |

### Core Features

#### Fenced Single-Writer Assignment

ID: fenced-single-writer-assignment
Root WI: 3378
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A service that authorises an external effect needs exactly one executor to
believe it holds the right to act. `FencedAssignment` provides that as
replicated state: an assignment binds an owner to a `FenceToken` carrying a
monotonic `AssignmentEpoch`, together with an absolute expiry supplied by the
proposer rather than read from a local clock. The first assignment allocates
epoch 1; every later assignment allocates the next epoch, and releasing an
assignment retains the epoch so a released-then-reassigned owner can never
replay the epoch it previously held.

`validate` answers a single question — may this caller act right now — and
answers it in a fixed order: unassigned first, then a stale epoch, then an
owner mismatch, then expiry. The order is the contract, because a caller
holding a superseded epoch must be told its epoch is stale rather than told
the assignment expired; those two conditions demand different recovery. An
assignment is expired at exactly the instant `now_ms` reaches `expires_at_ms`,
`assign` refuses an expiry that is already in the past at proposal time, and
`renew` requires both the current owner's epoch and a strictly later expiry,
so renewal can never shorten a fence.

Because every transition takes its time from the proposal rather than from
the machine executing it, two replicas applying the same command sequence
reach the same assignment state.

Surfaces:
- `FencedAssignment::{assign, validate, renew, release, expire}`
- `FenceToken`, `AssignmentEpoch`, `ActiveAssignment`, `AssignmentError`

Rust internal:
- `libs/raft-runtime/src/fenced_assignment.rs`

EC Dimensions:
- behavior: the epoch is monotonic across assign/release/assign; `validate`
  reports stale epoch before owner mismatch and owner mismatch before expiry;
  `renew` rejects an equal or earlier expiry.
- security: a released or superseded owner cannot act; a fence cannot be
  extended by a non-owner; expiry is evaluated against proposer-supplied time
  so a replica with a skewed clock cannot grant itself a longer window.

| Gate | Evidence |
|---|---|
| Exclusivity and epoch retention | `cargo test -p raft-runtime --lib fenced_assignment` — `no_token_exists_before_assignment`, `assignment_is_exclusive_until_explicit_release_or_expiry`, `release_retains_epoch_and_fences_late_completion`, `stale_owner_is_rejected_after_reassignment`, `renewal_requires_current_owner_epoch_and_later_expiry` |
| Replica determinism | `cargo test -p raft-runtime --lib fenced_assignment::tests::identical_commands_produce_identical_replica_state` |
| Executor-level fencing | `cargo test -p raft-runtime --test fenced_assignment` — `executor_cannot_act_before_assignment_commit`, `expiry_and_reassignment_fence_the_previous_executor`, `proposer_supplied_time_keeps_replica_transitions_deterministic` |

#### Deterministic Replicated State Machine

ID: deterministic-replicated-state-machine
Root WI: 3378
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A service implements `RaftStateMachine` over an opaque `Command` and gets one
guarantee in return: `apply` is called exactly once per committed index, in
ascending index order, on every replica. The state machine reports its own
`applied_index`, and the host uses that number — not a local guess — to decide
what still needs replaying. On restart the committed log is replayed into a
fresh state machine from the applied floor, so a process that dies between
commit and apply loses nothing.

An `apply` that returns an error is logged and still counted as applied. That
is deliberate: a state machine cannot decline a committed entry without
diverging from its peers, so the only safe treatment of an application-level
failure is to record it and advance. Failures that must be visible to the
caller belong in the applied outcome, not in a refusal to apply.

The applied floor is durable in its own right. `AppliedIndexStore` reads a
missing file as index 0, so a first boot is indistinguishable from a boot with
nothing applied, and refuses to interpret a corrupt file at all rather than
guessing a floor that would silently skip entries.

Surfaces:
- `RaftStateMachine::{apply, snapshot, restore, applied_index}`, `Command`
- `AppliedIndexStore::{new, path, load, store}`
- `RaftStore::{open, save, load, seed_snapshot}`, `FsyncPolicy`

Rust internal:
- `libs/raft-runtime/src/state_machine.rs`
- `libs/raft-runtime/src/applied_index_store.rs`
- `libs/raft-runtime/src/store.rs`

EC Dimensions:
- behavior: a restart replays committed entries into a fresh state machine
  and reproduces the pre-restart state; a missing applied-index file reads as
  0 and a stored floor round-trips.
- security: a corrupt applied-index file is an error rather than a silently
  lowered floor; `seed_snapshot` refuses to overwrite an existing snapshot
  path, so seeding cannot destroy committed state.

| Gate | Evidence |
|---|---|
| Restart replay | `cargo test -p raft-runtime --lib tests::restart_replays_committed_log_into_a_fresh_sm` |
| Applied floor durability | `cargo test -p raft-runtime --lib applied_index_store::tests::missing_is_zero_and_stored_floor_round_trips` |

#### Leader-Routed Proposal

ID: leader-routed-proposal
Root WI: 3378
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A caller should not have to know which replica is leader. `RaftHost::propose`
accepts a command on any replica and resolves one of three situations: this
node is the leader and applies locally; another node is the leader and the
command is forwarded to its `/raft/publish` endpoint under the same propose
deadline; or no leader is currently known, in which case the host retries on
a short fixed interval until the propose timeout expires rather than failing
on the first miss during an election.

In every case the call returns only after the command is applied *locally*.
A forwarded proposal that the leader has already committed is not answered
until this node's own state machine has caught up, which is what makes a
read immediately after a write on the same connection see that write. The
propose deadline is a single budget covering discovery, forwarding and local
apply, so a caller's timeout means the same thing regardless of which of the
three paths was taken.

The peer router is the other half of the surface: `/raft/request-vote`,
`/raft/append-entries` and `/raft/install-snapshot` carry consensus,
`/raft/publish` carries forwarded proposals, and `/raftz` exposes the
cluster state view. Shutdown aborts the tick and pump loops and then drains
in-flight peer RPCs within a bounded window rather than dropping them.

Surfaces:
- `RaftHost::{spawn, spawn_with_peer_transport, propose, is_leader, leader,
  applied_watch, router, shutdown}`
- `ClusterStateView`, `PeerAddr`, `RaftRole`
- `HostConfig` (tick, pump, rpc timeout, propose timeout)

Rust internal:
- `libs/raft-runtime/src/host.rs`
- `libs/raft-runtime/src/view.rs`
- `libs/raft-runtime/src/config.rs`

EC Dimensions:
- behavior: a proposal returns only after local apply, so a read-your-write
  immediately following the call observes the written value; an unknown
  leader is retried within the propose budget rather than failing fast.
- security: forwarding targets are drawn from the derived cluster topology
  and reached over the peer transport, never from a caller-supplied address.

| Gate | Evidence |
|---|---|
| Read-your-write on propose | `cargo test -p raft-runtime --lib tests::single_node_propose_applies_read_your_write` |
| Peer router reachable over mutual TLS | `cargo test -p raft-runtime --test peer_mtls trusted_mutual_peers_reach_the_http2_router` |

#### StatefulSet Cluster Topology

ID: statefulset-cluster-topology
Root WI: 3378
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A replica must know which replica it is before it can do anything else, and
the only trustworthy source for that is the pod's own identity. `ClusterDims`
takes `POD_NAME` and splits it at its last hyphen into a StatefulSet name and
an ordinal, then derives `shard_index = ordinal % shard_count` and
`replica_index = ordinal / shard_count`, with `is_voter` true exactly when
`replica_index < voter_count`. `peer_ordinal` is the exact inverse:
`replica * shard_count + shard_index`.

Peer DNS names are built from the StatefulSet prefix taken from `POD_NAME`,
not from the running binary's name. Those two are frequently the same string
in development and reliably different in production, which is exactly the
class of bug that only appears after deployment; the caller's fallback prefix
is used only when the pod name is unavailable or malformed. A pod name with
no hyphen, or an empty prefix, is rejected rather than silently producing a
nameless peer DNS suffix.

Replica mode is automatic: `REPLICAS_PER_SHARD > 1` means cluster mode, and
an unset environment means single-node. `ClusterTopology::from_env_with_scheme`
accepts only `http` and `https`, and validates that `SHARD_COUNT` and
`REPLICAS_PER_SHARD` are positive, that `1 <= VOTER_COUNT <= REPLICAS_PER_SHARD`,
and that this node's id is within the replica range — a misconfigured group
fails at startup rather than after it has already accepted writes.

Surfaces:
- `ClusterDims`, `ClusterTopology::from_env_with_scheme`
- `replica_mode`, `peer_ordinal`, `parse_peer_overrides`

Rust internal:
- `libs/raft-runtime/src/cluster.rs`

EC Dimensions:
- behavior: ordinal arithmetic round-trips through `peer_ordinal`; the peer
  DNS prefix follows the pod name and not the binary name; peer overrides are
  split, trimmed and emptied-filtered.
- security: an out-of-range node id, a non-positive shard or replica count, a
  voter count outside `1..=replicas`, an unknown URL scheme, and a nameless
  pod are each rejected at construction.

| Gate | Evidence |
|---|---|
| Ordinal derivation and inverse | `cargo test -p raft-runtime --lib cluster::tests::cluster_dims_derives_shard_and_replica_from_pod_ordinal`, `cluster::tests::peer_ordinal_matches_replica_times_shard_count_plus_shard` |
| Pod-derived peer prefix | `cargo test -p raft-runtime --lib cluster::tests::peer_dns_prefix_follows_the_pod_not_the_callers_binary_name`, `cluster::tests::pod_prefix_is_the_statefulset_name_and_rejects_a_nameless_pod` |
| Environment validation and defaults | `cargo test -p raft-runtime --lib cluster::tests::topology_from_env_with_local_override`, `cluster::tests::replica_mode_defaults_to_single_node`, `cluster::tests::cluster_dims_pod_ordinal_rejects_bad_suffix`, `cluster::tests::parse_peer_overrides_splits_trims_and_filters_empty` |

### Non-Core Features

#### Idempotent Proposal Outcomes

ID: idempotent-proposal-outcomes
Root WI: 3378
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A client that retries a proposal after a timeout must not be charged twice.
`ProposalCache` keys outcomes by proposal identity and keeps the **first**
recorded outcome for a key; a later insert under the same key does not
overwrite it. That direction is the whole point — the first outcome is the
one the state machine actually produced, and a retry that recorded a second
outcome would report a different answer for the same logical operation.

The cache is bounded at `DEFAULT_PROPOSAL_CACHE_CAPACITY` entries and evicts
in insertion order, and it participates in snapshot and restore so a restored
replica does not forget which proposals it has already answered.

`OutcomeWindow` bounds the same problem by applied index rather than by
count: `advance(index)` evicts entries *strictly below*
`index.saturating_sub(capacity)`, so the boundary index itself survives. An
off-by-one in that comparison would drop the oldest outcome a client may
still legitimately retry, which is why the boundary is pinned by its own
test rather than left implied by the eviction test.

Surfaces:
- `ProposalCache`, `DEFAULT_PROPOSAL_CACHE_CAPACITY`
- `OutcomeWindow`, `OUTCOME_WINDOW_DEFAULT_CAPACITY`

Rust internal:
- `libs/raft-runtime/src/proposal_cache.rs`
- `libs/raft-runtime/src/outcome_window.rs`

EC Dimensions:
- behavior: a repeated insert observes the first outcome; snapshot/restore
  preserves insertion order; `advance` keeps the entry exactly at the cutoff
  and drops everything below it.
- security: retention is bounded by capacity and by applied index, so an
  unbounded retry stream cannot grow host memory without limit.

| Gate | Evidence |
|---|---|
| First-outcome retention across restore | `cargo test -p raft-runtime --lib proposal_cache::tests::retains_first_outcome_and_restores_in_order` |
| Applied-index-bounded retention | `cargo test -p raft-runtime --lib outcome_window` — `insert_then_claim_round_trips`, `advance_evicts_entries_below_the_cutoff`, `advance_evicts_strictly_below_cutoff_inclusive_boundary_survives`, `claim_after_evict_returns_none`, `default_uses_the_documented_capacity` |

#### Read Consistency Header Contract

ID: read-consistency-header-contract
Root WI: 3378
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
One header, `x-read-consistency`, lets a caller choose what a read costs:
`leader` for a linearizable read, `bounded(<ms>)` to accept a replica whose
applied state is no more than that many milliseconds stale, and `any` to read
from whichever replica answers. The value is trimmed and lowercased before
parsing, so header casing and incidental whitespace are not a correctness
surface.

An absent header, an unparseable value, or an unrecognised mode falls back to
`Leader`. The fallback direction is deliberate and is the security property
here: a typo in a client's header must make a read stricter and slower, never
looser. A default of `any` would turn every client mistake into a silent
stale read that no test and no alert would catch.

Surfaces:
- `ReadConsistency::{Leader, Bounded, Any}`, `ReadConsistency::from_header`
- `READ_CONSISTENCY_HEADER`

Rust internal:
- `libs/raft-runtime/src/read_consistency.rs`

EC Dimensions:
- behavior: `leader`, `bounded(<ms>)` and `any` parse; surrounding whitespace
  and upper case parse identically.
- security: `None`, an empty value, an unknown mode and a malformed
  `bounded(...)` all resolve to `Leader`, never to a weaker mode.

| Gate | Evidence |
|---|---|
| Header parsing and strict fallback | `cargo test -p raft-runtime --lib read_consistency::tests::read_consistency_from_header` |

#### Hot-Reloadable Peer mTLS

ID: hot-reloadable-peer-mtls
Root WI: 3378
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Peer-to-peer consensus traffic is mutually authenticated, and the credentials
behind it rotate while the host is running. `PeerTransport::reload` builds and
validates a complete new snapshot before publishing it atomically; if the new
material is unusable the previously published snapshot stays in service and
the reload reports the failure. A rotation that produces a bad bundle
therefore degrades to "still running on the old certificate", not to a
cluster that has lost its peer transport.

`generation` starts at 1 and advances on each successful publish, which gives
an operator a way to confirm that a rotation actually took effect rather than
inferring it from the absence of an error. When peer TLS is not required the
transport builds no snapshot at all, so an unconfigured single-node
deployment pays nothing.

Authentication failures fail closed and fail early: an untrusted client, an
expired server certificate, and a hostname that does not match the presented
certificate are all rejected before any HTTP request is dispatched, so a
rejected peer never reaches the router.

Surfaces:
- `PeerTransport::{from_config, generation, reload, http_client, connect,
  accept, serve}`

Rust internal:
- `libs/raft-runtime/src/peer_transport.rs`

EC Dimensions:
- behavior: a successful reload advances the generation and serves the new
  material; a failed reload preserves the last known good transport.
- security: untrusted client certificates, expired server certificates and
  hostname mismatches are rejected before HTTP dispatch.

| Gate | Evidence |
|---|---|
| Mutual trust reaches the router | `cargo test -p raft-runtime --test peer_mtls trusted_mutual_peers_reach_the_http2_router` |
| Fail-closed rejection paths | `cargo test -p raft-runtime --test peer_mtls` — `hostname_mismatch_fails_before_http_dispatch`, `untrusted_client_and_expired_server_fail_closed` |
| Atomic rotation with last-known-good | `cargo test -p raft-runtime --test peer_mtls reload_is_atomic_and_preserves_last_known_good_on_error` |

#### Static Membership Guard

ID: static-membership-guard
Root WI: 3378
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
This host runs static Raft membership: the voter set is derived from the
declared topology and does not change through joint consensus at runtime.
`ensure_static_membership_unchanged` makes that assumption explicit by
comparing the desired replica count against the current one and refusing a
delta.

The refusal is the feature. A scaled StatefulSet under static membership
produces replicas that believe in different voter sets, and the failure mode
is a group that elects two leaders rather than a group that reports an error.
Turning that into a configuration-time rejection means the mistake is visible
in a rollout that fails to start, not in divergent committed state discovered
later.

Surfaces:
- `ensure_static_membership_unchanged`

Rust internal:
- `libs/raft-runtime/src/cluster.rs`

EC Dimensions:
- behavior: an unchanged replica count is accepted; any delta is refused.
- security: the refusal happens at configuration time, before the host can
  accept writes under an inconsistent voter set.

| Gate | Evidence |
|---|---|
| Replica-count delta refused | `cargo test -p raft-runtime --lib cluster::tests::static_membership_rejects_replica_delta` |

#### Snapshot Policy and Log Compaction

ID: snapshot-policy-and-log-compaction
Root WI: 3378
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A replicated log that is never compacted eventually costs more to replay than
the state it represents. `SnapshotPolicy` states when compaction happens as
configuration rather than as code: `Disabled` never snapshots, `EveryEntries(n)`
snapshots on an applied-entry interval, and `External` hands the decision to
whatever component already knows the right moment — typically the backup
runner that is about to read a snapshot anyway.

`Disabled` is the default. A host that has not been told a policy does not
quietly start rewriting its own log, and `snapshot_and_compact` on a host
whose applied index is 0 compacts nothing and reports zero rather than
producing an empty snapshot that a later restore would have to special-case.

`RaftStore` keeps the durable hard state behind this, and deduplicates a
save against the last saved value so an unchanged term and vote does not
turn every tick into a disk write.

Surfaces:
- `SnapshotPolicy::{Disabled, EveryEntries, External}`, `HostConfig`
- `RaftHost::snapshot_and_compact`
- `RaftStore::{save, load, seed_snapshot}`, `FsyncPolicy`

Rust internal:
- `libs/raft-runtime/src/config.rs`
- `libs/raft-runtime/src/host.rs`
- `libs/raft-runtime/src/store.rs`

EC Dimensions:
- behavior: the default policy is `Disabled`; a host with applied index 0
  compacts nothing; an unchanged hard state is not re-written.
- security: `seed_snapshot` refuses a path that already exists, so seeding a
  node cannot overwrite an existing snapshot.

| Gate | Evidence |
|---|---|
| Policy defaults and host wiring | `cargo test -p raft-runtime --lib` (26 unit tests, includes the `HostConfig` default policy) |
| Restart path exercises store load and replay | `cargo test -p raft-runtime --lib tests::restart_replays_committed_log_into_a_fresh_sm` |

## Not Promised Here

- **Consensus itself.** Elections, log replication, commit index advancement
  and joint-consensus mechanics belong to `raft-core`. This crate drives that
  engine; it does not reimplement it.
- **Command encoding and snapshot formats.** `Command` is `Vec<u8>` and the
  snapshot bytes are whatever the service's `RaftStateMachine` produces.
  Versioning, compatibility and validation of those bytes are the service's.
- **Executing fenced effects.** `FencedAssignment` decides who may act and
  until when. It never performs the external effect, never retries one, and
  does not own the application-level assignment key.
- **`tests/behavior_shared_raft_runtime_driver_contract.rs`.** This file is
  the AW external-contract harness stub and is `#[ignore]`d in a normal
  `cargo test` run. It is not a gate for any capability above and must not be
  cited as evidence; every gate in this document names a test that runs and
  passes in the default suite.
- **Scheduling and orchestration.** When a node is created, scaled, drained
  or replaced is the operator's decision. This crate only refuses the
  membership changes it cannot absorb.
- **Clocks.** Every time-dependent transition takes an explicit
  proposer-supplied instant. This crate reads no wall clock for replicated
  decisions, and a caller that passes a locally read clock into a replicated
  command has moved the determinism problem, not solved it.

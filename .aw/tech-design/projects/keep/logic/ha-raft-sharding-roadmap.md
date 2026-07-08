---
id: keep-ha-raft-sharding-roadmap
summary: Keep HA sharding and raft rollout design, preserving the phase plan behind capability keep:ha-raft and keep:primary-replicas.
fill_sections: [logic, e2e-test]
---

# TD: Keep HA / Raft Sharding Roadmap (#121)

HA lands in phases. **Phase A is implemented**; Phase C's raft-host substrate
and HTTP write propose path are implemented behind the `raft` feature; the
remaining staged work is failover/read-consistency hardening. The phases share
one substrate: node identity + the keyspace split (`src/cluster.rs`).

## Phase A — sharded scale-out (DONE)

`src/cluster.rs` + `GET /cluster`. The keyspace is `shard_count` virtual shards
mapped onto `node_count` nodes (`keep-0..keep-N`, a StatefulSet). A client routes:

```
shard = crc32(key) % shard_count
node  = shard % node_count          # owner_of_shard
```

Each node owns a disjoint shard subset (proven in `cluster::tests`); a node is
independent (its own WAL + durability). `GET /cluster` reports
`{node_id, node_count, shard_count, owned_shards, peers, mode}` so a client (or a
thin proxy) can self-route — keep ships no client, so routing is a documented
contract, same as the connection-count guidance.

Config: `KEEP_NODE_ID` (pod ordinal), `KEEP_NODE_COUNT`, `KEEP_SHARD_COUNT`,
`KEEP_PEERS`. Single-node default (`mode=single`, one node owns everything).

**Gives:** capacity + throughput scale-out, blast-radius isolation.
**Does not give:** redundancy — a node loss makes its shards unavailable.

## Phase B — async replicas (planned)

Per shard: 1 primary + R read replicas. The primary streams its WAL (already the
ordered mutation log) to replicas over HTTP/2; replicas apply and may serve
bounded-staleness reads. Failover = promote a replica. Cheap, but a crash loses
the unreplicated tail → does **not** meet #114 "durable before ack" under node
loss. A middle ground, optional.

## Phase C — raft / quorum via `raft-host` (per-shard groups)

**Consensus core + per-shard structure: implemented** (behind the `raft`
feature, `src/raft.rs`). keep uses the shared **`raft-host`** driver over
`raft-core`, with h2c peer transport, snapshots, compaction, and read-your-write
`propose`. Proven by `tests/raft_node.rs` and the feature-gated HTTP API raft
replication test (`cargo test -p keep --features raft`). What's implemented:

- **Command = `WalOp`** (the logical mutation — same type the WAL/recovery use),
  serialized as the Raft log entry; `Response { applied }`.
- **`KvStateMachine`** — one raft state machine fronting a shard of the engine.
  `ShardHosts::write()` proposes → commits → applies via
  `RecoveryManager::apply_one` (the exact WAL-replay path). `snapshot()` dumps
  the engine (`KvEngine::dump_values`) and filters to the shard keyspace; a
  follower that has fallen behind is shipped the snapshot (InstallSnapshot) and
  loads it via `load_values`.
- **`ShardHosts`** — one `RaftHost` per shard group hosted by this node. A write
  routes by `crc32(key) % shard_count → shard → group`, so each shard is its own
  independently replicated consensus group. In replica mode, every participating
  replica node hosts the shard group; single node = one sole-voter group per
  shard.
- **HTTP write path** — replica-mode `AppState` carries `ShardHosts`; data-plane
  mutations are encoded as `WalOp` and routed through `host.propose` before the
  handler acknowledges. Default/single-node mode keeps the direct engine path.
- voter/learner membership comes from the StatefulSet-derived topology and
  `raft-host` membership wiring.

**Remaining staged work:**
1. **Failover gate** — promote a surviving replica under real process loss and
   prove committed data survives under the service-level HTTP path.
2. **Read consistency** — leader reads + bounded-lag follower reads via
   `x-read-consistency`.
3. **Membership changes** — promote/demote on scale events beyond the initial
   StatefulSet ordinal-derived set (`keep-<i>.keep-headless`).

The consensus core, per-shard structure, peer transport, snapshot/compaction,
apply path, and HTTP write propose path are implemented + validated. The
remaining slice is service-level failover/read-consistency hardening.

### Original design notes (historical openraft 0.9 sketch)

1. **TypeConfig** (`declare_raft_types!`): node id = pod ordinal; request =
   `Command` (an enum mirroring `WalOp` — the existing logical mutations);
   response = the op result.
2. **State machine** (`RaftStateMachine`): apply a committed `Command` to
   `KvEngine` (the apply path already exists — it's `recovery::apply_wal_operation`,
   which maps every `WalOp` onto an engine call; reuse it verbatim). Snapshot =
   the existing `SnapshotWriter`/engine `export_*`.
3. **Log storage** (`RaftLogStorage`): **the raft log SUBSUMES the current WAL.**
   keep already has a segmented, CRC'd, fsync'd append log (`persistence::wal`)
   with group commit — wrap it as the raft log store (vote + committed index +
   entries). This is the invasive part: the write path moves from
   "engine.set → log_wal → apply" to "raft.client_write(Command) → quorum commit
   → state-machine apply". Durable-before-ack becomes "replicated-and-fsynced
   before ack" — strictly stronger; the group-commit + `durability_barrier`
   machinery maps onto raft's apply notification.
4. **Network** (`RaftNetworkFactory`/`RaftNetwork`): AppendEntries / Vote /
   InstallSnapshot as HTTP/2 POSTs to `peers[node]` (reuse the existing hyper
   client stack; peers come from `ClusterConfig`). A small `/raft/*` internal
   route group, auth-gated off the public API.
5. **Membership / discovery**: k8s StatefulSet ordinals → stable DNS
   (`keep-<i>.keep-headless`); `ClusterConfig::peers` already models this. Initial
   cluster = `initialize()` with the ordinal set; changes via `change_membership`.
6. **Reads**: leader reads by default; bounded-lag follower reads via a
   `x-read-consistency` header (mirror lumen's `ReadConsistency`).

**Current staged risk:** the raft-host substrate and HTTP write propose path are
implemented, but service-level failover and read-consistency semantics still
need the dedicated multi-process harness for partition, leader-loss, and
log-truncation cases. Those gates remain tracked under the HA / Raft and Primary
Replicas capability work roots.

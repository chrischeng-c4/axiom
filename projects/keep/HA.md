# keep — high availability (#121)

HA lands in phases. **Phase A is implemented** (this commit); B and C are the
staged plan. The phases share one substrate: node identity + the keyspace split
(`src/cluster.rs`).

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

## Phase C — raft / quorum via `raftcore` (per-shard groups)

**Consensus core + per-shard structure: DONE** (behind the `raft` feature,
`src/raft.rs`). keep now uses the shared **`raftcore`** crate (`libs/raftcore`,
serde-only) — the same verified, k8s-failover-tested engine relay uses —
**replacing openraft** (the heavy `openraft` dependency is gone). Proven by
`tests/raft_node.rs` (`cargo test -p keep --features raft`). What's implemented:

- **Command = `WalOp`** (the logical mutation — same type the WAL/recovery use),
  serialized as the Raft log entry; `Response { applied }`.
- **`RaftKv`** — one raftcore group fronting the engine. `write()` proposes →
  commits → applies via `RecoveryManager::apply_one` (the exact WAL-replay path).
  `snapshot()` dumps the engine (`KvEngine::dump_values`) and `compact`s the log;
  a follower that has fallen behind is shipped the snapshot (InstallSnapshot) and
  loads it via `load_values` — so a lagging/new replica never replays full history.
- **`ShardedRaft`** — **one raft group per owned shard** (`cluster.owned_shards()`):
  a write routes by `crc32(key) % shard_count → shard → group`, so each shard is
  its own independently-replicated consensus group (the natural fit for Phase A's
  keyspace split). Single node = one sole-voter group per shard.
- voter/learner membership comes from `raftcore::auto_membership` (odd voter set;
  the trailing even node is a read-only learner).

**Remaining (the multi-node networking slice):**
1. **h2c driver** — feed each group's `handle`/`take_outgoing`/`tick` over HTTP/2
   (`/raft/*` POSTs to `ClusterConfig::peers`), mirroring relay's `raft_driver`.
   raftcore already emits AppendEntries/RequestVote/InstallSnapshot; this wires the
   transport + the tick/flush loop + persistence (`raftcore::PersistedState`).
2. **Durable raft log** — back `PersistedState` with keep's on-disk WAL so the
   public write path moves from `engine.set → log_wal` to `raft write`;
   durable-before-ack becomes replicated-and-fsynced-before-ack (strictly stronger).
3. **Membership / discovery** — derive voters from the StatefulSet ordinal set
   (`keep-<i>.keep-headless`); promote/demote on scale.
4. **Reads** — leader reads + bounded-lag follower reads via `x-read-consistency`.

The consensus core, per-shard structure, snapshot/compaction, and apply path are
done + validated; the multi-node h2c driver reuses the pattern relay already
proved on a real kind cluster.

### Original design notes (openraft 0.9)

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

**Why staged, not done here:** (3) reworks the durability/write path that the WAL
+ durable-before-ack commits just hardened, and correctness can only be trusted
with a multi-node test harness (partition / leader-loss / log-truncation cases).
Rushing it would risk the durability guarantees already shipped. Phase A is the
safe, useful, non-breaking increment; C is a dedicated effort on top of it.

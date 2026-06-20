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

## Phase C — raft / quorum via openraft

**Single-node integration: DONE** (behind the `raft` feature, `src/raft.rs`).
A write proposed through raft is committed by the quorum and applied to the
engine by the state machine — proven end-to-end by `tests/raft_node.rs`
(`cargo test -p keep --features raft`). This validates the consensus machinery
(log → commit → apply → snapshot) with the engine wired in. What's implemented:

- `TypeConfig`: `D = WalOp` (the logical mutation — same type the WAL/recovery
  use), `R = Response`.
- `StateMachineStore`: applies committed commands via
  `RecoveryManager::apply_one` (the exact WAL-replay path); snapshots dump/restore
  engine key→value (`KvEngine::dump_values`/`load_values`).
- `LogStore`: in-memory raft log (vote / entries / committed / purge).
- `Network`: stub (a single node sends no RPCs).
- `RaftKv::single_node` + `write()` (client_write → commit → apply).

**Remaining (the multi-node big part):**
1. **Network over HTTP/2** — replace the stub `RaftNetwork` with real
   AppendEntries/Vote/InstallSnapshot as `/raft/*` POSTs to `ClusterConfig::peers`
   (reuse the hyper client). `RaftNetworkFactory::new_client` per peer.
2. **Durable raft log** — the in-memory `LogStore` becomes the on-disk log,
   subsuming the existing WAL (keep already has a segmented, CRC'd, group-committed
   fsync log to wrap). The public write path then moves from
   `engine.set → log_wal` to `raft.client_write`; durable-before-ack becomes
   replicated-and-fsynced-before-ack (strictly stronger).
3. **Membership / discovery** — initialize from the k8s StatefulSet ordinal set
   (`keep-<i>.keep-headless`); `change_membership` for scaling.
4. **Reads** — leader reads + bounded-lag follower reads via `x-read-consistency`.

These are a dedicated multi-node effort (with a partition/leader-loss test
harness) — but the hard trait wiring (storage + state machine + type config) is
now done and validated single-node.

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

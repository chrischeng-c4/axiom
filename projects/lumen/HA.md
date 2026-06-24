# lumen — high availability & replication

## Decision: lumen owns replication in multi-pod mode

lumen is a **log-replicated, derived, rebuildable search index**: the caller
still owns the source of truth, and lumen indexes the caller's `external_id`s`.
The deployment boundary changed now that `libs/raftcore` exists: multi-pod
lumen should own its own write ordering and replica synchronization instead of
requiring an external broker as the default HA path.

The mode split is:

- **standalone**: one pod, local WAL, direct apply.
- **primary-replica**: multiple lumen pods, `raftcore` elects a leader, the
  leader owns the ordered write log, and followers replicate/apply the same raw
  `WalRecord::encode()` bytes.
- **relay**: explicit external-broker mode for deployments that still want a
  separate log owner.

lumen remains rebuildable: the upstream source of truth can replay documents.
But in primary-replica mode, a write acknowledged by lumen must be ordered by
lumen's own replication layer, not by Relay/NATS. Relay is no longer the default
answer for multi-pod synchronization.

### The `raft` module is becoming replication, not only cluster view

`src/raft.rs` (CODEGEN, from `tech-design/.../projects-lumen-src-raft-rs.md`)
currently provides the cluster-view DTOs the API serves on `/debug/cluster`:
peer DNS map, per-pod role, `applied_index` / `replication_lag_ms`, and the
`X-Read-Consistency` parsing (`leader` / `bounded(ms)` / `any`). That surface
remains useful, but the spec must be reframed from "broker-owned log tailing"
to "Lumen-owned primary/replica replication".

**Remaining, via the aw spec:** replace the CODEGEN cluster-view skeleton with
a real `raftcore`-backed Lumen replication surface. The existing language that
calls Raft only a debug/cluster view is now stale.

## #124 — broker log-tailing moves NATS → relay broadcast

lumen tails a broker via the `WalLog` trait
(`publish`/`subscribe(from_seq)`/`latest_seq`). The serving and k8s/operator
default broker is now **relay broadcast** (`--wal relay`), so each lumen pod
tails relay's ordered log and folds it into its index. The old NATS JetStream
backend remains legacy compatibility/test coverage, not the deployment default.

**Backend shape (`src/wal_relay.rs`):**

- `publish(record)` → CBOR `POST {relay}/v1/{subject}/publish` with a compact
  versioned `WalRecord::encode()` payload envelope; the returned
  `AppendOutcome.seq + 1` is Lumen's 1-based WAL seq. The message id includes a
  process-unique publisher id, so a restarted serving process does not dedupe
  new writes into old Relay entries.
- `subscribe(from)` → `GET {relay}/v1/{subject}/subscribe?from_seq={from}&subscriber_id={pod}`;
  decode relay's length-prefixed CBOR `LogEntry` frames
  (`relay::wire::decode_frames`) and yield `(seq + 1, WalRecord)`. The subscriber
  id is per pod (`LUMEN_RELAY_SUBSCRIBER_ID`, defaulting to `POD_NAME` /
  `HOSTNAME`) so every serving node gets the full fan-out stream.
- `latest_seq` → `GET {relay}/v1/{subject}/len`, returning Relay's append count
  as Lumen's latest 1-based WAL position.

**Constraint — keep the serving binary openssl-free.** Relay is **h2c
(cleartext)**. The current backend uses `reqwest` with HTTP/2 prior knowledge
and the workspace rustls stack; no OpenSSL dependency is introduced. The backend
is compiled by the `relay-wal` feature and is included in Lumen debug/release
builds and the source Docker image.

Wiring: `WalBackend::Relay` sits alongside `Embedded` / legacy `Nats` in
`src/bin/lumen.rs`; everything downstream (`WriteCoordinator`, the fold loop) is
unchanged because it only depends on the `WalLog` trait.

**HA note:** managed Lumen k8s should move to Lumen-owned primary/replica when
serving replicas exceed one. Managed Relay remains an explicit broker mode, not
the default multi-pod synchronization path.

## Status

- ✅ Decision updated: multi-pod lumen owns replication; Relay is explicit
  external-broker mode.
- ⬜ Replace the CODEGEN `raft` cluster-view skeleton with a `raftcore`-backed
  primary/replica replication surface.
- ✅ #124 `RelayWal`: h2c `WalLog` backend + `--wal relay` + in-process Relay
  tests for publish/tail, two-node fan-out, restart dedupe, reconnect from last
  seq, and invalid payload reporting.
- ⬜ Operator auto mode: one serving pod renders standalone; multiple serving
  pods render Lumen primary/replica with stable pod identity; explicit
  `broker.externalUrl` keeps Relay mode.

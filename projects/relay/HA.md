<!-- HANDWRITE-BEGIN gap="missing-generator:logic:80f2de20" tracker="pending-tracker" reason="New archetype-required HA doc: auto-mode (REPLICAS_PER_SHARD>1 flips raft), RelayStateMachine (publish replication, snapshot/compaction, fsynced applied-index marker), node-local lease/ack at-least-once failover limitation, RELAY_PEERS override, operator CR as the production HA path, backup/restore semantics, peer-TLS surface + raft-host TLS seam gap." -->
# relay — high availability (#1207)

relay's HA is **auto-mode raft** on the shared `libs/raft-host` driver (WI
#1207 replaced the hand-rolled driver/store/topology stack). One substrate
serves every deployment shape: the same single `relay` bin is the single-node
dev process, the raft replica pod, and the backup source.

## Auto-mode — scaling flips HA, not flags

Bare `relay` runs a plain single-node broker: no cluster env, no flags, direct
engine writes. Replica mode turns on when the StatefulSet scales out —
`raft_host::cluster::replica_mode()` is true when `REPLICAS_PER_SHARD > 1`
(a downward-API value). In that mode the serve path builds
`ClusterTopology::from_env` from `POD_NAME`, `SHARD_COUNT=1`,
`REPLICAS_PER_SHARD`, `VOTER_COUNT`, and `RELAY_PEER_SERVICE` (the headless
Service for peer DNS), and spawns one `RaftHost` for the whole node.

- relay is a **single-group** adopter (like lumen, unlike keep's
  host-per-shard): one raft group replicates every subject's publishes.
- Peer RPCs (`/raft/*`, `/raftz`) ride the serve port as tokenless cluster
  traffic — one port, no separate peer listener.
- `RELAY_PEERS=host:port,...` overrides peer DNS to run a multi-node group on
  one machine (the local/dev cluster recipe; proven by
  `tests/raft_cluster.rs`).

## RelayStateMachine — what replicates, and the restart floor

`src/raft.rs` wires the engine as a `raft_host::RaftStateMachine`:

- **Command = `PubCommand`** `{subject, message_id, payload, headers,
  priority, not_before}` — a multi-subject publish, one raft log entry per
  message. Publishes route through `host.propose` (leader appends; a follower
  publish is forwarded to the leader), commit, then the sole applier publishes
  idempotently through the engine.
- **Snapshot / compaction** come from the host: every `SNAPSHOT_EVERY` (1024)
  applied entries the state machine serializes the **live (un-acked) backlog**
  — delete-on-ack means capture cost tracks consumer lag, not publish volume —
  and the host compacts the log; a lagging or fresh replica is caught up via
  InstallSnapshot instead of full replay.
- **Fsynced applied-index marker**: relay's engine is delete-on-ack with a
  bounded dedupe window, so cold-replaying an already-acked committed publish
  would resurrect finished work. The state machine persists its applied index
  to a small fsynced marker file in the raft data dir (`{data_dir}/raft`) and
  skips entries at or below the recovered floor. With the default
  `FsyncPolicy::Always` engine the append is durable before the marker
  advances, so the floor never runs ahead of engine state; a crash between the
  two at worst re-applies one entry still inside the dedupe window.

## The honest limitation — leases/acks are node-local

Replication scope is **publishes only** (deliberate, unchanged from the old
driver). Leases, acks, heartbeats, and consume state stay node-local, fenced
per node by lease epochs. On failover:

- work that was leased-but-unacked on the old leader **redelivers**;
- acked work a follower has not yet trimmed via a snapshot install can
  redeliver too.

Delivery across failover is therefore **at-least-once** — workers must stay
idempotent (they already must be, for lease-expiry redelivery). Committed
publishes are never lost.

## Production path — the operator CR

The production HA deployment is the operator, not hand-applied YAML:

- `relay k8s crd render` — the `Relay` CRD (`relay.dev/v1alpha1`).
- `relay k8s operator render [--namespace ...]` + `relay k8s operator run`
  (build with `--features operator`) — the controller.
- `relay k8s instance render --profile prod` — a 3-replica raft-HA `Relay` CR;
  the operator renders the StatefulSet topology with the exact downward-API
  env auto-mode reads (plus `RELAY_BIND`/`RELAY_DATA_DIR`/`RELAY_GRACE_SECS`,
  `/healthz`/`/readyz` probes, PVC storage, and opt-in auth Secret wiring).

`k8s/` stays a single-node direct install for kind/smoke
(`scripts/kind-failover-smoke.sh`).

## Backup / restore

`GET /admin/backup` on a RUNNING node returns a consistent snapshot — the
exact raft-snapshot bytes (live un-acked backlog + applied index; one snapshot
format shared with InstallSnapshot). `relay backup --url http://<node>:7000
--dest file:///path|s3://bucket/prefix [--retention-secs N]` (build with
`--features backup`) ships it to a `service-backup` sink; the endpoint needs
`admin` on `*` when auth is required (`--token` / `RELAY_BACKUP_TOKEN`; the
operator injects it from `spec.backup.adminTokenSecret` and renders a
`<name>-backup` CronJob). Restore feeds the artifact to `load_live` on a fresh
node — an idempotent per-`message_id` merge. Leases are node-local, so
restored work redelivers (at-least-once, same rule as failover).

## Peer TLS — config surface now, transport seam pending

Replica mode loads peer mTLS material via `service-tls`:
`RELAY_PEER_TLS_CERT` / `RELAY_PEER_TLS_KEY` / `RELAY_PEER_TLS_CA`
(+ `RELAY_PEER_MTLS=on` to require client certs). Serve validates the material
fail-fast at startup — partial config or a mis-pointed path exits nonzero.
**Honest limit:** raft-host's h2c peer transport has no TLS seam yet, so mTLS
termination is not applied to `/raft/*` traffic — peer RPCs stay cleartext h2c
inside the cluster until the shared seam lands (tracked as a raft-host gap;
relay adopts it the release it exists).
<!-- HANDWRITE-END -->

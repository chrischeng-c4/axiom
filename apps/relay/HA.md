<!-- HANDWRITE-BEGIN gap="missing-generator:logic:80f2de20" tracker="pending-tracker" reason="Relay HA contract: auto-mode, fully committed work-queue lifecycle, snapshots/applied floor, operator topology, backup/recovery, and shared authenticated peer transport." -->
# relay — high availability (#1207)

relay's HA is **auto-mode raft** on the shared `libs/raft-runtime` driver (WI
#1207 replaced the hand-rolled driver/store/topology stack). One substrate
serves every deployment shape: the same single `relay` bin is the single-node
dev process, the raft replica pod, and the backup source.

## Auto-mode — scaling flips HA, not flags

Bare `relay` runs a plain single-node broker: no cluster env, no flags, direct
engine writes. Replica mode turns on when the StatefulSet scales out —
`raft_runtime::cluster::replica_mode()` is true when `REPLICAS_PER_SHARD > 1`
(a downward-API value). In that mode the serve path builds
`ClusterTopology::from_env` from `POD_NAME`, `SHARD_COUNT=1`,
`REPLICAS_PER_SHARD`, `VOTER_COUNT`, and `RELAY_PEER_SERVICE` (the headless
Service for peer DNS), and spawns one `RaftHost` for the whole node.

- relay is a **single-group** adopter (like lumen, unlike keep's
  host-per-shard): one raft group replicates every subject's queue lifecycle.
- With peer mTLS enabled, peer RPCs (`/raft/*`, `/raftz`) use the dedicated
  authenticated listener (default 7001); local development may explicitly use
  the cleartext serve port.
- `RELAY_PEERS=host:port,...` overrides peer DNS to run a multi-node group on
  one machine (the local/dev cluster recipe; proven by
  `tests/raft_cluster.rs`).

## RelayStateMachine — what replicates, and the restart floor

`src/raft.rs` wires the engine as a `raft_runtime::RaftStateMachine`:

- **Command = `RelayCommand`** — publish, lease/batch lease, ack/batch ack,
  release, heartbeat, and reconcile all route through `host.propose`. The
  proposer resolves time and executor identity before commit; every state
  machine applies identical transitions.
- **Snapshot / compaction** come from the host: every `SNAPSHOT_EVERY` (1024)
  applied entries the state machine serializes the **live queue state**:
  original seq/next-seq, un-acked entries, leases, fencing epochs, committed
  offsets, retry/delay state, and recent proposal outcomes. Restore replaces
  local state exactly before log tailing resumes.
- **Fsynced applied-index marker**: relay's engine is delete-on-ack with a
  bounded dedupe window, so cold-replaying an already-acked committed publish
  would resurrect finished work. The state machine persists its applied index
  to a small fsynced marker file in the raft data dir (`{data_dir}/raft`) and
  skips entries at or below the recovered floor. With the default
  `FsyncPolicy::Always` engine the append is durable before the marker
  advances, so the floor never runs ahead of engine state; a crash between the
  two at worst re-applies one entry still inside the dedupe window.

## Committed delivery ownership

The assignment commits before Relay sends a work frame. Each lease carries an
executor node and monotonic epoch; ack, nack, and heartbeat must match both.
Another replica cannot concurrently lease the same entry or complete an old
owner's work. Delivery remains **at-least-once** because an external worker may
finish while its result is ambiguous, and expiry intentionally redelivers.

## Production path — the operator CR

The production HA deployment is the operator, not hand-applied YAML:

- `relay k8s crd render` — the `Relay` CRD (`relay.dev/v1alpha1`).
- `relay k8s operator render [--namespace ...]` + `relay k8s operator run`
  (build with `--features operator`) — the controller.
- `relay k8s instance render --profile prod` — a 3-replica raft-HA `Relay` CR;
  the operator renders the StatefulSet topology with the exact downward-API
  env auto-mode reads (plus `RELAY_BIND`/`RELAY_DATA_DIR`/`RELAY_GRACE_SECS`,
  `/healthz`/`/readyz` probes, PVC storage, and opt-in auth Secret wiring).

`k8s/` provides a layered direct base plus dev/staging/prod/template overlays;
the production operator CR remains the authoritative HA topology.
`scripts/kind-failover-smoke.sh` drives a disposable three-voter leader-kill
proof with the same standard topology variables.

## Backup / restore

`GET /admin/backup` on a RUNNING node returns a consistent snapshot — the
exact raft-snapshot bytes (live un-acked backlog + applied index; one snapshot
format shared with InstallSnapshot). `relay backup --url http://<node>:7000
--dest file:///path|s3://bucket/prefix [--retention-secs N]` (build with
`--features backup`) ships it to a `service-backup` sink; the endpoint needs
`admin` on `*` when auth is required (`--token` / `RELAY_BACKUP_TOKEN`; the
operator injects it from `spec.backup.adminTokenSecret` and renders a
`<name>-backup` CronJob). Restore feeds the artifact to `load_live` for exact
state replacement, preserving queue sequence space and committed ownership.

## Peer TLS

Replica mode loads peer mTLS material via `peer-tls`:
`RELAY_PEER_TLS_CERT` / `RELAY_PEER_TLS_KEY` / `RELAY_PEER_TLS_CA`
(+ `RELAY_PEER_MTLS=on` to require client certs). Serve validates the material
fail-fast at startup — partial config or a mis-pointed path exits nonzero.
`raft-runtime::PeerTransport` serves `/raft/*` on the dedicated listener with
mutual identity validation and dials every peer over the same reloadable
certificate snapshot. Unknown CAs, wrong server identities, expired chains,
and malformed rotations fail closed before the Raft router. Plain h2c remains
available only when peer mTLS is explicitly disabled for local development.
<!-- HANDWRITE-END -->

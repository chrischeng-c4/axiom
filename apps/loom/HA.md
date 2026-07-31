# loom — High Availability

loom is the control plane: it owns the sharded, strongly-consistent workflow /
DAG state and never sits in the data path (payload bytes claim-check through
keep). HA is therefore about **replicating that state machine** so a lost pod
does not lose or stall runs. loom uses the shared `libs/raft-host` driver over
`libs/raft-core` — the same consensus stack as lumen/keep/relay — so there is one
HA engine to operate, not one per service.

## The state machine

The replicated state is loom's run map (`WorkflowRunId → WorkflowRun`). Every
mutation is a `Command` (`PutRun` / `DeleteRun`) proposed to the raft leader;
`RaftHost::propose` returns only after the local state machine applies it
(read-your-write), so a subsequent read on the leader always sees the write.
`LoomSm` (loom's `raft_host::RaftStateMachine`) folds committed commands into the
map and persists a materialized snapshot (map + covered log index) to disk on
every apply. That snapshot is the **backup layer**: it powers install-snapshot
for follower catch-up and a cold-start recovery that does not replay the whole
log.

## Auto-mode: single-node by default, HA on scale-out

`loom controller` needs no flags to run. The store backend is chosen at startup:

| Condition | Backend |
|-----------|---------|
| `REPLICAS_PER_SHARD > 1` (downward API) | **raft replica mode** — `ClusterTopology::from_env` derives node id / voters / peers |
| `LOOM_CLUSTER_PEERS=0=url,1=url,…` | raft cluster from an explicit peer map (local multi-node testing) |
| `LOOM_RAFT_DIR=<dir>` | single-node durable raft (its own majority) |
| `LOOM_DATA_DIR=<dir>` | non-replicated file store |
| _(none)_ | in-memory (dev) |

So the **same image** is single-node in dev and a raft group in prod: the only
difference is `REPLICAS_PER_SHARD` (and the replica count), set by the k8s
overlay. `replica_mode()` reads it from the StatefulSet downward API — there is
no per-replica configuration and no pod-ordinal / peer-DNS math to hand-roll.

## Kubernetes topology

- **StatefulSet `loom`** — stable pod identity (`loom-0`, `loom-1`, …). The base
  is single-node; the `staging` / `prod` overlays scale to 3 and set
  `REPLICAS_PER_SHARD` / `VOTER_COUNT` to 3.
- **`podManagementPolicy: Parallel`** — raft forms quorum only once peers are up,
  so pods must start together (OrderedReady would deadlock: pod N waits for pod
  N-1 Ready, which needs quorum).
- **Headless service `loom-headless`** (`clusterIP: None`,
  `publishNotReadyAddresses: true`) — gives `loom-0.loom-headless…` DNS that raft
  peer discovery + `ClusterTopology::from_env` consume.
- **Client service `loom`** (ClusterIP) — where callers submit runs / poll status.
- **PodDisruptionBudget** (`maxUnavailable: 1`) — keeps a raft majority during
  drains / rolling updates.
- **Downward-API env** — `POD_NAME`, `POD_NAMESPACE`, `SHARD_COUNT`,
  `REPLICAS_PER_SHARD`, `VOTER_COUNT` feed the topology; `LOOM_RAFT_DIR=/data/raft`
  is the raft log + snapshot on the per-pod PVC.

## Deploy paths

- **Direct install (kind / smoke):** `kubectl apply -k projects/loom/k8s/overlays/dev`
  — the single-node base. Small and self-contained; relay + keep are prerequisites
  (their own services).
- **Production HA:** the operator CR path (`loom k8s operator run` +
  `loom k8s instance render --profile prod`) renders the sharded StatefulSet
  topology and the downward-API env that `raft-host` consumes. The operator owns
  RBAC / Services / StatefulSet / PDB / status + the backup CronJob; it never
  serializes loom's data — snapshot bytes come from `LoomSm`, `raft-host` installs
  snapshots and compacts logs, and `libs/service-backup` uploads them.

## Failure behavior

- **Leader loss:** a follower wins the next election; in-flight `propose`s retry
  against the new leader. Committed runs are durable (majority-replicated).
- **Follower rejoin / new replica:** catches up by tailing the log, or via
  install-snapshot when it is behind the leader's compaction point.
- **Full restart:** each pod cold-starts from its on-disk snapshot + log; the
  group re-elects and resumes. Runs are not lost.

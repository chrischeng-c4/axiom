# Tape Capabilities

<!-- aw:meta:project-capabilities:start -->
## Brief

Machine-readable capability contract for Tape.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
<!-- aw:meta:project-capabilities:end -->

## Verified Cloud Evidence

Standard GKE operator acceptance evidence for Tape (epic #2434 ordered
service 2, after Lumen run `0723041614`). The machine-readable capability
contract currently lives in `apps/tape/README.md` (`cap_path`); this section
records real-cloud proof runs until the #1848 cap_path relocation lands.
Harness: `benchmarks/gcp-operator-acceptance` (`ACCEPTANCE_APPS=tape`).

### GKE acceptance run 0723080156 (2026-07-23, partial)

- Cluster: persistent Standard GKE `axiom-operator-acceptance`
  (`asia-east1-a`, project `axiom-502607`), run-scoped bucket/GSA/Workload
  Identity created and destroyed by the run.
- Image: pinned immutable
  `courier/tape@sha256:03d57879282802c9fc8da8de77582e9bea877f224d1485a17e2125a82a5cfb5b`
  built by Cloud Build from commit `8a0f4dffbb` (includes the #2443
  `lost+found` seed fix) with features `operator backup`.
- Evidence root: `axiom-gcp-run-backup/evidence/0723080156/` (home-dir
  mirror of the volatile `/tmp` evidence tree); `run.log` carries the full
  transcript.

Proven in this run (each row names its artifact under the evidence root):

| Proof | Result | Artifact |
|---|---|---|
| Operator cell: RBAC, Lease, steady-state drift repair, leader-takeover reconcile | passed | `tape-operator-cell.json`; `kubernetes/tape-lease-holder-*.txt` |
| 1x1 reconcile with status-generation fence on Standard GKE | passed | `kubernetes/tape-crs.json`; `kubernetes/workloads-after-tape-deploy.json` |
| Domain lifecycle through the client Service: append offsets 0-2, replay, subscription create, pull cursor 0→3, cumulative ack, empty re-pull | passed | `kubernetes/tape-append.jsonl`; `kubernetes/tape-replay-initial.json`; `kubernetes/tape-pull-before-ack.json`; `kubernetes/tape-ack.json`; `kubernetes/tape-pull-after-ack.json` |
| Pod-restart data retention on the PVC journal (3 events + checkpoint offset 3 survive `tape-0` replacement) | passed | `kubernetes/tape-replay-after-restart.json`; `kubernetes/tape-checkpoint-after-restart.json` |
| Workload-Identity GCS backup: CronJob-triggered `tape backup` writes a 635-byte `JournalSnapshot` readback containing the 3 appended events | passed | `kubernetes/tape-backup.log`; `gcs/tape-first-object.json`; `gcs/tape-first-object-bytes.txt` |
| Verified cleanup: run-scoped bucket, GSA, IAM, image tag, namespaces, CRD destroyed; persistent cluster and pre-existing APIs preserved | passed | `cleanup.json` (`status: clean`, 2026-07-23T08:52:34Z) |

Not proven in this run (recorded, not claimed):

- Cold restore to fresh PVCs bootstrapping a 3-replica/3-voter topology
  (`bootstrapSeedUri`) and raft leader failover: the 3-replica StatefulSet
  could not converge — the render's pod anti-affinity needs one node per
  replica but the persistent node pool caps at 2 nodes, and the serving
  `tape` ServiceAccount has no Workload Identity binding for the GCS seed
  read. Rerun pending under WI #2436 after the harness fixes.
- Shard migration (`shardCount` pinned to 1), live in-place replica
  membership change (startup-static membership), and CPU/memory pressure
  actuation remain `not_claimed`; product gaps tracked in #2437.

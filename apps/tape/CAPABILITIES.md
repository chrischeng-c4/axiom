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

### GKE acceptance run 0723135853 (2026-07-23, PASSED — full)

- Cluster: persistent Standard GKE `axiom-operator-acceptance`
  (`asia-east1-a`, project `axiom-502607`), run-scoped bucket/GSA/Workload
  Identity created and destroyed by the run.
- Image: pinned immutable
  `courier/tape@sha256:05d2a6f0ff3a6de1ba2be2c9566a3c509e75c62025995713e6475a87792bd619`
  built by Cloud Build from commit `b69ad94947` (includes the #2443
  `lost+found` and #2465 genesis-index seed fixes) with features
  `operator backup`.
- Terminal artifacts: `acceptance.json` (`axiom.gcp.tape.acceptance.v1`,
  every claimed proof `passed`) and `cleanup.json` (`status: clean`,
  2026-07-23T14:18:01Z). Evidence root:
  `axiom-gcp-run-backup/evidence/0723135853/` (home-dir mirror of the
  volatile `/tmp` tree); `run.log` carries the full transcript.

Proven in this run (each row names its artifact under the evidence root):

| Proof | Result | Artifact |
|---|---|---|
| Operator cell: RBAC, Lease, steady-state drift repair, leader-takeover reconcile | passed | `tape-operator-cell.json`; `kubernetes/tape-lease-holder-*.txt` |
| 1x1 reconcile with status-generation fence on Standard GKE | passed | `kubernetes/tape-crs.json`; `kubernetes/workloads-after-tape-deploy.json` |
| Domain lifecycle through the client Service: append offsets 0-2, replay, subscription create, pull cursor 0→3, cumulative ack, empty re-pull | passed | `kubernetes/tape-append.jsonl`; `kubernetes/tape-replay-initial.json`; `kubernetes/tape-pull-*.json`; `kubernetes/tape-ack.json` |
| Pod-restart data retention on the PVC journal (3 events + checkpoint offset 3 survive `tape-0` replacement) | passed | `kubernetes/tape-replay-after-restart.json`; `kubernetes/tape-checkpoint-after-restart.json` |
| Workload-Identity GCS backup: CronJob-triggered `tape backup` writes a 635-byte `JournalSnapshot`; readback carries the appended events | passed | `kubernetes/tape-backup.log`; `gcs/tape-first-object.json` |
| Cold restore: the exact GCS object seeds a fresh-PVC 3-replica/3-voter topology via `bootstrapSeedUri`; restored replay shows exactly offsets 0-2 and checkpoint 3 | passed | `kubernetes/tape-restore-cr.yaml`; `kubernetes/tape-replay-after-restore.json`; `kubernetes/tape-checkpoint-after-restore.json` |
| One-shot seed cleared (#2468 contract): field removed post-restore, full no-seed rolling restart retains all events | passed | `kubernetes/tape-replay-after-seed-clear.json` |
| Raft leader disruption: leader pod deleted (grace 1s), group re-elects (term 12→14; the replaced pod legitimately re-won, `distinct: false` recorded honestly), post-failover write commits at offset 3 and replays as the fourth event | passed | `kubernetes/tape-raftz-initial.json`; `kubernetes/tape-raftz-after-failover.json`; `kubernetes/tape-append-after-failover.json`; `kubernetes/tape-replay-after-failover.json` |
| Verified cleanup: run-scoped bucket, GSA, IAM, image tag, namespaces, CRD destroyed; persistent cluster and pre-existing APIs preserved | passed | `cleanup.json` |

Product defects found and fixed by this campaign (each verified by the
passing run above): #2443 (`lost+found` on cloud PVs rejected cold seeds),
#2465 (single-node-origin `up_to: 0` seeds silently restored an EMPTY
group — genesis-index mapping added, regression
`apps/tape/tests/seed_ha_bootstrap.rs`), #2468 (`bootstrapSeedUri` is
one-shot: leaving it on the CR crash-loops any pod replacement; the
operator-side lifecycle decision remains open).

Exclusions (recorded, not claimed): shard migration (`shardCount` pinned
to 1), live in-place replica membership change (startup-static
membership), and CPU/memory pressure actuation — product gaps tracked in
#2437. Earlier partial runs' evidence (0723080156 six-proof subset and the
intermediate diagnosis runs) is retained under the same backup root.

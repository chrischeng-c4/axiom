# Lumen Capabilities

<!-- aw:meta:project-capabilities:start -->
## Brief

Machine-readable capability contract for Lumen.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
<!-- aw:meta:project-capabilities:end -->

## Verified Cloud Evidence

Standard GKE operator acceptance evidence for Lumen (epic #2434 ordered
service 1, before Tape run `0723135853`). The machine-readable capability
contract currently lives in `apps/lumen/README.md` (`cap_path`); this
section records real-cloud proof runs until the #1848 cap_path relocation
lands. Harness: `benchmarks/gcp-operator-acceptance` (Lumen-only mode).

### GKE retest runs 0723160506 / 0723163748 (2026-07-23, FAILED — post-split read visibility, #2489)

Retest with the released `lumen@0.4.24` GHCR image
(`ghcr.io/chrischeng-c4/lumen@sha256:f460c6cf…493e90`, pulled anonymously —
the GHCR distribution path itself works). Passed on both runs: 1x1
reconcile, operator cell, index/search, Workload-Identity GCS backup,
pod-restart retention, and the 1→2 split convergence with a fully converged
post-cutover fence. FAILED both runs at post-split read visibility:
searching the pre-split collection through the client Service returns
`collection not found` and stays unreadable through a bounded 180-second
poll while `phase: Complete` and `convergedShardMapVersion ==
shardMap.version` — tracked as #2489. The prior run `0723041614`'s
post-split pass asserted a single probe and cannot stand as disproof;
treat the Dynamic Shard Topology GKE claim as NOT proven until #2489
closes. Default 1-shard deployments (no reshardPolicy) are unaffected.
Evidence: `axiom-gcp-run-backup/evidence/<run>/`.

### GKE acceptance run 0723041614 (2026-07-23, PASSED)

- Cluster: persistent Standard GKE `axiom-operator-acceptance`
  (`asia-east1-a`, project `axiom-502607`), run-scoped
  bucket/GSA/Workload-Identity binding created and destroyed by the run.
- Image: pinned immutable
  `courier/lumen@sha256:da154652ff3fdf16fb406674240f0a3f4567047d5eb6e0e547bee0f389c68b1b`
  built from commit `f4762759d810` (`git_dirty: false`, `image_provenance:
  prebuilt`, tag `f4762759d810-0723041614`).
- Terminal artifacts: `acceptance.json`
  (`axiom.gcp.lumen.acceptance.v1`, every claimed proof `passed`) and
  `cleanup.json` (`status: clean`, verified `2026-07-23T04:25:33Z`).
  Evidence root: `axiom-gcp-run-backup/evidence/0723041614/` (home-dir
  mirror of the volatile `/tmp` tree); `run.log` carries the full
  transcript.

Proven in this run (each row names its artifact under the evidence root):

| Proof | Result | Artifact |
|---|---|---|
| Operator cell: RBAC, Lease creation, steady-state drift repair, leader-takeover reconcile (holder `...rrc6f` → `...5mlwx`) | passed | `lumen-operator-cell.json`; `kubernetes/lumen-lease-holder-*.txt` |
| 1x1 reconcile: one `Lumen` CR drives exactly one StatefulSet/shard to `Ready` on Standard GKE | passed | `kubernetes/lumen-crs.json`; `kubernetes/workloads-after-lumen-deploy.json` |
| Domain lifecycle through the client Service: create collection, index one document, search hit | passed | `kubernetes/lumen-create-collection.json`; `kubernetes/lumen-index.json`; `kubernetes/lumen-search-before-restart.json` |
| Pod-restart data retention: the indexed document survives a serving-pod replacement and is still searchable via the PVC-backed segment/WAL | passed | `kubernetes/lumen-search-after-restart.json` |
| Workload-Identity GCS backup: CronJob-triggered backup writes a non-empty 271-byte snapshot object; readback is non-empty | passed | `kubernetes/lumen-backup.log`; `gcs/lumen-first-object.json` (`gs://axiom-502607-axo-0723041614-backup/lumen/0723041614-1784780377.json`) |
| Acceptance-only disk-pressure auto-split: `reshardPolicy.maxShardBytes: 1` (a test-only trigger, not a production threshold) drives shard count 1 → 2, 2 ready pods, at least 2 PVCs, and the document stays searchable post-split | passed | `kubernetes/lumen-after-split.json`; `kubernetes/lumen-search-after-split.json` |
| Shard-map fence convergence: post-split CR status settles at `reshard.phase: Complete`, `targetShardCount` == `shardCount` == 2, `usageMeasuredAtMapVersion: 1`, `convergenceRemediationRestartCount: 0` (no remediation restarts needed) | passed | `kubernetes/lumen-after-split.json` |
| Verified cleanup: run-scoped GCS bucket, backup GSA, IAM bindings destroyed (`Destroy complete! Resources: 4 destroyed`, "no run-tagged Lumen/Sift operator acceptance resources remain"); persistent cluster, Artifact Registry, and pre-existing APIs preserved | passed | `cleanup.json`; `run.log` |

Exclusions (recorded, not claimed): CPU/memory pressure actuation
(`cpu_memory_actuator: not_claimed`) and live in-place replica-membership
change (`live_replica_membership: not_claimed`) — neither is exercised by
this harness. `reshardPolicy.maxShardBytes: 1` is an acceptance-only
trigger value chosen to force a split deterministically inside a short
run; it is not evidence of any production disk-pressure threshold. Sift
was deferred from this run (`sift_collection_deferred`); Tape's own run
(`0723135853`) is recorded separately in `apps/tape/CAPABILITIES.md`.

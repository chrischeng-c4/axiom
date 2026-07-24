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
lands. Harness: `benchmarks/gcp-operator-acceptance` (mode noted per run).

### GKE acceptance run 0724105144 (2026-07-24, PASSED — auth+CSI Secret Manager stack proven, #2457/#2456)

Full two-service digest-mode run (GHCR `sha-54742a8d6e40` images — zero
Cloud Build) adding the first live validation of the auth+CSI regression
leg: a `lumen-authcsi` CR with `auth: required`,
`tokensSecretProviderClass`, and `tokensSecretCsiDriver:
secrets-store-gke.csi.k8s.io` against a run-scoped Secret Manager secret
(SecretProviderClass `provider: gke`, `principal://` secretAccessor grant,
no GSA). Proven: CSI volume mounted with the GKE driver name, pod Ready
with **zero FailedMount events** (the exact #2456 failure signature),
tokens genuinely loaded from the CSI mount — authenticated search returns
the seeded document (`total: 1`) while unauthenticated returns exactly 401
`{"error":"unauthenticated"}`. All prior legs re-passed on the 0.4.26
candidate HEAD (cold-restore, admission, backup, auto-split 1→2), and
verified cleanup covers the new Secret Manager resources. Cluster
prerequisite recorded: the GKE Secret Manager add-on
(`--enable-secret-manager`) registers the CSIDriver; the leg self-skips
with evidence when absent. Evidence root:
`axiom-gcp-run-backup/evidence/0724105144/` (`kubernetes/lumen-authcsi-*`).

### GKE acceptance run 0724061548 (2026-07-24, PASSED — #2489 fix + cold-restore #2492 proven)

- Full two-service mode (Lumen and Sift both passed; the Sift rows live in
  the shared `acceptance.json`). Cluster: persistent Standard GKE
  `axiom-operator-acceptance` (`asia-east1-a`, project `axiom-502607`),
  run-scoped bucket/GSA/Workload-Identity bindings plus the restore-reader
  grant created and destroyed by the run.
- Image: Cloud Build from commit `70fd48ca5c44` (the `lumen@0.4.25`
  candidate — carries the #2489 scatter fix `9ffdb30513`, #2497
  `spec.serviceAccountName`, and the #2487 alert fix), tag
  `70fd48ca5c44-0724061548`, dirty-tree gate clean.
- Terminal artifacts: `lumen-acceptance.json`
  (`axiom.gcp.lumen.acceptance.v1`, every claimed proof `passed`) and
  `cleanup.json` (`status: clean`, verified `2026-07-24T06:56:11Z`).
  Evidence root: `axiom-gcp-run-backup/evidence/0724061548/`.

| Proof | Result | Artifact |
|---|---|---|
| Post-split read visibility (#2489): after the CONVERGED 1→2 auto-split, the pre-split collection is searchable through the client Service immediately — readability lag 0 s (vs `collection not found` for 180 s+ on both 0.4.24 retest runs). Restores the Dynamic Shard Topology GKE claim. | passed | `kubernetes/lumen-search-after-split.json`; `kubernetes/lumen-split-readable-after-seconds.txt` (`0`) |
| Cold-restore onto a fresh PVC (#2492): a second `lumen-restore` CR with `spec.serving.bootstrap.seedUri` pointed at the run's backup object (271 B, carries the `acceptance` collection) boots a genuinely fresh PVC and the seeded document is queryable (`total: 1`) | passed | `kubernetes/lumen-restore-search.json`; `gcs/lumen-first-object.json` |
| Seed-set restart retention: the restored instance keeps the seeded document across a serving-pod replacement | passed | `kubernetes/lumen-restore-after-restart-search.json` |
| Admission CR exposure (#2477): patching `spec.admission` renders the five `LUMEN_ADMISSION_*` env vars onto the StatefulSet pod spec (operator-propagation-aware poll), and removing the block rolls them back off | passed | `kubernetes/lumen-admission-env.txt` |
| Re-proven from `0723041614`: 1x1 reconcile, domain lifecycle (create/index/search), pod-restart data retention, Workload-Identity GCS backup (271-byte object) | passed | `kubernetes/…` per the matching rows in the `0723041614` table below |
| Verified cleanup: 6 run-scoped resources destroyed; "no run-tagged Lumen/Sift operator acceptance resources remain"; persistent cluster and Artifact Registry preserved | passed | `cleanup.json`; `run.log` |

Exclusions unchanged from `0723041614` (`cpu_memory_actuator`,
`live_replica_membership`: `not_claimed`). Deployer note for cold-restore:
the SERVING ServiceAccount of a `seedUri` instance reads GCS itself — it
needs `roles/storage.objectViewer` on the seed bucket (the backup GSA's
write grant does not cover it). The harness provisions this via
`storage.tf`'s `lumen_restore_reader` principal binding; real deployments
carry the same responsibility.

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
Evidence: `axiom-gcp-run-backup/evidence/<run>/`. Resolution: the #2489
scatter fix (`9ffdb30513`) is proven by run `0724061548` above — the claim
is restored there.

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

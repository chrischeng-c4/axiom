# GCP operator acceptance: Lumen, then Sift

This harness is a low-cost GKE proof for the shared service
operator shape. It completes an independent Lumen acceptance phase first; only
then does it install Sift and its node-level collector to collect Lumen's real
structured stdout. The image, Terraform, manifest-rendering, evidence, and
cleanup boundaries remain reusable for Tape, Relay, and Defer.

It keeps one named Standard GKE test cluster between runs, then creates these
short-lived resources tagged with one `RUN_ID`:

- one force-destroy GCS backup bucket with uniform bucket access, enforced
  public-access prevention, and a one-day object lifecycle;
- one shared backup GSA;
- bucket-scoped `roles/storage.objectAdmin` for that backup GSA;
- Workload Identity bindings for `lumen/lumen-backup` and
  `sift/sift-backup`.

It does **not** create Pub/Sub, Cloud Tasks, Cloud Run, a LoadBalancer, a NAT,
or an Artifact Registry repository. `courier` is the default existing Docker
repository and is only read. Required project APIs must already be enabled;
the harness records and preserves them rather than changing project API state.

## Acceptance boundary

The test is deliberately narrower than a production or competitor benchmark:

- Lumen first proves 1x1 reconcile, operator leader takeover and drift repair,
  an indexed-document read after a serving-pod restart, a real GCS JSON
  snapshot, and a one-byte acceptance threshold that changes `shardCount`
  from 1 to 2 exactly with two ready Pods and PVCs.
- Sift is not installed until that Lumen phase passes. It then proves its 1x1
  operator lifecycle, deploys the Sift-owned CRI collector DaemonSet, triggers
  a new Lumen audit event, and queries Sift's logging projection for that exact
  event plus GKE namespace/container metadata. It also proves the Sift live
  backup CronJob writes a non-empty JSON snapshot to GCS. No Sift topology
  beyond 1x1 is claimed.
- The one-byte split threshold is test-only. It avoids generating a chargeable
  gigabyte and says nothing about the production 1 GiB threshold.
- CPU/memory-driven replica actuation is not claimed.
- Live Raft replica membership changes are not claimed.
- The harness proves the controller, CR/status generation fence, child
  resources, backup identity, real GCS transport, and Lumen's supported +1
  auto-split. Domain performance and competitor parity stay in local/arena
  suites.

These exclusions are also written into `acceptance.json`, so a passing run
cannot be mistaken for broader scaling evidence.

## Prerequisites

Use a project where billing is enabled and these APIs already exist:

```text
artifactregistry.googleapis.com
cloudbuild.googleapis.com
compute.googleapis.com
container.googleapis.com
iam.googleapis.com
iamcredentials.googleapis.com
storage.googleapis.com
```

The caller needs permission to submit Cloud Builds, create/delete the initial Standard GKE
cluster, bind IAM, create/delete one bucket, and push to
the existing Docker repository. Required local commands are `cargo`, `curl`,
`gcloud`, `git`, `jq`, `kubectl`, and `terraform`.

## Exact lifecycle

Run the static gate first. It performs shell syntax checks, Terraform format,
an isolated `terraform init -backend=false`, and `terraform validate`; it does
not contact your GCP project or create resources.

```bash
benchmarks/gcp-operator-acceptance/scripts/check.sh
```

Then run acceptance with the billing project explicit. Region and existing
repository have conservative defaults:

```bash
PROJECT_ID=axiom-502607 \
REGION=asia-east1 \
GKE_ZONE=asia-east1-a \
ARTIFACT_REGISTRY_REPOSITORY=courier \
benchmarks/gcp-operator-acceptance/scripts/run.sh
```

For routine acceptance, pass the immutable GitHub-release-derived image
digests and no Cloud Build or staged source archive is used. A candidate can
replace just one service; the harness builds only the missing service target.

```bash
PROJECT_ID=axiom-502607 \
LUMEN_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/lumen@sha256:<digest> \
SIFT_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/sift@sha256:<digest> \
benchmarks/gcp-operator-acceptance/scripts/run.sh
```

The first run bootstraps `axiom-operator-acceptance`; later runs reuse it. To
create it explicitly (or select a different persistent name), run:

```bash
PROJECT_ID=axiom-502607 REGION=asia-east1 GKE_ZONE=asia-east1-a \
PERSISTENT_CLUSTER_NAME=axiom-operator-acceptance \
benchmarks/gcp-operator-acceptance/scripts/bootstrap-cluster.sh
```

`run.sh` performs this order:

1. verify pre-existing APIs/repository and build the Lumen/Sift deployment
   CLIs locally;
2. reuse caller-supplied immutable release/candidate images, or submit a
   target-specific Cloud Build only for each service image that was omitted;
3. resolve any source-built tag to a `sha256` digest reference;
4. use each app CLI to render CRD, operator, and instance layers, then validate
   the overlays with `kubectl kustomize`;
5. bootstrap or reuse the zonal Standard cluster and bounded node pool, then
   create only the run-scoped GCS bucket, GSA, and Workload Identity edges;
6. apply only the CLI-rendered Lumen layers, require status generation, perform
   operator drift/takeover tests, verify persistence and GCS backup, then
   require the exact 1-to-2 shard transition;
7. only after step 6 passes, apply Sift's CLI-rendered CRD/operator/instance,
   run its drift/takeover tests, deploy its CLI-rendered Standard-GKE CRI
   collector, and query the materialized Sift log produced by a new Lumen
   collection event; then prove Sift's live backup reaches GCS;
8. cancel any still-running Cloud Build, delete app namespaces and CRDs,
   destroy only run-scoped Terraform resources, remove only run-tagged image
   tags/digests and the exact Cloud Build source prefix, and independently
   verify cleanup. The persistent cluster stays available for the next run.

The cloud portion has a hard maximum of 2,700 seconds (45 minutes). `EXIT`,
`INT`, `TERM`, failures, and the watchdog all enter the same cleanup trap. A
separate explicit cluster teardown is deliberately required; a normal run
never deletes the reusable cluster.

If a shell or machine failure prevents the trap from finishing, rerun cleanup
with the values printed in `run.json`:

```bash
PROJECT_ID=axiom-502607 REGION=asia-east1 GKE_ZONE=asia-east1-a RUN_ID=0722123456 \
STATE_DIR=/tmp/axiom-gcp-operator-0722123456 \
EVIDENCE_DIR=/tmp/axiom-gcp-operator-evidence/0722123456 \
ACCEPTANCE_ROOT="$PWD/benchmarks/gcp-operator-acceptance" \
REGISTRY=asia-east1-docker.pkg.dev/axiom-502607/courier \
IMAGE_TAG=<git-sha>-0722123456 \
GCS_SOURCE_PREFIX=gs://axiom-502607_cloudbuild/source/axiom-gcp-operator-0722123456 \
benchmarks/gcp-operator-acceptance/scripts/cleanup.sh
```

Terraform state remains under `/tmp/axiom-gcp-operator-<run-id>/` so a failed
destroy is recoverable. The existing repository and pre-existing APIs are never
Terraform-owned and cleanup verifies that they remain.

## Evidence

Default evidence root:

```text
/tmp/axiom-gcp-operator-evidence/<run-id>/
├── run.json
├── run.log
├── images.json
├── cloud-build-submit.json
├── cloud-build-final.json
├── terraform-output.json
├── acceptance.json
├── cleanup.json
├── kubernetes/
│   ├── lumen-crs.json
│   ├── sift-crs.json
│   ├── lumen-after-split.json
│   ├── sift-collected-lumen-log.json
│   ├── sift-collector-daemonset.json
│   ├── sift-final.json
│   ├── workloads-final.json
│   ├── lumen-backup.log
│   └── sift-backup.log
└── gcs/
    ├── lumen-objects.txt
    ├── lumen-first-object.json
    ├── sift-objects.txt
    └── sift-first-object.json
```

[`evidence/schema.json`](evidence/schema.json) defines the terminal
`acceptance.json` contract. `cleanup.json` is separate and mandatory: a green
functional result without a green cleanup result is a failed run.

## Extension rule

Add a service by extending the shared builder with one runtime target, adding
its CLI-rendered layers to `render-manifests.sh`, and adding a namespace-scoped
Workload Identity member plus explicit acceptance/exclusion fields. Do not add
app manifests to Terraform, copy operator YAML into this harness, or broaden
the GSA to project-wide storage permissions.

# GCP operator acceptance: Lumen, Sift MVP, or Tape

This harness is a low-cost GKE proof for the shared service
operator shape. In full mode, it completes an independent Lumen acceptance phase first; only
then does it install Sift and its node-level collector to collect Lumen's real
structured stdout. In Tape-only mode, it proves Tape's domain-plane (Raft-based event log)
capabilities in isolation. The image, Terraform, manifest-rendering, evidence, and
cleanup boundaries remain reusable for Relay and Defer.

It keeps one named Standard GKE test cluster between runs, then creates these
short-lived resources tagged with one `RUN_ID`:

- one force-destroy GCS backup bucket with uniform bucket access, enforced
  public-access prevention, and a one-day object lifecycle;
- one shared backup GSA;
- bucket-scoped `roles/storage.objectAdmin` for that backup GSA;
- Workload Identity bindings for service account backups: `lumen/lumen-backup`
  and `sift/sift-backup` in full mode, or `tape/tape-backup` and `tape/tape`
  in Tape-only mode.

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

In Tape-only mode, the boundary is narrower:
- Tape proves 1x1 operator reconcile and standalone domain-plane lifecycle in a 3-replica
  raft cluster without any surrounding data plane (Lumen collection is outside Tape's scope).
- Event append, subscription replay, pull, and ack confirm the event log contract.
- Pod restart proves data retention on the PVC.
- GCS backup and readback prove snapshot export capability.
- Cold restore from a GCS backup object, with the #2468 bootstrap-if-empty assertion
  (pod restart with seedUri still present returns Ready and data intact).
- Failover proves raft group re-election and committed writes survive leader loss.
- The lag gauge proves subscription lag instrumentation is present in /metrics.
- CPU/memory-driven replica actuation is not claimed.
- Performance or scaling beyond a single raft group of three replicas is not claimed.

In Sift-only mode, the boundary is the Sift MVP gate:

- `ACCEPTANCE_APPS=sift` creates one run-scoped three-node `e2-standard-4` pool.
- It uses three store voters, three control voters, one gateway, one query role, and one agent per node.
- It sends 18,000,000 unique items during a 30-minute, 10,000-item-per-second load.
- It runs five more minutes at the same rate while it stops the current store leader VM.
- It checks OTLP, Remote Write, query, correlation, MCP, project isolation, PVC restart, latency, quorum, auto-repair, GCS outage behavior, 29/31/181-day boundaries, and fresh-PVC restore equality.
- Sift and Rig images must resolve to immutable digests. Rig is only the test runner.
- The cloud phase has a 90-minute hard limit. Cleanup covers the namespace, node pool, bucket, GSA, Workload Identity binding, disks, and temporary images.

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
`gcloud`, `git`, `jq`, `kubectl`, `python3`, and `terraform`.

## Exact lifecycle

Run the static gate first. It performs shell syntax checks, Terraform format,
an isolated `terraform init -backend=false`, and `terraform validate`; it does
not contact your GCP project or create resources.

```bash
acceptance/gcp/scripts/check.sh
```

Then run acceptance with the billing project explicit. Region and existing
repository have conservative defaults:

```bash
PROJECT_ID=axiom-502607 \
REGION=asia-east1 \
GKE_ZONE=asia-east1-a \
ARTIFACT_REGISTRY_REPOSITORY=courier \
acceptance/gcp/scripts/run.sh
```

`ACCEPTANCE_APPS` selects the mode and its value set is closed: `lumen sift`
(the default), `lumen auth`, `sift`, or `tape`. The earlier `LUMEN_ONLY=1` mode no longer exists —
it was removed when the harness gained Tape mode, and this section documented
it for several commits afterwards. Passing it today does nothing at all; the
run proceeds in full `lumen sift` mode, which is not what the caller asked
for. To skip Lumen's phases entirely, hand a completed Lumen proof to
`LUMEN_PRIOR_ACCEPTANCE` instead.

Either mode accepts prebuilt images. Supply them as immutable `@sha256`
digest references — a mutable tag is rejected, because an acceptance run has
to name the exact bytes it proved:

```bash
PROJECT_ID=axiom-502607 \
LUMEN_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/lumen@sha256:<digest> \
SIFT_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/sift@sha256:<digest> \
acceptance/gcp/scripts/run.sh
```

To run the dedicated Sift MVP gate, supply both candidate images by immutable
digest. This is a paid cloud operation:

```bash
PROJECT_ID=axiom-502607 \
ACCEPTANCE_APPS=sift \
SIFT_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/sift@sha256:<digest> \
RIG_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/rig@sha256:<digest> \
acceptance/gcp/scripts/run.sh
```

Omit both to build from the working tree via Cloud Build. That path requires
a **clean** tree: `run.sh` refuses to upload a dirty source, so the image can
always be traced back to a commit.

To prove Tape in isolation, select Tape-only mode and provide the exact immutable
Tape image (or omit it to trigger a local Cloud Build). This mode does not build,
render, deploy, or query Lumen or Sift; it proves only Tape's 1x1 operator
reconcile, domain-plane event append/replay/subscription lifecycle, pod restart
data retention, GCS backup readback, cold restore from backup, bootstrap-if-empty
seed survival, failover, and cleanup.

```bash
PROJECT_ID=axiom-502607 \
ACCEPTANCE_APPS=tape \
TAPE_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/tape@sha256:<digest> \
acceptance/gcp/scripts/run.sh
```

`ACCEPTANCE_APPS=tape` rejects `LUMEN_PRIOR_ACCEPTANCE`. The terminal
`acceptance.json` records only `acceptance.tape` with its complete lifecycle
proof, including the #2468 bootstrap-if-empty restart assertion and
subscription lag gauge instrumentation.

### Proving an unreleased commit: the `sha-*` dev/test line

`TAPE_IMAGE` does not have to name a release. Both Lumen (#2513) and Tape
(#2576) publish a second, internal-only image line keyed by commit rather
than version:

| line | tag | produced by | audience |
|------|-----|-------------|----------|
| release | `ghcr.io/chrischeng-c4/<app>:<semver>` + `latest` | `<app>-release.yml` | integrators |
| dev/test | `ghcr.io/chrischeng-c4/<app>:sha-<git12>` | `<app>-test-image.yml` | this harness only |

Dispatch `tape-test-image` (Actions → `tape-test-image` → *Run workflow*,
optionally naming a `ref`) and it builds both musl legs from that commit and
pushes a multi-arch `sha-<git12>` image. The run summary prints the tag
already pinned by digest; feed that to `TAPE_IMAGE` verbatim:

```bash
PROJECT_ID=axiom-502607 \
ACCEPTANCE_APPS=tape \
TAPE_IMAGE=ghcr.io/chrischeng-c4/tape:sha-<git12>@sha256:<digest> \
acceptance/gcp/scripts/run.sh
```

The harness needs no flag for this — a caller-supplied image is already
required to be an immutable `@sha256:` reference, and any such reference is
recorded as `image_provenance: prebuilt` regardless of which registry or tag
line it came from. The `sha-*` tags land on the same GHCR package as the
release line, so they inherit its public visibility and GKE pulls them
without a pull secret.

Never hand a `sha-*` tag to a user, and never let `<app>-test-image.yml` add
`latest` or a semver tag: the release line stays the only thing integrators
consume. Cutting a version bump purely to get an image into a cluster — how
tape reached 0.4.11 — is exactly what this line exists to stop.

For routine acceptance of both Lumen and Sift, pass the immutable GitHub-release-derived image
digests and no Cloud Build or staged source archive is used. A candidate can
replace just one service; the harness builds only the missing service target.

```bash
PROJECT_ID=axiom-502607 \
LUMEN_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/lumen@sha256:<digest> \
SIFT_IMAGE=asia-east1-docker.pkg.dev/axiom-502607/courier/sift@sha256:<digest> \
acceptance/gcp/scripts/run.sh
```

The first run bootstraps `axiom-operator-acceptance`; later runs reuse it. To
create it explicitly (or select a different persistent name), run:

```bash
PROJECT_ID=axiom-502607 REGION=asia-east1 GKE_ZONE=asia-east1-a \
PERSISTENT_CLUSTER_NAME=axiom-operator-acceptance \
acceptance/gcp/scripts/bootstrap-cluster.sh
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

The normal cloud portion has a hard maximum of 2,700 seconds (45 minutes).
Sift-only mode has a 5,400-second (90-minute) maximum. `EXIT`, `INT`, `TERM`,
failures, and the watchdog all enter the same cleanup trap. A
separate explicit cluster teardown is deliberately required; a normal run
never deletes the reusable cluster.

If a shell or machine failure prevents the trap from finishing, rerun cleanup
with the values printed in `run.json`:

```bash
PROJECT_ID=axiom-502607 REGION=asia-east1 GKE_ZONE=asia-east1-a RUN_ID=0722123456 \
STATE_DIR=/tmp/axiom-gcp-operator-0722123456 \
EVIDENCE_DIR=/tmp/axiom-gcp-operator-evidence/0722123456 \
ACCEPTANCE_ROOT="$PWD/acceptance/gcp" \
REGISTRY=asia-east1-docker.pkg.dev/axiom-502607/courier \
IMAGE_TAG=<git-sha>-0722123456 \
GCS_SOURCE_PREFIX=gs://axiom-502607_cloudbuild/source/axiom-gcp-operator-0722123456 \
acceptance/gcp/scripts/cleanup.sh
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
`acceptance.json` contract. Sift first writes `sift-mvp-verification.json`.
Cleanup then writes `cleanup.json`. Only the finalizer can create
`acceptance.json` after both files pass. A green functional result without a
green cleanup result is a failed run.

## Extension rule

Add a service by extending the shared builder with one runtime target, adding
its CLI-rendered layers to `render-manifests.sh`, and adding a namespace-scoped
Workload Identity member plus explicit acceptance/exclusion fields. Do not add
app manifests to Terraform, copy operator YAML into this harness, or broaden
the GSA to project-wide storage permissions.

# gke-harness — StatefulSet acceptance on the shared GKE cluster

Deploys keep / defer / relay / loom onto the shared free-tier zonal cluster
(`axiom-operator-acceptance`, `asia-east1-a`), verifies each one's StatefulSet
essence, tears the namespace down, and parks the node pool back to 0 nodes.
Driven by `.github/workflows/gke-acceptance.yml` (manual `workflow_dispatch`);
every script also runs locally with `gcloud` auth.

## What one run does

1. `scripts/ensure-cluster.sh` — creates the cluster via
   `acceptance/gcp/scripts/bootstrap-cluster.sh` only if absent (the free tier
   allows ONE zonal cluster per billing account — never create a second), then
   resizes `acceptance-pool` 0 → 1 and waits for a Ready node.
2. Per app, `scripts/run-app.sh <app>` — renders
   `apps/<app>/k8s/overlays/template` with the digest-pinned GHCR image,
   applies, waits for rollout, then runs `verify/<app>.sh`:
   readyz → app-API write + read-back → delete pod-0 → data still reads back
   after the replacement pod is Ready (PVC durability). Evidence
   (manifests, HTTP bodies, describes, logs) lands in `EVIDENCE_DIR`; the
   namespace is deleted on every exit path.
3. `scripts/park.sh` — resizes the pool back to 0. `run.sh`'s EXIT trap parks
   on every failure path; the workflow adds an `if: always()` park step for
   runner-teardown paths the trap can't catch.

Idle cost is the free-tier zonal management fee only; a full four-app run is
roughly one node-hour of `e2-standard-4` spot (a few US cents).

## One-time setup

### 1. Bootstrap (local state, run once from a workstation)

```bash
terraform -chdir=acceptance/gke-harness/bootstrap init
terraform -chdir=acceptance/gke-harness/bootstrap apply -var="project_id=<PROJECT_ID>"
```

Creates the versioned GCS tfstate bucket (default
`<PROJECT_ID>-axiom-tfstate`), the GitHub-OIDC Workload Identity pool +
provider (locked to this repository), and the deployer service account. Then
wire the repo — the apply prints the exact commands:

```bash
terraform -chdir=acceptance/gke-harness/bootstrap output -raw github_variable_commands
```

Run the four printed `gh variable set` lines
(`GCP_PROJECT_ID`, `GCP_TFSTATE_BUCKET`, `GCP_WIF_PROVIDER`,
`GCP_DEPLOYER_SA`).

### 2. Migrate the cluster tfstate /tmp → GCS (once, only if the old local state exists)

`acceptance/gcp/cluster` now declares a `gcs` backend (partial config; the
bucket comes from `-backend-config`). If `/tmp/axiom-gcp-operator-cluster`
still holds live local state, move it instead of re-importing:

```bash
cp /tmp/axiom-gcp-operator-cluster/terraform.tfstate acceptance/gcp/cluster/terraform.tfstate
TF_DATA_DIR=/tmp/axiom-gcp-operator-cluster/.terraform terraform -chdir=acceptance/gcp/cluster init -migrate-state -backend-config="bucket=<PROJECT_ID>-axiom-tfstate"
rm acceptance/gcp/cluster/terraform.tfstate
```

If the local state is already lost, skip this — `destroy-cluster.sh` prints
the six `terraform import` lines that rebuild state from the live resources.

### 3. Make the GHCR packages public (after the first image push)

The manifests carry no `imagePullSecrets`, so GKE pulls anonymously. After the
first successful build of each app, open
`https://github.com/users/<owner>/packages/container/<app>/settings` and set
visibility to Public — once per package (`keep`, `defer`, `relay`, `loom`).

## Running

GitHub → Actions → `gke-acceptance` → Run workflow. `apps` takes a
space-separated subset (default all four). First paid validation: dispatch
with `apps=keep`, check the evidence artifact and that the pool is back at 0,
then dispatch `apps=defer relay loom`.

Locally:

```bash
PROJECT_ID=<PROJECT_ID> KEEP_IMAGE=ghcr.io/<owner>/keep:sha-<git12>@sha256:... APPS=keep acceptance/gke-harness/scripts/run.sh
```

`<APP>_IMAGE` is required per selected app and should be digest-pinned.
`PARK=0` leaves the pool awake (e.g. the acceptance/gcp harness runs next).

## Local kind run (build-debug)

`scripts/run-app.sh` and `verify/` are cluster-agnostic (KUBECONFIG, kubectl,
kustomize, port-forward, and an image the nodes can pull), so the same
deploy → verify → teardown runs on a local kind cluster before anything is
pushed:

```bash
scripts/build/debug.sh keep            # docker build (cargo debug profile) → kind load → run-app.sh
scripts/build/debug.sh keep --keep     # leave the namespace up for inspection; prints the delete command
scripts/build/debug.sh keep --fresh    # recreate the kind cluster first
scripts/build/debug.sh keep --image ghcr.io/<owner>/keep@sha256:...   # deploy a prebuilt image instead
```

The kind cluster `axiom-build-debug` persists between runs (like the GKE
cluster; only the namespace is torn down). Evidence lands in
`${TMPDIR:-/tmp}/build-debug-<app>-<timestamp>/` (`docker-build.log`,
`kubeconfig`, and the same `<app>/` bundle the GHA run uploads). `run.sh`,
`ensure-cluster.sh`, and `park.sh` are GKE-only and are not used here.

## Shared-pool caution

`acceptance-pool` is shared with the acceptance/gcp lumen/sift/tape harness.
`park.sh` while one of those runs is in flight kills its nodes. v1 is
manual-dispatch only and the workflow's `concurrency: gke-acceptance` group
only serializes THIS workflow — the human dispatching arbitrates against
acceptance/gcp runs.

## Known repo oddity (harmless here)

`apps/loom/k8s/base` points `LOOM_RELAY` at
`relay.relay.svc.cluster.local:7400`, but relay's Service listens on 7000.
This harness patch-deletes `LOOM_RELAY`/`LOOM_KEEP` (template overlay), so
loom runs its in-process MemDispatcher and the mismatch never bites — but fix
the base before ever wiring loom to a real relay.

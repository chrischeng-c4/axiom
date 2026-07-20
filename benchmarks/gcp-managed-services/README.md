# GCP managed-service calibration

This bounded environment compares Tape with real Google Cloud Pub/Sub and
Defer with real Google Cloud Tasks while running Relay's durable work-queue
journey in the same GKE deployment. It is calibration evidence, not an
every-commit test. One guarded command owns provisioning, the workloads, and
teardown:

```bash
PROJECT_ID=axiom-502607 \
  bash benchmarks/gcp-managed-services/scripts/run.sh
```

For a workload-only retry, `RUN_ID` may be fresh while
`BOOTSTRAP_RUN_ID`, `BOOTSTRAP_STATE_DIR`, `IMAGE_TAG`, and `SKIP_BUILD=1`
reuse verified images. Final cleanup owns both state roots and deletes both the
fresh environment and reused bootstrap resources.

The command always traps exit, destroys both Terraform states, verifies the
cloud inventory is empty, disables the four APIs it temporarily enabled, and
deletes the local `/tmp` state containing the receiver secret. If GCP Service
Usage temporarily retains already-deleted GKE node-pool assets, cleanup proves
all billable resources absent and warns that the non-billable GKE API remains
enabled instead of waiting indefinitely. Raw JSON
evidence remains under `/tmp/axiom-gcp-bench-<run>.json`.

## Current evidence

Run `07191043` is recorded in `evidence/07191043-partial.json`. GKE accepted
all three CLI-rendered CRDs and reached 3/3 ready voters for Tape, Defer, and
Relay. All nine PVCs bound at the fixed `10Gi` size. Relay completed its
200-message replicated publish/lease/epoch-fenced-ack journey. Tape again
returned `503 raft_unavailable` during the 5,000-event durable prepare at
concurrency 64, so no Tape/Pub/Sub ratio may be claimed. Defer reached 3/3 but
its comparison stopped at a transient Cloud Run receiver `404`; receiver
warm-up now retries bounded control-plane/IAM convergence, but that fix has not
been re-run in GCP.

The earlier run `07181144` remains in `evidence/07181144-failure.json` as the
first independent observation of Tape's same concurrency-64 Raft routing
blocker. It predates Relay's inclusion and the independent comparison outcome
envelope.

The client now runs both comparisons independently and emits `passed`,
`partial`, or `failed` with a separate outcome for each pair. A future run can
form a load curve with `TAPE_PREPARE_CONCURRENCY` and
`TASK_CREATE_CONCURRENCY`; both remain 64 by default so the observed blocker
is not hidden by silently lowering pressure.

## Deployment-artifact ownership

Terraform owns only the GCP substrate and lifecycle. Tape, Defer, and Relay own
their Kubernetes deployment contracts through their `<service> k8s` CLIs: the
harness builds those CLIs before provisioning billable resources, then uses
`crd render`, `operator render`, and `instance render` for every service base.

The files under `k8s/instance-overlay/` are benchmark-only Kustomize patches.
They bound replicas, CPU, memory, and disk for the low-cost calibration; they
do not independently author the CRD, operator, or instance base. Patch
metadata only identifies the rendered custom resource being customized.
Rendered bases and assembled overlays live in the temporary Terraform state
directory and are removed during teardown. Set `TAPE_CLI` and `DEFER_CLI` only
when intentionally validating prebuilt binaries.

## Scaling boundary

The live run deliberately does not claim autonomous service autoscaling.
`service-k8s` can plan whole replica layers from CPU and memory utilization,
but the operators do not yet apply those plans through a safe Raft membership
transition. Its disk planner now requests one new shard when the busiest shard
is strictly above `1 GiB`; Tape, Defer, and Relay do not yet expose the
per-shard durable-byte signal or domain-safe migration actuator, so they still
pin `shardCount` to one and this low-cost cell fixes every PVC at `10Gi`.

The run still proves the bounded infrastructure transition from an empty
Autopilot cluster to three schedulable 3-voter services. It does not create a
fourth replica, inflate a disk, or manufacture an HPA that would churn voters.
The JSON report records this boundary so scheduling evidence cannot be
misreported as service-autoscaling evidence.

## Cost boundary

- One short-lived regional Autopilot cluster in `asia-east1`.
- Tape, Defer, and Relay each run three 500m CPU / 1Gi voters with 10Gi
  `axiom-standard-rwo` (`pd-standard`) PVCs. This benchmark-owned CSI storage
  class uses `WaitForFirstConsumer`, so disk zone affinity follows scheduling.
  Required pod anti-affinity produces three data nodes;
  the three services share those nodes.
- One benchmark Job with a 30-minute Kubernetes deadline.
- One Cloud Run receiver with `min_instances=0`, `max_instances=1`.
- 5,000 Pub/Sub messages fanned to five subscriptions by default.
- Five 200-task samples for Cloud Tasks and Defer by default.
- One 200-message Relay publish/lease/ack lifecycle by default.
- One Cloud Build invocation. The 8-vCPU builder reduces elapsed cluster-free
  preparation time; it does not keep a worker alive after the build.

Terraform budgets are alerts rather than a hard kill switch. The actual cost
guard is the bounded topology, small corpus, workload deadline, and mandatory
destroy. The GKE cluster management fee is normally covered by the billing
account's one-cluster monthly free-tier credit, but pod, disk, build, and
network usage can still be billable.

## Fairness contract

The benchmark client runs inside the same GKE region as both managed services.
Pub/Sub uses five named at-least-once subscriptions and `asia-east1`-only
message persistence. Tape uses five pull subscriptions and advances each
checkpoint explicitly. Both prepare the same logical backlog outside the
drain latency sample.

Cloud Tasks and Defer use identical per-task creates, a 500 dispatch/s and 100
concurrent dispatch queue, the same payload, and the same Cloud Run HTTP 204
receiver. A sample ends only after every unique receipt is observed and the
provider queue or Defer terminal state confirms completion.

The report can compare client-observed throughput, p50/p95/p99, errors,
duplicates, client CPU/RSS, and estimated API cost. GKE pod/PVC resources are
captured separately. Pub/Sub and Cloud Tasks internal CPU, RSS, replication,
and disk amplification are `provider_opaque`; the report must never invent or
compare those values.

This first low-cost calibration disables Axiom request auth and peer TLS inside
the private cluster. Any public production claim must add a security-on cell
and treat the current result as report-only.

## Terraform ownership

`bootstrap/` temporarily enables `container.googleapis.com`,
`cloudtasks.googleapis.com`, `file.googleapis.com`, and `sts.googleapis.com`, then creates one
Artifact Registry repository. `environment/` owns the Autopilot cluster,
service accounts, short-lived IAM bindings, Pub/Sub topic/subscriptions, Cloud
Tasks queue, and Cloud Run receiver. Teardown is deliberately ordered:

1. Destroy `environment`.
2. Verify GKE, Cloud Run, Cloud Tasks, Pub/Sub, PD, and addresses are absent.
3. Destroy `bootstrap`.
4. Verify Artifact Registry is absent and temporary APIs are disabled.

If the host itself is forcibly terminated, rerun cleanup with the preserved
state directory:

```bash
PROJECT_ID=axiom-502607 REGION=asia-east1 RUN_ID=<run> \
STATE_DIR=/tmp/axiom-gcp-bench-<run> \
BENCH_ROOT="$PWD/benchmarks/gcp-managed-services" \
REGISTRY=<bootstrap-output> IMAGE_TAG=<run-tag> \
  benchmarks/gcp-managed-services/scripts/cleanup.sh
```

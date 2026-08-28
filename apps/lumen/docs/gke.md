# Run Lumen on GKE

## Purpose

This guide owns Lumen's environment support tiers and its first Managed
production profile. Local development uses Standalone. The first production
target is Fleet-managed Lumen on GKE Standard Regional.

This guide separates three independent choices:

- Fleet selects which runtimes the operator manages.
- Shards select how search data is divided.
- Replicas and voters select failure tolerance inside each shard.

None of these choices silently enables another one.

## Standalone GKE instance

Standalone GKE uses the shared `StatefulInstancePlan` boundary to render one
StatefulSet plus a separately owned PVC instance. It
is not Managed or Fleet adoption, HA, TLS, Ingress, LoadBalancer, or general
Kubernetes support. The public `lumen.yaml` has only `name`, `namespace`,
`nodePool`, `cpu`, `memory`, `storageSize`, `storageClass`, and
`allowedServiceAccounts`; it has no image field.

Initialize and render the instance locally, then inspect the generated storage
and runtime directories before applying them:

```bash
lumen standalone gke init --out lumen.yaml
lumen standalone gke render --file lumen.yaml --out lumen-dist
kubectl apply -k lumen-dist/storage
kubectl apply -k lumen-dist/runtime
```

The output root may be absent, empty, or an existing Lumen-managed root. A
non-empty unmanaged root is refused. The public configuration has exactly these
eight fields and no image field. `name` and `namespace` default to `lumen`.
`storageSize` and `storageClass` default to `20Gi` and `premium-rwo`.
`nodePool`, `cpu`, `memory`, and a nonempty `allowedServiceAccounts` list are
required:

```yaml
name: lumen
namespace: lumen
nodePool: data-pool
cpu: 500m
memory: 512Mi
storageSize: 20Gi
storageClass: premium-rwo
allowedServiceAccounts:
  - apps/my-client
```

An app uses `LUMEN_URL=http://<name>.<namespace>.svc.cluster.local:7373`;
the concrete default is `http://lumen.lumen.svc.cluster.local:7373`. The
client reads the mounted token for the calling Pod's configured KSA, not an SA
named `default`. The server checks that KSA against `allowedServiceAccounts`.

The Standalone GKE live acceptance is controller-run, manual, and paid. It is
separate from the existing Managed/operator GCP acceptance. Candidate CI does
not claim GKE or `gcloud` coverage. Before the `lumen@0.4.29` release, the
controller must run this gate against the exact candidate. It checks the
Standalone `LUMEN_AUTH=in-cluster` matrix and the inherited Managed
`LUMEN_AUTH=required` continuity matrix: a projected `lumen.axiom.dev` token is
allowed, the same KSA's default token returns `401`, and a projected unlisted
KSA returns `403`. It also checks cleanup and writes a sanitized receipt only
after every check passes. The gate has not passed until the controller records
that receipt. Its private cluster inputs and exact execution remain
controller-owned.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| Current environment support | [Lumen status](../STATUS.md#support-matrix) | Read each environment and topology row before choosing a profile. |
| Future GKE, placement, rollout, and certificate outcomes | [Lumen roadmap](../ROADMAP.md) | Follow the named outcome from a Limited or Not supported STATUS row. |
| Install and upgrade order | [Deployment](deployment.md) | Use the commands for the selected current mode. |
| Current Kubernetes API | Generated Lumen CRDs | `lumen k8s crd render --out <directory>` |
| Current GCP substrate | [Installation Terraform](../terraform/README.md) | Read the current capacity-catalog and PKI boundaries. |
| Current GKE acceptance evidence | [GCP acceptance](../../../acceptance/gcp/README.md) | `bash acceptance/gcp/scripts/check.sh` |

## Support tiers

| Environment | Current support | Intended use |
|---|---|---|
| Bare binary or local container | Supported Standalone path. | Local development, tests, and trusted single-process use. |
| Direct Kubernetes Kustomize overlay | Supported compatibility path with one in-memory Standalone process. | Cluster-local development only. It is not Managed or production. |
| GKE Standard zonal | Limited acceptance environment. | Current operator, persistence, backup, placement, and selected lifecycle proof. It is not regional HA evidence. |
| GKE Standard Regional | Target first Managed production profile. It is not certified today. | Small persistent runtimes through multi-shard, replicated production runtimes. |
| GKE Autopilot | Not certified. | A later target after the portable placement contract no longer needs current node-pool control. |
| Other Kubernetes distributions | Not certified. | The public target remains Kubernetes-native, but support needs its own storage, networking, identity, and failure gates. |

Google recommends Autopilot for most workloads. Standard is the correct first
Lumen profile because Lumen currently needs granular stateful node, storage,
and placement control. The target cluster is Regional so its control plane and
node pools can span zones.

- [Choose a GKE mode](https://docs.cloud.google.com/kubernetes-engine/docs/concepts/choose-cluster-mode)
- [Create a regional cluster](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/creating-a-regional-cluster)

## Runtime size and topology

A Managed runtime declares a fixed topology. The total serving Pod count is:

```text
shardCount * replicasPerShard
```

| Shape | Meaning |
|---|---|
| `1 shard × 1 replica` | Smallest persistent Managed runtime. One Pod or one zone failure interrupts service. This is not HA. |
| `N shards × 1 replica` | More search capacity. Each shard still has one copy and no shard-level redundancy. |
| `N shards × 3 replicas`, `3 voters` | Production HA target. Each shard keeps quorum after one replica or one zone is unavailable. |
| `2 replicas` | Compatibility shape. A two-voter group loses write quorum when either voter is unavailable, so it is not the production recommendation. |

`shardCount` is the capacity dimension. `replicasPerShard` and `voterCount` are
the availability dimension. Fleet is the management dimension.

The Kubernetes node autoscaler may create nodes for Pending Pods. It does not
change `shardCount`, `replicasPerShard`, voter membership, or the total number
of Lumen Pods. Automatic Lumen membership changes remain separate future work.

## Kubernetes-native contract

The target public Lumen API describes workload intent with Kubernetes concepts:

- CPU and memory requests;
- PVC size and StorageClass;
- node selectors and tolerations;
- failure-domain topology intent; and
- fixed shard, replica, and voter topology.

The target API does not require a GCE machine type, GKE node-pool name, or
ComputeClass object. A GKE installation profile maps the portable intent to
platform configuration. Other Kubernetes platforms can supply a different
profile without changing Lumen's search contract.

Current Managed reconciliation has a bounded compatibility path. A non-empty
`placement.nodeSelector` with the default machine type renders that selector
and its tolerations directly, without reading the catalog. Empty selectors,
tolerations-only placement, and non-default machine types still read
`lumen-system/lumen-capacity-catalog` and resolve `placement.initialMachineType`.
These fields remain legacy compatibility inputs. Existing manifests must keep
materializing while new manifests move to the full portable placement contract.

Cluster, namespace, ComputeClass, node-pool, StorageClass, issuer, bucket, and
monitoring-backend lifecycle stays with the platform. Lumen owns the search
workload, placement intent, health meaning, backup and restore operations, and
search-aware rollout decisions inside the supplied cluster.

## GKE Standard Regional profile

The first certified production profile will require:

- one GKE Standard Regional cluster with nodes available in three zones;
- private nodes and a private ClusterIP Lumen serving endpoint;
- GKE Dataplane V2 and enforced Kubernetes NetworkPolicy;
- Workload Identity Federation for workloads that call Google APIs;
- a stateful ComputeClass or equivalent node capacity profile;
- a `WaitForFirstConsumer` StorageClass for zonal persistent volumes;
- resource requests that represent the actual serving working set;
- an operator certificate issuer and CA policy;
- a backup bucket and restore exercise; and
- immutable image digests for production acceptance.

The GKE platform owns the ComputeClass and node auto-provisioning policy. A
Lumen Pod selects that class through a standard node selector. Active migration
must be disabled for the stateful class because infrastructure-driven Pod
migration can disrupt data-bearing workloads.

- [GKE ComputeClass best practices](https://docs.cloud.google.com/kubernetes-engine/docs/best-practices/computeclasses)
- [GKE networking best practices](https://docs.cloud.google.com/kubernetes-engine/docs/best-practices/networking)

One regional cluster can run both the one-Pod profile and larger replicated
profiles. A regional cluster does not make a one-Pod runtime highly available.
The runtime topology still decides data-plane availability.

## Storage, placement, and disruption

The production placement target applies inside each shard group:

- replicas of one shard use hard hostname anti-affinity;
- a three-voter shard uses three zone failure domains;
- different shards and different runtimes may share a node when their resource
  requests fit; and
- all Pod labels must match the selectors used by their topology constraints.

The current renderer does not meet this target. It uses cross-namespace,
one-Lumen-Pod-per-node anti-affinity and soft whole-runtime zone spread. The
first rule is broader than one shard. The second rule cannot prove that every
three-voter shard occupies three zones.

A PodDisruptionBudget protects voluntary eviction. It does not limit a
StatefulSet controller's own rolling update. The target therefore uses two
controls:

- a quorum-safe PDB for drains and other eviction-based maintenance; and
- an operator-driven runtime rollout that updates one member, waits for the
  new generation, searchable readiness, quorum, and replication convergence,
  and only then advances.

Fleet rollout remains the outer control. It decides which runtime changes next.
Runtime rollout is the inner control. It decides which Raft member changes
next.

- [Kubernetes topology spread constraints](https://kubernetes.io/docs/concepts/scheduling-eviction/topology-spread-constraints/)
- [Kubernetes disruptions and PDB behavior](https://kubernetes.io/docs/concepts/workloads/pods/disruptions/)

Raft replication is not a backup. Production acceptance also needs a backup,
restore, and readback proof against a separately owned destination.

## Identity and networking

Two short-lived identity flows have different audiences and authorities:

| Identity flow | Purpose | Authority |
|---|---|---|
| Projected KSA token sent to Lumen | Authenticate one application workload to one Managed runtime. | Kubernetes TokenReview and SubjectAccessReview. |
| Workload Identity Federation | Let the operator, certificate controller, or backup workload call Google APIs without a service-account key. | Google IAM and the target Google API. |

The first flow never becomes a Google access token. The second flow never
becomes the bearer token sent to the Lumen search API.

Managed serving stays private by default. The operator creates a ClusterIP
Service. It does not create a LoadBalancer, Ingress, public DNS record, or
internet edge. NetworkPolicy is defense in depth. KSA authentication and RBAC
remain the request identity boundary because NetworkPolicy cannot authorize one
specific ServiceAccount.

The target operator derives separate serving and peer certificate identities,
requests and rotates their leaf certificates through a platform-supplied
issuer, and stores private material in runtime Secrets. It publishes only the
public client trust bundle into allowed client namespaces. See
[authentication](authentication.md) and [client integration](client-integration.md).

## Verification

The current static GCP gate is:

```bash
bash acceptance/gcp/scripts/check.sh
```

The current live harness uses a named zonal Standard cluster. Its documented
proof remains valuable, but it does not certify the target regional profile.

The future regional gate must prove at least:

- one `1 × 1` persistent runtime and one `1 × 3` runtime;
- more than one shard with three voters per shard;
- distinct hostname and zone placement for each shard's voters;
- no false scheduling conflict between different shards or runtimes;
- cluster node scale-up for declared Pending Pods without changing Lumen
  membership;
- Pod loss, node drain, and one-zone loss with expected read and write results;
- one-member-at-a-time runtime rollout, interruption, and recovery;
- serving and peer certificate issue, rotation, overlap, and hot reload;
- KSA access, CA distribution, and NetworkPolicy behavior; and
- backup, restore, ordered search results, and source-ID readback.

Passing local renderer or zonal acceptance tests must not promote the STATUS
row for the regional profile.

## Current boundaries

- GKE Standard Regional is the selected production target, not a current
  support claim.
- The current live GKE harness is zonal.
- Legacy Managed placement cases require the GCE-specific capacity catalog.
- The bounded native compatibility path supports a non-empty selector with the default machine type.
- Current placement does not prove per-shard three-zone distribution.
- Current PDB settings do not make application rollouts quorum-aware.
- Current leaf certificates and Secrets are supplied outside Fleet.
- Current Fleet materialization does not prove child runtime readiness.
- Membership-aware replica autoscaling and replicated shard expansion remain
  later work.
- GKE Autopilot and other Kubernetes distributions need separate acceptance
  evidence before they can be called certified.

## Supporting documents

- [Lumen README](../README.md)
- [Architecture](../ARCHITECTURE.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
- [Deployment](deployment.md)
- [Configuration](configuration.md)
- [Authentication](authentication.md)
- [Client integration](client-integration.md)
- [Operator runbook](runbooks/operator-control-plane.md)
- [Installation Terraform](../terraform/README.md)
- [GCP acceptance](../../../acceptance/gcp/README.md)

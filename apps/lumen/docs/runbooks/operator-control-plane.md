# Runbook — lumen operator control plane

On-call procedures for the two alerts shipped by the `operator-monitoring`
component (`apps/lumen/k8s/components/operator-monitoring/prometheusrule.yaml`).
Both watch the **control plane** — the `lumen-operator` Deployment in
`lumen-system` — not a `Lumen` instance. A serving StatefulSet keeps answering
reads from its last-reconciled state while nothing reconciles, so neither alert
is preceded by a user-visible symptom. That silence is the reason they exist.

**Scrape target**: Service `lumen-operator-metrics` in `lumen-system`, port
`metrics` (9090), path `/metrics`. Every replica is scraped separately — the
leader gauge is per-replica, so a collapsed target would make a lease handover
invisible.

**Published series** (from `libs/service-k8s/src/metrics.rs`, prefixed from the
service's `MANAGER`):

| Series | Type | Meaning |
|---|---|---|
| `lumen_operator_reconcile_total` | counter | Reconcile attempts on this replica, successful or not. Only the leader increments it; a follower's no-op is deliberately excluded. |
| `lumen_operator_reconcile_errors_total` | counter | Attempts that returned an error. Also emits a `Warning` Event on the CR. |
| `lumen_operator_reconcile_duration_seconds` | histogram | Wall-clock duration of one reconcile. |
| `lumen_operator_leader` | gauge | `1` on the replica holding the leader-election Lease, `0` on the standby. Read at scrape time, so an idle cluster's handover is still visible. |

---

## `LumenOperatorAbsent`

> No lumen operator replica is being scraped. Nothing is reconciling `Lumen`
> custom resources.

**Why it is written on `up`**: a threshold on any `lumen_operator_*` counter
cannot fire when the pod is gone, because the series stops existing. The rule
also carries an `absent()` arm so that a scale-to-zero — which removes the
Endpoints and therefore the `up` series itself — still pages.

### Triage

```bash
# 1. Does the Deployment want any replicas at all?
kubectl -n lumen-system get deploy lumen-operator

# 2. Are the pods scheduled, and if not, why?
kubectl -n lumen-system get pods -l app.kubernetes.io/name=lumen-operator -o wide
kubectl -n lumen-system describe pod -l app.kubernetes.io/name=lumen-operator | tail -40

# 3. If pods are Running but not scraped, the target — not the process — is broken.
kubectl -n lumen-system get endpoints lumen-operator-metrics
kubectl -n lumen-system port-forward deploy/lumen-operator 9090:9090 &
curl -s localhost:9090/metrics | head
```

### Causes, in the order they actually occur

| Observation | Cause | Action |
|---|---|---|
| `READY 0/2`, pods `CrashLoopBackOff` | Bad image, missing RBAC, unreachable apiserver | `kubectl -n lumen-system logs deploy/lumen-operator --previous` |
| `READY 0/0` | Deliberately scaled to zero, or a rollout that never scaled back | `kubectl -n lumen-system scale deploy/lumen-operator --replicas=2` |
| Pods `Pending` | No schedulable node; anti-affinity is `preferred`, so this is real pressure, not the affinity rule | `kubectl -n lumen-system describe pod <pod>` and read the scheduler event |
| Pods `Running`, `Endpoints` empty | Service selector no longer matches the pod labels | Compare `service.yaml`'s `spec.selector` with the Deployment's pod template labels |
| Pods `Running`, endpoints present, still no target | `ServiceMonitor` not selected by this Prometheus | Check the `release:` label matches your Prometheus's `serviceMonitorSelector` |
| Metrics port refuses the connection, but reconciles are happening | The listener failed to bind and the operator continued without it (by design — losing `/metrics` must not stop reconciliation) | `kubectl -n lumen-system logs deploy/lumen-operator \| grep "metrics listener"`; check for a port conflict or an `OPERATOR_METRICS_ADDR` override |

### Verify recovery

```bash
kubectl -n lumen-system get pods -l app.kubernetes.io/name=lumen-operator
# Exactly one replica should report leadership:
kubectl -n lumen-system port-forward deploy/lumen-operator 9090:9090 &
curl -s localhost:9090/metrics | grep lumen_operator_leader
```

The alert returns to `inactive` within one or two scrape intervals of the
target coming back.

---

## `LumenOperatorReconcileErrorRate`

> More than 10% of the leader's reconcile attempts have failed over the last
> 15 minutes.

**Why it is a ratio**: an operator watching three CRs runs very few reconciles,
so an absolute error rate would page on a single transient conflict. The rule
also requires a non-trivial attempt rate in the denominator, so a nearly idle
operator cannot reach the threshold on one failure.

### Triage

The failures are narrated on the objects themselves — the operator publishes a
`Warning` / `ReconcileFailed` Event per failed attempt, which is the fastest
path to *which* CR is failing and *why*:

```bash
# Every reconcile failure, newest last, across all namespaces.
kubectl get events -A --field-selector reason=ReconcileFailed \
  --sort-by=.lastTimestamp

# The operator's own view of the same failures.
kubectl -n lumen-system logs deploy/lumen-operator | grep "reconcile failed"

# Which replica is actually doing the work (errors only accrue on the leader).
kubectl -n lumen-system get lease -l app.kubernetes.io/name=lumen-operator
```

Events are deduplicated into an `EventSeries` over a 6-minute window, so a
tight failure loop shows as one Event with a rising `count`, not thousands of
rows. A `count` climbing while the alert is firing confirms the loop is live
rather than historical.

### Common causes

| Event `note` contains | Cause | Action |
|---|---|---|
| `is forbidden` / `cannot create` | Missing RBAC for a kind the operator renders | Reconcile `k8s/operator/rbac.yaml` against the kinds in `apps/lumen/src/operator/render.rs`; a newly rendered kind needs a new grant |
| `no matches for kind "ServiceMonitor"` / `"PrometheusRule"` | `spec.observability: true` on a cluster with no prometheus-operator CRDs | Install the CRDs, or set `spec.observability: false` on the CR |
| `the object has been modified` | Optimistic-concurrency conflict | Self-clearing; if it persists, something else is writing the same object — look for a second operator or a GitOps controller fighting over it |
| `connection refused` / `context deadline exceeded` | Apiserver unreachable or throttling | Check apiserver health and the operator's client-side rate limits |
| `Invalid value` on a child object | The CR asks for something the cluster rejects (e.g. a shrink on an immutable field) | `kubectl get lumen -A -o yaml` and compare the offending field with the child object's current state |

### Verify recovery

```bash
# The ratio the alert reads, evaluated by hand:
#   rate(lumen_operator_reconcile_errors_total[15m])
#     / rate(lumen_operator_reconcile_total[15m])
kubectl -n lumen-system port-forward deploy/lumen-operator 9090:9090 &
curl -s localhost:9090/metrics | grep -E 'reconcile_(total|errors_total)'
```

Both counters are monotonic, so watch them across two samples: the total must
keep climbing while the error counter stops. A frozen *total* is a different
failure — reconciles have stopped entirely, and `LumenOperatorAbsent` is the
alert to expect next.

---

## Managed Fleet materialization

The Fleet loop runs under its own `lumen-fleet` Lease. It scans every
`LumenFleet` every 30 seconds. Its status describes child-resource
materialization. It does not describe child runtime readiness.

```bash
kubectl get lumenfleet
kubectl get lumenfleet <fleet> -o yaml
kubectl get lumen -A -l lumen.dev/fleet=<fleet>
kubectl -n lumen-system get lease lumen-fleet -o yaml
kubectl -n lumen-system logs deploy/lumen-operator | grep "fleet"
```

Read each entry state literally:

| Entry state | Current meaning |
|---|---|
| `Created` | The child `Lumen` object was created in this pass. |
| `Applied` | The child `Lumen` object accepted the steady-state apply. |
| `Rejected` | The merged `LumenSpec` was invalid or named an unknown field. |
| `NamespaceMissing` | The target namespace does not exist. Fleet will not create it. |
| `NotAdopted` | A child with that name exists without this Fleet's ownership label. It was left untouched. |
| `ApplyFailed` | The Kubernetes create, apply, or prune call failed. |
| `Orphaned` | The child is no longer declared and `prunePolicy: Retain` kept it. |
| `Pruned` | `prunePolicy: Delete` requested deletion of the child object. This is not proof that its PVCs were deleted. |

An entry can say `Applied` while its child has `Ready=False`. Always continue
to the child-status checks below.

Some Kubernetes API errors can end one Fleet status pass before later entries
are refreshed. If status is stale, check the operator log and the Fleet Lease.
The next leader pass retries after 30 seconds.

## Managed authentication

Current Fleet materialization does not create client access RBAC. A Fleet entry
can be `Applied` while every protected request is denied.

First check the runtime's delegated-auth condition:

```bash
kubectl -n <namespace> get lumen <name> \
  -o jsonpath='{.status.conditions[?(@.type=="AuthDelegationReady")]}'
kubectl -n <namespace> get statefulset <name> \
  -o jsonpath='{.spec.template.spec.serviceAccountName}'
```

`AuthDelegationReady=False` means the runtime identity cannot complete its
TokenReview or SubjectAccessReview duty. Check the operator-owned
auth-delegator binding and ask Kubernetes about the exact runtime identity:

```bash
kubectl get clusterrolebinding \
  -l app.kubernetes.io/managed-by=lumen-operator
kubectl auth can-i create tokenreviews.authentication.k8s.io \
  --as=system:serviceaccount:<namespace>:<runtime-sa>
kubectl auth can-i create subjectaccessreviews.authorization.k8s.io \
  --as=system:serviceaccount:<namespace>:<runtime-sa>
```

If delegation is ready, inspect the current client bundle and the permission
that Lumen asks about:

```bash
kubectl -n <namespace> get serviceaccount,role,rolebinding \
  -l app.kubernetes.io/component=access
kubectl auth can-i get lumencollections.lumen.axiom.dev/<collection> \
  --namespace <namespace> \
  --as=system:serviceaccount:<client-namespace>:<client-sa>
```

A client pod that names a ServiceAccount still needs a projected token for
audience `lumen.axiom.dev`. Its HTTP client must add that token to the
Authorization header. Current generated clients do not read or rotate the
standard projected token automatically.

Interpret request failures as follows:

| Result | Current meaning |
|---|---|
| `401` | The Authorization header is missing, malformed, expired, for the wrong audience, or not a ServiceAccount identity. |
| `403` | TokenReview accepted the ServiceAccount, but SubjectAccessReview denied the current per-collection or instance-admin resource. |
| `503` from an auth decision | Kubernetes review transport or response validation failed. Lumen does not fall back to anonymous. |

`AccessPolicyReady` does not exist yet. The planned whole-runtime
`lumenruntimes/use` Role and RoleBinding are also not implemented. Do not use a
missing future condition to diagnose current access. See
[authentication](../authentication.md) for current resource mapping and the
planned contract.

When that target lands, one `use` grant will allow query, index,
collection-management, and admin requests for the complete runtime. It will not
be a fine-grained permission. `ClientTrustReady` and operator-published client
CA ConfigMaps are also future conditions. Do not diagnose them as current
objects.

## Capacity catalog

Every current Managed reconcile reads this ConfigMap:

```bash
kubectl -n lumen-system get configmap lumen-capacity-catalog
kubectl -n lumen-system get configmap lumen-capacity-catalog \
  -o jsonpath='{.data.catalog\.json}'
```

The `catalog.json` value must contain a compatible entry for the child's
`spec.placement.initialMachineType`. Check the child and the catalog together:

```bash
kubectl -n <namespace> get lumen <name> \
  -o jsonpath='{.spec.placement.initialMachineType}'
kubectl -n lumen-system describe configmap lumen-capacity-catalog
```

A missing, malformed, draining, full, or incompatible catalog stops the child
reconcile before workload apply. Current code does not provide a plain
Kubernetes fallback. Reconcile the Terraform capacity module or restore the
expected ConfigMap. Do not hand-edit a generated catalog while Terraform still
owns it.

This catalog is the current legacy placement path. Kubernetes-native placement
and the GKE Standard Regional profile are not implemented. Do not treat the
current zonal GKE acceptance result as regional HA evidence. See the
[GKE guide](../gke.md) for the exact support tiers.

## Child runtime status

Fleet status is not the readiness source. Inspect each materialized child:

```bash
kubectl -n <namespace> get lumen <name> -o yaml
kubectl -n <namespace> get statefulset,pod,service,pvc
kubectl -n <namespace> describe lumen <name>
kubectl -n <namespace> get events --sort-by=.lastTimestamp
```

Current child conditions include `Ready`, `Progressing`,
`ReshardInProgress`, `AuthDelegationReady`, and `PeerIdentityReady`. A
replicated instance also needs the named peer TLS Secret before its pods can
start securely.

The current operator consumes pre-created serving and peer Secrets. It does not
request or rotate leaf certificates, and it does not publish a public CA
ConfigMap for client workloads. Those duties are roadmap outcomes.

`Ready=True` means the current control plane considers the data plane
searchable. It is not a complete writable or capacity verdict. A pod in
storage-full read-only mode can keep `/readyz` at 200. Check the metric when
writes fail with `507 storage_full`:

```bash
kubectl -n <namespace> port-forward service/<name> 7373:7373
curl -fsS http://127.0.0.1:7373/metrics | grep lumen_storage_degraded
```

---

## Related

- `libs/service-k8s/src/controller.rs` — the reconcile loop, leader gate, and Event publication.
- `libs/service-k8s/src/metrics.rs` — the metric definitions and the `/metrics` listener.
- `apps/lumen/k8s/components/observability/` — the **instance** (data-plane) alerts, a separate concern from this runbook.
- `apps/lumen/docs/deployment.md` — Standalone and Managed install flow.
- `apps/lumen/docs/gke.md` — current GKE evidence and the regional production target.
- `apps/lumen/docs/client-integration.md` — current and planned client workload responsibilities.

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

## Related

- `libs/service-k8s/src/controller.rs` — the reconcile loop, leader gate, and Event publication.
- `libs/service-k8s/src/metrics.rs` — the metric definitions and the `/metrics` listener.
- `apps/lumen/k8s/components/observability/` — the **instance** (data-plane) alerts, a separate concern from this runbook.

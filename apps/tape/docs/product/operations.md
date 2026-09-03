# Operations

What an operator gets from a tape deployment: backup and restore, grants and
admission, Kubernetes assets, probes and telemetry, and the one performance
gate. This area spans the README capabilities `backup-and-seed`,
`security-hardening`, `kubernetes-native-deployment`,
`operations-observability`, and `local-performance-ceiling`.

## Whole-journal backup and cold seed

- Problem: none open as shipped; the limits below belong to the rebaseline.
- Who: operators.
- Promise: `GET /admin/backup` streams a whole-journal snapshot and requires
  the admin role when auth is required. `tape backup` ships it to a
  `file://`, `s3://`, or `gs://` destination with retention pruning, and the
  README, the runbook, the CLI help, and the LLM topic list every accepted
  scheme. `--bootstrap-seed-uri` restores it into an empty data directory
  only, never over existing data. The backup audit record is redacted.
- Limits today: only the `file://` sink is proven end to end in this
  repository; the cold-restore runbook lives in the deployment handoff page
  rather than under runbooks.
- Non-goals: export subscriptions; a subscription snapshot (that is
  [retention-seek-and-snapshots.md](retention-seek-and-snapshots.md)).
- Neighbours: none; first section of the area.
- Status rows: `backup-to-sink`, `cold-seed-bootstrap`, `management-audit`.

## Grants and bounded admission

- Problem: none open as shipped; the limits below belong to two outcomes.
- Who: operators issuing tokens; every caller under `--auth required`.
- Promise: with `--auth required`, append needs a write grant on the topic,
  reads need a read grant, backup needs the admin role, probes stay
  tokenless, and `--auth off` keeps every route tokenless. The body limit is
  enforced per request, and append is classified as write admission for the
  shared bounded-admission mechanism.
- Limits today: grants are per topic, not per subscription (closed by
  [subscriptions.md](subscriptions.md) § Subscription ack and competing
  subscribers); the default router keeps admission disabled and there is no
  per-topic or per-subscription quota (closed by Quotas and scale transition
  below).
- Non-goals: identity federation; the token registry is the shared library's.
- Neighbours: none within the area.
- Status rows: `per-topic-authorization`, `flow-control-quotas`.

## Kubernetes operator and direct install

- Problem: none open as shipped; the limits below belong to three outcomes.
- Who: operators.
- Promise: a `Tape` custom resource renders and reconciles a StatefulSet,
  Services, PodDisruptionBudget, ConfigMap, backup CronJob, and observability
  pair with status conditions; auth defaults to required; stale objects are
  pruned. A direct-install base deploys a durable singleton. Topics and
  subscriptions are provisioned declaratively. The kind script and the GKE
  acceptance path exercise the assets end to end.
- Limits today: `shardCount` is fixed at 1 and a replica-count change
  restarts members; the kind and GKE runs are manual, read back through the
  legacy routes, and prove no scale transition.
- Non-goals: a regional GKE profile; managed-service billing.
- Neighbours: extended by
  [replication-and-availability.md](replication-and-availability.md)
  § Live replica membership and § Multi-shard topology.
- Status rows: `k8s-deployment-assets`, `k8s-operator`,
  `kind-cluster-acceptance`, `gke-zonal-acceptance`.

## Health, metrics, traces, and drain

- Problem: none open as shipped.
- Who: operators and their alerting.
- Promise: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs`
  on the data-plane port; readiness flips to 503 on drain; request counters,
  latency sums, topic offset and subscription lag gauges; OTLP traces; a
  bounded stability run survives repeated restarts without losing history.
- Limits today: no oldest-unacked-age, delivery-attempt, or dead-letter
  counter, because there is no per-message delivery state (closed by the
  subscription outcome).
- Non-goals: a metrics surface other than Prometheus text exposition.
- Neighbours: none within the area.
- Status rows: `standard-operational-endpoints`, `otlp-tracing`,
  `bounded-stability-run`.

## Local performance ceiling

- Problem: none open as shipped.
- Who: operators sizing a node; the repository, as a regression gate.
- Promise: append, replay, and checkpoint stay inside the release-mode budget
  measured against tape's own baseline, durable append throughput rises with
  connection count, and tape never claims a win over another broker.
- Non-goals: any figure against Kafka, JetStream, or another broker.
- Neighbours: none within the area.
- Status rows: `local-performance-ceiling`.

## Quotas and scale transition (Milestone #126)

- Problem: nothing bounds a single tenant's topics, subscriptions, or bytes,
  and no run proves a replica-count change under load.
- Who: operators running more than one team on one instance.
- Promise: per-topic and per-subscription quotas as counts and bytes per
  window, refused with the shared error envelope; the default router enables
  bounded admission; the GCP acceptance script proves a replica-count
  transition under load with no committed loss, zonal.
- Non-goals: billing-style accounting; a regional profile.
- Open: the quota dimensions and defaults; whether quotas are set on the
  custom resource or through the API.
- Neighbours: extends Grants and bounded admission and Kubernetes operator
  and direct install; depends on
  [replication-and-availability.md](replication-and-availability.md)
  § Live replica membership for the transition it proves.
- Outcome: `quotas-and-scale-transition`. Tracking: [Milestone #126](https://github.com/chrischeng-c4/axiom/milestone/126)

## Non-goals in this area

- `peer-broker-benchmarks`: the earlier NATS JetStream and Kafka
  calibrations are history in `docs/benchmarks-scale.md`, not a claim.
- `export-subscriptions`: backup is disaster recovery only.

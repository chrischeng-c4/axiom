---
change: scheduler-runtime-complete
group: scheduler-runtime-complete
date: 2026-03-27
---

# Requirements

1. Push receiver HTTP endpoint — shared by Cloud Scheduler and K8s CronJob. Receives scheduled trigger callbacks, validates authentication (OIDC for GCP, ServiceAccount token for K8s), resolves task name from request, calls broker.publish(queue, message). Must be an axum Router mountable on the existing server.
2. K8sCronJobBackend implementing SchedulerBackend trait — CRUD K8s CronJobs via K8s API (create/update/delete/pause/resume). Reuse existing executor/k8s.rs infrastructure. Feature-gated as `k8s-scheduler`.
3. Schedule monitor — tracks expected_at (computed from cron/interval) vs actual_at (when callback received). Configurable leeway duration per task. Detects missed/late/on-time status. Emits metrics via existing metrics infrastructure: counter `scheduler_task_fire_total{status}`, histogram `scheduler_task_latency_seconds`.
4. PeriodicScheduler::start() mode selection — self-hosted backends (Ion/Memory) start internal tick loop; external backends (CloudScheduler/K8sCronJob) start HTTP push receiver server instead.
5. Upstream integration: each scheduler type registers tasks differently — self-hosted adds to in-memory task list, Cloud Scheduler creates GCP jobs via API, K8s creates CronJob resources. Registration must be unified through PeriodicScheduler::add_task().
6. Downstream integration: all trigger paths (tick loop or HTTP callback) converge to broker.publish(). Monitor hooks into both paths to record actual_at. Metrics/tracing integration uses existing cclab-queue metrics module.
7. Authentication design: push receiver validates Cloud Scheduler OIDC tokens (verify JWT against Google public keys, check audience), K8s ServiceAccount tokens (verify via TokenReview API or shared secret), and rejects unauthenticated requests.

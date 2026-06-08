---
change: gcp-cloud-integration
group: gcp-cloud-integration
date: 2026-03-26
---

# Requirements

1. Implement GCP Cloud Scheduler as a new SchedulerBackend implementation, offloading cron/interval scheduling to GCP Cloud Scheduler service instead of self-hosted leader election loop.
2. Complete the GCP Cloud Tasks broker in broker/cloudtasks.rs — replace stub publish() with actual HTTP API calls to the Cloud Tasks REST API (POST to create tasks, health check via GET).
3. Both integrations must support OIDC authentication for secure GCP API access.
4. Both must be feature-gated (Cargo features) so they don't pull in GCP dependencies unless opted in.
5. Integration points: CloudTasksBroker implements Broker + PushBroker + DelayedBroker traits; Cloud Scheduler backend implements SchedulerBackend trait.

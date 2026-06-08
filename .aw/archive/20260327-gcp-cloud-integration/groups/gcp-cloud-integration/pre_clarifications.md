---
change: gcp-cloud-integration
group: gcp-cloud-integration
date: 2026-03-26
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the GCP integrations use REST API directly (via reqwest) or a Google Cloud Rust SDK crate?
- **Answer**: Use reqwest + REST API directly. cclab-api is a server framework (Quasar/Axum) with no HTTP client component, so reqwest is the right choice. Minimal dependencies, full control, consistent with existing CloudTasksBroker stub pattern.

### Q2: General
- **Question**: Should the Cloud Scheduler backend manage schedule CRUD (create/update/delete jobs in GCP), or only trigger/read existing schedules?
- **Answer**: Full CRUD — create, update, delete, pause/resume Cloud Scheduler jobs. The SchedulerBackend manages the full lifecycle.

### Q3: General
- **Question**: Should CloudTasksBroker also implement the DelayedBroker trait (native scheduled/delayed task support)?
- **Answer**: Yes — Cloud Tasks natively supports scheduleTime. Implement DelayedBroker for publish_delayed() and publish_at().

### Q4: General
- **Question**: What feature flag names for the new GCP integrations?
- **Answer**: Separate flags: `cloudtasks` (already exists) for the broker, `cloud-scheduler` for the scheduler backend.


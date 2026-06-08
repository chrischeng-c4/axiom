---
change: scheduler-runtime-complete
group: scheduler-runtime-complete
date: 2026-03-27
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the push receiver HTTP server be standalone (separate port) or mount as routes on the existing cclab server?
- **Answer**: Mount on existing server — add axum routes (e.g. /scheduler/push/{task}) to the existing cclab server. No extra port, reuses existing TLS/middleware.

### Q2: General
- **Question**: For K8s CronJob, should the CronJob pod HTTP POST to the push receiver, or directly publish to broker?
- **Answer**: HTTP POST to push receiver — CronJob pod runs a minimal binary that POSTs to the scheduler push endpoint. Consistent with Cloud Scheduler pattern. Monitor catches all triggers through the unified push path.

### Q3: General
- **Question**: Should schedule monitor support alerting callbacks (webhook/slack) or metrics-only for now?
- **Answer**: Metrics + webhook callback — emit Prometheus counter + histogram, AND support configurable webhook URL for missed schedule alerts. More self-contained alerting for critical missed schedules.

### Q4: General
- **Question**: For K8s push receiver auth, which approach?
- **Answer**: Shared HMAC secret — CronJob pod signs request with shared secret. Simple, no cluster RBAC needed. Secret injected via K8s Secret.


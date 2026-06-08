---
change: scheduler-runtime-complete
group: scheduler-runtime-complete
date: 2026-03-27
written_by: artifact_cli
review_verdict: pass
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| scheduler-backends | cclab-meteor | high | R1: SchedulerBackend trait — acquire_leader(ttl), renew_leader(ttl), release_leader(), get_task_state(name), set_task_state(name, state), R2: IonSchedulerBackend — production backend using cclab-ion for distributed locking, R3: MemorySchedulerBackend — in-memory backend for testing, New K8sCronJobBackend must implement this same SchedulerBackend trait (not in current spec — will be added) |
| cloud-scheduler-backend | cclab-meteor | high | CloudSchedulerConfig with target_base_url — the push receiver endpoint URL, OIDC token cache and authentication for GCP API calls, httpTarget in CloudSchedulerJob points to push receiver endpoint, Push receiver must validate OIDC tokens from Cloud Scheduler callbacks |
| broker-traits | cclab-meteor | high | PushBroker trait: parse_push_request(headers, body), endpoint_path(), Broker trait: publish(queue, message) — the downstream path from push receiver, BrokerMessage: delivery_tag, payload (TaskMessage), headers, timestamp, Push receiver callback must construct TaskMessage and call broker.publish() |
| cloudtasks-broker | cclab-meteor | medium | CloudTasksBroker already handles push request parsing — pattern to follow for scheduler push receiver, R5: Full JWT/OIDC token validation defined in spec (decode JWT, verify Google public keys, check audience/email), endpoint_path returns /meteor/push/{queue} — push receiver needs similar path design |
| scheduler-backends-gcp | cclab-meteor | medium | Shows exact pattern for registering a new SchedulerBackend variant via cfg(feature) in scheduler/mod.rs, R4: CloudSchedulerBackend registered under cloud-scheduler feature flag, K8sCronJobBackend must follow same registration pattern for k8s-scheduler feature flag |
| scheduler-architecture | cclab-meteor | medium | R3: Pluggable backend architecture — SchedulerBackend is decoupled from PeriodicScheduler, Current leader election loop: acquire_leader → evaluate schedules → enqueue tasks → renew_leader, scheduler-mode-selection spec will extend this architecture with push receiver mode |
| k8s-job-executor | cclab-meteor | medium | R2: kube-rs integration — K8sJobExecutor uses kube-rs to communicate with K8s API, R3: Rich resource configuration — configurable resource limits (CPU, memory), requests (GPU, TPU), nodeSelector, tolerations, R4: Non-blocking spawning — worker acks message once K8s Job is successfully created, K8sCronJobBackend should reuse kube-rs client setup and resource configuration patterns from this executor |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| push-receiver | create | crates/cclab-fetch/scheduler/push-receiver.md | overview, schema, rest-api, changes |
| k8s-cronjob-backend | create | crates/cclab-fetch/scheduler/k8s-cronjob-backend.md | overview, schema, changes |
| schedule-monitor | create | crates/cclab-fetch/scheduler/schedule-monitor.md | overview, schema, changes |
| scheduler-mode-selection | create | crates/cclab-fetch/scheduler/scheduler-mode-selection.md | overview, schema, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: scheduler-runtime-complete

**Verdict**: pass

### Summary

Reference context covers all four pre-clarification areas and references 7 valid, relevant specs. Spec plan structure is sound: all 4 create entries use crates/cclab-fetch/scheduler/ subfolder paths, one logical unit per file, no duplicate section types. Two accuracy warnings: (1) scheduler-backends key requirements attribute method signatures (acquire_leader, renew_leader, etc.) that are not present in the old scheduler-backends.md spec directly — they exist in scheduler-backends-gcp scenario S3 and cloud-scheduler-backend R1, so the info is correct but the attribution is off; (2) cloud-scheduler-backend key requirements include 'Push receiver must validate OIDC tokens from Cloud Scheduler callbacks' which is a derived consequence of R5 (httpTarget.oidcToken config), not a stated requirement in that spec — this belongs to the push-receiver spec to be created. Neither issue blocks the change spec phase since the implementation agent will read the actual specs.

### Issues

- **[warn]** Method signatures (acquire_leader(ttl), renew_leader(ttl), release_leader(), get_task_state(name), set_task_state(name, state)) are attributed to scheduler-backends (old-format spec at crates/cclab-fetch/scheduler-backends.md), but that spec only states R1 as 'Define a trait to abstract away the leader election and state persistence' with no method-level detail. The method list is accurate — it appears in scheduler-backends-gcp scenario S3 (crates/cclab-fetch/scheduler/scheduler-backends.md line 45) and cloud-scheduler-backend R1. Attribution mismatch does not block implementation but could mislead an agent inspecting the old spec directly.
- **[warn]** 'Push receiver must validate OIDC tokens from Cloud Scheduler callbacks' is not a stated requirement in cloud-scheduler-backend spec. It is a derived consequence of R5 (httpTarget.oidcToken — configures GCP Cloud Scheduler to include an OIDC token when calling the push endpoint). The actual inbound OIDC validation requirement belongs in the push-receiver spec to be created. This creates a mismatch between what an agent reads in the spec vs what the key requirements claim.

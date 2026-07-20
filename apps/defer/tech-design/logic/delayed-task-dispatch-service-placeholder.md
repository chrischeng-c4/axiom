---
id: '766'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-delayed-task-dispatch-service
entry: accept_task
nodes:
  accept_task: { kind: start, label: "accept queue-scoped task with target, ETA, priority, attempt policy, and stable task id" }
  commit_create: { kind: process, label: "Raft commits task creation and every queue policy/control mutation before replicas expose it" }
  eligibility: { kind: decision, label: "task is due and queue is running with rate, burst, and in-flight budget?" }
  lease: { kind: process, label: "commit executor ownership, fence epoch, attempt id, and lease expiry before external HTTP effect" }
  dispatch: { kind: process, label: "shared bounded executor sends exact target request with stable idempotency key and optional length-delimited HMAC" }
  outcome: { kind: decision, label: "2xx result can still commit under the live fence?" }
  success: { kind: terminal, label: "commit Succeeded once; backup and replicas preserve the terminal state" }
  retryable: { kind: decision, label: "attempt budget remains?" }
  retry: { kind: process, label: "commit nack and reschedule using queue retry policy; retain stable idempotency key and issue a fresh attempt id" }
  dlq: { kind: terminal, label: "commit DeadLettered after max attempts" }
  lost_fence: { kind: process, label: "report LostOwnership; only a later fenced retry may commit terminal success" }
  shared_shell: { kind: terminal, label: "shared service libraries own HTTP, auth, metrics/OTLP, backup, Raft transport, operator, and deployment rendering" }
edges:
  - { from: accept_task, to: commit_create }
  - { from: commit_create, to: eligibility }
  - { from: eligibility, to: lease, label: "yes" }
  - { from: eligibility, to: eligibility, label: "not yet" }
  - { from: lease, to: dispatch }
  - { from: dispatch, to: outcome }
  - { from: outcome, to: success, label: "yes" }
  - { from: outcome, to: lost_fence, label: "2xx but stale fence" }
  - { from: outcome, to: retryable, label: "non-2xx or transport failure" }
  - { from: lost_fence, to: retry }
  - { from: retryable, to: retry, label: "yes" }
  - { from: retryable, to: dlq, label: "no" }
  - { from: retry, to: eligibility }
  - { from: commit_create, to: shared_shell, label: "non-domain surfaces" }
---
flowchart TD
    accept_task([accept scheduled HTTP task]) --> commit_create[Raft commit task and queue state]
    commit_create --> eligibility{due + queue permits?}
    eligibility -->|not yet| eligibility
    eligibility -->|yes| lease[commit fenced attempt lease]
    lease --> dispatch[bounded signed HTTP dispatch]
    dispatch --> outcome{2xx and live fence?}
    outcome -->|yes| success([Succeeded])
    outcome -->|2xx but stale fence| lost_fence[LostOwnership]
    outcome -->|failure| retryable{attempts remain?}
    lost_fence --> retry[commit retry with stable task key]
    retryable -->|yes| retry
    retryable -->|no| dlq([DeadLettered])
    retry --> eligibility
    commit_create --> shared_shell([shared service libraries own non-domain shell])
```

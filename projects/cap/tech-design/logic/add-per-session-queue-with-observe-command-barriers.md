---
id: add-per-session-queue-with-observe-command-barriers
summary: Add the first opt-in per-session queue slice where no-observe side effects return job metadata and observe commands drain prior same-session jobs.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "Session queueing changes how cap run schedules selected command strings while preserving conservative synchronous behavior for unknown commands."
  - id: config-logging-and-reap-policy
    role: primary
    gap: run-log-persistence
    claim: run-log-persistence
    coverage: partial
    rationale: "Queued commands return durable job metadata and expose prior job failures at observe barriers."
---

# Add Per-Session Queue With Observe Command Barriers

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-session-queue-observe-barriers
entry: cap_run
nodes:
  cap_run: { kind: start, label: "cap run receives command string" }
  session_id: { kind: decision, label: "CAP_SESSION_ID present?" }
  classify: { kind: decision, label: "profiled wait policy?" }
  enqueue: { kind: process, label: "enqueue no-observe side effect and return job id" }
  observe: { kind: process, label: "observe command drains same-session prior jobs" }
  prior_failed: { kind: decision, label: "prior queued job failed?" }
  sync_current: { kind: process, label: "run current command synchronously" }
  unknown_sync: { kind: process, label: "unknown/risky command stays synchronous" }
  terminal: { kind: terminal, label: "stdout/stderr/exit preserve causality" }
edges:
  - { from: cap_run, to: session_id, label: "command string mode" }
  - { from: session_id, to: unknown_sync, label: "no explicit session" }
  - { from: session_id, to: classify, label: "session enabled" }
  - { from: classify, to: enqueue, label: "touch side effect / no-observe" }
  - { from: classify, to: observe, label: "ls cat grep find observe" }
  - { from: classify, to: unknown_sync, label: "unknown or risky" }
  - { from: observe, to: prior_failed, label: "barrier complete" }
  - { from: prior_failed, to: terminal, label: "surface prior failure" }
  - { from: prior_failed, to: sync_current, label: "queue clean" }
  - { from: enqueue, to: terminal, label: "job metadata" }
  - { from: sync_current, to: terminal, label: "current command result" }
  - { from: unknown_sync, to: terminal, label: "existing behavior" }
---
flowchart TB
  cap_run["cap run receives command string"] --> session_id{"CAP_SESSION_ID present?"}
  session_id -->|no| unknown_sync["unknown/risky command stays synchronous"]
  session_id -->|yes| classify{"profiled wait policy?"}
  classify -->|touch no-observe| enqueue["enqueue side effect and return job id"]
  classify -->|ls/cat/grep/find observe| observe["drain same-session prior jobs"]
  classify -->|unknown| unknown_sync
  observe --> prior_failed{"prior queued job failed?"}
  prior_failed -->|yes| terminal["surface prior failure"]
  prior_failed -->|no| sync_current["run current command synchronously"]
  enqueue --> terminal
  sync_current --> terminal
  unknown_sync --> terminal
```

The first slice is deliberately opt-in through `CAP_SESSION_ID`; without an
explicit session id, `cap run` keeps existing synchronous behavior. The initial
profile set is conservative: `touch <path...>` may queue as a no-observe side
effect, while `ls`, `cat`, `grep`, and `find` act as observe barriers. All other
commands remain synchronous until profiled. Queue state is local and per-session,
not distributed, and observe barriers must report prior queued-job failures
before running the current observation command.

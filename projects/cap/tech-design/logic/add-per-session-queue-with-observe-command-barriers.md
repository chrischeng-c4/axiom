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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-session-queue-observe-barriers-tests
requirements:
  queued_side_effect:
    id: SQ-UT-1
    text: "With CAP_SESSION_ID set, a profiled no-observe touch command returns queued job metadata before an observe command."
    kind: functional
    risk: high
    verify: test
  observe_barrier:
    id: SQ-UT-2
    text: "A following observe command drains prior same-session jobs before returning output."
    kind: functional
    risk: high
    verify: test
  prior_failure:
    id: SQ-UT-3
    text: "A prior queued job failure is reported at the observe barrier with the failed job id and stderr."
    kind: functional
    risk: high
    verify: test
  unknown_sync:
    id: SQ-UT-4
    text: "Unknown command strings remain synchronous unless a profile opts them into queue behavior."
    kind: functional
    risk: medium
    verify: test
elements:
  session_queue_unit_tests:
    kind: test
    type: "cargo test -p cap session_queue"
relations:
  - { from: session_queue_unit_tests, verifies: queued_side_effect }
  - { from: session_queue_unit_tests, verifies: observe_barrier }
  - { from: session_queue_unit_tests, verifies: prior_failure }
  - { from: session_queue_unit_tests, verifies: unknown_sync }
---
requirementDiagram
  requirement queued_side_effect {
    id: SQ-UT-1
    text: "no-observe touch returns job metadata"
    risk: high
    verifymethod: test
  }
  requirement observe_barrier {
    id: SQ-UT-2
    text: "observe command drains prior same-session jobs"
    risk: high
    verifymethod: test
  }
  requirement prior_failure {
    id: SQ-UT-3
    text: "observe barrier reports prior queued failure"
    risk: high
    verifymethod: test
  }
  requirement unknown_sync {
    id: SQ-UT-4
    text: "unknown command remains synchronous"
    risk: medium
    verifymethod: test
  }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cap/src/session_queue.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: >
      Add the first opt-in per-session queue. CAP_SESSION_ID enables local
      session state, profiled no-observe commands enqueue background jobs, and
      observe commands drain prior same-session jobs.

  - path: projects/cap/src/session_queue.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: >
      Cover queued touch metadata, observe barrier draining, prior failure
      reporting, and unknown-command synchronous behavior.

  - path: projects/cap/src/cli.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Let command-string cap run consult the session queue before resident shell
      execution. Queue handling is opt-in and argv mode remains unchanged.

  - path: projects/cap/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Export the session queue module inside the cap crate.

  - path: projects/cap/tech-design/semantic/cap-src.md
    action: modify
    section: exports
    impl_mode: hand-written
    description: >
      Keep semantic export metadata aligned with the new session_queue module.

  - path: projects/cap/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: >
      Document the opt-in per-session queue, observe barriers, conservative
      default synchronous behavior, and prior-failure reporting.
```

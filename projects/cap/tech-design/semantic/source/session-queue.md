---
id: cap-source-session-queue
summary: Source coverage for projects/cap/src/session_queue.rs opt-in per-session command barriers.
fill_sections: [source, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: full
    rationale: "The session queue controls cap run command-string scheduling for profiled no-observe side effects and observe barriers."
  - id: config-logging-and-reap-policy
    role: primary
    gap: run-log-persistence
    claim: run-log-persistence
    coverage: full
    rationale: "Queued job metadata and failure files provide durable local evidence for observe-barrier diagnostics."
---

# Source TD: projects/cap/src/session_queue.rs

## Source
<!-- type: source lang: rust -->

`````rust
// CAP_SESSION_ID enables a local per-session queue.
// Profiled no-observe touch commands enqueue background jobs; observe commands
// drain prior jobs and surface prior failure diagnostics before running.
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/cap/src/session_queue.rs"
    action: modify
    section: source
    description: |
      Source coverage for opt-in per-session queueing, observe barriers, and
      prior queued-job failure reporting.
    impl_mode: hand-written
```

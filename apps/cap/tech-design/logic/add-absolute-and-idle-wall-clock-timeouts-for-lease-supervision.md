---
id: add-absolute-and-idle-wall-clock-timeouts-for-lease-supervision
summary: Add two independent, default-disabled per-lease termination triggers — an absolute wall-clock deadline and a CPU-time-delta idle-progress detector — that reuse the existing Killing lease state and kill_grace_secs escalation.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "New per-lease absolute and idle wall-clock timeout triggers extend tick()'s existing supervision loop, reusing the Killing state and kill_grace_secs escalation."
  - id: command-lease-throttling
    role: primary
    gap: memory-and-cpu-pressure-sampling
    claim: memory-and-cpu-pressure-sampling
    coverage: partial
    rationale: "A new CpuSampler, same shape as RssSampler, supplies per-process cumulative CPU time so the idle-timeout trigger can detect no-progress independently of memory/CPU pressure sampling."
---

# TD: cap absolute and idle wall-clock lease timeouts

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-lease-wall-clock-and-idle-timeouts
entry: tick_lease
nodes:
  tick_lease: { kind: start, label: "tick() evaluates each lease this sample" }
  paused_check: { kind: decision, label: "lease currently Paused?" }
  abs_configured: { kind: decision, label: "timeout_secs > 0 (flag or default_timeout_secs)?" }
  abs_check: { kind: decision, label: "now - spawned_at - paused_total >= timeout_secs?" }
  idle_configured: { kind: decision, label: "idle_timeout_secs > 0 (flag or default_idle_timeout_secs)?" }
  cpu_sample: { kind: process, label: "CpuSampler reads process-group cumulative CPU time" }
  cpu_progress: { kind: decision, label: "cumulative CPU time advanced since last sample?" }
  idle_debounce: { kind: decision, label: "no-progress ticks >= trigger_samples derived from idle_timeout_secs / sample_interval_ms?" }
  reset_idle: { kind: process, label: "reset no-progress counter; record last-active tick" }
  fire_abs: { kind: process, label: "classify AbsoluteTimeout; SIGTERM leader; lease -> Killing; kill_started_at set" }
  fire_idle: { kind: process, label: "classify IdleTimeout; SIGTERM leader; lease -> Killing; kill_started_at set" }
  escalate: { kind: terminal, label: "existing kill_grace_secs escalator promotes to group SIGKILL (unchanged)" }
  no_action: { kind: terminal, label: "lease continues; memory/CPU pressure path still runs independently" }
edges:
  - { from: tick_lease, to: paused_check }
  - { from: paused_check, to: no_action, label: "paused: both clocks frozen" }
  - { from: paused_check, to: abs_configured, label: "running" }
  - { from: abs_configured, to: idle_configured, label: "0 = disabled" }
  - { from: abs_configured, to: abs_check, label: "configured" }
  - { from: abs_check, to: fire_abs, label: "deadline exceeded" }
  - { from: abs_check, to: idle_configured, label: "still within budget" }
  - { from: idle_configured, to: no_action, label: "0 = disabled" }
  - { from: idle_configured, to: cpu_sample, label: "configured" }
  - { from: cpu_sample, to: cpu_progress }
  - { from: cpu_progress, to: reset_idle, label: "advanced" }
  - { from: cpu_progress, to: idle_debounce, label: "unchanged" }
  - { from: idle_debounce, to: fire_idle, label: "debounce threshold reached" }
  - { from: idle_debounce, to: no_action, label: "below threshold" }
  - { from: reset_idle, to: no_action }
  - { from: fire_abs, to: escalate }
  - { from: fire_idle, to: escalate }
---
flowchart TB
  tick_lease["tick() evaluates each lease this sample"] --> paused_check{"lease currently Paused?"}
  paused_check -->|paused: both clocks frozen| no_action(["lease continues; pressure path still runs"])
  paused_check -->|running| abs_configured{"timeout_secs > 0?"}
  abs_configured -->|0 = disabled| idle_configured{"idle_timeout_secs > 0?"}
  abs_configured -->|configured| abs_check{"now - spawned_at - paused_total >= timeout_secs?"}
  abs_check -->|deadline exceeded| fire_abs["classify AbsoluteTimeout; SIGTERM leader; Killing"]
  abs_check -->|within budget| idle_configured
  idle_configured -->|0 = disabled| no_action
  idle_configured -->|configured| cpu_sample["CpuSampler reads process-group cumulative CPU time"]
  cpu_sample --> cpu_progress{"cumulative CPU time advanced since last sample?"}
  cpu_progress -->|advanced| reset_idle["reset no-progress counter; record last-active tick"]
  cpu_progress -->|unchanged| idle_debounce{"no-progress ticks >= trigger_samples derived from idle_timeout_secs?"}
  idle_debounce -->|threshold reached| fire_idle["classify IdleTimeout; SIGTERM leader; Killing"]
  idle_debounce -->|below threshold| no_action
  reset_idle --> no_action
  fire_abs --> escalate(["existing kill_grace_secs escalator -> group SIGKILL (unchanged)"])
  fire_idle --> escalate
```

Both triggers are new, independent, default-disabled entry points into the
**existing** `Killing` lease state and `kill_grace_secs` SIGTERM -> grace ->
SIGKILL escalation in `throttle.rs::tick()` — no new kill mechanism. They run
alongside (not instead of) the existing memory/CPU pressure pause-kill path
every tick; a lease may still be paused or killed by pressure regardless of
its timeout configuration.

The absolute-timeout clock is `now - spawned_at - paused_total`: elapsed
wall time since the client reported the spawned PID, minus all time the
lease has spent `Paused` (SIGSTOPped by cap's own pressure logic), so cap's
pausing can never spuriously trip a caller's `--timeout`. `paused_total` is
the same field `build_run_record` already uses for the run log; an
in-flight `paused_since` (lease paused right now) is added the same way.

The idle-timeout clock is a debounced no-progress detector, not a raw
silence timer: each tick, a new `CpuSampler` (same shape as `RssSampler`,
added to `sampler.rs`, using `ProcessRefreshKind::new().with_cpu()`) reads
the process group's cumulative CPU time. If it has not advanced since the
last sample, a per-lease no-progress counter increments; `trigger_samples`
for the idle path is derived from `idle_timeout_secs` divided by
`sample_interval_ms` (same debounce shape `pause_used_percent`/
`kill_used_percent` already use via `State`'s `trigger_samples` pattern,
but kept per-lease since idle progress is per-lease, not system-wide). Any
CPU-time advance resets the counter to zero. While a lease is `Paused`, its
idle counter is neither advanced nor reset — the clock is frozen, so a
pressure-paused lease cannot accumulate idle-timeout no-progress ticks
against SIGSTOP time.

Both fire paths reuse the identical `TermedVictim`-style transition already
in `tick()`: SIGTERM the leader (never the process group, matching the
existing `kill_grace_secs > 0` branch), set `state = Killing` and
`kill_started_at`, and store a `KillEnvelope` carrying the new
`KillClassification::AbsoluteTimeout` / `KillClassification::IdleTimeout`
and a matching new `Action` variant distinct from the memory-pressure
`WaitAndRetry`/`ChangeStrategy`/`InspectAndWait` framing — these are not
resource-competition kills, so telling an agent to `cap wait` and retry
would be misleading. The existing grace-period escalator at the top of
`tick()` (oldest expired `Killing` lease -> group `SIGKILL`) requires no
change: it already escalates any `Killing` lease regardless of which path
put it there.

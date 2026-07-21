---
id: '2285'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: phase-stable-soak-rss-window-sampling
entry: capture_window
nodes:
  capture_window:
    kind: start
    label: "caller supplies a live pid, window seconds, and sample cadence for one steady window"
  validate_inputs:
    kind: decision
    label: "pid reachable and cadence/window/sample path valid?"
  start_sampler:
    kind: process
    label: "create a sampler session token plus temp sample artifact, register EXIT/INT/TERM cleanup, and launch a background loop that polls service_soak_rss_kb at the fixed cadence"
  run_window:
    kind: process
    label: "caller runs its existing bounded workload while the sampler session appends one RSS integer per line for the whole window"
  stop_sampler:
    kind: process
    label: "caller closes the sampler session; the helper terminates and waits for the background sampler before returning"
  validate_series:
    kind: decision
    label: "pid stayed alive, every sample is a non-negative integer, and coverage reached ceil(window_secs / sample_interval_secs)?"
  summarize:
    kind: process
    label: "sort the series and emit count, min, median, and max as deterministic integers"
  compare_windows:
    kind: process
    label: "compare summary medians with existing integer percent-growth semantics and leave the caller threshold outside the shared library"
  fail_closed:
    kind: terminal
    label: "exit non-zero and leave no sampler process behind"
  window_summary_ready:
    kind: terminal
    label: "validated window summary is ready for a caller-owned plateau policy"
  plateau_ready:
    kind: terminal
    label: "shared oracle reports median drift for a caller-owned RSS threshold"
edges:
  - { from: capture_window, to: validate_inputs }
  - { from: validate_inputs, to: fail_closed, label: "no" }
  - { from: validate_inputs, to: start_sampler, label: "yes" }
  - { from: start_sampler, to: run_window }
  - { from: run_window, to: stop_sampler }
  - { from: stop_sampler, to: validate_series }
  - { from: validate_series, to: fail_closed, label: "no" }
  - { from: validate_series, to: summarize, label: "yes" }
  - { from: summarize, to: window_summary_ready }
  - { from: window_summary_ready, to: compare_windows }
  - { from: compare_windows, to: plateau_ready }
---
flowchart TD
    start[caller requests one steady RSS window] --> inputs{pid, cadence, and window valid?}
    inputs -->|no| fail([fail closed and clean up sampler])
    inputs -->|yes| sampler[start sampler session and arm cleanup]
    sampler --> workload[caller runs bounded workload]
    workload --> stop[caller closes session; helper stops sampler]
    stop --> validate{numeric samples and enough coverage?}
    validate -->|no| fail
    validate -->|yes| summary[emit count, min, median, max]
    summary --> oracle[compare window medians with integer growth semantics]
    oracle --> ready([caller applies its own RSS threshold])
```

Contract invariants:

- `libs/service-observability/scripts/soak-metrics.sh` remains the shared process-metric owner. It adds an explicit sampler start/stop/summary lifecycle plus the plateau helper without taking ownership of any app workload, duration policy, or growth threshold.
- The new sampler session owns its lifecycle. Start returns a session token or handle for the caller's current window, creates the temporary sample artifact, records one RSS value per line at a fixed cadence for the whole measured window, and stop or the registered cleanup hook always kills and waits for the background sampler before returning, even when the caller is interrupted with `INT` or `TERM`.
- Sampling fails closed when the measured pid disappears before window completion, `service_soak_rss_kb` returns a non-numeric value, or the captured series does not reach `ceil(window_secs / sample_interval_secs)` samples. A missing or malformed series never degrades to a best-effort endpoint comparison.
- Window summaries are deterministic integer records. `count`, `min`, `median`, and `max` come from sorting the captured series; odd counts take the middle element, and even counts average the two middle elements with shell-integer division semantics so later growth math stays stable.
- Plateau comparison is median-to-median only. `service_soak_percent_growth` keeps its current integer percentage semantics, but the shared RSS oracle feeds it each window summary's median instead of one endpoint reading. A bounded transient spike therefore affects `max` while a sustained allocation increase moves `median` and can breach the caller-owned limit.
- Deterministic shell coverage exercises three cases: a fixed numeric series with a transient spike (`100 100 500 100 100`), a sustained-growth pair (`100` median to `120` median), and a real bounded child process whose sampler is cleaned up on both normal completion and forced interruption.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-observability/scripts/soak-metrics.sh
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add the explicit RSS sampler start/stop/summary lifecycle, validated count/min/median/max reduction, median-to-median plateau comparison, and cleanup that never leaves a background sampler running.
  - path: libs/service-observability/tests/soak_metrics_window_contract.sh
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Prove transient-spike median stability, sustained-growth breach behavior, malformed or insufficient sample failures, and real-child sampler cleanup on both normal completion and interruption.
```

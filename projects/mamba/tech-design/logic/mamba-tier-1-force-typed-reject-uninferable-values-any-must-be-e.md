---
id: '2011'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-force-typed-contract-completion
entry: accepted_contract
nodes:
  accepted_contract: { kind: start, label: "Accepted EC + WI #2011 define the only valid completion shape: inference failure stays a compile-time wall, explicit Any is the only authored dynamic ingress, and proof is external plus fail-closed" }
  collect_sources: { kind: process, label: "Load authoritative source sets before any implementation proof: the 23-row t1_forcetyped_completeness legacy denominator, the 7 implicit_any_ingress fixtures, the 7 explicit_any_acceptance fixtures, compile_diagnostic_probes, positive_runtime_baseline, concurrency probes, and performance applicability inputs" }
  pair_fixtures: { kind: process, label: "Pair each ingress family (local/global/class binding, parameter, return, comprehension, expression_join) by pair_id plus paired_template_sha256; the explicit fixture may differ only by the authored Any token" }
  compiler_wall: { kind: decision, label: "Does the public Mamba compile surface reject every implicit ingress with stable binding identity, inference-path classification, source span, and diagnostic class instead of silently synthesizing Any?" }
  wall_fail: { kind: terminal, label: "FAIL: an implicit ingress still widens to Any, lacks stable diagnostics, or the 7-row negative matrix drifts" }
  explicit_path: { kind: process, label: "Route the 7 explicit-Any counterparts through the same public compile plus run path; compile succeeds only because the annotation is authored, and runtime behavior still matches the pinned CPython 3.12 oracle" }
  inventory_rollup: { kind: process, label: "Reconcile legacy_negative_input, implicit_any_ingress_matrix, explicit_any_acceptance_matrix, compile_diagnostic_probes, positive_runtime_baseline, concurrency_parallel_progress_probe, concurrency_barrier_stability_probe, and performance_applicability into one manifest-backed rollup inventory plus evidence locks" }
  exact_accounting: { kind: decision, label: "Does every authoritative source identity appear exactly once in the rollup and exactly one channel among compile, behavior, concurrency, and performance?" }
  inventory_fail: { kind: terminal, label: "FAIL: omission, duplication, stale carry-over, self-oracle rows, or a cases.jsonl file inventing its own denominator" }
  concurrency_gate: { kind: decision, label: "Do the public-path concurrency probes prove both no global serialization (parallel progress) and deadlock-free stability (barrier soak)?" }
  concurrency_fail: { kind: terminal, label: "FAIL: the fix repairs type walls by serializing work, weakens barrier stability, or leaves multicore evidence ambiguous" }
  perf_gate: { kind: decision, label: "Does the independent baseline_sha/current_sha diff classify this change as performance-applicable?" }
  perf_verify: { kind: process, label: "If applicable, run the named perf pin against one baseline row identity shared by committed JSONL plus imported SQLite evidence; otherwise record machine-verifiable non-applicability from the same anchored diff" }
  perf_fail: { kind: terminal, label: "FAIL: unanchored baseline provenance, missing graded perf trial, host drift, or unexplained CPU or RSS regression blocks completion" }
  ec_pass: { kind: terminal, label: "PASS: Force Typed completion is exact and fail-closed: implicit Any is gone, explicit Any remains intentional, channels are fully accounted, concurrency stays multicore, and perf status is provenance-bound" }
edges:
  - { from: accepted_contract, to: collect_sources }
  - { from: collect_sources, to: pair_fixtures }
  - { from: pair_fixtures, to: compiler_wall }
  - { from: compiler_wall, to: wall_fail, label: "no" }
  - { from: compiler_wall, to: explicit_path, label: "yes" }
  - { from: explicit_path, to: inventory_rollup }
  - { from: inventory_rollup, to: exact_accounting }
  - { from: exact_accounting, to: inventory_fail, label: "no" }
  - { from: exact_accounting, to: concurrency_gate, label: "yes" }
  - { from: concurrency_gate, to: concurrency_fail, label: "no" }
  - { from: concurrency_gate, to: perf_gate, label: "yes" }
  - { from: perf_gate, to: perf_verify, label: "yes" }
  - { from: perf_gate, to: ec_pass, label: "no, record non-applicable from anchored diff" }
  - { from: perf_verify, to: perf_fail, label: "graded evidence missing or red" }
  - { from: perf_verify, to: ec_pass, label: "graded evidence green" }
---
flowchart TD
    A["Accepted EC + WI #2011 define the only valid completion shape:\ninference failure stays a compile-time wall, explicit Any is the\nonly authored dynamic ingress, and proof is external plus fail-closed"] --> B["Load authoritative source sets before any implementation proof:\n23-row t1_forcetyped_completeness legacy denominator,\n7 implicit_any_ingress fixtures, 7 explicit_any_acceptance fixtures,\ncompile_diagnostic_probes, positive_runtime_baseline,\nconcurrency probes, and performance applicability inputs"]
    B --> C["Pair each ingress family by pair_id plus paired_template_sha256:\nlocal_binding, global_binding, class_attribute, parameter,\nreturn, comprehension, expression_join. The explicit fixture\nmay differ only by the authored Any token"]
    C --> D{"Does the public Mamba compile surface reject every implicit\ningress with stable binding identity, inference-path classification,\nsource span, and diagnostic class instead of silently synthesizing Any?"}
    D -- "no" --> E["FAIL: an implicit ingress still widens to Any, lacks stable\ndiagnostics, or the 7-row negative matrix drifts"]
    D -- "yes" --> F["Route the 7 explicit-Any counterparts through the same public\ncompile plus run path; compile succeeds only because the annotation\nis authored, and runtime behavior still matches the pinned\nCPython 3.12 oracle"]
    F --> G["Reconcile legacy_negative_input, implicit_any_ingress_matrix,\nexplicit_any_acceptance_matrix, compile_diagnostic_probes,\npositive_runtime_baseline, concurrency_parallel_progress_probe,\nconcurrency_barrier_stability_probe, and performance_applicability\ninto one manifest-backed rollup inventory plus evidence locks"]
    G --> H{"Does every authoritative source identity appear exactly once\nin the rollup and exactly one channel among compile, behavior,\nconcurrency, and performance?"}
    H -- "no" --> I["FAIL: omission, duplication, stale carry-over, self-oracle rows,\nor a cases.jsonl file inventing its own denominator"]
    H -- "yes" --> J{"Do the public-path concurrency probes prove both no global\nserialization (parallel progress) and deadlock-free stability\n(barrier soak)?"}
    J -- "no" --> K["FAIL: the fix repairs type walls by serializing work, weakens\nbarrier stability, or leaves multicore evidence ambiguous"]
    J -- "yes" --> L{"Does the independent baseline_sha/current_sha diff classify\nthis change as performance-applicable?"}
    L -- "no, record anchored non-applicability" --> M["PASS: Force Typed completion is exact and fail-closed:\nimplicit Any is gone, explicit Any remains intentional,\nchannels are fully accounted, concurrency stays multicore,\nand perf status is provenance-bound"]
    L -- "yes" --> N["Run the named perf pin against one baseline row identity\nshared by committed JSONL plus imported SQLite evidence;\notherwise fail if provenance, host, grading, or CPU/RSS\nreconciliation is incomplete"]
    N -- "graded evidence missing or red" --> O["FAIL: unanchored baseline provenance, missing graded perf trial,\nhost drift, or unexplained CPU or RSS regression blocks completion"]
    N -- "graded evidence green" --> M
```

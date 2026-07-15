---
id: '1765'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-shared-labeled-prometheus-flow
entry: collect
nodes:
  collect:
    kind: start
    label: Pgpool reads live ConnectionBudget and BackendPool stats in configured pool order
  adapt:
    kind: process
    label: Build three SampleGroup values; each LabeledSample owns Label entries and the live u64 value
  group_loop:
    kind: process
    label: metrics-prometheus writes HELP and TYPE once for each group in caller order
  sort_labels:
    kind: process
    label: For each row, sort label references by name then value without mutating caller data
  escape_labels:
    kind: process
    label: Escape backslash, double quote, and newline in every label value
  render_row:
    kind: process
    label: Write metric name, canonical brace-delimited labels, and value; preserve sample order
  more_rows:
    kind: decision
    label: More samples or groups remain
  response:
    kind: terminal
    label: Return deterministic Prometheus 0.0.4 bytes with the existing Pgpool metric contract
edges:
  - { from: collect, to: adapt }
  - { from: adapt, to: group_loop }
  - { from: group_loop, to: sort_labels }
  - { from: sort_labels, to: escape_labels }
  - { from: escape_labels, to: render_row }
  - { from: render_row, to: more_rows }
  - { from: more_rows, to: sort_labels, label: next sample }
  - { from: more_rows, to: group_loop, label: next group }
  - { from: more_rows, to: response, label: complete }
---
flowchart TD
  collect([Read live Pgpool pool state]) --> adapt[Build shared SampleGroup and LabeledSample values]
  adapt --> group_loop[Emit group HELP and TYPE]
  group_loop --> sort_labels[Sort labels by name then value]
  sort_labels --> escape_labels[Escape label values]
  escape_labels --> render_row[Emit labeled row]
  render_row --> more_rows{More rows or groups?}
  more_rows -->|next row| sort_labels
  more_rows -->|next group| group_loop
  more_rows -->|done| response([Return unchanged scrape contract])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - { path: libs/metrics-prometheus/tech-design/semantic/source/libs-metrics-prometheus-src-lib-rs.md, action: modify, section: logic, impl_mode: hand-written, description: Add labeled sample groups, deterministic label ordering, and safe escaping to the canonical source unit. }
  - { path: libs/metrics-prometheus/README.md, action: modify, section: logic, impl_mode: hand-written, description: Document labeled exposition as part of the shared capability. }
  - { path: apps/pgpool/src/admin/metrics.rs, action: modify, section: logic, impl_mode: hand-written, description: Replace local exposition formatting with shared labeled sample groups. }
  - { path: apps/pgpool/Cargo.toml, action: modify, section: logic, impl_mode: hand-written, description: Depend on metrics-prometheus. }
  - { path: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md, action: modify, section: logic, impl_mode: hand-written, description: Record shared encoder ownership while preserving the Pgpool contract. }
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-shared-labeled-prometheus-verification
requirements:
  labeled_encoder:
    id: R1
    text: "metrics-prometheus emits HELP and TYPE once per group, sorts label names deterministically, and escapes backslash, quote, and newline in label values."
    kind: functional
    risk: high
    verify: cargo test -p metrics-prometheus
  pgpool_contract:
    id: R2
    text: "Pgpool preserves its three per-pool gauge names, HELP text, TYPE, pool labels, values, and row order while delegating generic exposition formatting."
    kind: regression
    risk: high
    verify: cargo test -p pgpool admin::metrics
  served_scrape:
    id: R3
    text: "The served /metrics endpoint continues exposing live labeled pool gauges over the existing admin plane."
    kind: regression
    risk: medium
    verify: cargo test -p pgpool --test admin_plane metrics_exposes_prometheus_pool_gauges -- --nocapture
---
flowchart TD
    r1[R1 labeled encoder] --> cargo_test_p_metrics_prometheus[cargo test -p metrics-prometheus]
    r2[R2 pgpool contract] --> cargo_test_p_pgpool_admin_metrics[cargo test -p pgpool admin::metrics]
    r3[R3 served scrape] --> cargo_test_p_pgpool_test_admin_plane_metrics_exposes_prometheus_pool_gauges_nocapture[cargo test -p pgpool --test admin_plane metrics_exposes_prometheus_pool_gauges -- --nocapture]
```

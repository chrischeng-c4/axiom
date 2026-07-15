# metrics-prometheus

## Brief

`metrics-prometheus` provides lock-free Prometheus metric primitives and a text
format encoder for service endpoints, including deterministic labeled metric
groups with Prometheus-safe label escaping.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Prometheus Metric Primitives | - | implemented | verified | smoke | ready | counters, gauges, latency observations, and labeled/unlabeled encoders |

### Shared Prometheus Metric Primitives

ID: shared-prometheus-metric-primitives
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `metrics_prometheus`.
EC Dimensions: behavior: `cargo test -p metrics-prometheus` - metric primitive and encoder coverage
Required Verification: smoke
Promise:
Services can expose deterministic Prometheus text metrics through shared
lock-free primitives without service-specific encoders. Labeled groups emit
one HELP/TYPE declaration, preserve caller row order, sort labels
deterministically, and escape backslash, quote, and newline values.
Gate Inventory: `cargo test -p metrics-prometheus`; `cargo test -p pgpool admin::metrics`; libs/metrics-prometheus/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-prometheus-metric-primitives-contract | epic | - | implemented | verified | smoke | `cargo test -p metrics-prometheus`; libs/metrics-prometheus/src/lib.rs |
| labeled-sample-encoder-adoption | change | #1765 | implemented | passing | conformance | `cargo test -p metrics-prometheus`; `cargo test -p pgpool admin::metrics` |

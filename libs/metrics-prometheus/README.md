# metrics-prometheus

## Brief

`metrics-prometheus` provides lock-free Prometheus metric primitives and a text
format encoder for HTTP services.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Prometheus Metric Primitives | - | implemented | verified | smoke | ready | counters, gauges, latency observations, and encoder |

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
lock-free primitives without service-specific encoders.
Gate Inventory: `cargo test -p metrics-prometheus`; libs/metrics-prometheus/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-prometheus-metric-primitives-contract | epic | - | implemented | verified | smoke | `cargo test -p metrics-prometheus`; libs/metrics-prometheus/src/lib.rs |

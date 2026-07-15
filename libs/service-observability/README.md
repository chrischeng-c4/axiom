<!-- HANDWRITE-BEGIN gap="missing-generator:logic:2721745f" tracker="pending-tracker" reason="Define the shared service observability capability and ownership boundary." -->
# service-observability

## Brief

`service-observability` is the protocol-neutral composition owner for service
logging, optional OTLP tracing, stable resource identity, metric providers, and
server lifecycle connection metrics. HTTP request and route adaptation remains
in `service-http`; Prometheus primitives and encoding remain in
`metrics-prometheus`.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Service Observability Integration | #1777 | implementing | planned | conformance | partial | protocol-neutral configuration, traces, providers, and lifecycle counters |

### Shared Service Observability Integration

ID: shared-service-observability-integration
Type: DeveloperTool
Root WI: 1777
Status: implementing
Surfaces: Rust API: `service_observability`.
EC Dimensions: behavior: `cargo test -p service-observability` - logging mode, identity, metric provider, and lifecycle counter coverage
Required Verification: conformance
Promise:
Services can compose one typed observability contract regardless of wire
protocol. OTLP failures retain structured logging; lifecycle connection events
render through the canonical Prometheus encoder without duplicating metric
primitives or HTTP policy.
Gate Inventory: `cargo test -p service-observability`; libs/service-observability/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-service-observability-integration | change | #1777 | implementing | planned | conformance | `cargo test -p service-observability` |
<!-- HANDWRITE-END -->

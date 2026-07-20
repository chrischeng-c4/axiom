<!-- HANDWRITE-BEGIN gap="missing-generator:logic:2721745f" tracker="1868" reason="Define the shared service observability capability and ownership boundary." -->
# service-observability

## Brief

`service-observability` is the protocol-neutral composition owner for service
logging, optional OTLP tracing, stable resource identity, metric providers,
server lifecycle connection metrics, and bounded-soak resource sampling. HTTP
request and route adaptation remains in `service-http`; Prometheus primitives
and encoding remain in `metrics-prometheus`.

## Structured stdout contract

Long-running services use `LogFormat::Json` as the collector-compatible stdout
surface. Each successful event is one compact JSON object followed by one
newline and conforms to `axiom.service.log.v1`; the checked-in schema lives at
`contracts/axiom.service.log.v1.schema.json`. Stable top-level fields include
service identity, severity, event, message, and optional W3C trace/span/request
correlation. Sensitive propagation values are excluded and remaining
attributes are bounded before serialization.

`LogFormat::Pretty` is explicitly development-only and must not be used as a
collector input. HTTP libraries supply correlation fields on request spans;
this crate owns their exporter-independent stdout serialization. Optional OTLP
export augments that path but never changes the JSONL wire contract.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Service Observability Integration | #1777 | implemented | passing | conformance | ready | protocol-neutral configuration, traces, providers, and lifecycle counters |

### Shared Service Observability Integration

ID: shared-service-observability-integration
Type: DeveloperTool
Root WI: 1777
Status: verified
Surfaces: Rust API: `service_observability`.
EC Dimensions: behavior: `cargo test -p service-observability` - logging mode, identity, metric provider, and lifecycle counter coverage
Required Verification: conformance
Promise:
Services can compose one typed observability contract regardless of wire
protocol. OTLP failures retain structured logging; lifecycle connection events
render through the canonical Prometheus encoder without duplicating metric
primitives or HTTP policy. Service-specific soak runners share portable
RSS/FD/thread/p99 sampling and plateau assertions while retaining ownership of
their domain workload.
Gate Inventory: `cargo test -p service-observability`; `bash -n libs/service-observability/scripts/soak-metrics.sh`; libs/service-observability/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-service-observability-integration | change | #1777 | implemented | passing | conformance | `cargo test -p service-observability`; `cargo test -p service-observability --features otlp` |
<!-- HANDWRITE-END -->

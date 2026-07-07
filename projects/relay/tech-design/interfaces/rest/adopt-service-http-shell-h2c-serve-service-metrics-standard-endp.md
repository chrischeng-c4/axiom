---
id: relay-service-http-shell
summary: >
  Adopt the shared service shell on relay's transport: service-http's
  standard probe routes (/healthz /readyz /metrics /openapi.json /docs) merged
  with the /v1 data plane, the h2c + HTTP/1.1 one-port serve loop with a
  SIGTERM-aware graceful drain (--grace-secs, RELAY_GRACE_SECS), the shared
  {error, message} ApiErr envelope on error paths, and RelayMetrics — per-op
  request counts + latency on libs/service-metrics primitives, recorded by a
  route_layer middleware and exposed through the MetricsProvider seam.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-service-http-shell-flow
entry: boot
nodes:
  boot:
    kind: start
    label: "serve_main: parse ServeArgs (--bind, --data-dir, --grace-secs / RELAY_GRACE_SECS default 10); init tracing (RUST_LOG wins, else info EnvFilter — keep's pattern)"
  state:
    kind: process
    label: "AppState::new: relay core + Arc<RelayMetrics> + draining AtomicBool; spawn_reconciler; AppState implements service_http::ReadinessHook and MetricsProvider"
  build:
    kind: process
    label: "router(): /v1 data plane with route_layer(metrics::track) merged onto service_http::standard_probe_routes(state, Some(metrics), crate::openapi::openapi); outer service_http::trace_layer()"
  serve:
    kind: process
    label: "service_http::serve(listener, app, shutdown_with_drain(start_drain, grace)) — HTTP/1.1 + h2c on the one serve port"
  req:
    kind: decision
    label: "Request arrives: probe route or /v1 data plane?"
  probes:
    kind: process
    label: "/healthz 200; /readyz 200 or 503 when ReadinessHook::is_draining; /metrics renders RelayMetrics through MetricsProvider; /openapi.json + /docs from the openapi accessor — hand-rolled healthz/openapi_json handlers are deleted"
  data:
    kind: process
    label: "Handler: decode JSON or CBOR body, run the Relay op, encode the response; decode errors return ApiErr 400 bad_request, engine/encode errors ApiErr 500 internal; consume rejects a non-Subscribe first frame with ApiErr 400 instead of a silent empty 200 stream"
  track:
    kind: process
    label: "metrics::track route_layer middleware: map the matched route pattern to its op family (publish / publish-batch / lease / ack / consume / other) and observe count + latency ms into RelayMetrics (service-metrics Latency primitives)"
  sigterm:
    kind: process
    label: "SIGTERM or SIGINT: start_drain flips the draining AtomicBool so /readyz reports 503, the grace window holds, then the listener closes"
  done:
    kind: terminal
    label: "Response returned on the same port — success encodings (JSON + CBOR fast path) untouched; every error body is the shared {error, message} envelope"
edges:
  - { from: boot, to: state }
  - { from: state, to: build }
  - { from: build, to: serve }
  - { from: serve, to: req, label: "request accepted" }
  - { from: req, to: probes, label: "probe/admin route" }
  - { from: req, to: data, label: "/v1/{subject}/* route" }
  - { from: data, to: track, label: "response recorded" }
  - { from: probes, to: done }
  - { from: track, to: done }
  - { from: serve, to: sigterm, label: "shutdown signal" }
  - { from: sigterm, to: done, label: "grace expired" }
---
flowchart TD
    boot([serve_main: flags + tracing init]) --> state[AppState: core + RelayMetrics + draining bool]
    state --> build[router: standard probes merged with v1 data plane + trace_layer]
    build --> serve[service_http serve: h2c + HTTP/1.1 one port]
    serve --> req{Probe or data plane?}
    req -->|probe| probes[healthz readyz metrics openapi.json docs]
    req -->|v1| data[decode body, run Relay op, encode; errors = ApiErr envelope]
    data --> track[metrics track: per-op count + latency into RelayMetrics]
    probes --> done([response])
    track --> done
    serve -->|SIGTERM| sigterm[start_drain: readyz 503, grace window, close]
    sigterm --> done
```

---
id: apps-pgpool-admin-plane
summary: Served admin HTTP plane for pgpool - a hand-rolled axum router bound on `RuntimePlan.admin_bind` via `http_server::serve_h2c_with_options`, exposing `/healthz`, `/readyz`, `/metrics` (Prometheus), `/openapi.json`, `/docs`, `GET /pools`, `GET /pools/{pool}/stats`, and `POST /drain`. A single shared `server_core::DrainController` (constructed once in `serve()` and cloned into both the TCP frontend's `TcpServerConfig.drain` and the admin plane's readiness/drain handlers) makes `/readyz` and the data-plane accept loop react identically to SIGTERM/SIGINT and to `POST /drain`, so in-flight sessions/transactions finish before the process exits. `/openapi.json` and `/pools`/`/pools/{pool}/stats` responses are built directly from the existing `apps/pgpool/src/spec.rs` JSON value and the WI #1289 `pool::BackendPool::stats()`/`PoolStats` accounting so the served contract and the offline `pgpool spec` inventory share one source of truth (R4/AC3) instead of round-tripping through a separately-typed OpenAPI document.
capability_refs:
  - id: standard-operational-endpoints
    role: primary
    gap: served-probes-and-drain-flip
    claim: served-probes-and-drain-flip
    coverage: full
    rationale: "Defines and closes the served-probes-and-drain-flip work root: the served /healthz, /readyz, /metrics, /openapi.json, /docs admin plane on RuntimePlan.admin_bind with drain-aware readiness driven by a shared DrainController across signals and POST /drain, verified by cargo test -p pgpool --test admin_plane (AC1, AC2, AC4)."
  - id: http2-api-list
    role: contributes
    gap: served-contract-matches-offline-spec
    claim: served-contract-matches-offline-spec
    coverage: full
    rationale: "Closes the served-contract-matches-offline-spec work root: the served /openapi.json and route set are built from the same apps/pgpool/src/spec.rs JSON value as `pgpool spec --format openapi`/`--format routes`, with a conformance test diffing served vs offline output (R4, AC3)."
fill_sections: [logic, state-machine, schema, config, unit-test, e2e-test]
---

# pgpool served admin plane — drain-aware readiness

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
(fill)
```

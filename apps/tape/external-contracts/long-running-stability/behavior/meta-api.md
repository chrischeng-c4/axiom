<!-- HANDWRITE-BEGIN gap="missing-generator:logic:1f91e4a8" tracker="pending-tracker" reason="Tape standard operational endpoint contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-long-running-stability-meta-api-ec
summary: Tape operational surface contract for liveness, drain-aware readiness, metrics, OpenAPI, and docs.
fill_sections: [e2e-test]
---

# EC: Long-Running Stability Meta API

Tape adopts the common service operational surface. The routes remain
auth-exempt so Kubernetes probes, metrics scraping, and offline service
diagnostics work while topic operations require authentication.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-long-running-stability-meta-api
    capability_id: standard-operational-endpoints
    claim_id: tape-meta-api-health-ready-metrics-openapi
    contract_id: tape-ops-meta-api-surface
    category: behavior
    command: "cargo test -p tape --test http_transport --test behavior_tape_claim_standard_operational_endpoints -- --nocapture"
    assertions:
      - "GET /healthz stays live and auth-exempt; GET /readyz is drain-aware and auth-exempt."
      - "GET /metrics emits Prometheus text for Tape operations and stays available to the scrape path under required auth."
      - "GET /openapi.json and GET /docs expose the same Tape API described by tape spec offline output."
      - "The operator's liveness and readiness probes use /healthz and /readyz respectively."
```
<!-- HANDWRITE-END -->

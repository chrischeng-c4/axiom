---
id: lumen-ops-meta-api-ec
summary: Operability meta API — liveness, readiness (drain-aware), Prometheus metrics, and version are exposed, auth-bypassed, and k8s-probe aligned.
fill_sections: [e2e-test]
---

# EC: Ops Meta API (health · readiness · metrics · version)

Operate without a DBA: the meta/observability endpoints must exist, bypass auth
(so k8s probes + Prometheus scrape work even when auth is required), and match
the k8s probe paths. Behavior-only contract (pure cargo e2e, no services).

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-ops-meta-api
    capability_id: ops-operability
    claim_id: meta-api-health-ready-metrics-version
    contract_id: ops-meta-api-surface
    category: behavior
    test_path: projects/lumen/tests/behavior_lumen_ops_meta_api.rs
    command: "cargo test -p lumen --test api_e2e -- --nocapture"
    assertions:
      - "GET /healthz (liveness) returns 200 always; GET /readyz returns 200 normally and 503 while draining; both bypass auth."
      - "GET /metrics returns Prometheus text v0.0.4 with the scrape content-type; bypasses auth."
      - "GET /version returns 200 with the build version (and git SHA / build time when available); bypasses auth."
      - "The k8s livenessProbe/startupProbe (/healthz) and readinessProbe (/readyz) paths match the actual endpoints; the Prometheus scrape path (/metrics) matches."
```

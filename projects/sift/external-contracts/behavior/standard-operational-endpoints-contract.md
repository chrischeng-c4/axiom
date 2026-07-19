---
id: standard-operational-endpoints-contract
summary: External contract for One-port standard operational endpoints contract.
fill_sections: [e2e-test]
---

# EC: One-port standard operational endpoints contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: one-port-health-readiness-metrics-contract
    capability_id: standard-operational-endpoints
    claim_id: one-port-health-readiness-metrics
    contract_id: sift.standard_operational_endpoints.v1
    category: behavior
    command: "cargo test -p sift --test ingest_api"
    assertions:
      - "The service exposes /healthz, /readyz, and Prometheus /metrics on its data-plane port."
  - id: served-openapi-and-docs-contract
    capability_id: standard-operational-endpoints
    claim_id: served-openapi-and-docs
    contract_id: sift.standard_operational_endpoints.v1
    category: behavior
    command: "cargo test -p sift --test ingest_api"
    assertions:
      - "The service exposes /openapi.json and /docs on that same port."
```

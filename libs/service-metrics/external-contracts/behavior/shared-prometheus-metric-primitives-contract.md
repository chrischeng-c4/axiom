---
id: shared-prometheus-metric-primitives-contract
summary: External contract for Shared Prometheus Metric Primitives.
fill_sections: [e2e-test]
---

# EC: Shared Prometheus Metric Primitives

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-prometheus-metric-primitives-contract
    capability_id: shared-prometheus-metric-primitives
    claim_id: shared-prometheus-metric-primitives-contract
    contract_id: shared-prometheus-metric-primitives-contract
    category: behavior
    command: "cargo test -p service-metrics"
    assertions:
      - "Shared Prometheus Metric Primitives public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

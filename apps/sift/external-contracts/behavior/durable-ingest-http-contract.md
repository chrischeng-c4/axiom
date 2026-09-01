---
id: durable-ingest-http-contract
summary: External contract for Durable h2c event ingest contract.
fill_sections: [e2e-test]
---

# EC: Durable h2c event ingest contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: durable-ingest-http-contract
    capability_id: operational-event-ingest
    claim_id: h2c-openapi-event-write-route
    contract_id: sift.durable_ingest_http.v1
    category: behavior
    command: "cargo test -p sift --test ingest_api"
    assertions:
      - "The h2c /v1/events route accepts validated six-signal envelopes and acknowledges only after durable storage."
      - "A repeated event id is idempotent across journal restart and replay returns the persisted envelope."
      - "The service exposes its OpenAPI and standard operational endpoints alongside the ingest contract."
```

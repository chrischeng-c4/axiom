---
id: lumen-competitor-feature-parity-functional-ec
summary: Competitive feature parity functional correctness — query planner, HTTP API, and search-flavor results.
fill_sections: [e2e-test]
---

# EC: Competitive Feature Parity Functional Behavior

The feature-parity capability needs a behavior contract: does the running engine
return correct search results across the surfaces it claims to replace? These
are pure cargo tests with no external service, so they gate production.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-competitor-feature-parity-api-and-search
    capability_id: competitor-feature-parity
    claim_id: query-planner-boolean-eval-roaring-postings
    contract_id: serve-functional-api-and-search-correctness
    category: behavior
    command: "cargo test -p lumen --test api_e2e --test vector_e2e --test planner_diff -- --nocapture"
    assertions:
      - "The HTTP API end-to-end (create -> index -> search -> hydrate ids) returns correct ranked external_ids and never documents."
      - "Vector kNN and filtered kNN return the nearest within the filter without recall collapse."
      - "The query planner produces byte-identical plans (planner_diff) across the search-flavor sub-capabilities."
```

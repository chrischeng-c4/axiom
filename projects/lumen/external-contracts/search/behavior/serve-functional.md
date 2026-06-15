---
id: lumen-serve-functional-ec
summary: Serve-pillar functional correctness — query planner, HTTP API, and search-flavor results.
fill_sections: [e2e-test]
---

# EC: Serve Functional Behavior

The serve pillar needs a 功能 (behavior) contract, not only efficiency/security/
stability — does the running engine return correct results? These are pure cargo
tests (no external service), so they gate production.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-serve-functional-api-and-search
    capability_id: search
    claim_id: query-planner-boolean-eval-roaring-postings
    contract_id: serve-functional-api-and-search-correctness
    category: behavior
    command: "cargo test -p lumen --test api_e2e --test vector_e2e --test planner_diff -- --nocapture"
    assertions:
      - "The HTTP API end-to-end (create -> index -> search -> hydrate ids) returns correct ranked external_ids and never documents."
      - "Vector kNN and filtered kNN return the nearest within the filter without recall collapse."
      - "The query planner produces byte-identical plans (planner_diff) across the search-flavor sub-capabilities."
```

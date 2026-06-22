---
id: lumen-security-hardening-search-ec
summary: Security hardening — RBAC-filtered results, pagination limits, query-injection safety, and score-leak prevention across filtering / ranking / pagination.
fill_sections: [e2e-test, tool-contract]
---

# EC: Security Hardening Search Boundary

Search must enforce access control on results, bound result size, resist
adversarial queries, and avoid leaking hit existence or scores across isolation
boundaries. guard owns the static posture scan; the dynamic RBAC/limit and
query-safety behavior runs as cargo e2e; meter supplies DoS / resource-abuse
evidence.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-security-hardening-access-control
    capability_id: security-hardening
    claim_id: role-based-authz-matrix-per-route
    contract_id: search-security-rbac-and-limit
    category: security
    test_path: projects/lumen/tests/security_lumen_security_hardening_access_control.rs
    command: "cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture"
    assertions:
      - "FILTERING: search over a collection the token cannot read returns 403; results never leak rows outside the caller's RBAC scope."
      - "PAGINATION: bulk/index requests over MAX_INDEX_ITEMS return 413; result pages are bounded (cursor), not unbounded."
  - id: lumen-security-hardening-query-injection
    capability_id: security-hardening
    claim_id: adversarial-query-safety
    contract_id: search-security-injection
    category: security
    test_path: projects/lumen/tests/security_lumen_security_hardening_query_injection.rs
    command: "cargo test -p lumen --test coverage_gaps_e2e search_security_query_injection_rejects_bad_queries -- --nocapture"
    assertions:
      - "C2: malformed JSON, deeply-nested JSON query DSL, special-char search text, inverted ranges, and range numeric overflow are rejected or evaluated safely (no panic, no 5xx, bounded work)."
  - id: lumen-security-hardening-result-leak
    capability_id: security-hardening
    claim_id: score-confidentiality
    contract_id: search-security-result-leak
    category: security
    test_path: projects/lumen/tests/security_lumen_security_hardening_result_leak.rs
    command: "cargo test -p lumen --test coverage_gaps_e2e search_security_result_leak_respects_collection_boundaries -- --nocapture"
    assertions:
      - "C3: relevance scores and hit existence do not leak documents across collection boundaries; RBAC denial coverage remains pinned by the authz matrix case."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-guard-search-security
    tool: guard
    manifest: guard-search.toml
    category: security
    command: "target/debug/guard scan projects/lumen --compact --no-persist"
    native:
      version: 1
      project: lumen
      source_contract: lumen-security-hardening-access-control
      target: projects/lumen
  - id: lumen-meter-search-security
    tool: meter
    manifest: meter-search-security.toml
    category: security
    command: "target/debug/meter test -- -p lumen --test api_e2e -- --ignored"
    native:
      version: 1
      project: lumen
      source_contract: lumen-security-hardening-access-control
      delegate_command: "target/debug/meter test -- -p lumen --test api_e2e -- --ignored"
```

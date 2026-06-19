---
id: lumen-search-security-ec
summary: Search security — RBAC-filtered results, pagination limits, and (tracked gaps) query-injection + score-leak across filtering / ranking / pagination.
fill_sections: [e2e-test, tool-contract]
---

# EC: Search Security (filtering · ranking · pagination)

Search must enforce access control on results, bound result size, and resist
adversarial queries. guard owns the static posture scan; the dynamic RBAC/limit
behavior runs as cargo e2e; meter supplies DoS / resource-abuse evidence. Two
cells are tracked gaps (no test yet) — defined here so the gate exists. Because
search is a Service capability, security is production-required, so these gaps
block production until their tests land.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-search-security-access-control
    capability_id: search
    claim_id: per-route-rbac-result-filtering
    contract_id: search-security-rbac-and-limit
    category: security
    test_path: projects/lumen/tests/security_lumen_search_security_access_control.rs
    command: "cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture"
    assertions:
      - "FILTERING: search over a collection the token cannot read returns 403; results never leak rows outside the caller's RBAC scope."
      - "PAGINATION: bulk/index requests over MAX_INDEX_ITEMS return 413; result pages are bounded (cursor), not unbounded."
  - id: lumen-search-security-query-injection
    capability_id: search
    claim_id: adversarial-query-safety
    contract_id: search-security-injection
    category: security
    test_path: projects/lumen/tests/security_lumen_search_security_query_injection.rs
    command: ""
    assertions:
      - "GAP (C2): malformed / oversized / deeply-nested JSON query DSL, special-char search text, and range numeric overflow are rejected safely (no panic, no UB, bounded work). Test not yet written."
  - id: lumen-search-security-result-leak
    capability_id: search
    claim_id: score-confidentiality
    contract_id: search-security-result-leak
    category: security
    test_path: projects/lumen/tests/security_lumen_search_security_result_leak.rs
    command: ""
    assertions:
      - "GAP (C3): relevance scores and hit existence do not leak documents across collection / RBAC boundaries. Confidentiality contract + test not yet written."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-guard-search-security
    tool: guard
    manifest: guard-search.toml
    category: security
    command: "guard scan projects/lumen --compact --no-persist"
    native:
      version: 1
      project: lumen
      source_contract: lumen-search-security-access-control
      target: projects/lumen
  - id: lumen-meter-search-security
    tool: meter
    manifest: meter-search-security.toml
    category: security
    command: "target/debug/meter test -- -p lumen --test api_e2e -- --ignored"
    native:
      version: 1
      project: lumen
      source_contract: lumen-search-security-access-control
      delegate_command: "target/debug/meter test -- -p lumen --test api_e2e -- --ignored"
```

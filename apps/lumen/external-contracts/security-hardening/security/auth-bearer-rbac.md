---
id: lumen-security-hardening-auth-bearer-rbac-ec
summary: Security-hardening contract for bearer-token authentication and route authorization.
fill_sections: [e2e-test, tool-contract]
---

# EC: Security Hardening Auth Bearer RBAC

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-security-hardening-auth-bearer-rbac
    capability_id: security-hardening
    claim_id: kubernetes-native-request-identity-and-authorization
    contract_id: bearer-token-auth-lumen-auth
    category: security
    command: "cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture"
    assertions:
      - "Under LUMEN_AUTH=required both a missing credential and an unresolvable one are rejected with 401, and the process refuses to start instead of serving an open API. (#2871 retired the bearer registry, so there is no valid token to accept until TokenReview lands.)"
      - "The authz matrix covers every API route from both sides - open server answers, required-auth server returns 401 - so route coverage cannot be faked by a route that no longer exists. (#2871 removed the read/write/admin dimension; it returns with SubjectAccessReview.)"
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-guard-auth-surface
    tool: guard
    manifest: guard.toml
    category: security
    command: "cargo run --quiet -p guard-cli --bin guard -- scan apps/lumen --compact --no-persist"
    native:
      version: 1
      project: lumen
      source_contract: lumen-security-hardening-auth-bearer-rbac
      target: apps/lumen
```

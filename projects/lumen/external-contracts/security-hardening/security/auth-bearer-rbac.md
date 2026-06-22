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
    claim_id: bearer-token-auth-lumen-auth
    contract_id: bearer-token-auth-lumen-auth
    category: security
    command: "cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture"
    assertions:
      - "Bearer-token auth rejects missing and invalid tokens when LUMEN_AUTH=required; accepts valid tokens."
      - "Per-route RBAC authz matrix enforces each token's role permissions on every API route (read vs write vs admin)."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-guard-auth-surface
    tool: guard
    manifest: guard.toml
    category: security
    command: "target/debug/guard scan projects/lumen --compact --no-persist"
    native:
      version: 1
      project: lumen
      source_contract: lumen-security-hardening-auth-bearer-rbac
      target: projects/lumen
```

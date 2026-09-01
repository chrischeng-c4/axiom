<!-- HANDWRITE-BEGIN gap="sift-auth-external-contract" tracker="1607" reason="Declare the bearer-auth and operational-probe external contract." -->
---
id: sift-security-hardening-bearer-auth-ec
summary: Security contract for Sift bearer authentication and probe exemptions.
fill_sections: [e2e-test, tool-contract]
---

# EC: Sift Bearer Authentication

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: sift-security-hardening-bearer-auth
    capability_id: security-hardening
    claim_id: shared-bearer-token-auth
    contract_id: sift.bearer_auth.v1
    category: security
    command: "cargo test -p sift --test runtime_security_e2e -- --nocapture"
    assertions:
      - "Required bearer authentication rejects unauthenticated data-plane requests and accepts an authorized write token."
      - "Health, readiness, metrics, OpenAPI, and docs probes remain reachable without a bearer token."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: sift-guard-auth-surface
    tool: guard
    manifest: guard.toml
    category: security
    command: "target/debug/guard scan apps/sift --compact --no-persist"
    native:
      version: 1
      project: sift
      source_contract: sift-security-hardening-bearer-auth
      target: apps/sift
```
<!-- HANDWRITE-END -->

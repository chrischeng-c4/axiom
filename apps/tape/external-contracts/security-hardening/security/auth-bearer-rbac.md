<!-- HANDWRITE-BEGIN gap="missing-generator:logic:92d0e1f0" tracker="pending-tracker" reason="Tape shared bearer-token and route-role authorization contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-security-hardening-auth-bearer-rbac-ec
summary: Bearer-token authentication and topic-role authorization contract for Tape.
fill_sections: [e2e-test]
---

# EC: Security Hardening Auth Bearer RBAC

Tape adopts the shared bearer contract from `service-auth`; Tape itself owns
the policy mapping of producer, consumer, and administrator roles onto topic
append, replay, and checkpoint operations.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-security-hardening-auth-bearer-rbac
    capability_id: security-hardening
    claim_id: tape-bearer-topic-role-auth
    contract_id: tape-bearer-token-topic-rbac
    category: security
    command: "cargo test -p tape --test service_auth -- --nocapture"
    assertions:
      - "When TAPE_AUTH=required, missing and unknown bearer tokens are rejected; a reader cannot append to a topic."
      - "Topic-scoped write grants authorize append, topic-scoped read grants authorize replay and checkpoint operations, and wildcard administrator grants cover every topic."
      - "The required-mode registry fails fast for missing, malformed, empty, or unknown auth configuration; TAPE_AUTH=off remains the explicit local tokenless mode."
      - "Auth rejection uses the shared unauthenticated/forbidden envelope and the standard probe, metrics, OpenAPI, and docs routes remain tokenless."
```
<!-- HANDWRITE-END -->

<!-- HANDWRITE-BEGIN gap="missing-generator:logic:89579b13" tracker="pending-tracker" reason="Tape topic/subscription authorization, admission-limit, and malformed-request security contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-security-hardening-access-control-ec
summary: Topic and subscription authorization, bounded write admission, credential rotation, and shared redacted-audit/error-shape contracts for Tape.
fill_sections: [e2e-test]
---

# EC: Security Hardening Access Control

Lumen's access-control EC category maps to Tape topic and subscription scopes,
not to search collection visibility, pagination, or relevance-score secrecy.
The existing guard EC remains the static posture evidence for this dynamic
authorization contract.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-security-hardening-access-control
    capability_id: security-hardening
    claim_id: tape-topic-subscription-authz-boundary
    contract_id: tape-topic-security-rbac-and-admission
    category: security
    command: "cargo test -p service-auth -- --nocapture && cargo test -p tape --test service_auth --test service_admission -- --nocapture"
    assertions:
      - "Appending to a topic requires that topic's write grant."
      - "Replay and checkpoint operations require that topic's read grant and never expose data from an unauthorized topic."
      - "Authentication failures retain the shared service-auth error shape while operational probes remain tokenless."
      - "Append is classified as write admission and a configured shared policy returns bounded 429 responses without limiting probes."
      - "The projected token registry rotates atomically without restarting Tape, while the shared service-auth suite independently verifies credential-free authorization audit events."
```
<!-- HANDWRITE-END -->

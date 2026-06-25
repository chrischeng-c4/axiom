---
id: per-namespace-key-isolation-x-keep-namespace-applied-after-token
capability_refs:
  - id: "security-hardening"
    role: primary
    gap: "auth-tls-network-policy-boundary"
    claim: "auth-tls-network-policy-boundary"
    coverage: partial
    rationale: "Adds per-namespace storage-key isolation on the claim-check data plane (X-Keep-Namespace applied after the token scope check), one access-boundary dimension toward the security-hardening negative gates."
fill_sections: [unit-test, changes]
---

# Keep Per-Namespace Key Isolation

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-ns-isolation-unit-test
coverage_kind: behavioral
strategy: prove namespace isolation, back-compat, and token-vs-bare-key ordering with hand-written HTTP API tests
tests:
  - id: keep-ns-isolates-same-bare-key
    capability_id: security-hardening
    command: "cargo test -p keep --test http_api -- --nocapture"
    generated: false
  - id: keep-ns-absent-is-backcompat
    capability_id: security-hardening
    command: "cargo test -p keep --test http_api -- --nocapture"
    generated: false
  - id: keep-ns-token-checks-bare-key
    capability_id: security-hardening
    command: "cargo test -p keep --test http_api -- --nocapture"
    generated: false
---
requirementDiagram

requirement KEEP_NS_ISOLATES_SAME_BARE_KEY {
  id: keep-ns-isolates-same-bare-key
  text: Two requests with different X-Keep-Namespace values writing the same bare claim-check key do not collide; each namespace reads back only its own value.
  risk: high
  verifymethod: test
}

requirement KEEP_NS_ABSENT_IS_BACKCOMPAT {
  id: keep-ns-absent-is-backcompat
  text: With no X-Keep-Namespace header the storage key stays the bare 'kind:id', preserving existing single-tenant behavior.
  risk: medium
  verifymethod: test
}

requirement KEEP_NS_TOKEN_CHECKS_BARE_KEY {
  id: keep-ns-token-checks-bare-key
  text: The claim-check token scope is verified against the bare key from the URL before the namespace prefix is applied, so a valid in-scope token works under any namespace.
  risk: high
  verifymethod: test
}
```

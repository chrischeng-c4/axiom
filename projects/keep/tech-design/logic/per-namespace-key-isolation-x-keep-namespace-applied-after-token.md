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

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: behavioral
changes:
  - path: projects/keep/src/http/handlers.rs
    action: modify
    impl_mode: hand-written
    description: |
      claim_get / claim_put gain a namespace argument; the engine storage key
      becomes `{ns}::{kind}:{id}` when the namespace is non-empty, and stays the
      bare `{kind}:{id}` otherwise (preserving single-tenant behavior). The four
      claim-check wrappers (get_input / put_input / get_result / put_result)
      extract the `X-Keep-Namespace` header and pass it through. The namespace
      prefix is applied AFTER check_scope verifies the claim-check token against
      the bare key from the URL, so token scope stays bare per the settled loom
      design.
  - path: projects/keep/tests/http_api.rs
    action: modify
    impl_mode: hand-written
    description: |
      Hand-written namespace behavior tests: (1) isolation — two requests with
      different X-Keep-Namespace values writing the same bare key do not collide
      and each namespace reads back only its own value; (2) back-compat — absent
      header keeps the bare key; (3) token-checks-bare-key — an in-scope token
      verifies against the bare URL key and works under any namespace.
```

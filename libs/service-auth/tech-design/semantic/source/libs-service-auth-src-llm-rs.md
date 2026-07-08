---
id: libs-service-auth-src-llm-rs
summary: Lossless rust-source-unit coverage for `libs/service-auth/src/llm.rs`.
capability_refs:
  - id: shared-http-request-auth-middleware
    role: primary
    claim: shared-http-request-auth-middleware-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Auth library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-auth/src/llm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-auth/src/llm.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TOPIC` | libs/service-auth/src/llm.rs | const | pub | 4 | pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic { |
| `topic` | libs/service-auth/src/llm.rs | function | pub | 52 | pub fn topic() -> &'static cli_std::llm::Topic { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! LLM topic provider for the shared service-auth contract.

/// Agent-facing topic describing the reusable auth primitive.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "service-auth",
    summary: "Shared bearer-token role-map auth: token registry, wildcard grants, and verifier middleware.",
    body: r#"# service-auth shared topic

## Runtime contract
Services that use the static role-map shape expose service-owned env names:

```env
<SVC>_AUTH=off|required
<SVC>_TOKEN_REGISTRY_FILE=/var/run/secrets/<svc>/token-registry.json
```

Clients send:

```http
Authorization: Bearer <token>
```

`service-auth` owns bearer extraction, verifier middleware, static token
registry loading, and principal injection. Each service owns its public env
prefix, resource names, route exemptions, and handler-level authorization.

## token registry shape
The registry is a JSON object keyed by exact bearer token. Each token maps to a
subject plus resource-role grants:

```json
{
  "admin-token": {
    "subject": "ops",
    "roles": { "*": "admin" }
  },
  "reader-token": {
    "subject": "reader",
    "roles": { "products": "read" }
  }
}
```

Roles are `read`, `write`, or `admin`; `admin` covers `write` and `read`, and
`write` covers `read`. The literal resource key `*` grants across all
resources. Missing or insufficient grants should reject at the service handler
with 403.
"#,
};

/// Return the shared auth topic for CLI composition.
pub fn topic() -> &'static cli_std::llm::Topic {
    &TOPIC
}

#[cfg(test)]
mod tests {
    #[test]
    fn llm_topic_is_nonempty() {
        let topic = super::topic();
        assert_eq!(topic.id, "service-auth");
        assert!(topic.body.contains("<SVC>_TOKEN_REGISTRY_FILE"));
        assert!(topic.body.contains("Authorization: Bearer"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-auth/src/llm.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-auth/src/llm.rs` captured during libs codegen standardization.
```

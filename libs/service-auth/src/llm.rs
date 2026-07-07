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

//! LLM topic provider for the shared OpenAPI codegen contract.

/// Agent-facing topic describing generated-client composition.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "openapi-codegen",
    summary:
        "Typed TypeScript, Python, and Rust client generation from a service OpenAPI document.",
    body: r#"# openapi-codegen shared topic

## CLI composition
Services should feed their own OpenAPI document into the shared generator and
expose the service-specific command shape:

```text
<cli> spec gen --lang ts|py|rust --out <dir>
```

The generator core is pure: `generate(spec_json, GenOptions)` returns an
in-memory `GeneratedOutput`, and `run(GenOptions)` is only the filesystem-writing
CLI helper.

## Emitted clients
- TypeScript emits types, a typed fetch/axios client, and optional TanStack
  Query hooks.
- Python emits pydantic models plus sync/async HTTP/2-capable runtime clients.
- Rust emits serde models plus a reqwest client.

Generated clients expose the same protocol-neutral transport policy in
TypeScript, Python, and Rust:

```text
target_concurrency        sizing hint for expected per-origin concurrency
max_connections           default 128 abstract client-side cap
max_keepalive_connections default 16 idle connection cap
max_in_flight_per_origin  hard admission cap; excess requests wait client-side
pool_timeout              default 5s queue wait before pool timeout
```

HTTP/2-capable runtimes map `target_concurrency` to
`ceil(ln(concurrency))` physical connections where the runtime controls the
pool. HTTP/1.1 runtimes keep the same knobs but need more sockets. The peer
server still determines practical request concurrency; the generated client
only bounds the application's in-flight queue and prevents unbounded async
fan-out.

Services own authentication headers, base URL defaults, command naming, and
which generated files are considered public artifacts.
"#,
};

/// Return the shared generated-client topic for CLI composition.
pub fn topic() -> &'static cli_std::llm::Topic {
    &TOPIC
}

#[cfg(test)]
mod tests {
    #[test]
    fn llm_topic_is_nonempty() {
        let topic = super::topic();
        assert_eq!(topic.id, "openapi-codegen");
        assert!(topic
            .body
            .contains("spec gen --lang ts|py|rust --out <dir>"));
        assert!(topic.body.contains("GeneratedOutput"));
        assert!(topic.body.contains("target_concurrency"));
        assert!(topic.body.contains("max_in_flight_per_origin"));
        assert!(topic.body.contains("pool_timeout"));
    }
}

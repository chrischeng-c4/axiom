// CODEGEN-BEGIN
//! LLM topic provider for the shared OpenAPI codegen contract.

/// Agent-facing topic describing generated-client composition.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "openapi-codegen",
    summary:
        "Typed TypeScript, Python, and Rust client generation from a service OpenAPI document.",
    body: r#"# openapi-codegen shared topic

This library owns the shared generator behavior below. Service CLIs import this
topic and add only their own command, defaults, authentication, and endpoint
facts. They do not copy this provider text into an app-owned runbook.

## CLI composition
Services should feed their own OpenAPI document into the shared generator and
expose the service-specific command shape:

```text
<cli> spec gen --lang ts|py|rust --out <dir>
```

The generator core is pure: `generate(spec_json, GenOptions)` returns an
in-memory `GeneratedOutput`, and `run(GenOptions)` is only the filesystem-writing
CLI helper.

## Target profiles and manifest
An explicit target profile pins supported Python, TypeScript, or Rust syntax.
It also emits `.openapi-codegen.json` with the selected target and current
requirements. The manifest does not yet contain complete OpenAPI provenance,
service compatibility, or every runtime dependency.

Generated source is a caller-vendored build artifact. This library does not
publish npm, PyPI, or crates.io packages.

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

## Private trust
Generated runtimes that accept a private CA replace the public roots with that
anchor. They verify the name addressed by the base URL. They do not expose a
skip-verification mode.

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
        assert!(topic.body.contains("Service CLIs import this"));
        assert!(topic.body.contains("provider text"));
        assert!(topic.body.contains("target profile"));
        assert!(topic.body.contains(".openapi-codegen.json"));
        assert!(topic.body.contains("replace the public roots"));
        assert!(topic.body.contains("skip-verification mode"));
        assert!(topic.body.contains("This library does not"));
        assert!(topic.body.contains("publish npm, PyPI, or crates.io"));
        assert!(topic.body.contains("target_concurrency"));
        assert!(topic.body.contains("max_in_flight_per_origin"));
        assert!(topic.body.contains("pool_timeout"));
    }
}
// CODEGEN-END

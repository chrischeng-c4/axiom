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
    }
}

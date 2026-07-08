---
id: libs-openapi-codegen-src-llm-rs
summary: Lossless rust-source-unit coverage for `libs/openapi-codegen/src/llm.rs`.
capability_refs:
  - id: multi-language-openapi-client-generation
    role: primary
    claim: multi-language-openapi-client-generation-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Openapi Codegen library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/openapi-codegen/src/llm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/openapi-codegen/src/llm.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TOPIC` | libs/openapi-codegen/src/llm.rs | const | pub | 4 | pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic { |
| `topic` | libs/openapi-codegen/src/llm.rs | function | pub | 52 | pub fn topic() -> &'static cli_std::llm::Topic { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/openapi-codegen/src/llm.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/openapi-codegen/src/llm.rs` captured during libs codegen standardization.
```

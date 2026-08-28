# openapi-codegen status

## Scope

This document describes the current source contract for the reusable
`openapi-codegen` crate. It does not claim that the named gate ran in this
working session.

Use the [README](README.md) for the generation workflow. Use the
[roadmap](ROADMAP.md) for future outcomes and explicit non-goals.

## State definitions

| State | Meaning |
|---|---|
| Supported | The current source has a public contract, an implementation, and a named executable gate for the stated scope. |
| Limited | The current source supports the stated scope, but the Limits cell names a material boundary. |
| Not supported | The behavior is not part of the current product contract. The Evidence cell points to a future outcome or a non-goal. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| TypeScript, Python, and Rust generation | `three-language-generation` | Supported | One OpenAPI document can produce typed models and operation methods for all three languages. | The generator supports its modeled OpenAPI subset, not every possible extension. | `cargo test -p openapi-codegen` |
| Versioned target profiles | `versioned-target-profiles` | Supported | Explicit Python, TypeScript, and Rust profiles control compatible syntax and emit deterministic target metadata. | No target keeps the legacy output and emits no manifest. | `cargo test -p openapi-codegen` |
| QUERY and POST fallback | `query-post-fallback` | Supported | OpenAPI 3.2 `query` operations emit an HTTP `QUERY` method and a runtime POST-twin choice in all three languages. | The service contract must provide a valid sibling or `x-post-twin` path. | `cargo test -p openapi-codegen` |
| Arbitrary additional operations | `arbitrary-additional-operations` | Limited | `additionalOperations` entries parse into the shared operation model without failing generation. | Emitters do not create typed methods for arbitrary methods beyond the dedicated `QUERY` path. See [Additional operation emission](ROADMAP.md#additional-operation-emission). | `cargo test -p openapi-codegen` |
| JSON operation model | `json-operation-model` | Limited | The shared operation model reads `application/json` request bodies and JSON responses for generated methods. | Non-JSON media types and streaming bodies are not represented. See [Media type and streaming operations](ROADMAP.md#media-type-and-streaming-operations). | `cargo test -p openapi-codegen` |
| Streaming operation emission | `streaming-operation-emission` | Not supported | Generated clients do not expose bounded upload or incremental response methods for streaming operations. | NDJSON and other streaming operations require a manual client today. | [Media type and streaming operations](ROADMAP.md#media-type-and-streaming-operations) |
| Structured client errors | `structured-client-errors` | Not supported | Generated methods surface transport-level HTTP failures without decoding a service error schema. | Callers do not receive one generated typed error for declared non-success responses. | [Structured client errors](ROADMAP.md#structured-client-errors) |
| Rust schema type parity | `rust-schema-type-parity` | Limited | Rust emits ordinary object fields and scalar types for the supported schema subset. | Unions fall back to `serde_json::Value`, and enums fall back to `String`. See [Cross-language type parity](ROADMAP.md#cross-language-type-parity). | `cargo test -p openapi-codegen` |
| Private service trust | `private-service-trust` | Supported | Generated clients can replace public roots with a supplied private CA and require the base URL to match the asserted server name. | The caller must distribute the public CA material. No skip-verification mode exists. | `cargo test -p openapi-codegen` |
| Static request auth values | `static-request-auth-values` | Limited | TypeScript accepts fixed default headers. Python accepts a fixed token or fixed default headers at construction. | Rust has no equivalent static constructor input. The opt-in file-bearer extension is the separate path. See [Dynamic request auth provider](ROADMAP.md#dynamic-request-auth-provider). | `cargo test -p openapi-codegen --locked` |
| Target dependency manifest | `target-dependency-manifest` | Limited | Explicit target profiles emit deterministic target metadata next to generated source. | The manifest does not yet record complete OpenAPI and generator provenance, caller-supplied compatibility, or every runtime dependency. See [Complete target dependency manifest](ROADMAP.md#complete-target-dependency-manifest). | `cargo test -p openapi-codegen` |
| Dynamic per-request auth provider | `dynamic-per-request-auth-provider` | Limited | An explicit generation-time generic file-bearer extension rereads a configured file once before each eligible request and fails before transport on provider errors. | Legacy generation remains default-off. A general injected provider, cancellation contract, and app-owned identity policy remain future work. See [Dynamic request auth provider](ROADMAP.md#dynamic-request-auth-provider). | `cargo test -p openapi-codegen --locked` |
| Operation-aware retry hooks | `operation-aware-retry-hooks` | Not supported | Generated clients do not expose a shared hook for an app-supplied retry policy, deadline, cancellation, backoff, or `Retry-After`. | The generator must not guess which service writes are idempotent. | [Operation-aware retry hooks](ROADMAP.md#operation-aware-retry-hooks) |
| Strict cross-language execution gate | `strict-cross-language-execution-gate` | Not supported | No required harness proves generated TypeScript, Python, and Rust output in one clean run. | A consumer gate can still skip a language when its toolchain is absent. | [Strict cross-language generation gates](ROADMAP.md#strict-cross-language-generation-gates) |
| Credential acquisition from OpenAPI | `credential-acquisition-from-openapi` | Not supported | A bearer security scheme does not make the generator obtain a token or select an identity system. | Apps and auth libraries own credential sources and service policy. | [Identity policy and token acquisition](ROADMAP.md#identity-policy-and-token-acquisition) |
| Package publication | `package-publication` | Not supported | The library emits source and target metadata into a caller-selected directory. | It does not publish or maintain generated npm, PyPI, or crates.io packages. | [Package publication](ROADMAP.md#package-publication) |

## Evidence policy

The commands above are required gates for each supported scope. This document
does not store execution output. CI and local test logs own run evidence.

Update a row with any public API, behavior, or evidence change. Move a future
outcome into current support only after its implementation and executable gate
exist.

# openapi-codegen

## Brief

`openapi-codegen` generates typed TypeScript, Python, and Rust API clients from
OpenAPI 3.0 and 3.1 documents.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Multi-Language OpenAPI Client Generation | - | implemented | verified | smoke | ready | emits typed clients for supported target languages |

### Multi-Language OpenAPI Client Generation

ID: multi-language-openapi-client-generation
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `cclab_openapi_codegen`.
EC Dimensions: behavior: `cargo test -p cclab-openapi-codegen` - OpenAPI parser and emitter coverage
Required Verification: smoke
Promise:
Projects can derive typed client code from OpenAPI documents without copying
language-specific generation logic.
Gate Inventory: `cargo test -p cclab-openapi-codegen`; libs/openapi-codegen/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| multi-language-openapi-client-generation-contract | epic | - | implemented | verified | smoke | `cargo test -p cclab-openapi-codegen`; libs/openapi-codegen/src/lib.rs |

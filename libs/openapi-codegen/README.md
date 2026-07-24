# openapi-codegen

## Brief

`openapi-codegen` generates typed TypeScript, Python, and Rust API clients from
OpenAPI 3.0, 3.1, and 3.2 documents, including OpenAPI 3.2's `query` path-item
keyword (the HTTP `QUERY` method, RFC 10008) and `additionalOperations` map.

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

## Versioned target profiles

`generate` keeps the ergonomic language-only API and chooses a conservative
default profile: Python 3.11, TypeScript 5.0, or Rust 2021. Consumers that need
an exact generated-artifact contract call `generate_for_target` with a profile;
the returned `GeneratedOutput` carries the selected `target` and deterministic
`requirements` (minimum version and generated-runtime dependencies).

Projects pin their defaults in a `codegen.toml` `[targets]` table and may offer
an explicit target override. When an output is materialized through
`GeneratedOutput::write_to_dir`, it always includes
`.cclab-openapi-codegen.json`, recording the exact target, minimum version,
and runtime dependencies for downstream verification.

| Language | Profiles | Artifact effect |
|---|---|---|
| Python | 3.11, 3.12, 3.13, 3.14 | All use native `T \| None` and `Self`; 3.12+ also uses PEP 695 `type` aliases. |
| TypeScript | 5.0 | Records the compiler floor. The current emitter has no safe version-specific syntax improvement, so the generated API stays identical. |
| Rust | 2021, 2024 | Rust 2024 treats schema fields named `gen` as reserved and emits `gen_` plus `#[serde(rename = "gen")]`. |

```rust
use cclab_openapi_codegen::{
    generate_for_target, GenOptions, PythonTarget, TargetProfile,
};

let output = generate_for_target(
    spec_json,
    &options,
    TargetProfile::Python(PythonTarget::Py312),
)?;
assert_eq!(output.requirements.minimum_version, "3.12");
```

## OpenAPI 3.2 and HTTP QUERY

There is no hard version gate: `Spec.openapi` is parsed as an opaque string,
so 3.0, 3.1, 3.2, and any other `3.x.y` document parse the same way. OpenAPI
3.2 adds two path-item keywords, both modeled on `ir::openapi::PathItem`:

- `query` — a sibling of `get`/`post`/... for the HTTP `QUERY` method (RFC
  10008): a read operation that carries a JSON request body. It is treated as
  a query-shaped operation end to end (e.g. TanStack Query hooks in the
  TypeScript emitter), and every generator emits a client method that sends
  an actual `QUERY` request with the JSON body:
  - **TypeScript**: `method: "QUERY"` via `fetch`/`axios` (both runtimes
    accept arbitrary method strings).
  - **Python**: `self._client.request("QUERY", ..., json=...)` against the
    generated `h2c_runtime.py` client or any injected httpx-like object.
  - **Rust**: `reqwest::Method::from_bytes(b"QUERY")` — `reqwest::blocking::
    Client` has no dedicated `.query()` verb method (that name is already the
    querystring builder), so the generated client dispatches through
    `self.http.request(method, url)` instead.
- `additionalOperations` — a map of UPPERCASE HTTP method name → operation for
  methods with no dedicated keyword (e.g. `PURGE`). These parse without
  choking and are included in the language-neutral operation IR
  (`ir::operations::build`); a method already covered by a dedicated keyword
  (including `query`) is not duplicated. Generating typed client methods for
  `additionalOperations` entries beyond `QUERY` is not yet wired into the
  emitters — parse-don't-crash is the bar for those today.

### POST-twin fallback (epic #1296 policy)

Epic #1296 establishes that every `QUERY` endpoint has a POST twin. Each
generated client exposes a **runtime** (not build-time) option that routes a
`QUERY` operation's call through its POST twin instead:

- **TypeScript**: `ClientConfig.usePostFallback?: boolean` (constructor
  config passed to the `createClient`-style factory).
- **Python**: `Client(..., use_post_fallback: bool = False)` /
  `AsyncClient(..., use_post_fallback: bool = False)`.
- **Rust**: `Client::with_post_fallback(bool) -> Self` builder method.

Twin resolution, computed once in the shared IR
(`ir::operations::OperationIR::post_twin_path`) and consumed identically by
all three emitters:

1. If the `query` operation carries the vendor extension
   `x-post-twin: "<path>"`, that path is the twin.
2. Otherwise, the twin defaults to the sibling `post` operation on the same
   path item (OpenAPI 3.2 allows `query` and `post` as sibling keywords on one
   path), i.e. the same path template with method `POST` instead of `QUERY`.

Toggling the fallback flag changes only the emitted `method`/path selection at
call time; the rest of the generated method (parameter grouping, JSON body,
response typing) is unchanged.

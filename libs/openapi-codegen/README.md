# openapi-codegen

## Brief

`openapi-codegen` generates TypeScript, Python, and Rust client source from an
OpenAPI document. It provides versioned language targets, transport policy,
private trust, HTTP `QUERY`, and a runtime POST-twin fallback.

The generator turns an API description into client code. It does not decide
who may call the API or how a caller obtains a credential. An OpenAPI bearer
security scheme describes an Authorization header. It does not define token
discovery, Kubernetes ServiceAccount projection, Fleet policy, or RBAC.

## Primary workflow

1. Load one OpenAPI document and select TypeScript, Python, or Rust.
2. Select a versioned target profile when generated syntax must be pinned.
3. Generate models, operation methods, transport code, and target metadata.
4. Configure the generated client with its base URL, trust, and current static
   request options.

## Generate typed clients

The parser builds one language-neutral operation model. Each emitter creates
models and one method per supported JSON operation. `GeneratedOutput::write_to_dir`
writes the files and, for an explicit target, a deterministic
`.openapi-codegen.json` manifest.

The current manifest does not record the OpenAPI SHA, generator version,
caller-supplied service compatibility, or every runtime dependency. Those are
future output-provenance requirements. Generated source remains a build
artifact. This library does not publish language packages.

The supported languages are TypeScript, Python, and Rust. The generated source
is a build artifact. The OpenAPI document remains the service contract. This
library does not publish npm, PyPI, or crates.io packages.

## Select a target profile

`GenOptions::target` is the explicit target contract. `target: None` preserves
the legacy output and emits no target manifest. `target: Some(profile)` or
`generate_for_target` enables version-aware syntax and records the selected
compiler or language floor.

| Language | Profiles | Current effect |
|---|---|---|
| Python | 3.11, 3.12, 3.13, 3.14 | Native union and `Self` syntax; 3.12 and later use PEP 695 type aliases. |
| TypeScript | 5.0 | Records the compiler floor. |
| Rust | 2021, 2024 | Rust 2024 escapes schema fields such as `gen` and preserves their wire names. |

A project can pin defaults in `codegen.toml` under `[targets]`.

## Use QUERY and POST twins

OpenAPI 3.2 `query` operations produce an HTTP `QUERY` request with a JSON
body. Each generated client also supports a runtime POST fallback. An
`x-post-twin` extension selects a different path. Without that extension, the
sibling POST operation on the same path is the twin.

`additionalOperations` entries are parsed into the language-neutral model.
The emitters do not yet create typed methods for arbitrary methods other than
the dedicated `QUERY` path.

## Configure transport and request headers

Generated clients provide bounded transport settings and private-CA trust. The
private trust replaces public roots. It does not add a private CA to the public
root set. There is no skip-verification option.

Current credential inputs are static:

| Language | Current request credential surface |
|---|---|
| TypeScript | `ClientConfig.headers` can hold a fixed Authorization header. |
| Python | `auth_token` or `default_headers` is copied when the client is created. |
| Rust | No generated default Authorization-header input exists. |

No generated client calls a provider before each request. No generated client
discovers or rotates a Kubernetes ServiceAccount token. See
[current support](STATUS.md) and the
[dynamic provider outcome](ROADMAP.md#dynamic-request-auth-provider).

No generated client applies an operation-aware retry policy. The planned
generic hook will expose request metadata and accept an app-supplied policy. It
will not decide that a service write is safe to retry.

## Understand current output boundaries

The shared operation model reads `application/json` request bodies and JSON
responses. It does not model streaming operations. An operation such as an
NDJSON upload or streaming response therefore has no generated streaming
method today.

Generated methods expose transport-level HTTP failures. They do not decode a
service's structured error envelope into a generated error type. Services own
the meaning of those error bodies. The generator owns the future reusable
mapping mechanism.

Type mapping is also not equal across languages. Rust unions fall back to
`serde_json::Value`, and Rust enums fall back to `String`. The TypeScript target
manifest does not yet list every runtime dependency needed by the emitted
source. See [current support](STATUS.md) for the exact boundaries and
[future outcomes](ROADMAP.md) for completion evidence.

The crate tests all emitters, but it does not yet provide one required
cross-language execution gate that fails when a selected toolchain is missing.

## Contract discovery

| Need | Source of truth |
|---|---|
| Public Rust API | `cargo doc -p openapi-codegen --no-deps` |
| OpenAPI parser and operation model | `libs/openapi-codegen/src/ir/` |
| TypeScript emitter | `libs/openapi-codegen/src/emit/ts/` |
| Python emitter | `libs/openapi-codegen/src/emit/py/` |
| Rust emitter | `libs/openapi-codegen/src/emit/rust/` |
| Target profiles and output manifest | `libs/openapi-codegen/src/target.rs` and `lib.rs` |
| Executable behavior | `cargo test -p openapi-codegen` |

## Capabilities

Every entry below is an equal library capability. Each source states its direct
contribution.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| Multi-language client generation | `multi-language-client-generation` | Generate typed TypeScript, Python, and Rust clients from one OpenAPI document. | `libs/openapi-codegen` |
| Versioned target profiles | `versioned-target-profiles` | Pin generated syntax and runtime requirements to a declared language target. | `libs/openapi-codegen` |
| QUERY and POST-twin dispatch | `query-post-twin-dispatch` | Call OpenAPI `query` operations directly or through their documented POST twin. | `libs/openapi-codegen` |
| Transport and private trust | `transport-private-trust` | Generate bounded transports that can replace public trust with one private CA. | `libs/openapi-codegen` |
| Static request credentials | `static-request-credentials` | Supply fixed request headers in TypeScript and fixed token or header values in Python. | `libs/openapi-codegen` |

### Multi-language client generation

- ID: `multi-language-client-generation`
- Promise: Parse one OpenAPI contract and emit typed models and operation
  methods for TypeScript, Python, and Rust.
- Sources:
  - [`libs/openapi-codegen`](./) provides the parser, language-neutral model,
    name mapping, language emitters, and deterministic output writer.
- Gate: `cargo test -p openapi-codegen`

### Versioned target profiles

- ID: `versioned-target-profiles`
- Promise: Record and apply a selected Python, TypeScript, or Rust target
  without changing legacy output when no profile is selected.
- Sources:
  - [`libs/openapi-codegen`](./) provides target enums, version-aware syntax,
    requirements, project config loading, and the output manifest.
- Gate: `cargo test -p openapi-codegen`

### QUERY and POST-twin dispatch

- ID: `query-post-twin-dispatch`
- Promise: Generate `QUERY` methods and let the caller select their POST twin
  at runtime without changing request or response types.
- Sources:
  - [`libs/openapi-codegen`](./) provides OpenAPI 3.2 query parsing, twin
    resolution, language emission, and runtime fallback controls.
- Gate: `cargo test -p openapi-codegen`

### Transport and private trust

- ID: `transport-private-trust`
- Promise: Apply bounded connection behavior and verify a service against an
  explicit private CA and matching server name.
- Sources:
  - [`libs/openapi-codegen`](./) provides generated transport policy, private
    trust types, name checks, root replacement, and refusal behavior.
- Gate: `cargo test -p openapi-codegen`

### Static request credentials

- ID: `static-request-credentials`
- Promise: Let TypeScript and Python callers attach a fixed bearer value or
  fixed headers without claiming token discovery or rotation.
- Sources:
  - [`libs/openapi-codegen`](./) provides TypeScript default headers and Python
    construction-time token and header inputs.
- Gate: `cargo test -p openapi-codegen`

## Supporting documents

| Document | Use it for |
|---|---|
| [STATUS.md](STATUS.md) | Current language and transport boundaries |
| [ROADMAP.md](ROADMAP.md) | Future generated-client outcomes and non-goals |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Edit rules and required verification |
| [OpenAPI Security Scheme Object](https://spec.openapis.org/oas/latest.html#security-scheme-object) | What an API security scheme describes |

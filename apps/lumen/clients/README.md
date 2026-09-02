# Lumen generated clients

## Contract

[`openapi.json`](openapi.json) is Lumen's committed consumer reference for HTTP
methods, paths, operation IDs, declared request and response schemas, status
codes, media types, and security. It is generated from the same Rust route and
schema source used by the live router.

The OpenAPI document does not own behavior that spans requests. The
[indexing guide](../docs/indexing.md) owns write, durability, and rebuild
meaning. The [querying guide](../docs/querying.md) owns selection, results,
facets, limits, and source hydration. The
[protocol guide](../docs/protocol.md) maps each HTTP fact to its maintained
source. The [client integration guide](../docs/client-integration.md) owns
connection profiles, request resilience, the Kubernetes workload template, and
source-hydration helpers.

Generated language source is an on-demand convenience build. Lumen does not
publish an npm package, a PyPI package, or a crates.io client crate. A consumer
that needs a pinned copy can vendor the generated output.

## Generate

Generate a client from the current Lumen contract:

| Language | Command |
|---|---|
| TypeScript | `cargo run -q -p lumen --bin lumen -- spec gen --lang ts --out apps/lumen/clients/ts` |
| Python | `cargo run -q -p lumen --bin lumen -- spec gen --lang py --out apps/lumen/clients/python` |
| Rust | `cargo run -q -p lumen --bin lumen -- spec gen --lang rust --out apps/lumen/clients/rust` |

Generated language directories are ignored. The committed
[`codegen.toml`](codegen.toml) selects TypeScript 5.0, Python 3.14, and Rust
2024. Each output contains `.openapi-codegen.json` with its selected target and
requirements. Use `--target <profile>` only for a deliberate compatibility
override.

The target manifest is incomplete. The future generated output records the
OpenAPI SHA, generator version, target profile, Lumen compatibility, and every
runtime dependency. It remains generated source, not a published package.

TypeScript defaults to the fetch runtime. Use `--http axios` to generate the
axios runtime. Both forms also emit TanStack Query hooks.

## Language matrix

| Language | Generated form | Transport | Auth input | Current limits |
|---|---|---|---|---|
| TypeScript | Promise-based typed client with fetch or axios runtime and TanStack Query hooks | HTTP/1.1 through fetch or axios; Node-only file-bearer opt-in for eligible cluster URLs | Explicit file-bearer provider or fixed headers | No published package, NDJSON stream method, or typed API error. Browser use hard-fails for eligible token URLs. |
| Python | Pydantic models with sync `Client` and async `AsyncClient` | Generated HTTP/2-capable h2c and TLS runtime | Explicit file-bearer opt-in or fixed headers | No published package, NDJSON stream method, or typed API error. The runtime needs Python and Pydantic 2. |
| Rust | Serde models with a blocking client | `reqwest::blocking` over HTTP/1.1 or negotiated TLS | Explicit file-bearer opt-in | No published package, async client, NDJSON stream method, or typed API error. Component unions become `serde_json::Value`; string enums become `String`. |

All three generated clients support Lumen's dedicated `QUERY` operations and a
runtime POST fallback. They generate ordinary JSON operations. The current
operation model ignores non-JSON request and response content.

## Connect

Standalone in-cluster clients use `http://lumen.lumen.svc.cluster.local:7373`
and the generated Lumen client opts into the Kubernetes default ServiceAccount
token at `/var/run/secrets/kubernetes.io/serviceaccount/token`.

Managed uses `https://<instance>.<namespace>.svc:7373`. Supply the public
serving CA and the Service DNS name that the certificate asserts. Private trust
replaces public roots. No generated client offers skip-verification mode.

OpenAPI declares bearer authentication. The Lumen generation command adds the
explicit generic file-bearer provider for in-cluster HTTP and HTTPS only.

The provider rereads the Kubernetes token before each request. Explicit
Authorization headers always win. External and local hosts do not read it.

`ManagedKsa` remains separate. It requires HTTPS, private CA trust, and the
fixed `/var/run/secrets/lumen.axiom.dev/token` path. It keeps its private
audience contract unchanged. A missing, unreadable, or empty token fails before
transport. A `401` never downgrades to anonymous.
Server-side TokenReview, not client-side token parsing, remains authoritative.

For a human shell, use `lumen connect`. It obtains an audience-bound token
through Kubernetes TokenRequest and keeps it inside the loopback proxy process.
See the [authentication guide](../docs/authentication.md) for the current and
planned identity flows.

## Current boundaries

The [Lumen support matrix](../STATUS.md#support-matrix) is authoritative. The
main generated-client boundaries are:

- `POST /collections/{collection_id}/reindex/stream` uses a `text/plain` NDJSON
  request and a streaming response. No generated client exposes that operation
  correctly today.
- TypeScript fetch rejects a non-success response without decoding the Lumen
  body. Python uses its transport status error. Rust uses
  `error_for_status`. None returns one typed `{error,message}` API error.
- The OpenAPI operation declarations do not yet include the complete shared
  `401`, `413`, `429`, and `500` response set or the
  `X-Read-Consistency` request header.
- Rust does not preserve the full OpenAPI union and enum type surface.
- No generated client has typed Search v2 facet or metric methods, definitions, or
  results. The target `kind`-discriminated search unions are not in the current
  OpenAPI snapshot.
- No generated client applies Lumen's planned operation-aware backoff,
  `Retry-After`, deadline, cancellation, or idempotency-safe write retry.
- No generated client accepts a caller-owned bulk source-fetch callback or
  restores Lumen hit order after hydration.
- Lumen does not ship a Managed client-workload Kustomize template. The current
  `k8s/overlays/template` is a Standalone runtime template.
- The generated-client CRUD test can skip Python, TypeScript, or Rust when its
  local interpreter, dependency, compiler, Node runtime, or Cargo tool is
  missing. A green run is not proof that all three languages executed.
- There is no general HTTP compatibility and deprecation policy. Do not infer
  one from the committed snapshot.

These are future outcomes. They are not hidden installation steps. See
[Generated-client protocol parity](../ROADMAP.md#generated-client-protocol-parity),
[Strict generated-client gates](../ROADMAP.md#strict-generated-client-gates),
[Generated-client search v2 parity](../ROADMAP.md#generated-client-search-v2-parity),
[Generated-client request resilience](../ROADMAP.md#generated-client-request-resilience),
[Generated-client source-integration helpers](../ROADMAP.md#generated-client-source-integration-helpers),
[Versioned client workload template](../ROADMAP.md#versioned-client-workload-template),
and [Protocol compatibility policy](../ROADMAP.md#protocol-compatibility-policy).

## Verification

Regenerate the committed OpenAPI reference with:

```bash
cargo run -q -p lumen --bin lumen -- spec > apps/lumen/clients/openapi.json
```

The local snapshot assertion is:

```bash
cargo test -p lumen --test spec_cli openapi_committed_snapshot_matches_live_generation
```

The generator and current cross-language happy-path gates are:

```bash
cargo test -p lumen --test spec_gen_e2e
cargo test -p lumen --test generated_clients_crud_e2e -- --nocapture
cargo test -p openapi-codegen
```

No current GitHub workflow runs a `clients-drift` job. The snapshot assertion is
real, but CI enforcement is a future outcome.

## Supporting documents

- [Lumen README](../README.md)
- [Protocol guide](../docs/protocol.md)
- [Indexing](../docs/indexing.md)
- [Querying](../docs/querying.md)
- [Search v2 migration](../docs/migration-search-v2.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
- [Authentication](../docs/authentication.md)
- [Client integration](../docs/client-integration.md)
- [GKE support profile](../docs/gke.md)
- [`openapi-codegen`](../../../libs/openapi-codegen/README.md)

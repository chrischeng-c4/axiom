# Lumen protocol

## Purpose

This guide maps each public protocol fact to the source that owns it. It is a
navigation document. It does not copy the complete route list, request schema,
query grammar, or retry table.

Use the machine-readable contract for exact current HTTP shapes. Use the
indexing and querying guides for behavior that spans requests. Those guides
keep current behavior separate from the 0.5 target.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| HTTP methods, paths, operation IDs, declared request and response schemas, status codes, media types, and security | [`lumen spec` OpenAPI](../clients/openapi.json) | Run `lumen spec` or request `GET /openapi.json`. |
| QUERY and POST twins, read-consistency header, and routed response behavior | This protocol guide and current OpenAPI | Run `lumen spec`, then use `lumen llm --topic local-search` or `select-query`. |
| Current and target schema, write, durability, rebuild, and activation meaning | [Indexing guide](indexing.md) | Read the current and 0.5 subsections separately. |
| Source ownership, current query navigation, and target query, result, facet, metric, and limit meaning | [Querying guide](querying.md) | Run `lumen llm --topic querying --format json`. |
| Source-database adapter boundary | [Architecture](../ARCHITECTURE.md#source-data-flow) | Run `lumen llm --topic integrate-source-db`. |
| 0.4.x to 0.5.0 wire and activation changes | [0.5 search migration](migration-0.5-search.md) | Follow the compatibility table before changing a caller. |
| Generated source, target languages, connection inputs, and language-specific limits | [Generated-client guide](../clients/README.md) | Run `lumen spec gen --lang <language> --out <dir>` for `ts`, `py`, or `rust`. |
| Connection profiles, request resilience, workload projection, and source helpers | [Client integration guide](client-integration.md) | Separate current manual work from the planned generated-client behavior. |
| GKE endpoint, topology, and support tier | [GKE guide](gke.md) | Check current zonal evidence separately from the regional target. |
| Current implementation support and material limits | [STATUS.md](../STATUS.md) | Read the support matrix before selecting a client path. |
| Future outcomes and explicit non-goals | [ROADMAP.md](../ROADMAP.md) | Follow the stable outcome linked from a STATUS limit. |
| Managed request identity and credential flow | [Authentication guide](authentication.md) | Separate current KSA behavior from the planned generated-client provider. |

The OpenAPI document is a committed consumer reference generated from the live
router. The focused `local-search`, `select-query`, and `integrate-source-db`
topics are generated from Lumen source. The new `querying` topic navigates the
current and target query contracts. This file keeps their responsibility and
discovery paths visible in one place.

## Use the protocol

Choose the connection profile first:

| Mode | Endpoint | Transport and identity |
|---|---|---|
| Standalone | `http://127.0.0.1:7373` by default | HTTP/1.1 or h2c. Authentication is off by default. |
| Managed | `https://<instance>.<namespace>.svc:7373` | Private serving CA, HTTP/2 or HTTP/1.1 through TLS ALPN, and a bearer ServiceAccount token. |

The OpenAPI tags divide the public operations into four families:

| Family | Use it for |
|---|---|
| Collections | Declare, list, inspect, and remove collection schemas. |
| Index | Merge indexed fields, replace complete indexed rows, delete values, unindex a bounded caller-selected ID list, truncate one collection's indexed documents, or run the current stream-reindex endpoint. |
| Query | Search one or many collections, inspect statistics, and find duplicates. |
| Admin and operations | Probe the process, inspect version and metrics, and run backup, restore, checkpoint, debug, or reshard operations. |

Prefer the RFC 10008 `QUERY` methods when the complete HTTP path supports them.
The POST twins remain the permanent compatibility path. Inspect `lumen spec`
for the method and path pairs. Use `lumen llm --topic select-query` for current
query-shape selection.

The current runtime accepts `X-Read-Consistency` with `leader`,
`bounded(<ms>)`, or `any` in replicated mode. Routed requests can return `503`
when the caller must retry after ownership or leader change. Use
`lumen spec --shapes` for copy-ready current query bodies. Use the OpenAPI
document for declared status and media-type details.

Use `lumen llm --topic querying` to choose between current request shapes and
the documented 0.5 target. Use `lumen llm --topic integrate-source-db` for the
caller-owned adapter boundary. Neither topic implements a migration.

`POST /collections/{collection_id}/reindex/stream` is a special NDJSON path.
Use a manual streaming HTTP client today. The generated clients do not expose
its request and response stream contract.

## Current boundaries

The [support matrix](../STATUS.md#support-matrix) is the current contract. The
important protocol and generated-client boundaries are:

- `X-Read-Consistency` is implemented and documented here, but it is not
  declared as an OpenAPI request header.
- Shared authentication, body-limit, admission, and internal failure paths can
  return `401`, `413`, `429`, or `500`. The OpenAPI operation responses do not
  yet describe this complete shared set and its structured error envelope.
- The stream-reindex OpenAPI request uses `text/plain`, but its successful
  streaming response has no declared content schema. The current code generator
  models JSON request and response bodies only.
- Generated clients raise or return transport-level HTTP failures. They do not
  decode Lumen's structured error envelope into one typed API error.
- Lumen has no general HTTP compatibility and deprecation policy. The explicit
  QUERY/POST twin promise remains the only stated long-lived compatibility
  rule.
- Generated-client token rotation is a separate authentication outcome. It is
  not part of protocol parity and is not implemented.
- The 0.5 scoring/filter split, strict result contract, facets, metrics, and
  capability activation are documented targets. They are not current OpenAPI
  operations or schemas.

See [Protocol contract completeness](../ROADMAP.md#protocol-contract-completeness),
[Generated-client protocol parity](../ROADMAP.md#generated-client-protocol-parity),
[Strict generated-client gates](../ROADMAP.md#strict-generated-client-gates),
and [Protocol compatibility policy](../ROADMAP.md#protocol-compatibility-policy)
for completion evidence. These links do not mean those outcomes are complete.

## Supporting documents

- [Lumen README](../README.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
- [Generated clients](../clients/README.md)
- [Indexing](indexing.md)
- [Querying](querying.md)
- [0.5 search migration](migration-0.5-search.md)
- [Authentication](authentication.md)
- [Deployment](deployment.md)
- [Client integration](client-integration.md)
- [GKE support profile](gke.md)

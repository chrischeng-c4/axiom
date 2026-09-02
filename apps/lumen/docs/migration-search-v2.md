# Lumen Search v2 migration

## Purpose

This guide defines the caller-visible move from the current 0.4.x search
contract to Search v2. Search v2 activation remains planned for
`lumen@0.37.0`. The guide is a target contract, not current support.

## Compatibility window

| Surface | Current contract | Search v2 | Required action |
|---|---|---|---|
| Unknown search fields | Strict rejection is a required 0.4.x prerequisite. | Strict parsing remains required. | Upgrade every serving member before sending a Search v2 field. |
| Mixed-version requests | New fields are unsafe while a serving member lacks the prerequisite. | Mixed versions reject activation. | Do not send Search v2 fields until convergence completes. |
| Numeric schema | `number` is the legacy numeric type. | `float64` replaces `number`. | Convert the schema through a shadow generation. |
| Multi-value schema | `set` is the legacy exact-string collection. | `keyword` with `multi=true` replaces `set`. | Convert and rebuild the collection. |
| Total request | Boolean `track_total` remains accepted during the window. | Tagged `none`, `up_to`, and `exact` are required. | Move each request to the tagged form. |
| Total response | `total` is a number. | `total` is null or a decimal-string value with a relation. | Update response decoding without fixed-width coercion. |
| Cursor mismatch | A mismatch can restart at the first page. | Request mismatch is `400`; stale generation is `409`. | Treat each cursor failure as a distinct caller decision. |
| Collapse | Current collapse uses the current response shape. | Collapse preserves source IDs and has `collapse_key` and group totals. | Update hit and total decoding before activation. |
| Duplicate discovery | `/duplicates` remains available during the window. | `/duplicates` is removed. | Use a terms facet, then a term filter for each selected value. |
| Managed capability | No capability activation contract exists. | Activation requires member convergence and final compatibility version. | Wait for the operator to report Search v2 active. |

## Schema migration

Convert legacy `number` fields to `float64`. Convert legacy `set` fields to
`keyword` with `multi=true`. Legacy collections need a shadow rebuild. An
in-place label change is not sufficient.

Search v2 also introduces strict `int64`, `decimal(p,s)`, `timestamp`, `date`,
and `boolean` types plus `facetable`. The future tool refuses a conversion that
cannot select a safe target type.

## Request migration

Move scoring nodes into `query: ScoringQuery`. Move business predicates into
`filter: FilterExpr`. Search v2 matches all when both are absent and intersects
them when both are present.

Replace Boolean `track_total` with `none`, `up_to`, or `exact`. State missing
sort behavior as `first` or `last`. A cursor is bound to the semantic request.
The offline request tool refuses ambiguous mixed `OR` and `NOT` trees.

## Response migration

Search v2 returns `total` as null or an object with decimal-string `value` and
`eq` or `gte` relation. It preserves each source `external_id` during collapse
and adds `collapse_key` and group totals. `/duplicates` has no Search v2
response.

## Managed activation

The target `/version` response separates binary capabilities, active
capabilities, and effective compatibility version. Managed activation waits for
every serving member to advertise the required capability. The first facet
capability is `search_facets_v1`.

Mixed versions reject activation. Lumen does not route data or queries around
an incompatible serving member. Search v2 activation remains planned for
`lumen@0.37.0`.

## Migration tools

The planned commands are:

```text
lumen migrate search-request
lumen migrate collection-schema
```

Each command reads current JSON from standard input and writes Search v2 JSON
plus a report. It does not contact a Lumen runtime, write a collection, or
activate a capability.

## Verification

Before activation, verify strict parsing, member capability convergence, shadow
rebuild completion, tagged totals, explicit missing-value handling, new total
and cursor decoding, generated-client parity, and operator activation at the
finalized compatibility version.

## Supporting documents

- [Lumen README](../README.md)
- [Indexing](indexing.md)
- [Querying](querying.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
- [Protocol](protocol.md)
- [Generated clients](../clients/README.md)
- [Deployment](deployment.md)

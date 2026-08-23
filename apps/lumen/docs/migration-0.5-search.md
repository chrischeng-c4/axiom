# Lumen 0.5 search migration

## Purpose

This guide defines the caller-visible migration from the 0.4.x search contract
to 0.5.0. It owns the compatibility window, required caller changes, Managed
activation rule, and planned offline migration tools.

Version 0.5 search is not implemented. The commands and activation behavior in
this guide are target contracts. Use [STATUS.md](../STATUS.md) for current
support.

## Compatibility window

| Surface | 0.4.x | 0.5.0 | Required action |
|---|---|---|---|
| Unknown search fields | `SearchRequest` and `BatchSearchItem` first gain strict unknown-field rejection as a 0.4.x prerequisite. | Strict request parsing remains required. | Upgrade every serving member to the strict 0.4.x prerequisite before sending any new field. |
| Mixed-version requests | New 0.5 fields are unsafe until every serving member has the prerequisite. | Mixed-version serving rejects 0.5 activation. | Do not send new fields or enable the capability while one serving member lacks support. |
| Numeric schema | `number` is the legacy numeric type. | `float64` replaces `number`. | Convert the schema and rebuild the collection through a shadow generation. |
| Multi-value schema | `set` is the legacy exact string collection. | `keyword` with `multi=true` replaces `set`. | Convert the schema and rebuild the collection through a shadow generation. |
| Total request | Boolean `track_total` remains accepted. Tagged `none`, `up_to`, and `exact` are added. Legacy bool emits a deprecation warning. | Only tagged `none`, `up_to`, and `exact` are accepted. | Move every request to the tagged form before 0.5.0. |
| Total response | `total` remains a number. | `total` is an object with decimal-string `value` and `eq` or `gte` relation, or it is null. | Update response decoding and do not coerce the value to a fixed-width integer. |
| Sort missing default | Omitted `missing` keeps the legacy `exclude` behavior and emits a warning. | Omitted `missing` means `last`. Only `first` and `last` are accepted. | State `first` or `last`. Add an `exists` filter when missing rows must be excluded. |
| Cursor mismatch | Current mismatch behavior can return or restart at the first page. | Shape or query mismatch returns `400 invalid_cursor`. Changed collection UID or generation returns `409 stale_cursor`. | Treat the cursor as bound to the request and restart only through an explicit caller decision. |
| Collapse | Current collapse keeps the 0.4.x response behavior. | Collapse preserves source IDs and adds canonical `collapse_key` plus group totals. | Update hit and total decoding before selecting the 0.5 compatibility version. |
| Duplicate discovery | `/duplicates` remains available and emits a deprecation warning. | `/duplicates` is removed. | Use a terms facet with `min_count=2`, then page IDs with a term filter for each selected value. |
| Managed capability | No `search_facets_v1` activation contract exists. | Activation needs all serving members and finalized `compatibilityVersion`. | Wait for the operator to report the active capability before sending 0.5 facet requests. |

The strict 0.4.x unknown-field prerequisite is mandatory. A mixed-version
runtime must not receive new fields before every serving member has it. Lumen
does not use version-aware data routing to make an incompatible member safe.

## Schema migration

Convert legacy `number` fields to `float64`. Convert legacy `set` fields to
`keyword` fields with `multi=true`.

Legacy collections need a shadow rebuild. An in-place schema label change is
not sufficient because the 0.5 index representation and validation contract
are different.

The 0.5 schema also introduces strict `int64`, `decimal(p,s)`, `timestamp`,
`date`, and `boolean` types plus orthogonal `facetable`. Read the
[indexing schema contract](indexing.md#schema-contract) before converting a
collection.

The future schema migration tool reports every unambiguous conversion. It
refuses a field when the old schema or sampled value cannot select a safe 0.5
type.

## Request migration

Move scoring nodes into `query: ScoringQuery`. Move business predicates into
`filter: FilterExpr`. When both are absent, 0.5 performs match-all. When both
are present, it intersects them without changing score from the filter.

Replace boolean `track_total` with one tagged choice:

- `none` disables totals.
- `up_to` counts through a supplied bound.
- `exact` requires an exact count or a request failure.

State sort missing behavior as `first` or `last`. Version 0.5 treats an omitted
value as `last`. To exclude records that have no sort value, add an `exists`
filter. Do not use the old `exclude` sort value.

A cursor is bound to the semantic request. A caller must treat
`400 invalid_cursor`, `409 stale_cursor`, and `410 cursor_expired` as distinct
conditions. Lumen does not silently fall back to the first page.

The offline request migration tool refuses an ambiguous mixed `OR` or `NOT`
tree. It reports the ambiguity instead of guessing whether a legacy node was a
scoring clause or a filter.

## Response migration

Version 0.4.x keeps a numeric `total`. Version 0.5 returns either `null` or an
object with:

- `value`, as a decimal string; and
- `relation`, as `eq` or `gte`.

Callers must not decode the new value into a fixed-width integer without an
explicit range check.

The 0.5 collapse response keeps each source `external_id`. It adds the
canonical `collapse_key`. `total` reports matching documents before collapse,
and `collapsed_total` reports groups. Update response models and hydration code
before activation.

`/duplicates` has no 0.5 response. Replace it with two explicit steps:

1. Run a terms facet with `min_count=2` to select duplicated values.
2. Run a term filter for a value and page the matching source IDs.

This replacement does not promise one unbounded grouped response.

## Managed activation

The target `/version` response separates:

- binary capabilities, which the member executable understands;
- active capabilities, which the runtime can currently serve; and
- effective compatibility version, which controls the accepted public
  contract.

The capability ID for the first facet contract is `search_facets_v1`.

A new Standalone binary can use its supported contract directly. A Managed
runtime must wait until every serving member reports the required binary
capability. The operator can enable it only after `compatibilityVersion`
finalization completes.

Mixed versions reject activation. Lumen does not route 0.5 data or queries
around an incompatible serving member.

This activation contract is future work. Current `/version` and the operator
do not publish or converge these three capability dimensions.

## Migration tools

Two offline commands are planned:

```bash
lumen migrate search-request
lumen migrate collection-schema
```

Each command reads 0.4 JSON from standard input. It writes 0.5 JSON and a
migration report to standard output. It does not connect to a Lumen runtime,
write a collection, or activate a capability.

The request tool refuses ambiguous mixed `OR` and `NOT`. The schema tool
refuses a conversion that cannot select a safe strict type. Neither tool
silently guesses.

These commands do not exist in the current CLI. See
[Search 0.5 migration](../ROADMAP.md#search-v0-5-migration).

## Verification

Before a Managed activation, verify all of these conditions:

1. Every serving member rejects unknown 0.4.x search fields.
2. Every serving member advertises the required binary capability.
3. Every legacy collection completed its shadow rebuild.
4. Every caller uses tagged totals and explicit missing-value behavior.
5. Every caller handles the new total, cursor, and collapse responses.
6. Generated clients in use support the complete 0.5 discriminated types.
7. The operator reports `search_facets_v1` active at the finalized
   compatibility version.

Future migration tests cover numeric and set schemas, tagged totals, unknown
fields, missing sort behavior, cursor errors, collapse, duplicate replacement,
offline-tool refusal, and mixed-version activation. The current repository has
not completed these gates.

## Supporting documents

- [Lumen README](../README.md)
- [Indexing](indexing.md)
- [Querying](querying.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
- [Protocol](protocol.md)
- [Generated clients](../clients/README.md)
- [Deployment](deployment.md)

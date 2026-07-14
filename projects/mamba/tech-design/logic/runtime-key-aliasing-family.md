# Runtime-key aliasing family — display name ≠ CLASS_REGISTRY key

Status: 4 sites landed (`35b6be9f8` total_ordering, `0e5b826a9` singledispatch,
`7ee9fb8e1` logging; #1600/#1595/wave-1). Backfill TD; family may have more sites.

## Mechanism

Since the upstream runtime-key namespacing work, a user class's registry key is
a namespaced string (`__mamba_user_class__:<file>:<line>:<Name>@<n>`), distinct
from its display `__name__`. An `Instance`'s `class_name` field holds the
REGISTRY key; a `type` object's `__name__` holds the DISPLAY name. Any lookup
that feeds a display name into `CLASS_REGISTRY` (or compares the two domains)
silently misses for user classes — symptoms range from "dispatch falls back to
base impl" (#1600 `A -> A`) to "cls is the literal string 'type'" (#1600 sdm)
to "setLoggerClass roundtrip fails" (wave 1).

## Invariant

- To get a registry key from a `type` object: `type_object_registry_key`.
- To get a registry key from an `Instance`: read its raw `class_name` field.
- Display names are for error messages ONLY — route through
  `class_display_name` (see #226 fix `c9cebdbb7`), never into registry lookups.

## Fix pattern

Replace `__name__`-based probes with the two accessors above. When a receiver
may be a `type` object (classmethod stacking), branch on it explicitly before
treating `class_name` as an instance class (see `sdm_receiver_class`).

## Verification contract

Per site: a register→dispatch (or set→get) round-trip probe with a USER class,
byte-identical vs python3.12. Family fixtures: functools singledispatch pair,
`logging/{manager_uses_custom_class,setloggerclass_roundtrip}.py`,
total_ordering fixture. Related-but-distinct: DictKey hash-domain family
(`1566-dictkey-hash-domain-audit.md`) — that one is about HASH domains on dict
probes; this one is about NAME domains on registry lookups.

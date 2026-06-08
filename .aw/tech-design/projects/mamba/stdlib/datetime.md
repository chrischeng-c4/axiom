---
id: stdlib-datetime
title: stdlib datetime — Date and Time
crate: mamba
files:
  - crates/mamba/src/runtime/stdlib/datetime_mod.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 8323393b7
---

# stdlib `datetime`

Date / datetime / timedelta values + format / parse / arithmetic.
Mamba uses `chrono` Rust crate as the substrate; Python-visible
`datetime`, `date`, `time`, `timedelta` are Instance wrappers
(`class_name = "datetime.datetime"` etc.) carrying the `chrono`
value in `_inner` field.

Three load-bearing invariants:

1. **Naive vs aware datetime split** — naive datetime has no tzinfo;
   aware does. Mamba's current impl handles naive only; aware is
   open gap.
2. **`fromisoformat` accepts CPython 3.11+ extended format** — `T`
   separator, `Z` UTC suffix, `+HH:MM` offsets. `chrono::DateTime::parse_from_rfc3339`
   covers most cases.
3. **`timedelta` arithmetic preserves sign** — `dt + td`, `dt - dt`,
   `td * n` all per CPython rules. No silent overflow at i64 bounds.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: datetime-types
types:
  DatetimeMod:    { kind: struct }
  DatetimeInst:   { kind: struct, label: "Instance class_name=datetime.datetime" }
  DateInst:       { kind: struct, label: "Instance class_name=datetime.date" }
  TimedeltaInst:  { kind: struct, label: "Instance class_name=datetime.timedelta" }
  ChronoCrate:    { kind: struct, label: "chrono::DateTime / NaiveDateTime / Duration" }
edges:
  - { from: DatetimeMod, to: DatetimeInst,  kind: owns }
  - { from: DatetimeMod, to: DateInst,      kind: owns }
  - { from: DatetimeMod, to: TimedeltaInst, kind: owns }
  - { from: DatetimeInst, to: ChronoCrate,  kind: references }
---
classDiagram
    class DatetimeMod
    class DatetimeInst
    class DateInst
    class TimedeltaInst
    class ChronoCrate
    DatetimeMod --> DatetimeInst : owns
    DatetimeMod --> DateInst : owns
    DatetimeMod --> TimedeltaInst : owns
    DatetimeInst --> ChronoCrate : refs
```

## Function catalog
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "datetime-catalog"
$defs:
  StdlibFnEntry:
    type: object
    properties:
      python_name:    { type: string }
      mb_fn:          { type: string }
      arity:          { type: integer }
      cpython_parity: { type: string, enum: [full, partial, gap] }
      notes:          { type: string }
    required: [python_name, mb_fn, arity, cpython_parity]
  DatetimeCatalog:
    type: array
    items: { $ref: "#/$defs/StdlibFnEntry" }
    examples:
      - - { python_name: "datetime.datetime.now",            mb_fn: "mb_datetime_now",            arity: 0, cpython_parity: full,    notes: "naive local now" }
        - { python_name: "datetime.datetime",                mb_fn: "mb_datetime_new",            arity: -1, cpython_parity: partial, notes: "(year, month, day, hour, minute, second, microsecond); tzinfo gap" }
        - { python_name: "datetime.datetime.fromtimestamp",  mb_fn: "mb_datetime_fromtimestamp",  arity: 1, cpython_parity: partial, notes: "no tz arg" }
        - { python_name: "datetime.datetime.fromisoformat",  mb_fn: "mb_datetime_fromisoformat",  arity: 1, cpython_parity: partial, notes: "RFC 3339 subset; T sep + Z suffix + offsets" }
        - { python_name: "datetime.timestamp",                mb_fn: "mb_datetime_timestamp",      arity: 1, cpython_parity: full }
        - { python_name: "datetime.isoformat",                mb_fn: "mb_datetime_isoformat",      arity: 1, cpython_parity: full }
        - { python_name: "datetime.strftime",                 mb_fn: "mb_datetime_strftime",       arity: 2, cpython_parity: partial, notes: "chrono format-spec subset" }
        - { python_name: "datetime + timedelta",              mb_fn: "mb_datetime_add_timedelta",  arity: 2, cpython_parity: full }
        - { python_name: "datetime.timezone / aware datetime", mb_fn: "(gap)", arity: -1, cpython_parity: gap, notes: "tzinfo subclass support not wired" }
```

## Acceptance scenarios
<!-- type: overview lang: markdown -->

```mermaid
---
id: datetime-acceptance
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Fixture, kind: system }
messages:
  - { from: User,    to: Mamba,   name: "run stdlib/datetime_basic.py" }
  - { from: Mamba,   to: Fixture, name: "datetime(2026, 4, 27); .isoformat(); .timestamp()" }
  - { from: Fixture, to: Mamba,   name: "'2026-04-27T00:00:00'; epoch seconds" }
  - { from: User,    to: Mamba,   name: "run stdlib/datetime_arith.py" }
  - { from: Mamba,   to: Fixture, name: "dt + timedelta(days=1)" }
  - { from: Fixture, to: Mamba,   name: "next-day datetime" }
  - { from: User,    to: Mamba,   name: "run stdlib/datetime_iso.py" }
  - { from: Mamba,   to: Fixture, name: "datetime.fromisoformat('2026-04-27T08:00:00Z')" }
  - { from: Fixture, to: Mamba,   name: "datetime parsed via chrono RFC 3339" }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Fixture
    User->>Mamba: basic
    Mamba->>Fixture: ctor + isoformat + timestamp
    Fixture-->>Mamba: round-trip
    User->>Mamba: arith
    Mamba->>Fixture: dt + td
    Fixture-->>Mamba: next-day
    User->>Mamba: fromisoformat
    Mamba->>Fixture: RFC 3339
    Fixture-->>Mamba: parsed
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: datetime_basic
    name: "stdlib/datetime_basic.py"
    paired: "stdlib/datetime_basic.expected"
  - id: datetime_arith
    name: "stdlib/datetime_arith.py"
    paired: "stdlib/datetime_arith.expected"
  - id: datetime_iso_round_trip
    name: "stdlib/datetime_iso_round_trip.py"
    paired: "stdlib/datetime_iso_round_trip.expected"
  - id: datetime_strftime
    name: "stdlib/datetime_strftime.py"
    paired: "stdlib/datetime_strftime.expected"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/stdlib/datetime_mod.rs
    action: modify
    impl_mode: hand-written
    description: "datetime / date / timedelta wrappers around chrono. Hand-written; aware datetime + timezone subclass are gaps."
```

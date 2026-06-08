---
id: stdlib-coverage-top5
main_spec_ref: "crates/mamba/testing/stdlib-coverage-lower.md"
merge_strategy: extend
fill_sections: [overview, requirements, scenarios, test-plan, changes]
filled_sections: [overview, requirements, scenarios, test-plan, changes]
create_complete: true
---

# Mamba Test Coverage Spec

## Overview

Bring the 5 lowest-coverage files in `crates/mamba/` from 0–8% to ≥50% line coverage by adding inline `#[cfg(test)] mod tests {}` blocks.

**Target files:**

| File | Path | Before | Target |
|------|------|--------|--------|
| `c_types.rs` | `src/ffi/c_types.rs` | 0% | ≥50% |
| `queue_mod.rs` | `src/runtime/stdlib/queue_mod.rs` | 4% | ≥50% |
| `statistics_mod.rs` | `src/runtime/stdlib/statistics_mod.rs` | 5% | ≥50% |
| `shlex_mod.rs` | `src/runtime/stdlib/shlex_mod.rs` | 7% | ≥50% |
| `calendar_mod.rs` | `src/runtime/stdlib/calendar_mod.rs` | 8% | ≥50% |

**Scope boundary:**

- 4 stdlib modules + 1 core FFI types module
- Inline `#[cfg(test)]` only — no integration test file
- Self-contained: no network, no FS writes, no external deps
- No coverage-exclusion annotations
- Measurement: `cargo llvm-cov --branch -p mamba`
## Requirements

### R1: ffi/c_types.rs (0% → ≥50%)

Pure data-model module with 1 method and 7 structs/enums. All derive `Debug, Clone, PartialEq`.

| Item | Branches to cover |
|------|-------------------|
| `CType::display_name()` | All 17 enum variants: Void, Int8–Int64, UInt8–UInt64, Float, Double, Bool, ConstChar, MutChar, Pointer(inner), ConstPointer(inner), Named(s) |
| `CType` | Debug format, Clone equality, PartialEq positive + negative |
| `CFunction` | Construct with params + return_type, Clone, PartialEq |
| `CParam` | Construct, Clone, PartialEq |
| `CStruct` | Construct with fields, Clone, PartialEq |
| `CField` | Construct, Clone |
| `CEnum` | Construct with variants, Clone, PartialEq |
| `CEnumVariant` | `value: Some(n)` vs `value: None` |
| `CHeader` | `Default` (empty vecs), push to each vec |

### R2: queue_mod.rs (4% → ≥50%)

| Function | Branches to cover |
|----------|-------------------|
| `mb_queue_Queue(maxsize)` | dict creation; `__type__` = "Queue" |
| `mb_queue_LifoQueue(maxsize)` | dict creation; `__type__` = "LifoQueue" |
| `mb_queue_PriorityQueue(maxsize)` | dict creation; `__type__` = "PriorityQueue" |
| `queue_items_ptr(q)` | valid dict → Some; `MbValue::none()` → None |
| `mb_queue_put(q, item)` | valid queue (item pushed); invalid queue (no-op) |
| `mb_queue_get(q)` | non-empty list (remove+return first); empty list → none; invalid queue → none |
| `mb_queue_empty(q)` | empty list → true; non-empty → false; invalid → true |
| `mb_queue_qsize(q)` | N items → N; empty → 0; invalid �� 0 |
| `mb_queue_full(_q)` | always returns false |

Concurrency: two `std::thread` threads interleave `put`/`get` on shared queue without panic.

### R3: statistics_mod.rs (5% → ≥50%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_floats(list)` | int items → f64 conversion; float items → passthrough; empty list; non-list MbValue |
| `mb_statistics_mean(data)` | non-empty → sum/count; empty → none |
| `mb_statistics_median(data)` | odd-length → middle; even-length → average of two middle; empty → none |
| `mb_statistics_mode(data)` | single mode → most frequent; empty → none |
| `mb_statistics_variance(data)` | ≥2 items → sample variance; <2 → none |
| `mb_statistics_stdev(data)` | ≥2 items → sqrt(variance); <2 → none |
| `mb_statistics_geometric_mean(data)` | non-empty → exp(mean(ln)); empty → none |
| `mb_statistics_harmonic_mean(data)` | non-empty → n/sum(1/x); empty → none |

### R4: shlex_mod.rs (7% → ≥50%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str(val)` | Str variant → Some; non-Str → None |
| `extract_list(val)` | List variant → Some; non-List → None |
| `mb_shlex_split(s)` | plain space-separated; quoted string (space inside quotes preserved); empty input |
| `mb_shlex_quote(s)` | safe alphanumeric → unchanged; unsafe (spaces) → wrapped in quotes; empty → `""` |
| `mb_shlex_join(tokens)` | non-empty list → joined; empty list → empty string |

### R5: calendar_mod.rs (8% → ≥50%)

| Function | Branches to cover |
|----------|-------------------|
| `mb_calendar_isleap(year)` | ÷400 → true; ÷100 not ÷400 → false; ÷4 not ÷100 → true; not ÷4 → false |
| `mb_calendar_leapdays(y1, y2)` | positive range; zero range (y1==y2) |
| `mb_calendar_monthrange(year, month)` | 31-day months; 30-day months; Feb leap (29); Feb non-leap (28); invalid month → 30 |
| `mb_calendar_month_name()` | 13-element list; index 0 is empty string |
| `mb_calendar_day_name()` | 7-element list |
| `mb_calendar_weekday(year, month, day)` | known date verified; `m < 3` triggers Zeller year/month adjustment |
## Scenarios

### S-ffi-1: CType display_name — all 17 variants

- **Function**: `CType::display_name`
- **Inputs**: each enum variant (Void → "void", Int8 → "int8_t", …, Pointer(Int32) → "int32_t*", ConstPointer(UInt8) → "const uint8_t*", Named("Foo") → "Foo")
- **Expected**: exact string match per variant
- **Branch covered**: all 17 match arms

### S-ffi-2: CType Debug / Clone / PartialEq derived traits

- **Items**: `CType`, `CFunction`, `CParam`, `CStruct`, `CField`, `CEnum`, `CEnumVariant`
- **Expected**: Debug formats non-empty; Clone produces equal value; PartialEq distinguishes different variants

### S-ffi-3: CHeader Default and push

- **Function**: `CHeader::default()`
- **Setup**: default header → push function, struct, enum
- **Expected**: initially empty vecs; each push increments len by 1

### S-ffi-4: CEnumVariant value Some vs None

- **Input A**: `CEnumVariant { name: "A", value: Some(42) }` → value == Some(42)
- **Input B**: `CEnumVariant { name: "B", value: None }` → value == None

### S-queue-1: Queue FIFO ordering

- **Functions**: `mb_queue_Queue`, `mb_queue_put`, `mb_queue_get`
- **Setup**: create Queue maxsize=0; put items 1, 2, 3
- **Expected**: successive get returns 1, 2, 3; 4th get returns none

### S-queue-2: Queue empty / qsize transitions

- **Functions**: `mb_queue_empty`, `mb_queue_qsize`
- **Setup A**: fresh queue → empty=true, qsize=0
- **Setup B**: after one put → empty=false, qsize=1
- **Setup C**: after get → empty=true, qsize=0

### S-queue-3: Queue with invalid MbValue

- **Input**: `MbValue::none()` as queue argument
- **Expected**: no panic; put→none, get→none, empty→true, qsize→0

### S-queue-4: LifoQueue and PriorityQueue construction

- **Input**: maxsize=5 for each
- **Expected**: dict with correct `__type__` string

### S-queue-5: Queue full always false

- **Input**: any MbValue
- **Expected**: `MbValue::from_bool(false)`

### S-queue-6: Concurrent put/get via std::thread

- **Setup**: Queue; producer thread puts 50 items; main thread gets 50
- **Expected**: no panic, no deadlock; 50 items transferred

### S-stats-1: mean — basic and empty

- **Input A**: int list `[1,2,3,4,5]` → 3.0
- **Input B**: float list `[1.5,2.5]` → 2.0
- **Input C**: empty list → none

### S-stats-2: median — odd / even / empty

- **Input A**: `[1,3,2]` → 2.0 (odd)
- **Input B**: `[1,2,3,4]` → 2.5 (even)
- **Input C**: `[]` → none

### S-stats-3: variance and stdev — guard on < 2 items

- **Input A**: `[2.0,4.0]` → variance=2.0, stdev≈1.414
- **Input B**: `[1.0]` → none
- **Input C**: `[]` → none

### S-stats-4: mode, geometric_mean, harmonic_mean

- **Mode**: `[1,2,2,3]` → 2.0; `[]` → none
- **Geometric**: `[1.0,4.0]` → ≈2.0; `[]` → none
- **Harmonic**: `[1.0,2.0,4.0]` → ≈1.714

### S-shlex-1: split plain / quoted / empty

- **Input A**: `"hello world"` → `["hello", "world"]`
- **Input B**: `"\"hello world\" foo"` → `["hello world", "foo"]`
- **Input C**: `""` → `[]`

### S-shlex-2: quote safe / unsafe / empty

- **Input A**: `"hello_world"` → unchanged
- **Input B**: `"hello world"` → `"\"hello world\""`
- **Input C**: `""` → `"\"\""`

### S-shlex-3: join

- **Input A**: `["a","b","c"]` → `"a b c"`
- **Input B**: `[]` → `""`

### S-cal-1: isleap — all four branches

- 2000 → true (÷400); 1900 → false (÷100 not ÷400); 2024 → true (÷4 not ÷100); 2023 → false

### S-cal-2: monthrange — all match arms

- (2024,1) → 31; (2024,4) → 30; (2024,2) → 29 (leap Feb); (2023,2) → 28; (2024,13) → 30 (fallback)

### S-cal-3: weekday — known date + Zeller adjustment

- (2024,1,1) → 0 (Monday); exercises `m < 3` branch

### S-cal-4: month_name / day_name list lengths

- month_name: 13 elements, index 0 = ""
- day_name: 7 elements

### S-cal-5: leapdays range

- (1900,2000) → 25; (2000,2000) → 0
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

### Coverage Target

All 5 target files → ≥50% line coverage under `cargo llvm-cov -p mamba`.

### Execution Command

```bash
cargo llvm-cov --branch --package cclab-mamba -- --test-threads=4
```

### Test Matrix

| Module | File | Inline Tests | Goal |
|--------|------|-------------|------|
| `c_types.rs` | `src/ffi/c_types.rs` | 27 | ≥50% |
| `queue_mod.rs` | `src/runtime/stdlib/queue_mod.rs` | 7 | ≥50% |
| `statistics_mod.rs` | `src/runtime/stdlib/statistics_mod.rs` | 14 | ≥50% |
| `shlex_mod.rs` | `src/runtime/stdlib/shlex_mod.rs` | 7 | ≥50% |
| `calendar_mod.rs` | `src/runtime/stdlib/calendar_mod.rs` | 12 | ≥50% |

### Inline Tests per Module

**c_types.rs** (27 tests):
- `test_display_name_{void,int8,int16,int32,int64,uint8,uint16,uint32,uint64,float,double,bool,const_char,mut_char,pointer,const_pointer,named}` — all 17 `display_name()` arms
- `test_debug_pointer_variant` — Debug derive
- `test_ctype_eq_int32_int32`, `test_ctype_neq_int32_int64` — PartialEq
- `test_cenum_clone` — Clone + PartialEq on CEnum
- `test_cfunction_construct_clone_eq` — CFunction Clone/PartialEq
- `test_cparam_construct_clone_eq` — CParam Clone/PartialEq
- `test_cstruct_construct_clone_eq` — CStruct Clone/PartialEq
- `test_cfield_construct_clone` — CField Clone
- `test_cenumvariant_value_some`, `test_cenumvariant_value_none` — Option<i64> branches
- `test_cheader_default_and_push` — Default trait + push to each vec

**queue_mod.rs** (7 tests):
- `test_queue_construction` — Queue, LifoQueue, PriorityQueue dicts with `__type__`
- `test_queue_put_get_fifo` — put 3, get 3 in FIFO order, 4th get → none
- `test_queue_empty_and_qsize` — empty↔non-empty transitions
- `test_queue_invalid_value` — `MbValue::none()` as queue arg → no panic
- `test_queue_full_always_false` — unconditional false
- `test_queue_concurrent_put_get` — `std::thread::spawn` producer/consumer 50 items

**statistics_mod.rs** (14 tests):
- `test_mean_basic` — int list + float list paths
- `test_mean_empty` — empty → none
- `test_median_odd` / `test_median_even` / `test_median_empty`
- `test_mode_basic` / `test_mode_empty`
- `test_variance_basic` / `test_variance_too_few`
- `test_stdev_basic` / `test_stdev_too_few`
- `test_geometric_mean_basic` / `test_geometric_mean_empty`
- `test_harmonic_mean_basic`

**shlex_mod.rs** (7 tests):
- `test_split_plain` / `test_split_quoted` / `test_split_empty`
- `test_quote_safe` / `test_quote_unsafe` / `test_quote_empty`
- `test_join_basic` (includes empty list sub-case)

**calendar_mod.rs** (12 tests):
- `test_isleap_400` / `test_isleap_100` / `test_isleap_4` / `test_isleap_none` — all 4 leap branches
- `test_leapdays_range` — (1900→2000) + zero range
- `test_monthrange_31` / `test_monthrange_30` / `test_monthrange_feb_leap` / `test_monthrange_feb_normal` / `test_monthrange_invalid_month`
- `test_month_name_count` / `test_day_name_count`
- `test_weekday_known_date` — Jan 1 2024 = Monday (m<3 branch)

### Acceptance Criteria

```yaml
criteria:
  - cargo test -p mamba passes with 0 failures
  - cargo llvm-cov -p mamba reports ≥50% line for all 5 target files
  - No coverage-exclusion annotations (#[allow(dead_code)], tarpaulin attrs)
  - All tests self-contained: no network, no persistent FS writes
  - queue_mod concurrent test completes without panic or deadlock
```
## Changes

```yaml
files:
  - path: crates/mamba/src/ffi/c_types.rs
    action: modify
    description: |
      Add/expand #[cfg(test)] mod tests with 27 inline tests covering
      all 17 CType::display_name arms, Debug/Clone/PartialEq for all
      7 structs/enums, CEnumVariant Some/None, CHeader Default+push.

  - path: crates/mamba/src/runtime/stdlib/queue_mod.rs
    action: modify
    description: |
      Add #[cfg(test)] mod tests with 7 inline tests: construction
      (Queue/LifoQueue/PriorityQueue), FIFO put/get, empty/qsize
      transitions, invalid MbValue safety, full always false,
      concurrent std::thread put/get (50 items).

  - path: crates/mamba/src/runtime/stdlib/statistics_mod.rs
    action: modify
    description: |
      Add #[cfg(test)] mod tests with 14 inline tests covering
      mean/median/mode/variance/stdev/geometric_mean/harmonic_mean
      for both non-empty and empty list branches, plus int→float
      conversion in extract_floats.

  - path: crates/mamba/src/runtime/stdlib/shlex_mod.rs
    action: modify
    description: |
      Add #[cfg(test)] mod tests with 7 inline tests covering
      split (plain/quoted/empty), quote (safe/unsafe/empty),
      and join (non-empty/empty list).

  - path: crates/mamba/src/runtime/stdlib/calendar_mod.rs
    action: modify
    description: |
      Add #[cfg(test)] mod tests with 12 inline tests covering
      isleap (4 branches), leapdays range, monthrange (5 match arms),
      month_name/day_name list lengths, weekday known date with
      Zeller m<3 adjustment branch.
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews

---
id: mamba-test-coverage-spec
main_spec_ref: "crates/mamba/testing/stdlib-coverage-lower.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test-plan, changes, logic, dependency]
fill_sections: [overview, requirements, scenarios, test-plan, changes, logic, dependency]
create_complete: true
---

# Mamba Test Coverage Spec

## Overview

Add inline `#[cfg(test)]` test modules and integration tests to the 10 lowest-coverage stdlib modules in `crates/mamba/src/runtime/stdlib/`, raising each from 4%–14% to 100% line + branch coverage measured by `cargo llvm-cov --branch`.

**Target modules by spec group:**

| File | Coverage | Spec Group |
|------|----------|------------|
| `queue_mod.rs` | 4% | concurrency |
| `statistics_mod.rs` | 5% | numeric |
| `shlex_mod.rs` | 7% | text-processing |
| `calendar_mod.rs` | 8% | datetime |
| `locale_mod.rs` | 10% | text-processing |
| `lzma_mod.rs` | 11% | archive-and-compression |
| `zlib_mod.rs` | 11% | archive-and-compression |
| `secrets_mod.rs` | 12% | random |
| `bisect_mod.rs` | 14% | numeric |
| `abc_mod.rs` | 14% | typing-and-inspect |

**Test strategy:**

- Replace stub `fn test_stub() { assert!(true); }` with full inline `#[cfg(test)] mod tests {}` blocks in each `*_mod.rs`
- Add `crates/mamba/tests/stdlib_coverage_lower_tests.rs` for integration tests
- Self-contained: no network, no filesystem writes outside temp dirs
- `lzma_mod` / `zlib_mod`: fixed in-memory byte arrays; no external files
- `queue_mod`: concurrent put/get exercised via `std::thread`
- No coverage exclusion annotations (`#[cfg(not(tarpaulin))]`, etc.); remove dead branches instead

**Coverage target:** 100% line + 100% branch (`cargo llvm-cov --branch`), all 10 files in one atomic PR.
## Requirements

### R1: concurrency — queue_mod.rs (4%)

| Function | Branches to cover |
|----------|-------------------|
| `mb_queue_Queue(maxsize)` | dict creation path |
| `mb_queue_LifoQueue(maxsize)` | dict creation path |
| `mb_queue_PriorityQueue(maxsize)` | dict creation path |
| `queue_items_ptr(q)` | valid dict → Some; null ptr → None; non-dict ObjData → None |
| `mb_queue_put(q, item)` | valid queue (item pushed); invalid queue (no-op) |
| `mb_queue_get(q)` | non-empty list (remove+return first); empty list (→ none); invalid queue |
| `mb_queue_empty(q)` | empty list → true; non-empty → false; invalid → true |
| `mb_queue_qsize(q)` | N items → N; empty → 0; invalid → 0 |
| `mb_queue_full(_q)` | always returns false |

Concurrency: two `std::thread` threads interleave `put` / `get` on a shared queue without panic.

### R2: numeric — statistics_mod.rs (5%) + bisect_mod.rs (14%)

**statistics_mod.rs:**

| Function | Branches to cover |
|----------|-------------------|
| `extract_floats(list)` | non-empty list (int items); non-empty list (float items); empty list; non-list MbValue |
| `mb_statistics_mean(data)` | non-empty list; empty → none |
| `mb_statistics_median(data)` | odd-length list; even-length list; empty → none |
| `mb_statistics_mode(data)` | single mode; empty → none |
| `mb_statistics_variance(data)` | ≥ 2 items; < 2 items → none |
| `mb_statistics_stdev(data)` | ≥ 2 items; < 2 items → none |
| `mb_statistics_geometric_mean(data)` | non-empty; empty → none |
| `mb_statistics_harmonic_mean(data)` | non-empty; empty → none |

**bisect_mod.rs:**

| Function | Branches to cover |
|----------|-------------------|
| `item_key(v)` | int value; float value; other → 0 |
| `read_list(val)` | valid List; non-List MbValue |
| `mb_bisect_bisect_left(a, x)` | x present (returns left of equal elements); x at boundary (before-all, after-all) |
| `mb_bisect_bisect_right(a, x)` | x present (returns right of equal elements); x at boundary |
| `mb_bisect_insort_left(a, x)` | valid list (inserts at bisect_left position); invalid MbValue (no-op) |
| `mb_bisect_insort_right(a, x)` | valid list (inserts at bisect_right position); invalid MbValue (no-op) |

### R3: text-processing — shlex_mod.rs (7%) + locale_mod.rs (10%)

**shlex_mod.rs:**

| Function | Branches to cover |
|----------|-------------------|
| `extract_str(val)` | Str variant → Some; non-Str → None |
| `extract_list(val)` | List variant → Some; non-List → None |
| `mb_shlex_split(s)` | plain space-separated tokens; quoted string (space inside quotes preserved); mixed; empty input |
| `mb_shlex_quote(s)` | safe alphanumeric string → returned unchanged; unsafe (spaces/special chars) → wrapped in quotes; empty string → `""` |
| `mb_shlex_join(tokens)` | non-empty list → joined with spaces; empty list → empty string |

**locale_mod.rs:**

| Function | Branches to cover |
|----------|-------------------|
| `extract_str(val)` | Str variant → Some; non-Str → None |
| `mb_locale_getlocale()` | returns ("en_US", "UTF-8") tuple |
| `mb_locale_setlocale(_cat, locale_str)` | Str input → echoes back; non-Str → returns default "en_US.UTF-8" |
| `mb_locale_format_string(fmt, val)` | int val → `%d` substituted; float val → `%f` substituted; other → format unchanged |
| `mb_locale_LC_ALL()` | returns 6 |
| `mb_locale_LC_CTYPE()` | returns 0 |
| `mb_locale_LC_TIME()` | returns 2 |
| `mb_locale_LC_NUMERIC()` | returns 1 |

### R4: datetime — calendar_mod.rs (8%)

| Function | Branches to cover |
|----------|-------------------|
| `mb_calendar_isleap(year)` | divisible by 400 → true; divisible by 100 but not 400 → false; divisible by 4 but not 100 → true; not divisible by 4 → false |
| `mb_calendar_leapdays(y1, y2)` | positive range (b > a); zero range (y1 == y2) |
| `mb_calendar_monthrange(year, month)` | 31-day months (1,3,5,7,8,10,12); 30-day months (4,6,9,11); February leap year (29 days); February non-leap (28 days); invalid month → fallback 30 |
| `mb_calendar_month_name()` | returns 13-element list; index 0 is empty string |
| `mb_calendar_day_name()` | returns 7-element list |
| `mb_calendar_weekday(year, month, day)` | known date → verified weekday; month < 3 triggers Zeller adjustment branch |

### R5: archive-and-compression — lzma_mod.rs (11%) + zlib_mod.rs (11%)

**lzma_mod.rs:**

| Function | Branches to cover |
|----------|-------------------|
| `extract_bytes(val)` | `ObjData::Bytes` variant; `ObjData::ByteArray` variant; `ObjData::Str` variant (UTF-8 bytes); all-other variant → empty |
| `mb_lzma_compress(data)` | fixed 16-byte in-memory payload → returns bytes |
| `mb_lzma_decompress(data)` | fixed 16-byte in-memory payload → returns bytes |
| `mb_lzma_LZMAFile()` | dict with `__type__: LZMAFile` |
| `mb_lzma_open(_path, _mode)` | delegates to `mb_lzma_LZMAFile` → returns correct dict |
| `mb_lzma_FORMAT_AUTO/XZ/ALONE/RAW()` | 0, 1, 2, 3 |
| `mb_lzma_CHECK_NONE/CRC32/CRC64/SHA256()` | 0, 1, 4, 10 |

**zlib_mod.rs:**

| Function | Branches to cover |
|----------|-------------------|
| `extract_bytes(val)` | `ObjData::Bytes`; `ObjData::ByteArray`; `ObjData::Str`; other → empty |
| `mb_zlib_compress(data)` | fixed payload → returns bytes |
| `mb_zlib_decompress(data)` | fixed payload → returns bytes |
| `mb_zlib_crc32(data)` | empty input → 0x00000000; known payload → verified value; inner loop exercises both `crc & 1 != 0` (XOR) and `crc & 1 == 0` (shift-only) branches |
| `mb_zlib_adler32(data)` | empty input → 1; known single-byte payload → verified value |

### R6: random — secrets_mod.rs (12%)

| Function | Branches to cover |
|----------|-------------------|
| `mb_secrets_token_bytes(n)` | n > 0 → buffer of length n; n = 0 → empty buffer |
| `mb_secrets_token_hex(n)` | n > 0 → 2n-character lowercase hex string |
| `mb_secrets_token_urlsafe(n)` | n > 0 → hex string of length 2n |
| `mb_secrets_choice(seq)` | non-empty list → returns one element; empty list → none; non-list MbValue → none |
| `mb_secrets_randbits(k)` | k < 64 → value masked to k bits; k = 0 → value is 0; k ≥ 64 → mask = u64::MAX |

### R7: typing-and-inspect — abc_mod.rs (14%)

| Function | Branches to cover |
|----------|-------------------|
| `mb_abc_ABC()` | dict has `__class__ = "ABC"` and `__abstract__ = true` |
| `mb_abc_abstractmethod(func)` | dict has `__class__ = "abstractmethod"`, `__abstract__ = true`, and `__func__` equals the supplied func |
| `mb_abc_ABCMeta()` | dict has `__class__ = "ABCMeta"` and `__abstract__ = true` |
## Scenarios

### S-concurrency-1: Queue FIFO ordering

- **Functions**: `mb_queue_Queue`, `mb_queue_put`, `mb_queue_get`
- **Setup**: create Queue with `maxsize = MbValue::from_int(0)`; put items 1, 2, 3
- **Expected**: successive `get` calls return 1, 2, 3 in order
- **Branch covered**: valid queue path in `queue_items_ptr`; non-empty list in `mb_queue_get`

### S-concurrency-2: Queue empty / qsize transitions

- **Functions**: `mb_queue_empty`, `mb_queue_qsize`
- **Setup A**: fresh queue → `empty` = true, `qsize` = 0
- **Setup B**: after one `put` → `empty` = false, `qsize` = 1
- **Branch covered**: empty-list branch and non-empty-list branch in both functions

### S-concurrency-3: Queue with invalid MbValue

- **Functions**: `mb_queue_put`, `mb_queue_get`, `mb_queue_empty`, `mb_queue_qsize`
- **Input**: `MbValue::none()` passed as the queue argument
- **Expected**: no panic; `put` → none, `get` → none, `empty` → true, `qsize` → 0
- **Branch covered**: `queue_items_ptr` returns None (null-ptr path)

### S-concurrency-4: LifoQueue and PriorityQueue construction

- **Functions**: `mb_queue_LifoQueue`, `mb_queue_PriorityQueue`
- **Input**: `maxsize = MbValue::from_int(5)`
- **Expected**: dict with correct `__type__` string and `_maxsize` stored

### S-concurrency-5: Queue full always false

- **Function**: `mb_queue_full`
- **Input**: any MbValue
- **Expected**: `MbValue::from_bool(false)`

### S-concurrency-6: Concurrent put / get via std::thread

- **Functions**: `mb_queue_put`, `mb_queue_get`
- **Setup**: create Queue; spawn producer thread that puts 50 items; main thread gets 50 items
- **Expected**: no panic, no deadlock; 50 items transferred
- **Branch covered**: RwLock write/read under concurrent access

### S-numeric-1: statistics_mean — basic and empty

- **Function**: `mb_statistics_mean`, `extract_floats`
- **Input A**: list `[1, 2, 3, 4, 5]` (ints) → mean = 3.0
- **Input B**: list `[1.5, 2.5]` (floats) → mean = 2.0
- **Input C**: empty list → none
- **Branch covered**: int→float conversion, empty path, sum/count path

### S-numeric-2: statistics_median — odd / even / empty

- **Function**: `mb_statistics_median`
- **Input A**: `[1, 3, 2]` (sorted to [1,2,3]) → 2.0 (odd branch)
- **Input B**: `[1, 2, 3, 4]` → 2.5 (even branch)
- **Input C**: `[]` → none

### S-numeric-3: statistics_variance and stdev — guard on < 2 items

- **Functions**: `mb_statistics_variance`, `mb_statistics_stdev`
- **Input A**: `[2.0, 4.0]` → variance = 2.0, stdev ≈ 1.414
- **Input B**: `[1.0]` → none (< 2 items branch)
- **Input C**: `[]` → none

### S-numeric-4: statistics_mode, geometric_mean, harmonic_mean

- **Functions**: `mb_statistics_mode`, `mb_statistics_geometric_mean`, `mb_statistics_harmonic_mean`
- **Input A mode**: `[1, 2, 2, 3]` → mode = 2.0
- **Input B mode**: `[]` → none
- **Input A geo**: `[1.0, 4.0]` → geometric_mean ≈ 2.0
- **Input A harm**: `[1.0, 2.0, 4.0]` → harmonic_mean ≈ 1.714

### S-numeric-5: bisect_left / bisect_right with duplicates

- **Functions**: `mb_bisect_bisect_left`, `mb_bisect_bisect_right`
- **Input**: sorted list `[1, 2, 2, 3]`, x = 2
  - `bisect_left` → 1 (first 2 position)
  - `bisect_right` → 3 (after last 2)
- **Boundary**: x = 0 → both return 0; x = 4 → both return 4

### S-numeric-6: insort_left / insort_right

- **Functions**: `mb_bisect_insort_left`, `mb_bisect_insort_right`
- **Input A**: sorted list `[1, 3]`, insort_left(2) → `[1, 2, 3]`
- **Input B**: sorted list `[1, 2, 3]`, insort_right(2) → `[1, 2, 2, 3]`
- **Input C**: `MbValue::none()` as list → no-op, no panic

### S-numeric-7: item_key type dispatch

- **Function**: `item_key`
- **Input A**: `MbValue::from_int(7)` → 7
- **Input B**: `MbValue::from_float(3.9)` → 3 (truncated)
- **Input C**: `MbValue::none()` → 0

### S-text-1: shlex_split plain / quoted / empty

- **Function**: `mb_shlex_split`
- **Input A**: `"hello world"` → `["hello", "world"]`
- **Input B**: `"\"hello world\" foo"` → `["hello world", "foo"]` (space inside quotes preserved)
- **Input C**: `""` → `[]`
- **Branch covered**: `in_q` toggle; `cur` non-empty push; loop drain after end

### S-text-2: shlex_quote safe / unsafe / empty

- **Function**: `mb_shlex_quote`
- **Input A**: `"hello_world"` → `"hello_world"` (safe, unchanged)
- **Input B**: `"hello world"` → `"\"hello world\""` (wrapped)
- **Input C**: `""` → `"\"\""` (empty → quoted, `safe && !text.is_empty()` = false)
- **Branch covered**: `safe && !text.is_empty()` true/false

### S-text-3: shlex_join

- **Function**: `mb_shlex_join`
- **Input A**: list `["a", "b", "c"]` → `"a b c"`
- **Input B**: empty list → `""`

### S-text-4: locale_setlocale with / without string

- **Function**: `mb_locale_setlocale`
- **Input A**: `(_cat, MbValue::from_str("fr_FR.UTF-8"))` → `"fr_FR.UTF-8"`
- **Input B**: `(_cat, MbValue::none())` → `"en_US.UTF-8"`
- **Branch covered**: `extract_str` Some/None

### S-text-5: locale_format_string — int / float / other

- **Function**: `mb_locale_format_string`
- **Input A**: fmt=`"count: %d"`, val=int(42) → `"count: 42"`
- **Input B**: fmt=`"pi=%f"`, val=float(3.14159) → `"pi=3.141590"`
- **Input C**: fmt=`"x=%d"`, val=none → `"x=%d"` (no substitution)
- **Branch covered**: `val.as_int()` Some; `val.as_float()` Some; neither

### S-text-6: locale constants

- **Functions**: `mb_locale_LC_ALL`, `mb_locale_LC_CTYPE`, `mb_locale_LC_TIME`, `mb_locale_LC_NUMERIC`
- **Expected**: 6, 0, 2, 1 respectively

### S-datetime-1: isleap — all four branches

- **Function**: `mb_calendar_isleap`
- **Input A**: 2000 → true (÷ 400 rule)
- **Input B**: 1900 → false (÷ 100 but not ÷ 400)
- **Input C**: 2024 → true (÷ 4 but not ÷ 100)
- **Input D**: 2023 → false (not ÷ 4)

### S-datetime-2: monthrange — all match arms

- **Function**: `mb_calendar_monthrange`
- **Input A**: (2024, 1) → (weekday, 31) — 31-day arm
- **Input B**: (2024, 4) → (weekday, 30) — 30-day arm
- **Input C**: (2024, 2) → (weekday, 29) — February leap
- **Input D**: (2023, 2) → (weekday, 28) — February non-leap
- **Input E**: (2024, 13) → (weekday, 30) — invalid month fallback

### S-datetime-3: weekday — known date

- **Function**: `mb_calendar_weekday`
- **Input**: (2024, 1, 1) — January 1 2024 is a Monday
- **Expected**: 0 (Monday = 0 in 0-indexed Mon..Sun)
- **Branch covered**: `m < 3` triggers Zeller year/month adjustment

### S-datetime-4: month_name and day_name list lengths

- **Functions**: `mb_calendar_month_name`, `mb_calendar_day_name`
- **Expected**: month_name list length = 13 (index 0 empty); day_name list length = 7

### S-compress-1: extract_bytes — all four variants (lzma_mod + zlib_mod)

- **Function**: `extract_bytes`
- **Input A**: `MbObject::new_bytes(vec![1, 2, 3])` → `[1, 2, 3]`
- **Input B**: `MbObject::new_byte_array(vec![4, 5, 6])` → `[4, 5, 6]`
- **Input C**: `MbObject::new_str("abc")` → `[97, 98, 99]`
- **Input D**: `MbObject::new_dict()` → `[]`
- **Branch covered**: Bytes, ByteArray, Str, other match arms

### S-compress-2: zlib_crc32 — empty and known payload

- **Function**: `mb_zlib_crc32`
- **Input A**: `[]` → 0x00000000
- **Input B**: `[0x00]` → 0xD202EF8D (verified CRC32 of a single zero byte)
- **Input C**: multi-byte payload exercises both `crc & 1 != 0` (XOR branch) and `crc & 1 == 0` (shift-only branch) in the inner loop

### S-compress-3: zlib_adler32 — empty and single byte

- **Function**: `mb_zlib_adler32`
- **Input A**: `[]` → 1 (a=1 s=0 → (0 << 16) | 1 = 1)
- **Input B**: `[0x01]` → a=2, s=2 → (2 << 16) | 2 = 131074

### S-compress-4: lzma constants

- **Functions**: `mb_lzma_FORMAT_AUTO`, `FORMAT_XZ`, `FORMAT_ALONE`, `FORMAT_RAW`, `CHECK_NONE`, `CHECK_CRC32`, `CHECK_CRC64`, `CHECK_SHA256`
- **Expected**: 0, 1, 2, 3, 0, 1, 4, 10

### S-compress-5: lzma_open returns LZMAFile dict

- **Function**: `mb_lzma_open`
- **Input**: any path and mode MbValues
- **Expected**: dict with `__type__ = "LZMAFile"`

### S-secrets-1: token_bytes length

- **Function**: `mb_secrets_token_bytes`
- **Input A**: `n = MbValue::from_int(16)` → bytes value with exactly 16 elements
- **Input B**: `n = MbValue::from_int(0)` → empty bytes

### S-secrets-2: token_hex and token_urlsafe format

- **Functions**: `mb_secrets_token_hex`, `mb_secrets_token_urlsafe`
- **Input**: `n = MbValue::from_int(8)` → string of length 16 matching `^[0-9a-f]+$`

### S-secrets-3: choice — non-empty / empty / non-list

- **Function**: `mb_secrets_choice`
- **Input A**: list `[1, 2, 3]` → returns one element (non-deterministic, but not none)
- **Input B**: empty list → none
- **Input C**: `MbValue::none()` → none
- **Branch covered**: `items.is_empty()` true; non-list → `and_then` returns None

### S-secrets-4: randbits — mask branches

- **Function**: `mb_secrets_randbits`
- **Input A**: `k = 4` → result is in `[0, 15]`
- **Input B**: `k = 0` → result is 0 (mask = `(1u64 << 0) - 1 = 0`)
- **Input C**: `k = 64` → `bits >= 64` branch; mask = `u64::MAX`
- **Branch covered**: `bits >= 64` true / false

### S-abc-1: ABC dict fields

- **Function**: `mb_abc_ABC`
- **Expected**: returned dict contains `__class__ = "ABC"` and `__abstract__ = true`

### S-abc-2: abstractmethod wraps func

- **Function**: `mb_abc_abstractmethod`
- **Input**: `func = MbValue::from_int(42)`
- **Expected**: dict with `__class__ = "abstractmethod"`, `__abstract__ = true`, and `__func__` storing the supplied value

### S-abc-3: ABCMeta dict fields

- **Function**: `mb_abc_ABCMeta`
- **Expected**: dict with `__class__ = "ABCMeta"` and `__abstract__ = true`
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

All 10 target modules → 100% line + 100% branch under `cargo llvm-cov --branch`.

### Execution Command

```bash
cargo llvm-cov --branch --package cclab-mamba -- --test-threads=4
```

### Test Matrix

| Module | Inline Unit Tests | Integration Tests | Goal |
|--------|-----------------|------------------|------|
| `queue_mod.rs` | 6 | 1 | 100% |
| `statistics_mod.rs` | 14 | 0 | 100% |
| `shlex_mod.rs` | 7 | 0 | 100% |
| `calendar_mod.rs` | 9 | 0 | 100% |
| `locale_mod.rs` | 6 | 0 | 100% |
| `lzma_mod.rs` | 7 | 0 | 100% |
| `zlib_mod.rs` | 7 | 0 | 100% |
| `secrets_mod.rs` | 7 | 0 | 100% |
| `bisect_mod.rs` | 7 | 0 | 100% |
| `abc_mod.rs` | 3 | 0 | 100% |

### Inline Tests per Module

**queue_mod.rs** — replace stub with:
- `test_queue_construction` — exercises `mb_queue_Queue`, `mb_queue_LifoQueue`, `mb_queue_PriorityQueue`; verifies `__type__` field
- `test_queue_put_get_fifo` — put 3 items, get 3 items, assert FIFO order
- `test_queue_empty_and_qsize` — empty→non-empty transitions
- `test_queue_invalid_value` — `MbValue::none()` as queue → no panic, empty/qsize return defaults
- `test_queue_full_always_false` — `mb_queue_full` returns false unconditionally
- `test_queue_concurrent_put_get` — `std::thread::spawn` producer puts 50 items; main thread gets 50 items

**statistics_mod.rs** — replace stub with:
- `test_mean_basic` — `[1,2,3,4,5]` → 3.0
- `test_mean_empty` — `[]` → none
- `test_median_odd` — `[1,3,2]` → 2.0
- `test_median_even` — `[1,2,3,4]` → 2.5
- `test_median_empty` — `[]` → none
- `test_mode_basic` — `[1,2,2,3]` → 2.0
- `test_mode_empty` — `[]` → none
- `test_variance_basic` — `[2.0,4.0]` → 2.0
- `test_variance_too_few` — `[1.0]` → none
- `test_stdev_basic` — `[2.0,4.0]` → ~1.414
- `test_stdev_too_few` — `[1.0]` → none
- `test_geometric_mean_basic` — `[1.0,4.0]` → ~2.0
- `test_geometric_mean_empty` — `[]` → none
- `test_harmonic_mean_basic` — `[1.0,2.0,4.0]` → ~1.714

**shlex_mod.rs** — replace stub with:
- `test_split_plain` — `"hello world"` → 2 tokens
- `test_split_quoted` — `"\"hello world\" foo"` → `["hello world", "foo"]`
- `test_split_empty` — `""` → 0 tokens
- `test_quote_safe` — `"hello_world"` → unchanged
- `test_quote_unsafe` — `"hello world"` → wrapped in quotes
- `test_quote_empty` — `""` → `"\"\""`
- `test_join_basic` — `["a","b","c"]` → `"a b c"`

**calendar_mod.rs** — replace stub with:
- `test_isleap_400` — 2000 → true
- `test_isleap_100` — 1900 → false
- `test_isleap_4` — 2024 → true
- `test_isleap_none` — 2023 → false
- `test_leapdays_range` — leapdays(1900, 2000) → expected count
- `test_monthrange_31` — (2024, 1) → 31 days
- `test_monthrange_30` — (2024, 4) → 30 days
- `test_monthrange_feb_leap` — (2024, 2) → 29 days
- `test_monthrange_feb_normal` — (2023, 2) → 28 days
- `test_month_name_count` — list length = 13, index 0 empty
- `test_day_name_count` — list length = 7
- `test_weekday_known_date` — (2024, 1, 1) → 0 (Monday)

**locale_mod.rs** — replace stub with:
- `test_getlocale_tuple` — first element = "en_US", second = "UTF-8"
- `test_setlocale_with_str` — echoes supplied locale string
- `test_setlocale_without_str` — returns "en_US.UTF-8" default
- `test_format_string_int` — `%d` substituted
- `test_format_string_float` — `%f` substituted
- `test_lc_constants` — LC_ALL=6, LC_CTYPE=0, LC_TIME=2, LC_NUMERIC=1

**lzma_mod.rs** — replace stub with:
- `test_extract_bytes_bytes_variant`
- `test_extract_bytes_str_variant` — UTF-8 to bytes
- `test_extract_bytes_other_variant` — dict → empty
- `test_compress_returns_bytes`
- `test_decompress_returns_bytes`
- `test_lzmafile_type_field` — `__type__ = "LZMAFile"`
- `test_format_and_check_constants`

**zlib_mod.rs** — replace stub with:
- `test_extract_bytes_bytes_variant`
- `test_extract_bytes_str_variant`
- `test_extract_bytes_other_variant`
- `test_compress_returns_bytes`
- `test_crc32_empty` — `[]` → 0
- `test_crc32_known` — single zero byte → 0xD202EF8D; exercises both inner-loop branches
- `test_adler32_empty` — `[]` → 1
- `test_adler32_known` — `[0x01]` → 131074

**secrets_mod.rs** — replace stub with:
- `test_token_bytes_length` — n=16 → 16 elements
- `test_token_bytes_zero` — n=0 → empty
- `test_token_hex_format` — n=8 → 16-char hex string
- `test_token_urlsafe_format` — n=4 → 8-char hex string
- `test_choice_nonempty` — list `[1,2,3]` → not none
- `test_choice_empty` — empty list → none
- `test_randbits_bounds` — k=4 → value in [0,15]; k=0 → 0; k=64 → any u64

**bisect_mod.rs** — replace stub with:
- `test_bisect_left_duplicates` — `[1,2,2,3]`, x=2 → 1
- `test_bisect_right_duplicates` — `[1,2,2,3]`, x=2 → 3
- `test_bisect_boundary_before` — x=0 → 0
- `test_bisect_boundary_after` — x=4 → 4
- `test_insort_left` — maintains sorted order
- `test_insort_right` — inserts after equal elements
- `test_item_key_variants` — int, float, none → correct i64

**abc_mod.rs** — replace stub with:
- `test_abc_fields` — `__class__="ABC"`, `__abstract__=true`
- `test_abstractmethod_wraps_func` — `__func__` stores supplied value
- `test_abcmeta_fields` — `__class__="ABCMeta"`, `__abstract__=true`

### Integration Test (`crates/mamba/tests/stdlib_coverage_lower_tests.rs`)

```yaml
tests:
  - name: test_queue_concurrent_cross_module
    desc: |
      Create queue via mb_queue_Queue. Spawn producer thread that calls
      mb_queue_put 100 times. Main thread calls mb_queue_get 100 times.
      Assert total non-none results == 100. No panic or deadlock.
```

### Acceptance Criteria

```yaml
criteria:
  - cargo test -p mamba passes with 0 failures
  - cargo llvm-cov --branch -p mamba reports 100% line for all 10 target files
  - cargo llvm-cov --branch -p mamba reports 100% branch for all 10 target files
  - No coverage-exclusion annotations added (no #[allow(dead_code)], no tarpaulin attrs)
  - All tests are self-contained (no network I/O, no persistent filesystem writes)
```
## Changes

files:
  - path: crates/mamba/src/runtime/stdlib/queue_mod.rs
    action: modify
    description: |
      Replace stub #[cfg(test)] block with 6 inline unit tests:
      test_queue_construction, test_queue_put_get_fifo, test_queue_empty_and_qsize,
      test_queue_invalid_value, test_queue_full_always_false, test_queue_concurrent_put_get.
      Concurrent test uses std::thread::spawn; no external dependencies.

  - path: crates/mamba/src/runtime/stdlib/statistics_mod.rs
    action: modify
    description: |
      Replace stub block with 14 inline unit tests covering mean/median/mode/variance/
      stdev/geometric_mean/harmonic_mean — both non-empty and empty list branches.

  - path: crates/mamba/src/runtime/stdlib/shlex_mod.rs
    action: modify
    description: |
      Replace stub block with 7 inline unit tests covering split (plain/quoted/empty),
      quote (safe/unsafe/empty string), and join (basic/empty).

  - path: crates/mamba/src/runtime/stdlib/calendar_mod.rs
    action: modify
    description: |
      Replace stub block with 9 inline unit tests covering isleap (4 branches),
      leapdays, monthrange (5 match arms), month_name list length, day_name list length,
      and weekday for a known date.

  - path: crates/mamba/src/runtime/stdlib/locale_mod.rs
    action: modify
    description: |
      Replace stub block with 6 inline unit tests covering getlocale tuple,
      setlocale (str/non-str branches), format_string (int/%d, float/%f, other),
      and all four LC_* constants.

  - path: crates/mamba/src/runtime/stdlib/lzma_mod.rs
    action: modify
    description: |
      Replace stub block with 7 inline unit tests covering extract_bytes
      (Bytes/ByteArray/Str/other variants), compress/decompress roundtrip with
      fixed in-memory payload, LZMAFile dict construction, open() delegation,
      and FORMAT_*/CHECK_* int constants.

  - path: crates/mamba/src/runtime/stdlib/zlib_mod.rs
    action: modify
    description: |
      Replace stub block with 7 inline unit tests covering extract_bytes
      (all 4 variants), compress/decompress, crc32 (empty=0, known single-byte
      value 0xD202EF8D, multi-byte payload exercising both inner-loop branches),
      adler32 (empty=1, known single-byte value 131074).

  - path: crates/mamba/src/runtime/stdlib/secrets_mod.rs
    action: modify
    description: |
      Replace stub block with 7 inline unit tests covering token_bytes (length/zero),
      token_hex (hex format), token_urlsafe (hex format), choice (nonempty/empty/
      non-list branches), randbits (k<64, k=0, k>=64 branches).

  - path: crates/mamba/src/runtime/stdlib/bisect_mod.rs
    action: modify
    description: |
      Replace stub block with 7 inline unit tests covering bisect_left/right
      with duplicate elements, boundary cases (before-all, after-all), insort_left/
      right (maintains order, handles null list), and item_key type dispatch
      (int/float/none).

  - path: crates/mamba/src/runtime/stdlib/abc_mod.rs
    action: modify
    description: |
      Replace stub block with 3 inline unit tests: ABC dict fields (__class__,
      __abstract__), abstractmethod dict + __func__ storage, ABCMeta dict fields.

  - path: crates/mamba/tests/stdlib_coverage_lower_tests.rs
    action: create
    description: |
      New integration test file. Contains test_queue_concurrent_cross_module:
      creates a queue, spawns a producer thread that puts 100 items, main thread
      gets 100 items, asserts total non-none results == 100.
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

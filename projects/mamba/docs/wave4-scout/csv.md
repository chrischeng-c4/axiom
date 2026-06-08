# csv — startup-dominated for short streams, balanced for long

**Source:** `projects/mamba/vendor/typeshed/stdlib/csv.pyi` (151 lines).

## Surface (20-entry __all__ + 2 conditional)

- **Constants (4 + 2 conditional):** `QUOTE_MINIMAL`, `QUOTE_ALL`,
  `QUOTE_NONNUMERIC`, `QUOTE_NONE`. Plus 3.12+: `QUOTE_STRINGS`,
  `QUOTE_NOTNULL`.
- **Def (5):** `reader`, `writer`, `register_dialect`, `unregister_dialect`,
  `get_dialect`, `list_dialects`, `field_size_limit`.
- **Class (7):** `Error`, `Dialect`, `excel`, `excel_tab`, `unix_dialect`,
  `DictReader`, `DictWriter`, `Sniffer`.

## Hot-path classification — line-iterator + tokenize

CSV is **stream-iterator + per-line tokenize**. `reader(iterable)`
returns an iterator yielding `list[str]` per row. Tokenization is
state-machine (~5 states: in-field, in-quoted, post-quote, etc.).

- Hot path: `for row in reader(file_lines): ...` — one
  `mb_iter`/`mb_next` per row, internally calls a tokenize fn that
  builds a list of fields (one new str per field via slice).
- For 10k rows × 5 fields: ~50k str allocations + 10k list allocations
  + 10k iterator state advances. **Borderline subset B at scale**,
  but realistic CSV workloads are 1k-10k rows = sub-#2096-threshold.

## Predicted regime

- **Short stream (< 100 rows)**: startup-dominated. Wall ratio
  depends entirely on mamba boot + module-import. Probably wall PASS.
- **Long stream (10k rows × 5 cols)**: balanced. Tokenize is pure
  string scan (compute-friendly); list/str allocation is the friction.
  Expected wall 1.5-5×, internal 0.8-3×.

**Tier:** **compute** for tokenize hot loop; **balanced** overall.

**Expected G2:** wall 1.5-5×, internal 0.8-3×, mem 0.7-1.0× (subset
B borderline at 10k+ rows).

## Blockers / carve-outs

- **Iterator protocol**: `reader()` returns an iterator; need
  `__iter__`/`__next__` dispatch through mamba's iter table.
  Established pattern (used by re.finditer, zip, etc.) — reuse.
- **DictReader / DictWriter**: subclass logic adding fieldnames.
  Can be deferred to wave-5 if the LOC budget runs out; forward
  ship is `reader`/`writer` only.
- **Dialect classes**: 3 dialect Instance shells (excel, excel_tab,
  unix_dialect). Implement as Instance with frozen attribute fields.
- **Sniffer**: heuristic delimiter detection. Deferred — niche, not
  on any realistic perf hot path.
- **No callbacks** in reader/writer hot path. No #2100 risk.
- **No operator overloads.** No #2129 risk.
- **Subset B borderline at scale**: document but don't carve. Realistic
  CSV workloads are < 100k rows.

## Bench sketch

```python
# bench/reader_rows.py — tier: compute
import csv, io, sys, time
_reader = csv.reader  # #2097 hoist
CSV_TEXT = "\n".join([",".join([f"v{i}" for i in range(5)]) for _ in range(10_000)])
ITERS = 5
acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    rdr = _reader(io.StringIO(CSV_TEXT))
    for row in rdr:
        acc += len(row)
_t1 = time.perf_counter()
print("reader_rows:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

## LOC / G2 estimate

- **Estimated LOC**: ~320 (reader + writer state machines + 3 dialect
  Instances + 5 constants + Error class).
- **Expected G2**: wall 1.5-5× PASS likely, internal 0.8-3× PASS
  borderline, mem 0.7-1.0× PASS borderline.

## Ship-now confidence: **MEDIUM-HIGH** (wave-4 #2)

Rationale: no risky blockers (no #2100, no #2129, sub-#2096-threshold
at realistic sizes). State-machine tokenize is well-understood. Main
risk is the iterator protocol wiring + StringIO interop. Slightly
behind urllib.parse on pure-compute confidence but well ahead of
pickle and xml.

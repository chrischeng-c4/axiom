# pickle — subset B small-allocation-many-times (json-shaped)

**Source:** `projects/mamba/vendor/typeshed/stdlib/pickle.pyi` (240 lines, 82-entry __all__).

## Surface (82-entry __all__)

- **Constants (~70):** `HIGHEST_PROTOCOL`, `DEFAULT_PROTOCOL`, `MARK`, `STOP`,
  `POP`, `POP_MARK`, `DUP`, `FLOAT`, `INT`, `BININT`, `LONG`, `NONE`, `PERSID`,
  `BINPERSID`, `REDUCE`, `STRING`, `BINSTRING`, `SHORT_BINSTRING`, `UNICODE`,
  `BINUNICODE`, `APPEND`, `BUILD`, `GLOBAL`, `DICT`, `EMPTY_DICT`, `APPENDS`,
  `GET`, `BINGET`, `INST`, `LONG_BINGET`, `LIST`, `EMPTY_LIST`, `OBJ`, `PUT`,
  `BINPUT`, `LONG_BINPUT`, `SETITEM`, `TUPLE`, `EMPTY_TUPLE`, `SETITEMS`,
  `BINFLOAT`, `PROTO`, `NEWOBJ`, `EXT1`, `EXT2`, `EXT4`, `TUPLE1`, `TUPLE2`,
  `TUPLE3`, `NEWTRUE`, `NEWFALSE`, `LONG1`, `LONG4`, `BINBYTES`, `SHORT_BINBYTES`,
  `SHORT_BINUNICODE`, `BINUNICODE8`, `BINBYTES8`, `EMPTY_SET`, `ADDITEMS`,
  `FROZENSET`, `NEWOBJ_EX`, `STACK_GLOBAL`, `MEMOIZE`, `FRAME`, `BYTEARRAY8`,
  `NEXT_BUFFER`, `READONLY_BUFFER`. All `Final = b"<opcode-byte>"` constants.
- **Def (4):** `dump`, `dumps`, `load`, `loads`.
- **Class (8):** `PickleBuffer`, `PickleError`, `PicklingError`,
  `UnpicklingError`, `Pickler`, `Unpickler` (+ private `_Pickler`,
  `_Unpickler`).

## Hot-path classification — **subset B small-allocation-many-times**

Per `project_mamba_phase2_crosscutting_blockers` memory update:
**pickle is pre-classified as subset B** — many small objects per call.
A `pickle.dumps(complex_dict)` allocates 50-500 internal objects
(opcodes, memo entries, len-prefixes); `pickle.loads` reverses this
with str/list/dict allocations.

- Hot loop: `dumps({"k": "v" for k in range(N)})` allocates ~N memo
  entries + N str opcodes + N value strings = ~3N small objects.
- Mamba's per-object header overhead vs CPython's PyObject_Malloc
  arena will produce a **mem ratio < 0.5×** at scale (json json
  precedent: 0.31× at 10k iters × 50 small objects).

## Predicted regime — startup-dominated for small, allocation-bound for large

- Small workload (single small dict round-trip): startup-dominated;
  wall PASS by accident.
- Large workload (10k iters × 50-key dict dumps+loads): mamba mem
  FAIL **by design per #2096 subset B**; wall ratio could go either
  way depending on how mamba's allocator behaves under sustained
  pressure.

**Tier:** **compute** (no callbacks beyond `__reduce__` dunder, which
is OOP-handle territory; forward ship of dumps/loads for builtin
types only).

**Expected G2:** wall ~0.5-2×, internal 0.3-1.5×, mem **0.3-0.5× FAIL
by-design per #2096 subset B**. Ship anyway with the carve-out
documented (precedent: array Task #35, json Task #29).

## Blockers / carve-outs

- **#2096 subset B (mem FAIL by-design)**: documented carve-out.
- **`__reduce__` / `__reduce_ex__` dunder methods**: any user-defined
  class going through pickle must call its `__reduce__` — that's
  #2100 callback territory. **Carve out**: forward ship of pickle
  for *builtin types only* (int, float, str, bytes, list, dict,
  tuple, frozenset, set, None, True, False). Custom classes raise
  `PicklingError` with explicit "custom __reduce__ unsupported in
  wave-4 ship".
- **Protocol versions**: support protocol 4 (3.8+) only; protocols
  0/1/2/3 deferred. Justification: protocol 4 is the modern default
  and covers the realistic perf benchmark.
- **PickleBuffer + out-of-band buffers**: deferred. Forward ship
  uses in-band bytes only.
- **persistent_id / persistent_load**: deferred (callback-bound,
  #2100).

## Bench sketch

```python
# bench/dumps_loads_dict.py — tier: compute (mem FAIL by-design per #2096 B)
import pickle, sys, time
_dumps = pickle.dumps  # #2097 hoist
_loads = pickle.loads
DATA = {f"k{i}": f"v{i}" for i in range(50)}
ITERS = 10_000
acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    b = _dumps(DATA)
    d = _loads(b)
    acc += len(d)
_t1 = time.perf_counter()
print("dumps_loads_dict:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)
```

## LOC / G2 estimate

- **Estimated LOC**: ~420 (opcode emitter + opcode parser; builtin
  type dispatch table; ~70 opcode constants register block).
- **Expected G2**: wall 0.5-2×, internal 0.3-1.5×, **mem 0.3-0.5×
  FAIL by-design**.

## Ship-now confidence: **MEDIUM** (wave-4 #3)

Rationale: pre-classified subset B mem FAIL is expected, NOT a new
finding. Forward ship of builtin-only protocol 4 is well-scoped at
~420 LOC. The bytecode dispatch table is mechanical. Risk: opcode
emitter correctness on edge cases (long ints, NaN floats, recursive
dicts) — needs careful conformance testing against CPython's pickle
test suite.

# Spec seed for CPython IndexError / KeyError contract on the
# `list.pop(out_of_range_idx)` / `range(N)[out_of_range_idx]` /
# `del list[out_of_range_idx]` / `del dict[missing_key]` corners
# where mamba silently no-ops or silently returns `None` instead of
# raising the canonical IndexError / KeyError.
#
# Surface: CPython rejects (1) `list.pop(idx)` when `idx` is out of
# range — IndexError("pop index out of range"); (2) `range(N)[idx]`
# when `idx >= N` or `idx < -N` — IndexError("range object index
# out of range"); (3) `del list[idx]` when `idx` is out of range —
# IndexError("list assignment index out of range"); (4)
# `del dict[k]` when `k` is missing — KeyError(k); (5)
# `list.__delitem__(idx)` (the explicit del-protocol call) when
# `idx` is out of range — IndexError.
#
# Mamba accepts every form and silently returns `None` for
# `list.pop(99)` / `range(N)[99]` (so code like
# `task_queue.pop(priority_idx)` silently produces `None` whenever
# the priority index has been bumped beyond the queue length, masking
# a real "no task at that priority" condition), and silently no-ops
# `del list[idx]` / `del dict[k]` (so cleanup code like
# `del cache[stale_key]` silently leaves a stale entry whenever the
# key is already absent, masking a double-free / double-eviction
# bug).
#
# Existing lang_indexerror_zerodiv_silent.py covers the READING
# subscript family (`list[idx]`, `tuple[idx]`, `str[idx]`,
# `bytes[idx]` out of range) — those are READ operations.
# Existing lang_dict_view_del_tuple_index_silent.py covers
# `tuple.index(absent)` and `dict_view[idx]`. This seed covers the
# FRESH divergence family — the MUTATING `pop` / `del` operations
# (which CPython enforces at the SAME bounds-check level as reads,
# but mamba silently passes through), plus the `range` subscript
# corner (which is a separate type with its own __getitem__).
#
# Probes (every form CPython rejects, mamba silently no-ops):
#   • [1,2,3].pop(99)              → mamba: None         (IndexError)
#   • [1,2,3].pop(-99)             → mamba: None         (IndexError)
#   • [].pop(0)                    → mamba: None? raises (IndexError)
#   • range(0)[0]                  → mamba: None         (IndexError)
#   • range(5)[99]                 → mamba: None         (IndexError)
#   • range(5)[-99]                → mamba: None         (IndexError)
#   • del L[99]                    → mamba: list intact  (IndexError)
#   • del L[-99]                   → mamba: list intact  (IndexError)
#   • del d[missing]               → mamba: dict intact  (KeyError)
#   • L.__delitem__(99)            → mamba: list intact  (IndexError)
#
# CPython contract (uniform across every form):
#   list.pop(out_of_range_idx) / list.__delitem__(out_of_range_idx)
#       → IndexError("pop index out of range" /
#                    "list assignment index out of range");
#   range(N)[out_of_range_idx]
#       → IndexError("range object index out of range");
#   del dict[missing_key]
#       → KeyError(key).
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

# list.pop(99) — index past end
try:
    _l: Any = [1, 2, 3]
    _ = _l.pop(99)
    raise AssertionError("[1,2,3].pop(99) must raise IndexError")
except IndexError:
    _ledger.append(1)

# list.pop(-99) — index past start
try:
    _l: Any = [1, 2, 3]
    _ = _l.pop(-99)
    raise AssertionError("[1,2,3].pop(-99) must raise IndexError")
except IndexError:
    _ledger.append(1)

# [].pop(0) — empty list, any explicit index
try:
    _l: Any = []
    _ = _l.pop(0)
    raise AssertionError("[].pop(0) must raise IndexError")
except IndexError:
    _ledger.append(1)

# range(0)[0] — empty range, any index
try:
    _r: Any = range(0)
    _ = _r[0]
    raise AssertionError("range(0)[0] must raise IndexError")
except IndexError:
    _ledger.append(1)

# range(5)[99] — index past end
try:
    _r: Any = range(5)
    _ = _r[99]
    raise AssertionError("range(5)[99] must raise IndexError")
except IndexError:
    _ledger.append(1)

# range(5)[-99] — index past start
try:
    _r: Any = range(5)
    _ = _r[-99]
    raise AssertionError("range(5)[-99] must raise IndexError")
except IndexError:
    _ledger.append(1)

# del list[99] — index past end
try:
    _l: Any = [1, 2, 3]
    del _l[99]
    raise AssertionError("del [1,2,3][99] must raise IndexError")
except IndexError:
    _ledger.append(1)

# del list[-99] — index past start
try:
    _l: Any = [1, 2, 3]
    del _l[-99]
    raise AssertionError("del [1,2,3][-99] must raise IndexError")
except IndexError:
    _ledger.append(1)

# del dict[missing] — key not in dict
try:
    _d: Any = {1: 2, 3: 4}
    del _d["missing"]
    raise AssertionError("del {1:2,3:4}['missing'] must raise KeyError")
except KeyError:
    _ledger.append(1)

# del dict[missing_int] — int key not in dict
try:
    _d: Any = {1: 2}
    del _d[99]
    raise AssertionError("del {1:2}[99] must raise KeyError")
except KeyError:
    _ledger.append(1)

# del {}[anything] — empty dict
try:
    _d: Any = {}
    del _d["k"]
    raise AssertionError("del {}['k'] must raise KeyError")
except KeyError:
    _ledger.append(1)

# list.__delitem__(99) — explicit del-protocol call
try:
    _l: Any = [1, 2, 3]
    _l.__delitem__(99)
    raise AssertionError("[1,2,3].__delitem__(99) must raise IndexError")
except IndexError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_mutator_pop_del_out_of_range_silent {sum(_ledger)} asserts")

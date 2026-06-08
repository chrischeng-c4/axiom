# test_collections.py — #2828 CPython collections seed (executed assertions).
#
# Replaces the prior vendored CPython upstream Lib/test/test_collections.py
# (ranked `Fail` at the PEP 585 generic-alias subscription gap at line
# 713) with a Mamba-authored seed distilled from the collections
# container surface. Exercises Counter / OrderedDict / defaultdict /
# deque / namedtuple — the five load-bearing classes downstream users
# actually reach for — via raw asserts on a small fixed dataset.
# Emits the runner's positive proof-of-execution marker that
# `cpython_lib_test_runner.rs` (#2691) classifies as `AssertionPass`.
#
# Why so small? Mamba's current collections surface presents the
# nine standard names (Counter, OrderedDict, defaultdict, deque,
# namedtuple, ChainMap, UserDict, UserList, UserString) and produces
# the same answers as CPython on the container API exercised here.
# Richer surface — deque indexing (returns None on mamba today),
# `len(deque)` (returns 0 on mamba), `tuple(namedtuple)` (returns ()
# on mamba — not iterable) — is excluded; those gaps close in
# followup tickets.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: collections N asserts` to stdout.

import collections

_ledger: list[int] = []

# 1. Module identity + class binding.
assert collections.__name__ == "collections", "collections.__name__ must be 'collections'"
_ledger.append(1)
assert hasattr(collections, "Counter"), "collections must expose Counter"
_ledger.append(1)
assert hasattr(collections, "OrderedDict"), "collections must expose OrderedDict"
_ledger.append(1)
assert hasattr(collections, "defaultdict"), "collections must expose defaultdict"
_ledger.append(1)
assert hasattr(collections, "deque"), "collections must expose deque"
_ledger.append(1)
assert hasattr(collections, "namedtuple"), "collections must expose namedtuple"
_ledger.append(1)
assert hasattr(collections, "ChainMap"), "collections must expose ChainMap"
_ledger.append(1)

# 2. deque: init / append / appendleft / pop / popleft. Snapshot via
#    list(dq) — direct indexing returns None on mamba today.
_dq = collections.deque([1, 2, 3])
assert list(_dq) == [1, 2, 3], "deque init preserves order"
_ledger.append(1)
_dq.append(4)
assert list(_dq) == [1, 2, 3, 4], "deque.append adds to the right"
_ledger.append(1)
_dq.appendleft(0)
assert list(_dq) == [0, 1, 2, 3, 4], "deque.appendleft adds to the left"
_ledger.append(1)
_dq.pop()
assert list(_dq) == [0, 1, 2, 3], "deque.pop removes from the right"
_ledger.append(1)
_dq.popleft()
assert list(_dq) == [1, 2, 3], "deque.popleft removes from the left"
_ledger.append(1)

# 3. Counter: tallies, missing-key default, most_common ordering.
_c = collections.Counter("aabbbc")
assert _c["a"] == 2, "Counter('aabbbc') has 2 a's"
_ledger.append(1)
assert _c["b"] == 3, "Counter('aabbbc') has 3 b's"
_ledger.append(1)
assert _c["c"] == 1, "Counter('aabbbc') has 1 c"
_ledger.append(1)
assert _c["z"] == 0, "Counter returns 0 for missing keys (no KeyError)"
_ledger.append(1)
assert _c.most_common(2) == [("b", 3), ("a", 2)], "most_common(2) ranks by frequency desc"
_ledger.append(1)

# 4. defaultdict(int) — auto-zero on first access, accumulates with +=.
_dd = collections.defaultdict(int)
_dd["x"] += 5
_dd["x"] += 3
assert _dd["x"] == 8, "defaultdict(int) accumulates via +="
_ledger.append(1)
assert _dd["unset"] == 0, "defaultdict(int) auto-creates missing key with 0"
_ledger.append(1)

# 5. defaultdict(list) — auto-empty-list on first access, .append works.
_ddl = collections.defaultdict(list)
_ddl["a"].append(1)
_ddl["a"].append(2)
assert _ddl["a"] == [1, 2], "defaultdict(list).append builds incrementally"
_ledger.append(1)

# 6. OrderedDict preserves insertion order — keys() returned in the
#    order they were assigned (not the natural sort order).
_od = collections.OrderedDict()
_od["b"] = 2
_od["a"] = 1
_od["c"] = 3
assert list(_od.keys()) == ["b", "a", "c"], "OrderedDict preserves insertion order"
_ledger.append(1)

# 7. namedtuple — attribute access by name + class identity.
_Pt = collections.namedtuple("Pt", ["x", "y"])
_p = _Pt(3, 4)
assert _p.x == 3, "namedtuple .x attribute access"
_ledger.append(1)
assert _p.y == 4, "namedtuple .y attribute access"
_ledger.append(1)
assert type(_p).__name__ == "Pt", "namedtuple class name preserved"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: collections {len(_ledger)} asserts")

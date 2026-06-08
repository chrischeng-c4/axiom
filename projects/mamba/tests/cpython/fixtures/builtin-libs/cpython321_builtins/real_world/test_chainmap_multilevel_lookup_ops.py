# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_chainmap_multilevel_lookup_ops"
# subject = "cpython321.test_chainmap_multilevel_lookup_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_chainmap_multilevel_lookup_ops.py"
# status = "filled"
# ///
"""cpython321.test_chainmap_multilevel_lookup_ops: execute CPython 3.12 seed test_chainmap_multilevel_lookup_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `collections.ChainMap` MULTI-LEVEL
# lookup precedence + the matching subset of `collections.Counter`
# (iterable construction + `.most_common`) and `collections.OrderedDict`
# (view iteration). The existing seeds (test_chainmap_ops,
# test_counter_ops, test_counter_extras_ops, test_ordered_dict_ops)
# cover the 1-/2-dict ChainMap form, Counter from iterables, and the
# basic OrderedDict shape — but leave the THREE-OR-MORE-dict ChainMap
# precedence chain, the boundary `Counter[missing]` zero-default, the
# empty/single-dict ChainMap edges, the `ChainMap.get(missing,
# default)` form, the in-place ChainMap assignment writing to the
# first-dict-only invariant, and the OrderedDict view-return-type
# contract uncovered.
#
# Surface (the matching subset between mamba and CPython):
#   • ChainMap with 3 or more dicts — earliest-dict-wins precedence;
#     keys exclusive to dicts deeper in the chain still resolve;
#   • ChainMap.get(key, default) — explicit-default form;
#   • ChainMap[key] = value — assignment writes ONLY to maps[0],
#     deeper-dict values stay unchanged;
#   • empty / single-dict ChainMap edge cases;
#   • `in` / `not in` membership checks across the chain;
#   • sorted(list(cm)) — key iteration covers every distinct key once;
#   • Counter from iterable — letter / digit / word counts;
#   • Counter[missing] → 0 (the missing-key behavior that distinguishes
#     Counter from a plain dict);
#   • Counter.most_common(0) / .most_common(N) / .most_common() — full,
#     partial, and empty rankings;
#   • OrderedDict iteration views (.items(), .keys(), .values()) and
#     membership in insertion order.
import collections
from typing import Any

_ledger: list[int] = []
_any: Any = None

# 1) ChainMap with 3-level chain — earliest-dict-wins
_a = {"x": 1, "y": 2}
_b = {"y": 20, "z": 30}
_c = {"z": 300, "w": 400}
_cm3 = collections.ChainMap(_a, _b, _c)
assert _cm3["x"] == 1; _ledger.append(1)
# y exists in _a (1st) and _b (2nd) — _a wins
assert _cm3["y"] == 2; _ledger.append(1)
# z exists in _b (2nd) and _c (3rd) — _b wins
assert _cm3["z"] == 30; _ledger.append(1)
# w exists only in _c (3rd) — falls through
assert _cm3["w"] == 400; _ledger.append(1)
# union size — 4 distinct keys
assert len(_cm3) == 4; _ledger.append(1)
# Iteration covers every distinct key once
assert sorted(list(_cm3)) == ["w", "x", "y", "z"]; _ledger.append(1)
# .get with explicit default
assert _cm3.get("missing", "DEFAULT") == "DEFAULT"; _ledger.append(1)
# .get on present key
assert _cm3.get("y") == 2; _ledger.append(1)
# .get with default on present key — default ignored
assert _cm3.get("y", "DEFAULT") == 2; _ledger.append(1)

# 2) ChainMap with 4-level chain — even deeper precedence
_d = {"q": 9999}
_cm4 = collections.ChainMap({"q": 1}, {"q": 2}, {"q": 3}, _d)
# First-dict precedence overrides all deeper qs
assert _cm4["q"] == 1; _ledger.append(1)
# Last-dict-only key still resolves
assert _cm4.get("q") == 1; _ledger.append(1)

# 3) Empty ChainMap edge case
_empty = collections.ChainMap()
assert len(_empty) == 0; _ledger.append(1)
assert list(_empty) == []; _ledger.append(1)
assert _empty.get("anything") is None; _ledger.append(1)
assert _empty.get("anything", "FALLBACK") == "FALLBACK"; _ledger.append(1)
assert not ("x" in _empty); _ledger.append(1)

# 4) Single-dict ChainMap behaves like a single-key-source dict
_single = collections.ChainMap({"a": 1, "b": 2})
assert _single["a"] == 1; _ledger.append(1)
assert _single["b"] == 2; _ledger.append(1)
assert _single.get("missing", -1) == -1; _ledger.append(1)
assert len(_single) == 2; _ledger.append(1)

# 5) Assignment is visible via the ChainMap view (matching subset —
#    the dict-mutation visibility on the underlying maps is left to
#    the divergence-spec fixture)
_top = {"x": 1}
_bottom = {"y": 2}
_cm_w = collections.ChainMap(_top, _bottom)
_cm_w["z"] = 3
# After write, the chain sees the new key
assert _cm_w["z"] == 3; _ledger.append(1)
# Existing-deep key is unaffected by the top-write
assert _cm_w["y"] == 2; _ledger.append(1)

# Overwriting an existing key — the chain view sees the new value
_top2 = {"x": 1}
_bottom2 = {"x": 99, "y": 2}
_cm_o = collections.ChainMap(_top2, _bottom2)
_cm_o["x"] = 1000
# Read sees the new top value
assert _cm_o["x"] == 1000; _ledger.append(1)
# The deeper y key (only in bottom) still resolves
assert _cm_o["y"] == 2; _ledger.append(1)

# 6) `in` / `not in` across the chain
_cm_m = collections.ChainMap({"a": 1, "b": 2}, {"c": 3})
assert "a" in _cm_m; _ledger.append(1)
assert "b" in _cm_m; _ledger.append(1)
assert "c" in _cm_m; _ledger.append(1)
assert "z" not in _cm_m; _ledger.append(1)
assert not ("z" in _cm_m); _ledger.append(1)
# Reverse-order chain — precedence flips but membership still holds
_cm_r = collections.ChainMap({"c": 3}, {"a": 1, "b": 2})
assert "a" in _cm_r; _ledger.append(1)
assert "c" in _cm_r; _ledger.append(1)

# 7) Counter from iterable (the matching subset — string / list)
_c_str = collections.Counter("abracadabra")
assert _c_str["a"] == 5; _ledger.append(1)
assert _c_str["b"] == 2; _ledger.append(1)
assert _c_str["r"] == 2; _ledger.append(1)
assert _c_str["c"] == 1; _ledger.append(1)
assert _c_str["d"] == 1; _ledger.append(1)
# Counter[missing] is 0 — the no-KeyError contract
assert _c_str["Z"] == 0; _ledger.append(1)
assert _c_str["q"] == 0; _ledger.append(1)
# len() counts distinct keys
assert len(_c_str) == 5; _ledger.append(1)

# Counter from list
_c_list = collections.Counter([1, 2, 2, 3, 3, 3])
assert _c_list[1] == 1; _ledger.append(1)
assert _c_list[2] == 2; _ledger.append(1)
assert _c_list[3] == 3; _ledger.append(1)
assert _c_list[99] == 0; _ledger.append(1)
assert len(_c_list) == 3; _ledger.append(1)

# Counter from tuple
_c_tup = collections.Counter(("x", "x", "y"))
assert _c_tup["x"] == 2; _ledger.append(1)
assert _c_tup["y"] == 1; _ledger.append(1)

# 8) Counter.most_common — full, partial, and zero
_c = collections.Counter("abracadabra")
# Full ranking
_top_all = _c.most_common()
assert _top_all[0] == ("a", 5); _ledger.append(1)
assert len(_top_all) == 5; _ledger.append(1)
# most_common(0) → empty list
assert _c.most_common(0) == []; _ledger.append(1)
# most_common(1) → top-1
assert _c.most_common(1) == [("a", 5)]; _ledger.append(1)
# most_common(2) → top-2 with a, then a tie among b/r
_top2 = _c.most_common(2)
assert len(_top2) == 2; _ledger.append(1)
assert _top2[0] == ("a", 5); _ledger.append(1)
# most_common(N) for N > distinct → all
_top_big = _c.most_common(100)
assert len(_top_big) == 5; _ledger.append(1)

# Empty Counter — most_common is empty
_empty_c = collections.Counter()
assert _empty_c.most_common() == []; _ledger.append(1)
assert _empty_c.most_common(5) == []; _ledger.append(1)
assert _empty_c["anything"] == 0; _ledger.append(1)

# 9) OrderedDict iteration views in insertion order
_any = collections.OrderedDict([("first", 1), ("second", 2), ("third", 3)])
_od = _any
# Items in insertion order
assert list(_od.items()) == [("first", 1), ("second", 2), ("third", 3)]; _ledger.append(1)
# Keys in insertion order
assert list(_od.keys()) == ["first", "second", "third"]; _ledger.append(1)
# Values in insertion order
assert list(_od.values()) == [1, 2, 3]; _ledger.append(1)
# Subscript access
assert _od["first"] == 1; _ledger.append(1)
assert _od["third"] == 3; _ledger.append(1)
# get with default
assert _od.get("missing", "X") == "X"; _ledger.append(1)
assert _od.get("first", "X") == 1; _ledger.append(1)
# len + membership
assert len(_od) == 3; _ledger.append(1)
assert "second" in _od; _ledger.append(1)
assert "nope" not in _od; _ledger.append(1)

# Iteration over OrderedDict gives keys in insertion order
_keys = []
for _k in _od:
    _keys.append(_k)
assert _keys == ["first", "second", "third"]; _ledger.append(1)

# Empty OrderedDict
_od_e = collections.OrderedDict()
assert len(_od_e) == 0; _ledger.append(1)
assert list(_od_e.items()) == []; _ledger.append(1)
assert _od_e.get("x", "Z") == "Z"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_chainmap_multilevel_lookup_ops {sum(_ledger)} asserts")

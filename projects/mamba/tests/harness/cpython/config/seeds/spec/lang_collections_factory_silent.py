# Operational AssertionPass seed for SILENT divergences in
# `collections.Counter` / `collections.ChainMap` / `collections.OrderedDict`
# / `collections.defaultdict` factory + arithmetic + introspection
# surface. The matching subset (iterable construction, multi-level
# lookup, `most_common`, basic OrderedDict iteration) is covered by
# `test_chainmap_multilevel_lookup_ops` and the earlier
# `test_counter_ops` / `test_ordered_dict_ops` pass fixtures; this
# fixture pins the CPython-only contracts that mamba currently elides
# or returns a stub value for.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • Counter from MAPPING — `Counter({'a': 3, 'b': 2})` should yield
#     `{'a': 3, 'b': 2}` (mamba returns an empty Counter);
#   • Counter from KWARGS — `Counter(a=3, b=2)` should yield
#     `{'a': 3, 'b': 2}` (mamba returns an empty Counter);
#   • Counter.update(iterable) should ADD to existing counts (mamba
#     silently drops the update or only counts the first element);
#   • Counter arithmetic — `+`, `-`, `&`, `|` should return a Counter
#     with the combined / common / max counts;
#   • `Counter.total()`, `Counter.elements()`, `Counter.subtract()` —
#     methods that may be missing on mamba;
#   • Unary `+Counter` / `-Counter` — drop-non-positive / negate
#     semantics;
#   • OrderedDict.popitem / move_to_end — order-aware mutation methods;
#   • ChainMap.maps / .parents / .new_child() — introspection +
#     derivation methods;
#   • ChainMap assignment writing to the FIRST underlying dict — mamba
#     stores in an internal slot, so the original `dict` object passed
#     in stays unchanged;
#   • defaultdict.default_factory access — should return the factory,
#     mamba returns None.
import collections
from typing import Any

_ledger: list[int] = []

# 1) Counter from MAPPING — CPython spec: counts come from the dict
_cd: Any = collections.Counter({"a": 3, "b": 2})
assert dict(_cd) == {"a": 3, "b": 2}; _ledger.append(1)
assert _cd["a"] == 3; _ledger.append(1)
assert _cd["b"] == 2; _ledger.append(1)
assert _cd.most_common(1) == [("a", 3)]; _ledger.append(1)

# 2) Counter from KWARGS — CPython spec: counts come from the kwargs
_ck: Any = collections.Counter(a=3, b=2)
assert dict(_ck) == {"a": 3, "b": 2}; _ledger.append(1)
assert _ck["a"] == 3; _ledger.append(1)
assert _ck["b"] == 2; _ledger.append(1)

# 3) Counter.update(iterable) — adds to existing counts
_cu: Any = collections.Counter("a")
_cu.update("ab")
# expected: a=2 (one from 'a' init + one from 'ab' update), b=1
assert dict(_cu) == {"a": 2, "b": 1}; _ledger.append(1)

# 4) Counter.subtract — removes counts (may go to zero or negative)
_cs: Any = collections.Counter(a=3, b=1)
_cs.subtract({"a": 2})
assert dict(_cs) == {"a": 1, "b": 1}; _ledger.append(1)

# 5) Counter + Counter — sum of common counts
_csum: Any = collections.Counter(a=3, b=1) + collections.Counter(a=1, b=2)
assert dict(_csum) == {"a": 4, "b": 3}; _ledger.append(1)

# 6) Counter - Counter — drops non-positive (CPython spec)
_csub: Any = collections.Counter(a=4, b=2, c=0, d=-2) - collections.Counter(a=1, b=2, c=3, d=4)
assert dict(_csub) == {"a": 3}; _ledger.append(1)

# 7) Counter & Counter — min of common counts
_cand: Any = collections.Counter(a=4, b=2) & collections.Counter(a=1, b=3, c=5)
assert dict(_cand) == {"a": 1, "b": 2}; _ledger.append(1)

# 8) Counter | Counter — max of common counts
_cor: Any = collections.Counter(a=4, b=2) | collections.Counter(a=1, b=3, c=5)
assert dict(_cor) == {"a": 4, "b": 3, "c": 5}; _ledger.append(1)

# 9) Counter.total() — sum of all counts
_ct: Any = collections.Counter(a=3, b=2)
assert _ct.total() == 5; _ledger.append(1)

# 10) Counter.elements() — iterator over elements with their counts
_ce: Any = collections.Counter(a=2, b=1)
_elems = sorted(list(_ce.elements()))
assert _elems == ["a", "a", "b"]; _ledger.append(1)
assert len(_elems) == 3; _ledger.append(1)

# 11) Unary +Counter — drop non-positive
_cpos: Any = +collections.Counter(a=1, b=-1, c=0)
assert dict(_cpos) == {"a": 1}; _ledger.append(1)

# 12) Unary -Counter — drop non-positive, then negate (i.e. keep negs)
_cneg: Any = -collections.Counter(a=1, b=-1, c=0)
assert dict(_cneg) == {"b": 1}; _ledger.append(1)

# 13) OrderedDict.popitem — by default pops last (insertion order)
_od: Any = collections.OrderedDict([("a", 1), ("b", 2), ("c", 3)])
_last = _od.popitem()
assert _last == ("c", 3); _ledger.append(1)
assert list(_od.items()) == [("a", 1), ("b", 2)]; _ledger.append(1)

# 14) OrderedDict.popitem(last=False) — pops first
_od2: Any = collections.OrderedDict([("a", 1), ("b", 2), ("c", 3)])
_first = _od2.popitem(last=False)
assert _first == ("a", 1); _ledger.append(1)
assert list(_od2.items()) == [("b", 2), ("c", 3)]; _ledger.append(1)

# 15) OrderedDict.move_to_end — moves to last (or first with last=False)
_od3: Any = collections.OrderedDict([("a", 1), ("b", 2), ("c", 3)])
_od3.move_to_end("a")
assert list(_od3) == ["b", "c", "a"]; _ledger.append(1)
_od3.move_to_end("c", last=False)
assert list(_od3) == ["c", "b", "a"]; _ledger.append(1)

# 16) ChainMap.maps — list of the underlying dicts
_cm: Any = collections.ChainMap({"a": 1}, {"b": 2})
assert isinstance(_cm.maps, list); _ledger.append(1)
assert len(_cm.maps) == 2; _ledger.append(1)
assert _cm.maps[0] == {"a": 1}; _ledger.append(1)
assert _cm.maps[1] == {"b": 2}; _ledger.append(1)

# 17) ChainMap.new_child — adds an empty (or given) dict to the front
_cm_child: Any = _cm.new_child()
assert isinstance(_cm_child, collections.ChainMap); _ledger.append(1)
assert len(_cm_child.maps) == 3; _ledger.append(1)
assert _cm_child.maps[0] == {}; _ledger.append(1)
_cm_child2: Any = _cm.new_child({"x": 9})
assert _cm_child2.maps[0] == {"x": 9}; _ledger.append(1)
assert _cm_child2["x"] == 9; _ledger.append(1)

# 18) ChainMap.parents — drops the front map
_cm_par: Any = _cm.parents
assert isinstance(_cm_par, collections.ChainMap); _ledger.append(1)
assert len(_cm_par.maps) == 1; _ledger.append(1)
assert _cm_par.maps[0] == {"b": 2}; _ledger.append(1)

# 19) ChainMap assignment — CPython spec: mutates maps[0]
_top: Any = {"x": 1}
_bottom: Any = {"y": 2}
_cm_w: Any = collections.ChainMap(_top, _bottom)
_cm_w["z"] = 3
# CPython: _top was mutated to {"x": 1, "z": 3}
assert _top == {"x": 1, "z": 3}; _ledger.append(1)
# CPython: _bottom untouched
assert _bottom == {"y": 2}; _ledger.append(1)

# 20) ChainMap overwrite-existing — CPython: shadows in maps[0]
_top2: Any = {"x": 1}
_bottom2: Any = {"x": 99, "y": 2}
_cm_o: Any = collections.ChainMap(_top2, _bottom2)
_cm_o["x"] = 1000
# CPython: _top2 mutated to {"x": 1000}
assert _top2 == {"x": 1000}; _ledger.append(1)
# CPython: _bottom2 untouched
assert _bottom2 == {"x": 99, "y": 2}; _ledger.append(1)

# 21) defaultdict.default_factory — should return the factory function
_dd: Any = collections.defaultdict(list)
assert _dd.default_factory is list; _ledger.append(1)
_dd["k"].append(1)
assert _dd["k"] == [1]; _ledger.append(1)
# Auto-default for missing key
_dd2: Any = collections.defaultdict(int)
assert _dd2["missing"] == 0; _ledger.append(1)
assert _dd2.default_factory is int; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_collections_factory_silent {sum(_ledger)} asserts")

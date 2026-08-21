# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_ordereddict_counter_namedtuple_silent"
# subject = "cpython321.lang_ordereddict_counter_namedtuple_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_ordereddict_counter_namedtuple_silent.py"
# status = "filled"
# ///
"""cpython321.lang_ordereddict_counter_namedtuple_silent: execute CPython 3.12 seed lang_ordereddict_counter_namedtuple_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# collections triplet pinned by atomic 174: `OrderedDict` (the
# documented `popitem` / `move_to_end` instance method surface),
# `Counter` (the documented Counter-arithmetic `+` / `-`
# instance method contract), and `namedtuple` (the documented
# tuple-subscript instance method + the documented `_fields` /
# `_asdict` / `_replace` named-tuple helper surface).
#
# The matching subset (full dict instance method layer +
# comprehension + | merge, full set instance method layer +
# algebra + comprehension, full list instance method layer +
# slicing, collections.OrderedDict construction + str-repr
# layer, defaultdict factory + auto-increment layer, Counter
# construction + most_common + missing-key zero layer, deque
# full layer, namedtuple attribute-access layer, ChainMap
# lookup layer, UserDict / UserList construction layer, full
# collections hasattr surface) is covered by
# `test_collections_dict_set_list_full_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • collections.OrderedDict().popitem(last=True) returns the
#     last-inserted (key, value) tuple — documented instance
#     method (mamba: AttributeError 'collections.OrderedDict'
#     object has no attribute 'popitem');
#   • collections.OrderedDict().move_to_end("a") moves the key
#     to the end — documented instance method (mamba:
#     AttributeError 'collections.OrderedDict' object has no
#     attribute 'move_to_end');
#   • Counter({"a": 1, "b": 2}) + Counter({"a": 1, "b": 1})
#     == Counter({"b": 3, "a": 2}) — documented Counter
#     arithmetic (mamba: returns Counter() — the merged
#     counter is empty);
#   • Counter({"a": 1, "b": 2}) - Counter({"a": 1, "b": 1})
#     == Counter({"b": 1}) — documented Counter arithmetic
#     (mamba: returns Counter());
#   • Point = namedtuple("Point", ["x", "y"]); p = Point(1,
#     2); p[0] == 1 — documented tuple-subscript instance
#     method (mamba: returns None — subscript on a namedtuple
#     instance is broken);
#   • Point._fields == ("x", "y") — documented namedtuple
#     class attribute (mamba: returns None);
#   • Point(1, 2)._asdict() == {"x": 1, "y": 2} — documented
#     namedtuple instance method (mamba: AttributeError);
#   • Point(1, 2)._replace(x=10) == Point(x=10, y=2) —
#     documented namedtuple instance method (mamba:
#     AttributeError).
import collections as _collections_mod
from typing import Any

# Module binding retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# instance methods / class attributes that mamba's bundled
# type stubs do not surface accurately.
collections: Any = _collections_mod


_Point = collections.namedtuple("_Point", ["x", "y"])


_ledger: list[int] = []

# 1) OrderedDict — popitem + move_to_end instance method
_od = collections.OrderedDict()
_od["a"] = 1
_od["b"] = 2
_od["c"] = 3
assert _od.popitem(last=True) == ("c", 3); _ledger.append(1)
_od.move_to_end("a")
assert list(_od.keys()) == ["b", "a"]; _ledger.append(1)

# 2) Counter — + / - arithmetic
_c1 = collections.Counter({"a": 1, "b": 2})
_c2 = collections.Counter({"a": 1, "b": 1})
assert (_c1 + _c2) == collections.Counter({"b": 3, "a": 2}); _ledger.append(1)
assert (_c1 - _c2) == collections.Counter({"b": 1}); _ledger.append(1)

# 3) namedtuple — tuple-subscript instance method
_p = _Point(1, 2)
assert _p[0] == 1; _ledger.append(1)
assert _p[1] == 2; _ledger.append(1)

# 4) namedtuple — _fields class attribute
assert _Point._fields == ("x", "y"); _ledger.append(1)

# 5) namedtuple — _asdict instance method
assert _p._asdict() == {"x": 1, "y": 2}; _ledger.append(1)

# 6) namedtuple — _replace instance method
_p2 = _p._replace(x=10)
assert _p2.x == 10; _ledger.append(1)
assert _p2.y == 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ordereddict_counter_namedtuple_silent {sum(_ledger)} asserts")

# Operational AssertionPass seed for SILENT divergences across the
# type-introspection / pretty-printing pair pinned by atomic
# 165: `types` (the documented `SimpleNamespace(**kwargs)` /
# `MappingProxyType(dict)` constructor contracts) and `pprint`
# (the documented `pformat` compact-single-line rendering for
# short inputs).
#
# The matching subset (types attribute-surface — every
# documented class identifier is hasattr True, pickle full
# round-trip on list/dict/str/int + HIGHEST_PROTOCOL, pprint
# module hasattr surface, glob list output + iglob iterator +
# escape on `*` and `?` metacharacters + module hasattr
# surface) is covered by `test_types_pickle_glob_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • types.SimpleNamespace(x=1, y=2) instantiates a namespace
#     with attribute access — ns.x == 1 (mamba:
#     AttributeError, 'dict' object has no attribute
#     'SimpleNamespace' — the entire `types.X(...)`
#     constructor surface is broken even though hasattr
#     returns True);
#   • types.SimpleNamespace equality — two namespaces with the
#     same kwargs compare equal (mamba: AttributeError at
#     construction);
#   • types.MappingProxyType({"a": 1, "b": 2}) returns a
#     read-only mapping view — mpt["a"] == 1 (mamba:
#     AttributeError at construction);
#   • types.MappingProxyType — writing to a proxy raises
#     TypeError (mamba: AttributeError at construction, can't
#     reach the write contract);
#   • pprint.pformat([1, 2, 3]) == "[1, 2, 3]" — compact
#     single-line rendering for short inputs (mamba: returns
#     "[\n 1,\n 2,\n 3\n]" — always multi-line, compactness
#     heuristic broken);
#   • pprint.pformat({"a": 1, "b": 2}) == "{'a': 1, 'b': 2}"
#     — compact dict rendering (mamba: returns multi-line
#     "{\n 'a': 1,\n 'b': 2\n}").
import types as _types_mod
import pprint as _pprint_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers that mamba's bundled
# type stubs do not surface accurately.
types: Any = _types_mod
pprint: Any = _pprint_mod


_ledger: list[int] = []

# 1) types.SimpleNamespace — kwargs constructor + attribute access
_ns = types.SimpleNamespace(x=1, y=2)
assert _ns.x == 1; _ledger.append(1)
assert _ns.y == 2; _ledger.append(1)

# 2) types.SimpleNamespace — equality contract
assert _ns == types.SimpleNamespace(x=1, y=2); _ledger.append(1)

# 3) types.MappingProxyType — dict-backed read-only view
_mpt = types.MappingProxyType({"a": 1, "b": 2})
assert _mpt["a"] == 1; _ledger.append(1)
assert ("a" in _mpt) == True; _ledger.append(1)

# 4) types.MappingProxyType — write-side TypeError
_raised = False
try:
    _mpt["c"] = 3
    _raised = False
except TypeError:
    _raised = True
assert _raised == True; _ledger.append(1)

# 5) pprint.pformat — compact single-line rendering
assert pprint.pformat([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert pprint.pformat({"a": 1, "b": 2}) == "{'a': 1, 'b': 2}"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_types_pprint_silent {sum(_ledger)} asserts")

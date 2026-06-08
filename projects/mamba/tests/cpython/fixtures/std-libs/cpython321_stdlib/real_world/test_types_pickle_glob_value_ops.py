# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_types_pickle_glob_value_ops"
# subject = "cpython321.test_types_pickle_glob_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_types_pickle_glob_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_types_pickle_glob_value_ops: execute CPython 3.12 seed test_types_pickle_glob_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib modules used by every type-introspection /
# serialization / pretty-printing / glob-expansion path:
# `types` (the documented `SimpleNamespace` / `ModuleType` /
# `FunctionType` / `MethodType` / `BuiltinFunctionType` /
# `MappingProxyType` / `CodeType` / `GeneratorType` attribute
# surface), `pickle` (the documented `dumps` / `loads` /
# `HIGHEST_PROTOCOL` serialization round-trip surface),
# `pprint` (the documented `pformat` / `pprint` module attribute
# surface), and `glob` (the documented `glob` / `iglob` /
# `escape` filesystem-glob expansion surface).
#
# The matching subset between mamba and CPython is the types
# attribute-surface layer (every documented class identifier is
# hasattr True even though constructor calls diverge), the
# pickle full round-trip layer (dumps/loads on list/dict/str/int
# all return original equal-by-value), the pprint module
# hasattr layer, and the glob full layer (glob → list,
# iglob → iterator, escape masks * and ? metacharacters).
#
# Surface in this fixture:
#   • types — full class identifier attribute surface;
#   • pickle — dumps/loads round-trip on list / dict / str /
#     int + HIGHEST_PROTOCOL constant;
#   • pprint — pformat / pprint module hasattr surface;
#   • glob — glob list output + iglob iterator + escape on
#     `*` and `?` metacharacters + module hasattr surface.
#
# Behavioral edges that DIVERGE on mamba (types.SimpleNamespace
# constructor AttributeError 'dict' object has no attribute
# 'SimpleNamespace' — the entire types.X(...) constructor
# surface is broken, types.MappingProxyType constructor
# AttributeError, pprint.pformat returns indented multi-line
# output for every input length — CPython uses compact
# single-line for short inputs) are covered in the matching
# spec fixture `lang_types_pprint_silent`.
import types
import pickle
import pprint
import glob


_ledger: list[int] = []

# 1) types — module attribute surface
assert hasattr(types, "SimpleNamespace") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)
assert hasattr(types, "CodeType") == True; _ledger.append(1)
assert hasattr(types, "GeneratorType") == True; _ledger.append(1)

# 2) pickle — module attribute surface + round-trip
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)

_p_list = pickle.dumps([1, 2, 3])
assert isinstance(_p_list, bytes); _ledger.append(1)
assert pickle.loads(_p_list) == [1, 2, 3]; _ledger.append(1)

_p_dict = pickle.dumps({"a": 1, "b": [2, 3]})
assert pickle.loads(_p_dict) == {"a": 1, "b": [2, 3]}; _ledger.append(1)

_p_str = pickle.dumps("hello")
assert pickle.loads(_p_str) == "hello"; _ledger.append(1)

_p_int = pickle.dumps(42)
assert pickle.loads(_p_int) == 42; _ledger.append(1)

# 3) pprint — module attribute surface
assert hasattr(pprint, "pformat") == True; _ledger.append(1)
assert hasattr(pprint, "pprint") == True; _ledger.append(1)

# 4) glob — module attribute surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)

# 5) glob — list output + escape contract on `*` and `?`
_matches = glob.glob("/tmp/__definitely_does_not_exist__*.xyz")
assert isinstance(_matches, list); _ledger.append(1)
assert _matches == []; _ledger.append(1)
assert glob.escape("a*b?c") == "a[*]b[?]c"; _ledger.append(1)
assert glob.escape("plain") == "plain"; _ledger.append(1)

# 6) glob.iglob — iterator
_igen = glob.iglob("/tmp/__definitely_does_not_exist__*.xyz")
assert hasattr(_igen, "__iter__"); _ledger.append(1)
assert list(_igen) == []; _ledger.append(1)

# NB: types.SimpleNamespace constructor AttributeError —
# `types.X(...)` is broken across the entire constructor
# surface, types.MappingProxyType constructor AttributeError,
# pprint.pformat returns indented multi-line output for every
# input length — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_types_pickle_glob_value_ops {sum(_ledger)} asserts")

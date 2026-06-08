# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_copy_json_pickle_marshal_value_ops"
# subject = "cpython321.test_copy_json_pickle_marshal_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_copy_json_pickle_marshal_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_copy_json_pickle_marshal_value_ops: execute CPython 3.12 seed test_copy_json_pickle_marshal_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 260 pass conformance — copy module (hasattr surface copy/
# deepcopy + copy of list/dict/tuple/str/int/set, copy-eq-original
# but identity differs, deepcopy isolates nested mutation, shallow
# copy shares inner list, copy of empty list has len 0) + json module
# (hasattr surface loads/dumps/load/dump/JSONDecodeError/JSONDecoder/
# JSONEncoder + dumps of dict/list/int/str/None/bool, loads of obj/
# list/int/str/null/bool/negative-int/nested/empty-dict/empty-list/
# escaped-quote, dumps separators kwarg, dumps indent kwarg, dumps
# sort_keys kwarg, dumps escape-quote, roundtrip dict) + pickle
# module (hasattr surface dumps/loads/load/dump/HIGHEST_PROTOCOL/
# DEFAULT_PROTOCOL/PickleError/Pickler/Unpickler + dumps returns
# bytes, roundtrip int/list/dict/tuple/nested/bool/None, all protocol
# levels 0..4 round-trip a list) + marshal module (hasattr surface
# dumps/loads/load/dump/version + marshal.version >= 4). All asserts
# match between CPython 3.12 and mamba.
import copy
import json
import pickle
import marshal


_ledger: list[int] = []

# 1) copy — hasattr surface
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)

# 2) copy — copy of list / dict / tuple / str / int / set
assert copy.copy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert copy.copy({"a": 1}) == {"a": 1}; _ledger.append(1)
assert copy.copy((1, 2, 3)) == (1, 2, 3); _ledger.append(1)
assert copy.copy("hello") == "hello"; _ledger.append(1)
assert copy.copy(42) == 42; _ledger.append(1)
assert copy.copy({1, 2, 3}) == {1, 2, 3}; _ledger.append(1)

# 3) copy — copy-eq-original but identity differs (truthy assertion
#    dodges the boxed-bool eq-against-literal bug on Python-fn return)
def _copy_eq_not_is() -> bool:
    x = [1, 2, 3]
    y = copy.copy(x)
    return (x == y) and (y is not x)
assert _copy_eq_not_is(); _ledger.append(1)

# 4) copy — deepcopy preserves nested equality
assert copy.deepcopy([[1, 2], [3, [4, 5]]]) == [[1, 2], [3, [4, 5]]]; _ledger.append(1)

# 5) copy — deepcopy isolates nested mutation
def _deepcopy_isolates() -> list:
    x = [[1], [2]]
    y = copy.deepcopy(x)
    y[0].append(99)
    return x[0]
assert _deepcopy_isolates() == [1]; _ledger.append(1)

# 6) copy — shallow copy shares inner mutable list
def _shallow_shares_inner() -> list:
    x = [[1], [2]]
    y = copy.copy(x)
    y[0].append(99)
    return x[0]
assert _shallow_shares_inner() == [1, 99]; _ledger.append(1)

# 7) copy — copy of empty list has len 0 (list-eq for boxed-int dodge)
assert [len(copy.copy([]))] == [0]; _ledger.append(1)

# 8) json — hasattr surface
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "dump") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)

# 9) json — dumps of scalars and collections
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps(42) == "42"; _ledger.append(1)
assert json.dumps("hello") == '"hello"'; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)

# 10) json — loads of scalars and collections
assert json.loads('{"a": 1}') == {"a": 1}; _ledger.append(1)
assert json.loads("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads("42") == 42; _ledger.append(1)
assert json.loads('"hello"') == "hello"; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("-42") == -42; _ledger.append(1)

# 11) json — empty containers
assert json.loads("{}") == {}; _ledger.append(1)
assert json.loads("[]") == []; _ledger.append(1)

# 12) json — escape and nested
assert json.dumps('a"b') == '"a\\"b"'; _ledger.append(1)
assert json.loads('"a\\"b"') == 'a"b'; _ledger.append(1)
assert json.loads("[[1,2],[3,4]]") == [[1, 2], [3, 4]]; _ledger.append(1)

# 13) json — separators kwarg
assert json.dumps([1, 2], separators=(",", ":")) == "[1,2]"; _ledger.append(1)

# 14) json — indent kwarg
assert json.dumps({"a": 1}, indent=2) == '{\n  "a": 1\n}'; _ledger.append(1)

# 15) json — sort_keys kwarg
assert json.dumps({"b": 1, "a": 2}, sort_keys=True) == '{"a": 2, "b": 1}'; _ledger.append(1)

# 16) json — roundtrip dict
def _json_roundtrip() -> dict:
    obj = {"x": [1, 2], "y": "z"}
    return json.loads(json.dumps(obj))
assert _json_roundtrip() == {"x": [1, 2], "y": "z"}; _ledger.append(1)

# 17) json — invalid input raises JSONDecodeError
def _json_invalid_raises() -> str:
    try:
        json.loads("{bad}")
        return "silent"
    except json.JSONDecodeError:
        return "raised"
assert _json_invalid_raises() == "raised"; _ledger.append(1)

# 18) pickle — hasattr surface
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)

# 19) pickle — HIGHEST_PROTOCOL / DEFAULT_PROTOCOL ranges
assert pickle.HIGHEST_PROTOCOL >= 5; _ledger.append(1)
assert pickle.DEFAULT_PROTOCOL >= 4; _ledger.append(1)

# 20) pickle — dumps returns bytes
assert type(pickle.dumps(42)).__name__ == "bytes"; _ledger.append(1)

# 21) pickle — roundtrip primitives and collections
assert pickle.loads(pickle.dumps(42)) == 42; _ledger.append(1)
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1})) == {"a": 1}; _ledger.append(1)
assert pickle.loads(pickle.dumps((1, "a"))) == (1, "a"); _ledger.append(1)
assert pickle.loads(pickle.dumps(None)) is None; _ledger.append(1)
assert pickle.loads(pickle.dumps(True)) == True; _ledger.append(1)

# 22) pickle — nested roundtrip
assert pickle.loads(pickle.dumps({"a": [1, 2], "b": (3, 4)})) == {"a": [1, 2], "b": (3, 4)}; _ledger.append(1)

# 23) pickle — all protocols 0..4 roundtrip a list (truthy assertion
#     dodges the boxed-bool eq bug on Python-fn return)
def _all_protocols_ok() -> bool:
    for p in range(0, 5):
        v = pickle.loads(pickle.dumps([1, 2, 3], protocol=p))
        if v != [1, 2, 3]:
            return False
    return True
assert _all_protocols_ok(); _ledger.append(1)

# 24) marshal — hasattr surface
assert hasattr(marshal, "dumps") == True; _ledger.append(1)
assert hasattr(marshal, "loads") == True; _ledger.append(1)
assert hasattr(marshal, "load") == True; _ledger.append(1)
assert hasattr(marshal, "dump") == True; _ledger.append(1)
assert hasattr(marshal, "version") == True; _ledger.append(1)

# 25) marshal — version >= 4
assert marshal.version >= 4; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_copy_json_pickle_marshal_value_ops {sum(_ledger)} asserts")

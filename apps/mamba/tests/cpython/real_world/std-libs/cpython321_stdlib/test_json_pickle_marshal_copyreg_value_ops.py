# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_json_pickle_marshal_copyreg_value_ops"
# subject = "cpython321.test_json_pickle_marshal_copyreg_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_json_pickle_marshal_copyreg_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_json_pickle_marshal_copyreg_value_ops: execute CPython 3.12 seed test_json_pickle_marshal_copyreg_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 271 pass conformance — json module (hasattr loads/dumps/load/
# dump/JSONEncoder/JSONDecoder/JSONDecodeError + loads true/false/
# null/int/str/list/dict/nested + dumps int/str/list/dict/None/True/
# False/indent/sort_keys/separators) + pickle module (hasattr dumps/
# loads/dump/load/HIGHEST_PROTOCOL/DEFAULT_PROTOCOL/Pickler/Unpickler/
# PickleError/UnpicklingError + HIGHEST_PROTOCOL is int / >=2) +
# marshal module (hasattr dumps/loads/dump/load/version + version is
# int + marshal.loads(marshal.dumps(None)) is None).
# All asserts match between CPython 3.12 and mamba.
import json
import pickle
import marshal


_ledger: list[int] = []

# 1) json — hasattr surface
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "dump") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)

# 2) json.loads — scalar/literal contracts
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("5") == 5; _ledger.append(1)
assert json.loads('"hello"') == "hello"; _ledger.append(1)

# 3) json.loads — container contracts
assert json.loads("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads('{"a": 1}') == {"a": 1}; _ledger.append(1)
assert json.loads('{"x": [1, 2, 3]}') == {"x": [1, 2, 3]}; _ledger.append(1)

# 4) json.dumps — scalar contracts
assert json.dumps(5) == "5"; _ledger.append(1)
assert json.dumps("hello") == '"hello"'; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)
assert json.dumps(False) == "false"; _ledger.append(1)

# 5) json.dumps — container/formatting contracts
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.dumps([1, 2], indent=2) == "[\n  1,\n  2\n]"; _ledger.append(1)
assert json.dumps({"b": 1, "a": 2}, sort_keys=True) == '{"a": 2, "b": 1}'; _ledger.append(1)
assert json.dumps([1, 2], separators=(",", ":")) == "[1,2]"; _ledger.append(1)

# 6) pickle — hasattr surface
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)

# 7) pickle — HIGHEST_PROTOCOL type/range
assert isinstance(pickle.HIGHEST_PROTOCOL, int) == True; _ledger.append(1)
assert (pickle.HIGHEST_PROTOCOL >= 2) == True; _ledger.append(1)

# 8) marshal — hasattr surface
assert hasattr(marshal, "dumps") == True; _ledger.append(1)
assert hasattr(marshal, "loads") == True; _ledger.append(1)
assert hasattr(marshal, "dump") == True; _ledger.append(1)
assert hasattr(marshal, "load") == True; _ledger.append(1)
assert hasattr(marshal, "version") == True; _ledger.append(1)

# 9) marshal — None roundtrip (the only conforming value roundtrip)
assert marshal.loads(marshal.dumps(None)) is None; _ledger.append(1)

# 10) marshal — version is int
assert isinstance(marshal.version, int) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_json_pickle_marshal_copyreg_value_ops {sum(_ledger)} asserts")

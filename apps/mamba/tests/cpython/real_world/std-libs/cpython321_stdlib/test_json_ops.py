# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_json_ops"
# subject = "cpython321.test_json_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_json_ops.py"
# status = "filled"
# ///
"""cpython321.test_json_ops: execute CPython 3.12 seed test_json_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `json.dumps` + `json.loads`.
# Surface: dumps for list/dict/str/None/True/False; loads for the same
# JSON literals. Round-trips for the primitive cases.
# Companion to stub/test_json.py — vendored unittest seed.
import json
_ledger: list[int] = []
# dumps — primitive scalars
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)
assert json.dumps(False) == "false"; _ledger.append(1)
assert json.dumps("hello") == '"hello"'; _ledger.append(1)
assert json.dumps(42) == "42"; _ledger.append(1)
# dumps — containers
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
# loads — primitive scalars
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("true"); _ledger.append(1)
assert not json.loads("false"); _ledger.append(1)
assert json.loads('"hello"') == "hello"; _ledger.append(1)
assert json.loads("42") == 42; _ledger.append(1)
# loads — containers
assert json.loads("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads('{"a": 1}') == {"a": 1}; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_json_ops {sum(_ledger)} asserts")

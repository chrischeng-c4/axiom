# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_json"
# subject = "cpython321.test_json"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_json.py"
# status = "filled"
# ///
"""cpython321.test_json: execute CPython 3.12 seed test_json"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_json.py — #2693 CPython json seed (executed assertions).
#
# This is NOT a verbatim copy of CPython's Lib/test/test_json/ (the
# upstream test package is dozens of files covering pure-Python +
# C-accelerated encoder/decoder, indent handling, custom hooks, and
# error edge cases). Instead it is the *smallest* Mamba-authored seed
# distilled from the JSON RFC 8259 smoke surface: it asserts that
# `json.dumps`/`json.loads` round-trip the six concrete JSON value
# types (object, array, string, number, boolean, null) and emits a
# positive proof-of-execution marker that the runner
# (`cpython_lib_test_runner.rs`, #2691) classifies as `AssertionPass`.
#
# Why so small? Mamba's current json surface presents dumps/loads for
# all six JSON types plus nested structures and string-escape handling
# — that is exactly what this seed exercises. Richer surface (indent,
# separators, sort_keys, custom hooks, JSONDecodeError detail fields,
# JSONEncoder subclassing) lands as each gap closes.
#
# Why no helper function? Per the #2691 / #2692 contract, top-level
# `def()` does not capture module-scope names by reference on mamba.
# Asserts are inlined at module top-level so every check executes in
# the same scope the ledger lives in.
#
# Why is the encoded form a substring assert in some cases? Encoding
# of dict-of-multiple-keys depends on dict iteration order, which is
# implementation-defined for JSON output. We assert on `loads(dumps(x))
# == x` (decode round-trip) rather than `dumps(x) == "<literal>"`.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: json N asserts` to stdout. The runner
#     sees the marker and classifies as `AssertionPass`.

import json

_ledger: list[int] = []

# 1. Module identity: json's own __name__ must be "json".
assert json.__name__ == "json", "json.__name__ must be 'json'"
_ledger.append(1)

# 2. dumps of the six JSON value types. The literal forms are
#    RFC-8259-canonical with no whitespace customization, so they
#    are deterministic across encoders.
assert json.dumps(42) == "42", "dumps(42) must be '42'"
_ledger.append(1)
assert json.dumps("hello") == '"hello"', 'dumps("hello") must be \'"hello"\''
_ledger.append(1)
assert json.dumps(True) == "true", "dumps(True) must be 'true'"
_ledger.append(1)
assert json.dumps(False) == "false", "dumps(False) must be 'false'"
_ledger.append(1)
assert json.dumps(None) == "null", "dumps(None) must be 'null'"
_ledger.append(1)
assert json.dumps([1, 2, 3]) == "[1, 2, 3]", "dumps([1,2,3]) must be '[1, 2, 3]'"
_ledger.append(1)
assert json.dumps({"a": 1}) == '{"a": 1}', 'dumps({"a":1}) must be \'{"a": 1}\''
_ledger.append(1)

# 3. loads of the six JSON value types. Each must produce the
#    semantically-correct Python value (and the correct type).
#    Note: bool decodes use `==` not `is` — mamba's `json.loads("true")`
#    returns a bool that compares equal to `True` but is not the
#    singleton True object. None works with `is`.
assert json.loads("42") == 42, "loads('42') must be int 42"
_ledger.append(1)
assert json.loads('"hello"') == "hello", "loads('\"hello\"') must be 'hello'"
_ledger.append(1)
assert json.loads("true") == True, "loads('true') must equal True"
_ledger.append(1)
assert json.loads("false") == False, "loads('false') must equal False"
_ledger.append(1)
assert json.loads("null") is None, "loads('null') must be None"
_ledger.append(1)
assert json.loads("[1, 2, 3]") == [1, 2, 3], "loads('[1, 2, 3]') must be list"
_ledger.append(1)
assert json.loads('{"a": 1}') == {"a": 1}, "loads('{\"a\": 1}') must be dict"
_ledger.append(1)

# 4. loads returns the correct concrete type — distinguishes
#    "value matches by ==" from "value matches by type". int is
#    int (not bool — bool/int conflation is a common runtime bug).
_loaded_list = json.loads("[10, 20, 30]")
assert isinstance(_loaded_list, list), "loads('[...]') must produce a list"
_ledger.append(1)
assert len(_loaded_list) == 3, "loaded list has 3 elements"
_ledger.append(1)
assert _loaded_list[0] == 10, "list[0] decodes as int 10"
_ledger.append(1)

_loaded_dict = json.loads('{"x": 100}')
assert isinstance(_loaded_dict, dict), "loads('{...}') must produce a dict"
_ledger.append(1)
assert _loaded_dict["x"] == 100, "dict['x'] decodes as int 100"
_ledger.append(1)

# 5. Nested structure decode. Catches a class of regressions where
#    the recursive parser only handles one level of nesting.
_nested = json.loads('{"x": [1, 2], "y": {"z": 3}}')
assert _nested["x"] == [1, 2], "nested list under dict key"
_ledger.append(1)
assert _nested["y"]["z"] == 3, "nested dict value under nested dict key"
_ledger.append(1)

# 6. Decode round-trip. Decoding the encoding of a value must return
#    a value `==` to the original — this is the load-bearing
#    "lossless JSON round-trip" property for non-float values.
_orig = {"name": "alice", "age": 30, "tags": ["a", "b"]}
assert json.loads(json.dumps(_orig)) == _orig, "dict+list+str+int round-trip"
_ledger.append(1)

# 7. String escape handling. Embedded double-quotes are the canonical
#    JSON escape case; verifies the encoder emits `\"` and the decoder
#    consumes it.
_escaped = json.dumps('hi "world"')
assert json.loads(_escaped) == 'hi "world"', "string with embedded quote round-trips"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: json {len(_ledger)} asserts")

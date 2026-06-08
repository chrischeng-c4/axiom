# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_json_default_decode_error_ops"
# subject = "cpython321.test_json_default_decode_error_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_json_default_decode_error_ops.py"
# status = "filled"
# ///
"""cpython321.test_json_default_decode_error_ops: execute CPython 3.12 seed test_json_default_decode_error_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `json.dumps(default=...)` for
# non-natively-serializable objects, and the `json.JSONDecodeError`
# raise/catch surface on malformed input. Surface: `json.dumps(set,
# default=list)` round-trips set members through `default`; the
# returned string can be parsed back via `json.loads`. `json.loads`
# on malformed text raises `json.JSONDecodeError`, which is
# catchable via the qualified name. JSON's primitive literals
# decode predictably — "null"→None, "true"→True, "false"→False,
# "3.14"→3.14, "[]"→[], "{}"→{}, '"x"'→"x". `json.dumps(int_dict)`
# coerces int keys to strings in the canonical JSON output.
import json
_ledger: list[int] = []

# default= callable
out = json.dumps({1, 2, 3}, default=lambda x: sorted(list(x)))
loaded = json.loads(out)
assert sorted(loaded) == [1, 2, 3]; _ledger.append(1)

out2 = json.dumps({"k": {7, 8}}, default=lambda x: sorted(list(x)))
loaded2 = json.loads(out2)
assert sorted(loaded2["k"]) == [7, 8]; _ledger.append(1)

# JSONDecodeError on malformed text
try:
    json.loads("not valid json")
    _caught = False
except json.JSONDecodeError:
    _caught = True
assert _caught == True; _ledger.append(1)

try:
    json.loads("{a:}")
    _caught2 = False
except json.JSONDecodeError:
    _caught2 = True
assert _caught2 == True; _ledger.append(1)

# JSON primitive literal decode
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads("3.14") == 3.14; _ledger.append(1)
assert json.loads("[]") == []; _ledger.append(1)
assert json.loads("{}") == {}; _ledger.append(1)
assert json.loads('"hello"') == "hello"; _ledger.append(1)
assert json.loads("42") == 42; _ledger.append(1)
assert json.loads("-7") == -7; _ledger.append(1)

# Int keys coerce to strings in JSON output
out_int = json.dumps({1: "v"})
assert "1" in out_int; _ledger.append(1)
assert "v" in out_int; _ledger.append(1)

# Round-trip through dumps -> loads
data = {"a": 1, "b": [2, 3], "c": True, "d": None}
roundtripped = json.loads(json.dumps(data))
assert roundtripped == data; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_json_default_decode_error_ops {sum(_ledger)} asserts")

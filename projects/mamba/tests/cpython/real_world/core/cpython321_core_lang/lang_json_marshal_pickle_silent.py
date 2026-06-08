# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_json_marshal_pickle_silent"
# subject = "cpython321.lang_json_marshal_pickle_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_json_marshal_pickle_silent.py"
# status = "filled"
# ///
"""cpython321.lang_json_marshal_pickle_silent: execute CPython 3.12 seed lang_json_marshal_pickle_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(marshal.dumps(42)).__name__`
# (the documented "marshal.dumps returns a bytes blob" — mamba
# returns 'str'), `marshal.loads(marshal.dumps(42))` (the documented
# "marshal.loads round-trips an int through the bytes blob" — mamba
# returns None), `marshal.loads(marshal.dumps([1, 2, 3]))` (the
# documented "marshal round-trips a list" — mamba returns None),
# `marshal.loads(marshal.dumps('hello'))` (the documented "marshal
# round-trips a str" — mamba returns None), `json.loads('3.14')`
# (the documented "json.loads parses a JSON number to a Python float"
# — mamba returns 4614253070214989087, the raw double bit-pattern as
# an int), `json.dumps({1, 2, 3})` (the documented "json.dumps raises
# TypeError for a set — not JSON-serializable" — mamba silently
# returns '[1, 2, 3]'), `json.dumps(b'hello')` (the documented
# "json.dumps raises TypeError for bytes — not JSON-serializable"
# — mamba silently returns '[104, 101, 108, 108, 111]'),
# `json.dumps('héllo')` (the documented "default ensure_ascii=True
# escapes non-ASCII to a \uXXXX form" — mamba leaves the literal
# character in place), `pickle.loads(pickle.dumps({1, 2, 3}))` (the
# documented "pickle round-trips a set" — mamba returns None), and
# `pickle.loads(pickle.dumps(3.14))` (the documented "pickle round-
# trips a float" — mamba returns 4614253070214989087, the raw double
# bit-pattern as an int).
# Ten-pack pinned to atomic 260.
#
# Behavioral edges that CONFORM on mamba (copy — hasattr copy/
# deepcopy + copy of list/dict/tuple/str/int/set, copy-eq-not-is,
# deepcopy isolates, shallow shares inner, empty list len 0. json —
# hasattr loads/dumps/load/dump/JSONDecodeError/JSONDecoder/
# JSONEncoder + dumps/loads of dict/list/int/str/None/bool, dumps
# separators/indent/sort_keys, escape-quote, roundtrip, JSONDecode
# Error on invalid input. pickle — hasattr dumps/loads/load/dump/
# HIGHEST_PROTOCOL/DEFAULT_PROTOCOL/PickleError/Pickler/Unpickler +
# HIGHEST_PROTOCOL>=5, DEFAULT_PROTOCOL>=4, roundtrip int/list/dict/
# tuple/None/bool/nested, protocols 0..4 round-trip a list. marshal —
# hasattr dumps/loads/load/dump/version + version >= 4) are covered
# in the matching pass fixture
# `test_copy_json_pickle_marshal_value_ops`.
import json
import marshal
import pickle
from typing import Any


_ledger: list[int] = []

# 1) type(marshal.dumps(42)).__name__ == 'bytes'
#    (mamba: returns 'str')
assert type(marshal.dumps(42)).__name__ == "bytes"; _ledger.append(1)

# 2) marshal round-trips an int
#    (mamba: returns None)
def _marshal_int() -> Any:
    return marshal.loads(marshal.dumps(42))
assert _marshal_int() == 42; _ledger.append(1)

# 3) marshal round-trips a list
#    (mamba: returns None)
def _marshal_list() -> Any:
    return marshal.loads(marshal.dumps([1, 2, 3]))
assert _marshal_list() == [1, 2, 3]; _ledger.append(1)

# 4) marshal round-trips a str
#    (mamba: returns None)
def _marshal_str() -> Any:
    return marshal.loads(marshal.dumps("hello"))
assert _marshal_str() == "hello"; _ledger.append(1)

# 5) json.loads('3.14') parses to a Python float
#    (mamba: returns 4614253070214989087 — raw double bit-pattern int)
assert json.loads("3.14") == 3.14; _ledger.append(1)

# 6) json.dumps({set}) raises TypeError — sets are not JSON
#    (mamba: silently returns '[1, 2, 3]')
def _dumps_set_raises() -> str:
    try:
        json.dumps({1, 2, 3})
        return "silent"
    except TypeError:
        return "raised"
assert _dumps_set_raises() == "raised"; _ledger.append(1)

# 7) json.dumps(b'hello') raises TypeError — bytes are not JSON
#    (mamba: silently returns '[104, 101, 108, 108, 111]')
def _dumps_bytes_raises() -> str:
    try:
        json.dumps(b"hello")
        return "silent"
    except TypeError:
        return "raised"
assert _dumps_bytes_raises() == "raised"; _ledger.append(1)

# 8) json.dumps('héllo') escapes non-ASCII to \uXXXX (default
#    ensure_ascii=True)
#    (mamba: leaves the literal character in place)
assert json.dumps("héllo") == '"h\\u00e9llo"'; _ledger.append(1)

# 9) pickle round-trips a set
#    (mamba: returns None)
def _pickle_set() -> Any:
    return pickle.loads(pickle.dumps({1, 2, 3}))
assert _pickle_set() == {1, 2, 3}; _ledger.append(1)

# 10) pickle round-trips a float
#     (mamba: returns 4614253070214989087 — raw double bit-pattern int)
assert pickle.loads(pickle.dumps(3.14)) == 3.14; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_json_marshal_pickle_silent {sum(_ledger)} asserts")

"""Surface contract for third-party orjson package.

# type-regime: monomorphic

Probes: orjson.dumps, orjson.loads, orjson.JSONDecodeError,
orjson.JSONEncodeError, orjson.OPT_INDENT_2.
CPython 3.12 is the oracle.
"""

import orjson  # type: ignore[import]

# Core API
assert hasattr(orjson, "dumps"), "dumps"
assert hasattr(orjson, "loads"), "loads"
assert hasattr(orjson, "JSONDecodeError"), "JSONDecodeError"
assert hasattr(orjson, "JSONEncodeError"), "JSONEncodeError"
assert hasattr(orjson, "OPT_INDENT_2"), "OPT_INDENT_2"
assert hasattr(orjson, "OPT_SORT_KEYS"), "OPT_SORT_KEYS"
assert hasattr(orjson, "OPT_NAIVE_UTC"), "OPT_NAIVE_UTC"

# Callables
assert callable(orjson.dumps), "dumps callable"
assert callable(orjson.loads), "loads callable"

# dumps returns bytes (not str — key difference from json)
_b = orjson.dumps({"key": "value"})
assert isinstance(_b, bytes), f"dumps type = {type(_b)!r}"

# loads accepts bytes/str/memoryview
_d = orjson.loads(b'{"key": "value"}')
assert isinstance(_d, dict), f"loads type = {type(_d)!r}"
assert _d["key"] == "value", f"loads = {_d!r}"

# Round-trip
_data = {"name": "Alice", "score": 42, "active": True}
_packed = orjson.dumps(_data)
_back = orjson.loads(_packed)
assert _back == _data, f"round-trip = {_back!r}"

# JSONDecodeError is exception
assert issubclass(orjson.JSONDecodeError, (ValueError, Exception)), \
    "JSONDecodeError < Exception"
assert issubclass(orjson.JSONEncodeError, (TypeError, Exception)), \
    "JSONEncodeError < Exception"

# OPT constants are integers
assert isinstance(orjson.OPT_INDENT_2, int), \
    f"OPT_INDENT_2 type = {type(orjson.OPT_INDENT_2)!r}"
assert isinstance(orjson.OPT_SORT_KEYS, int), \
    f"OPT_SORT_KEYS type = {type(orjson.OPT_SORT_KEYS)!r}"

# Module attributes stable
_d_ref = orjson.dumps
assert orjson.dumps is _d_ref, "dumps stable"
_l_ref = orjson.loads
assert orjson.loads is _l_ref, "loads stable"
_jde_ref = orjson.JSONDecodeError
assert orjson.JSONDecodeError is _jde_ref, "JSONDecodeError stable"
_jee_ref = orjson.JSONEncodeError
assert orjson.JSONEncodeError is _jee_ref, "JSONEncodeError stable"

print("surface OK")

"""Behavior contract for third-party orjson package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import orjson  # type: ignore[import]

# Rule 1: dumps returns bytes (not str)
_b1 = orjson.dumps({"key": "value"})
assert isinstance(_b1, bytes), f"dumps type = {type(_b1)!r}"

# Rule 2: loads accepts bytes and returns dict
_d2 = orjson.loads(b'{"name": "Alice", "score": 42}')
assert isinstance(_d2, dict), f"loads type = {type(_d2)!r}"
assert _d2["name"] == "Alice", f"name = {_d2['name']!r}"
assert _d2["score"] == 42, f"score = {_d2['score']!r}"

# Rule 3: round-trip for nested structures
_data3 = {
    "users": [
        {"id": 1, "name": "Alice", "active": True},
        {"id": 2, "name": "Bob", "active": False},
    ]
}
_packed3 = orjson.dumps(_data3)
_back3 = orjson.loads(_packed3)
assert _back3 == _data3, f"round-trip = {_back3!r}"

# Rule 4: OPT_SORT_KEYS produces sorted keys
_data4 = {"b": 2, "a": 1, "c": 3}
_b4 = orjson.dumps(_data4, option=orjson.OPT_SORT_KEYS)
_s4 = _b4.decode("utf-8")
_pos_a = _s4.index('"a"')
_pos_b = _s4.index('"b"')
_pos_c = _s4.index('"c"')
assert _pos_a < _pos_b < _pos_c, f"sorted order: {_s4!r}"

# Rule 5: OPT_INDENT_2 produces indented output
_b5 = orjson.dumps({"key": "value"}, option=orjson.OPT_INDENT_2)
_s5 = _b5.decode("utf-8")
assert "\n" in _s5, f"indented = {_s5!r}"

# Rule 6: JSONDecodeError raised for invalid JSON
_raised6 = False
try:
    orjson.loads(b"not-valid-json")
except orjson.JSONDecodeError:
    _raised6 = True
assert _raised6, "JSONDecodeError on invalid JSON"

# Rule 7: Module attributes are identity-stable
_d_ref = orjson.dumps
_l_ref = orjson.loads
_jde_ref = orjson.JSONDecodeError
_jee_ref = orjson.JSONEncodeError
for _ in range(5):
    assert orjson.dumps is _d_ref, "dumps stable"
    assert orjson.loads is _l_ref, "loads stable"
    assert orjson.JSONDecodeError is _jde_ref, "JSONDecodeError stable"
    assert orjson.JSONEncodeError is _jee_ref, "JSONEncodeError stable"

print("behavior OK")

"""Behavior contract for third-party attrs package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import attrs  # type: ignore[import]

# Rule 1: @define creates a class with __init__, __repr__, __eq__
@attrs.define
class _Point1:
    x: int
    y: int

_p1 = _Point1(3, 4)
assert _p1.x == 3, f"x = {_p1.x!r}"
assert _p1.y == 4, f"y = {_p1.y!r}"
assert repr(_p1) == "_Point1(x=3, y=4)", f"repr = {repr(_p1)!r}"
assert _p1 == _Point1(3, 4), "equality by value"
assert _p1 != _Point1(3, 5), "inequality by value"

# Rule 2: @attrs.frozen creates immutable class
@attrs.frozen
class _Vec2:
    x: float
    y: float

_v2 = _Vec2(1.0, 2.0)
assert _v2.x == 1.0, f"x = {_v2.x!r}"
_raised2 = False
try:
    _v2.x = 9.0  # type: ignore[misc]
except (attrs.exceptions.FrozenInstanceError, AttributeError):
    _raised2 = True
assert _raised2, "frozen raises on setattr"

# Rule 3: fields() returns Attribute objects with name and type
@attrs.define
class _Item3:
    name: str
    count: int = 0

_flds3 = attrs.fields(_Item3)
assert isinstance(_flds3, tuple), f"fields type = {type(_flds3)!r}"
assert len(_flds3) == 2, f"field count = {len(_flds3)}"
_names3 = [f.name for f in _flds3]
assert _names3 == ["name", "count"], f"field names = {_names3!r}"

# Rule 4: asdict() converts instance to plain dict
@attrs.define
class _Coord4:
    lat: float
    lon: float

_c4 = _Coord4(51.5, -0.1)
_d4 = attrs.asdict(_c4)
assert isinstance(_d4, dict), f"asdict type = {type(_d4)!r}"
assert _d4 == {"lat": 51.5, "lon": -0.1}, f"asdict = {_d4!r}"

# Rule 5: astuple() converts instance to plain tuple
_t5 = attrs.astuple(_c4)
assert isinstance(_t5, tuple), f"astuple type = {type(_t5)!r}"
assert _t5 == (51.5, -0.1), f"astuple = {_t5!r}"

# Rule 6: field() with default value
@attrs.define
class _Config6:
    host: str = attrs.field(default="localhost")
    port: int = attrs.field(default=8080)

_cfg6 = _Config6()
assert _cfg6.host == "localhost", f"default host = {_cfg6.host!r}"
assert _cfg6.port == 8080, f"default port = {_cfg6.port!r}"
_cfg6b = _Config6(host="example.com", port=443)
assert _cfg6b.host == "example.com", "override host"
assert _cfg6b.port == 443, "override port"

# Rule 7: Factory for mutable defaults avoids shared-state bug
@attrs.define
class _Container7:
    items: list = attrs.Factory(list)

_a7 = _Container7()
_b7 = _Container7()
_a7.items.append(1)
assert _a7.items == [1], f"a items = {_a7.items!r}"
assert _b7.items == [], f"b items independent = {_b7.items!r}"

# Rule 8: has() detects attrs class
@attrs.define
class _Tagged8:
    value: str

assert attrs.has(_Tagged8) is True, "has returns True for attrs class"
assert attrs.has(int) is False, "has returns False for non-attrs class"
assert attrs.has(str) is False, "has returns False for str"

print("behavior OK")

"""Surface contract for third-party attrs package.

# type-regime: monomorphic

Probes: attrs.define, attrs.field, attrs.asdict, attrs.fields,
attrs.Factory, attrs.validators, attrs.converters, attrs.exceptions.
CPython 3.12 is the oracle.
"""

import attrs

# Core API
assert hasattr(attrs, "define"), "define"
assert hasattr(attrs, "mutable"), "mutable"
assert hasattr(attrs, "frozen"), "frozen"
assert hasattr(attrs, "field"), "field"
assert hasattr(attrs, "fields"), "fields"
assert hasattr(attrs, "asdict"), "asdict"
assert hasattr(attrs, "astuple"), "astuple"
assert hasattr(attrs, "has"), "has"
assert hasattr(attrs, "Factory"), "Factory"
assert hasattr(attrs, "validators"), "validators"
assert hasattr(attrs, "converters"), "converters"
assert hasattr(attrs, "exceptions"), "exceptions"

# Version
assert hasattr(attrs, "__version__"), "__version__"
assert isinstance(attrs.__version__, str), \
    f"version type = {type(attrs.__version__)!r}"

# Callables
assert callable(attrs.define), "define callable"
assert callable(attrs.field), "field callable"
assert callable(attrs.fields), "fields callable"
assert callable(attrs.asdict), "asdict callable"
assert callable(attrs.astuple), "astuple callable"
assert callable(attrs.has), "has callable"

# define creates a class
@attrs.define
class _Point:
    x: int
    y: int

assert hasattr(_Point, "__attrs_attrs__"), "attrs class has __attrs_attrs__"
assert callable(_Point), "Point is callable"
_p = _Point(1, 2)
assert _p.x == 1, f"x = {_p.x!r}"
assert _p.y == 2, f"y = {_p.y!r}"

# fields() returns tuple of Attribute objects
_flds = attrs.fields(_Point)
assert isinstance(_flds, tuple), f"fields type = {type(_flds)!r}"
assert len(_flds) == 2, f"field count = {len(_flds)}"
_names = [f.name for f in _flds]
assert "x" in _names, "x in fields"
assert "y" in _names, "y in fields"

# asdict() converts to dict
_d = attrs.asdict(_p)
assert isinstance(_d, dict), f"asdict type = {type(_d)!r}"
assert _d["x"] == 1, f"asdict x = {_d['x']!r}"
assert _d["y"] == 2, f"asdict y = {_d['y']!r}"

# Module attributes stable
_define_ref = attrs.define
assert attrs.define is _define_ref, "define stable"
_field_ref = attrs.field
assert attrs.field is _field_ref, "field stable"
_asdict_ref = attrs.asdict
assert attrs.asdict is _asdict_ref, "asdict stable"
_fields_ref = attrs.fields
assert attrs.fields is _fields_ref, "fields stable"

print("surface OK")

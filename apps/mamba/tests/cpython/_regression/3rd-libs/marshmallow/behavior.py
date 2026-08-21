"""Behavior contract for third-party marshmallow package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import marshmallow  # type: ignore[import]
import marshmallow.fields as _fields  # type: ignore[import]
import marshmallow.validate as _validate  # type: ignore[import]

# Rule 1: Schema.load deserializes data
class _ItemSchema1(marshmallow.Schema):  # type: ignore[misc]
    name = _fields.String(required=True)
    price = _fields.Float()

_s1 = _ItemSchema1()
_r1 = _s1.load({"name": "widget", "price": 9.99})
assert _r1["name"] == "widget", f"name = {_r1['name']!r}"
assert abs(_r1["price"] - 9.99) < 0.001, f"price = {_r1['price']!r}"

# Rule 2: Schema.dump serializes data
_r2 = _s1.dump({"name": "gadget", "price": 4.50})
assert _r2["name"] == "gadget", f"dump name = {_r2['name']!r}"

# Rule 3: required=True raises ValidationError when missing
class _RequiredSchema3(marshmallow.Schema):  # type: ignore[misc]
    email = _fields.Email(required=True)

_s3 = _RequiredSchema3()
_raised3 = False
try:
    _s3.load({})
except marshmallow.ValidationError as _e3:
    _raised3 = True
    assert "email" in _e3.messages, f"messages = {_e3.messages!r}"
assert _raised3, "ValidationError for missing required"

# Rule 4: validate.Length constrains string length
_vl4 = _validate.Length(min=2, max=10)
_raised4 = False
try:
    _vl4("x")  # too short
except marshmallow.ValidationError:
    _raised4 = True
assert _raised4, "Length raises for too short"
_vl4("hello")  # valid

# Rule 5: validate.Range constrains numbers
_vr5 = _validate.Range(min=0, max=100)
_raised5 = False
try:
    _vr5(-1)
except marshmallow.ValidationError:
    _raised5 = True
assert _raised5, "Range raises for out-of-range"

# Rule 6: EXCLUDE/INCLUDE/RAISE are distinct constants
assert marshmallow.EXCLUDE != marshmallow.INCLUDE, "EXCLUDE != INCLUDE"
assert marshmallow.INCLUDE != marshmallow.RAISE, "INCLUDE != RAISE"
assert marshmallow.EXCLUDE != marshmallow.RAISE, "EXCLUDE != RAISE"

# Rule 7: Module attributes are identity-stable
_schema_ref = marshmallow.Schema
_fields_ref = marshmallow.fields
_validate_ref = marshmallow.validate
_ve_ref = marshmallow.ValidationError
for _ in range(5):
    assert marshmallow.Schema is _schema_ref, "Schema stable"
    assert marshmallow.fields is _fields_ref, "fields stable"
    assert marshmallow.validate is _validate_ref, "validate stable"
    assert marshmallow.ValidationError is _ve_ref, "ValidationError stable"

print("behavior OK")

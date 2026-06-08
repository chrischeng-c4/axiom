"""Behavior contract for third-party pydantic package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import pydantic  # type: ignore[import]
from pydantic import BaseModel, Field, ValidationError  # type: ignore[import]

# Rule 1: BaseModel enforces types and sets defaults
class _Item(BaseModel):
    name: str
    price: float
    quantity: int = 1

_i1 = _Item(name="widget", price=9.99)
assert _i1.name == "widget", f"name = {_i1.name!r}"
assert abs(_i1.price - 9.99) < 1e-9, f"price = {_i1.price!r}"
assert _i1.quantity == 1, f"default quantity = {_i1.quantity!r}"

# Rule 2: model_dump returns plain dict
_d2 = _i1.model_dump()
assert isinstance(_d2, dict), f"model_dump type = {type(_d2)!r}"
assert _d2 == {"name": "widget", "price": 9.99, "quantity": 1}, f"dump = {_d2!r}"

# Rule 3: model_validate parses from dict
_i3 = _Item.model_validate({"name": "gadget", "price": 19.99, "quantity": 5})
assert _i3.name == "gadget", f"validated name = {_i3.name!r}"
assert _i3.quantity == 5, f"validated quantity = {_i3.quantity!r}"

# Rule 4: model_validate_json parses from JSON string
import json
_json4 = json.dumps({"name": "thing", "price": 1.0, "quantity": 3})
_i4 = _Item.model_validate_json(_json4)
assert _i4.name == "thing", f"json name = {_i4.name!r}"
assert _i4.quantity == 3, f"json quantity = {_i4.quantity!r}"

# Rule 5: ValidationError on missing required field
_raised5 = False
try:
    _Item(price=9.99)  # missing 'name'
except ValidationError as _e:
    _raised5 = True
    assert _e.error_count() >= 1, f"error count = {_e.error_count()!r}"
assert _raised5, "ValidationError on missing field"

# Rule 6: ValidationError on wrong type
_raised6 = False
try:
    _Item(name=42, price="not-a-float")  # type: ignore[arg-type]
except ValidationError as _e:
    _raised6 = True
    _errors = _e.errors()
    assert isinstance(_errors, list), f"errors type = {type(_errors)!r}"
assert _raised6, "ValidationError on wrong types"

# Rule 7: Field with default_factory and constraints
class _Config(BaseModel):
    tags: list = Field(default_factory=list)
    max_size: int = Field(default=100, gt=0)

_cfg7 = _Config()
assert _cfg7.tags == [], f"default_factory = {_cfg7.tags!r}"
assert _cfg7.max_size == 100, f"default max_size = {_cfg7.max_size!r}"
_cfg7b = _Config(tags=["a", "b"], max_size=50)
assert _cfg7b.tags == ["a", "b"], f"tags = {_cfg7b.tags!r}"

# Rule 8: TypeAdapter validates standalone types
_ta8 = pydantic.TypeAdapter(list[int])
_vals8 = _ta8.validate_python([1, 2, 3])
assert _vals8 == [1, 2, 3], f"TypeAdapter list[int] = {_vals8!r}"
_raised8 = False
try:
    _ta8.validate_python(["a", "b"])
except ValidationError:
    _raised8 = True
assert _raised8, "TypeAdapter raises on bad type"

print("behavior OK")

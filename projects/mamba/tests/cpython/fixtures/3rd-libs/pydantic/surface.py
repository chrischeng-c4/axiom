"""Surface contract for third-party pydantic package.

# type-regime: monomorphic

Probes: pydantic.BaseModel, pydantic.Field, pydantic.ValidationError,
pydantic.field_validator, pydantic.model_validator, pydantic.ConfigDict,
pydantic.TypeAdapter, model_dump, model_validate.
CPython 3.12 is the oracle.
"""

import pydantic
from pydantic import BaseModel, Field, ValidationError

# Core names
assert hasattr(pydantic, "BaseModel"), "BaseModel"
assert hasattr(pydantic, "Field"), "Field"
assert hasattr(pydantic, "ValidationError"), "ValidationError"
assert hasattr(pydantic, "field_validator"), "field_validator"
assert hasattr(pydantic, "model_validator"), "model_validator"
assert hasattr(pydantic, "ConfigDict"), "ConfigDict"
assert hasattr(pydantic, "TypeAdapter"), "TypeAdapter"
assert hasattr(pydantic, "validator"), "validator"
assert hasattr(pydantic, "root_validator"), "root_validator"

# ValidationError is an exception
assert issubclass(pydantic.ValidationError, Exception), "ValidationError < Exception"

# Define a simple model
class _Person(BaseModel):
    name: str
    age: int = 0
    email: str = ""

# Instantiation
_p = _Person(name="Alice", age=30, email="alice@example.com")
assert isinstance(_p, BaseModel), f"instance is BaseModel: {type(_p)!r}"
assert _p.name == "Alice", f"name = {_p.name!r}"
assert _p.age == 30, f"age = {_p.age!r}"

# model_dump returns dict
_d = _p.model_dump()
assert isinstance(_d, dict), f"model_dump type = {type(_d)!r}"
assert _d["name"] == "Alice", f"dict name = {_d['name']!r}"
assert _d["age"] == 30, f"dict age = {_d['age']!r}"

# model_validate from dict
_p2 = _Person.model_validate({"name": "Bob", "age": 25})
assert _p2.name == "Bob", f"model_validate name = {_p2.name!r}"

# ValidationError on bad types
_raised = False
try:
    _Person(name=123, age="not-int")  # type: ignore[arg-type]
except ValidationError:
    _raised = True
assert _raised, "ValidationError on bad types"

# model_json_schema exists
assert hasattr(_Person, "model_json_schema"), "model_json_schema"
_schema = _Person.model_json_schema()
assert isinstance(_schema, dict), f"schema type = {type(_schema)!r}"
assert "properties" in _schema or "title" in _schema, "schema has properties or title"

# TypeAdapter
_ta = pydantic.TypeAdapter(list[int])
assert hasattr(_ta, "validate_python"), "TypeAdapter.validate_python"
_vals = _ta.validate_python([1, 2, 3])
assert _vals == [1, 2, 3], f"TypeAdapter = {_vals!r}"

# Module attributes stable
_bm_ref = pydantic.BaseModel
assert pydantic.BaseModel is _bm_ref, "BaseModel stable"
_ve_ref = pydantic.ValidationError
assert pydantic.ValidationError is _ve_ref, "ValidationError stable"

print("surface OK")

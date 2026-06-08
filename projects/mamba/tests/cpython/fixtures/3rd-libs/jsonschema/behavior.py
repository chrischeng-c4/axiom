"""Behavior contract for third-party jsonschema package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import jsonschema  # type: ignore[import]

# Rule 1: validate passes for valid data
jsonschema.validate(instance=42, schema={"type": "integer"})
jsonschema.validate(instance="hello", schema={"type": "string"})
jsonschema.validate(instance=[1, 2], schema={"type": "array"})
jsonschema.validate(instance={"a": 1}, schema={"type": "object"})

# Rule 2: validate raises ValidationError for invalid data
_raised2 = False
try:
    jsonschema.validate(instance="not-int", schema={"type": "integer"})
except jsonschema.ValidationError:
    _raised2 = True
assert _raised2, "ValidationError on wrong type"

# Rule 3: required property validation
_schema3 = {
    "type": "object",
    "required": ["name", "age"],
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer"},
    }
}
jsonschema.validate(instance={"name": "Alice", "age": 30}, schema=_schema3)
_raised3 = False
try:
    jsonschema.validate(instance={"name": "Alice"}, schema=_schema3)
except jsonschema.ValidationError:
    _raised3 = True
assert _raised3, "ValidationError for missing required"

# Rule 4: Draft7Validator.is_valid
_v4 = jsonschema.Draft7Validator({
    "type": "array",
    "items": {"type": "integer"},
    "minItems": 1,
})
assert _v4.is_valid([1, 2, 3]) is True, "valid array"
assert _v4.is_valid([]) is False, "empty array invalid"
assert _v4.is_valid([1, "a"]) is False, "mixed array invalid"

# Rule 5: iter_errors yields errors
_v5 = jsonschema.Draft7Validator({"type": "integer", "minimum": 0})
_errs5 = list(_v5.iter_errors(-1))
assert len(_errs5) >= 1, f"error count = {len(_errs5)}"
assert all(isinstance(e, jsonschema.ValidationError) for e in _errs5), \
    "all errors are ValidationError"

# Rule 6: ValidationError message is a string
_raised6 = False
try:
    jsonschema.validate(42, {"type": "string"})
except jsonschema.ValidationError as _e6:
    _raised6 = True
    assert isinstance(_e6.message, str), f"message type = {type(_e6.message)!r}"
    assert len(_e6.message) > 0, "message non-empty"
assert _raised6, "ValidationError raised"

# Rule 7: Module attributes are identity-stable
_v_ref = jsonschema.validate
_d7_ref = jsonschema.Draft7Validator
_se_ref = jsonschema.SchemaError
_ve_ref = jsonschema.ValidationError
for _ in range(5):
    assert jsonschema.validate is _v_ref, "validate stable"
    assert jsonschema.Draft7Validator is _d7_ref, "Draft7Validator stable"
    assert jsonschema.SchemaError is _se_ref, "SchemaError stable"
    assert jsonschema.ValidationError is _ve_ref, "ValidationError stable"

print("behavior OK")

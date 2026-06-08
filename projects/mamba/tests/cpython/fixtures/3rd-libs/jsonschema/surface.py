"""Surface contract for third-party jsonschema package.

# type-regime: monomorphic

Probes: jsonschema.validate, jsonschema.Draft7Validator,
jsonschema.SchemaError, jsonschema.ValidationError.
CPython 3.12 is the oracle.
"""

import jsonschema  # type: ignore[import]

# Core API
assert hasattr(jsonschema, "validate"), "validate"
assert hasattr(jsonschema, "Draft7Validator"), "Draft7Validator"
assert hasattr(jsonschema, "Draft4Validator"), "Draft4Validator"
assert hasattr(jsonschema, "SchemaError"), "SchemaError"
assert hasattr(jsonschema, "ValidationError"), "ValidationError"
assert hasattr(jsonschema, "FormatChecker"), "FormatChecker"

# Callables
assert callable(jsonschema.validate), "validate callable"
assert callable(jsonschema.Draft7Validator), "Draft7Validator callable"
assert callable(jsonschema.FormatChecker), "FormatChecker callable"

# ValidationError is an exception
assert issubclass(jsonschema.ValidationError, Exception), \
    "ValidationError < Exception"
assert issubclass(jsonschema.SchemaError, Exception), \
    "SchemaError < Exception"

# validate accepts schema and instance
jsonschema.validate(instance={"name": "Alice"}, schema={"type": "object"})

# Draft7Validator construction
_v = jsonschema.Draft7Validator({"type": "integer"})
assert hasattr(_v, "validate"), "validator.validate"
assert hasattr(_v, "is_valid"), "validator.is_valid"
assert hasattr(_v, "iter_errors"), "validator.iter_errors"

# is_valid returns bool
assert _v.is_valid(42) is True, "42 is valid int"
assert _v.is_valid("hello") is False, "'hello' is not valid int"

# ValidationError has expected attrs
_raised = False
try:
    jsonschema.validate(instance="not-int",
                        schema={"type": "integer"})
except jsonschema.ValidationError as _e:
    _raised = True
    assert hasattr(_e, "message"), "e.message"
    assert hasattr(_e, "path"), "e.path"
    assert hasattr(_e, "schema_path"), "e.schema_path"
    assert hasattr(_e, "validator"), "e.validator"
assert _raised, "ValidationError raised"

# Module attributes stable
_v_ref = jsonschema.validate
assert jsonschema.validate is _v_ref, "validate stable"
_d7_ref = jsonschema.Draft7Validator
assert jsonschema.Draft7Validator is _d7_ref, "Draft7Validator stable"
_se_ref = jsonschema.SchemaError
assert jsonschema.SchemaError is _se_ref, "SchemaError stable"
_ve_ref = jsonschema.ValidationError
assert jsonschema.ValidationError is _ve_ref, "ValidationError stable"

print("surface OK")

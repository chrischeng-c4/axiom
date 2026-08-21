"""Surface contract for third-party pydantic_core package.

# type-regime: monomorphic

Probes: pydantic_core.ValidationError, pydantic_core.SchemaValidator,
pydantic_core.SchemaSerializer, pydantic_core.Url, pydantic_core.core_schema.
CPython 3.12 is the oracle.
"""

import pydantic_core
from pydantic_core import SchemaValidator, core_schema

# Core classes
assert hasattr(pydantic_core, "ValidationError"), "ValidationError"
assert hasattr(pydantic_core, "SchemaValidator"), "SchemaValidator"
assert hasattr(pydantic_core, "SchemaSerializer"), "SchemaSerializer"
assert hasattr(pydantic_core, "Url"), "Url"
assert hasattr(pydantic_core, "core_schema"), "core_schema"
assert hasattr(pydantic_core, "PydanticCustomError"), "PydanticCustomError"
assert hasattr(pydantic_core, "PydanticUndefined"), "PydanticUndefined"
assert hasattr(pydantic_core, "from_json"), "from_json"
assert hasattr(pydantic_core, "to_json"), "to_json"

# Version
assert hasattr(pydantic_core, "__version__"), "__version__"
assert isinstance(pydantic_core.__version__, str), \
    f"version type = {type(pydantic_core.__version__)!r}"

# ValidationError is an exception
assert issubclass(pydantic_core.ValidationError, Exception), \
    "ValidationError < Exception"

# SchemaValidator usage
_sv_str = SchemaValidator(core_schema.str_schema())
assert hasattr(_sv_str, "validate_python"), "sv.validate_python"
assert hasattr(_sv_str, "validate_json"), "sv.validate_json"
_result = _sv_str.validate_python("hello")
assert _result == "hello", f"str validate = {_result!r}"

_sv_int = SchemaValidator(core_schema.int_schema())
assert _sv_int.validate_python(42) == 42, "int validate"

# ValidationError on wrong type
_raised = False
try:
    _sv_int.validate_python("not an int")
except pydantic_core.ValidationError:
    _raised = True
assert _raised, "ValidationError on bad type"

# Url parsing
_url = pydantic_core.Url("https://api.example.com/v1?key=val")
assert hasattr(_url, "scheme"), "Url.scheme"
assert hasattr(_url, "host"), "Url.host"
assert hasattr(_url, "path"), "Url.path"
assert _url.scheme == "https", f"scheme = {_url.scheme!r}"
assert _url.host == "api.example.com", f"host = {_url.host!r}"

# Module attributes stable
_ve_ref = pydantic_core.ValidationError
assert pydantic_core.ValidationError is _ve_ref, "ValidationError stable"
_sv_ref = pydantic_core.SchemaValidator
assert pydantic_core.SchemaValidator is _sv_ref, "SchemaValidator stable"
_url_ref = pydantic_core.Url
assert pydantic_core.Url is _url_ref, "Url stable"

print("surface OK")

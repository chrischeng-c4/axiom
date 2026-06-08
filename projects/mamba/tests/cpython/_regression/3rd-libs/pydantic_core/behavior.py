"""Behavior contract for third-party pydantic_core package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import pydantic_core  # type: ignore[import]
from pydantic_core import SchemaValidator, core_schema  # type: ignore[import]

# Rule 1: SchemaValidator validates str and rejects non-str
_sv1 = SchemaValidator(core_schema.str_schema())
assert _sv1.validate_python("hello") == "hello", "str validates"
assert _sv1.validate_python("") == "", "empty str validates"
_raised1 = False
try:
    _sv1.validate_python(42)
except pydantic_core.ValidationError:
    _raised1 = True
assert _raised1, "int rejected by str schema"

# Rule 2: SchemaValidator validates int and coerces compatible types
_sv2 = SchemaValidator(core_schema.int_schema())
assert _sv2.validate_python(42) == 42, "int validates"
assert _sv2.validate_python(-7) == -7, "negative int"
_raised2 = False
try:
    _sv2.validate_python("not a number")
except pydantic_core.ValidationError:
    _raised2 = True
assert _raised2, "str rejected by int schema"

# Rule 3: ValidationError has error_count() and errors() list
_sv3 = SchemaValidator(core_schema.typed_dict_schema({
    "name": core_schema.typed_dict_field(core_schema.str_schema()),
    "age": core_schema.typed_dict_field(core_schema.int_schema()),
}))
_raised3 = False
try:
    _sv3.validate_python({"name": 123, "age": "bad"})
except pydantic_core.ValidationError as _e3:
    _raised3 = True
    assert _e3.error_count() >= 1, f"error count: {_e3.error_count()!r}"
    _errs3 = _e3.errors()
    assert isinstance(_errs3, list), f"errors type = {type(_errs3)!r}"
    assert len(_errs3) >= 1, "at least one error"
assert _raised3, "ValidationError on typed dict"

# Rule 4: from_json / to_json round-trip
_data4 = {"name": "Alice", "score": 100}
_json4 = pydantic_core.to_json(_data4)
assert isinstance(_json4, bytes), f"to_json type = {type(_json4)!r}"
_back4 = pydantic_core.from_json(_json4)
assert _back4 == _data4, f"json round-trip = {_back4!r}"

# Rule 5: Url parses scheme/host/path/query correctly
_url5 = pydantic_core.Url("https://user:pass@api.example.com:8080/v1?q=test#frag")
assert _url5.scheme == "https", f"scheme = {_url5.scheme!r}"
assert _url5.host == "api.example.com", f"host = {_url5.host!r}"
assert _url5.port == 8080, f"port = {_url5.port!r}"
assert _url5.path == "/v1", f"path = {_url5.path!r}"
assert _url5.username == "user", f"username = {_url5.username!r}"

# Rule 6: SchemaValidator.validate_json parses JSON bytes
_sv6 = SchemaValidator(core_schema.int_schema())
_result6 = _sv6.validate_json(b"42")
assert _result6 == 42, f"validate_json = {_result6!r}"

# Rule 7: Module attributes are identity-stable
_ve_ref = pydantic_core.ValidationError
_sv_ref = pydantic_core.SchemaValidator
_url_ref = pydantic_core.Url
for _ in range(5):
    assert pydantic_core.ValidationError is _ve_ref, "ValidationError stable"
    assert pydantic_core.SchemaValidator is _sv_ref, "SchemaValidator stable"
    assert pydantic_core.Url is _url_ref, "Url stable"

print("behavior OK")

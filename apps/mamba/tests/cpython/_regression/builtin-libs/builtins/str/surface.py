"""Surface contract for builtins.str.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, key string methods present.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "str"), "builtins.str missing"
assert builtins.str is str, "builtins.str is str divergence"
assert callable(builtins.str), "builtins.str not callable"

# str is a class (type)
assert type(builtins.str).__name__ == "type", \
    f"type(builtins.str).__name__ = {type(builtins.str).__name__!r}"
assert issubclass(builtins.str, object), "str is not a subclass of object"

assert builtins.str.__name__ == "str", \
    f"builtins.str.__name__ = {builtins.str.__name__!r}"

# str instances
assert isinstance("", str), "isinstance('', str) failed"
assert isinstance("hello", str), "isinstance('hello', str) failed"

# Key string methods present
for _meth in ("upper", "lower", "strip", "lstrip", "rstrip", "split",
              "join", "replace", "find", "startswith", "endswith",
              "encode", "format", "count", "index", "isdigit",
              "isalpha", "isalnum", "isspace", "title", "center",
              "zfill", "ljust", "rjust"):
    assert hasattr(str, _meth), f"str.{_meth} missing"
    assert callable(getattr(str, _meth)), f"str.{_meth} not callable"

# str.__doc__ exists
assert isinstance(builtins.str.__doc__, str) and len(builtins.str.__doc__) > 0, \
    "builtins.str.__doc__ missing or empty"

print("surface OK")

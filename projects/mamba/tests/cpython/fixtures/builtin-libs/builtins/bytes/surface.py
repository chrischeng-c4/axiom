"""Surface contract for builtins.bytes.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, key bytes methods present.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "bytes"), "builtins.bytes missing"
assert builtins.bytes is bytes, "builtins.bytes is bytes divergence"
assert callable(builtins.bytes), "builtins.bytes not callable"

# bytes is a class (type)
assert type(builtins.bytes).__name__ == "type", \
    f"type(builtins.bytes).__name__ = {type(builtins.bytes).__name__!r}"
assert issubclass(builtins.bytes, object), "bytes is not a subclass of object"

assert builtins.bytes.__name__ == "bytes", \
    f"builtins.bytes.__name__ = {builtins.bytes.__name__!r}"

# bytes instances
assert isinstance(b"", bytes), "isinstance(b'', bytes) failed"
assert isinstance(b"hello", bytes), "isinstance(b'hello', bytes) failed"
assert isinstance(bytes(3), bytes), "isinstance(bytes(3), bytes) failed"

# Key bytes methods present
for _meth in ("decode", "hex", "count", "find", "index", "replace",
              "split", "startswith", "endswith", "strip", "upper", "lower",
              "fromhex", "join"):
    assert hasattr(bytes, _meth), f"bytes.{_meth} missing"
    assert callable(getattr(bytes, _meth)), f"bytes.{_meth} not callable"

# bytes.__doc__ exists
assert isinstance(builtins.bytes.__doc__, str) and len(builtins.bytes.__doc__) > 0, \
    "builtins.bytes.__doc__ missing or empty"

print("surface OK")

"""Surface contract for builtins.bytearray.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, mutable contrast to bytes,
key methods, fromhex/hex, decode.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "bytearray"), "builtins.bytearray missing"
assert builtins.bytearray is bytearray, "builtins.bytearray is bytearray divergence"
assert callable(builtins.bytearray), "builtins.bytearray not callable"

# bytearray is a class (type)
assert type(builtins.bytearray).__name__ == "type", \
    f"type(builtins.bytearray).__name__ = {type(builtins.bytearray).__name__!r}"
assert issubclass(bytearray, object), "bytearray not subclass of object"
assert builtins.bytearray.__name__ == "bytearray", \
    f"bytearray.__name__ = {builtins.bytearray.__name__!r}"

# bytearray instances
assert isinstance(bytearray(), bytearray), "isinstance(bytearray(), bytearray) failed"
assert isinstance(bytearray(3), bytearray), "isinstance(bytearray(3), bytearray) failed"
assert isinstance(bytearray(b"hi"), bytearray), "isinstance(bytearray(b'hi'), bytearray) failed"

# bytearray is mutable — unlike bytes
ba = bytearray(b"hello")
ba[0] = 72  # 'H'
assert ba[0] == 72, f"bytearray mutation failed: {ba[0]!r}"

# Key methods present
for _meth in ("append", "extend", "insert", "pop", "remove", "reverse",
              "decode", "hex", "fromhex", "find", "replace", "split"):
    assert hasattr(bytearray, _meth), f"bytearray.{_meth} missing"

# bytearray.__doc__ exists
assert isinstance(builtins.bytearray.__doc__, str) and len(builtins.bytearray.__doc__) > 0, \
    "builtins.bytearray.__doc__ missing"

print("surface OK")

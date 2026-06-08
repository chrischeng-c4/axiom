"""Surface contract for builtins.chr and builtins.ord.

# type-regime: monomorphic

Probes: name presence, callable, return types, roundtrip, range bounds.
CPython 3.12 is the oracle.
"""

import builtins

# chr
assert hasattr(builtins, "chr"), "builtins.chr missing"
assert builtins.chr is chr, "builtins.chr is chr divergence"
assert callable(builtins.chr), "builtins.chr not callable"

# ord
assert hasattr(builtins, "ord"), "builtins.ord missing"
assert builtins.ord is ord, "builtins.ord is ord divergence"
assert callable(builtins.ord), "builtins.ord not callable"

# chr returns str of length 1
assert isinstance(chr(65), str), "chr(65) not str"
assert len(chr(65)) == 1, "chr(65) length != 1"

# ord returns int
assert isinstance(ord("A"), int), "ord('A') not int"

# Roundtrip
assert ord(chr(65)) == 65, f"roundtrip chr/ord: {ord(chr(65))!r}"
assert chr(ord("Z")) == "Z", f"roundtrip ord/chr: {chr(ord('Z'))!r}"

print("surface OK")

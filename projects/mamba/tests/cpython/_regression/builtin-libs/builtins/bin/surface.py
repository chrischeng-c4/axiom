"""Surface contract for builtins.bin, hex, and oct.

# type-regime: monomorphic

Probes: name presence, callable, return str with correct prefix.
CPython 3.12 is the oracle.
"""

import builtins

# bin
assert hasattr(builtins, "bin"), "builtins.bin missing"
assert builtins.bin is bin, "builtins.bin is bin divergence"
assert callable(builtins.bin), "builtins.bin not callable"

# hex
assert hasattr(builtins, "hex"), "builtins.hex missing"
assert builtins.hex is hex, "builtins.hex is hex divergence"
assert callable(builtins.hex), "builtins.hex not callable"

# oct
assert hasattr(builtins, "oct"), "builtins.oct missing"
assert builtins.oct is oct, "builtins.oct is oct divergence"
assert callable(builtins.oct), "builtins.oct not callable"

# All return str
assert isinstance(bin(10), str), "bin(10) not str"
assert isinstance(hex(255), str), "hex(255) not str"
assert isinstance(oct(8), str), "oct(8) not str"

# Correct prefixes
assert bin(10).startswith("0b"), f"bin(10) = {bin(10)!r}"
assert hex(255).startswith("0x"), f"hex(255) = {hex(255)!r}"
assert oct(8).startswith("0o"), f"oct(8) = {oct(8)!r}"

print("surface OK")

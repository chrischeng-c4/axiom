"""Surface contract for language bitwise operators.

# type-regime: monomorphic

Probes: &, |, ^, ~, <<, >> operators on int, precedence,
chained bitwise ops, interaction with bool.
CPython 3.12 is the oracle.
"""

# AND operator
assert (0b1010 & 0b1100) == 0b1000, f"AND = {(0b1010 & 0b1100)!r}"

# OR operator
assert (0b1010 | 0b1100) == 0b1110, f"OR = {(0b1010 | 0b1100)!r}"

# XOR operator
assert (0b1010 ^ 0b1100) == 0b0110, f"XOR = {(0b1010 ^ 0b1100)!r}"

# NOT operator (bitwise complement)
assert (~0) == -1, f"~0 = {(~0)!r}"
assert (~5) == -6, f"~5 = {(~5)!r}"
assert (~(-1)) == 0, f"~(-1) = {(~(-1))!r}"

# Left shift
assert (1 << 3) == 8, f"1<<3 = {(1 << 3)!r}"
assert (3 << 2) == 12, f"3<<2 = {(3 << 2)!r}"

# Right shift
assert (8 >> 2) == 2, f"8>>2 = {(8 >> 2)!r}"
assert (7 >> 1) == 3, f"7>>1 = {(7 >> 1)!r}"

# Shift by zero
assert (5 << 0) == 5, f"5<<0 = {(5 << 0)!r}"
assert (5 >> 0) == 5, f"5>>0 = {(5 >> 0)!r}"

# Precedence: ~ binds tightest, then <</>>, then &, then ^, then |
assert (~0 & 0xFF) == 0xFF, f"~0 & 0xFF = {(~0 & 0xFF)!r}"
assert (1 | 2 ^ 4) == 7, f"1 | 2 ^ 4 = {(1 | 2 ^ 4)!r}"

# Bitwise on large ints (within 64-bit)
assert (0xDEAD << 16 | 0xBEEF) == 0xDEADBEEF, f"compose = {(0xDEAD << 16 | 0xBEEF)!r}"

# In-place operators return correct types
_x = 0b1111
_x &= 0b0101
assert _x == 0b0101, f"&= result = {_x!r}"
assert isinstance(_x, int), f"&= type = {type(_x)!r}"

_y = 0b0101
_y |= 0b1010
assert _y == 0b1111, f"|= result = {_y!r}"

_z = 0b1111
_z ^= 0b1010
assert _z == 0b0101, f"^= result = {_z!r}"

_s = 1
_s <<= 4
assert _s == 16, f"<<= result = {_s!r}"

_r = 64
_r >>= 3
assert _r == 8, f">>= result = {_r!r}"

print("surface OK")

"""Behavior contract for language bitwise operators.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: AND clears bits not in mask
assert (0xFF & 0x0F) == 0x0F, f"AND mask = {(0xFF & 0x0F)!r}"
assert (0xAA & 0x55) == 0x00, f"AND complement = {(0xAA & 0x55)!r}"

# Rule 2: OR sets bits from either operand
assert (0xA0 | 0x0B) == 0xAB, f"OR combine = {(0xA0 | 0x0B)!r}"
assert (0xFF | 0x00) == 0xFF, f"OR identity = {(0xFF | 0x00)!r}"

# Rule 3: XOR toggles bits
assert (0xFF ^ 0xFF) == 0x00, f"XOR self = {(0xFF ^ 0xFF)!r}"
assert (0xAA ^ 0x55) == 0xFF, f"XOR complement = {(0xAA ^ 0x55)!r}"
assert (0xAA ^ 0x55 ^ 0x55) == 0xAA, f"XOR round-trip = {(0xAA ^ 0x55 ^ 0x55)!r}"

# Rule 4: NOT is two's complement (-(x+1))
for v in (0, 1, -1, 100, -100):
    assert ~v == -(v + 1), f"~{v} = {~v!r}, expected {-(v+1)!r}"

# Rule 5: Left shift multiplies by powers of two
for shift in range(8):
    assert (1 << shift) == 2 ** shift, f"1 << {shift} = {1 << shift!r}"

# Rule 6: Right shift floor-divides by powers of two (arithmetic for negatives)
assert (100 >> 2) == 25, f"100>>2 = {(100 >> 2)!r}"
assert (-1 >> 1) == -1, f"-1>>1 = {(-1 >> 1)!r}"
assert (-4 >> 1) == -2, f"-4>>1 = {(-4 >> 1)!r}"

# Rule 7: Bitwise ops work on bool (treated as int)
assert (True & False) == 0, f"True & False = {(True & False)!r}"
assert (True | False) == 1, f"True | False = {(True | False)!r}"
assert (True ^ True) == 0, f"True ^ True = {(True ^ True)!r}"
assert isinstance(True & False, int), f"bool & bool type = {type(True & False)!r}"

# Rule 8: Bit manipulation idioms — test a bit
_val = 0b10110
assert (_val >> 2) & 1 == 1, "bit 2 should be set"
assert (_val >> 3) & 1 == 0, "bit 3 should be clear"

# Rule 9: Set and clear bit idioms
_x = 0b0000
_x |= (1 << 3)   # set bit 3
assert _x == 0b1000, f"set bit = {_x!r}"
_x &= ~(1 << 3)  # clear bit 3
assert _x == 0, f"clear bit = {_x!r}"

# Rule 10: Swap via XOR
_a = 42
_b = 17
_a ^= _b
_b ^= _a
_a ^= _b
assert _a == 17, f"swap a = {_a!r}"
assert _b == 42, f"swap b = {_b!r}"

# Rule 11: Popcount via bin() and count
_n = 0b10110111
assert bin(_n).count("1") == 6, f"popcount = {bin(_n).count('1')!r}"

print("behavior OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_bitwise_ops"
# subject = "cpython321.lang_bitwise_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_bitwise_ops.py"
# status = "filled"
# ///
"""cpython321.lang_bitwise_ops: execute CPython 3.12 seed lang_bitwise_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for bitwise operators on integers.
# Surface: AND (&), OR (|), XOR (^), NOT (~), left shift (<<), right
# shift (>>); operator precedence (& binds tighter than |); shifts
# work on negative integers using arithmetic-shift semantics; common
# bit-pattern manipulations — masking, setting, toggling — compose
# from these primitives.
_ledger: list[int] = []

# Bitwise AND — only bits set in BOTH operands remain
assert (0xFF & 0x0F) == 0x0F; _ledger.append(1)
assert (0xAA & 0x55) == 0; _ledger.append(1)
assert (0xFF & 0xFF) == 0xFF; _ledger.append(1)
# AND with zero is zero
assert (0xFF & 0) == 0; _ledger.append(1)

# Bitwise OR — any bit set in either operand
assert (0x0F | 0xF0) == 0xFF; _ledger.append(1)
assert (0xAA | 0x55) == 0xFF; _ledger.append(1)
assert (0 | 0) == 0; _ledger.append(1)
# OR with zero leaves the value unchanged
assert (0xAB | 0) == 0xAB; _ledger.append(1)

# Bitwise XOR — bits differ between operands
assert (0xFF ^ 0x0F) == 0xF0; _ledger.append(1)
assert (0xAA ^ 0xAA) == 0; _ledger.append(1)
assert (0 ^ 0xFF) == 0xFF; _ledger.append(1)
# Double-XOR with the same key is identity
assert ((0xAB ^ 0xCD) ^ 0xCD) == 0xAB; _ledger.append(1)

# Bitwise NOT — flips every bit; ~x == -x - 1
assert ~0 == -1; _ledger.append(1)
assert ~-1 == 0; _ledger.append(1)
assert ~5 == -6; _ledger.append(1)
assert ~~5 == 5; _ledger.append(1)

# Left shift — multiplication by 2**n
assert (1 << 0) == 1; _ledger.append(1)
assert (1 << 1) == 2; _ledger.append(1)
assert (1 << 4) == 16; _ledger.append(1)
assert (1 << 8) == 256; _ledger.append(1)
assert (3 << 2) == 12; _ledger.append(1)

# Right shift — floor division by 2**n
assert (8 >> 0) == 8; _ledger.append(1)
assert (8 >> 1) == 4; _ledger.append(1)
assert (8 >> 3) == 1; _ledger.append(1)
assert (256 >> 4) == 16; _ledger.append(1)
assert (0xFF >> 4) == 0xF; _ledger.append(1)

# Right shift on negatives uses arithmetic shift (sign-preserving)
assert (-8 >> 1) == -4; _ledger.append(1)
assert (-1 >> 1) == -1; _ledger.append(1)

# Common bit-twiddling combinations
# Build the all-ones mask for the low 8 bits via (1 << 8) - 1
assert ((1 << 8) - 1) == 255; _ledger.append(1)
# Set two distinct bits via OR
assert ((1 << 4) | (1 << 2)) == 20; _ledger.append(1)
# Clear a bit by ANDing with its complement
assert (0xFF & ~0x0F) == 0xF0; _ledger.append(1)
# Mask the low nibble after a high-nibble overwrite
assert (((0xFF & 0xF0) | 0x0A)) == 0xFA; _ledger.append(1)

# Operator precedence — & binds tighter than | which binds tighter
# than +/- (these parens make the assertion read literally)
assert ((3 + 4) & 1) == 1; _ledger.append(1)
assert (3 | (4 & 1)) == 3; _ledger.append(1)

# Bit-check pattern: (x & (1 << k)) != 0  means bit k is set
def _bit_set(x, k):
    return (x & (1 << k)) != 0

assert _bit_set(0xFF, 0) == True; _ledger.append(1)
assert _bit_set(0xFF, 7) == True; _ledger.append(1)
assert _bit_set(0xFF, 8) == False; _ledger.append(1)
assert _bit_set(0xF0, 3) == False; _ledger.append(1)
assert _bit_set(0xF0, 4) == True; _ledger.append(1)

# Toggle pattern: x ^ (1 << k) flips bit k
assert (0x00 ^ (1 << 3)) == 0x08; _ledger.append(1)
assert (0xFF ^ (1 << 3)) == 0xF7; _ledger.append(1)

# Round-trip: ((x >> n) << n) clears the low n bits
assert ((0xFF >> 4) << 4) == 0xF0; _ledger.append(1)
assert ((0xAB >> 4) << 4) == 0xA0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_bitwise_ops {sum(_ledger)} asserts")

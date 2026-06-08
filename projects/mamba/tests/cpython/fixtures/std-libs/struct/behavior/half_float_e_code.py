# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "half_float_e_code"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'e' is 16-bit IEEE 754: exact LE/BE bit patterns for representable values, smallest normal/subnormal round-trip, NaN packs with exponent bits set, values past the half range raise OverflowError, and nearest-even rounding at a tie"""
import math
import struct

# 'e' is two bytes wide.
assert struct.calcsize("e") == 2, "e is 16-bit"

# Exact little-endian / big-endian bit patterns for representable values.
cases = [
    (b"\x00<", 1.0),
    (b"\x00\xc0", -2.0),
    (b"\xff{", 65504.0),       # largest finite half
    (b"\x00\x00", 0.0),
    (b"\x00\x80", -0.0),
    (b"\x00|", float("inf")),
    (b"\x00\xfc", float("-inf")),
]
for le_bits, value in cases:
    be_bits = le_bits[::-1]
    assert struct.unpack("<e", le_bits)[0] == value, f"<e unpack {value!r}"
    assert struct.pack("<e", value) == le_bits, f"<e pack {value!r}"
    assert struct.unpack(">e", be_bits)[0] == value, f">e unpack {value!r}"
    assert struct.pack(">e", value) == be_bits, f">e pack {value!r}"

# Smallest normal and smallest subnormal round-trip.
assert struct.unpack("<e", b"\x00\x04")[0] == 2.0 ** -14, "smallest normal"
assert struct.unpack("<e", b"\x01\x00")[0] == 2.0 ** -24, "smallest subnormal"

# NaN bit patterns unpack to NaN, and packing a NaN sets the exponent/quiet bits.
assert math.isnan(struct.unpack("<e", b"\x00~")[0]), "NaN unpacks to nan"
packed_nan = struct.pack("<e", math.nan)
assert packed_nan[1] & 0x7e == 0x7e, "packed NaN has exponent bits set"

# Values too large for a half float overflow rather than round to inf.
for value in (65520.0, 65536.0, 1e300, -65536.0, -1e300):
    try:
        struct.pack(">e", value)
        raise AssertionError(f"expected OverflowError for {value!r}")
    except OverflowError:
        pass

# Rounding to nearest-even when a value falls between representable halves.
assert struct.pack(">e", 2.0 ** -25) == b"\x00\x00", "tie rounds down to 0"
assert struct.pack(">e", 2.0 ** -25 + 2.0 ** -35) == b"\x00\x01", "above tie rounds up"

print("half_float_e_code OK")

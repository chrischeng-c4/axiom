# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "crc32_exact_values_and_seed_chaining"
# subject = "zlib.crc32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.crc32: crc32 returns the IEEE 802.3 unsigned 32-bit value, seeds with 0, passes a seed through for empty input, matches documented values for explicit seeds, chains incrementally to equal the one-shot over the concatenation, and stays unsigned for a large seed"""
import zlib

# Exact values (unsigned) and known small inputs.
assert zlib.crc32(b"abcdefghijklmnop") == 2486878355, "crc32 abc..p"
assert zlib.crc32(b"spam") == 1138425661, "crc32 spam"
assert zlib.crc32(b"hello") == 907060870, "crc32 hello"
assert zlib.crc32(b"\x00") == 3523407757, "crc32 nul"

# Start value is the identity/seed: crc32 seeds with 0.
assert zlib.crc32(b"") == 0, "crc32 empty default = 0"
assert zlib.crc32(b"") == zlib.crc32(b"", 0), "crc32 default seed is 0"
# Empty input returns the supplied seed unchanged.
assert zlib.crc32(b"", 432) == 432, "crc32 empty passes seed through"

# Explicit seed produces documented values.
assert zlib.crc32(b"penguin", 0) == 3854672160, "crc32 penguin seed 0"
assert zlib.crc32(b"penguin", 1) == 1136044692, "crc32 penguin seed 1"

# Seed chaining equals one-shot over the concatenation.
_part = zlib.crc32(b"hel")
assert zlib.crc32(b"lo", _part) == zlib.crc32(b"hello"), "crc32 incremental"

# Large seed (0xFFFFFFFF) is accepted and result stays unsigned 32-bit.
assert 0 <= zlib.crc32(b"abc", 4294967295) <= 0xFFFFFFFF, "crc32 big seed unsigned"

print("crc32_exact_values_and_seed_chaining OK")

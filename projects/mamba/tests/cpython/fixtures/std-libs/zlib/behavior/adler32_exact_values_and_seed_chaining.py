# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "adler32_exact_values_and_seed_chaining"
# subject = "zlib.adler32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.adler32: adler32 returns the mod-65521 unsigned 32-bit value, seeds with 1, passes a seed through for empty input, matches documented values for explicit seeds, chains incrementally to equal the one-shot over the concatenation, and stays unsigned for a large seed"""
import zlib

# Exact values (unsigned) and known small inputs.
assert zlib.adler32(b"abcdefghijklmnop" * 2) == 3573550353, "adler32 abc..p x2"
assert zlib.adler32(b"spam") == 72286642, "adler32 spam"
assert zlib.adler32(b"hello") == 103547413, "adler32 hello"
assert zlib.adler32(b"\x01") == 131074, "adler32 soh"

# Start value is the identity/seed: adler32 seeds with 1.
assert zlib.adler32(b"") == 1, "adler32 empty default = 1"
assert zlib.adler32(b"") == zlib.adler32(b"", 1), "adler32 default seed is 1"
# Empty input returns the supplied seed unchanged.
assert zlib.adler32(b"", 432) == 432, "adler32 empty passes seed through"

# Explicit seed produces documented values.
assert zlib.adler32(b"penguin", 0) == 198116086, "adler32 penguin seed 0"
assert zlib.adler32(b"penguin", 1) == 198574839, "adler32 penguin seed 1"

# Seed chaining equals one-shot over the concatenation.
_part = zlib.adler32(b"hel")
assert zlib.adler32(b"lo", _part) == zlib.adler32(b"hello"), "adler32 incremental"

# Large seed (0xFFFFFFFF) is accepted and result stays unsigned 32-bit.
assert 0 <= zlib.adler32(b"abc", 4294967295) <= 0xFFFFFFFF, "adler32 big seed unsigned"

print("adler32_exact_values_and_seed_chaining OK")

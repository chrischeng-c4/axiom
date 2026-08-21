# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "crc32_known_and_incremental"
# subject = "binascii.crc32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.crc32: crc32 known values plus incremental seeding equals one-shot"""
import binascii

_crc = binascii.crc32(b"test data")
assert isinstance(_crc, int), f"crc32 type = {type(_crc)!r}"
assert binascii.crc32(b"") == 0, "crc32(empty) = 0"
assert binascii.crc32(b"hello") == 907060870, "crc32(hello)"
_crc1 = binascii.crc32(b"hel")
assert binascii.crc32(b"lo", _crc1) == binascii.crc32(b"hello"), "incremental crc32"

print("crc32_known_and_incremental OK")

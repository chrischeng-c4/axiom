# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "crc_hqx_known_and_mask"
# subject = "binascii.crc_hqx"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.crc_hqx: crc_hqx incremental value, and empty-data returns seed masked to 16 bits"""
import binascii

_q = binascii.crc_hqx(b"Test the CRC-32 of", 0)
assert isinstance(_q, int), f"crc_hqx type = {type(_q)!r}"
_q = binascii.crc_hqx(b" this string.", _q)
assert _q == 14290, f"crc_hqx incremental = {_q}"
# Empty data returns the seed masked to 16 bits.
assert binascii.crc_hqx(b"", 0) == 0, "crc_hqx empty seed 0"
assert binascii.crc_hqx(b"", -1) == 0xffff, "crc_hqx masks seed to 16 bits"
assert binascii.crc_hqx(b"", 0x12345678) == 0x5678, "crc_hqx 16-bit mask"

print("crc_hqx_known_and_mask OK")

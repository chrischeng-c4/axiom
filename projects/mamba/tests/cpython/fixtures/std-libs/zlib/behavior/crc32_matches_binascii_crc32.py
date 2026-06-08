# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "crc32_matches_binascii_crc32"
# subject = "zlib.crc32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.crc32: zlib.crc32 and binascii.crc32 produce identical checksums over the same input"""
import binascii
import zlib

assert binascii.crc32(b"abcdefghijklmnop") == zlib.crc32(b"abcdefghijklmnop"), "binascii parity abc..p"
assert binascii.crc32(b"spam") == zlib.crc32(b"spam"), "binascii parity spam"
assert binascii.crc32(b"") == zlib.crc32(b""), "binascii parity empty"

print("crc32_matches_binascii_crc32 OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "pascal_string_p_code"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'Np' is a Pascal string (length byte then data): '2p' b'abc' -> b'\\x01a', '4p' b'abc' -> b'\\x03abc', the stored length caps at 255, and unpack drops the length byte"""
import struct

# 'p' is a Pascal string: the first byte is the length, then the data.
assert struct.pack("2p", b"abc") == b"\x01a", "2p: len byte + 1 char"
assert struct.pack("4p", b"abc") == b"\x03abc", "4p: len byte + 3 chars"
assert struct.unpack("4p", b"\x03abc")[0] == b"abc", "p unpack drops len byte"
# A 'p' field longer than 256 caps the stored length at 255.
big = struct.pack("1000p", b"x" * 1000)
assert big[0] == 255, "p length byte caps at 255"
assert struct.unpack("1000p", big)[0] == b"x" * 255, "p unpack truncates to 255"

print("pascal_string_p_code OK")

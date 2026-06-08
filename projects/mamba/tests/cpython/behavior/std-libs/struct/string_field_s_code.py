# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "string_field_s_code"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'Ns' is a fixed N-byte field: short input zero-pads ('3s' b'ab' -> b'ab\\x00'), long input truncates ('3s' b'abcd' -> b'abc'), '0s' is empty, and unpack returns the raw N bytes"""
import struct

# 'Ns' is a fixed-width byte field: short input zero-pads, long input truncates.
assert struct.pack(">3s", b"ab") == b"ab\x00", "short 3s zero-pads"
assert struct.pack(">3s", b"abcd") == b"abc", "long 3s truncates"
# '0s' is an empty field.
assert struct.pack(">0s", b"hi") == b"", "0s is empty"
# A 5-byte field round-trips its bytes exactly.
_sv = struct.pack("5s", b"hello")
assert _sv == b"hello", f"5s pack = {_sv!r}"
assert struct.unpack("5s", _sv) == (b"hello",), "5s unpack"
# unpack returns the raw N bytes.
assert struct.unpack(">3s", b"xyz") == (b"xyz",), "3s unpack"

print("string_field_s_code OK")

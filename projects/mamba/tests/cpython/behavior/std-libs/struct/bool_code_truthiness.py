# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "bool_code_truthiness"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: '?' packs any object by truthiness to 0x00/0x01 (0/[] -> 0, 5/[1] -> 1) and unpacks any non-zero byte to True, only b'\\x00' to False"""
import struct

# '?' packs any truthy/falsy value to a single 0x00/0x01 byte.
assert struct.pack(">?", 0) == b"\x00", "falsy -> 0"
assert struct.pack(">?", 5) == b"\x01", "truthy int -> 1"
assert struct.pack(">?", []) == b"\x00", "empty list is falsy"
assert struct.pack(">?", [1]) == b"\x01", "non-empty list is truthy"
# Any non-zero byte unpacks back to True; only 0x00 is False.
for raw in (b"\x01", b"\x7f", b"\xff", b"\xf0"):
    assert struct.unpack(">?", raw)[0] is True, f"nonzero byte -> True ({raw!r})"
assert struct.unpack(">?", b"\x00")[0] is False, "zero byte -> False"

print("bool_code_truthiness OK")

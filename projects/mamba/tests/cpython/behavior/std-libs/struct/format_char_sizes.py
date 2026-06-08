# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "format_char_sizes"
# subject = "struct.calcsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: calcsize maps each standard code to its fixed width: b=1, h=2, i=4, q=8, e=2, f=4, d=8; width is independent of the byte-order prefix ('>I' and '<I' both 4)"""
import struct

# Each standard code has a fixed width.
for code, width in [("b", 1), ("h", 2), ("i", 4), ("q", 8),
                    ("e", 2), ("f", 4), ("d", 8)]:
    assert struct.calcsize(code) == width, f"calcsize({code!r}) = {struct.calcsize(code)!r}"

# Width is independent of the byte-order prefix.
assert struct.calcsize(">I") == struct.calcsize("<I") == 4, "endianness does not change width"

print("format_char_sizes OK")

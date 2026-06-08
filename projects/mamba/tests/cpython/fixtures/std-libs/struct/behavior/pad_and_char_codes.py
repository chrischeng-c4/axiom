# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "pad_and_char_codes"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'c' packs one literal byte and 'x' is a zero pad byte that consumes no argument: '>xc' b'a' -> b'\\x00a', '>cx' b'a' -> b'a\\x00'"""
import struct

# 'c' packs a single literal byte.
assert struct.pack(">c", b"a") == b"a", "c code"
# 'x' is a pad byte: it contributes a zero and consumes no argument.
assert struct.pack(">xc", b"a") == b"\x00a", "x pad then c"
assert struct.pack(">cx", b"a") == b"a\x00", "c then x pad"

print("pad_and_char_codes OK")

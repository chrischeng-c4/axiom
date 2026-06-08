# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "charmap_decode_string_map"
# subject = "codecs.charmap_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode with a string map indexes byte n -> map[n]: b'\\x00\\x01\\x02' over 'abc' yields ('abc', 3), including an astral target via '\\U0010ffffbc'"""
import codecs

# String map: byte n -> map[n].
assert codecs.charmap_decode(b"\x00\x01\x02", "strict", "abc") == ("abc", 3)
# String map may target astral code points.
assert codecs.charmap_decode(
    b"\x00\x01\x02", "strict", "\U0010ffffbc"
) == ("\U0010ffffbc", 3)

print("charmap_decode_string_map OK")

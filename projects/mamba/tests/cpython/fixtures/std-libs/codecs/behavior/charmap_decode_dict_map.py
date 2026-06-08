# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "charmap_decode_dict_map"
# subject = "codecs.charmap_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode with an int->int dict map looks up each byte value: {0:ord('a'),1:ord('b'),2:ord('c')} yields ('abc',3) and a value of sys.maxunicode reaches the max code point"""
import codecs

import sys
_a, _b, _c = ord("a"), ord("b"), ord("c")
# int->int dict map.
assert codecs.charmap_decode(
    b"\x00\x01\x02", "strict", {0: _a, 1: _b, 2: _c}
) == ("abc", 3)
# Dict map may reach the maximum Unicode code point.
assert codecs.charmap_decode(
    b"\x00\x01\x02", "strict", {0: sys.maxunicode, 1: _b, 2: _c}
) == (chr(sys.maxunicode) + "bc", 3)

print("charmap_decode_dict_map OK")

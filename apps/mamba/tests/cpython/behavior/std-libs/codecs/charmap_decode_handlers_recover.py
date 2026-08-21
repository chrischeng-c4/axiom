# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "charmap_decode_handlers_recover"
# subject = "codecs.charmap_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: on a missing map slot the error handlers recover: 'replace' yields the U+FFFD char, 'ignore' drops it, 'backslashreplace' emits the \\xHH escape"""
import codecs

# 'ab' has no slot for byte 0x02; handlers recover differently.
_short = "ab"
assert codecs.charmap_decode(b"\x00\x01\x02", "replace", _short) == ("ab�", 3)
assert codecs.charmap_decode(b"\x00\x01\x02", "ignore", _short) == ("ab", 3)
assert codecs.charmap_decode(
    b"\x00\x01\x02", "backslashreplace", _short
) == ("ab\\x02", 3)

print("charmap_decode_handlers_recover OK")

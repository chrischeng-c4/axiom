# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf16_decode_handlers_on_lone_byte"
# subject = "codecs.utf_16_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.utf_16_decode: the low-level utf_16_decode applies its handler to a lone trailing byte: 'replace' yields the U+FFFD char with consumed 1, 'ignore' yields '' with consumed 1"""
import codecs

assert codecs.utf_16_decode(b"\x01", "replace", True) == ("�", 1)
assert codecs.utf_16_decode(b"\x01", "ignore", True) == ("", 1)

print("utf16_decode_handlers_on_lone_byte OK")

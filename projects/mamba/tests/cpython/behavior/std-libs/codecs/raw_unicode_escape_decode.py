# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "raw_unicode_escape_decode"
# subject = "codecs.raw_unicode_escape_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.raw_unicode_escape_decode: raw_unicode_escape_decode keeps \\xHH literal but decodes \\u escapes: b'\\\\xff' -> ('\\\\xff',4) and b'\\\\u00e9' -> ('é',6)"""
import codecs

# raw_unicode_escape keeps \xHH literal but decodes \u escapes.
assert codecs.raw_unicode_escape_decode(b"\\xff") == ("\\xff", 4)
assert codecs.raw_unicode_escape_decode(b"\\u00e9") == ("é", 6)

print("raw_unicode_escape_decode OK")

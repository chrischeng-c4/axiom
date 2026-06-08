# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "unicode_escape_decode_handlers_recover"
# subject = "codecs.unicode_escape_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.unicode_escape_decode: unicode_escape_decode handlers recover from truncated \\x/\\u/\\U and from a code point past U+10FFFF; unicode_escape_encode leaves printable ASCII as-is"""
import codecs

_udecode = codecs.unicode_escape_decode
# Truncated \x \u \U: handlers recover and report consumed length.
for _c in (b"x", b"u", b"U"):
    _data = b"[\\" + _c + b"0]\\" + _c + b"0"
    assert _udecode(_data, "ignore") == ("[]", len(_data))
    assert _udecode(_data, "replace") == ("[�]�", len(_data))
# A code point past U+10FFFF is an error; handlers recover.
assert _udecode(b"\\U00110000", "ignore") == ("", 10)
assert _udecode(b"\\U00110000", "replace") == ("�", 10)
# unicode_escape_encode leaves printable ASCII as-is.
assert codecs.unicode_escape_encode("A") == (b"A", 1)

print("unicode_escape_decode_handlers_recover OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "escape_decode_handlers_recover"
# subject = "codecs.escape_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.escape_decode: escape_decode error handlers recover from a truncated \\x and report consumed length: 'ignore' on b'[\\x]\\x' is (b'[]',6), 'replace' is (b'[?]?',6); a plain byte passes through (b'A0',2)"""
import codecs

_decode = codecs.escape_decode
# Error handlers recover from a truncated \x and report bytes consumed.
assert _decode(b"[\\x]\\x", "ignore") == (b"[]", 6)
assert _decode(b"[\\x]\\x", "replace") == (b"[?]?", 6)
assert _decode(b"[\\x0]\\x0", "ignore") == (b"[]", 8)
assert _decode(b"[\\x0]\\x0", "replace") == (b"[?]?", 8)
# A plain byte that is not a backslash passes through verbatim.
assert _decode(b"A0") == (b"A0", 2)

print("escape_decode_handlers_recover OK")

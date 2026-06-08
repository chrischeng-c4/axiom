# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "charmap_decode_short_string_map_raises"
# subject = "codecs.charmap_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode_short_string_map_raises (errors)."""
import codecs

_raised = False
try:
    codecs.charmap_decode(b'\x00\x01\x02', 'strict', 'ab')
except UnicodeDecodeError:
    _raised = True
assert _raised, "charmap_decode_short_string_map_raises: expected UnicodeDecodeError"
print("charmap_decode_short_string_map_raises OK")

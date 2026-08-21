# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "charmap_decode_dict_value_overflow_raises"
# subject = "codecs.charmap_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode_dict_value_overflow_raises (errors)."""
import codecs

_raised = False
try:
    codecs.charmap_decode(b'\x00', 'strict', {0: __import__('sys').maxunicode + 1})
except TypeError:
    _raised = True
assert _raised, "charmap_decode_dict_value_overflow_raises: expected TypeError"
print("charmap_decode_dict_value_overflow_raises OK")

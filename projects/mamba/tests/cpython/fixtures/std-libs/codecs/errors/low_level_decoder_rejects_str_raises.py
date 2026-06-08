# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "low_level_decoder_rejects_str_raises"
# subject = "codecs.utf_8_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.utf_8_decode: low_level_decoder_rejects_str_raises (errors)."""
import codecs

_raised = False
try:
    codecs.utf_8_decode('xxx')
except TypeError:
    _raised = True
assert _raised, "low_level_decoder_rejects_str_raises: expected TypeError"
print("low_level_decoder_rejects_str_raises OK")

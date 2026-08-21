# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "a85decode_out_of_range_char_raises"
# subject = "base64.a85decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.a85decode: a85decode_out_of_range_char_raises (errors)."""
import base64

_raised = False
try:
    base64.a85decode(b'!!!!y')
except ValueError:
    _raised = True
assert _raised, "a85decode_out_of_range_char_raises: expected ValueError"
print("a85decode_out_of_range_char_raises OK")

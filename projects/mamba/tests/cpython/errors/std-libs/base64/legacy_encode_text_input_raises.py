# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "legacy_encode_text_input_raises"
# subject = "base64.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.encode: legacy_encode_text_input_raises (errors)."""
import base64
from io import BytesIO, StringIO

_raised = False
try:
    base64.encode(StringIO('YWJj\n'), BytesIO())
except TypeError:
    _raised = True
assert _raised, "legacy_encode_text_input_raises: expected TypeError"
print("legacy_encode_text_input_raises OK")

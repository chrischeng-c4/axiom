# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "b2a_base64_str_input_raises"
# subject = "binascii.b2a_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_base64: b2a_base64_str_input_raises (errors)."""
import binascii

_raised = False
try:
    binascii.b2a_base64('text')
except TypeError:
    _raised = True
assert _raised, "b2a_base64_str_input_raises: expected TypeError"
print("b2a_base64_str_input_raises OK")

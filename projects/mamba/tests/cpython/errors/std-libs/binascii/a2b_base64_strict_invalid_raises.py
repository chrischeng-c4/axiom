# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_base64_strict_invalid_raises"
# subject = "binascii.a2b_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64_strict_invalid_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_base64(b'not_b64!@#$', strict_mode=True)
except binascii.Error:
    _raised = True
assert _raised, "a2b_base64_strict_invalid_raises: expected binascii.Error"
print("a2b_base64_strict_invalid_raises OK")

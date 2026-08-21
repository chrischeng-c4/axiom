# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "unhexlify_odd_length_raises"
# subject = "binascii.unhexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.unhexlify: unhexlify_odd_length_raises (errors)."""
import binascii

_raised = False
try:
    binascii.unhexlify(b'abc')
except binascii.Error:
    _raised = True
assert _raised, "unhexlify_odd_length_raises: expected binascii.Error"
print("unhexlify_odd_length_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "b2a_uu_over_45_bytes_raises"
# subject = "binascii.b2a_uu"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: b2a_uu_over_45_bytes_raises (errors)."""
import binascii

_raised = False
try:
    binascii.b2a_uu(b'!' * 46)
except binascii.Error:
    _raised = True
assert _raised, "b2a_uu_over_45_bytes_raises: expected binascii.Error"
print("b2a_uu_over_45_bytes_raises OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_hex_non_hex_char_raises"
# subject = "binascii.a2b_hex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_hex: a2b_hex_non_hex_char_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_hex(b'0G')
except binascii.Error:
    _raised = True
assert _raised, "a2b_hex_non_hex_char_raises: expected binascii.Error"
print("a2b_hex_non_hex_char_raises OK")

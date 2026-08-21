# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "a2b_base64_non_ascii_str_raises"
# subject = "binascii.a2b_base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64_non_ascii_str_raises (errors)."""
import binascii

_raised = False
try:
    binascii.a2b_base64('\x80')
except ValueError:
    _raised = True
assert _raised, "a2b_base64_non_ascii_str_raises: expected ValueError"
print("a2b_base64_non_ascii_str_raises OK")

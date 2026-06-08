# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "hexlify_no_arg_raises"
# subject = "binascii.hexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.hexlify: hexlify_no_arg_raises (errors)."""
import binascii

_raised = False
try:
    binascii.hexlify()
except TypeError:
    _raised = True
assert _raised, "hexlify_no_arg_raises: expected TypeError"
print("hexlify_no_arg_raises OK")

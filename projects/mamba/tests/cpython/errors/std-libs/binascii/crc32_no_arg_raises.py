# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "errors"
# case = "crc32_no_arg_raises"
# subject = "binascii.crc32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.crc32: crc32_no_arg_raises (errors)."""
import binascii

_raised = False
try:
    binascii.crc32()
except TypeError:
    _raised = True
assert _raised, "crc32_no_arg_raises: expected TypeError"
print("crc32_no_arg_raises OK")

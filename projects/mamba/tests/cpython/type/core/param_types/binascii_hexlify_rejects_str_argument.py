# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "binascii_hexlify_rejects_str_argument"
# subject = "binascii.hexlify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""binascii.hexlify: binascii_hexlify_rejects_str_argument (errors)."""
import binascii

try:
    result = binascii.hexlify("abc")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])

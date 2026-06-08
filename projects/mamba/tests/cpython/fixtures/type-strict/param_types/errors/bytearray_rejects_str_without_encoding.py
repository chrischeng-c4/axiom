# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "param_types"
# dimension = "errors"
# case = "bytearray_rejects_str_without_encoding"
# subject = "bytearray"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""bytearray: bytearray_rejects_str_without_encoding (errors)."""
try:
    result = bytearray("abc")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])

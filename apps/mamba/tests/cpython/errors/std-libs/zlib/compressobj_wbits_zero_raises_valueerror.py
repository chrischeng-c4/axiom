# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "compressobj_wbits_zero_raises_valueerror"
# subject = "zlib.compressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: compressobj_wbits_zero_raises_valueerror (errors)."""
import zlib

_raised = False
try:
    zlib.compressobj(1, zlib.DEFLATED, 0)
except ValueError:
    _raised = True
assert _raised, "compressobj_wbits_zero_raises_valueerror: expected ValueError"
print("compressobj_wbits_zero_raises_valueerror OK")

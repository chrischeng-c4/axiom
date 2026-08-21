# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "decompressobj_flush_zero_length_raises_valueerror"
# subject = "zlib.decompressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: decompressobj_flush_zero_length_raises_valueerror (errors)."""
import zlib

_raised = False
try:
    zlib.decompressobj().flush(0)
except ValueError:
    _raised = True
assert _raised, "decompressobj_flush_zero_length_raises_valueerror: expected ValueError"
print("decompressobj_flush_zero_length_raises_valueerror OK")

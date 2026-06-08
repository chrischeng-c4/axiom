# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "adler32_non_bytes_raises_typeerror"
# subject = "zlib.adler32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.adler32: adler32_non_bytes_raises_typeerror (errors)."""
import zlib

_raised = False
try:
    zlib.adler32(42)
except TypeError:
    _raised = True
assert _raised, "adler32_non_bytes_raises_typeerror: expected TypeError"
print("adler32_non_bytes_raises_typeerror OK")

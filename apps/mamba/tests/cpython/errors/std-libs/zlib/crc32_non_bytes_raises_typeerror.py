# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "crc32_non_bytes_raises_typeerror"
# subject = "zlib.crc32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.crc32: crc32_non_bytes_raises_typeerror (errors)."""
import zlib

_raised = False
try:
    zlib.crc32('not bytes')
except TypeError:
    _raised = True
assert _raised, "crc32_non_bytes_raises_typeerror: expected TypeError"
print("crc32_non_bytes_raises_typeerror OK")

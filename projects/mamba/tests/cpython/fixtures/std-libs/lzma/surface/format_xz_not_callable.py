# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "format_xz_not_callable"
# subject = "lzma.FORMAT_XZ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.FORMAT_XZ: format_xz_not_callable (surface)."""
import lzma

assert not callable(lzma.FORMAT_XZ)
print("format_xz_not_callable OK")

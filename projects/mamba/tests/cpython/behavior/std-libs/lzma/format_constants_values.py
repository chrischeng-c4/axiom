# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "format_constants_values"
# subject = "lzma.FORMAT_XZ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.FORMAT_XZ: FORMAT_AUTO/XZ/ALONE/RAW equal CPython's 0/1/2/3"""
import lzma


assert lzma.FORMAT_AUTO == 0, f"FORMAT_AUTO = {lzma.FORMAT_AUTO!r}"
assert lzma.FORMAT_XZ == 1, f"FORMAT_XZ = {lzma.FORMAT_XZ!r}"
assert lzma.FORMAT_ALONE == 2, f"FORMAT_ALONE = {lzma.FORMAT_ALONE!r}"
assert lzma.FORMAT_RAW == 3, f"FORMAT_RAW = {lzma.FORMAT_RAW!r}"
print("format_constants_values OK")

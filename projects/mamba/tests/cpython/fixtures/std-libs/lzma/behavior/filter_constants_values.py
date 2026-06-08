# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "filter_constants_values"
# subject = "lzma.FILTER_LZMA2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.FILTER_LZMA2: FILTER_LZMA2 == 33 and FILTER_DELTA == 3 match CPython"""
import lzma


assert lzma.FILTER_LZMA2 == 33, f"FILTER_LZMA2 = {lzma.FILTER_LZMA2!r}"
assert lzma.FILTER_DELTA == 3, f"FILTER_DELTA = {lzma.FILTER_DELTA!r}"
print("filter_constants_values OK")

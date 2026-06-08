# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "filter_lzma2_not_callable"
# subject = "lzma.FILTER_LZMA2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.FILTER_LZMA2: filter_lzma2_not_callable (surface)."""
import lzma

assert not callable(lzma.FILTER_LZMA2)
print("filter_lzma2_not_callable OK")

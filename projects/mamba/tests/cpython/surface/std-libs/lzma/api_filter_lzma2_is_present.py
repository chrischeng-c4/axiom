# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_filter_lzma2_is_present"
# subject = "lzma.FILTER_LZMA2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.FILTER_LZMA2: api_filter_lzma2_is_present (surface)."""
import lzma

assert hasattr(lzma, "FILTER_LZMA2")
print("api_filter_lzma2_is_present OK")

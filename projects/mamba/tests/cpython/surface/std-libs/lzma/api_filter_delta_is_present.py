# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_filter_delta_is_present"
# subject = "lzma.FILTER_DELTA"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.FILTER_DELTA: api_filter_delta_is_present (surface)."""
import lzma

assert hasattr(lzma, "FILTER_DELTA")
print("api_filter_delta_is_present OK")

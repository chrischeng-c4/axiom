# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_mode_fast_is_present"
# subject = "lzma.MODE_FAST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.MODE_FAST: api_mode_fast_is_present (surface)."""
import lzma

assert hasattr(lzma, "MODE_FAST")
print("api_mode_fast_is_present OK")

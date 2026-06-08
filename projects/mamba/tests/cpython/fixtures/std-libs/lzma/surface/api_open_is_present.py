# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_open_is_present"
# subject = "lzma.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.open: api_open_is_present (surface)."""
import lzma

assert hasattr(lzma, "open")
print("api_open_is_present OK")

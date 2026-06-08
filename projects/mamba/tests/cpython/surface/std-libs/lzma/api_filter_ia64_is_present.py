# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_filter_ia64_is_present"
# subject = "lzma.FILTER_IA64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.FILTER_IA64: api_filter_ia64_is_present (surface)."""
import lzma

assert hasattr(lzma, "FILTER_IA64")
print("api_filter_ia64_is_present OK")

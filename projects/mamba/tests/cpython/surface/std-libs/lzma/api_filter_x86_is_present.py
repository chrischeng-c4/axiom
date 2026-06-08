# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_filter_x86_is_present"
# subject = "lzma.FILTER_X86"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.FILTER_X86: api_filter_x86_is_present (surface)."""
import lzma

assert hasattr(lzma, "FILTER_X86")
print("api_filter_x86_is_present OK")

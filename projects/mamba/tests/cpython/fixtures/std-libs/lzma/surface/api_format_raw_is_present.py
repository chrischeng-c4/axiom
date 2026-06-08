# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_format_raw_is_present"
# subject = "lzma.FORMAT_RAW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.FORMAT_RAW: api_format_raw_is_present (surface)."""
import lzma

assert hasattr(lzma, "FORMAT_RAW")
print("api_format_raw_is_present OK")

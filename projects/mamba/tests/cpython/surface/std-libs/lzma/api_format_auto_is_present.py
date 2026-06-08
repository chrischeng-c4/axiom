# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_format_auto_is_present"
# subject = "lzma.FORMAT_AUTO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.FORMAT_AUTO: api_format_auto_is_present (surface)."""
import lzma

assert hasattr(lzma, "FORMAT_AUTO")
print("api_format_auto_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_is_check_supported_is_present"
# subject = "lzma.is_check_supported"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.is_check_supported: api_is_check_supported_is_present (surface)."""
import lzma

assert hasattr(lzma, "is_check_supported")
print("api_is_check_supported_is_present OK")

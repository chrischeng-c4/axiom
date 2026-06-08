# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_max_wbits_is_present"
# subject = "zlib.MAX_WBITS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.MAX_WBITS: api_max_wbits_is_present (surface)."""
import zlib

assert hasattr(zlib, "MAX_WBITS")
print("api_max_wbits_is_present OK")

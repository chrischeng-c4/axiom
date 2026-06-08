# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_deflated_is_present"
# subject = "zlib.DEFLATED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.DEFLATED: api_deflated_is_present (surface)."""
import zlib

assert hasattr(zlib, "DEFLATED")
print("api_deflated_is_present OK")

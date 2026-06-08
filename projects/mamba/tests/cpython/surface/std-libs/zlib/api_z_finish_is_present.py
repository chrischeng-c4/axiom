# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_z_finish_is_present"
# subject = "zlib.Z_FINISH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.Z_FINISH: api_z_finish_is_present (surface)."""
import zlib

assert hasattr(zlib, "Z_FINISH")
print("api_z_finish_is_present OK")

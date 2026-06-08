# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "api_def_mem_level_is_present"
# subject = "zlib.DEF_MEM_LEVEL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zlib.DEF_MEM_LEVEL: api_def_mem_level_is_present (surface)."""
import zlib

assert hasattr(zlib, "DEF_MEM_LEVEL")
print("api_def_mem_level_is_present OK")

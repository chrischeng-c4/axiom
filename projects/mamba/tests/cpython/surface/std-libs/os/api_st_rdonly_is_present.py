# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_st_rdonly_is_present"
# subject = "os.ST_RDONLY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.ST_RDONLY: api_st_rdonly_is_present (surface)."""
import os

assert hasattr(os, "ST_RDONLY")
print("api_st_rdonly_is_present OK")

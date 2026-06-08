# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_st_nosuid_is_present"
# subject = "os.ST_NOSUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.ST_NOSUID: api_st_nosuid_is_present (surface)."""
import os

assert hasattr(os, "ST_NOSUID")
print("api_st_nosuid_is_present OK")

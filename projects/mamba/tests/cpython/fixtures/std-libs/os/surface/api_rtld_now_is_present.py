# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_rtld_now_is_present"
# subject = "os.RTLD_NOW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.RTLD_NOW: api_rtld_now_is_present (surface)."""
import os

assert hasattr(os, "RTLD_NOW")
print("api_rtld_now_is_present OK")

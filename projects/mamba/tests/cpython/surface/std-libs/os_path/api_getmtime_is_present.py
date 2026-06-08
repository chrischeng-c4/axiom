# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_getmtime_is_present"
# subject = "os.path.getmtime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.getmtime: api_getmtime_is_present (surface)."""
import os.path

assert hasattr(os.path, "getmtime")
print("api_getmtime_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getabsfile_is_present"
# subject = "inspect.getabsfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getabsfile: api_getabsfile_is_present (surface)."""
import inspect

assert hasattr(inspect, "getabsfile")
print("api_getabsfile_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getfile_is_present"
# subject = "inspect.getfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getfile: api_getfile_is_present (surface)."""
import inspect

assert hasattr(inspect, "getfile")
print("api_getfile_is_present OK")

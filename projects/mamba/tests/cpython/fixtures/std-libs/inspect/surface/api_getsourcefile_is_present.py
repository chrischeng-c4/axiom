# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getsourcefile_is_present"
# subject = "inspect.getsourcefile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getsourcefile: api_getsourcefile_is_present (surface)."""
import inspect

assert hasattr(inspect, "getsourcefile")
print("api_getsourcefile_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getsource_is_present"
# subject = "inspect.getsource"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getsource: api_getsource_is_present (surface)."""
import inspect

assert hasattr(inspect, "getsource")
print("api_getsource_is_present OK")

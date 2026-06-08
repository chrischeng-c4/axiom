# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getgeneratorstate_is_present"
# subject = "inspect.getgeneratorstate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getgeneratorstate: api_getgeneratorstate_is_present (surface)."""
import inspect

assert hasattr(inspect, "getgeneratorstate")
print("api_getgeneratorstate_is_present OK")

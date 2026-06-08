# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_ismethodwrapper_is_present"
# subject = "inspect.ismethodwrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.ismethodwrapper: api_ismethodwrapper_is_present (surface)."""
import inspect

assert hasattr(inspect, "ismethodwrapper")
print("api_ismethodwrapper_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isclass_is_present"
# subject = "inspect.isclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isclass: api_isclass_is_present (surface)."""
import inspect

assert hasattr(inspect, "isclass")
print("api_isclass_is_present OK")

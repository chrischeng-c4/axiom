# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isgeneratorfunction_is_present"
# subject = "inspect.isgeneratorfunction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isgeneratorfunction: api_isgeneratorfunction_is_present (surface)."""
import inspect

assert hasattr(inspect, "isgeneratorfunction")
print("api_isgeneratorfunction_is_present OK")

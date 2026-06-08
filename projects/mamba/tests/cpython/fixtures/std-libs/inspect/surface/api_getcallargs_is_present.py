# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getcallargs_is_present"
# subject = "inspect.getcallargs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getcallargs: api_getcallargs_is_present (surface)."""
import inspect

assert hasattr(inspect, "getcallargs")
print("api_getcallargs_is_present OK")

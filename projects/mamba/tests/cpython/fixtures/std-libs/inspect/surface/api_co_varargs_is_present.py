# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_co_varargs_is_present"
# subject = "inspect.CO_VARARGS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.CO_VARARGS: api_co_varargs_is_present (surface)."""
import inspect

assert hasattr(inspect, "CO_VARARGS")
print("api_co_varargs_is_present OK")

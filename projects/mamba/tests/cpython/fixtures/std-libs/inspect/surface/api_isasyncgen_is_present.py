# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isasyncgen_is_present"
# subject = "inspect.isasyncgen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isasyncgen: api_isasyncgen_is_present (surface)."""
import inspect

assert hasattr(inspect, "isasyncgen")
print("api_isasyncgen_is_present OK")

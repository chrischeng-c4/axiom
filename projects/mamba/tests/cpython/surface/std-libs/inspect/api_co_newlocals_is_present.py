# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_co_newlocals_is_present"
# subject = "inspect.CO_NEWLOCALS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.CO_NEWLOCALS: api_co_newlocals_is_present (surface)."""
import inspect

assert hasattr(inspect, "CO_NEWLOCALS")
print("api_co_newlocals_is_present OK")

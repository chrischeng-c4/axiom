# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_co_optimized_is_present"
# subject = "inspect.CO_OPTIMIZED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.CO_OPTIMIZED: api_co_optimized_is_present (surface)."""
import inspect

assert hasattr(inspect, "CO_OPTIMIZED")
print("api_co_optimized_is_present OK")

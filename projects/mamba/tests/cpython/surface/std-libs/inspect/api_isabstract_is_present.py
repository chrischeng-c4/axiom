# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isabstract_is_present"
# subject = "inspect.isabstract"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isabstract: api_isabstract_is_present (surface)."""
import inspect

assert hasattr(inspect, "isabstract")
print("api_isabstract_is_present OK")

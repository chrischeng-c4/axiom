# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getblock_is_present"
# subject = "inspect.getblock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getblock: api_getblock_is_present (surface)."""
import inspect

assert hasattr(inspect, "getblock")
print("api_getblock_is_present OK")

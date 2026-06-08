# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_unwrap_is_present"
# subject = "inspect.unwrap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.unwrap: api_unwrap_is_present (surface)."""
import inspect

assert hasattr(inspect, "unwrap")
print("api_unwrap_is_present OK")

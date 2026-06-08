# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_iscoroutine_is_present"
# subject = "inspect.iscoroutine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.iscoroutine: api_iscoroutine_is_present (surface)."""
import inspect

assert hasattr(inspect, "iscoroutine")
print("api_iscoroutine_is_present OK")

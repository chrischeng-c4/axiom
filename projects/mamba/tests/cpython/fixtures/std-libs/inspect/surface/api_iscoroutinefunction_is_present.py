# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_iscoroutinefunction_is_present"
# subject = "inspect.iscoroutinefunction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.iscoroutinefunction: api_iscoroutinefunction_is_present (surface)."""
import inspect

assert hasattr(inspect, "iscoroutinefunction")
print("api_iscoroutinefunction_is_present OK")

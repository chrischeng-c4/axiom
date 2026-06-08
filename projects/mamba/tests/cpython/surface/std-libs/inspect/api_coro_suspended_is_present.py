# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_coro_suspended_is_present"
# subject = "inspect.CORO_SUSPENDED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.CORO_SUSPENDED: api_coro_suspended_is_present (surface)."""
import inspect

assert hasattr(inspect, "CORO_SUSPENDED")
print("api_coro_suspended_is_present OK")

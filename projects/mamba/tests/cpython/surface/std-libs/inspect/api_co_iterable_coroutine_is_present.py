# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_co_iterable_coroutine_is_present"
# subject = "inspect.CO_ITERABLE_COROUTINE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.CO_ITERABLE_COROUTINE: api_co_iterable_coroutine_is_present (surface)."""
import inspect

assert hasattr(inspect, "CO_ITERABLE_COROUTINE")
print("api_co_iterable_coroutine_is_present OK")

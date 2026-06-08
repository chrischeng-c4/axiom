# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_co_nofree_is_present"
# subject = "inspect.CO_NOFREE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.CO_NOFREE: api_co_nofree_is_present (surface)."""
import inspect

assert hasattr(inspect, "CO_NOFREE")
print("api_co_nofree_is_present OK")

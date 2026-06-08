# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getclasstree_is_present"
# subject = "inspect.getclasstree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getclasstree: api_getclasstree_is_present (surface)."""
import inspect

assert hasattr(inspect, "getclasstree")
print("api_getclasstree_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_walktree_is_present"
# subject = "inspect.walktree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.walktree: api_walktree_is_present (surface)."""
import inspect

assert hasattr(inspect, "walktree")
print("api_walktree_is_present OK")

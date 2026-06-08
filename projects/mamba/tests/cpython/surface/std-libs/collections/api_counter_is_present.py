# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_counter_is_present"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.Counter: api_counter_is_present (surface)."""
import collections

assert hasattr(collections, "Counter")
print("api_counter_is_present OK")

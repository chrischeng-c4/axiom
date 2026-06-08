# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_deque_is_present"
# subject = "collections.deque"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.deque: api_deque_is_present (surface)."""
import collections

assert hasattr(collections, "deque")
print("api_deque_is_present OK")

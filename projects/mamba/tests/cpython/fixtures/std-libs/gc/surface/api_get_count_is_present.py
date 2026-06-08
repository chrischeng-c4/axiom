# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_get_count_is_present"
# subject = "gc.get_count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.get_count: api_get_count_is_present (surface)."""
import gc

assert hasattr(gc, "get_count")
print("api_get_count_is_present OK")

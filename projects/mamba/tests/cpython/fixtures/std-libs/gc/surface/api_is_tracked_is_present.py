# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_is_tracked_is_present"
# subject = "gc.is_tracked"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.is_tracked: api_is_tracked_is_present (surface)."""
import gc

assert hasattr(gc, "is_tracked")
print("api_is_tracked_is_present OK")

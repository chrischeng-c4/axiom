# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_debug_uncollectable_is_present"
# subject = "gc.DEBUG_UNCOLLECTABLE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.DEBUG_UNCOLLECTABLE: api_debug_uncollectable_is_present (surface)."""
import gc

assert hasattr(gc, "DEBUG_UNCOLLECTABLE")
print("api_debug_uncollectable_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_debug_leak_is_present"
# subject = "gc.DEBUG_LEAK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.DEBUG_LEAK: api_debug_leak_is_present (surface)."""
import gc

assert hasattr(gc, "DEBUG_LEAK")
print("api_debug_leak_is_present OK")

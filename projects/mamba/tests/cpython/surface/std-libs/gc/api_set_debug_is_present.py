# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_set_debug_is_present"
# subject = "gc.set_debug"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.set_debug: api_set_debug_is_present (surface)."""
import gc

assert hasattr(gc, "set_debug")
print("api_set_debug_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_garbage_is_present"
# subject = "gc.garbage"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.garbage: api_garbage_is_present (surface)."""
import gc

assert hasattr(gc, "garbage")
print("api_garbage_is_present OK")

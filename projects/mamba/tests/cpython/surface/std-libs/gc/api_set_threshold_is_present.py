# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_set_threshold_is_present"
# subject = "gc.set_threshold"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.set_threshold: api_set_threshold_is_present (surface)."""
import gc

assert hasattr(gc, "set_threshold")
print("api_set_threshold_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "surface"
# case = "api_unfreeze_is_present"
# subject = "gc.unfreeze"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gc.unfreeze: api_unfreeze_is_present (surface)."""
import gc

assert hasattr(gc, "unfreeze")
print("api_unfreeze_is_present OK")
